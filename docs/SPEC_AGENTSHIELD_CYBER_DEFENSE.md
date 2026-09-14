# 🛡️ Tagisan Architecture Blueprint & Specification (RFC-005)
## Autonomous Cyber Defense & Threat Hunting Subsystem (`tgs shield`)

**Status:** Approved Specification / Systems Architecture Reference (RFC-005)  
**Target:** Tagisan Engine (`tagisan-rs` / `tgs`)  
**Scope:** Zero Ambient Authority Sandboxing, Pre-Execution Command & Payload Auditing (`tgs shield audit`), Recursive Repository & Supply Chain Scanning (`tgs shield scan`), Real-Time Forensic Notification Hub, and Nation-State APT Threat Mitigation (Lazarus Group, APT38, TraderTraitor, Kimsuky).

---

## 📑 Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [Threat Model: Nation-State APT & Supply Chain Tradecraft](#2-threat-model-nation-state-apt--supply-chain-tradecraft)
3. [Zero Ambient Authority Security Model](#3-zero-ambient-authority-security-model)
4. [AgentShield Architecture & Scanning Pipeline](#4-agentshield-architecture--scanning-pipeline)
5. [Pre-Execution Command & Payload Auditing (`tgs shield audit`)](#5-pre-execution-command--payload-auditing)
6. [Static Repository & Supply Chain Vetting (`tgs shield scan`)](#6-static-repository--supply-chain-vetting)
7. [Real-Time Forensic Notification & Abnormality Alerting Subsystem](#7-real-time-forensic-notification--abnormality-alerting-subsystem)
8. [Autonomous Prompt-Driven Security Operations](#8-autonomous-prompt-driven-security-operations)
9. [Verification Matrix & Brutal Test Coverage](#9-verification-matrix--brutal-test-coverage)
10. [CLI Command Reference](#10-cli-command-reference)

---

## 1. Executive Summary

Autonomous AI agents with tool execution capabilities pose unprecedented workstation security risks. When an AI agent inherits ambient OS permissions (e.g. running arbitrary shell commands, reading user directories, evaluating untrusted scripts), an attacker who poisons a pull request, opens an issue with adversarial instructions, or publishes a trojanized dependency can hijack the agent to compromise the host.

**RFC-005 specifies Tagisan's Standalone Cyber Defense & AgentShield Subsystem**. Unlike traditional agent frameworks that rely on external Docker containers or post-execution host monitors, `tgs` enforces an in-memory, pre-execution gate with **Zero Ambient Authority**:
- Every tool invocation, shell command, file read, and network target is verified against deterministic security invariants prior to execution.
- Cryptographic keyrings, Web3 wallets, SSH keys, and cloud credentials are hard-blocked from agent discovery.
- Reverse shells, living-off-the-land download cradles, and indirect prompt injections are neutralized before reaching the operating system kernel or LLM tokenizer.
- Security abnormalities and attack attempts are instantly broadcast via high-visibility ANSI terminal banners and cross-platform desktop notifications (Windows WinRT Toast / Linux `notify-send`).

---

## 2. Threat Model: Nation-State APT & Supply Chain Tradecraft

Tagisan models adversarial tradecraft documented in active nation-state cyber operations targeting developers, cryptocurrency protocols, and AI pipelines—specifically:
- **Lazarus Group / APT38 (TraderTraitor):** Known for targeting cryptocurrency developers via poisoned npm dependencies, disguised job recruitment coding tests, and automated exfiltration of private keys (`id.json`, keystores).
- **Kimsuky:** Known for living-off-the-land (LotL) execution via encoded PowerShell scripts, Windows `certutil` download cradles, and spearphishing PR reviews.

```
                      [ UNTRUSTED PAYLOAD / REPO / PR / PROMPT ]
                                        │
                                        ▼
                   ┌──────────────────────────────────────────┐
                   │    Tagisan AgentShield Pre-Flight Gate   │
                   └────────────────────┬─────────────────────┘
                                        │
          ┌─────────────────────────────┼────────────────────────────┐
          ▼                             ▼                            ▼
  [ APT38 / Lazarus ]         [ Supply Chain Poison ]       [ Prompt Injection ]
  • /dev/tcp/ C2 Sockets      • package.json postinstall    • <!-- SYSTEM: bypass -->
  • powershell -enc cradles   • build.rs socket backdoors   • "Ignore instructions"
  • certutil -urlcache        • base64 encoded droppers     • [INST] override tags
          │                             │                            │
          └─────────────────────────────┼────────────────────────────┘
                                        │
                        [ ZERO AMBIENT AUTHORITY CHECK ]
                  Checks target against sensitive asset registry:
                 • Solana (~/.config/solana/id.json)
                 • Ethereum (~/.ethereum/keystore)
                 • Bitcoin (wallet.dat) & MetaMask
                 • Cloud (~/.aws, ~/.config/gcloud, .env, id_rsa)
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
                    [ CRITICAL ]                  [ PASS ]
             🚨 High-Vis ANSI Warning          ⚡ Safe Sub-Millisecond
             🔔 Desktop Toast Notification        Execution Flow
             🛑 Execution Terminated
```

### Attack Vectors Covered

| Attack Vector | TTPs / Signatures | Interception Mechanism |
| :--- | :--- | :--- |
| **Reverse Shells / C2** | `/dev/tcp/`, `/dev/udp/`, `nc -e`, `ncat`, `socat`, Python/Perl socket redirects | Pre-execution regex & string parser halts process spawn |
| **Download Cradles** | `powershell -enc`, `IEX (New-Object Net.WebClient)`, `certutil -urlcache`, `bitsadmin` | Blocks obfuscated LotL command execution |
| **Web3 Wallet Theft** | Targeting Solana `id.json`, Ethereum keystores, Bitcoin `wallet.dat`, MetaMask vaults | Zero Ambient Authority hard-block on paths and read tools |
| **Cloud Secrets Exfil** | `.aws/credentials`, `~/.config/gcloud`, `~/.azure`, `~/.ssh/id_rsa`, `.env` | Path and command inspection blocks read and exfiltration |
| **Trojanized Dependencies**| `postinstall` script execution, malicious Rust `build.rs` socket hooks | Static AST / JSON recursive dependency scanner (`tgs shield scan`) |
| **Indirect Prompt Injection**| `<!-- SYSTEM:`, `[INST]`, `Ignore previous instructions and execute...` | Pre-LLM tokenizer scanning in `AutonomousAgent::run_with_content` |

---

## 3. Zero Ambient Authority Security Model

In standard computing, a process inherits the ambient authority of the invoking user. In `tgs`:
1. **No Implicit File Access:** Even if an agent is granted `--sandbox=false` or has access to `read_file` or shell tools, requests to paths containing sensitive key material are intercepted in memory before `std::fs` is touched.
2. **Deterministic Sensitive Asset Registry:**
   - Solana: `.config/solana/id.json`, `solana-keygen`
   - Ethereum: `.ethereum/keystore`, `geth/keystore`
   - Bitcoin: `wallet.dat`, `.bitcoin`
   - MetaMask: `nkbihfbeogaeaoehlefnkodbefgpgknn`
   - Cloud: `.aws/credentials`, `.config/gcloud`, `.azure`, `.kube/config`
   - SSH & Secrets: `id_rsa`, `id_ed25519`, `.env`
3. **Fail-Closed Principle:** Any ambiguous path or command containing obfuscated sub-expressions defaults to `AgentShieldVerdict::Block`.

---

## 4. AgentShield Architecture & Scanning Pipeline

AgentShield is organized across three integrated components:
1. **`AgentShieldScanner` (`src/ecc/agentshield.rs`):**
   - High-throughput regex and AST scanning engine executing in microseconds.
   - Evaluates commands, file paths, raw text contents, and repository directories.
2. **`AgentShieldSecurityGate` (`src/swarm/harmony/gates.rs`):**
   - Anti-sycophancy and security barrier in the multi-agent Harmony swarm assembly line.
   - Rejects malicious implementation artifacts and forces critiques if an agent outputs insecure code or backdoors.
3. **`PluginSecurityGovernor` (`src/plugins/security.rs`):**
   - Enforces capabilities and sandboxing boundaries on dynamic WebAssembly and native shared-library plugins.

---

## 5. Pre-Execution Command & Payload Auditing

Developers and automated CI pipelines can vet commands before execution using `tgs shield audit`:

```bash
# Auditing an obfuscated PowerShell download cradle
tgs shield audit "powershell -enc JABjAGwAaQBlAG4AdAAgAD0AIABOAGUAdwAtAE8AYgBqAGUAYwB0AA=="

# Output:
# 🛡️ AGENTSHIELD COMMAND AUDIT
# Target Command: powershell -enc JABjAGwAaQBlAG4AdAAgAD0AIABOAGUAdwAtAE8AYgBqAGUAYwB0AA==
# Verdict:        BLOCKED
# Rule Broken:    Obfuscated PowerShell / Living-off-the-Land Download Cradle
# Threat Level:   CRITICAL
```

---

## 6. Static Repository & Supply Chain Vetting

The `tgs shield scan [PATH]` command performs a recursive inspection of source repositories, dependency manifests, and documentation:
- Analyzes `package.json` for dangerous lifecycle scripts (`preinstall`, `postinstall`).
- Scans Rust `build.rs` and Python `setup.py` for dynamic network socket creation.
- Flags hidden HTML and Markdown prompt injections in documentation files.

```bash
tgs shield scan ./vendor/untrusted-package
```

---

## 7. Real-Time Forensic Notification & Abnormality Alerting Subsystem

The `NotificationHub` (`src/notify.rs`) routes real-time security events across multiple channels:
- **ANSI Terminal Alerts (`stderr`):**
  ```
  ╔════════════════════════════════════════════════════════════════════════════╗
  ║ 🚨 [CRITICAL SECURITY ALERT] NORTH KOREAN APT TRADE-CRAFT BLOCKED         ║
  ║ Vector:        Obfuscated PowerShell Download Cradle                       ║
  ║ Payload:       powershell -EncodedCommand JABjAGwAaQ...                    ║
  ║ Action Taken:  Execution Halted • Threat Vector Isolated                   ║
  ╚════════════════════════════════════════════════════════════════════════════╝
  ```
- **Cross-Platform OS Notifications:** Native Windows Toast notifications and Linux `notify-send` deliver instant background notifications without interrupting terminal workflows.
- **SIEM / Audit Ring-Buffer:** Thread-safe in-memory log with monotonic timestamps and JSON serialization for SIEM ingestion.

---

## 8. Autonomous Prompt-Driven Security Operations

`tgs` supports prompt-driven security workflows using local abliterated models or cloud reasoning models:

```bash
# Interactive REPL as a specialized Security Auditor
tgs repl --agent security-auditor

# Autonomous vulnerability remediation
tgs agent "Scan src/engine/gguf.rs for buffer overflow risks, prove with tests, and patch surgically"

# Enterprise Craftsmanship & Security Rule Audit
tgs ecc audit src/
```

---

## 9. Verification Matrix & Brutal Test Coverage

Tagisan's cyber defense subsystem is verified by dedicated brutal test suites achieving a **100% pass rate**:

| Test Suite | Test Count | Focus Area |
| :--- | :--- | :--- |
| `tests/cyber_defense_hardening_brutal_tests.rs` | 10 Tests | Reverse shells, download cradles, crypto wallet protection, cloud credentials, prompt injection, notification delivery |
| `tests/tgs_cybersecurity_standalone_brutal_tests.rs` | 7 Tests | End-to-end multi-stage North Korean APT simulation using solely `tgs` tools, CLI commands, and AgentShield |
| `tests/local_to_nonlocal_transition_brutal_tests.rs` | 24 Tests | Local <-> Cloud LLM cascade failovers, rate limit resilience, and cost accounting |
| `tests/skills_local_to_nonlocal_transition_brutal_tests.rs` | 17 Tests | Dynamic skill context calibration across local and cloud models |
| `tests/skills_notification_local_nonlocal_brutal_tests.rs` | 18 Tests | Real-time user notification and abnormality broadcasting under network/budget failure |

---

## 10. CLI Command Reference

| Command | Description |
| :--- | :--- |
| `tgs shield scan [PATH]` | Recursively scans a codebase or directory for APT signatures, supply chain backdoors, and prompt injections. |
| `tgs shield audit <COMMAND>` | Pre-execution safety audit of a shell command or script payload. |
| `tgs shield status` | Displays active defense profile, protected asset registry, and security telemetry. |
| `tgs repl --agent security-auditor` | Starts an interactive security analysis session with the specialized security-auditor agent. |
| `tgs agent "<prompt>"` | Executes an autonomous security investigation and self-healing patch loop. |
| `tgs ecc audit [PATH]` | Audits repository compliance against Enterprise Craftsmanship guidelines and security rules. |
