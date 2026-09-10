# CVE-2026-PENDING-DIRTYFRAG 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2026-PENDING-DIRTYFRAG |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Page Cache Pollution Chain / Network Subsystem UAF |
| 影响内核版本 | v4.15 ~ 最新 (xfrm-ESP), v5.15 ~ 最新 (RxRPC) |
| 利用技术 | xfrm-ESP + RxRPC 页缓存污染链 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/V4bel/dirtyfrag/blob/master/exp.c
- **来源类型**: GitHub (V4bel)

### 利用原理
DirtyFrag链接xfrm-ESP (CVE-2026-43284)和RxRPC (CVE-2026-43500)两个网络子系统页缓存污染漏洞。通过netlink XFRM SA + UDP ESP over UDP + splice组合触发ESN验证逻辑bug实现确定性页缓存污染。

与Dirty Pipe和Copy Fail属于同一bug class。关键优势在于无需竞态条件(race-free)利用路径确定性极高。V4bel实现提供完整漏洞链设计文档和exploit代码。

注：CVE编号为临时pending状态，涵盖两个子漏洞。

### 关键技术点
1. xfrm-ESP ESN验证逻辑在fragment处理中存在缺陷
2. RxRPC页缓存引用隔离失败
3. 无需竞态条件的确定性页缓存污染

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
- **Phase 1 (Prepare)**: 加载xfrm/esp/rxrpc模块、配置XFRM SA(ESN+UDP封装)、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过xfrm-ESP+RxRPC链实现页缓存污染覆写目标文件
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理XFRM SA配置、刷新页缓存、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_XFRM)`
- `sendmsg(XFRM_MSG_NEWSA)`
- `socket(AF_RXRPC)`
- `splice()`
- `sendmsg()`

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
