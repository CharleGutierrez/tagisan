# CVE-2016-9793 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2016-9793 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Buffer Overflow (sock_setsockopt) |
| 影响内核版本 | 3.5 ~ 4.8.13 |
| 利用技术 | sk_sndbuf/sk_rcvbuf负值处理不当导致内存溢出，通过setsockopt实现提权 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/xairy/kernel-exploits/blob/master/CVE-2016-9793/poc.c
- **来源类型**: GitHub

### 利用原理
sock_setsockopt()函数不正确处理SO_SNDBUFFORCE和SO_RCVBUFFORCE的负值参数，导致缓冲区内存溢出。具有CAP_NET_ADMIN能力的攻击者通过特殊的setsockopt调用可以导致内存损坏或执行任意代码。

该漏洞的触发条件为setsockopt SO_SNDBUFFORCE/SO_RCVBUFFORCE负值。攻击者通过精心构造的系统调用序列，利用Buffer Overflow (sock_setsockopt)类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为sk_buff缓冲区溢出覆盖相邻内核对象。最终的提权路径通过通过缓冲区溢出修改内核数据结构提权完成，使非特权用户获得root权限。

### 关键技术点
1. setsockopt SO_SNDBUFFORCE/SO_RCVBUFFORCE负值
2. sk_buff缓冲区溢出覆盖相邻内核对象
3. 通过缓冲区溢出修改内核数据结构提权

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: setsockopt缓冲区溢出可覆写内核内存，实现任意写和完整提权

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
- **Phase 1 (Prepare)**: 创建套接字，检查CAP_NET_ADMIN权限
- **Phase 2 (Run)**: 设置负值缓冲区大小触发溢出
- **Phase 3 (Post)**: 验证提权结果、关闭套接字

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket`
- `setsockopt`
- `sendmsg`

## 改造要点

### 核心修改
1. 添加CTF输出协议
2. 添加CLI参数解析
3. 添加权限检查和安全退出

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
# Original PoC Source for CVE-2016-9793
# Vulnerability: sock_setsockopt() SO_SNDBUFFORCE/SO_RCVBUFFORCE 未正确校验负数/极大值，val * 2 触发有符号整数溢出，导致 alloc_skb 路径堆内存越界写，可覆写相邻内核对象实现本地提权。利用需要 CAP_NET_ADMIN（可通过 user namespace 获取）。
#
# Reference URL: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2016-9793/
#
# This file documents the original PoC source location.
# The actual PoC code should be downloaded from the URL above
# and saved here for reference during code review.
#
# Download instructions:
# 1. Visit: https://github.com/bcoles/kernel-exploits/blob/master/CVE-2016-9793/
# 2. Download the .c or .py exploit file
# 3. Save as: original.txt (replacing this file)
#
# Note: The current poc.c has been integrated with the
# poc_common.h CTF framework and implements the exploit logic.
```
