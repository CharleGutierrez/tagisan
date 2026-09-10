# CVE-2024-53125 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2024-53125 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Out-of-Bounds Access / Register Tracking Error |
| 影响内核版本 | v6.1 ~ v6.11 |
| 利用技术 | BPF sync_linked_regs()中subreg_def保持错误 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2024-53125_lts
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
BPF验证器在sync_linked_regs()处理链接寄存器同步时未正确保持subreg_def属性，导致子寄存器定义信息丢失。验证器对后续sub-register access范围判断错误。

攻击者可构造BPF程序利用subreg_def跟踪错误使验证器高估或低估寄存器值范围，最终实现越界内存访问和本地提权。

### 关键技术点
1. sync_linked_regs未正确保持subreg_def属性
2. 子寄存器定义信息在同步中丢失
3. 验证器范围判断错误导致越界访问

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
- **Phase 1 (Prepare)**: 验证BPF子系统可用、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过BPF subreg_def漏洞实现提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、卸载BPF程序和map、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `bpf(BPF_MAP_CREATE)`
- `bpf(BPF_PROG_LOAD)`
- `bpf(BPF_PROG_ATTACH)`

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
