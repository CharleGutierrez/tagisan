# CVE-2026-43500 漏洞技术分析

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2026-43500 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Page Cache Pollution via RxRPC/rxkad |
| 影响内核版本 | v5.15 ~ 最新 |
| 利用技术 | AF_RXRPC + rxkad v1 token + fcrypt in-place decrypt |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/V4bel/dirtyfrag/blob/master/exp.c
- **来源类型**: GitHub (V4bel)

### 利用原理
CVE-2026-43500 利用 RxRPC 子系统中 rxkad 认证的缺陷。攻击者通过 AF_RXRPC socket 创建连接，内核在处理 rxkad v1 token 验证时，`rxkad_verify_packet_1()` 函数对页缓存页面执行 in-place fcrypt 解密，导致共享的页缓存页面被污染，实现以普通用户身份覆写 root 文件。

核心利用链：
```
add_key("rxrpc", keyname, rxkad_v1_token)
→ socket(AF_RXRPC, SOCK_DGRAM, PF_INET)
→ setsockopt(RXRPC_SECURITY_KEY)
→ bind → sendmsg (initiate call)
→ UDP server receives DATA → sends CHALLENGE
→ Build malicious DATA with correct cksum
→ vmsplice(header) + splice(file_data) → pipe → UDP
→ kernel's rxkad_verify_packet_1() decrypts in-place on page cache
→ 每次精确写入 8 字节到目标文件对应偏移
```

### 关键技术点
1. **fcrypt in-place decrypt** — 内核 rxkad 验证包时直接在页缓存页上解密
2. **不需要 user namespace** — 更安全，不会被 AppArmor userns profile 拦截
3. **每次触发写入 8 字节** — 比 ESP 变体（4 字节）更高效
4. **需要 AF_RXRPC 模块** — WSL2 等环境可能不可用
5. **随机端口分配** — 避免 TIME_WAIT 端口冲突

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 该漏洞的利用路径最终可实现对root所有文件的写入能力

### CLI参数规范化
PoC需支持以下标准CLI参数：
- `--mode`: 测试模式 (read_root_file/write_root_file)
- `--root-file`: root权限目标文件路径
- `--write-value`: write模式的写入值 (ctf{...})
- `--log-file`: syscall日志文件路径
- `--verbose`: 详细输出开关

### 三阶段验证集成
- **Phase 1 (Prepare)**: 创建CTF目标文件（fcrypt加密的初始内容）、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过RxRPC/rxkad实现页缓存污染
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理环境

### Syscall日志记录
关键syscall需要记录到log-file：
- `add_key("rxrpc", ...)`
- `socket(AF_RXRPC)`
- `sendmsg()` / `recvmsg()`
- `splice()`
- `vmsplice()`

## 编译要求
- 编译器: gcc
- 静态链接: `-static`
- ELF掩蔽: 编译后前4字节替换为 `CVE\x00`
