# CVE-2026-43284 漏洞技术分析

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2026-43284 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Page Cache Pollution via xfrm-ESP |
| 影响内核版本 | v4.15 ~ 最新 |
| 利用技术 | XFRM SA + ESP-in-UDP + ESN seq_hi 字段 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/V4bel/dirtyfrag/blob/master/exp.c
- **来源类型**: GitHub (V4bel)

### 利用原理
CVE-2026-43284 利用 xfrm（IPsec）子系统中 ESP-in-UDP 解封装路径的缺陷。攻击者在用户命名空间内创建 XFRM SA，通过构造特定 ESP 报文触发内核解密流程，将 SA 结构中 `xfrm_replay_state_esn.seq_hi` 字段的值写入到目标文件的页缓存中，实现以普通用户身份覆写 root 文件。

核心利用链：
```
unshare(CLONE_NEWUSER|CLONE_NEWNET)
→ 创建 N 个 XFRM SA（每个 SA 的 seq_hi 承载 4 字节 payload）
→ 对每个 SA 独立执行 do_one_write():
    open(target, O_RDONLY) → pipe()
    → vmsplice(24-byte ESP header into pipe)
    → splice(target_fd → pipe, 16 bytes)
    → splice(pipe → udp_socket, 40 bytes)
    → 内核 ESP 解封装触发页缓存污染
→ 每次精确写入 4 字节到目标文件对应偏移
```

### 关键技术点
1. **ESN seq_hi 是数据传输的核心通道** — 每创建一个 SA 并触发一次 ESP 解密，就能向目标文件的页缓存写入精确的 4 字节
2. **加密算法选择** — 使用 `hmac(sha256) + cbc(aes)` 分离式认证+加密组合，兼容性最广
3. **ESP 报文头必须完整（24 字节）** — 4(SPI) + 4(Seq) + 16(ICV padding)
4. **多 SA 循环触发** — N 个独立 SA，每个承载 4 字节，逐个触发写入
5. **user namespace 隔离** — 需要 `unshare(CLONE_NEWUSER|CLONE_NEWNET)` 获得 XFRM SA 创建权限

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

### 输出协议适配
需同时支持CTF协议和Legacy协议：
```
CTF_FLAG:<value>           // 挑战成功
CTF_FAIL:<reason>          // 漏洞未触发
POC_RESULT:EXPLOITABLE     // Legacy兼容
POC_RESULT:NOT_EXPLOITABLE // Legacy兼容
```

### 三阶段验证集成
- **Phase 1 (Prepare)**: 创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过xfrm-ESP实现页缓存污染覆写目标文件
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理环境、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `unshare(CLONE_NEWUSER|CLONE_NEWNET)`
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_XFRM)`
- `sendmsg(XFRM_MSG_NEWSA)`
- `splice()`
- `vmsplice()`

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
