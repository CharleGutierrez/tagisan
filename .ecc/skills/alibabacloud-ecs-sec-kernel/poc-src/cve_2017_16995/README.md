# CVE-2017-16995 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2017-16995 |
| CVSS | 7.8 |
| 严重等级 | CRITICAL |
| 漏洞类型 | BPF Verifier Bypass |
| 影响内核版本 | 4.4.0 ~ 4.15.x |
| 利用技术 | 符号扩展漏洞（s32→u64），绕过eBPF验证器限制 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/rapid7/metasploit-framework/blob/master/data/exploits/cve-2017-16995/exploit.c
- **来源类型**: GitHub

### 利用原理
eBPF模块中s32到u64的错误符号扩展导致验证器被绕过，允许非特权进程执行恶意BPF代码获得root权限。

该漏洞的触发条件为eBPF verifier s32→u64符号扩展错误。攻击者通过精心构造的系统调用序列，利用BPF Verifier Bypass类型的缺陷，在内核空间中实现代码执行或数据篡改。

利用过程中，关键的内存操作为绕过验证器后通过BPF实现任意内核读写。最终的提权路径通过利用BPF任意读写修改current->cred完成，使非特权用户获得root权限。

### 关键技术点
1. eBPF verifier s32→u64符号扩展错误
2. 绕过验证器后通过BPF实现任意内核读写
3. 利用BPF任意读写修改current->cred

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: BPF verifier bypass可获得任意内核内存读写原语，实现完整LPE提权链

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
- **Phase 1 (Prepare)**: 检查BPF子系统和unprivileged_bpf_disabled状态
- **Phase 2 (Run)**: 加载恶意BPF程序绕过验证器实现提权
- **Phase 3 (Post)**: 验证提权结果、卸载BPF程序

### Syscall日志记录
关键syscall需要记录到log-file：
- `bpf`
- `socket`
- `sendmsg`

## 改造要点

### 核心修改
1. 改造BPF payload为CTF模式
2. 添加CLI参数解析
3. 添加验证器状态检查

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
