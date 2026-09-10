# CVE-2015-1328 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2015-1328 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Permission Check Bypass (overlayfs) |
| 影响内核版本 | 3.13 ~ 3.19.0 |
| 利用技术 | overlayfs权限检查失效，通过用户命名空间和层叠挂载绕过访问控制 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/SecWiki/linux-kernel-exploits/blob/master/2015/CVE-2015-1328/37292.c
- **来源类型**: GitHub

### 利用原理
overlayfs实现未正确检查上层文件系统的权限。结合user namespace，攻击者可以在无特权的user namespace中创建overlayfs挂载，绕过文件权限检查，实现权限提升。

该漏洞的触发条件为overlayfs上层文件系统权限检查缺失。攻击者通过精心构造的系统调用序列，利用Permission Check Bypass (overlayfs)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为通过overlay层写入特权目录文件。最终的提权路径通过绕过权限检查写入/etc/passwd或SUID文件完成，使非特权用户获得root权限。

### 关键技术点
1. overlayfs上层文件系统权限检查缺失
2. 通过overlay层写入特权目录文件
3. 绕过权限检查写入/etc/passwd或SUID文件

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
- **Phase 1 (Prepare)**: 创建overlay工作目录和user namespace
- **Phase 2 (Run)**: 挂载overlayfs绕过权限检查写入目标文件
- **Phase 3 (Post)**: 验证文件写入结果、卸载overlay、清理临时目录

### Syscall日志记录
关键syscall需要记录到log-file：
- `mount`
- `unshare`
- `open`
- `write`

## 改造要点

### 核心修改
1. 改造文件写入为CTF验证输出
2. 添加CLI参数支持write模式
3. 添加overlay卸载清理

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
# Original PoC Source for CVE-2015-1328
# Vulnerability: OverlayFS 在 user namespace 中挂载时未正确验证上层目录权限，攻击者可通过在 user namespace 中创建 overlayfs 挂载并以任意权限在上层目录创建文件，实现对 root 文件的写入。
#
# Reference URL: https://github.com/0xf1d0/CVE-2015-1328
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/0xf1d0/CVE-2015-1328
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
