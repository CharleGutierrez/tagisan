# CVE-2025-38502 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2025-38502 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Out-of-Bounds Read/Write / Bounds Check Error |
| 影响内核版本 | v5.8 ~ v6.8+ |
| 利用技术 | BPF cgroup local storage map边界检查缺陷 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2025-38502_lts
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
BPF cgroup local storage访问map元素时边界检查不足。helper函数路径中偏移验证缺陷允许BPF程序构造越界内存访问。

攻击者可加载精心构造的BPF程序利用cgroup local storage边界检查漏洞实现OOB读写，读取内核敏感信息和写入关键数据结构实现本地提权。

### 关键技术点
1. cgroup local storage map元素访问边界检查不足
2. helper函数路径中的偏移验证缺陷
3. BPF程序触发OOB读写实现提权

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
- **Phase 1 (Prepare)**: 验证BPF和cgroup子系统可用、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过BPF cgroup storage OOB实现提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、卸载BPF程序和map、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `bpf(BPF_MAP_CREATE, BPF_MAP_TYPE_CGROUP_STORAGE)`
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
