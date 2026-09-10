# CVE-2024-53141 漏洞技术分析与PoC改造计划

## 漏洞概述

| 属性 | 值 |
|------|-----|
| CVE ID | CVE-2024-53141 |
| CVSS | 7.8 |
| 严重等级 | HIGH |
| 漏洞类型 | Integer Overflow / Out-of-Bounds Write |
| 影响内核版本 | v4.15 ~ v6.11 |
| 利用技术 | netfilter ipset bitmap_ip_uadt范围检查整数溢出 |

## 原始PoC分析

### 参考来源
- **PoC URL**: https://github.com/google/security-research/tree/master/pocs/linux/kernelctf/CVE-2024-53141_cos_mitigation
- **来源类型**: Google security-research (kernelCTF)

### 利用原理
netfilter ipset的bitmap:ip集合在bitmap_ip_uadt中存在范围检查缺陷。IP范围参数导致整数溢出时上界验证被绕过允许越界写入bitmap。

攻击者可通过ipset netlink接口发送精心构造的IP范围参数触发bitmap越界写入，破坏相邻堆内存覆写关键内核对象实现本地提权。

### 关键技术点
1. bitmap_ip_uadt IP范围计算的整数溢出
2. 溢出绕过上界验证导致bitmap越界写入
3. 通过ipset netlink接口触发堆溢出

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
- **Phase 1 (Prepare)**: 加载ip_set和ip_set_bitmap_ip模块、创建CTF目标文件、记录系统状态快照
- **Phase 2 (Run)**: 降权nobody执行PoC binary，通过ipset bitmap整数溢出实现提权
- **Phase 3 (Post)**: 验证CTF flag写入结果、清理ipset集合、清理临时文件、恢复系统状态

### Syscall日志记录
关键syscall需要记录到log-file：
- `socket(AF_NETLINK, SOCK_RAW, NETLINK_NETFILTER)`
- `sendmsg()`

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
