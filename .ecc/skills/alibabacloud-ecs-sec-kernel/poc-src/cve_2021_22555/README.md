# CVE-2021-22555 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2021-22555 |
| CVSS | 8.1 |
| 严重等级 | CRITICAL |
| 漏洞类型 | Heap Out-of-Bounds Write |
| 影响内核版本 | 2.6.19-rc1 ~ 5.12.x |
| 利用技术 | Netfilter x_tables中64→32位转换堆溢出，可绕过SMEP/SMAP |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/blob/master/pocs/linux/cve-2021-22555/exploit.c
- **来源类型**: GitHub

### 利用原理
15年历史的net/netfilter/x_tables.c越界写入漏洞。x86_64内核为32位进程处理xt_table时整数溢出导致堆溢出。Google kCTF获奖PoC。

该漏洞的触发条件为x_tables 64→32位compat转换整数溢出。攻击者通过精心构造的系统调用序列，利用Heap Out-of-Bounds Write类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为堆溢出覆盖相邻slab对象（msg_msg）。最终的提权路径通过通过msg_msg对象覆盖实现任意读写提权完成，使非特权用户获得root权限。

### 关键技术点
1. x_tables 64→32位compat转换整数溢出
2. 堆溢出覆盖相邻slab对象（msg_msg）
3. 通过msg_msg对象覆盖实现任意读写提权

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

### 输出协议适配
需同时支持CTF协议和Legacy协议：
```
CTF_FLAG:<value>           // 挑战成功
CTF_FAIL:<reason>          // 漏洞未触发
POC_RESULT:EXPLOITABLE     // Legacy兼容
POC_RESULT:NOT_EXPLOITABLE // Legacy兼容
```

### 三阶段验证集成
- **Phase 1 (Prepare)**: 创建netfilter规则环境，准备msg_msg喷射
- **Phase 2 (Run)**: 触发x_tables溢出覆盖msg_msg实现提权
- **Phase 3 (Post)**: 验证提权结果、清理netfilter规则、释放消息队列

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket`
- `setsockopt`
- `getsockopt`
- `msgsnd`
- `msgrcv`

## 改造要点

### 核心修改
1. 改造Google kCTF exploit为CTF验证
2. 添加CLI参数解析
3. 添加netfilter清理

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
