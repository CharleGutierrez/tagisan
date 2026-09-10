# CVE-2026-31431 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2026-31431 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Page Cache Pollution / Reference Count Corruption |
| 影响内核版本 | v5.15 ~ 最新 |
| 利用技术 | AF_ALG AEAD authencesn splice path页缓存污染 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/theori-io/copy-fail-CVE-2026-31431
- **来源类型**: 基于公开 PoC 实现 CTF 模式改造

### 利用原理
内核密码API (AF_ALG) AEAD模式处理associated data时splice路径存在页缓存引用隔离缺陷。crypto_aead_encrypt处理sendmsg+splice组合时未正确隔离页面引用，导致非特权用户能覆写root所有的文件页缓存。

属于Dirty Pipe同类型页缓存污染bug class。无需竞态条件具有确定性利用路径。攻击者只需创建AF_ALG socket配置AEAD算法通过splice操作将数据定向到目标文件页缓存即可覆写任意root文件。

注：基于 theori-io 的 copy-fail 实现进行 CTF 模式改造。

### 关键技术点
1. AF_ALG AEAD splice路径页缓存引用隔离失败
2. sendmsg+splice组合覆写root文件页缓存
3. 无需竞态的确定性利用路径

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
- **Phase 1 (Prepare)**: 加载AF_ALG和AEAD模块、创建CTF目标文件(root所有)、记录页缓存状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过AF_ALG AEAD splice路径污染页缓存覆写目标文件
- **Phase 3 (Post)**: 验证CTF flag写入结果、刷新页缓存、清理AF_ALG资源、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_ALG, SOCK_SEQPACKET)`
- `bind(algif_aead)`
- `setsockopt(ALG_SET_KEY)`
- `accept()`
- `sendmsg()`
- `splice()`

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
