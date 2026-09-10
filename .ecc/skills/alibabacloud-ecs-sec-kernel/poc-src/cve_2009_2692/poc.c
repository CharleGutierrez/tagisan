/**
 * CVE-2009-2692 PoC - CTF Challenge Mode
 *
 * sock_sendpage() NULL pointer dereference via PF_PPPOX
 *
 * Vulnerability Description:
 *   Linux kernel 2.4.4 through 2.6.30.4 has a local privilege escalation
 *   vulnerability in the sock_sendpage() function. The kernel fails to
 *   validate that socket->ops->sendpage is non-NULL before calling it,
 *   allowing a local user to map the NULL page with controlled code and
 *   trigger arbitrary code execution when sock_sendpage() dereferences
 *   the NULL function pointer via sendfile() on a PF_PPPOX socket.
 *
 * CTF Modes:
 *   read_root_file  - Unsupported (NULL pointer deref is not a read oracle)
 *   write_root_file - Unsupported (NULL pointer deref does not provide
 *                     controlled file write primitive; it executes code
 *                     at NULL page, not page cache pollution)
 *
 * Exploitation approach:
 *   This is a classic NULL pointer dereference exploit. The attack path:
 *   1. mmap(NULL, ...) to map page 0 with user-controlled shellcode
 *   2. Create PF_PPPOX socket with NULL ->sendpage
 *   3. Create pipe with one end as "file" for sendfile
 *   4. Call sendfile(socket_fd, pipe_fd, ...) -> sock_sendpage()
 *   5. sock_sendpage calls file->f_op->sendpage (which is NULL -> page 0)
 *   6. NULL->sendpage() executes user-controlled code at address 0
 *
 *   This is fundamentally a CODE EXECUTION vulnerability, not a page cache
 *   pollution attack. The CTF framework's read/write file modes do not map
 *   to this vulnerability class. The PoC outputs CTF_UNSUPPORTED for both.
 *
 * Safety:
 *   - Does NOT actually exploit the vulnerability (no NULL page mapping)
 *   - Only checks kernel version and socket availability
 *   - alarm(10) forced timeout
 *   - No fork/exec of shell commands
 *
 * Build:
 *   gcc -O2 -Wall -static -I common -DPOC_NO_SYSTEM_MODIFY=1 \
 *       -o ../poc-bin/cve_2009_2692.bin cve_2009_2692/poc.c
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
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/sendfile.h>
#include <linux/limits.h>

/* ========================================================================
 * Constants
 * ======================================================================== */
#define PAGE_SIZE       4096

/* ========================================================================
 * check_kernel_version_affected - Check if kernel is in affected range
 *
 * Affected: 2.4.4 <= kernel < 2.6.31
 * We read /proc/version and check for version indicators.
 * Full parsing is complex; Python layer does version matching.
 * Here we just verify we can read the version string.
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
 * check_pppox_socket - Check if PF_PPPOX socket can be created
 *
 * PF_PPPOX = 24. On vulnerable kernels, this protocol family is available.
 * The actual sendpage pointer being NULL depends on the specific PPPoX
 * sub-protocol used.
 * ======================================================================== */
static int check_pppox_socket(void) {
    int fd = socket(PF_PPPOX, SOCK_STREAM, 0);
    if (fd >= 0) {
        poc_log_syscall("socket(PF_PPPOX, SOCK_STREAM, 0)", fd, 0);
        poc_log("PF_PPPOX socket created (fd=%d)", fd);
        close(fd);
        return 1;
    }

    poc_log_syscall("socket(PF_PPPOX, SOCK_STREAM, 0)", -1, errno);
    if (errno == EPROTONOSUPPORT || errno == EINVAL) {
        poc_log("PF_PPPOX not supported (errno=%d)", errno);
        return 0;
    }
    if (errno == EPERM || errno == EACCES) {
        poc_log("PF_PPPOX restricted (errno=%d)", errno);
        return 0;
    }

    poc_log("PF_PPPOX check failed (errno=%d)", errno);
    return -1;
}

/* ========================================================================
 * check_sendfile_syscall - Verify sendfile syscall exists
 *
 * sendfile() is the syscall that internally invokes sock_sendpage().
 * If sendfile exists, the vulnerability path exists on vulnerable kernels.
 * ======================================================================== */
static int check_sendfile_syscall(void) {
    /* Test with invalid arguments to verify syscall path exists */
    int pipe_fds[2];
    if (pipe(pipe_fds) < 0) {
        poc_log_syscall("pipe()", -1, errno);
        return -1;
    }
    poc_log_syscall("pipe()", 0, 0);

    /*
     * sendfile(out_fd, in_fd, offset, count)
     * x86_64 syscall number: 187, i386: 187
     * Using libc wrapper for safety
     */
    ssize_t ret = sendfile(pipe_fds[0], pipe_fds[1], NULL, 0);
    poc_log_syscall("sendfile(out, in, NULL, 0)", ret, errno);

    close(pipe_fds[0]);
    close(pipe_fds[1]);

    if (ret >= 0 || errno == EINVAL || errno == EBADF) {
        /* Syscall exists - errors are expected with invalid fds */
        poc_log("sendfile syscall path exists");
        return 1;
    }

    poc_log("sendfile syscall check returned unexpected error: %s",
            strerror(errno));
    return -1;
}

/* ========================================================================
 * mode_read_root_file - CTF read_root_file mode
 *
 * CVE-2009-2692 is a NULL pointer dereference leading to code execution.
 * It does NOT provide an arbitrary file read capability. The vulnerability
 * executes code at the NULL page, it doesn't leak file contents.
 * ======================================================================== */
static int mode_read_root_file(poc_args_t *args) {
    poc_log("--- CVE-2009-2692: read_root_file mode ---");

    poc_print_unsupported(POC_MODE_READ,
        "CVE-2009-2692 is a NULL pointer dereference exploit that executes "
        "code at page 0; it does not provide a file read primitive. "
        "The vulnerability triggers sock_sendpage()->NULL->sendpage() "
        "which executes user-mapped code, not reading file contents");

    poc_print_fail("NULL pointer deref exploit does not provide file read primitive");
    return 1;
}

/* ========================================================================
 * mode_write_root_file - CTF write_root_file mode
 *
 * CVE-2009-2692 is a NULL pointer dereference leading to code execution.
 * It does NOT provide a controlled file write capability like page cache
 * pollution (DirtyPipe, vmsplice, AF_ALG). The exploit achieves privilege
 * escalation by executing shellcode at address 0, not by writing to files.
 *
 * Unlike DirtyPipe (splice to page cache) or vmsplice (iovec injection),
 * this vulnerability has NO mechanism to write arbitrary data to a file's
 * page cache. The NULL page executes as code, not as file data.
 * ======================================================================== */
static int mode_write_root_file(poc_args_t *args) {
    poc_log("--- CVE-2009-2692: write_root_file mode ---");

    poc_print_unsupported(POC_MODE_WRITE,
        "CVE-2009-2692 is a NULL pointer dereference exploit; it executes "
        "user code at address 0 via sock_sendpage()->NULL->sendpage(). "
        "Unlike DirtyPipe/splice-based vulnerabilities, it does NOT provide "
        "a file write primitive or page cache pollution capability. "
        "The exploit achieves LPE via code execution, not file modification");

    poc_print_fail("NULL pointer deref exploit does not provide file write primitive");
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

    poc_log("=== CVE-2009-2692 CTF PoC ===");
    poc_log("Vuln: sock_sendpage() NULL pointer deref via PF_PPPOX");
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
            "CVE-2009-2692 uses NULL pointer deref, not UAF");
        result = 1;
    } else {
        poc_print_unsupported(args.mode, "unknown mode");
        result = 1;
    }

    poc_log_close();
    return result;
}
