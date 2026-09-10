# CVE-2004-0077 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2004-0077 |
| CVSS | 9.0 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Memory Corruption (mremap) |
| 影响内核版本 | 2.2.x ~ 2.6.2 |
| 利用技术 | 利用mremap系统调用中缺失的返回值检查，通过PTE操作导致内存映射错误，执行任意代码 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-0077/160.c
- **来源类型**: GitHub

### 利用原理
do_mremap()函数未正确检查返回值，导致虚拟内存管理失效。攻击者通过精心构造的mremap调用，利用页表操作漏洞在内存中注入shellcode，最终获得root权限。

该漏洞的触发条件为mremap系统调用返回值未检查。攻击者通过精心构造的系统调用序列，利用Memory Corruption (mremap)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为PTE页表操作导致内存映射覆盖。最终的提权路径通过通过内存注入shellcode获得root完成，使非特权用户获得root权限。

### 关键技术点
1. mremap系统调用返回值未检查
2. PTE页表操作导致内存映射覆盖
3. 通过内存注入shellcode获得root

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: mremap内存损坏可覆写内核页表，获得任意内存写入能力，实现完整LPE提权

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
- **Phase 1 (Prepare)**: 创建目标内存映射区域，记录当前VMA状态
- **Phase 2 (Run)**: 执行mremap利用触发PTE混乱，注入提权代码
- **Phase 3 (Post)**: 验证提权结果、恢复内存映射状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `mremap`
- `mmap`
- `munmap`

## 改造要点

### 核心修改
1. 重写mremap调用链添加CTF输出
2. 添加CLI参数解析
3. 添加内存映射状态恢复逻辑

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
# Original PoC Source for CVE-2004-0077
# Vulnerability: mremap() 函数在 VMA 描述符耗尽时未检查 do_munmap() 返回值，导致产生'无主' PTE 条目。攻击者通过 saturate VMA 配额后触发 mremap 失败，未清除的页帧进入 slab 缓存，后续进程分配时可插入恶意数据到特权进程虚拟内存空间，实现本地提权。
#
# Reference URL: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-0077/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2004/CVE-2004-0077/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
