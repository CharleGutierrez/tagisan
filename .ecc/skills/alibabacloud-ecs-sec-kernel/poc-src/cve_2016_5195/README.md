# CVE-2016-5195 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2016-5195 |
| CVSS | 7.0 |
| 严重等级 | HIGH |
| 漏洞类型 | Race Condition (Copy-on-Write) |
| 影响内核版本 | 2.6.22 ~ 4.8.3 |
| 利用技术 | COW机制竞态条件，通过madvise和mmap结合绕过只读保护 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2016/CVE-2016-5195/pokemon.c
- **来源类型**: GitHub

### 利用原理
Dirty COW漏洞利用copy-on-write机制中的竞态条件，通过/proc/self/mem和madvise的精妙配合，在只读内存映射上进行写入。广泛用于修改/etc/passwd、SUID二进制或其他只读文件。

该漏洞的触发条件为/proc/self/mem写入与madvise(MADV_DONTNEED)竞态。攻击者通过精心构造的系统调用序列，利用Race Condition (Copy-on-Write)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为COW页面竞态绕过只读保护直接写入物理页。最终的提权路径通过覆写只读文件（如/etc/passwd）实现提权完成，使非特权用户获得root权限。

### 关键技术点
1. /proc/self/mem写入与madvise(MADV_DONTNEED)竞态
2. COW页面竞态绕过只读保护直接写入物理页
3. 覆写只读文件（如/etc/passwd）实现提权

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
- **Phase 1 (Prepare)**: 创建目标只读文件映射，启动竞态线程
- **Phase 2 (Run)**: 并发执行/proc/self/mem写入和madvise触发COW竞态
- **Phase 3 (Post)**: 验证文件写入结果、停止竞态线程

### Syscall日志记录
关键syscall需要记录到log-file：
- `mmap`
- `madvise`
- `open`
- `write`
- `lseek`

## 改造要点

### 核心修改
1. 改造文件写入目标为CTF验证文件
2. 添加CLI参数支持
3. 优化竞态线程同步

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
# Original PoC Source for CVE-2016-5195
# Vulnerability: Dirty COW - race condition in mm/gup.c copy-on-write handling allows writing to read-only memory mappings via /proc/self/mem racing against MADV_DONTNEED, causing page cache pollution
#
# Reference URL: https://github.com/firefart/dirtycow
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/firefart/dirtycow
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
