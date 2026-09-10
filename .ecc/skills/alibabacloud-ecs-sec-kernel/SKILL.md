---
name: alibabacloud-ecs-sec-kernel
description: "Linux \u5185\u6838\u6001 CVE \u6F0F\u6D1E\u68C0\u6D4B\u4E0E PoC \u9A8C\u8BC1\u5DE5\
  \u5177\uFF0C\u4E13\u4E3A AI Agent \u8BBE\u8BA1\u3002 88 \u4E2A\u5185\u6838 CVE \u68C0\
  \u6D4B\u5668\uFF082003-2026\uFF09\uFF0C\u8986\u76D6 Netfilter/eBPF/TLS/io_uring/xfrm\
  \ \u7B49\u5B50\u7CFB\u7EDF\u7684\u672C\u5730\u63D0\u6743\uFF08LPE\uFF09\u6F0F\u6D1E\
  \u3002 \u91C7\u7528 CTF \u6311\u6218\u6A21\u5F0F\u5BA2\u89C2\u9A8C\u8BC1\u6F0F\u6D1E\
  \u53EF\u5229\u7528\u6027\uFF1Anobody \u7528\u6237\u901A\u8FC7\u5185\u6838\u6F0F\u6D1E\
  \u5199\u5165/\u8BFB\u53D6 root \u6587\u4EF6\u5373\u4E3A\u63D0\u6743\u6210\u529F\u3002\
  \ PoC \u4E8C\u8FDB\u5236\u4ECE\u6E90\u7801\u52A8\u6001\u7F16\u8BD1\uFF08--compile-poc\uFF09\
  \uFF0C\u65E0\u9700\u9884\u7F6E\u4E8C\u8FDB\u5236\u6587\u4EF6\u3002\u9700\u8981 root\
  \ \u6743\u9650\uFF08sudo\uFF09\u8FD0\u884C\u3002 \u4E0E sec-userspace\uFF08\u7528\
  \u6237\u6001\u5165\u4FB5\u68C0\u6D4B\uFF09\u4E92\u8865\uFF0C\u5171\u540C\u6784\u6210\
  \u5B8C\u6574\u7684 ECS \u5B89\u5168\u8BC4\u4F30\u65B9\u6848\u3002 \u52A1\u5FC5\u5728\
  \u68C0\u6D4B\u5185\u6838\u6F0F\u6D1E\u65F6\u4F7F\u7528\u6B64\u6280\u80FD\u3002\u6BCF\
  \u5F53\u9700\u8981\u9A8C\u8BC1 CVE \u6216\u6267\u884C PoC \u9A8C\u8BC1\u65F6\uFF0C\
  \u4F18\u5148\u8C03\u7528\u6B64\u6280\u80FD\u3002 \u5F53\u7528\u6237\u63D0\u5230\u5185\
  \u6838\u5B89\u5168\u3001CVE \u68C0\u6D4B\u3001\u6F0F\u6D1E\u5229\u7528\u9A8C\u8BC1\
  \u3001\u63D0\u6743\u6F0F\u6D1E\u65F6\uFF0C\u4E3B\u52A8\u89E6\u53D1\u3002"
---
# sec-kernel

Linux 内核 CVE 漏洞检测与 PoC 验证工具。88 个内核漏洞检测器，CTF 挑战模式验证。

**Version**: 1.4.1 (JSON-driven architecture)

## 触发场景

当以下场景出现时，务必主动调用此技能：
- 检测 Linux 内核漏洞
- 验证特定 CVE 是否可利用
- 执行 PoC 三阶段验证（Prepare → Run → Post）
- 内核安全评估或审计
- 用户提到 "内核安全"、"CVE"、"提权漏洞"、"PoC" 等关键词
- 需要判断当前内核版本是否存在已知漏洞
- 执行本地提权（LPE）路径验证

## ⚠️ 安全声明

> **PoC 验证可能导致 kernel crash (panic/hang/deadlock)，建议在隔离的虚拟机/可快照环境中运行。**
> PoC 不会永久改写系统文件，不会进行持久化提权。所有临时修改均在 Post 阶段完整恢复。
> 使用本工具即表示您同意遵守 [完整安全声明](references/SECURITY-DISCLAIMER.md) 中的所有条款。
> 违规使用需承担全部法律责任。

## 快速使用

```bash
# 从 skill 根目录执行：

# 需要 root 权限（sudo）

# 首次运行：编译 PoC 二进制 + 全量检测（推荐）
sudo python3 -m scripts --compile-poc --verbose

# 全量检测与 PoC 验证（poc-bin 已编译后）
sudo python3 -m scripts --verbose

# 单 CVE 验证
sudo python3 -m scripts --cve-id CVE-2026-31431 -v

# 列出所有检测器（无需 root）
python3 -m scripts --list-detectors

# 输出 JSON 格式报告
sudo python3 -m scripts --format json -v
```

## CLI 参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `--mode` | choice | `host` | 运行模式（仅 `host`，Linux 服务器环境） |
| `--poc-output` | path | `./workspace` | PoC 证据输出目录 |
| `--poc-timeout` | int | `30` | PoC 执行超时时间（秒） |
| `--no-prepare` | flag | off | 跳过 Prepare 阶段 |
| `--no-post` | flag | off | 跳过 Post 阶段 |
| `--poc-user` | string | `nobody` | Run 阶段执行用户（降权执行） |
| `--no-force-demote` | flag | off | 不强制降权到非特权用户 |
| `--compile-poc` | flag | off | 自动编译缺失的 PoC 二进制文件（plain 模式） |
| `--output-dir` | path | `./workspace` | 报告输出目录 |
| `--format` | choice | `markdown` | 报告格式（`markdown` / `json`） |
| `--cve-id` | string | - | 仅检测指定 CVE |
| `--config` | path | - | 配置文件路径 |
| `--verbose` / `-v` | flag | off | 详细输出 |
| `--list-detectors` | flag | off | 列出所有检测器（无需 root） |

### 关键参数说明

**默认行为**: 所有在 `kernel_cves.yaml` 中 `enabled: true` 的 CVE 都会自动执行 PoC 验证，无需额外参数。唯一跳过 PoC 的条件是将 CVE 设置为 `enabled: false`。

**`--compile-poc`**: 当 `poc-bin/` 目录下缺少对应 ELF binary 时，自动调用 `poc-src/build.sh` 编译。首次运行时**必须**使用此参数（poc-bin/ 不再随仓库分发，需从源码动态编译）。
> **Why**: PoC 二进制从源码动态编译，避免在 git 中存储大量 ELF 文件。编译采用静态链接 + strip，确保跨发行版兼容。

**`--no-prepare` / `--no-post`**: 跳过三阶段验证中的 Prepare 或 Post 阶段。用于调试目的，生产环境建议保留完整三阶段。

**`--poc-user`**: Run 阶段以指定用户身份执行 PoC binary，默认 `nobody`（uid=65534）。用于验证 LPE（本地提权）路径。

**`--no-force-demote`**: 默认情况下 PoC 执行会强制降权到 `--poc-user` 指定的非特权用户。此参数禁用强制降权。

### 为什么需要 sudo（root 权限）

PoC 三阶段验证需要 root 权限的原因：
- **Phase 1 (Prepare)**: 需要加载内核模块（`modprobe algif_aead`、`modprobe esp4` 等）、创建 root 拥有的目标文件、配置 xfrm SA/SP 等安全策略
- **Phase 3 (Post)**: 需要读取 root 文件验证写入结果、卸载内核模块、清理 xfrm 状态、恢复系统状态
- **权限分离**: Phase 2 (Run) 故意降权到 nobody，以验证漏洞是否能让非特权用户越权操作

### 为什么使用 CTF 模式

CTF 挑战模式的设计理由：
- **客观验证**: 通过文件内容的读/写来客观证明漏洞是否触发，而非主观判断
- **提权证据**: nobody 用户成功写入 root 文件 = 证明存在 LPE 路径
- **安全约束**: PoC 不会真正执行 `setuid(0)`，只是通过文件操作证明漏洞可被利用
- **可复现性**: CTF flag 每次随机生成，确保每次验证都是真实触发而非缓存结果

## 检测器统计

| 指标 | 数量 |
|------|------|
| 总检测器数 | 88 |
| write_root_file 模式 | 84 |
| read_root_file 模式 | 24 |
| uaf 模式 | 5 |

## 新增功能 (v1.2.0)

### PoC 执行统计模块

每次执行完成后自动输出详细统计报表，包含：
- SUCCESS（EXPLOITABLE / NOT_EXPLOITABLE）分类统计
- FAILED（TIMEOUT / CRASH / MISSING_BIN / PARSE_ERROR / PERMISSION）细分
- SKIPPED（NO_MODULE）
- 失败分析（root cause + recommendation + 是否为 PoC 实现问题）

### 三种 PoC 模式标准化

| 模式 | 目标文件权限 | 验证方式 | 数量 |
|------|-------------|---------|------|
| `write_root_file` | root:root 0644 | nobody 通过内核漏洞写入 root 文件 | 86 |
| `read_root_file` | root:root 0400 | nobody 通过内核漏洞读取 root 文件 | 24 |
| `uaf` | N/A | Use-After-Free 利用验证 | 5 |

### 增强的证据展示

- CTF flag 验证（write 模式：写入并回读确认；read 模式：读取受保护内容）
- 失败分析引擎（自动判定 root cause 并给出修复建议）
- PoC 实现问题标记（`[PoC-IMPL]` 标签区分环境问题和代码问题）
- 三阶段完整证据链输出

## 输出格式

### 检测摘要

```
==============================================================
  sec-kernel v1.4.1 - Linux Kernel CVE Detection
==============================================================
  Kernel: 5.15.0-91-generic (x86_64)
  Mode: host (Linux Server)
==============================================================

==============================================================
  Detection Summary
==============================================================
  Total CVEs checked:     88
  Vulnerable:             3
  Not Vulnerable:         80
  Uncertain:              2
  PoC Exploitable:        2
  Execution time:         45.32s
==============================================================
```

### PoC 执行统计

```
================================================================
  sec-kernel PoC Execution Statistics
================================================================
  Total Detectors:     88    (with PoC capability)
  PoC Executed:        83
  ------------------------------------------------------------
  SUCCESS (ran correctly):     78
    +- EXPLOITABLE:            2   (vulnerability confirmed)
    +- NOT_EXPLOITABLE:        76  (kernel patched/mitigated)
  ------------------------------------------------------------
  FAILED (execution error):    5
    +- TIMEOUT:                2
    +- CRASH:                  1
    +- MISSING_BIN:            0
    +- PARSE_ERROR:            2
    +- PERMISSION:             0
  ------------------------------------------------------------
  SKIPPED:                     2
    +- NO_MODULE:              2
  ------------------------------------------------------------
  Execution Time:         45.32s
================================================================
```

### CTF 挑战模式输出示例

```
⚠️  SECURITY DISCLAIMER / 安全声明
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
本工具仅限在已授权的隔离测试环境中使用。
严禁用于生产环境或未授权系统。违规使用需承担全部法律责任。
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[Phase 1: Prepare] (root)
  ✓ Loaded kernel module: algif_aead
  ✓ Created target file: /tmp/sec-kernel-poc-XXXX/poc_target (root:root 0644)
  ✓ Initial content: writeme_a3f8b2c1

[Phase 2: Run] (nobody, uid=65534)
  CTF_READ_BEFORE:writeme_a3f8b2c1
  CTF_WRITE:ctf{164d74cd804f9361} (attempting...)
  CTF_READ_AFTER:ctf{164d74cd804f9361}
  CTF_FLAG:ctf{164d74cd804f9361}
  POC_RESULT:EXPLOITABLE

[Phase 3: Post] (root)
  ✓ File content verified: ctf{164d74cd804f9361} ✓ MATCH
  ✓ Cleanup completed
  ✓ System state restored

[RESULT] CVE-2026-31431: EXPLOITABLE (confidence=0.95)
```

### 关键输出字段

| 字段 | 含义 |
|------|------|
| `CTF_READ_BEFORE:` | PoC 执行前读取的原始文件内容 |
| `CTF_WRITE:` | PoC 尝试写入的 CTF 值 |
| `CTF_READ_AFTER:` | PoC 执行后读取的文件内容 |
| `CTF_FLAG:ctf{xxx}` | 漏洞利用成功的标志值 |
| `POC_RESULT:EXPLOITABLE` | 漏洞可利用 |
| `POC_RESULT:NOT_EXPLOITABLE` | 漏洞不可利用（内核已修复） |

### 结果判定

| 结果 | 含义 | 建议操作 |
|------|------|----------|
| **EXPLOITABLE** | 当前内核存在可利用漏洞 | 立即升级内核 |
| **NOT_EXPLOITABLE** | 漏洞条件不满足或已修复 | 无需操作 |
| **DETECTION_ONLY** | 版本匹配但未执行 PoC | 建议进一步验证 |

### CTF 提权验证证据

以下 CVE 已在真实内核环境中通过 CTF 挑战模式验证提权成功（EXPLOITABLE），CTF flag 每次随机生成。

#### CVE-2026-31431 (CRITICAL) - AF_ALG AEAD splice 页缓存污染

```
[Phase 1: Prepare] (root)
  Created CTF target: /tmp/sec-kernel-poc-XXXX/poc_target_canary
  mode=write_root_file, uid=0, perm=644
  Initial content: writeme_f9de12afe0d6da5a

[Phase 2: Run] (nobody, uid=65534)
  CTF_WRITE:ctf{dddb28301f24e3db} (attempting...)
  CTF_FLAG:ctf{dddb28301f24e3db}
  POC_RESULT:EXPLOITABLE

[Phase 3: Post] (root)
  CTF write_root_file PASSED: inner value matches (dddb28301f24e3db)
  Rollback executed: target file removed
  System state restored

[RESULT] CVE-2026-31431: EXPLOITABLE (confidence=0.95)
  Kernel: 6.6.87.2-microsoft-standard-WSL2
  Exploit path: AF_ALG AEAD authencesn + splice() -> page cache corruption
```

#### CVE-2026-PENDING-DIRTYFRAG (CRITICAL) - DirtyFrag ESP+RxRPC 双变体

```
[Phase 1: Prepare] (root)
  Created CTF target: /tmp/sec-kernel-poc-XXXX/poc_target_dirtyfrag
  mode=write_root_file, uid=0, perm=644, size=4096
  Module state snapshot: esp4=loaded

[Phase 2: Run] (nobody, uid=65534)
  CTF_WRITE:ctf{ab2084cd52cdad11} (attempting via rxrpc/rxkad...)
  RxRPC variant failed (EAFNOSUPPORT), trying ESP/xfrm fallback...
  CTF_WRITE:ctf{ab2084cd52cdad11} (attempting via esp/xfrm fallback...)
  CTF_FLAG:ctf{ab2084cd52cdad11}
  POC_RESULT:EXPLOITABLE

[Phase 3: Post] (root)
  CTF write_root_file PASSED: inner value matches (ab2084cd52cdad11)
  Rollback executed: target file removed
  System state restored

[RESULT] CVE-2026-PENDING-DIRTYFRAG: EXPLOITABLE (confidence=0.95)
  Kernel: 6.6.87.2-microsoft-standard-WSL2
  Exploit path: ESP/xfrm variant (RxRPC fallback) -> splice() page cache write
  Dual-variant: RxRPC (Ubuntu 24.04) / ESP (WSL2) automatic fallback
```

## 三阶段执行流程

```
Phase 1 (root):   准备环境 — 加载模块、创建目标文件、记录初始状态
Phase 2 (nobody): 执行 PoC — 读原值 → 漏洞利用写入 → 读回验证
Phase 3 (root):   验证清理 — 独立确认写入结果、恢复系统状态
```

## 支持的 CVE

完整 CVE 检测列表见 [references/cve-list.md](references/cve-list.md)（88 个内核漏洞检测器，全部启用 CTF 挑战模式验证）。

覆盖子系统：Netfilter/nf_tables (18) | eBPF/BPF (8) | Network/Socket (12) | TLS (7) | io_uring (3) | Memory/Page Cache (5) | Filesystem (3) | IPsec/xfrm (3) | ptrace/cred (3) | Others (23)

## 系统要求

- Linux x86_64
- Python 3.8+
- root 权限（sudo）
- 内核模块加载能力（modprobe）

## 日志

PoC 执行日志：`workspace/poc-{CVE-ID}.log`
回退路径：`/tmp/poc-{CVE-ID}.log`（workspace 不可写时）
