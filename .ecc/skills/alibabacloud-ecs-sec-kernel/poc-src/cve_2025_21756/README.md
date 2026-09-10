# CVE-2025-21756 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2025-21756 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Use-After-Free / Reference Count Decrement Error |
| 影响内核版本 | v5.10 ~ v6.8+ |
| 利用技术 | vsock transport reassignment期间绑定状态引用计数管理缺陷 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/hoefler02/CVE-2025-21756/blob/main/exploit.c
- **来源类型**: GitHub (hoefler02)

### 利用原理
vsock虚拟套接字在transport重新分配过程中存在引用计数管理缺陷。vsock_assign_transport对所有socket统一执行引用计数递减但未区分是否处于绑定状态，对unbound socket多余递减导致提前释放。

攻击者可通过创建vsock socket触发transport reassignment利用引用计数错误实现socket对象UAF。该漏洞有完整公开exploit，利用成功率高。

### 关键技术点
1. vsock transport reassignment错误递减unbound socket引用计数
2. 多余递减导致socket对象被提前释放
3. hoefler02提供完整exploit实现

## PoC改造计划

### CTF挑战模式适配
- **推荐测试模式**: write_root_file
- **模式选择理由**: 该漏洞的利用路径最终可实现对root所有文件的写入能力，适合write_root_file模式验证

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
- **Phase 1 (Prepare)**: 验证vsock模块可用、加载vhost_vsock模块、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过vsock transport reassignment UAF实现提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理vsock资源、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_VSOCK, SOCK_STREAM)`
- `bind()`
- `connect()`
- `setsockopt()`

## 改造要点

### 核心修改
1. 添加CTF模式CLI参数解析（getopt_long实现）
2. 集成CTF_FLAG/CTF_FAIL输出协议
3. 添加syscall日志记录功能
4. 实现--root-file和--write-value参数支持
5. 添加执行超时自动退出机制

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
