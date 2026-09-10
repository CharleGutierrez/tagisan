# CVE-2004-1235 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2004-1235 |
| CVSS | 8.0 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Race Condition (ELF/Uselib) |
| 影响内核版本 | 2.4.29 |
| 利用技术 | 利用load_elf_library和binfmt_aout中的VMA插入竞态条件，通过精确时序覆盖内存实现提权 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-1235/744.c
- **来源类型**: GitHub

### 利用原理
uselib()系统调用在插入VMA时存在竞态条件，未正确同步。利用LDT描述符修改和特殊的竞态时序，攻击者可以在内核空间中执行任意代码。

该漏洞的触发条件为uselib()系统调用VMA插入竞态窗口。攻击者通过精心构造的系统调用序列，利用Race Condition (ELF/Uselib)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为LDT描述符修改实现内核空间代码执行。最终的提权路径通过通过VMA竞态覆盖内核内存获得root完成，使非特权用户获得root权限。

### 关键技术点
1. uselib()系统调用VMA插入竞态窗口
2. LDT描述符修改实现内核空间代码执行
3. 通过VMA竞态覆盖内核内存获得root

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: uselib竞争条件导致内存损坏，可实现内核代码执行和任意文件写入

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
- **Phase 1 (Prepare)**: 准备ELF库文件和LDT描述符环境
- **Phase 2 (Run)**: 并发触发uselib竞态条件，利用时序窗口修改LDT
- **Phase 3 (Post)**: 验证提权结果、恢复LDT描述符状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `uselib`
- `mmap`
- `modify_ldt`

## 改造要点

### 核心修改
1. 添加竞态时序控制逻辑
2. 集成CTF输出协议
3. 添加LDT状态恢复代码

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
# Original PoC Source for CVE-2004-1235
# Vulnerability: uselib() race condition in load_elf_library: kernel generates library's brk segment without holding mmap_sem, allowing concurrent processes to alter VMA structures. Local users can gain root privileges via LDT call gate construction.
#
# Reference URL: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-1235/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-1235/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
