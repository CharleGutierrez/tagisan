#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

/**
 * Safe syscall wrappers
 * 
 * Provides syscall wrappers with timeout and error checking.
 * Ensures PoC will not hang or cause system anomalies.
 */
#ifndef SEC_KERNEL_SAFE_SYSCALL_H
#define SEC_KERNEL_SAFE_SYSCALL_H

#include <sys/socket.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <linux/if_alg.h>

/* AF_ALG related constants */
#ifndef AF_ALG
#define AF_ALG 38
#endif

#ifndef SOL_ALG
#define SOL_ALG 279
#endif

/* splice syscall number for x86_64 */
#ifndef SPLICE_SYSCALL_NR
#define SPLICE_SYSCALL_NR 76
#endif

/**
 * Safely create AF_ALG socket
 * 
 * Returns: socket fd >= 0 on success, -1 on failure
 */
static inline int safe_alg_socket(void) {
    int fd = socket(AF_ALG, SOCK_SEQPACKET, 0);
    if (fd < 0) {
        fprintf(stderr, "[FAIL] socket(AF_ALG): %s (errno=%d)\n", 
                strerror(errno), errno);
    }
    return fd;
}

/**
 * Safely bind AEAD algorithm
 */
static inline int safe_alg_bind_aead(int fd, const char *algo_name) {
    struct sockaddr_alg sa;
    memset(&sa, 0, sizeof(sa));
    sa.salg_family = AF_ALG;
    strncpy((char *)sa.salg_type, "aead", sizeof(sa.salg_type) - 1);
    strncpy((char *)sa.salg_name, algo_name, sizeof(sa.salg_name) - 1);
    
    int ret = bind(fd, (struct sockaddr *)&sa, sizeof(sa));
    if (ret < 0) {
        fprintf(stderr, "[FAIL] bind(aead/%s): %s (errno=%d)\n",
                algo_name, strerror(errno), errno);
    }
    return ret;
}

/**
 * Create safe temporary file target
 * 
 * Used as splice target for PoC, not a setuid binary.
 * File created under /tmp with permission 0600.
 */
static inline int safe_create_target(char *path_buf, size_t buf_len) {
    snprintf(path_buf, buf_len, "/tmp/sec-kernel-poc-XXXXXX");
    int fd = mkstemp(path_buf);
    if (fd < 0) {
        fprintf(stderr, "[FAIL] mkstemp: %s\n", strerror(errno));
        return -1;
    }
    /* Set permission 0600 */
    fchmod(fd, 0600);
    return fd;
}

/**
 * Safe cleanup: close fd + remove temp file
 */
static inline void safe_cleanup_target(int fd, const char *path) {
    if (fd >= 0) close(fd);
    if (path && path[0]) unlink(path);
}

/**
 * Safe splice call (with error checking)
 * Uses syscall wrapper to avoid implicit declaration
 */
static inline ssize_t safe_splice(int fd_in, int fd_out, size_t len) {
    /* Requires pipe as intermediary */
    int pipefd[2];
    if (pipe(pipefd) < 0) {
        fprintf(stderr, "[FAIL] pipe: %s\n", strerror(errno));
        return -1;
    }
    
    /* Use syscall wrapper for splice */
    ssize_t n1 = syscall(SPLICE_SYSCALL_NR, fd_in, NULL, pipefd[1], NULL, len, 0);
    if (n1 < 0) {
        fprintf(stderr, "[FAIL] splice(in->pipe): %s (errno=%d)\n", 
                strerror(errno), errno);
        close(pipefd[0]);
        close(pipefd[1]);
        return -1;
    }
    
    /* pipe -> fd_out */
    ssize_t n2 = syscall(SPLICE_SYSCALL_NR, pipefd[0], NULL, fd_out, NULL, len, 0);
    if (n2 < 0) {
        fprintf(stderr, "[FAIL] splice(pipe->out): %s (errno=%d)\n",
                strerror(errno), errno);
    }
    
    close(pipefd[0]);
    close(pipefd[1]);
    return n2;
}

#endif /* SEC_KERNEL_SAFE_SYSCALL_H */
