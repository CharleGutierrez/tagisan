# CVE-2016-0728 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2016-0728 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Refcount Overflow / Use-After-Free (keyring) |
| 影响内核版本 | 3.8.0 ~ 4.4.0 |
| 利用技术 | keyring对象引用计数溢出导致UAF，通过keyctl系统调用实现提权 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2016/CVE-2016-0728/cve-2016-0728.c
- **来源类型**: GitHub

### 利用原理
join_session_keyring()函数在错误处理路径中对keyring对象的引用计数处理不当，导致整数溢出和use-after-free。通过特殊的keyctl命令序列，攻击者可以在内核中执行任意代码。

该漏洞的触发条件为keyring引用计数整数溢出。攻击者通过精心构造的系统调用序列，利用Refcount Overflow / Use-After-Free (keyring)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为UAF后keyring对象重用实现代码执行。最终的提权路径通过通过keyctl序列溢出refcount触发UAF提权完成，使非特权用户获得root权限。

### 关键技术点
1. keyring引用计数整数溢出
2. UAF后keyring对象重用实现代码执行
3. 通过keyctl序列溢出refcount触发UAF提权

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: keyring引用计数溢出导致UAF，可实现任意内核写入和完整LPE提权

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
- **Phase 1 (Prepare)**: 创建测试keyring对象，记录初始引用计数
- **Phase 2 (Run)**: 循环调用keyctl溢出引用计数触发UAF
- **Phase 3 (Post)**: 验证提权结果、清理keyring对象

### Syscall日志记录
关键syscall需要记录到log-file：
- `keyctl`
- `add_key`
- `request_key`

## 改造要点

### 核心修改
1. 添加CTF输出协议
2. 添加CLI参数解析
3. 优化引用计数溢出循环性能

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
# Original PoC Source for CVE-2016-0728
# Vulnerability: join_session_keyring() 函数中 keyring 引用计数溢出（32 位有符号整数），攻击者通过反复加入同一 session keyring 使计数溢出归零，导致 keyring 对象被释放但仍有引用指向它（UAF）。利用堆喷射 + ROP 可实现本地提权。完整利用需要约 2^32 次调用（数小时），不适合 CTF 文件读写验证模式。
#
# Reference URL: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2016/CVE-2016-0728/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2016/CVE-2016-0728/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
