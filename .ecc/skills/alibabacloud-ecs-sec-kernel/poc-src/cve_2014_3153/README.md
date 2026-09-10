# CVE-2014-3153 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2014-3153 |
| CVSS | 7.2 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Use-After-Free (Futex) |
| 影响内核版本 | 2.6.32 ~ 3.14.5 |
| 利用技术 | Futex自重入导致的UAF，通过精心构造futex操作实现提权 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/elongl/CVE-2014-3153
- **来源类型**: GitHub

### 利用原理
futex_requeue()函数允许相同地址的futex操作，导致内核中的use-after-free。Towelroot利用此漏洞通过futex重排操作修改内核内存中的credentials结构体实现提权。

该漏洞的触发条件为futex_requeue相同地址自重入。攻击者通过精心构造的系统调用序列，利用Use-After-Free (Futex)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为UAF修改task_struct credentials。最终的提权路径通过通过futex UAF覆盖cred结构体UID/GID为0完成，使非特权用户获得root权限。

### 关键技术点
1. futex_requeue相同地址自重入
2. UAF修改task_struct credentials
3. 通过futex UAF覆盖cred结构体UID/GID为0

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: futex UAF可通过堆喷控制freed对象实现任意写，完成完整LPE提权链

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
- **Phase 1 (Prepare)**: 准备futex地址和子线程环境
- **Phase 2 (Run)**: 触发futex_requeue自重入UAF修改credentials
- **Phase 3 (Post)**: 验证提权结果、清理futex状态和子线程

### Syscall日志记录
关键syscall需要记录到log-file：
- `futex`
- `clone`
- `waitpid`

## 改造要点

### 核心修改
1. 改造credentials修改为CTF验证
2. 添加CLI参数解析
3. 添加线程清理逻辑

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
# Original PoC Source for CVE-2014-3153
# Vulnerability: futex_requeue() 未验证两个 futex 地址是否不同，攻击者通过构造相同地址的 futex_requeue 调用触发竞争条件，导致 waiter 状态被不安全地修改，可绕过 VFS 权限检查实现对 root 文件的读写。
#
# Reference URL: https://github.com/timwr/CVE-2014-3153
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/timwr/CVE-2014-3153
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
