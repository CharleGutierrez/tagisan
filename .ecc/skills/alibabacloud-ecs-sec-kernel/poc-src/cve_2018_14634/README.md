# CVE-2018-14634 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2018-14634 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Integer Overflow |
| 影响内核版本 | 4.x ~ 4.18 |
| 利用技术 | create_elf_tables()函数中整数溢出，导致栈溢出 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/luan0ap/cve-2018-14634/blob/master/exploit.c
- **来源类型**: GitHub

### 利用原理
create_elf_tables函数在构建ELF程序头时整数溢出，导致栈缓冲区溢出。非特权用户可通过SUID程序触发。

该漏洞的触发条件为create_elf_tables ELF程序头整数溢出。攻击者通过精心构造的系统调用序列，利用Integer Overflow类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为栈缓冲区溢出覆盖返回地址。最终的提权路径通过通过SUID程序触发栈溢出执行提权代码完成，使非特权用户获得root权限。

### 关键技术点
1. create_elf_tables ELF程序头整数溢出
2. 栈缓冲区溢出覆盖返回地址
3. 通过SUID程序触发栈溢出执行提权代码

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
- **Phase 1 (Prepare)**: 准备触发条件：大量环境变量/参数填充栈空间
- **Phase 2 (Run)**: 执行SUID程序触发create_elf_tables溢出
- **Phase 3 (Post)**: 验证提权结果、清理临时SUID文件

### Syscall日志记录
关键syscall需要记录到log-file：
- `execve`
- `mmap`
- `getauxval`

## 改造要点

### 核心修改
1. 改造栈溢出payload为CTF验证
2. 添加CLI参数解析
3. 添加SUID文件安全清理

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
