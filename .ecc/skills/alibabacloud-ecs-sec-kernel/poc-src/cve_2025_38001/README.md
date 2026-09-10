# CVE-2025-38001 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2025-38001 |
| CVSS | 8.4 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Use-After-Free / Race Condition |
| 影响内核版本 | v5.10 ~ v6.8+ |
| 利用技术 | HFSC packet scheduler class销毁的竞速UAF |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2025-38001_mitigation
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
HFSC调度器在class销毁过程中存在竞争条件，并发数据包处理仍在引用正被销毁的class时产生UAF。该漏洞能突破kernelctf各种缓解措施表明利用质量很高。

攻击者可配置HFSC qdisc创建多个class后并发执行class删除和数据包发送操作，利用销毁和数据路径竞争实现对已释放class对象的控制，最终实现可靠本地提权。

### 关键技术点
1. HFSC class销毁与数据包处理并发导致UAF
2. 能突破kernelctf mitigations的高质量利用
3. 多核系统上通过并发qdisc操作稳定触发

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
- **Phase 1 (Prepare)**: 创建虚拟网络设备和HFSC qdisc结构、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过HFSC class销毁竞争触发UAF并提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理qdisc配置和网络设备、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE)`
- `sendmsg(RTM_NEWQDISC)`
- `sendmsg(RTM_NEWTCLASS)`
- `sendmsg(RTM_DELTCLASS)`

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
