# CVE-2006-2451 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2006-2451 |
| CVSS | 7.2 |
| 严重等级 | HIGH |
| 漏洞类型 | Core Dump (suid_dumpable) |
| 影响内核版本 | 2.6.13 ~ 2.6.17 |
| 利用技术 | 利用PR_SET_DUMPABLE和logrotate配合，通过core dump文件覆盖SUID二进制文件 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/0xdea/exploits/blob/master/linux/raptor_prctl2.c
- **来源类型**: GitHub

### 利用原理
suid_dumpable支持允许本地用户在无权限目录中创建core dump文件，通过与logrotate的配合，可以覆盖SUID二进制文件，获得root权限。

该漏洞的触发条件为PR_SET_DUMPABLE标志配合SUID程序崩溃。攻击者通过精心构造的系统调用序列，利用Core Dump (suid_dumpable)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为core dump文件写入到特权目录。最终的提权路径通过覆盖SUID二进制文件获得root执行完成，使非特权用户获得root权限。

### 关键技术点
1. PR_SET_DUMPABLE标志配合SUID程序崩溃
2. core dump文件写入到特权目录
3. 覆盖SUID二进制文件获得root执行

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 漏洞利用链可直接覆写文件内容，适合通过写入CTF标志验证

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
- **Phase 1 (Prepare)**: 设置PR_SET_DUMPABLE标志，准备目标SUID二进制路径
- **Phase 2 (Run)**: 触发SUID程序core dump到特权目录，覆盖目标文件
- **Phase 3 (Post)**: 验证文件覆盖结果、清理core dump残留

### Syscall日志记录
关键syscall需要记录到log-file：
- `prctl`
- `abort`
- `execve`

## 改造要点

### 核心修改
1. 改造core dump触发逻辑集成CTF输出
2. 添加CLI参数支持write模式
3. 添加文件覆盖验证逻辑

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

Source: based on existing poc.c analysis and vulnerability description

```c
# Original PoC Source for CVE-2006-2451
# Vulnerability: prctl(PR_SET_DUMPABLE, 2) allows unprivileged users to set suidsafe mode; crash-induced core dumps as root can write to restricted directories, enabling local privilege escalation.
#
# Reference URL: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2006/CVE-2006-2451/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2006/CVE-2006-2451/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
