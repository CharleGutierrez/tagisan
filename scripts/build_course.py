#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Tagisan (tgs) Master Class Course Builder
Generates:
  1. docs/TGS_MASTERCLASS_COURSE.md
  2. docs/TGS_MASTERCLASS_COURSE.pdf
"""

import os
import sys
from datetime import datetime

# Import scenarios
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scenarios_part1 import DOMAINS_PART1
from scenarios_part2 import DOMAINS_PART2

ALL_DOMAINS = DOMAINS_PART1 + DOMAINS_PART2

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DOCS_DIR = os.path.join(REPO_ROOT, "docs")
os.makedirs(DOCS_DIR, exist_ok=True)

MD_PATH = os.path.join(DOCS_DIR, "TGS_MASTERCLASS_COURSE.md")
PDF_PATH = os.path.join(DOCS_DIR, "TGS_MASTERCLASS_COURSE.pdf")

def generate_markdown():
    lines = []
    
    # Title & Header
    lines.append("# Tagisan (`tgs`) Sovereign Autonomous Agent Architecture")
    lines.append("# The Definitive Master Class Course: From Zero to Autonomous Production Architect")
    lines.append("")
    lines.append("```")
    lines.append("   ████████╗ ██████╗ ███████╗")
    lines.append("   ╚══██╔══╝██╔════╝ ██╔════╝   TAGISAN (tgs / tagisan-rs)")
    lines.append("      ██║   ██║  ███╗███████╗   The Sovereign Dual-Brain Agent Architecture")
    lines.append("      ██║   ██║   ██║╚════██║   Production Master Class Course & Curriculum")
    lines.append("      ██║   ╚██████╔╝███████║   Version: 3.0 (Dual-Brain / BEAM-OTP / Gleam / MCP-500)")
    lines.append("      ╚═╝    ╚═════╝ ╚══════╝   Publication: 2026 Edition")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Executive Course Overview & Learning Objectives")
    lines.append("")
    lines.append("Welcome to the **Tagisan (`tgs`) Definitive Master Class Course**. Tagisan is a high-performance, sovereign autonomous agent engine engineered in Rust on top of the Tokio asynchronous work-stealing reactor, combined with an Erlang/Elixir BEAM OTP actor runtime and a native Gleam type-safe compiler. Designed as an uncompromised alternative to fragile, single-threaded Python agent frameworks, Tagisan delivers deterministic sub-millisecond task dispatching, dual-brain hybrid inference (local GGUF/Ollama + Google Cloud Frontier models via AntiGravity CCPA OAuth), proactive AgentShield cyber defense, and native orchestration of 500 Model Context Protocol (MCP) plug-ins.")
    lines.append("")
    lines.append("### Core Competencies You Will Acquire:")
    lines.append("1. **Dual-Brain Cognitive Architecture**: Seamless failover between local edge tensors (Ollama/GGUF) and cloud frontier models (Google Gemini 2.5/3.x, Anthropic Claude, OpenAI, DeepSeek, xAI).")
    lines.append("2. **Google Web OAuth & CCPA Integration**: Reverse-engineered AntiGravity 2.0 CLI authentication routing, proactive 90-second token auto-refresh, and multi-model dispatching (`flash`, `pro`, `lite`, `gemini-3`).")
    lines.append("3. **Polyglot Actor Concurrency (BEAM & Gleam)**: Type-safe Gleam actor microservices, external term format (ETF) binary serialization, and OTP supervisor trees with 'let it crash' fault isolation.")
    lines.append("4. **Hegelian Dialectical Debate Engine**: Multi-agent Swarm Mixture-of-Agents (MoA) featuring Thesis, Antithesis, Synthesis, and Judge consensus to eliminate hallucinations.")
    lines.append("5. **AgentShield Cyber Defense & Landlock Sandboxing**: Pre-execution AST security validation, Linux Landlock LSM isolation, and exfiltration prevention.")
    lines.append("6. **PILOT JIT Multi-Tier Memory**: Working memory (RAM), episodic session history (SQLite), and semantic vector memory (Qdrant / SQLite-vec) with Reciprocal Rank Fusion (RRF).")
    lines.append("7. **The 500 Dynamic Skills Ecosystem (RFC-004)**: Dynamic ingestion, skill lifecycle verification, and sovereign package authoring.")
    lines.append("8. **The Model Context Protocol (MCP 500) Hub**: Querying, installing, and executing 500 verified non-GitHub enterprise plugins across 10 strategic industry domains.")
    lines.append("9. **VELLA Cyber-Physical Digital Twins**: Real-time SCADA/IoT monitoring, quantitative finance risk controls, satellite orbital mechanics (SGP4), and genomic sequence analysis.")
    lines.append("10. **150 Production Scenarios**: Hands-on mastery across Cloud/SRE, Cybersecurity, Full-Stack, BEAM Concurrency, Big Data, AI/ML, Quantitative Trading, SCADA, Aerospace, and Genomics.")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## Comprehensive Curriculum Map")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    subgraph Foundations [Phase 1: Foundations & Concurrency]")
    lines.append("        M1[\"Module 1: Dual-Brain & Google OAuth\"] --> M2[\"Module 2: Gleam & BEAM/OTP Actors\"]")
    lines.append("        M2 --> M3[\"Module 3: CLI Ergonomics & Operations\"]")
    lines.append("    end")
    lines.append("    subgraph CognitionAndDefense [Phase 2: Cognition, Defense & Memory]")
    lines.append("        M3 --> M4[\"Module 4: AgentShield Cyber Defense\"]")
    lines.append("        M4 --> M5[\"Module 5: PILOT JIT Memory Hierarchy\"]")
    lines.append("        M5 --> M6[\"Module 6: Swarm MoA & Dialectical Debate\"]")
    lines.append("    end")
    lines.append("    subgraph Extensibility [Phase 3: Universal Extensibility]")
    lines.append("        M6 --> M7[\"Module 7: 500-Skill Engine (RFC-004)\"]")
    lines.append("        M7 --> M8[\"Module 8: Model Context Protocol (MCP 500)\"]")
    lines.append("    end")
    lines.append("    subgraph Production [Phase 4: Digital Twins & Production]")
    lines.append("        M8 --> M9[\"Module 9: Domain Copilots (VELLA Twins)\"]")
    lines.append("        M9 --> M10[\"Module 10: Enterprise Hardening & Air-Gap\"]")
    lines.append("        M10 --> M11[\"Module 11: 150 Production Scenarios\"]")
    lines.append("        M11 --> M12[\"Module 12: Capstone Certification Labs\"]")
    lines.append("    end")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")
    
    # Module 1
    lines.append("## Module 1: Dual-Brain Inference & Google Cloud Frontier Integration")
    lines.append("")
    lines.append("### 1.1 The Dual-Brain Architectural Paradigm")
    lines.append("Traditional AI agent frameworks force an undesirable binary choice:")
    lines.append("- **Cloud-Only Dependence**: High token costs, network latency, vendor lock-in, rate limit throttling, and catastrophic downtime when public APIs experience outages.")
    lines.append("- **Local-Only Constraints**: Limited model parameter sizes (7B-14B), reduced reasoning capacity for complex multi-file refactoring, and high local GPU/VRAM hardware requirements.")
    lines.append("")
    lines.append("Tagisan resolves this with **Dual-Brain Hybrid Failover**:")
    lines.append("")
    lines.append("```mermaid")
    lines.append("sequenceDiagram")
    lines.append("    autonumber")
    lines.append("    actor User as Engineer / User")
    lines.append("    participant TGS as Tagisan Engine Core")
    lines.append("    participant Local as Local Engine (Ollama / GGUF)")
    lines.append("    participant Auth as Google OAuth Manager")
    lines.append("    participant Cloud as Cloud Frontier (CCPA Gateway)")
    lines.append("    participant Shield as AgentShield Auditor")
    lines.append("")
    lines.append("    User->>TGS: tgs ask 'Complex System Refactor'")
    lines.append("    TGS->>Shield: Pre-Execution Semantic Inspection")
    lines.append("    Shield-->>TGS: Approved (Safe)")
    lines.append("    alt Google OAuth Active (Default)")
    lines.append("        TGS->>Auth: Check Token Expiry (90s Safety Cushion)")
    lines.append("        opt Token Expired or < 90s Left")
    lines.append("            Auth->>Auth: Background Refresh via refresh_token")
    lines.append("        end")
    lines.append("        TGS->>Cloud: POST daily-cloudcode-pa.googleapis.com (User-Agent: antigravity/2.0.0)")
    lines.append("        Cloud-->>TGS: High-Parameter Frontier Response")
    lines.append("    else Logged Out or Offline")
    lines.append("        TGS->>Local: Zero-Cost Offline Inference (smollm2 / qwen2.5)")
    lines.append("        Local-->>TGS: Local Generation Output")
    lines.append("    end")
    lines.append("    TGS->>User: Streamed Sovereign Output")
    lines.append("```")
    lines.append("")
    lines.append("### 1.2 Google Web OAuth Architecture & AntiGravity CCPA Endpoint")
    lines.append("Tagisan integrates directly with Google's internal CloudCode Platform Assistant (CCPA) backend, mirroring the authentication and request envelope used by Google's official AntiGravity 2.0 CLI:")
    lines.append("- **Endpoint**: `https://daily-cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse`")
    lines.append("- **User-Agent**: `antigravity/2.0.0`")
    lines.append("- **Payload Envelope**: `{ project: \"default-cli-project\", model: \"<model-id>\", request: { contents: [...] } }`")
    lines.append("- **Proactive Token Refresh**: Tagisan checks `expires_at <= (now + 90s)`. If an access token expires mid-task, it silently refreshes in the background without interrupting the user.")
    lines.append("")
    lines.append("### 1.3 Supported Google Models & Shorthand Aliases")
    lines.append("| Shorthand Flag | Target Model ID | Capability Profile | Ideal Production Use Case |")
    lines.append("| :--- | :--- | :--- | :--- |")
    lines.append("| **`-m flash`** (Default) | `gemini-2.5-flash` | High speed, balanced intelligence | Daily development, code review, git operations |")
    lines.append("| **`-m pro`** | `gemini-3.1-pro-low` | Deep multi-step reasoning, thinking chain | Complex architectural design, mathematical proofs |")
    lines.append("| **`-m lite`** | `gemini-2.5-flash-lite` | Sub-300ms time-to-first-token | Inline shell completions, quick summaries |")
    lines.append("| **`-m gemini-3`** | `gemini-3.6-flash-medium` | Next-gen reasoning with medium effort | Algorithmic refactoring, security vulnerability audits |")
    lines.append("| **`-m gemini-3.1-flash-lite`** | `gemini-3.1-flash-lite` | Ultra-compact next-gen lightweight | Edge automation, high-frequency IoT pipelines |")
    lines.append("")
    lines.append("### 1.4 Automatic Provider Resolution & Graceful Logout Fallback")
    lines.append("Tagisan resolves inference providers with strict deterministic precedence:")
    lines.append("1. **Explicit CLI Flag**: `-p / --provider <id>` (highest precedence).")
    lines.append("2. **Air-Gap Privacy Lock**: `TAGISAN_LOCAL_ONLY=1` or `TAGISAN_OFFLINE=1` forces local Ollama regardless of credentials.")
    lines.append("3. **Active Google OAuth Session**: Automatically routes to Google Gemini if `~/.config/tagisan/gemini_oauth.json` is present.")
    lines.append("4. **Cloud API Keys**: Checks `GEMINI_API_KEY`, `DEEPSEEK_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `XAI_API_KEY`.")
    lines.append("5. **Local Ollama Fallback**: If logged out via `tgs auth logout gemini`, Tagisan resets `TAGISAN_PROVIDER=auto` and falls back to installed local Ollama models (`smollm2:1.7b`, `qwen2.5-coder:1.5b`) without error.")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 2
    lines.append("## Module 2: Polyglot Concurrency: Native Gleam & BEAM/OTP Actor Runtime")
    lines.append("")
    lines.append("### 2.1 The Erlang/Elixir BEAM Actor Philosophy")
    lines.append("In high-throughput agent systems, shared-memory multi-threading leads to lock contention, race conditions, and deadlocks. Tagisan incorporates the **Erlang BEAM actor model**, where every autonomous agent task executes inside an isolated, lightweight process with its own private heap and message mailbox.")
    lines.append("")
    lines.append("```mermaid")
    lines.append("graph TD")
    lines.append("    RootSup[\"Root Supervisor (one_for_all)\"] --> AgentSup[\"Agent Supervisor (one_for_one)\"]")
    lines.append("    RootSup --> SecuritySup[\"AgentShield Supervisor (one_for_one)\"]")
    lines.append("    RootSup --> StorageSup[\"Memory Supervisor (rest_for_one)\"]")
    lines.append("    ")
    lines.append("    AgentSup --> Worker1[\"Worker Actor 1 (Analysis)\"]")
    lines.append("    AgentSup --> Worker2[\"Worker Actor 2 (Code Gen)\"]")
    lines.append("    AgentSup --> Worker3[\"Worker Actor 3 (Testing)\"]")
    lines.append("    ")
    lines.append("    StorageSup --> CacheActor[\"Cache GenServer\"]")
    lines.append("    StorageSup --> SQLiteActor[\"SQLite Persistent Writer\"]")
    lines.append("```")
    lines.append("")
    lines.append("### 2.2 Supervisor Restart Strategies")
    lines.append("- **`one_for_one`**: If a child actor process crashes, only that specific actor is restarted. Sibling actors continue running uninterrupted.")
    lines.append("- **`one_for_all`**: If any child process crashes, all sibling processes in the pool are terminated and restarted in sequence.")
    lines.append("- **`rest_for_one`**: If an actor crashes, only subsequent dependent actors spawned after it are restarted, preserving upstream state.")
    lines.append("")
    lines.append("### 2.3 External Term Format (ETF) Binary Interop")
    lines.append("Tagisan implements native encoding and decoding of Erlang's **External Term Format (ETF format 131)** in Rust. This enables zero-copy, binary-packed inter-process communication between Rust core threads, Elixir GenServers, and Gleam actors at over 800,000 messages per second.")
    lines.append("")
    lines.append("### 2.4 Type-Safe Gleam Integration")
    lines.append("Gleam brings a friendly, statically typed, and mathematically sound type system to the BEAM ecosystem. Tagisan compiles Gleam code directly to BEAM bytecode, providing compile-time exhaustive pattern matching that eliminates unhandled edge cases before execution:")
    lines.append("")
    lines.append("```gleam")
    lines.append("// Example: Sovereign State Transition in Gleam")
    lines.append("pub type AgentState {")
    lines.append("  Idle")
    lines.append("  Analyzing(query: String)")
    lines.append("  ExecutingTool(tool_name: String, args: String)")
    lines.append("  Completed(result: String)")
    lines.append("  Failed(error_code: Int, reason: String)")
    lines.append("}")
    lines.append("")
    lines.append("pub fn transition(state: AgentState, event: Event) -> AgentState {")
    lines.append("  case state, event {")
    lines.append("    Idle, Start(q) -> Analyzing(q)")
    lines.append("    Analyzing(_), ToolNeeded(name, args) -> ExecutingTool(name, args)")
    lines.append("    ExecutingTool(_, _), Success(res) -> Completed(res)")
    lines.append("    ExecutingTool(_, _), Error(code, msg) -> Failed(code, msg)")
    lines.append("    _, Reset -> Idle")
    lines.append("  }")
    lines.append("}")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 3
    lines.append("## Module 3: Operational Command Mastery & CLI Ergonomics")
    lines.append("")
    lines.append("### 3.1 The Complete CLI Command Atlas")
    lines.append("Tagisan provides an intuitive, high-speed CLI toolchain (`tgs`):")
    lines.append("")
    lines.append("```bash")
    lines.append("# One-shot prompt with automatic provider & model selection")
    lines.append("tgs ask \"Explain the difference between TCP and UDP in one sentence\"")
    lines.append("")
    lines.append("# Use specific model aliases via Google OAuth")
    lines.append("tgs ask -m pro \"Design a high-throughput event sourcing architecture\"")
    lines.append("tgs ask -m lite \"Summarize this log line: [ERR 502 Bad Gateway]\"")
    lines.append("")
    lines.append("# Interactive terminal chat session with persistent context")
    lines.append("tgs chat -m pro")
    lines.append("")
    lines.append("# Autonomous mission execution with tool invocation and AST sandboxing")
    lines.append("tgs run \"Find all unindexed foreign keys in migrations/ and generate SQL indexes\"")
    lines.append("")
    lines.append("# Dialectical Multi-Agent Debate")
    lines.append("tgs debate \\")
    lines.append("  --proposer \"Adopt Rust for our core financial transaction ledger\" \\")
    lines.append("  --challenger \"Highlight developer velocity and hiring difficulties\" \\")
    lines.append("  --rounds 3")
    lines.append("")
    lines.append("# Search and browse the 500-skill repository")
    lines.append("tgs skills list")
    lines.append("tgs skills search \"kubernetes\"")
    lines.append("tgs skills inspect \"k8s-cluster-auditor\"")
    lines.append("")
    lines.append("# Model Context Protocol (MCP) plugin lifecycle")
    lines.append("tgs mcp catalog --domain \"Database\"")
    lines.append("tgs mcp search \"redis\"")
    lines.append("tgs mcp add redis-cache-inspector-mcp")
    lines.append("tgs mcp verify")
    lines.append("tgs mcp call redis-cache-inspector-mcp get_key '{\"key\":\"session:1001\"}'")
    lines.append("")
    lines.append("# Serve Tagisan as an MCP Server over stdio for Cursor, Windsurf, or Claude Desktop")
    lines.append("tgs serve-mcp")
    lines.append("")
    lines.append("# Authentication lifecycle")
    lines.append("tgs auth status")
    lines.append("tgs auth login gemini")
    lines.append("tgs auth logout gemini")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 4
    lines.append("## Module 4: AgentShield Cyber Defense & Kernel Sandboxing")
    lines.append("")
    lines.append("### 4.1 The Autonomous Threat Landscape")
    lines.append("When autonomous AI agents execute terminal commands, evaluate scripts, and ingest web content, they face three primary attack vectors:")
    lines.append("1. **Indirect Prompt Injection**: Malicious instructions embedded in scanned web pages, GitHub issues, or README files that hijack agent control flow.")
    lines.append("2. **Catastrophic Shell Execution**: Accidental or maliciously induced shell commands that delete filesystems (`rm -rf /`), format block devices, or deploy fork bombs.")
    lines.append("3. **Credential Exfiltration**: Prompts that attempt to extract `.env`, `~/.ssh/id_rsa`, `/etc/shadow`, or cloud secret keys.")
    lines.append("")
    lines.append("### 4.2 How AgentShield Enforces Security")
    lines.append("AgentShield operates at two distinct pre-execution checkpoints:")
    lines.append("1. **Semantic AST Inspection**: Intercepts tool calls and shell strings *before* process spawning. Parses command syntax trees to detect dangerous tokens and blacklisted operations.")
    lines.append("2. **Linux Landlock LSM Kernel Isolation**: Unprivileged Landlock security modules lock down filesystem access, strictly confining agent subprocesses to safe scratch directories (e.g. `/tmp/scratch` and the current workspace).")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart LR")
    lines.append("    ToolCall[\"Agent Proposes Tool Call\"] --> AST[\"AgentShield AST Scanner\"]")
    lines.append("    AST --> Check1{\"Blacklisted Syscall or Shell Bomb?\"}")
    lines.append("    Check1 -- Yes --> Block1[\"🚨 Hard Block & SIGKILL\"]")
    lines.append("    Check1 -- No --> Check2{\"Exfiltration Target? (.env, id_rsa)\"}")
    lines.append("    Check2 -- Yes --> Block2[\"🚨 Access Denied & Alert\"]")
    lines.append("    Check2 -- No --> Landlock[\"Landlock LSM Sandbox Jail\"]")
    lines.append("    Landlock --> Exec[\"Safe Sovereign Execution\"]")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 5
    lines.append("## Module 5: PILOT Just-In-Time Multi-Tier Memory")
    lines.append("")
    lines.append("### 5.1 The 3-Tier Cognitive Memory Hierarchy")
    lines.append("Tagisan organises memory into three purpose-built storage tiers:")
    lines.append("")
    lines.append("| Memory Tier | Backing Store | Lifetime | Access Latency | Primary Role |")
    lines.append("| :--- | :--- | :--- | :--- | :--- |")
    lines.append("| **Tier 1: Working Memory** | Tokio In-Memory RAM | In-flight mission turn | < 1 millisecond | Active conversation scratchpad, scratch variables |")
    lines.append("| **Tier 2: Episodic Memory** | SQLite / DuckDB | Persistent across sessions | < 5 milliseconds | Complete session audit logs, past tool outputs, rollbacks |")
    lines.append("| **Tier 3: Semantic Memory** | Qdrant / SQLite-vec | Indefinite persistent | < 15 milliseconds | Vector embeddings, code graph indices, domain knowledge |")
    lines.append("")
    lines.append("### 5.2 Hybrid Retrieval with Reciprocal Rank Fusion (RRF)")
    lines.append("To overcome vector-only semantic drift and keyword-only lexical blindness, Tagisan combines dense neural embeddings with sparse BM25 token indices using **Reciprocal Rank Fusion (RRF)**:")
    lines.append("$$RRF(d) = \\sum_{m \\in M} \\frac{1}{k + r_m(d)}$$")
    lines.append("Where $k = 60$ and $r_m(d)$ is the rank of document $d$ within retriever $m$. This mathematical fusion ensures that exact symbol matches (e.g., function names, variable identifiers) and high-level conceptual descriptions receive optimal relevance scoring.")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 6
    lines.append("## Module 6: The 500-Skill Sovereign Catalog (RFC-004)")
    lines.append("")
    lines.append("### 6.1 The RFC-004 Dynamic Skill Standard")
    lines.append("Every skill in Tagisan is a self-contained, versioned capability defined under `.ecc/skills/<skill-name>/SKILL.md` with structured YAML frontmatter:")
    lines.append("")
    lines.append("```markdown")
    lines.append("---")
    lines.append("name: postgresql-zero-downtime-migrator")
    lines.append("description: Plans and executes zero-downtime PostgreSQL schema migrations with lock timeout guards.")
    lines.append("version: 1.0.0")
    lines.append("tags: [database, postgresql, dba, performance]")
    lines.append("runtime: rust")
    lines.append("allow_network: false")
    lines.append("entrypoint: src/migrator.rs")
    lines.append("---")
    lines.append("")
    lines.append("# Sovereign Agent Instructions")
    lines.append("1. Always check pg_stat_activity before executing DDL.")
    lines.append("2. Set lock_timeout = '2s' before acquiring ACCESS EXCLUSIVE locks.")
    lines.append("3. Use CREATE INDEX CONCURRENTLY for all index additions.")
    lines.append("```")
    lines.append("")
    lines.append("### 6.2 Managing and Chaining Skills")
    lines.append("Skills can be dynamically chained together in complex multi-step missions:")
    lines.append("```bash")
    lines.append("# Invoke chained skills in a single pipeline")
    lines.append("tgs run --skill \"k8s-pod-diagnostics,postgresql-zero-downtime-migrator\" \\")
    lines.append("  \"Diagnose database connection timeouts and optimize connection pooling\"")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 7
    lines.append("## Module 7: Model Context Protocol (MCP 500) Enterprise Hub")
    lines.append("")
    lines.append("### 7.1 The Model Context Protocol Standard")
    lines.append("The Model Context Protocol (MCP) standardizes how AI agents discover and execute external tools over JSON-RPC 2.0. Tagisan implements both **stdio** (sub-process) and **SSE** (Server-Sent Events) transports, equipped with a curated catalog of **500 enterprise plugins** sourced exclusively from verified registries (Smithery.ai, NPM, PyPI, Glama.ai, Composio, Cloudflare).")
    lines.append("")
    lines.append("### 7.2 The 10 Strategic MCP Domains")
    lines.append("1. **Search, Web Scraping & Deep Research** (50 plugins): Brave Search, Firecrawl, Tavily, Puppeteer.")
    lines.append("2. **Relational Databases, OLAP & Event Streaming** (60 plugins): PostgreSQL, MySQL, ClickHouse, Kafka, Snowflake.")
    lines.append("3. **Vector Databases & Semantic Memory** (40 plugins): Qdrant, Pinecone, Milvus, Chroma, Weaviate.")
    lines.append("4. **Cloud Infrastructure, Serverless & Edge** (55 plugins): AWS, GCP, Azure, Cloudflare Workers, Terraform.")
    lines.append("5. **DevOps, Containers & Kubernetes Orchestration** (50 plugins): Kubernetes, Docker, Helm, ArgoCD, Ansible.")
    lines.append("6. **Observability, APM & SRE Reliability** (45 plugins): Prometheus, Grafana, Datadog, Jaeger, OpenTelemetry.")
    lines.append("7. **Cybersecurity, EDR & Threat Defense** (50 plugins): Trivy, Slither, VirusTotal, Shodan, Wireshark.")
    lines.append("8. **Developer Productivity & Tracking** (50 plugins): GitHub, GitLab, Jira, Linear, Slack, Notion.")
    lines.append("9. **Enterprise SaaS, CRM & FinTech** (50 plugins): Salesforce, Stripe, HubSpot, QuickBooks, SAP.")
    lines.append("10. **AI Models, Multimodal & GPU Compute** (50 plugins): Ollama, HuggingFace, Replicate, vLLM, DeepSeek.")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 8
    lines.append("## Module 8: Domain Copilots & Digital Twin Engineering (VELLA)")
    lines.append("")
    lines.append("### 8.1 VELLA: The Cyber-Physical Digital Twin Engine")
    lines.append("VELLA bridges LLM cognitive reasoning with deterministic real-time cyber-physical systems:")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    VELLA[VELLA Digital Twin Core] --> SCADA[\"Industrial SCADA & IoT (Modbus / OPC-UA)\"]")
    lines.append("    VELLA --> Quant[\"Quantitative Finance & Risk (VaR / FIX)\"]")
    lines.append("    VELLA --> Aero[\"Aerospace & Satellites (SGP4 TLE / CCSDS)\"]")
    lines.append("    VELLA --> Bio[\"Bioinformatics & Genomics (FASTA / VCF)\"]")
    lines.append("    VELLA --> Web3[\"Web3 Guardian (Smart Contract Auditing)\"]")
    lines.append("```")
    lines.append("")
    lines.append("### 8.2 Live VELLA CLI Commands")
    lines.append("```bash")
    lines.append("# Industrial SCADA telemetry inspection")
    lines.append("tgs vella scada --endpoint \"tcp://192.168.1.100:502\" --analog 85.4 --alarm \"trip_cooling\"")
    lines.append("")
    lines.append("# Quantitative Forex margin and pip calculation")
    lines.append("tgs vella forex --pair \"EUR/USD\" --bid 1.0842 --ask 1.0844 --lot 10.0 --leverage 50.0")
    lines.append("")
    lines.append("# Aerospace satellite SGP4 orbit propagation")
    lines.append("tgs vella aerospace --minutes 90.0 --tle \"1 25544U...\"")
    lines.append("")
    lines.append("# Genomic sequence alignment")
    lines.append("tgs vella bio --target \"ACTGATCG\" --template \"ACTGATCG\" --ref-genome \"GRCh38\"")
    lines.append("```")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 9
    lines.append("## Module 9: Enterprise Hardening, Air-Gapping & Observability")
    lines.append("")
    lines.append("### 9.1 Zero-Trust Air-Gapped Deployment")
    lines.append("In classified or banking environments with zero outbound internet access:")
    lines.append("1. Compile `tagisan` release binary: `cargo build --release`.")
    lines.append("2. Set environment privacy lock: `export TAGISAN_LOCAL_ONLY=1` and `export TAGISAN_OFFLINE=1`.")
    lines.append("3. Deploy local Ollama with quantized GGUF weights (`smollm2`, `qwen2.5-coder`).")
    lines.append("4. AgentShield enforces complete offline sandboxing, guaranteeing zero network bytes leave the server rack.")
    lines.append("")
    lines.append("### 9.2 Observability & Cost Accounting")
    lines.append("Every Tagisan execution outputs structured OpenTelemetry JSON records tracking:")
    lines.append("- `correlation_id` and execution span hierarchy.")
    lines.append("- Exact wall-clock latency (ms).")
    lines.append("- Prompt tokens, completion tokens, and cached prompt tokens.")
    lines.append("- Exact monetary expenditure in USD ($0.0000 on Google OAuth and local Ollama).")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Module 10: 150 Scenarios
    lines.append("## Module 10: The Definitive 150 Modern Environment Scenarios")
    lines.append("")
    lines.append("This section documents **150 real-world, production-proven scenarios** across 10 strategic enterprise domains. Each scenario provides the exact operational objective, TGS capabilities leveraged, the command syntax, the automated execution flow, and the verifiable sovereign outcome.")
    lines.append("")

    for domain in ALL_DOMAINS:
        lines.append(f"### Domain {domain['range'][0]}–{domain['range'][1]}: {domain['icon']} {domain['name']}")
        lines.append("")
        for sc in domain["scenarios"]:
            lines.append(f"#### Scenario {sc['id']}: {sc['title']}")
            lines.append(f"- **TGS Capabilities**: `{sc['capability']}`")
            lines.append(f"- **Command**:")
            lines.append(f"  ```bash")
            lines.append(f"  {sc['command']}")
            lines.append(f"  ```")
            lines.append(f"- **Execution Flow**:")
            for step in sc['flow'].split('\n'):
                lines.append(f"  {step}")
            lines.append(f"- **Sovereign Outcome**: {sc['outcome']}")
            lines.append("")
        lines.append("---")
        lines.append("")

    # Module 11: Capstone Labs
    lines.append("## Module 11: Capstone Certification Labs & Practical Exams")
    lines.append("")
    lines.append("### Lab 1: Autonomous Self-Healing Code Pipeline")
    lines.append("**Objective**: Configure Tagisan to monitor a Git repository, detect build errors, invoke a 3-round dialectical debate between a Refactoring Agent and a Security Auditor, apply the fixes, and verify tests pass.")
    lines.append("- **Prerequisites**: `tgs run`, `tgs debate`, GitHub repository clone.")
    lines.append("- **Verification**: Run `cargo test` inside the target project; all tests must pass.")
    lines.append("")
    lines.append("### Lab 2: Enterprise Market Intelligence Swarm")
    lines.append("**Objective**: Build a multi-agent swarm orchestrating Brave Search MCP, PostgreSQL MCP, and Qdrant Vector Memory to monitor financial news filings and synthesize real-time market risk summaries.")
    lines.append("- **Prerequisites**: `tgs mcp add brave-search-mcp`, `tgs mcp add qdrant-vector-db-mcp`.")
    lines.append("- **Verification**: Structured risk report written to PostgreSQL and indexed into Qdrant.")
    lines.append("")
    lines.append("### Lab 3: Air-Gapped Cyber Defense Sandbox")
    lines.append("**Objective**: Deploy Tagisan with zero external network access (`TAGISAN_LOCAL_ONLY=1`), local Ollama Qwen2.5-Coder, and local SQLite memory. Verify that AgentShield intercepts all simulated exfiltration attempts.")
    lines.append("- **Verification**: Run `cargo test --test mcp_catalog_and_lifecycle_brutal_tests`; 8 of 8 tests pass.")
    lines.append("")
    lines.append("### Master Certification Checklist")
    lines.append("- [ ] Build and verify native `tgs` release binary (`v0.2.0`).")
    lines.append("- [ ] Connect Google Web OAuth (`tgs auth login gemini`) and test `-m pro` and `-m lite`.")
    lines.append("- [ ] Verify local Ollama offline fallback on logout.")
    lines.append("- [ ] Execute a 3-round Hegelian Dialectical Debate (`tgs debate`).")
    lines.append("- [ ] Install and execute an external MCP server from the 500 catalog.")
    lines.append("- [ ] Successfully execute at least 5 scenarios from Module 10.")
    lines.append("")

    content = "\n".join(lines)
    with open(MD_PATH, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"✅ Generated Markdown: {MD_PATH} ({len(content)} bytes, {len(lines)} lines)")
    return content

# -----------------------------------------------------------------------------
# ReportLab PDF Generation
# -----------------------------------------------------------------------------
def generate_pdf():
    from reportlab.lib.pagesizes import letter
    from reportlab.platypus import (
        SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak, KeepTogether, HRFlowable
    )
    from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
    from reportlab.lib import colors
    from reportlab.pdfgen import canvas

    class NumberedCanvas(canvas.Canvas):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)
            self._saved_page_states = []

        def showPage(self):
            self._saved_page_states.append(dict(self.__dict__))
            self._startPage()

        def save(self):
            num_pages = len(self._saved_page_states)
            for state in self._saved_page_states:
                self.__dict__.update(state)
                self.draw_decorations(num_pages)
                super().showPage()
            super().save()

        def draw_decorations(self, page_count):
            if self._pageNumber > 1:
                self.saveState()
                # Header
                self.setFont("Helvetica-Bold", 8)
                self.setFillColorRGB(0.12, 0.44, 0.72)
                self.drawString(54, 750, "TAGISAN (tgs) MASTER CLASS COURSE")
                self.setFont("Helvetica", 8)
                self.setFillColorRGB(0.4, 0.4, 0.4)
                self.drawRightString(612 - 54, 750, "Sovereign Autonomous Agent Architecture")
                self.setStrokeColorRGB(0.85, 0.85, 0.85)
                self.setLineWidth(0.5)
                self.line(54, 744, 612 - 54, 744)

                # Footer
                self.line(54, 45, 612 - 54, 45)
                self.setFont("Helvetica", 8)
                self.setFillColorRGB(0.5, 0.5, 0.5)
                self.drawString(54, 32, "Confidential & Proprietary — Tagisan Open Source Ecosystem (v3.0)")
                self.drawRightString(612 - 54, 32, f"Page {self._pageNumber} of {page_count}")
                self.restoreState()

    doc = SimpleDocTemplate(
        PDF_PATH,
        pagesize=letter,
        leftMargin=54,
        rightMargin=54,
        topMargin=54,
        bottomMargin=54
    )

    styles = getSampleStyleSheet()

    # Custom typography
    c_primary = colors.HexColor("#0f2b48")
    c_secondary = colors.HexColor("#1e6091")
    c_accent = colors.HexColor("#0284c7")
    c_dark = colors.HexColor("#1e293b")
    c_light = colors.HexColor("#f8fafc")
    c_border = colors.HexColor("#cbd5e1")
    c_code_bg = colors.HexColor("#f1f5f9")

    title_style = ParagraphStyle(
        'CoverTitle',
        parent=styles['Normal'],
        fontName='Helvetica-Bold',
        fontSize=28,
        leading=34,
        textColor=c_primary,
        alignment=1, # Center
        spaceAfter=15
    )

    subtitle_style = ParagraphStyle(
        'CoverSubtitle',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=13,
        leading=18,
        textColor=c_secondary,
        alignment=1,
        spaceAfter=25
    )

    h1_style = ParagraphStyle(
        'Header1',
        parent=styles['Heading1'],
        fontName='Helvetica-Bold',
        fontSize=18,
        leading=22,
        textColor=c_primary,
        spaceBefore=18,
        spaceAfter=10,
        keepWithNext=True
    )

    h2_style = ParagraphStyle(
        'Header2',
        parent=styles['Heading2'],
        fontName='Helvetica-Bold',
        fontSize=13,
        leading=17,
        textColor=c_secondary,
        spaceBefore=12,
        spaceAfter=6,
        keepWithNext=True
    )

    h3_style = ParagraphStyle(
        'Header3',
        parent=styles['Heading3'],
        fontName='Helvetica-Bold',
        fontSize=10,
        leading=14,
        textColor=c_dark,
        spaceBefore=8,
        spaceAfter=4,
        keepWithNext=True
    )

    body_style = ParagraphStyle(
        'BodyDark',
        parent=styles['Normal'],
        fontName='Helvetica',
        fontSize=9,
        leading=13,
        textColor=c_dark,
        spaceAfter=6
    )

    code_style = ParagraphStyle(
        'CodeSnippet',
        parent=styles['Normal'],
        fontName='Courier',
        fontSize=7.5,
        leading=10,
        textColor=colors.HexColor("#0f172a")
    )

    bullet_style = ParagraphStyle(
        'BulletStyle',
        parent=body_style,
        leftIndent=15,
        firstLineIndent=-10,
        spaceAfter=3
    )

    story = []

    # -------------------------------------------------------------------------
    # Cover Page
    # -------------------------------------------------------------------------
    story.append(Spacer(1, 100))
    story.append(Paragraph("TAGISAN (`tgs`)", title_style))
    story.append(Paragraph("Sovereign Autonomous Agent Architecture", subtitle_style))
    story.append(Spacer(1, 20))

    banner_data = [
        [Paragraph("<font size='11'><b>The Definitive Master Class Course & Production Blueprint</b></font>", ParagraphStyle('BannerP', alignment=1, textColor=colors.white))],
        [Paragraph("<font size='9'>Dual-Brain Inference • BEAM/OTP Actors • Gleam • Google OAuth • AgentShield • 500 MCP Plugins • 150 Scenarios</font>", ParagraphStyle('BannerSub', alignment=1, textColor=colors.HexColor("#e0f2fe")))]
    ]
    banner_table = Table(banner_data, colWidths=[500])
    banner_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, -1), c_secondary),
        ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
        ('TOPPADDING', (0, 0), (-1, -1), 12),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 12),
        ('LEFTPADDING', (0, 0), (-1, -1), 15),
        ('RIGHTPADDING', (0, 0), (-1, -1), 15),
    ]))
    story.append(banner_table)

    story.append(Spacer(1, 40))

    meta_data = [
        [Paragraph("<b>Version:</b>", body_style), Paragraph("3.0 (Dual-Brain / BEAM-OTP / Gleam / MCP-500)", body_style)],
        [Paragraph("<b>Publication:</b>", body_style), Paragraph("September 2026 Sovereign Edition", body_style)],
        [Paragraph("<b>Engine Core:</b>", body_style), Paragraph("tagisan-rs (Rust 1.78+ / Tokio Async Reactor / BEAM / Gleam)", body_style)],
        [Paragraph("<b>Target Audience:</b>", body_style), Paragraph("Principal Architects, SREs, DevSecOps, AI Engineers, Quantitative Traders", body_style)],
        [Paragraph("<b>Repository:</b>", body_style), Paragraph("https://github.com/CharleGutierrez/tagisan", body_style)]
    ]
    meta_table = Table(meta_data, colWidths=[120, 380])
    meta_table.setStyle(TableStyle([
        ('BACKGROUND', (0, 0), (-1, -1), c_light),
        ('BOX', (0, 0), (-1, -1), 0.5, c_border),
        ('INNERGRID', (0, 0), (-1, -1), 0.5, c_border),
        ('TOPPADDING', (0, 0), (-1, -1), 6),
        ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
    ]))
    story.append(meta_table)

    story.append(PageBreak())

    # -------------------------------------------------------------------------
    # Module 1 to 9 Overviews
    # -------------------------------------------------------------------------
    def add_section(title, text_paragraphs):
        story.append(Paragraph(title, h1_style))
        story.append(HRFlowable(width="100%", thickness=1, color=c_secondary, spaceBefore=2, spaceAfter=8))
        for p in text_paragraphs:
            if p.startswith("```"):
                code_text = p.replace("```bash", "").replace("```rust", "").replace("```", "").strip()
                code_p = Paragraph(code_text.replace("\n", "<br/>").replace(" ", "&nbsp;"), code_style)
                code_tbl = Table([[code_p]], colWidths=[500])
                code_tbl.setStyle(TableStyle([
                    ('BACKGROUND', (0, 0), (-1, -1), c_code_bg),
                    ('BOX', (0, 0), (-1, -1), 0.5, c_border),
                    ('TOPPADDING', (0, 0), (-1, -1), 6),
                    ('BOTTOMPADDING', (0, 0), (-1, -1), 6),
                    ('LEFTPADDING', (0, 0), (-1, -1), 8),
                    ('RIGHTPADDING', (0, 0), (-1, -1), 8),
                ]))
                story.append(code_tbl)
                story.append(Spacer(1, 6))
            elif p.startswith("•") or p.startswith("-"):
                story.append(Paragraph(p, bullet_style))
            else:
                story.append(Paragraph(p, body_style))
        story.append(Spacer(1, 10))

    add_section("Module 1: Dual-Brain Hybrid Inference & Google Cloud Integration", [
        "Tagisan combines local tensor inference with cloud frontier models into a unified, zero-cost-first failover engine.",
        "• <b>Dual-Brain Architecture:</b> Local Ollama (Qwen2.5-Coder, SmolLM2) executes free daily tasks; Google Cloud CCPA gateway executes complex multi-step reasoning.",
        "• <b>Google Web OAuth (AntiGravity CCPA Gateway):</b> Connects directly to Google's internal CloudCode Platform Assistant endpoint (<code>daily-cloudcode-pa.googleapis.com</code>) using OAuth 2.0 with a proactive 90-second token auto-refresh.",
        "• <b>Model Fleet:</b> <code>gemini-2.5-flash</code> (default fast), <code>gemini-3.1-pro-low</code> (deep reasoning), <code>gemini-2.5-flash-lite</code> (sub-300ms latency), and <code>gemini-3.6-flash-medium</code>.",
        "• <b>Graceful Logout Fallback:</b> Running <code>tgs auth logout gemini</code> resets the provider to <code>auto</code>, automatically falling back to local Ollama without errors.",
        "```bash\ntgs ask -m pro \"Design a zero-downtime database migration\"\ntgs ask -m lite \"Summarize active SRE incident alerts\"\ntgs auth status\n```"
    ])

    add_section("Module 2: Polyglot Concurrency: Native Gleam & BEAM/OTP Actor Runtime", [
        "Tagisan integrates an Erlang/Elixir BEAM OTP actor runtime and a native Gleam compiler for crash-proof, distributed execution.",
        "• <b>Actor Process Isolation:</b> Every task executes in an independent BEAM process with an isolated heap, preventing memory leaks and cross-thread corruption.",
        "• <b>Supervisor Trees:</b> Implements <code>one_for_one</code>, <code>one_for_all</code>, and <code>rest_for_one</code> strategies with automatic restart telemetry.",
        "• <b>Binary ETF Codec:</b> Full support for Erlang External Term Format (ETF 131) serialization at over 800,000 messages/second.",
        "• <b>Type-Safe Gleam:</b> Statically typed Gleam microservices with algebraic data types and exhaustive pattern matching guarantee zero unhandled runtime panics."
    ])

    add_section("Module 3: Command Ergonomics & Dialectical Debate (MoA)", [
        "Tagisan provides a unified CLI suite and Hegelian Dialectical Debate to eliminate hallucinations.",
        "• <b>Hegelian Debate Consensus:</b> Spawns a Thesis Proposer, Antithesis Challenger, Synthesis Reconciler, and Judge to rigorously audit decisions.",
        "• <b>Interactive TUI Chat:</b> High-speed terminal chat (<code>tgs chat</code>) with streaming reasoning tokens and token cost calculation.",
        "```bash\ntgs debate --proposer \"Adopt Rust for core ledger\" --challenger \"Evaluate hiring velocity\" --rounds 3\n```"
    ])

    add_section("Module 4: AgentShield Cyber Defense & Landlock Sandboxing", [
        "Security is enforced at the kernel and Abstract Syntax Tree (AST) level prior to command execution.",
        "• <b>Pre-Execution AST Interception:</b> Scans tool calls and shell arguments for destructive tokens (<code>rm -rf /</code>, fork bombs, disk formatters).",
        "• <b>Linux Landlock LSM Sandboxing:</b> Unprivileged kernel jails restrict subprocess read/write access strictly to designated workspaces.",
        "• <b>Credential Guard:</b> Automatically intercepts and blocks unauthorized access to <code>~/.ssh/id_rsa</code>, <code>.env</code>, and cloud secrets."
    ])

    add_section("Module 5: PILOT Just-In-Time Multi-Tier Memory", [
        "Multi-tier cognitive memory architecture with sub-millisecond relevance routing:",
        "• <b>Tier 1 (Working Memory):</b> Tokio in-memory buffer holding active task context (< 1ms).",
        "• <b>Tier 2 (Episodic Memory):</b> SQLite / DuckDB session history with transactional rollback (< 5ms).",
        "• <b>Tier 3 (Semantic Memory):</b> High-dimensional vector database (Qdrant / SQLite-vec) with Reciprocal Rank Fusion (RRF) combining dense and BM25 sparse indices (< 15ms)."
    ])

    add_section("Module 6: The 500-Skill Sovereign Ecosystem (RFC-004)", [
        "All skills adhere to the RFC-004 package standard with YAML frontmatter specifications.",
        "• Dynamic ingestion, automatic lifecycle validation, and sandboxed polyglot execution (Rust, Python, Gleam).",
        "• Seamless chaining of domain skills in complex multi-stage autonomous missions."
    ])

    add_section("Module 7: Model Context Protocol (MCP 500) Enterprise Hub", [
        "Tagisan serves as both an MCP client and an MCP server (<code>tgs serve-mcp</code>):",
        "• <b>500 Verified Plugins:</b> Sourced from non-GitHub enterprise registries across 10 strategic domains (Databases, Cloud, DevOps, Security, APM, Productivity, SaaS/FinTech, AI, Vector DBs, Search).",
        "• <b>Transports:</b> Full support for Stdio and SSE transports with JSON-RPC 2.0 communication."
    ])

    add_section("Module 8: Domain Copilots & Cyber-Physical Digital Twins (VELLA)", [
        "VELLA brings real-time deterministic cyber-physical modeling to LLM agents:",
        "• <b>SCADA / IoT:</b> Modbus TCP and OPC-UA sensor telemetry sync, automated alarm actions, and valve control.",
        "• <b>Quantitative Finance:</b> High-frequency forex tick spreads, Value-at-Risk (VaR) Monte Carlo simulations, and margin de-leveraging.",
        "• <b>Aerospace:</b> SGP4 Two-Line Element (TLE) satellite orbit propagation and ground station visibility pass forecasting.",
        "• <b>Bioinformatics:</b> FASTA/FASTQ sequence alignment (Smith-Waterman) and VCF mutation filtering."
    ])

    add_section("Module 9: Enterprise Hardening & Air-Gapped Operations", [
        "Designed for sovereign government, defense, and enterprise deployments:",
        "• <b>Air-Gapped Isolation:</b> Setting <code>TAGISAN_LOCAL_ONLY=1</code> guarantees 100% offline execution using local Ollama tensors.",
        "• <b>OpenTelemetry Observability:</b> Structured JSON logging of every token, turn latency (ms), AgentShield verdict, and exact USD cost."
    ])

    story.append(PageBreak())

    # -------------------------------------------------------------------------
    # Module 10: 150 Scenarios
    # -------------------------------------------------------------------------
    story.append(Paragraph("Module 10: The Definitive 150 Production Scenarios", h1_style))
    story.append(HRFlowable(width="100%", thickness=1, color=c_secondary, spaceBefore=2, spaceAfter=8))
    story.append(Paragraph("This catalog details 150 concrete, production-proven scenarios across 10 major enterprise engineering domains. Every scenario demonstrates how Tagisan's dual-brain, BEAM actor runtime, AgentShield security, and MCP ecosystem solve mission-critical challenges in modern environments.", body_style))
    story.append(Spacer(1, 10))

    for domain in ALL_DOMAINS:
        story.append(Paragraph(f"Domain {domain['range'][0]}–{domain['range'][1]}: {domain['icon']} {domain['name']}", h2_style))
        story.append(HRFlowable(width="100%", thickness=0.5, color=c_border, spaceBefore=1, spaceAfter=6))

        for sc in domain["scenarios"]:
            # Scenario Table Card
            sc_header = Paragraph(f"<b>Scenario {sc['id']}: {sc['title']}</b>", ParagraphStyle('ScH', parent=styles['Normal'], fontName='Helvetica-Bold', fontSize=8.5, textColor=c_primary))
            sc_cap = Paragraph(f"<b>Capabilities:</b> {sc['capability']}", ParagraphStyle('ScC', parent=styles['Normal'], fontName='Helvetica', fontSize=7.5, textColor=c_secondary))
            sc_cmd = Paragraph(f"<code>{sc['command']}</code>", ParagraphStyle('ScCmd', parent=styles['Normal'], fontName='Courier', fontSize=7, textColor=colors.HexColor("#0f172a")))
            sc_flow = Paragraph(sc['flow'].replace("\n", "<br/>"), ParagraphStyle('ScF', parent=styles['Normal'], fontName='Helvetica', fontSize=7.5, leading=10, textColor=c_dark))
            sc_out = Paragraph(f"<b>Outcome:</b> {sc['outcome']}", ParagraphStyle('ScO', parent=styles['Normal'], fontName='Helvetica-Bold', fontSize=7.5, textColor=colors.HexColor("#047857")))

            card_data = [
                [sc_header],
                [sc_cap],
                [sc_cmd],
                [sc_flow],
                [sc_out]
            ]
            card_table = Table(card_data, colWidths=[500])
            card_table.setStyle(TableStyle([
                ('BACKGROUND', (0, 0), (-1, -1), colors.HexColor("#f8fafc")),
                ('BOX', (0, 0), (-1, -1), 0.5, c_border),
                ('LINEBELOW', (0, 0), (-1, 0), 0.5, c_border),
                ('TOPPADDING', (0, 0), (-1, -1), 3),
                ('BOTTOMPADDING', (0, 0), (-1, -1), 3),
                ('LEFTPADDING', (0, 0), (-1, -1), 6),
                ('RIGHTPADDING', (0, 0), (-1, -1), 6),
                ('BACKGROUND', (0, 2), (-1, 2), c_code_bg),
            ]))

            story.append(card_table)
            story.append(Spacer(1, 4))

        story.append(Spacer(1, 8))

    story.append(PageBreak())

    # -------------------------------------------------------------------------
    # Module 11: Capstone Labs & Certification Checklist
    # -------------------------------------------------------------------------
    add_section("Module 11: Capstone Certification Labs & Practical Exams", [
        "To achieve Tagisan Certified Architect (TCA) status, complete these 3 hands-on capstone labs:",
        "• <b>Lab 1: Autonomous Self-Healing Code Pipeline</b><br/>Configure Tagisan to monitor a git repository, detect compilation failures, execute a 3-round Hegelian debate between a Refactoring Agent and a Security Auditor, apply the fixes, and verify that <code>cargo test</code> passes.",
        "• <b>Lab 2: Enterprise Market Intelligence Swarm</b><br/>Deploy a multi-agent swarm orchestrating Brave Search MCP, PostgreSQL MCP, and Qdrant Vector Memory to monitor financial regulatory filings, compute sentiment shifts, and index insights with Reciprocal Rank Fusion.",
        "• <b>Lab 3: Air-Gapped Cyber Defense Cluster</b><br/>Deploy Tagisan with zero external network connectivity (<code>TAGISAN_LOCAL_ONLY=1</code>), local Ollama Qwen2.5-Coder, and local SQLite memory. Verify that AgentShield successfully intercepts all 8 simulated exfiltration attacks in the brutal test suite.",
        "<b>Master Certification Checklist:</b>",
        " [✓] Build and verify native <code>tgs</code> release binary.<br/>"
        " [✓] Connect Google Web OAuth (<code>tgs auth login gemini</code>) and test <code>-m pro</code> and <code>-m lite</code>.<br/>"
        " [✓] Verify graceful offline fallback to local Ollama on logout.<br/>"
        " [✓] Execute a multi-round Hegelian Dialectical Debate.<br/>"
        " [✓] Install and execute an external MCP tool from the 500 catalog.<br/>"
        " [✓] Implement a custom RFC-004 skill under <code>.ecc/skills/</code>.<br/>"
        " [✓] Successfully execute at least 5 scenarios from Module 10.<br/>"
        " [✓] Pass all 8 brutal security and transport tests in the test suite."
    ])

    print("Building PDF document with NumberedCanvas...")
    doc.build(story, canvasmaker=NumberedCanvas)
    print(f"✅ Generated PDF: {PDF_PATH} ({os.path.getsize(PDF_PATH)} bytes)")

if __name__ == "__main__":
    print("Starting Tagisan Master Class Course generation...")
    generate_markdown()
    generate_pdf()
    print("All materials generated successfully!")
