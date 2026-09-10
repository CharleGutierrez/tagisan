# CVE-2024-58239 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2024-58239 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Use-After-Free / Loop Logic Error |
| 影响内核版本 | v5.15 ~ v6.6 |
| 利用技术 | TLS RX路径recv()无限循环导致页面释放后使用 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2024-58239_mitigation
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
内核TLS接收路径处理non-DATA记录时循环逻辑错误，process_rx_list完成后未正确停止接收循环继续处理。结合splice()传输页面可触发已释放SKB引用的UAF。

攻击者可通过建立TLS连接发送特定序列TLS记录，同时使用splice()操作页面传输触发页面UAF，实现页面级别任意写入实现本地提权。

### 关键技术点
1. TLS接收循环在process_rx_list完成后未停止
2. non-DATA record处理后的循环条件错误
3. splice()传输页面时触发已释放SKB引用

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
- **Phase 1 (Prepare)**: 验证内核TLS支持、创建TLS测试服务器、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过TLS recv循环漏洞触发页面UAF并提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、关闭TLS连接、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_INET, SOCK_STREAM)`
- `setsockopt(SOL_TLS)`
- `recv()`
- `splice()`

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
