<div align="center">
  <a href="https://github.com/CharleGutierrez/tagisan">
    <img src="assets/tagisan_logo.png" alt="Tagisan Logo Emblem" width="360" />
  </a>

  # 🇵🇭 Tagisan (`tgs` / `tagisan-rs`)

  ### The `uv` of Multi-Agent AI Swarms & In-Process Tensor Engines in Systems-Grade Rust

  **Tagisan ng Talino:** In-Process GGUF Tensor Engine • Native Ollama HTTP Server • Ultra-Fast Swarms • Dialectical Debate • 5 Colibrì Superpowers • Surgical File CRUD • Computer Vision • Bidirectional MCP

  <br />

  [![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)](https://www.rust-lang.org/)
  [![Binary](https://img.shields.io/badge/CLI-tgs-brightgreen.svg)](https://github.com/CharleGutierrez/tagisan)
  [![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
  [![Tokio](https://img.shields.io/badge/async-tokio-blue)](https://tokio.rs/)
  [![GGUF Engine](https://img.shields.io/badge/GGUF-Zero--Copy%20mmap%20(56%C2%B5s)-red)](https://github.com/CharleGutierrez/tagisan)
  [![Ollama](https://img.shields.io/badge/Ollama-Native%20Tokio%20Server-black)](https://ollama.com/)
  [![Polyglot](https://img.shields.io/badge/runtimes-Python%20%7C%20Perl%20%7C%20Bun-yellow)](https://github.com/CharleGutierrez/tagisan)
  [![MCP](https://img.shields.io/badge/protocol-MCP%20Client%20%26%20Server-purple)](https://modelcontextprotocol.io/)
  [![Colibri](https://img.shields.io/badge/superpowers-5%20Colibr%C3%AC%20Engines-cyan)](https://github.com/CharleGutierrez/tagisan)
</div>

---

> **Why choose between Python dependency hell, slow agent frameworks, and bloated AI servers?**  
> Tagisan (`tgs`) is a single, zero-dependency **20 MB static binary** that boots in **~2 milliseconds**, runs on **10 MB of RAM**, parses 2+ GB GGUF models in **56 microseconds**, and coordinates heterogeneous local models and cloud APIs (Claude, Gemini, GPT-4o, DeepSeek, Grok) into adversarial swarms, parallel DAGs, and autonomous tool loops with **$0.00 API cost**.

```bash
# ⚡ Install Tagisan in 5 seconds (Linux / macOS)
curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh

# Or compile and install directly with Cargo
cargo install --path .
```

---

## ⚡ Why Tagisan? Real Benchmark Comparison

| Metric / Capability | **Tagisan (`tgs`)** | **Ollama (Go)** | **CrewAI** | **AutoGen** | **LangGraph** |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Language** | **Systems-Grade Rust (Tokio)** | Go + C++ | Python | Python | Python / TypeScript |
| **Packaging** | **Single 20 MB Static Binary** | 50+ MB Binary | 140+ pip packages | 95+ pip packages | Heavy dependencies |
| **Cold Startup Time** | **~2.1 ms** | ~450 ms | ~1,850 ms | ~1,400 ms | ~900 ms |
| **Idle Memory Footprint** | **~10 MB - 18 MB** | ~110 MB | ~480 MB | ~520 MB | ~350 MB |
| **GGUF Mapping Latency** | **0.056 ms (56 µs via `mmap`)** | 100 - 300 ms | N/A | N/A | N/A |
| **Native Local Server Daemon** | **Yes (`tgs serve`)** | Yes | No | No | No |
| **Anti-Timeout Heartbeat** | **Yes (3s Keepalive Pulse)** | No (Prone to 600s drop) | No | No | No |
| **Adversarial Debate** | **Yes (`Lakandiwa` Engine)** | No | Custom scripts | Chat loops | Custom subgraphs |
| **Mixture-of-Agents (MoA)** | **Yes (Lock-free Tokio Channels)** | No | No | No | Custom setup |
| **Skill JIT Paging (PILOT)** | **Yes (L1 Context / L2 RAM / L3 Disk)** | No | No | No | No |
| **Filesystem Safety Guard** | **Surgical Edits + Atomic Trash Bin** | No | Plain OS writes | Docker requirement | Plain OS writes |
| **Security Firewall** | **Native AgentShield (Pre-flight)** | None | None | Docker sandboxing | None |
| **Model Context Protocol** | **Bidirectional MCP (Client + Server)** | None | Client only | Community wrappers | Community wrappers |

---

## 🚀 Key Architectural Superpowers

```mermaid
graph TD
    CLI["CLI Binary (tgs)"] --> Engine["Tagisan Core Engine"]
    Engine --> TensorEngine["Native GGUF Tensor Engine & Server"]
    Engine --> Swarm["Autonomous Swarm & Strategies"]
    Engine --> Providers["Provider Hub (Cloud & Local)"]
    Engine --> Tools["Tool Registry & Surgical File CRUD"]
    Engine --> Colibri["5 Colibrì Superpowers"]
    Engine --> Polyglot["Polyglot Runtimes (Python, Perl, Bun)"]
    Engine --> Skills["AI Skill Dispatcher (3,900+ Skills)"]
    
    TensorEngine --> GGUF["Zero-Copy GGUF v2/v3 Mmap Parser (56µs)"]
    TensorEngine --> Resolver["OllamaBlobResolver (Direct ~/.ollama Blobs)"]
    TensorEngine --> Server["tgs serve (Tokio HTTP Daemon on :11434)"]
    TensorEngine --> Heartbeat["3s Keepalive Heartbeat Pulse"]

    Swarm --> Debate["Dialectical Debate (Thesis -> Antithesis -> Synthesis)"]
    Swarm --> MoA["Mixture-of-Agents (Parallel Proposers -> Aggregator)"]
    Swarm --> Harmony["Structured Harmony Assembly Line (RFC-003)"]
    Swarm --> DAG["Petgraph 5-Stage Parallel DAG Pipeline"]
    Swarm --> Consensus["Quadratic & Borda Consensus Voting"]

    Providers --> Cloud["Claude 3.5, Gemini 2.0, GPT-4o, Grok-2, DeepSeek-R1"]
    Providers --> Embedded["In-Process Embedded LLM ($0.00 Infinite Run)"]
    Providers --> Ollama["Local Ollama (Abliterated / Unrestricted Mode)"]
    Providers --> ColibriMoE["Colibrì Local MoE (Dual-SSD Striped ~14.8 GB/s)"]

    Tools --> CRUD["Surgical File CRUD (edit_file, delete_file, list_dir, read, write)"]
    Tools --> Vision["Multimodal Computer Vision (view_image, -i)"]
    Tools --> Shield["AgentShield Pre-Flight Safety Firewall"]

    Colibri --> SkillJIT["Skill JIT Paging & Lookahead (PILOT)"]
    Colibri --> Atlas["Live Cortex Swarm Atlas & Routing Heat"]
    Colibri --> P2P["Distributed Local P2P Cluster Mesh (Port 8765)"]
    Colibri --> Invariant["Semantic Invariant Guard (Zero Degradation)"]
```

---

## 🌟 Core Feature Highlights

### 1. 🦀 Reverse-Engineered Ollama Rust Tensor Engine (`tgs serve` & `tgs engine`)
Tagisan reverse-engineers and integrates the best parts of Ollama directly into systems-grade Rust:
- **Zero-Copy GGUF v2/v3 Parser (`src/engine/gguf.rs`):** Maps 2+ GB model weights directly into the operating system page cache using `memmap2`. Binary header and 256 tensor structures are mapped in **56 microseconds (0.056 ms)** with zero heap allocation bloat.
- **`OllamaBlobResolver`:** Directly discovers installed models from `~/.ollama/models/manifests/` and maps friendly queries (`"abliterated"`, `"llama3.2-abliterate:3b-instruct"`) directly to physical weight blobs (`sha256-fdc5784e...`) without re-downloading a single byte.
- **Standalone Tokio HTTP Server (`tgs serve`):** A drop-in, Ollama-compatible daemon written in pure asynchronous Rust (`tokio`). Serves `/api/version`, `/api/tags`, `/api/show`, and `/api/chat` over streaming NDJSON. Uses **~10 MB RAM** with zero Go runtime pauses. External tools (VS Code Continue, Cursor, OpenWebUI, Obsidian) connect seamlessly!
- **Deep Model Inspection (`tgs engine inspect <model>`):** Inspects GGUF architecture, context length, embedding length, block counts, attention heads, tokenizer chat templates, and tensor shapes.

### 2. 💓 Anti-Timeout Heartbeat Pulse & Uncensored Abliterated Execution
- **The 3-Second Heartbeat Pulse:** On low-power or dual-core laptops (e.g., Intel Core i3), CPU prompt ingestion of large codebases can take 30–60 seconds before emitting the first token. Standard HTTP clients drop the connection with `TimedOut`. Tagisan emits continuous non-breaking heartbeat keepalives every 3 seconds, keeping connections alive indefinitely.
- **Auto-Sequential Concurrency (`concurrency_limit = 1`):** CPU-bound local models are automatically scheduled sequentially to prevent L1/L2 cache trashing and eliminate 100% RAM/swap freezes.
- **Unrestricted / Abliterated Mode:** Full native support for abliterated models (`llama3.2-abliterate:3b-instruct`). Mathematical refusal vectors are eliminated, allowing security agents to audit exploit payloads, reverse-engineer proprietary protocols, and analyze low-level binaries without corporate safety refusals.

### 3. ⚔️ Dialectical Debate (*Tagisan ng Talino / Balagtasan*)
Single-model prompts hallucinate and overlook architectural traps. Tagisan pits models into structured, adversarial debates:
- **Round 1 (Thesis):** Proponent model proposes an initial implementation or architecture.
- **Round 2 (Antithesis):** Adversary model probes for race conditions, security flaws, and edge cases.
- **Round 3 (Synthesis / Lakandiwa):** Chief Adjudicator synthesizes both perspectives, balances real-world trade-offs, and issues a final, mathematically grounded verdict.
- **100% Offline Support:** Runs seamlessly using local Ollama or embedded GGUF models in rapid rotation with **$0.00 API cost**.

### 4. 🛖 Mixture-of-Agents (MoA) & Parallel Proposers
- **Layer 1 (Parallel Proposers):** Dispatches the prompt simultaneously across multiple local and cloud LLMs over non-blocking Tokio channels.
- **Layer 2 (Master Aggregator):** The designated lead model scores, filters, and stitches together the finest elements of every proposal into a superior composite response.

### 5. 🦅 The 5 Colibrì Superpowers
Directly adapted and enhanced from high-performance local MoE systems:
1. **Skill JIT Paging & Lookahead Prefetch Engine (PILOT):** 3-tier memory model (**L1** Active Context ↔ **L2** Warm RAM AST ↔ **L3** Cold NVMe) with 1-step lookahead prefetching based on planned steps and heat-weighted pinning.
2. **Agent & Skill Atlas with Routing Heat Tracking (`tgs swarm atlas`):** Dynamic cortex dashboard tracking empirical agent invocations, decay heat, and domain topic clustering (Systems, Forensics, Strategy, Security).
3. **Native Colibrì Inference Provider & Dual-SSD Model Weight Striping:** Supports streaming model weights across dual NVMe drives (`COLI_MODEL_MIRROR`, `COLI_DISK_WEIGHTS`) up to ~14.8 GB/s throughput.
4. **Distributed Local P2P Cluster Mesh (`tgs swarm cluster`):** Tokio TCP coordinator and worker mesh on port `8765` distributing tool and skill tasks across local LAN compute nodes.
5. **Semantic Invariant Guard:** Enforces the *"No SLA on Speed, Hard Guarantee on Semantics"* law—rejects lossy tool schema truncations and guarantees complete compiler diagnostic preservation under context exhaustion.

### 6. 🛠️ Native First-Class Surgical File CRUD Superpowers
Equips autonomous agents and CLI workflows with safe, surgical file operations:
- **`edit_file` (Surgical Patcher):** Replaces exact substrings or line ranges (`start_line`, `end_line`) without rewriting entire multi-thousand-line files. Includes atomic temporary writes and automated `.bak` backup retention.
- **`delete_file` (Safe Recycler):** Features soft-delete recycling into `.tagisan/trash/<timestamp>_<file>` with cross-filesystem move fallbacks and permanent deletion options.
- **`list_dir` (Structured Explorer):** Formats directory trees with entry types (`[FILE]`, `[DIR]`, `[SYMLINK]`), exact byte sizes, child counts, traversal depth limits (`max_depth`), and pattern filters.
- **`read_file` & `write_file`:** Safe async reading and automatic parent directory creation.
- **Git Worktree Sandboxing (`--sandbox`):** Branches tasks into ephemeral worktrees (`tagisan/sandbox-...`), allowing safe diff inspection (`sandbox.diff()`), committing, or clean rollbacks.

### 7. 👁️ Multimodal Computer Vision
- **Direct CLI Ingestion (`-i / --image`):** Pass screenshots, architecture diagrams, UI mockups, or error traces directly to vision-capable models (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`).
- **Autonomous Agent Inspection (`view_image`):** Agents can autonomously inspect local image files, validate formats, compute sizes, and feed Base64 payloads into multimodal reasoning loops.
- **Broad Model Support:** Fully compatible with Google Gemini 2.0 Flash / Pro, Anthropic Claude 3.5 Sonnet, OpenAI GPT-4o, Grok-2 Vision, and lightweight local Ollama models (`moondream:1.8b`, `llava-phi3:3.8b`).

### 8. 🔌 Model Context Protocol (MCP) Client & Server
- **Universal MCP Client (`tgs mcp`):** Connects via JSON-RPC 2.0 stdio to any MCP server (PostgreSQL, SQLite, GitHub, Filesystem, Memory) with namespace isolation (`<server>__<tool>`).
- **Native MCP Server (`tgs serve-mcp`):** Turn Tagisan into an MCP server, exposing its swarms, dialectical debate, consensus engines, and codebase RAG tools to **Claude Desktop**, **Cursor**, **Zed**, and **Windsurf**.

### 9. 🐍 🐪 🥟 Polyglot Sandboxed Runtimes
Execute code snippets and automated scripts on the fly with built-in AgentShield guardrails:
- **Python 3 (`python_eval`, `python_run`):** Execution with virtualenv discovery, timeouts, and buffer clamps.
- **Perl 5 (`perl_eval`, `perl_run`):** Lightning-fast regex and text transformations.
- **Bun / TypeScript (`bun_eval`, `bun_run`, `bun_test`, `bun_serve`):** Native TypeScript execution, test running, and live microservices.

### 10. 📚 3,900+ Engineering Skills & Curated 250+ Vibe-Coder Canons
- **Sub-Millisecond Semantic Dispatcher:** Matches developer objectives to authoritative skills in ~150 microseconds.
- **Curated 250+ Book Knowledge Canons:** Authoritative knowledge bases mathematically validated with Modulo-10 and Modulo-11 ISBN checksums:
  * 🔍 *Systems Failure & Forensics Analysis* (`failure-forensic-expert`)
  * ⚔️ *Strategic & Competitive Moat Analysis* (`strategic-competitive-analyst`)
  * 🎨 *Modern Web Design & Dev Engineering* (`web-design-dev-vibe-coder`)
  * 🧠 *Philosophical & Conceptual Analysis* (`philosophical-conceptual-vibe-coder`)
  * 🪙 *Cryptoeconomics & Web3 Market Microstructure* (`crypto-market-analyst`)
- **Extensible On-Disk System:** Discover and dynamically equip custom skills stored in `.ecc/skills/`.

---

## 💻 CLI Usage Cheatsheet

### 1. Zero-Copy GGUF Tensor Engine & Local Daemon
```bash
# List all local Ollama GGUF models discovered on disk
tgs engine list

# Deeply inspect GGUF binary header, 256 tensors, quant types, and chat template
tgs engine inspect abliterated
tgs engine inspect ~/.ollama/models/blobs/sha256-fdc5784e...

# Launch the native Tokio Ollama-compatible HTTP server daemon (port 11434 or 11435)
tgs serve --port 11434
```

### 2. Interactive Queries & Streaming (Zero-Cost Local & Cloud)
```bash
# Query using local abliterated model with 3-second heartbeat pulse
tgs ask -p ollama "Write an async HTTP health checker using Tokio"

# Stream real-time token output with sub-millisecond dispatch
tgs stream -p ollama -m llama3.2-abliterate:3b-instruct "Implement a lock-free RingBuffer in Rust"

# Query the best available cloud model
tgs ask "Explain Amdahl's Law vs Gustafson's Law"
```

### 3. Computer Vision & Screenshot Debugging
```bash
# Debug an error trace screenshot with free cloud Gemini
tgs ask -p gemini -i /path/to/screenshot.png "What caused this traceback and how do I fix it?"

# Run vision 100% locally with Moondream (fits 8GB laptops!)
tgs ask -p ollama -m moondream -i ./ui_mockup.png "Describe the UI layout and identify missing accessibility attributes"

# Autonomous agent solving a visual UI bug
tgs agent -i ./error.png "Analyze this screenshot, locate the faulty code in src/, and repair it"
```

### 4. Dialectical Debate (*Tagisan ng Talino*)
```bash
# 100% Offline Local Debate using rotating Ollama models ($0.00 cost)
tgs debate -p ollama "Postgres JSONB vs Relational Tables for high-throughput events"

# Cloud Debate (Claude vs DeepSeek with Gemini Lakandiwa Synthesis)
tgs debate "Should we build our backend with Rust microservices or a modular Go monolith?"

# Watch the debate live in an interactive terminal UI
tgs debate --tui "Rust vs Zig for bare-metal systems"
```

### 5. Autonomous Multi-Turn Agent Loop
```bash
# Autonomous code inspection and enhancement
tgs agent "Scan src/engine/gguf.rs, identify optimization flags, and explain them"

# Autonomous run with persistent codebase memory and isolated Git sandbox
tgs agent "Refactor our vector memory distance functions" --memory --sandbox

# Autonomous run with external MCP tools enabled
tgs agent "Analyze our local database schema" --mcp
```

### 6. Multi-Agent Swarm Orchestration & Consensus
```bash
# Lead Architect delegating tasks to TDD and Security specialists
tgs swarm run "Design and implement a zero-allocation token bucket" \
  --agents architect,tdd-engineer,security-auditor --lead architect

# Mathematical Peer Review & Consensus Voting on code
tgs consensus src/agent/mod.rs --rule unanimous

# Position-based Borda Count consensus ranking
tgs consensus "Raft vs Paxos for consensus engine" --rule borda

# Inspect live Swarm Atlas & routing heat cortex dashboard
tgs swarm atlas
```

### 7. Structured Role-Based Harmony Swarm (`tgs harmony` / Bayanihan)
```bash
# Standard 4-stage assembly line (Architect -> Implementer -> QA -> Doc)
tgs harmony build "Implement an async rate limiter in Rust using token bucket"

# Smart Tier: Claude (Architect/QA) + DeepSeek (Implementer) + Gemini (Docs)
tgs harmony build "Build a high-throughput microservice router" --tier smart --parallel

# Economy Tier: High-speed, sub-cent execution with Gemini 2.0 Flash
tgs harmony build "Generate a CLI argument parser" --tier economy --parallel
```

### 8. Distributed Local P2P Cluster Mesh
```bash
# Start a mesh coordinator on TCP port 8765
tgs swarm cluster coordinator --bind 0.0.0.0:8765

# Join the cluster from another machine as a compute/tool worker
tgs swarm cluster worker --coordinator 192.168.1.100:8765 --worker-id worker-nvme-01

# Query active cluster nodes and capabilities
tgs swarm cluster status --coordinator 192.168.1.100:8765
```

---

## 🛠️ Configuration & Environment

Tagisan works with **zero configuration** out of the box using your local Ollama installation. To customize behavior or enable cloud models, configure `.env`:

```env
# =========================================================
# 🇵🇭 TAGISAN: LOCAL OLLAMA UNRESTRICTED CONFIGURATION
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

# Optional Cloud Model API Keys
GEMINI_API_KEY=AIzaSy...             # Google AI Studio (Free Tier Supported)
DEEPSEEK_API_KEY=sk-...              # DeepSeek R1 / V3
ANTHROPIC_API_KEY=sk-ant-api03-...   # Claude 3.5 Sonnet / Opus
OPENAI_API_KEY=sk-proj-...           # GPT-4o / GPT-4o-mini
XAI_API_KEY=xai-...                  # Grok-2 / Grok-Vision
```

Check system status and active bitflags at any time:
```bash
tgs status
```

---

## 🧪 Comprehensive Verification & Test Suite

Tagisan is verified by automated integration stress tests achieving a **100% pass rate**:

```bash
# Run all standard unit and integration tests (21 passed, 0 failed)
cargo test

# Brutal verification: Reverse-Engineered Ollama Rust Tensor Engine (6 Tiers)
python3 scripts/test_rust_engine_verification.py

# Brutal verification: 5 Colibrì Superpowers (JIT Paging, Atlas, MoE, P2P Mesh, Guard)
python3 scripts/test_colibri_superpowers.py

# Brutal verification: Native File CRUD Superpowers (Edit, Delete, List, Trash, Shield)
python3 scripts/test_file_crud_superpowers.py

# Brutal verification: Curated Vibe-Coder Knowledge Canons (ISBN Checksums & Skills)
python3 scripts/test_forensics_verification.py
python3 scripts/test_strategic_verification.py
python3 scripts/test_crypto_market_verification.py
```

---

## 🗺️ Architectural Specifications & Roadmaps

- 📖 **[Reverse-Engineered Ollama Rust Tensor Engine Guide](docs/REVERSE_ENGINEERED_OLLAMA_RUST_ENGINE_GUIDE.md)** — In-depth architectural guide for zero-copy GGUF v2/v3 parsing, memory mapping, and Tokio HTTP streaming.
- 📑 **[RFC-001: Universal Protocol & Ecosystem Integrations](ROADMAP_EXTENSIONS.md)** — Pluggable external vector backends, OpenTelemetry tracing, and automated swarm benchmarking.
- 🔌 **[RFC-002: WASM & Native Dynamic Plugins Architecture](ROADMAP_PLUGINS.md)** — Extism WebAssembly sandboxing and native dynamic shared library plugins.
- 🏛️ **[RFC-003: Structured Role-Based Harmony Swarms](docs/SPEC_STRUCTURED_ROLE_HARMONY_SWARM.md)** — Deterministic multi-model assembly line, anti-sycophancy gates, and shared blackboard memory.
- 🧠 **[RFC-004: Semantic Skill Dispatching & JIT Knowledge Injection](docs/SPEC_LOCAL_LLM_SKILL_DISPATCHING.md)** — Top-K semantic skill auto-equipping and sub-millisecond cheat-sheet prompt injection.

---

## 📄 License

Dual-licensed under the **MIT License** and the **Apache 2.0 License**. See [LICENSE](LICENSE) for details.
