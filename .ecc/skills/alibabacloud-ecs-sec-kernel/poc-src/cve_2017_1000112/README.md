# CVE-2017-1000112 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2017-1000112 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Out-of-Bounds Write |
| 影响内核版本 | 2.6.19-rc1 ~ 4.10.x |
| 利用技术 | UDP Fragmentation Offset (UFO) 路径切换导致内存越界写入 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2017-1000112/poc.c
- **来源类型**: GitHub

### 利用原理
漏洞源于UFO和非UFO路径间的切换，在构建UFO数据包时通过fragmentation引发内存损坏。包含KASLR/SMEP绕过。

该漏洞的触发条件为UDP fragmentation offset路径切换。攻击者通过精心构造的系统调用序列，利用Out-of-Bounds Write类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为SKB缓冲区越界写入覆盖相邻对象。最终的提权路径通过通过OOB write修改内核数据结构实现提权完成，使非特权用户获得root权限。

### 关键技术点
1. UDP fragmentation offset路径切换
2. SKB缓冲区越界写入覆盖相邻对象
3. 通过OOB write修改内核数据结构实现提权

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
- **Phase 1 (Prepare)**: 创建UDP套接字，配置UFO相关选项
- **Phase 2 (Run)**: 通过UFO路径切换触发OOB写入实现提权
- **Phase 3 (Post)**: 验证提权结果、关闭套接字、清理网络状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket`
- `sendmsg`
- `setsockopt`

## 改造要点

### 核心修改
1. 添加KASLR绕过的CTF适配
2. 添加CLI参数解析
3. 添加SMEP绕过逻辑

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
# Original PoC Source for CVE-2017-1000112
# Vulnerability: __ip_append_data() UFO to non-UFO path switch causes heap out-of-bounds write, overwriting adjacent skb_shared_info metadata to control pipe_buffer structures for arbitrary write -> LPE
#
# Reference URL: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2017-1000112/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2017-1000112/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
