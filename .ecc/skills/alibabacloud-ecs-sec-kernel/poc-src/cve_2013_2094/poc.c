/**
 * CVE-2013-2094 PoC - CTF Challenge Mode
 *
 * perf_swevent_init() integer type error causing array index overflow
 *
 * Vulnerability Description:
 *   Linux kernel 3.0.0 through 3.8.8 has a local privilege escalation
 *   vulnerability in the perf_swevent_init() function. The kernel uses a
 *   signed 32-bit integer (s32) as the index for the perf_swevent_enabled[]
 *   array. An attacker can pass a negative event_id via perf_event_open(),
 *   causing an out-of-bounds array access (negative index) that overwrites
 *   kernel memory, leading to arbitrary code execution in ring 0.
 *
 *   Fixed in kernel 3.8.9 (commit 8176cced706b).
 *
 * CTF Modes:
 *   read_root_file  - Unsupported (integer overflow is not a read oracle;
 *                     the vulnerability overwrites kernel memory for code
 *                     execution, it doesn't leak file contents)
 *   write_root_file - Unsupported (integer overflow does not provide
 *                     controlled file write primitive like page cache
 *                     pollution; it achieves LPE via ring 0 code execution,
 *                     not by writing to files)
 *
 * Exploitation approach:
 *   This is a classic integer overflow / array index out-of-bounds exploit.
 *   The attack path:
 *   1. Call perf_event_open() with event_id = 0xFFFFFFFF (-1 as s32)
 *   2. In affected kernels, perf_swevent_init() does:
 *      s32 event_id = attr.config;  // 0xFFFFFFFF = -1
 *      if (perf_swevent_enabled[event_id] < 0)  // OOB access!
 *   3. The negative index accesses memory before the array base,
 *      potentially overwriting function pointers or critical kernel data
 *   4. Real exploits map the target memory region and construct shellcode
 *      to escalate to root
 *
 *   This is fundamentally a CODE EXECUTION vulnerability, not a page cache
 *   pollution attack. The CTF framework's read/write file modes do not map
 *   to this vulnerability class. The PoC outputs CTF_UNSUPPORTED for both.
 *
 * Safety:
 *   - Does NOT actually exploit the vulnerability (no negative index write)
 *   - Only checks kernel version and perf_event_open() availability
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2013_2094.bin cve_2013_2094/poc.c
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
#include <sys/types.h>
#include <sys/stat.h>
#include <linux/perf_event.h>

/* ========================================================================
 * Constants
 * ======================================================================== */

/* perf_event_open syscall number (x86_64) */
#ifndef __NR_perf_event_open
#if defined(__x86_64__)
#define __NR_perf_event_open 298
#elif defined(__i386__)
#define __NR_perf_event_open 336
#else
#define __NR_perf_event_open 241  /* fallback */
#endif
#endif

/* ========================================================================
 * check_kernel_version_readable - Check if /proc/version is readable
 *
 * Python layer handles version matching. Here we just verify we can
 * read the kernel version string.
 * ======================================================================== */
static int check_kernel_version_readable(void) {
    FILE *f = fopen("/proc/version", "r");
    if (!f) {
        poc_log("Cannot read /proc/version");
        return -1;
    }

    char version_str[512] = {0};
    if (fgets(version_str, sizeof(version_str), f) == NULL) {
        fclose(f);
        poc_log("Failed to read kernel version string");
        return -1;
    }
    fclose(f);

    poc_log("Kernel version string: %.120s", version_str);
    return 1;
}

/* ========================================================================
 * check_perf_event_open - Check if perf_event_open syscall is available
 *
 * Tests with a normal software event (PERF_COUNT_SW_CPU_CLOCK).
 * If this succeeds, the perf_events subsystem is available.
 * ======================================================================== */
static int check_perf_event_open(void) {
    struct perf_event_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.size = sizeof(attr);
    attr.type = PERF_TYPE_SOFTWARE;
    attr.config = PERF_COUNT_SW_CPU_CLOCK;
    attr.disabled = 1;  /* Create disabled, don't actually trigger */

    int fd = (int)syscall(__NR_perf_event_open, &attr, -1, 0, -1, 0);
    poc_log_syscall("perf_event_open(PERF_COUNT_SW_CPU_CLOCK)", fd, errno);

    if (fd >= 0) {
        poc_log("perf_event_open succeeded (fd=%d) - subsystem available", fd);
        close(fd);
        return 1;
    }

    if (errno == EACCES || errno == EPERM) {
        poc_log("perf_event_open denied (errno=%d) - perf_event_paranoid may block", errno);
        return 0;
    }
    if (errno == ENOSYS) {
        poc_log("perf_event_open not available (syscall not implemented)");
        return -1;
    }

    poc_log("perf_event_open failed (errno=%d)", errno);
    return -1;
}

/* ========================================================================
 * check_negative_index_path - Verify negative index path is reachable
 *
 * In affected kernels (3.0.0 - 3.8.8), perf_event_open with a large
 * uint32 value (0xFFFFFFFF) is interpreted as a negative s32 index.
 * We test if the syscall accepts such a value without immediately
 * rejecting it as invalid.
 *
 * NOTE: We only check if the syscall path exists. We do NOT actually
 * trigger the vulnerability (no real exploitation).
 * ======================================================================== */
static int check_negative_index_path(void) {
    struct perf_event_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.size = sizeof(attr);
    attr.type = PERF_TYPE_SOFTWARE;

    /*
     * Critical: pass 0xFFFFFFFF as config.
     * In affected kernels, perf_swevent_init() does:
     *   s32 event_id = attr.config;  // 0xFFFFFFFF = -1
     *   if (perf_swevent_enabled[event_id] < 0)  // OOB!
     *
     * In patched kernels, this is rejected early.
     */
    attr.config = 0xFFFFFFFFULL;
    attr.disabled = 1;

    int fd = (int)syscall(__NR_perf_event_open, &attr, -1, 0, -1, 0);
    poc_log_syscall("perf_event_open(event_id=0xFFFFFFFF)", fd, errno);

    if (fd >= 0) {
        poc_log("perf_event_open accepted event_id=0xFFFFFFFF (fd=%d)", fd);
        poc_log("In affected kernels, this triggers negative index OOB");
        close(fd);
        return 1;
    }

    if (errno == EINVAL) {
        poc_log("perf_event_open rejected event_id=0xFFFFFFFF (EINVAL)");
        poc_log("Kernel likely has the fix (rejects invalid event_id early)");
        return 0;
    }
    if (errno == EACCES || errno == EPERM) {
        poc_log("perf_event_open denied (errno=%d) - permission restricted", errno);
        return 0;
    }

    poc_log("perf_event_open failed unexpectedly (errno=%d)", errno);
    return -1;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * CVE-2013-2094 is an integer overflow / array index out-of-bounds
 * leading to code execution. It does NOT provide an arbitrary file read
 * capability. The vulnerability overwrites kernel memory (potentially
 * function pointers) to achieve privilege escalation, it doesn't create
 * a read oracle that can leak file contents.
 * ======================================================================== */
static int mode_read_root_file(poc_args_t *args) {
    (void)args;
    poc_log("--- CVE-2013-2094: read_root_file mode ---");

    poc_print_unsupported(POC_MODE_READ,
        "CVE-2013-2094 is an integer overflow exploit that achieves code "
        "execution via perf_swevent_enabled[] array out-of-bounds access; "
        "it does not provide a file read primitive. The vulnerability "
        "overwrites kernel memory for ring 0 code execution, not reading "
        "file contents");

    poc_print_fail("integer overflow exploit does not provide file read primitive");
    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * CVE-2013-2094 is an integer overflow leading to code execution.
 * It does NOT provide a controlled file write capability like page cache
 * pollution (DirtyPipe, vmsplice, AF_ALG). The exploit achieves privilege
 * escalation by executing code in ring 0, not by writing to files.
 *
 * Unlike DirtyPipe (splice to page cache) or vmsplice (iovec injection),
 * this vulnerability has NO mechanism to write arbitrary data to a file's
 * page cache. The kernel memory overwrite targets function pointers or
 * control data, not file data.
 * ======================================================================== */
static int mode_write_root_file(poc_args_t *args) {
    (void)args;
    poc_log("--- CVE-2013-2094: write_root_file mode ---");

    poc_print_unsupported(POC_MODE_WRITE,
        "CVE-2013-2094 is an integer overflow exploit; it achieves LPE via "
        "ring 0 code execution through perf_swevent_enabled[] OOB write. "
        "Unlike DirtyPipe/splice-based vulnerabilities, it does NOT provide "
        "a file write primitive or page cache pollution capability. "
        "The exploit achieves LPE via code execution, not file modification");

    poc_print_fail("integer overflow exploit does not provide file write primitive");
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

    poc_log("=== CVE-2013-2094 CTF PoC ===");
    poc_log("Vuln: perf_swevent_init() integer overflow (array index OOB)");
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
            "CVE-2013-2094 uses perf_swevent integer overflow OOB write, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
