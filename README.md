<div align="center">
  <a href="https://github.com/CharleGutierrez/tagisan">
    <img src="assets/tagisan_logo.png" alt="Tagisan Logo Emblem" width="360" />
  </a>

  # 🇵🇭 Tagisan (`tgs` / `tagisan-rs`)

  ### The `uv` of Multi-Agent AI Swarms & In-Process Tensor Engines in Systems-Grade Rust

  **Tagisan ng Talino:** In-Process GGUF Tensor Engine • Standalone Tokio Daemon (`tgs serve`) • 3s Anti-Timeout Heartbeat • Self-Healing Compiler (`tgs autofix`) • AST Codebase Graph (`tgs graph`) • Dialectical Debate (`Lakandiwa`) • Mixture-of-Agents • 5 Colibrì Superpowers • AgentShield Cyber Defense • 56 MS Copilot Tools • 10 Enterprise Pillars • 2,063+ Compiled Skills • Polyglot Sandboxes • Bidirectional MCP

  <br />

  [![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
  [![Tokio](https://img.shields.io/badge/async-tokio%201.43-blue?style=for-the-badge&logo=tokio)](https://tokio.rs/)
  [![Binary](https://img.shields.io/badge/binary-20%20MB%20static-brightgreen?style=for-the-badge&logo=rust)](https://github.com/CharleGutierrez/tagisan)
  [![CLI Commands](https://img.shields.io/badge/CLI-37%20subcommands-yellow?style=for-the-badge&logo=gnu-bash)](https://github.com/CharleGutierrez/tagisan)
  [![MS Copilot](https://img.shields.io/badge/MS%20Copilot-56%20enterprise%20tools-0078D4?style=for-the-badge&logo=microsoft)](https://github.com/CharleGutierrez/tagisan)
  [![Enterprise Pillars](https://img.shields.io/badge/pillars-10%20enterprise%20engines-purple?style=for-the-badge&logo=azure)](https://github.com/CharleGutierrez/tagisan)
  [![Compiled Skills](https://img.shields.io/badge/skills-2%2C063%2B%20compiled%20(4%2C600%2B%20catalog)-red?style=for-the-badge&logo=git)](https://github.com/CharleGutierrez/tagisan)
  [![AgentShield](https://img.shields.io/badge/AgentShield-Lazarus%20%26%20APT38%20EDR-darkred?style=for-the-badge&logo=security)](https://github.com/CharleGutierrez/tagisan)
  [![Zero Ambient Authority](https://img.shields.io/badge/security-Zero%20Ambient%20Authority-blueviolet?style=for-the-badge&logo=shield)](https://github.com/CharleGutierrez/tagisan)
  [![GGUF Engine](https://img.shields.io/badge/GGUF-mmap%2056%C2%B5s-blue?style=for-the-badge&logo=speedtest)](https://github.com/CharleGutierrez/tagisan)
  [![Ollama](https://img.shields.io/badge/daemon-native%20tokio%20%3A11434-black?style=for-the-badge&logo=server)](https://ollama.com/)
  [![Polyglot](https://img.shields.io/badge/runtimes-Bun%20%7C%20Python%20%7C%20Perl%20%7C%20OTP-brightgreen?style=for-the-badge)](https://github.com/CharleGutierrez/tagisan)
  [![MCP](https://img.shields.io/badge/MCP-Bidirectional%20Client%20%2B%20Server-9cf?style=for-the-badge)](https://modelcontextprotocol.io/)
  [![Brutal Tests](https://img.shields.io/badge/tests-84%20brutal%20suites%20(%3E300k%20ops%2Fsec)-success?style=for-the-badge&logo=testing-library)](https://github.com/CharleGutierrez/tagisan)
  [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=for-the-badge)](LICENSE)
</div>

---

> **Why compromise between Python dependency hell, bloated agent frameworks, and multi-gigabyte AI runtimes?**  
> Tagisan (`tgs`) is a single, zero-dependency **20 MB static binary** written in systems-grade Rust that cold-boots in **~2.1 milliseconds**, operates inside **10 MB of RAM**, parses 2+ GB GGUF model weights in **56 microseconds**, and coordinates heterogeneous local models and frontier cloud APIs (Claude 3.5, Gemini 2.0, GPT-4o, DeepSeek-R1, Grok-2) into dialectical swarms, self-healing compiler loops, and enterprise-governed workflows with **$0.00 API cost**.

```bash
# ⚡ Install Tagisan in 5 seconds (Linux / macOS / WSL)
curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh

# 📦 Or build and install directly with Cargo (Windows / Linux / macOS)
cargo install --path .

# 🚀 Verify instantaneous sub-millisecond boot
tgs status
```

---

## ⚡ Definitive Benchmark Comparison Matrix

| # | Technical Metric / Capability | **Tagisan (`tgs`)** | **Ollama (Go)** | **CrewAI** | **AutoGen** | **LangGraph** |
| :-: | :--- | :--- | :--- | :--- | :--- | :--- |
| **1** | **Implementation Language** | **Systems-Grade Rust (Tokio 1.43)** | Go + C++ (llama.cpp) | Python 3.10+ | Python 3.10+ | Python / TypeScript |
| **2** | **Packaging & Binary Size** | **Single 20 MB Static Binary** | 50+ MB Binary + deps | 140+ pip packages | 95+ pip packages | Heavy node/pip graphs |
| **3** | **Cold Startup Time** | **~2.1 ms** | ~450 ms | ~1,850 ms | ~1,400 ms | ~900 ms |
| **4** | **Idle Memory Footprint** | **~10 MB - 18 MB RAM** | ~110 MB RAM | ~480 MB RAM | ~520 MB RAM | ~350 MB RAM |
| **5** | **GGUF Mapping Latency** | **56 µs (0.056 ms via `memmap2`)** | 100 - 300 ms | N/A | N/A | N/A |
| **6** | **Native Local Server Daemon** | **Yes (`tgs serve` Tokio :11434)** | Yes | No | No | No |
| **7** | **Anti-Timeout Heartbeat Pulse** | **Yes (3s Keepalive Pulse)** | No (Prone to 600s drop) | No | No | No |
| **8** | **Self-Healing Compiler** | **Yes (`tgs autofix` Polyglot)** | No | No | No | No |
| **9** | **AST Codebase Knowledge Graph** | **Yes (`tgs graph` Petgraph DAG)** | No | No | No | No |
| **10** | **Live Desktop COM / ROT Bridge** | **Yes (`MsDesktopRuntimeBridge`)** | No | No | No | No |
| **11** | **Fabric OneLake Delta Engine** | **Yes (DirectLake TMDL Sync)** | No | No | No | No |
| **12** | **Azure Service Bus AMQP 1.0** | **Yes (Pure-Rust AMQP Frames)** | No | No | No | No |
| **13** | **Zero-Cost Local Inference** | **Yes ($0.00 Infinite Run)** | Yes | Cloud API dependent | Cloud API dependent | Cloud API dependent |
| **14** | **Host Security EDR & Sandbox** | **Native AgentShield (Zero Ambient Authority)** | None | None | Docker requirement | None |
| **15** | **Nation-State APT Mitigation** | **Yes (Lazarus / APT38 Intercept)** | No | No | No | No |
| **16** | **Pre-Execution Shell Auditing** | **Yes (AST & Regex Intercept `tgs shield`)**| No | No | No | No |
| **17** | **Real-Time Forensic Alerts** | **Yes (ANSI Banners & WinRT Toast)** | No | No | No | No |
| **18** | **Model Context Protocol (MCP)** | **Bidirectional (Client + Server)** | None | Client only | Community wrappers | Community wrappers |

---

## 🏗️ Comprehensive System Architecture

```mermaid
flowchart TD
    subgraph UI["Access, IDE & Orchestration Layer"]
        CLI["CLI Binary (tgs) — 37 Commands"]
        TUI["Ratatui Terminal Dashboard & REPL"]
        IDE["IDE LSP & Extension Bridge (VS Code, Cursor, Zed)"]
        Teams["Microsoft Teams & Copilot Studio Gateway"]
    end

    subgraph CoreEngine["Tagisan Systems Core Engine (Tokio 1.43)"]
        Budget["Token Budget & Prompt Cache Tracker ($5.00 Cap / 90% Discount)"]
        Heartbeat["Anti-Timeout 3s Heartbeat Pulse Generator"]
        NotifyHub["Real-Time Forensic Notification Hub (ANSI & WinRT Toast)"]
        ToolReg["Dynamic Tool Registry (110+ Tools)"]
    end

    subgraph TensorDaemon["In-Process GGUF Tensor Engine & Local Daemon"]
        Mmap["memmap2 Zero-Copy Parser (56µs Header & 256 Tensors)"]
        Resolver["OllamaBlobResolver (~/.ollama/models/blobs)"]
        Server["tgs serve (Tokio HTTP Daemon on :11434 / :11435)"]
        Abliterate["Unrestricted / Refusal-Orthogonalized Execution Engine"]
    end

    subgraph SwarmStrategies["Multi-Agent Reasoning & Swarm Orchestration"]
        Debate["Dialectical Debate (Thesis -> Antithesis -> Lakandiwa Synthesis)"]
        MoA["Mixture-of-Agents (Parallel Proposers -> Aggregator)"]
        Harmony["Structured Harmony Assembly Line (RFC-003: Arch -> Impl -> QA -> Doc)"]
        Consensus["Consensus Engine (Majority, Borda Count, Unanimous)"]
        DAG["Petgraph Directed Asynchronous Graph Execution Engine"]
        ColibriP2P["Distributed Local P2P Cluster Mesh (Port 8765)"]
    end

    subgraph CodeIntelligence["Code Intelligence, Compiler & Verification"]
        Autofix["Self-Healing Compiler (tgs autofix: Rust, TS, Py, Go)"]
        GraphEngine["AST Codebase Knowledge Graph & Blast Radius (tgs graph)"]
        Grounding["Deterministic Closed-Loop Grounding (tgs ground)"]
        CRUD["Surgical File CRUD & Git Worktree Sandboxing"]
    end

    subgraph CyberDefense["Autonomous Cyber Defense (AgentShield & tgs shield)"]
        ZeroAuth["Zero Ambient Authority Gate (Solana, ETH, BTC, AWS, SSH Isolation)"]
        APT38["Nation-State APT Mitigation (Lazarus, Sockets, Download Cradles)"]
        PromptArmor["Indirect Prompt Injection Armor & Homoglyph Normalizer"]
    end

    subgraph EnterpriseCopilot["Microsoft 365 Copilot Subsystem (56 Registered Tools)"]
        subgraph Strengthened["5 Strengthened Enterprise Engines"]
            COM["Live Desktop Bridge (Named Pipe, ROT, ShapeSheet, Undo VBA)"]
            OneLake["Fabric OneLake Delta Engine (ABFS, DirectLake TMDL Sync)"]
            A2A["Copilot Studio Agent-to-Agent (A2A) Swarm Engine"]
            Dataverse["Dataverse CDC & Solution ALM Packager (.zip <-> XML)"]
            CAE["Continuous Access Evaluation (CAE) Zero-Trust Guard"]
        end
        subgraph Frontier["5 Frontier Enterprise Engines"]
            SecCopilot["Security Copilot & Intune Device Posture (BitLocker, DHA)"]
            TeamsMedia["Teams Real-Time Media Calling Audio Swarm & VAD"]
            EntraID["Entra Verified ID Cryptographic W3C DID Credentials"]
            AMQP["Azure Service Bus AMQP 1.0 Streaming Engine"]
            CFBF["Legacy OLE Compound Document Binary Forensics Engine"]
        end
    end

    subgraph PolyglotRuntimes["Polyglot Sandboxed Runtimes & Domain OS"]
        BunRuntime["Bun / TypeScript (Eval, Test, Standalone Binary Compile)"]
        PyRuntime["Python 3 (uv, pip, pytest, mypy)"]
        PerlRuntime["Perl 5 (Regex text forensics, cpanm)"]
        VellaOS["Vella Sovereign OS (SCADA, Robotics, FHE TFHE, Orbital)"]
        ActorOTP["OTP / BEAM & Gleam Actor Subsystems"]
    end

    subgraph SkillsMemory["Curated 2,063+ Skills & JIT Paging Memory"]
        PILOT["Colibrì PILOT: 3-Tier JIT Memory (L1 Context / L2 RAM / L3 NVMe)"]
        Canons["Authoritative Canons (Visio, Access, Office, PowerPlatform, Vibe Books)"]
        Personas["13 Curated Agent Personas (.ecc/agents/)"]
        MCPHub["Bidirectional Model Context Protocol (tgs mcp / tgs serve-mcp)"]
    end

    CLI --> Budget
    Budget --> Mmap
    Budget --> Debate
    Budget --> Autofix
    Budget --> ZeroAuth
    Budget --> COM
    Budget --> BunRuntime
    Budget --> PILOT
```

---

## 🔬 Core Engine Superpowers (Deep Dives)

### 1. 🦀 Zero-Copy GGUF Tensor Engine & Standalone Daemon (`tgs serve`)
Tagisan reverse-engineers the best elements of Ollama and rebuilds them directly into systems-grade asynchronous Rust (`src/engine/gguf.rs`, `src/engine/server.rs`):
- **Zero-Copy GGUF v2/v3 Parser:** Employs `memmap2` to map multi-gigabyte weights directly into operating system page cache memory. Parses binary magic (`0x46554747`), file version, tensor descriptors, metadata key-values, and 256 tensor structures in **56 microseconds (0.056 ms)** with zero heap duplication.
- **`OllamaBlobResolver`:** Crawls `~/.ollama/models/manifests/` and instantly maps human model aliases (`"abliterated"`, `"llama3.2-abliterate:3b-instruct"`) to physical content-addressed weight blobs (`sha256-fdc5784e...`) without downloading or duplicating weights.
- **Standalone Tokio HTTP Daemon (`tgs serve`):** Drop-in, Ollama-compatible HTTP daemon written in pure Tokio. Serves `/api/version`, `/api/tags`, `/api/show`, and `/api/chat` over streaming NDJSON. Runs on **~10 MB RAM** with zero garbage collection pauses. Cursor, VS Code Continue, OpenWebUI, and Obsidian connect out of the box!
- **Deep Model Inspection (`tgs engine inspect <model>`):** Directly extracts GGUF architecture, context lengths, embedding dimensions, block counts, attention heads, tokenizer chat templates, and quantization profiles.

### 2. 💓 Anti-Timeout 3s Heartbeat Pulse & Uncensored Abliterated Mode
- **The 3-Second Heartbeat Pulse:** On dual-core or power-constrained laptops (e.g., Intel Core i3 7th Gen), CPU prompt ingestion of large repos can take 30–60 seconds before emitting the first token. Standard HTTP clients disconnect with `TimedOut`. Tagisan automatically interleaves non-breaking SSE / NDJSON keepalive heartbeats every 3 seconds, sustaining connections indefinitely.
- **Auto-Sequential Concurrency Scheduling (`concurrency_limit = 1`):** High-load local CPU inference is automatically scheduled sequentially to eliminate L1/L2 cache thrashing and prevent 100% RAM/swap system freezes.
- **Unrestricted / Refusal-Orthogonalized Mode:** Full native support for abliterated models (`llama3.2-abliterate:3b-instruct`). Mathematical refusal directions are removed from residual activations, allowing security teams to audit live exploit payloads, analyze proprietary binary protocols, and perform offensive red-team forensics without corporate refusals.

### 3. ⚔️ Dialectical Debate (*Tagisan ng Talino / Lakandiwa*)
Single-agent prompt workflows suffer from hallucinations and blindspots. Tagisan implements the classical Filipino parliamentary debate format (*Balagtasan / Tagisan ng Talino*):
- **Round 1 (Thesis):** Proponent model proposes an architecture, proof, or code patch.
- **Round 2 (Antithesis):** Adversary model probes for race conditions, memory leaks, security vulnerabilities, and edge cases.
- **Round 3 (Lakandiwa Synthesis):** The Chief Adjudicator synthesizes both arguments, balances engineering trade-offs, and issues a mathematically binding verdict with verified invariant constraints.
- **100% Offline Multi-Model Rotation:** Rotate local quantized models across rounds with **$0.00 cloud cost**.

### 4. 🛖 Mixture-of-Agents (MoA) & Parallel Proposers
- **Layer 1 (Parallel Proposers):** Broadcasts prompts concurrently across diverse local and cloud models over lock-free Tokio channels.
- **Layer 2 (Master Aggregator):** The designated lead model scores, filters, and synthesizes the best components of each proposal into a superior composite solution.

### 5. 🦅 The 5 Colibrì Superpowers
Adapted from high-performance local MoE systems:
1. **Skill JIT Paging & Lookahead Prefetch Engine (PILOT):** 3-tier memory model (**L1** Active Context ↔ **L2** Warm RAM AST ↔ **L3** Cold NVMe) with 1-step lookahead prefetching based on planned steps and heat-weighted pinning.
2. **Live Cortex Swarm Atlas (`tgs swarm atlas`):** Real-time dashboard tracking empirical agent invocations, decay heat, and domain topic clustering (Systems, Forensics, Strategy, Security).
3. **Dual-SSD Model Weight Striping:** Supports streaming model weights across dual NVMe drives (`COLI_MODEL_MIRROR`, `COLI_DISK_WEIGHTS`) up to **~14.8 GB/s** throughput.
4. **Distributed Local P2P Cluster Mesh (`tgs swarm cluster`):** Tokio TCP coordinator and worker mesh on port `8765` distributing tool and skill tasks across local LAN compute nodes.
5. **Semantic Invariant Guard:** Enforces *"No SLA on Speed, Hard Guarantee on Semantics"*—rejects lossy tool schema truncations and guarantees complete compiler diagnostic preservation under context exhaustion.

### 6. 🔧 Self-Healing Compiler & TDD Healer (`tgs autofix`)
Tagisan delivers an autonomous, zero-prompt compiler diagnostic healer and test repair engine (`src/engine/autofix.rs`):
- **Polyglot Compiler Diagnostics:** Ingests machine-readable JSON compiler streams (`cargo check --message-format=json`), TypeScript compiler output (`tsc`), Python syntax errors and test failures (`py_compile` & `pytest`), and Go compiler diagnostics (`go vet`).
- **Surgical Span & AST Replacement:** Computes precise 1-based character/byte boundary offsets in source files. Applies compiler-recommended fixes (`MachineApplicable`), auto-prefixes unused symbols with underscores, and repairs syntax errors without modifying surrounding code.
- **Atomic Snapshots & Rollback:** Automatically captures `.bak` file snapshots prior to patch application, guaranteeing zero code degradation.
- **Iterative TDD Healing Loop:** Continuously loops diagnosis, patch application, and verification passes (up to `--max-attempts N`) until the target codebase or test suite compiles with 0 errors.

### 7. 🌲 AST Codebase Knowledge Graph & Blast-Radius Engine (`tgs graph`)
- **Polyglot AST Extraction:** Indexes Rust (`.rs`), Python (`.py`), TypeScript/JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`), and Go (`.go`) across the entire repository using parallel Rayon threads.
- **Directed Graph Topology:** Extracts functions, structs, traits, interfaces, enums, modules, and call sites. Models relationships via `Calls`, `Defines`, `Implements`, `Imports`, and `References` using directed graphs (`petgraph`).
- **Sub-Millisecond Symbol Navigation:** Instantly resolves symbol definitions, documentation, signatures, and incoming/outgoing call hierarchies across thousands of files.
- **Transitive Blast-Radius Risk Analysis:** Calculates the entire ripple effect of refactoring a symbol and classifies risk level (`LOW`, `MEDIUM`, `HIGH`, `CRITICAL`), providing actionable refactoring advice.

### 8. ⚖️ Deterministic Closed-Loop Grounding (`tgs ground`)
- **Closed-Loop Verification Engine:** Prevents LLM hallucinations by coupling prompt objectives with AST invariant extraction, strict dialectical critique, and post-generation compile passes.
- **Deterministic Code Emittance:** Ensures code output satisfies pre-established architectural invariants before writing to disk.

### 9. 🛠️ First-Class Surgical File CRUD & Git Worktree Sandboxing
- **`edit_file` (Surgical Patcher):** Replaces exact substrings or line ranges (`start_line`, `end_line`) without rewriting entire multi-thousand-line files. Includes atomic temporary writes and automated `.bak` backups.
- **`delete_file` (Safe Recycler):** Features soft-delete recycling into `.tagisan/trash/<timestamp>_<file>` with cross-filesystem fallbacks and permanent deletion flags.
- **`list_dir` (Structured Explorer):** Formats directory trees with entry types (`[FILE]`, `[DIR]`, `[SYMLINK]`), exact byte sizes, child counts, traversal depth limits (`max_depth`), and pattern filters.
- **`read_file` & `write_file`:** Safe async reading and automatic parent directory creation.
- **Git Worktree Sandboxing (`--sandbox`):** Branches tasks into ephemeral worktrees (`tagisan/sandbox-...`), allowing safe diff inspection (`sandbox.diff()`), committing, or clean rollbacks.

### 10. 👁️ Multimodal Computer Vision
- **Direct CLI Ingestion (`-i / --image`):** Pass screenshots, architecture diagrams, UI mockups, or error traces directly to vision-capable models (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`).
- **Autonomous Agent Inspection (`view_image`):** Agents autonomously inspect local image files, validate formats, compute dimensions, and feed Base64 payloads into multimodal reasoning loops.
- **Broad Model Support:** Compatible with Google Gemini 2.0 Flash / Pro, Anthropic Claude 3.5 Sonnet, OpenAI GPT-4o, Grok-2 Vision, and lightweight local models (`moondream:1.8b`, `llava-phi3:3.8b`).

### 11. 🔌 Universal Model Context Protocol (MCP Client & Server)
- **Universal MCP Client (`tgs mcp`):** Connects via JSON-RPC 2.0 stdio to any MCP server (PostgreSQL, SQLite, GitHub, Filesystem, Memory) with namespace isolation (`<server>__<tool>`).
- **Native MCP Server (`tgs serve-mcp`):** Exposes Tagisan's swarms, dialectical debate, consensus engines, and codebase tools to **Claude Desktop**, **Cursor**, **Zed**, and **Windsurf**.

### 12. 💰 Token Budget & Prompt Cache Tracker
- **Global Session Budget Cap (`--max-budget 5.00`):** Enforces a strict default session expenditure limit (e.g. $5.00 USD). Halts execution and notifies the user before accidental cloud overages occur.
- **90% Prompt Cache Discount Accounting:** Accurately tracks Anthropic and OpenAI prompt cache reads, modeling true API expenditures down to four decimal places.
- **Automatic Budget Evacuation:** Can automatically evacuate ongoing cloud swarms to zero-cost local Ollama models if expenditure limits are approached (`--evacuate-on-budget`).

---

## 🛡️ Autonomous Cyber Defense & Threat Hunting (`tgs shield`)

> **"Assume Breach at the Agent Layer."**  
> Modern developer workstations and autonomous agent workflows are primary targets for nation-state advanced persistent threats (**Lazarus Group**, **APT38**, **TraderTraitor**, **Kimsuky**) and autonomous **Rogue AI Cyberwarfare Payloads** launching cognitive jailbreaks, synthetic tool-call spoofing, covert DNS exfiltration, and in-memory reflection attacks. Tagisan equips security engineers, DevSecOps teams, and SOC analysts with a **systems-grade, prompt-driven cyber defense workstation** built directly into the 20 MB static Rust binary.

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
   [ APT38 / Lazarus ]         [ Supply Chain Poison ]       [ Rogue AI Cyberwarfare ]
   • /dev/tcp/ C2 Sockets      • package.json postinstall    • Synthetic tool injections
   • powershell -enc cradles   • build.rs socket backdoors   • DNS / HTTP POST tunnels
   • certutil -urlcache        • base64 encoded droppers     • AMSI bypass & reflection
   • Web3 wallet harvesting    • living-off-the-land scripts • Zero-width / Homoglyphs
           │                             │                            │
           └─────────────────────────────┼────────────────────────────┘
                                         │
                         [ ZERO AMBIENT AUTHORITY CHECK ]
                   Checks target against sensitive asset registry:
                  • Solana (~/.config/solana/id.json)
                  • Ethereum (~/.ethereum/keystore)
                  • Bitcoin (wallet.dat) & MetaMask vaults
                  • Cloud (~/.aws, ~/.config/gcloud, .env, id_rsa)
                                         │
                          ┌──────────────┴──────────────┐
                          ▼                             ▼
                     [ CRITICAL ]                  [ PASS ]
              🚨 High-Vis ANSI Warning          ⚡ Safe Sub-Millisecond
              🔔 Desktop Toast Notification        Execution Flow
              🛑 Execution Terminated
```

### 🥷 Threat Vector Mitigation Capabilities

1. **Zero Ambient Authority Enforcement:** Hard-blocks any agent access to cryptocurrency wallets (`id.json`, `keystore`, `wallet.dat`, MetaMask) and cloud credentials (`~/.aws`, `~/.config/gcloud`, `~/.azure`, `~/.kube`, `~/.ssh/id_rsa`, `.env`), regardless of agent prompt permissions.
2. **Nation-State Reverse Shell Interception:** Detects and blocks Unix `/dev/tcp/`, `/dev/udp/` pseudo-device sockets, netcat reverse shells (`nc -e`, `ncat`, `socat`), and language socket spawns (`python -c "import socket..."`, `perl -e 'use Socket;'`).
3. **Living-off-the-Land (LotL) Download Cradle Quarantine:** Intercepts Base64-encoded PowerShell payloads (`powershell -enc`), in-memory cradles (`IEX DownloadString`), and transfer abuse (`certutil -urlcache`, `bitsadmin`, `mshta`).
4. **Indirect Prompt Injection Armor:** Strips invisible zero-width Unicode characters (`\u{200b}`, `\u{200c}`, `\u{200d}`), normalizes Cyrillic/Greek homoglyphs, and neutralizes injection anchors (`[INST]`, `<!-- SYSTEM:`, DAN jailbreaks).
5. **Real-Time Notification Hub:** High-contrast ANSI warning banners on `stderr` and cross-platform desktop notifications (Windows WinRT Toast / Linux `notify-send`) alert analysts to threat vectors in real time.

---

## 🏢 Microsoft 365 Copilot & Enterprise Tech Stack (`tgs copilot`)

Tagisan includes an enterprise-hardened integration connecting its dialectical reasoning engine, petgraph AST codebase graph, and formal invariant verification directly into the **Microsoft 365 Enterprise Ecosystem** (Teams, SharePoint, OneDrive, Outlook, Microsoft Search, Excel, Planner, and Copilot Studio).

```mermaid
flowchart TD
    subgraph M365["Microsoft 365 Enterprise Ecosystem"]
        Teams["Microsoft Teams / Bot Webhook"]
        Copilot["M365 Copilot Studio / Declarative Agent"]
        SharePoint["SharePoint & OneDrive"]
        Planner["Microsoft Planner & To-Do"]
        Sentinel["Microsoft Sentinel / Azure Monitor SIEM"]
        AdminCenter["M365 Admin Center / Marketplace"]
    end

    subgraph AuthLayer["Enterprise Identity & Governance"]
        Entra["Entra ID / MSAL"]
        OBO["On-Behalf-Of (OBO) JWT Bearer Flow"]
        X509["X.509 Client Certificate Assertion"]
        Purview["Microsoft Purview Dynamic Label Taxonomy"]
    end

    subgraph TagisanCore["Tagisan Copilot Subsystem (tgs)"]
        Throttler["429 Adaptive Throttler & Token Bucket"]
        Webhook["Graph Webhook Subscription Lifecycle Engine"]
        AgentShield["AgentShield Enterprise DLP Gate"]
        SIEM["Sentinel CEF / RFC 5424 SIEM Telemetry Bridge"]
        Attestation["App Compliance & Attestation Engine"]
        Tools56["56 Autonomous Copilot MCP Tools"]
    end

    Copilot -->|"User JWT"| OBO
    OBO -->|"Graph Bearer Token"| Throttler
    X509 -->|"Client Assertion"| Entra
    Throttler -->|"Rate-Limited API Requests"| Teams
    Throttler -->|"Ingestion & Sanitization"| SharePoint
    Webhook -->|"Validation Handshake (under 10s)"| Teams
    SIEM -->|"CEF / RFC 5424 Security Events"| Sentinel
    Purview -->|"GUID Policy Binding"| AgentShield
    Attestation -->|"compliance.json & Attestation"| AdminCenter
```

### 🚀 Core Enterprise Scenarios & Autonomous Pipelines

1. **Microsoft Purview Sensitivity & Zero-Egress Air-Gapping (`tgs copilot purview` / `copilot_purview_guard`):**
   - Automatically classifies data into `General`, `Confidential`, `HighlyConfidential`, and `Secret` sensitivity labels.
   - When sensitivity is `Confidential` or `HighlyConfidential`/`Secret`, dynamically enforces **Zero-Egress Air-Gapped mode**: strictly routes tasks to Tagisan's local in-process GGUF/offline tensor engine, completely forbidding external cloud API calls.
   - Issues cryptographic **SHA-256 audit receipts** (`PurviewAuditReceipt`) with content digest, timestamp, sensitivity classification, routing enforcement proof, and HMAC/signature.

2. **Architecture Decision Record (ADR) Sync to OneNote & SharePoint (`tgs copilot adr` / `copilot_adr_sync`):**
   - Synthesizes dialectical debate verdicts and technical RFCs into standard Markdown Architectural Decision Records (**MADR 3.0 format**): Title, Status, Deciders, Context, Considered Options, Decision Outcome, Formal Invariants, and Consequences.
   - Automatically synchronizes ADRs to **Microsoft OneNote notebooks** (`sync_onenote_page`) and **SharePoint document libraries / wikis** (`upload_sharepoint_file`) via `GraphClient`.

3. **Direct Git Branch & PR Automation (`tgs copilot pr` / `copilot_create_pr`):**
   - Ingests synthesized patches from `meeting-to-code`, creates an ephemeral Git branch (`tgs/m2c-...`), and generates conventional commit messages.
   - Formats comprehensive GitHub and Azure DevOps PR markdown descriptions with embedded **Adaptive Card v1.5 blast-radius telemetry**, risk assessments, and file diffs.
   - Automatically dispatches PR notification cards to Microsoft Teams channels or chats.

4. **Interactive Teams Bot Webhook Action Handler (`tgs copilot listen` / `src/copilot/bot.rs`):**
   - Processes interactive Adaptive Card `Action.Submit` callbacks from Teams users directly into Tagisan:
     - `approve_patch`: Approves and merges ephemeral patch, generating commit SHA and updating card state.
     - `run_autofix`: Dispatches Tagisan iterative self-healing autofix engine on target files.
     - `run_debate`: Initiates dialectical debate on the card's proposal and displays Lakandiwa synthesis.
     - `sync_adr`: Synthesizes MADR architecture decision record and syncs to OneNote & SharePoint.
   - Returns refreshed Adaptive Card state with visual confirmation badges and audit trails.

5. **Executive Presentation Deck Generator (`tgs copilot deck` / `copilot_export_deck`):**
   - Compiles responsive, presentation-ready 5-slide executive briefing decks (in Fluent UI HTML and Marp Markdown format) covering:
     1. Executive Summary & KPI metrics
     2. High-Risk Codebase Blast Hotspots
     3. Dialectical Invariants formally verified
     4. Cost Savings of Local Compute ($18,450/mo savings via offline tensor inference)
     5. AgentShield Compliance Clearance & Purview Zero-Egress certification
   - Supports direct export to disk and dispatch via Outlook email.

6. **Meeting-to-Code Pipeline (`tgs copilot meeting-to-code` / `copilot_meeting_to_code`):**
   - Ingests raw or live Microsoft Teams meeting transcripts via Microsoft Graph.
   - Automatically decomposes dialogue into prioritized engineering action items (`High`/`Medium`/`Low`), assigning owners and categories.
   - Maps each task to concrete symbols in the codebase and computes the **AST transitive blast radius** and ripple-effect risk level.
   - Synthesizes automated, surgical code patches and diffs with **AgentShield DLP security clearance**, and dispatches execution plans to Teams.

7. **Codebase Telemetry & Blast Radius Cards (`tgs copilot blast-report` / `copilot_blast_radius_report`):**
   - Evaluates the transitive call graph and architectural depth for any struct, function, method, or trait.
   - Emits **Microsoft Teams Adaptive Card v1.5 JSON** with interactive action buttons and color-coded risk indicators (`LOW`, `MEDIUM`, `HIGH`, `CRITICAL`).
   - Produces executive **Microsoft Fluent UI HTML reports** optimized for Outlook, PowerPoint, and Excel.

8. **Dialectical Debate Dispatch (`tgs copilot debate` / `copilot_debate_dispatch`):**
   - Orchestrates Tagisan's 3-round adversarial debate on technical RFCs and architecture decisions:
     - **Round 1 (Thesis):** Rigorous proposal detailing throughput, zero-copy safety, and latency wins.
     - **Round 2 (Antithesis):** Adversarial attack on edge cases, lock contention, token invalidation, and memory overhead.
     - **Round 3 (Lakandiwa Synthesis):** Definitive binding consensus and formal invariant constraints.
   - Enforces pre-flight AgentShield DLP scanning and broadcasts the verdict directly to Microsoft Teams channels or Outlook stakeholders.

9. **Microsoft Search Graph Connector (`tgs copilot index`):**
   - Registers external connection schema (`tagisan_enterprise_index`) with searchable, queryable, and retrievable properties.
   - Ingests 495+ built-in engineering skills, debate transcripts, and architecture diagrams into Microsoft Search so users can query Tagisan intelligence natively within Microsoft 365 Copilot.

10. **Declarative Agent Manifest & Plugin Generator (`tgs copilot package` / `plugin`):**
    - Emits a complete, sideloadable Teams App package bundle:
      - `manifest.json` (Teams App manifest v1.16)
      - `declarativeAgent.json` (Microsoft Copilot Declarative Agent v1.0)
      - `ai-plugin.json` (Copilot Studio & ChatGPT Plugin schema)
      - `openapi.json` (OpenAPI 3.0.3 specification exposing all 16 endpoints)
      - `color.png` & `outline.png` (RFC 2083 valid binary PNG icons generated in-memory)

11. **Native Excel Custom Functions Engine & Add-in Packager (`tgs copilot excel` / `copilot_excel_functions`):**
    - Evaluates native Excel formula expressions dynamically directly against the Tagisan engine:
      - `=TGS.BLAST_RADIUS(symbol, path)`: Calculates transitive affected symbol count and risk tier.
      - `=TGS.COMPLEXITY(symbol, path)`: Computes AST cyclomatic complexity and risk rating.
      - `=TGS.COST_SAVINGS(prompt_tokens, completion_tokens)`: Models API cost avoidance vs OpenAI GPT-4o pricing.
      - `=TGS.INVARIANT_CHECK(target, code)`: Formally verifies mathematical and architectural invariants.
    - Generates complete Office Add-in packages containing `manifest.xml`, `functions.json` schema, and TypeScript bridge `functions.js`.

12. **Live Server-Sent Events (SSE) Streaming Gateway (`tgs copilot stream` / `copilot_stream_gateway`):**
    - Real-time event streaming protocol for Copilot Studio, Teams bots, and webhooks formatted as standard `text/event-stream` or NDJSON.
    - Dispatches streaming frames (`round_start`, `token`, `keepalive`, `verdict`, `done`) with 3-second anti-timeout heartbeat pulses, ensuring zero connection drops during heavy reasoning workloads.

13. **Microsoft Planner & To-Do Task Synchronizer (`tgs copilot planner` / `copilot_planner_sync`):**
    - Synchronizes meeting action items, debate outcomes, and architectural tasks directly to **Microsoft Planner** plan buckets or **Microsoft To-Do** personal task lists.
    - Supports setting priorities (`Urgent`, `Important`, `Medium`, `Low`), checklist items, assignees, and linking back to Git commit SHAs and pull requests.

14. **Teams "@Tagisan" CI/CD Incident Debugger (`tgs copilot incident` / `copilot_incident_debugger`):**
    - Ingests raw CI/CD failure logs (rustc errors, stack trace panics, TypeScript diagnostics, pytest failures), extracts root cause lines, and computes symbol blast radius.
    - Synthesizes automated unified diff patches with confidence scores and emits interactive **Adaptive Cards v1.5** featuring an actionable `[Apply Autofix & Rerun CI]` `Action.Submit` button.

15. **Windows Copilot+ PC Hardware Telemetry & Datacenter Energy Efficiency (`tgs copilot hardware` / `copilot_hardware_telemetry`):**
    - Detects on-device silicon acceleration (NPU, DirectML GPUs, AVX-512/NEON SIMD) and calculates TOPS capability and local energy consumption (Joules & kWh).
    - Benchmarks local inference against 8x H100 datacenter clusters, modeling dollars saved and grams of CO2 emissions avoided.
    - Computes a cryptographic **Data Sovereignty Score (100% Air-Gapped)** and generates an executive Fluent UI Adaptive Card.

16. **Entra ID On-Behalf-Of (OBO) Flow & Certificate Assertions (`tgs copilot obo` / `copilot_obo_exchange`):**
    - RFC 7523 OAuth2 JWT bearer token exchange preserving user security context (UPN, OID, tenant, roles, scopes) across downstream Microsoft Graph calls.
    - X.509 client certificate assertion with SHA-1/SHA-256 thumbprints for zero-password production daemon authentication.

17. **Microsoft Graph Webhook Subscriptions & Handshake Engine (`tgs copilot subscribe` / `copilot_subscription_manage`):**
    - High-speed validation challenge handler responding to `validationToken` in under 10 seconds.
    - Full lifecycle management (create, renew, delete, list) for online meetings, SharePoint drives, and Teams chats with HMAC-SHA256 `clientState` signature verification.

18. **Microsoft Graph 429 Adaptive Throttling & Token Bucket Rate Limiting:**
    - Per-resource token bucket rate limiters intercepting HTTP 429 status codes.
    - Parses integer, float, and RFC 2822/3339 `Retry-After` headers with truncated exponential backoff and full jitter to eliminate thundering herds.

19. **Dynamic Microsoft Purview Sensitivity Taxonomy Synchronization (`tgs copilot sync-labels` / `copilot_purview_sync`):**
    - Queries `/informationProtection/policy/labels` to synchronize tenant custom sensitivity labels and GUIDs directly to local Zero-Egress air-gap policies.

20. **Microsoft Sentinel CEF, RFC 5424 & Azure Monitor SIEM Telemetry Bridge (`tgs copilot sentinel` / `copilot_sentinel_audit`):**
    - Structured security event telemetry formatted in Common Event Format (CEF:0), RFC 5424 Syslog, and Azure Monitor DCR custom log formats.
    - Emits real-time audit events for AST blast-radius calculations, DLP blocks, Purview air-gap triggers, and autonomous PR commits.

21. **M365 Admin Center App Compliance & Publisher Attestation (`tgs copilot certify` / `copilot_certify`):**
    - Automated compliance generator emitting `compliance.json` for Microsoft Partner Center certification.
    - Attests MPN ID, valid domains, SOC 2 Type II, ISO/IEC 27001:2022, GDPR, HIPAA, and zero-retention ephemeral storage policies.

22. **Continuous Access Evaluation (CAE) Claims-Challenge Negotiation (`copilot_cae_handler`):**
    - Intercepts HTTP 401 `WWW-Authenticate: Bearer error="insufficient_claims"` challenges per RFC 8693 / MS Graph CAE specification.
    - Automatically parses base64-encoded JSON claims objects, evaluates step-up authentication requirements, and issues remediation tokens with continuous zero-trust policy enforcement.

23. **Microsoft Graph Rich Notification JWE Decryption (`copilot_jwe_decrypt`):**
    - Decrypts RFC 7516 JSON Web Encryption (JWE) payloads received via real-time Microsoft Graph webhooks for Teams chats, channel messages, and online meeting transcripts.
    - Employs AES-256-GCM symmetric authenticated encryption with RSA-OAEP asymmetric key transport and HMAC-SHA256 signature verification.

24. **Microsoft Graph JSON Batching & DAG Dependency Engine (`copilot_graph_batch`):**
    - Consolidates up to 20 individual Microsoft Graph requests into a single RFC 2046 / OData `$batch` POST envelope, reducing network round-trips by up to 95%.
    - Formally validates dependencies using topological sort DAG analysis (`dependsOn`), detects cyclic dependencies, and auto-chunks oversized request batches.

25. **Incremental Delta Query & Tombstone Change Tracking (`copilot_delta_sync`):**
    - Manages stateful change tracking across SharePoint document libraries, Teams messages, and Planner tasks using `/delta` query endpoints.
    - Caches `@odata.deltaLink` tokens in `.tagisan/copilot_delta_cache.json` and accurately tracks tombstone deletions (`@removed`) and entity updates across synchronization cycles.

26. **Teams Adaptive Cards 1.6 Universal Actions (`copilot_universal_action`):**
    - Processes Modern Teams Adaptive Cards v1.6 `Action.Execute` callbacks with Single Sign-On (SSO) context validation.
    - Dispatches surgical actions (`approve_patch`, `run_autofix`, `run_debate`, `sync_adr`) and generates per-user `refresh` views tailored to each viewer's identity and security permissions.

27. **Azure Information Protection (AIP/RMS) Cryptographic Guard (`copilot_rms_guard`):**
    - Deeply inspects enterprise files protected by Microsoft Purview / Rights Management Services (AIP/RMS), including `.pfile` containers and Compound File Binary (CFB) format streams.
    - Formally verifies user license capabilities (`VIEW`, `EDIT`, `EXTRACT`) before permitting local ingestion, preventing unauthorized LLM processing of protected enterprise assets.

28. **Sovereign Clouds, Azure Managed Identity & Workload Identity Federation (`copilot_workload_identity`):**
    - Authenticates across multiple sovereign cloud partitions: Commercial, US Gov GCC High, US Gov DoD, and China (21Vianet), dynamically configuring login and Graph API endpoints.
    - Natively supports passwordless Azure Instance Metadata Service (IMDS) Managed Identity and RFC 7523 Workload Identity Federation OIDC token exchange for secure Kubernetes / GitHub Actions / Azure DevOps CI/CD runners.

---

### 🏛️ The 5 Strengthened Enterprise Engines

1. **Live Desktop Runtime Bridge (`MsDesktopRuntimeBridge`):**
   - Windows Named Pipe IPC (`\\.\pipe\tagisan_ms_desktop_bridge`) with cross-platform headless simulator for Linux/macOS CI.
   - Running Object Table (ROT) interrogation (`Visio.Application`, `Access.Application`, `Excel.Application`).
   - Direct ShapeSheet formula evaluation (`GUARD`, `SETATREF`, `DEPENDSON`).
   - Atomic VBA procedure execution inside managed undo scopes with automated rollback.
   - ACE Database direct SQL query runner.
2. **Microsoft Fabric OneLake Delta Engine (`FabricOneLakeEngine`):**
   - Direct OneLake ABFS (`abfss://workspace@onelake.dfs.fabric.microsoft.com/`) URI parsing & validation.
   - Parquet & Delta log transaction reader (`_delta_log/*.json`).
   - DirectLake semantic model synchronizer (transpiles Delta schemas directly to Power BI TMDL).
   - Fabric REST API client for automated workspace deployments and semantic model refreshes.
   - Schema drift & partition pruning analytics.
3. **Copilot Studio Agent-to-Agent (A2A) Swarm Engine (`CopilotA2ASwarmEngine`):**
   - Direct Line v3 / Open Agent streaming protocol.
   - Multi-agent negotiation, task delegation, and dialectical consensus handshakes.
   - Dynamic skill manifest exporter (exporting Tagisan ECC skills into Copilot Studio agent plugins).
   - Real-time activity stream handling with zero memory leak guarantees.
4. **Dataverse CDC & Enterprise Solution ALM Engine (`DataverseAlmEngine`):**
   - Pure-Rust Power Platform Solution Packager & Unpackager (`.zip` <-> extracted folder).
   - Component tree extraction (`customizations.xml`, `solution.xml`, `[Content_Types].xml`, `Workflows/`, `CanvasApps/`).
   - Change Data Capture (CDC) webhook processor parsing `RemoteExecutionContext` payloads.
   - Virtual Table & Polymorphic lookup integrity validator.
5. **Continuous Access Evaluation (CAE) Zero-Trust Guard (`CaeZeroTrustGuard`):**
   - CAE 401 challenge parser (`WWW-Authenticate: Bearer ... claims="..."`).
   - Decodes base64 JSON claims challenge and extracts `access_token` rules (`client_ip_changed`, `device_compliance_lost`, `pwd_reset_detected`).
   - Zero-Trust step-up token renewal orchestrator.
   - Memory-safe encrypted token caching with zeroization upon drop.

---

### 🚀 The 5 Frontier Enterprise Engines

1. **Microsoft Security Copilot & Intune DHA Engine (`MsSecurityCopilotIntuneEngine`):**
   - Security Copilot custom plugin & skill manifest exporter.
   - Intune device compliance verification (BitLocker, SecureBoot, Jailbreak, Antivirus age).
   - Conditional access gatekeeper before executing high-risk code changes or database migrations.
2. **Teams Graph Real-Time Media & Voice Calling AI Swarm (`TeamsRealTimeMediaEngine`):**
   - Graph Calling API session state machine (`/communications/calls`).
   - Real-time audio stream demuxer & jitter buffer simulation.
   - Voice Activity Detection (VAD) with RMS decibel energy analysis.
   - Real-time architectural speech invariant monitor and intervention announcer.
3. **Microsoft Entra Verified ID & Cryptographic Agent Credentials (`EntraVerifiedIdEngine`):**
   - W3C Verifiable Credentials (VC) & Decentralized Identifiers (DID: `did:web`, `did:ion`).
   - Pure-Rust Ed25519 / SHA-256 cryptographic signing of ADRs and Git commits.
   - Verifiable Presentation (VP) generation and trust-chain validation.
4. **Azure Service Bus & Event Grid AMQP 1.0 Streaming Engine (`AzureServiceBusAmqpEngine`):**
   - Pure-Rust AMQP 1.0 protocol frame handler (`Open`, `Begin`, `Attach`, `Transfer`, `Disposition`).
   - High-throughput credit prefetch window and backpressure management.
   - Dead-letter queue (DLQ) routing and message deduplication filters.
5. **Legacy OLE Compound Document Binary Format (CFBF) Forensics Engine (`OleBinaryForensicsEngine`):**
   - Pure-Rust cross-platform parser for legacy `.vsd`, `.mdb`, `.xls`, `.doc` binary containers.
   - Magic bytes verification (`0xD0CF11E0A1B11AE1`) and sector allocation table (FAT/MiniFAT) traversal.
   - Stream directory extractor (`Contents`, `VisioDocument`, `CompObj`, `SummaryInformation`).
   - Corruption detector and stream integrity auditor.

---

### 🛠️ Complete Catalog of All 56 Registered Microsoft Autonomous Tools

| # | Tool Identifier | Subsystem / Category | Description | DLP & Policy Enforcement |
| :-: | :--- | :--- | :--- | :--- |
| **1** | `copilot_teams_post` | Teams Communication | Post engineering alerts, debate verdicts, and status updates to Teams | Outbound Secret Scanned |
| **2** | `copilot_sharepoint_get` | SharePoint / OneDrive | Ingest Markdown, text, and DOCX files from SharePoint/OneDrive | Inbound Prompt Sanitized |
| **3** | `copilot_meeting_action_items` | Teams Calling & Meetings | Extract prioritized action items from Teams meeting transcripts | Inbound Prompt Sanitized |
| **4** | `copilot_export_report` | Outlook / Exchange | Dispatch styled HTML engineering reports via Outlook | Outbound Secret Scanned |
| **5** | `copilot_meeting_to_code` | Autonomous Engineering | End-to-end transcript extraction, AST blast radius checking, and code patching | Dual Inbound & Outbound |
| **6** | `copilot_blast_radius_report` | Telemetry & Cards | Generate Adaptive Card v1.5 and Fluent HTML reports for symbol blast radius | Outbound Secret Scanned |
| **7** | `copilot_debate_dispatch` | Dialectical Debate | Execute 3-round dialectical debate and dispatch verdict to Teams or Outlook | Outbound Secret Scanned |
| **8** | `copilot_purview_guard` | Information Protection | Evaluate Purview sensitivity labels, enforce zero-egress air-gapping, issue audit receipts | Air-Gap Zero-Egress Enforced |
| **9** | `copilot_adr_sync` | Architectural Records | Synthesize MADR architecture decision records and sync to OneNote & SharePoint | Outbound Secret Scanned |
| **10** | `copilot_create_pr` | Git / DevSecOps | Ephemeral Git branch creation, conventional commit generation, and PR blast telemetry | Outbound Secret Scanned |
| **11** | `copilot_export_deck` | Executive Presentations | Compile responsive executive briefing slide decks in Fluent HTML and Marp Markdown | Outbound Secret Scanned |
| **12** | `copilot_excel_functions` | Excel Modern Add-ins | Evaluate dynamic `=TGS.*` formulas and package Excel Office Add-in manifests | Formula Sanitized & Scanned |
| **13** | `copilot_stream_gateway` | Streaming Infrastructure | Live SSE and NDJSON real-time token streaming for Copilot Studio & Teams | Anti-Timeout Heartbeat |
| **14** | `copilot_planner_sync` | Project Management | Synchronize action items and tasks to Microsoft Planner buckets and To-Do lists | Outbound Secret Scanned |
| **15** | `copilot_incident_debugger` | Site Reliability (SRE) | Ingest CI/CD logs, synthesize autofix diffs, and generate Adaptive Cards with Action.Submit | Patch Sanitized & Verified |
| **16** | `copilot_hardware_telemetry` | Silicon & Green Computing| Measure Copilot+ PC NPU/DirectML telemetry, energy savings vs H100s, and CO2 avoided | Invariant Verified |
| **17** | `copilot_obo_exchange` | Identity & Security | Exchange incoming Copilot user JWT for downstream Graph token via OBO flow | User Security Context Preserved |
| **18** | `copilot_subscription_manage` | Graph Webhooks | Manage Graph webhook subscriptions and handle validation challenge handshakes (<10s) | HMAC-SHA256 Verified |
| **19** | `copilot_purview_sync` | Data Governance | Dynamically synchronize tenant Purview sensitivity taxonomy from Graph API | Tenant Policy Bound |
| **20** | `copilot_sentinel_audit` | SIEM & SecOps | Emit structured CEF:0, RFC 5424, and Azure Monitor DCR SIEM security events | Cryptographically Audited |
| **21** | `copilot_certify` | Compliance & Governance | Audit and certify Microsoft 365 Admin Center App Compliance and Publisher Attestation | Partner Center Certified |
| **22** | `copilot_cae_handler` | Zero-Trust Identity | Intercept HTTP 401 CAE claims challenges, parse step-up rules, issue remediation tokens | Continuous Zero-Trust Enforced |
| **23** | `copilot_jwe_decrypt` | Cryptography & Messaging | Decrypt RFC 7516 JWE payloads for real-time Teams/Graph rich notification webhooks | Cryptographically Decrypted |
| **24** | `copilot_graph_batch` | Graph API Optimization | Execute high-throughput JSON batch requests ($batch) with topological DAG ordering | Quota Optimized & DAG Validated |
| **25** | `copilot_delta_sync` | State Synchronization | Incremental change tracking via /delta with @odata.deltaLink state caching | Change Tracking & Delta Cached |
| **26** | `copilot_universal_action` | Modern UI & Cards | Handle Teams Adaptive Cards 1.6 Action.Execute callbacks, validate SSO context | Universal Action Verified |
| **27** | `copilot_rms_guard` | Content Protection | Cryptographic guard for Azure Information Protection (AIP/RMS) .pfile containers | RMS Rights & Egress Guarded |
| **28** | `copilot_workload_identity` | Cloud Infrastructure | Authenticate across Azure Managed Identity (IMDS), Workload Identity, Sovereign Clouds | Zero-Trust Sovereign Bound |
| **29** | `copilot_sharepoint_crawler` | Enterprise Knowledge | Deep recursive crawler for SharePoint document libraries and metadata extraction | Access Controlled & Scanned |
| **30** | `copilot_airgap_router` | Air-Gap Routing | Intelligent task router enforcing local tensor routing for confidential workloads | Zero-Egress Air-Gap Locked |
| **31** | `copilot_loop_sync` | Modern Collaboration | Bidirectional synchronizer for Microsoft Loop components and pages | Structured Schema Validated |
| **32** | `copilot_ooxml_generator` | Document Generation | Pure-Rust generator for styled Word (.docx) and PowerPoint (.pptx) packages | CRC32 & OPC Schema Verified |
| **33** | `copilot_calendar_pre_read` | Executive Briefings | Analyze upcoming Outlook calendar events and compile technical pre-read executive briefs | Calendar Context Sanitized |
| **34** | `copilot_outlook_draft` | Email Automation | Draft structured engineering status updates and architecture proposals in Outlook | Outbound Secret Scanned |
| **35** | `copilot_fabric_query` | Data Analytics | Execute DAX and REST queries against Microsoft Fabric DirectLake semantic models | Read-Only Credential Scoped |
| **36** | `copilot_perms_auditor` | Security Auditing | Audit delegated and application Microsoft Graph permissions against least-privilege | Scope Minimization Verified |
| **37** | `copilot_ado_sync` | DevOps Engineering | Synchronize tasks, pull requests, and work items with Azure DevOps (ADO) | Workload Identity Bound |
| **38** | `copilot_substrate_ingest` | Search & Intelligence | Ingest structured documents into Microsoft Substrate / Microsoft Search external connections | Schema Validated |
| **39** | `copilot_wam_auth` | Windows Authentication | Interoperate with Windows Web Account Manager (WAM) broker for silent enterprise SSO | Platform Broker Authenticated |
| **40** | `copilot_icm_bridge` | Incident Management | Correlate git commits with Microsoft Incident Care & Management (IcM) severe alerts | Correlation Confidence Ranked |
| **41** | `copilot_sdl_audit` | Security Development | Audit codebases against Microsoft Security Development Lifecycle (SDL), CredScan, PoliCheck | Policy Rule Checked |
| **42** | `copilot_studio_packager` | Copilot Extensions | Package Tagisan agents into Copilot Studio declarative agent solutions and plugins | Schema 1.0 Compliant |
| **43** | `copilot_viva_sync` | Employee Experience | Synchronize engineering OKRs and meeting agendas to Microsoft Viva Goals & Insights | Privacy Boundary Maintained |
| **44** | `copilot_dataverse_sync` | Power Platform Data | Synchronize data, entities, and ALM solution manifests with Microsoft Dataverse | OData v4 Contract Enforced |
| **45** | `copilot_power_automate` | Workflow Automation | Trigger and orchestrate Power Automate cloud flows via HMAC-authenticated webhooks | HMAC-SHA256 Signed |
| **46** | `copilot_powerplatform_packager`| Enterprise ALM | Packager and unpackager for Microsoft Power Platform solutions and PCF controls | Solution XML & Zip Validated |
| **47** | `copilot_access` | Database Modernization | Access 365 schema analysis, VBA transpilations, and Dataverse migration planning | SQL & Twip Boundary Checked |
| **48** | `copilot_vscode` | Developer Experience | VS Code extension bridge, Copilot Chat participant provider, and LSP code actions | In-Memory IPC Verified |
| **49** | `copilot_defender` | Threat Intelligence | Ingest Microsoft Defender for Endpoint alerts and synthesize automated virtual patches | Reachability & CVE Ranked |
| **50** | `copilot_bicep` | Infrastructure as Code | Transpile and validate Azure Bicep templates against Azure Verified Modules (AVM) | ARM Transpilation Verified |
| **51** | `copilot_powerbi` | Business Intelligence | TMDL and TMSL model generator and validator for Power BI DirectLake models | Semantic Model Validated |
| **52** | `copilot_visio` | Architecture Visuals | Visio 365 ShapeSheet evaluator, C4 modeler, and Data Visualizer flowchart generator | ShapeSheet Geometry Verified |
| **53** | `copilot_sarif` | Security Reporting | Static Analysis Results Interchange Format (SARIF 2.1.0) generator and ADO pipeline bridge| OASIS SARIF Schema Validated |
| **54** | `copilot_jet_binary` | Binary Forensics | Pure-Rust JET/ACE database binary parser (.mdb/.accdb) and integrity auditor | Page Size & Magic Verified |
| **55** | `copilot_calling` | Real-Time Voice | Real-time Teams Calling WebRTC and audio demuxing session manager with VAD analysis | RMS Energy & VAD Monitored |
| **56** | `copilot_delta_lake` | Big Data & Analytics | DirectDelta log reader and OneLake table snapshot analyzer with partition pruning | Transaction Log Audited |

---

### 💻 Dedicated CLI Usage Guide (`tgs copilot`)

```bash
# 1. Inspect Copilot Subsystem & Entra ID status (all 56 tools active)
tgs copilot status

# 2. Authenticate with Entra ID via OAuth2 Device Code Flow
tgs copilot auth
tgs copilot auth --status

# 3. Enforce Microsoft Purview Sensitivity & Zero-Egress Air-Gapping
tgs copilot purview --content "Confidential enterprise ledger" --label "Confidential"

# 4. Synthesize MADR Architecture Decision Record & Sync to OneNote / SharePoint
tgs copilot adr --proposal "Adopt zero-egress in-process GGUF routing" --title "Zero Egress Architecture"

# 5. Create Ephemeral Git Branch & Automated Pull Request with Blast Telemetry
tgs copilot pr --patch "diff --git a/file b/file" --title "feat(copilot): harden graph client" --channel "general"

# 6. Generate Responsive Executive Presentation Deck (HTML & Markdown)
tgs copilot deck --title "Q3 Engineering Copilot Briefing" --format all --output ".tagisan/executive_deck.html"

# 7. Start Teams Bot Webhook Listener for Adaptive Card Action.Submit Callbacks
tgs copilot listen --port 3978
tgs copilot listen --test-action "approve_patch" --target "src/copilot/mod.rs"

# 8. Execute Meeting-to-Code Pipeline on Teams Transcript
tgs copilot meeting-to-code --meeting "sprint_42_sync" --path "." --channel "general"

# 9. Generate Codebase Blast Radius Telemetry & Adaptive Cards
tgs copilot blast-report --symbol "EntraAuthManager" --max-depth 3 --format all

# 10. Dispatch 3-Round Dialectical Debate on Architecture Proposal
tgs copilot debate --proposal "Adopt lock-free concurrent channels for vector sync" --post-to-teams "architecture-channel"

# 11. Post an update directly to Teams
tgs copilot post --channel "engineering-alerts" --message "Tagisan grounding invariants verified: 0 regressions."

# 12. Ingest and inspect meeting transcript
tgs copilot transcript --meeting "latest_sync" --parse-items

# 13. Index skills and architecture diagrams into Microsoft Search Connector
tgs copilot index

# 14. Export complete Copilot package bundle for sideloading
tgs copilot package --output-dir ".tagisan/copilot_package" --base-url "https://api.tagisan.ai"

# 15. Evaluate Excel Custom Functions or export Office Add-in package
tgs copilot excel --formula '=TGS.BLAST_RADIUS("EntraAuthManager", ".")'
tgs copilot excel --export-package --output-dir ".tagisan/excel_addin"

# 16. Stream debate or token generation via Server-Sent Events (SSE)
tgs copilot stream --topic "Lock-free channel architecture" --format sse

# 17. Synchronize tasks to Microsoft Planner or To-Do
tgs copilot planner --title "Verify Purview Air-Gap Invariants" --plan-id "plan_prod_01" --bucket-id "bucket_security" --priority 1

# 18. Debug CI/CD incident logs and synthesize autofix Adaptive Card
tgs copilot incident --log-file "ci_failure.log" --channel "incident-response"

# 19. Inspect Windows Copilot+ PC hardware telemetry, NPU acceleration, and CO2 savings
tgs copilot hardware --prompt-tokens 50000 --completion-tokens 10000

# 20. On-Behalf-Of (OBO) Token Exchange with downstream Graph scopes
tgs copilot obo --assertion "eyJhbGciOiJSUzI1NiIs..." --scopes "User.Read Files.Read.All" --use-cert

# 21. Manage Microsoft Graph Webhook Subscriptions & Handle Challenge
tgs copilot subscribe --action create --resource "me/onlineMeetings" --notification-url "https://api.tagisan.ai/copilot/webhook"
tgs copilot subscribe --action list
tgs copilot subscribe --action validate_challenge --validation-token "CHALLENGE_TOKEN_XYZ"

# 22. Emit Microsoft Sentinel & Azure Monitor SIEM Telemetry
tgs copilot sentinel --event-type ast_blast_radius --severity High --message "Refactoring EntraAuthManager impacts 14 dependents" --format cef

# 23. Synchronize Tenant Purview Sensitivity Label Taxonomy
tgs copilot sync-labels --action sync
tgs copilot sync-labels --action list

# 24. Certify and Export M365 Admin Center Compliance Bundle
tgs copilot certify --action audit
tgs copilot certify --action export --output-dir ".tagisan/copilot_package"

# 25. Run built-in Copilot subsystem self-test suite
tgs copilot test
```

---

## 🐍 🐪 🥟 Polyglot Sandboxed Runtimes & Domain Engines

Tagisan provides native execution runtimes with built-in AgentShield sandboxing:

- **Bun / TypeScript Runtime (`bun_eval`, `bun_run`, `bun_test`, `bun_serve`, `bun_compile`):**
  Execute TypeScript without transpilation steps, run tests with sub-millisecond execution, launch microservices, and compile scripts directly to standalone executables. Includes built-in SQLite vector storage.
- **Python 3 Runtime (`python_eval`, `python_run`, `python_install`):**
  Full integration with `uv` and `pip` environments. Supports in-memory evaluation, test execution via `pytest`, and static analysis with `mypy`.
- **Perl 5 Runtime (`perl_eval`, `perl_run`, `perl_install`):**
  Lightning-fast regex text forensics, stream transformations, and CPAN package management.
- **Vella Sovereign OS (`tgs vella`):**
  Deep-systems framework integrating SCADA protocol engines, algorithmic trading, robotics kinematic solvers, medical telemetry, Fully Homomorphic Encryption (TFHE), orbital astrodynamics, and hardware E-Stop latches.
- **OTP / BEAM & Gleam Actor Subsystems (`tgs otp`, `tgs gleam`):**
  Native actor supervision trees, Erlang External Term Format (ETF) serialization, type checking, and fault-tolerant concurrency.

---

## 📚 Curated 2,063+ Skill Canons & 13 Agent Personas

Tagisan contains **2,063 compiled skills** in systems memory, mathematically validated with Modulo-10 and Modulo-11 ISBN checksums, with an on-disk extensible catalog in `.ecc/skills/` surpassing **4,600+ skills**:

```
                              [ CURATED 2,063+ SKILL CANONS ]
 ┌──────────────────────────────────────────┬──────────────────────────────────────────┐
 │ Microsoft Stack Enterprise Canons        │ Vibe-Coding, Engineering & Theory Canons  │
 ├──────────────────────────────────────────┼──────────────────────────────────────────┤
 │ • Visio 365 Architecture (350 skills)    │ • Agentic Engineering Canon (100 books)  │
 │ • Access 365 Database (350 skills)       │ • Enterprise ERP Architecture (50 books) │
 │ • Office 365 Subsystems (350 skills)     │ • Data Analytics & Lakes (50 books)      │
 │ • Power Platform & ALM (350 skills)      │ • Business Analysis & Strategy (50 books)│
 │ • M365 Copilot & Graph (150 skills)      │ • Systems Architecture Canon (50 books)  │
 │ • Frontier & Systems LLM (30 skills)     │ • Behavioral Design & Qual (50 books)    │
 │                                          │ • Financial & Quant Analysis (50 books)  │
 │                                          │ • Applied Mathematics & Crypto (20 books)│
 └──────────────────────────────────────────┴──────────────────────────────────────────┘
```

### 🎭 13 Curated Agent Personas (`.ecc/agents/`)

Tagisan equips specialized agent personas with custom system prompts and domain invariants:
1. `code-explorer` — Repository structure explorer and call-tree analyzer.
2. `codebase-graph-architect` — Petgraph AST dependency and blast-radius modeling specialist.
3. `compiler-healer-expert` — Autonomous diagnostic healer and TDD test repair engineer.
4. `crypto-market-analyst` — Web3 tokenomics and decentralized protocol risk auditor.
5. `debugger` — Root-cause stack trace and memory corruption investigator.
6. `failure-forensic-expert` — Post-mortem systems failure and invariant breach analyst.
7. `forex-quant-expert` — High-frequency quantitative currency and volatility modeler.
8. `github-superpowers-architect` — GitHub Actions, security automation, and PR orchestration lead.
9. `local-grounding-expert` — Offline deterministic grounding and formal specification verifier.
10. `philosophical-conceptual-analyst` — Dialectical reasoner balancing fundamental trade-offs.
11. `rust-engine-architect` — Systems-grade zero-copy Rust and Tokio concurrency specialist.
12. `strategic-competitive-analyst` — Business moat analysis and architectural positioning auditor.
13. `web-designer-developer` — Modern responsive UI, design systems, and frontend engineer.

---

## 💻 Comprehensive CLI Command Reference (All 37 Subcommands)

```bash
# ==============================================================================
# 1. LOCAL TENSOR ENGINE & SERVER DAEMON
# ==============================================================================
# Launch native Ollama-compatible Tokio HTTP daemon on port 11434 (uses ~10 MB RAM)
tgs serve --port 11434 --host 127.0.0.1

# Discover and list all local GGUF models on disk (mapped from ~/.ollama/models)
tgs engine list

# Deeply inspect GGUF binary header, 256 tensors, quant types, and chat templates
tgs engine inspect abliterated

# ==============================================================================
# 2. INTERACTIVE QUERIES & STREAMING
# ==============================================================================
# Direct query with 3-second heartbeat pulse (auto-detects best local or cloud model)
tgs ask "Explain Amdahl's Law vs Gustafson's Law in concurrent systems"

# Real-time token streaming with sub-millisecond dispatch
tgs stream -p ollama -m llama3.2-abliterate:3b-instruct "Implement a lock-free RingBuffer in Rust"

# ==============================================================================
# 3. MULTIMODAL COMPUTER VISION
# ==============================================================================
# Debug an error trace screenshot with Google Gemini
tgs ask -p gemini -i ./traceback.png "Identify the panic source and provide the patch"

# Run multimodal vision 100% locally with Moondream (fits 8GB laptops!)
tgs ask -p ollama -m moondream -i ./ui_mockup.png "List UI elements and missing ARIA attributes"

# ==============================================================================
# 4. DIALECTICAL DEBATE (TAGISAN NG TALINO)
# ==============================================================================
# 100% Offline 3-round debate using local models ($0.00 cost)
tgs debate -p ollama "Postgres JSONB vs Relational Normalization for event sourcing"

# Cloud debate (Claude vs DeepSeek with Gemini Lakandiwa Synthesis)
tgs debate "Should we build our backend with Rust microservices or a modular Go monolith?"

# Launch interactive multi-pane Terminal User Interface (TUI)
tgs debate --tui "Rust vs Zig for bare-metal systems"

# ==============================================================================
# 5. MIXTURE-OF-AGENTS (MoA) & PARALLEL PROPOSERS
# ==============================================================================
tgs moa "Design a high-throughput, low-latency rate limiter with sliding window"

# ==============================================================================
# 6. AUTONOMOUS MULTI-TURN AGENT
# ==============================================================================
# Run agent with tools enabled and AgentShield cyber defense gate active
tgs agent "Scan src/engine/gguf.rs, identify performance bottlenecks, and optimize them"

# Autonomous run with persistent codebase memory and isolated Git sandbox
tgs agent "Refactor vector memory distance functions" --memory --sandbox

# Autonomous run with external MCP tools enabled
tgs agent "Analyze our database schema and propose index optimizations" --mcp

# ==============================================================================
# 7. WORKFLOW & MULTI-AGENT DAG PIPELINES
# ==============================================================================
# Plan an autonomous multi-stage workflow
tgs workflow plan "Migrate auth service from JWT to WebAuthn"

# Execute a pre-planned DAG workflow with concurrency bounds
tgs workflow run --plan workflow.json --concurrency 4

# ==============================================================================
# 8. STRUCTURED ROLE-BASED HARMONY SWARM (tgs harmony)
# ==============================================================================
# 4-stage assembly line (Architect -> Implementer -> QA -> Doc)
tgs harmony build "Implement an async lock-free ringbuffer in Rust"

# Smart Tier: Claude (Architect/QA) + DeepSeek (Implementer) + Gemini (Docs)
tgs harmony build "Build a microservice router" --tier smart --parallel

# Economy Tier: High-speed, sub-cent execution with Gemini 2.0 Flash
tgs harmony build "Generate CLI argument parser" --tier economy --parallel

# ==============================================================================
# 9. MULTI-AGENT SWARM ORCHESTRATION & CONSENSUS
# ==============================================================================
# Specialist delegation with Lead Architect
tgs swarm run "Design zero-allocation token bucket" --agents architect,tdd-engineer,security-auditor --lead architect

# Mathematical consensus review on a source file
tgs consensus src/engine/autofix.rs --rule unanimous

# Position-based Borda Count consensus ranking
tgs consensus "Raft vs Paxos for consensus engine" --rule borda

# Inspect live Swarm Atlas & routing heat cortex dashboard
tgs swarm atlas

# ==============================================================================
# 10. DISTRIBUTED LOCAL P2P CLUSTER MESH
# ==============================================================================
# Start mesh coordinator on TCP port 8765
tgs swarm cluster coordinator --bind 0.0.0.0:8765

# Join cluster from another compute node
tgs swarm cluster worker --coordinator 192.168.1.100:8765 --worker-id worker-nvme-01

# Query active cluster nodes and latency metrics
tgs swarm cluster status --coordinator 192.168.1.100:8765

# ==============================================================================
# 11. SELF-HEALING COMPILER & TDD HEALER (tgs autofix)
# ==============================================================================
# Automatically diagnose and heal compiler issues in current directory
tgs autofix

# Dry run inspection (shows proposed surgical AST patches without editing disk)
tgs autofix --dry-run

# Target specific project with 10 max iterative repair attempts
tgs autofix /path/to/project --max-attempts 10

# Include full test suites during healing passes
tgs autofix . --test

# ==============================================================================
# 12. AST CODEBASE KNOWLEDGE GRAPH & BLAST RADIUS (tgs graph)
# ==============================================================================
# Display overall AST graph metrics and top architectural hubs
tgs graph stats

# Locate symbol definition, signature, file span, and docstring
tgs graph symbol TokenBudgetTracker

# Trace all incoming callers across the repository
tgs graph callers apply_span_replacement

# Trace outgoing function and method calls invoked by a symbol
tgs graph callees handle_autofix_command

# Calculate transitive blast radius and refactoring risk
tgs graph blast-radius AutofixEngine --max-depth 4

# Export directed knowledge graph in Graphviz DOT or JSON format
tgs graph export --format dot > codebase_graph.dot
tgs graph export --format json > codebase_graph.json

# ==============================================================================
# 13. DETERMINISTIC CLOSED-LOOP GROUNDING (tgs ground)
# ==============================================================================
tgs ground "Implement a concurrent LRU cache with zero-copy get" --path src/cache.rs

# ==============================================================================
# 14. AUTONOMOUS CYBER DEFENSE & THREAT HUNTING (tgs shield)
# ==============================================================================
# Recursively audit a codebase or untrusted PR for APT IOCs & backdoors
tgs shield scan ./untrusted-repo

# Pre-execution vetting of an arbitrary shell command or script payload
tgs shield audit "powershell -enc JABjAGwAaQBlAG4AdAAgAD0AIABOAGUAdwAtAE8AYgBqAGUAYwB0AA=="

# Display active cyber defense status and protected asset registry
tgs shield status

# ==============================================================================
# 15. MICROSOFT 365 COPILOT & ENTERPRISE INTEGRATION (tgs copilot)
# ==============================================================================
# Inspect Copilot Subsystem & Entra ID status (all 56 tools active)
tgs copilot status

# Authenticate with Entra ID via OAuth2 Device Code Flow
tgs copilot auth

# Enforce Microsoft Purview Sensitivity & Zero-Egress Air-Gapping
tgs copilot purview --content "Confidential ledger" --label "Confidential"

# Synthesize MADR Architecture Decision Record & Sync to OneNote / SharePoint
tgs copilot adr --proposal "Adopt zero-egress GGUF routing" --title "Zero Egress Architecture"

# Create Ephemeral Git Branch & Automated PR with Blast Telemetry
tgs copilot pr --patch "diff --git a/file b/file" --title "feat(copilot): harden graph client" --channel "general"

# Generate Responsive Executive Presentation Deck (HTML & Markdown)
tgs copilot deck --title "Q3 Engineering Copilot Briefing" --format all --output ".tagisan/executive_deck.html"

# Start Teams Bot Webhook Listener for Adaptive Card Action.Submit Callbacks
tgs copilot listen --port 3978

# Execute Meeting-to-Code Pipeline on Teams Transcript
tgs copilot meeting-to-code --meeting "sprint_42_sync" --path "." --channel "general"

# Generate Codebase Blast Radius Telemetry & Adaptive Cards
tgs copilot blast-report --symbol "EntraAuthManager" --max-depth 3 --format all

# Dispatch 3-Round Dialectical Debate on Architecture Proposal to Teams
tgs copilot debate --proposal "Adopt lock-free concurrent channels" --post-to-teams "architecture-channel"

# Index skills and architecture diagrams into Microsoft Search Connector
tgs copilot index

# Export complete Copilot package bundle for sideloading
tgs copilot package --output-dir ".tagisan/copilot_package" --base-url "https://api.tagisan.ai"

# Evaluate Excel Custom Functions or export Office Add-in package
tgs copilot excel --formula "=TGS.BLAST_RADIUS(\"EntraAuthManager\", \".\")"

# Synchronize tasks to Microsoft Planner or To-Do
tgs copilot planner --title "Verify Purview Air-Gap Invariants" --plan-id "plan_prod_01" --bucket-id "bucket_sec"

# Debug CI/CD incident logs and synthesize autofix Adaptive Card
tgs copilot incident --log-file "ci_failure.log" --channel "incident-response"

# Inspect Windows Copilot+ PC hardware telemetry, NPU acceleration, and CO2 savings
tgs copilot hardware --prompt-tokens 50000 --completion-tokens 10000

# On-Behalf-Of (OBO) Token Exchange with downstream Graph scopes
tgs copilot obo --assertion "eyJhbGciOiJSUzI1NiIs..." --scopes "User.Read Files.Read.All" --use-cert

# Manage Microsoft Graph Webhook Subscriptions
tgs copilot subscribe --action create --resource "me/onlineMeetings" --notification-url "https://api.tagisan.ai/webhook"

# Emit Microsoft Sentinel & Azure Monitor SIEM Telemetry
tgs copilot sentinel --event-type ast_blast_radius --severity High --message "Refactoring EntraAuthManager impacts 14 dependents"

# Synchronize Tenant Purview Sensitivity Label Taxonomy
tgs copilot sync-labels --action sync

# Certify and Export M365 Admin Center Compliance Bundle
tgs copilot certify --action audit

# Run built-in Copilot subsystem self-test suite
tgs copilot test

# ==============================================================================
# 16. MODEL CONTEXT PROTOCOL (MCP) & EXTENSIONS
# ==============================================================================
# List discovered local and dynamic MCP servers
tgs mcp list

# Search MCP plugin catalog
tgs mcp search postgres

# Launch native Tagisan MCP server over stdio for Claude Desktop or Cursor
tgs serve-mcp

# ==============================================================================
# 17. POLYGLOT SANDBOXED RUNTIMES (Bun, Python, Perl, OTP, Gleam)
# ==============================================================================
# Bun TypeScript evaluation and standalone native compilation
tgs bun eval "console.log('Bun speed:', performance.now())"
tgs bun compile script.ts --outfile standalone_binary

# Python execution via isolated uv / virtualenv
tgs python run script.py

# Perl text forensics and one-liner execution
tgs perl eval 'while(<>){ print if /CRITICAL/ }' logfile.log

# Vella Sovereign OS status and Hardware E-Stop trigger
tgs vella status
tgs vella estop

# OTP supervision tree and Gleam type verification
tgs otp supervise
tgs gleam check

# ==============================================================================
# 18. REPL, SESSIONS & PERSISTENT MEMORY
# ==============================================================================
# Launch interactive REPL with specialized persona
tgs repl --agent security-auditor --memory

# Manage persistent sessions
tgs session list
tgs session resume session_xyz

# Vector memory indexing and semantic search
tgs memory index src/
tgs memory search "How does GGUF zero-copy parsing work?"
```

---

## 🛠️ Configuration & Environment

Tagisan works with **zero configuration** out of the box using your local Ollama installation. To customize behavior, enable cloud models, or adjust hardware parameters, configure `.env`:

```env
# =========================================================
# 🇵🇭 TAGISAN: LOCAL OLLAMA & GGUF CONFIGURATION
# =========================================================
TAGISAN_PROVIDER=ollama
OLLAMA_MODEL=huihui_ai/llama3.2-abliterate:3b-instruct

# Unrestricted Mode: Remove safety barriers for security audits
TAGISAN_UNRESTRICTED=1
TAGISAN_NO_RESTRICTIONS=1

# Local Ollama Tuning for 8GB Laptops
OLLAMA_FLASH_ATTENTION=1
OLLAMA_KV_CACHE_TYPE=q4_0
OLLAMA_KEEP_ALIVE=24h
OLLAMA_NUM_PARALLEL=1

# Global Token Budget Cap (Session Limit in USD)
TAGISAN_MAX_BUDGET=5.00

# =========================================================
# ☁️ CLOUD MODEL API KEYS (OPTIONAL)
# =========================================================
GEMINI_API_KEY=AIzaSy...             # Google AI Studio (Free Tier Supported)
DEEPSEEK_API_KEY=sk-...              # DeepSeek R1 / V3
ANTHROPIC_API_KEY=sk-ant-api03-...   # Claude 3.5 Sonnet / Opus
OPENAI_API_KEY=sk-proj-...           # GPT-4o / GPT-4o-mini
XAI_API_KEY=xai-...                  # Grok-2 / Grok-Vision

# =========================================================
# 🏢 MICROSOFT 365 & ENTRA ID CONFIGURATION (OPTIONAL)
# =========================================================
ENTRA_TENANT_ID=00000000-0000-0000-0000-000000000000
ENTRA_CLIENT_ID=00000000-0000-0000-0000-000000000000
ENTRA_CLIENT_SECRET=...
```

Inspect active runtime configuration and bitflags at any time:
```bash
tgs status
```

---

## 🧪 Comprehensive Verification & Brutal Test Suite Matrix

Tagisan is rigorously verified by **84 brutal, high-concurrency integration test suites** in [`tests/`](tests/) achieving a **100% pass rate** and sustained stress throughput of **>300,000 ops/sec** across 50 concurrent worker threads:

```bash
# Run all standard unit and integration tests (21 passed, 0 failed)
cargo test

# 🏢 Microsoft Frontier Enterprise Brutal Tests (13 Tests: Security Copilot, Teams Audio VAD, Entra DID, AMQP 1.0, OLE CFBF)
cargo test --test ms_frontier_enterprise_brutal_tests -- --nocapture

# 🏢 Microsoft Strengthened Enterprise Brutal Tests (16 Tests: Desktop Bridge, Fabric OneLake, Copilot A2A, Dataverse, CAE)
cargo test --test ms_strengthened_enterprise_brutal_tests -- --nocapture

# 🏢 Microsoft 365 Copilot & Graph Communication System (30 Tests)
cargo test --test ms365_copilot_brutal_tests -- --nocapture

# 🏢 Microsoft Access 365 Canon & ACE Geometry (350 Skills)
cargo test --test access365_skills_brutal_tests -- --nocapture

# 🏢 Microsoft Visio 365 Canon & ShapeSheet recalculation (350 Skills)
cargo test --test visio365_skills_brutal_tests -- --nocapture

# 🏢 Microsoft Office 365 & Power Platform Subsystems (700 Skills)
cargo test --test office365_skills_brutal_tests -- --nocapture
cargo test --test power_platform_skills_brutal_tests -- --nocapture

# 🛡️ Autonomous Cyber Defense & Nation-State Hardening (10 Tests)
cargo test --test cyber_defense_hardening_brutal_tests -- --nocapture

# 🛡️ Standalone tgs Cyber Defense & Threat Hunting (7 Tests)
cargo test --test tgs_cybersecurity_standalone_brutal_tests -- --nocapture

# 🛡️ Rogue AI Cyberwarfare Defense & Autonomous Threats (14 Tests)
cargo test --test rogue_ai_cyberwarfare_brutal_tests -- --nocapture

# 🔄 Local <-> Non-Local Engine Failovers & Cloud Evacuation (24 Tests)
cargo test --test local_to_nonlocal_transition_brutal_tests -- --nocapture

# 🔔 Real-Time Notification Hub & Abnormality Alerting (18 Tests)
cargo test --test skills_notification_local_nonlocal_brutal_tests -- --nocapture

# 📚 Skills Authenticity & LLM Integration Census (7 Tests, 4,622 Skills)
cargo test --test skills_authenticity_and_integration_brutal_tests -- --nocapture

# 🐝 Structured Role-Based Harmony Swarm Stress Tests
cargo test --test harmony_swarm_stress_tests -- --nocapture

# 🌲 AST Codebase Graph & Blast Radius Verification (7 Phases)
python3 scripts/test_codebase_graph_verification.py

# 🔧 Self-Healing Compiler & TDD Healer Verification (6 Phases)
python3 scripts/test_autofix_verification.py

# 🦀 Reverse-Engineered Ollama Rust Tensor Engine Verification (6 Tiers)
python3 scripts/test_rust_engine_verification.py

# 🦅 5 Colibrì Superpowers Verification (JIT Paging, Atlas, MoE, P2P Mesh, Guard)
python3 scripts/test_colibri_superpowers.py
```

---

## 🗺️ Architectural Specifications & Roadmaps

- 🛡️ **[Cyber Defense & AgentShield Specification](docs/SPEC_AGENTSHIELD_CYBER_DEFENSE.md)** — Architectural specification for zero ambient authority sandboxing, nation-state APT mitigation (Lazarus Group / APT38), pre-execution command auditing, and real-time notification alerts.
- ⚡ **[Top 50 Essential Skills for TGS Ecosystem](docs/TOP_50_ESSENTIAL_SKILLS_FOR_TGS_ECOSYSTEM.md)** — The authoritative canon of 50 open-source engineering capabilities, protocols, and architectures powering the Tagisan runtime.
- 🌲 **[Codebase AST Graph & Blast Radius Guide](docs/CODEBASE_AST_GRAPH_AND_BLAST_RADIUS_GUIDE.md)** — Architectural guide for multi-language AST extraction, petgraph call topologies, and transitive blast-radius refactoring risk modeling.
- 🔧 **[Self-Healing Compiler & TDD Healer Guide](docs/SELF_HEALING_COMPILER_AUTODEV_GUIDE.md)** — Architectural guide for AST span replacement, polyglot compiler JSON ingestion, and autonomous test self-repair.
- 📖 **[Reverse-Engineered Ollama Rust Tensor Engine Guide](docs/REVERSE_ENGINEERED_OLLAMA_RUST_ENGINE_GUIDE.md)** — In-depth architectural guide for zero-copy GGUF v2/v3 parsing, memory mapping, and Tokio HTTP streaming.
- 📑 **[RFC-001: Universal Protocol & Ecosystem Integrations](ROADMAP_EXTENSIONS.md)** — Pluggable external vector backends, OpenTelemetry tracing, and automated swarm benchmarking.
- 🔌 **[RFC-002: WASM & Native Dynamic Plugins Architecture](ROADMAP_PLUGINS.md)** — Extism WebAssembly sandboxing and native dynamic shared library plugins.
- 🏛️ **[RFC-003: Structured Role-Based Harmony Swarms](docs/SPEC_STRUCTURED_ROLE_HARMONY_SWARM.md)** — Deterministic multi-model assembly line, anti-sycophancy gates, and shared blackboard memory.
- 🧠 **[RFC-004: Semantic Skill Dispatching & JIT Knowledge Injection](docs/SPEC_LOCAL_LLM_SKILL_DISPATCHING.md)** — Top-K semantic skill auto-equipping and sub-millisecond cheat-sheet prompt injection.

---

## 📄 License

Dual-licensed under the **MIT License** and the **Apache 2.0 License**. See [LICENSE](LICENSE) for details.
