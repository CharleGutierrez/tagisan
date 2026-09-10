# Fragnesia 漏洞技术分析

## 漏洞概述

| 属性 | 值 |
|------|-----|
| 内部 ID | CVE-2026-PENDING-FRAGNESIA |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Page Cache Pollution via ESP-in-TCP |
| 影响内核版本 | v4.15 ~ 最新（2026-05-13 补丁前） |
| 利用技术 | XFRM SA + ESP-in-TCP ULP + AES-GCM 密钥流查找表 |
| 补丁 | https://lists.openwall.net/netdev/2026/05/13/79 |
| 原始 PoC | https://github.com/v12-security/pocs/tree/main/fragnesia |
| 发现者 | William Bowling (V12 Security) |

## 原始PoC分析

### 利用原理

核心机制: skb_try_coalesce() 在 TCP 报文合并时丢失 SKBFL_SHARED_FRAG 标记。
当 TCP socket 在数据已通过 splice 从文件加入接收队列后转换为 espintcp ULP 模式时，
内核将排队的文件页作为 ESP 密文处理，AES-GCM 密钥流字节直接 XOR 到缓存的文件页中。

核心利用链:
1. unshare(CLONE_NEWUSER|CLONE_NEWNET) 获得 CAP_NET_ADMIN
2. 通过 NETLINK_XFRM 创建 ESP-in-TCP SA（AES-128-GCM，SPI 0x100）
3. 构建 256 项 AES-GCM 密钥流查找表（通过 AF_ALG ecb(aes)）
4. Splice-then-ULP 触发: 
   - fork() 创建 sender/receiver 对
   - sender: splice(target_fd → pipe → tcp)，前缀 ESP-in-TCP 长度字和 ESP 头
   - receiver: 延迟 TCP_ULP espintcp 安装至数据已入队
   - ULP 启用时内核原地 GCM 解密 → XOR 密钥流到 splice 映射的文件页
5. 逐字节迭代: 每个需要修改的字节查表选择 IV nonce 后触发一次

### 与 CVE-2026-43284 (ESP-in-UDP) 的关键差异

| 方面 | CVE-2026-43284 | Fragnesia |
|------|---------------|-----------|
| 子系统 | ESP-in-UDP + ESN seq_hi | ESP-in-TCP + TCP ULP espintcp |
| 加密 | hmac(sha256) + cbc(aes) | AES-128-GCM |
| 写入粒度 | 4 字节/次 | 1 字节/次 |
| 触发方式 | vmsplice+splice→UDP | splice→TCP + 延迟 ULP |

## PoC改造计划

### CTF挑战模式适配
- **测试模式**: write_root_file
- **模式理由**: 该漏洞可对任意只读文件页缓存实现单字节精确写入

### CLI参数规范化
- `--mode`: 测试模式 (write_root_file)
- `--root-file`: root权限目标文件路径
- `--write-value`: write模式的写入值 (ctf{...})
- `--log-file`: 日志文件路径
- `--verbose`: 详细输出开关

### 输出协议适配
CTF_READ_BEFORE:<original_content>
CTF_WRITE:<value> (attempting...)
CTF_READ_AFTER:<new_content>
CTF_FLAG:<value>
POC_RESULT:EXPLOITABLE

### 安全约束
- 执行超时: 30秒
- 目标文件须为 Prepare handler 创建的 4096 字节文件

### 编译要求
- 编译器: gcc -O2 -Wall -Wextra -static
- ELF掩蔽: 编译后前4字节替换为 CVE\x00
