# 🔌 Tagisan Plugin Architecture & GitHub Ecosystem Integration (RFC-002)

**Status:** Proposed / Blueprint for Implementation  
**Target:** Tagisan (`tgs` / `tagisan-rs`)  
**Scope:** Universal Plugin Architecture, Multi-Runtime Execution Engines (WASM/Extism, Bun/TypeScript, Model Context Protocol, Native cdylib), Capability-Based Sandboxing, Lifecycle Extension Hooks, CLI Ergonomics, and Curated GitHub Plugin Ecosystem.

---

## 📑 Table of Contents
1. [Executive Vision](#1-executive-vision)
2. [Architectural Topology](#2-architectural-topology)
3. [The 4 Plugin Tiers (Execution Engines)](#3-the-4-plugin-tiers-execution-engines)
4. [Plugin Manifest Specification (`tgs-plugin.toml`)](#4-plugin-manifest-specification-tgs-plugintoml)
5. [Cognitive Lifecycle Extension Hooks](#5-cognitive-lifecycle-extension-hooks)
6. [Security & Capability-Based Sandboxing](#6-security--capability-based-sandboxing)
7. [Curated GitHub Plugin Catalog for `tgs`](#7-curated-github-plugin-catalog-for-tgs)
8. [CLI Ergonomics & Workflow](#8-cli-ergonomics--workflow)
9. [Concrete Implementation SDK (Rust & TypeScript)](#9-concrete-implementation-sdk-rust--typescript)
10. [Phased Implementation Roadmap](#10-phased-implementation-roadmap)

---

## 1. Executive Vision

While existing agent systems rely on fragile, un-sandboxed Python scripts that risk system compromise and memory bloat, **Tagisan (`tgs`)** implements a **high-performance, polyglot, capability-sandboxed Plugin Operating System**.

By pairing Tagisan's zero-cost Rust core with its native Bun runtime, automated hybrid `SkillDispatcher`, and `AgentShield` security interceptor, `tgs` plugins achieve:
- **Zero Core Bloat:** The core binary stays fast, compact, and deterministic. Integrations are loaded dynamically on-demand.
- **Polyglot Developer Freedom:** Developers can write plugins in **Rust, TypeScript/JavaScript, Go, C++, Zig, or Python**.
- **Hardware-Enforced Sandboxing:** Capability-based limits (network, filesystem, environment, subprocesses) prevent unintended data destruction or exfiltration.
- **Dynamic Hot-Swapping:** Agents can query, install, and invoke plugins mid-conversation as their task requirements evolve.

---

## 2. Architectural Topology

```
                                  ┌───────────────────────────┐
                                  │    Tagisan Core Engine    │
                                  │   (Rust Host & Router)    │
                                  └─────────────┬─────────────┘
                                                │
         ┌──────────────────┬───────────────────┼───────────────────┬──────────────────┐
         ▼                  ▼                   ▼                   ▼                  ▼
┌─────────────────┐ ┌───────────────┐   ┌───────────────┐   ┌───────────────┐  ┌───────────────┐
│   Tier A: WASM  │ │ Tier B: Bun/TS│   │  Tier C: MCP  │ │ Tier D: Native│  │  Skill Packs  │
│  (WASI/Extism)  │ │ (Built-in V8) │   │  (Stdio/SSE)  │ │ (cdylib/.so)  │  │ (Markdown/ECC)│
├─────────────────┤ ├───────────────┤   ├───────────────┤   ├───────────────┤  ├───────────────┤
│ • Zero-trust    │ │ • Full NPM    │   │ • Anthropic   │ │ • Sub-µs perf │  │ • Prompts     │
│ • Sandboxed mem │ │ • Hot-reload  │   │   standard    │ │ • Direct GPU  │  │ • Triggers    │
│ • Polyglot      │ │ • Async I/O   │   │ • 500+ tools  │ │ • Bare-metal  │  │ • Auto-ranked │
└─────────────────┘ └───────────────┘   └───────────────┘   └───────────────┘  └───────────────┘
         │                  │                   │                   │                  │
         └──────────────────┴─────────┬─────────┴───────────────────┴──────────────────┘
                                      ▼
                        ┌───────────────────────────┐
                        │   AgentShield Guardrail   │
                        │  (Capability Enforcement) │
                        └─────────────┬─────────────┘
                                      ▼
                        ┌───────────────────────────┐
                        │    Tool Registry & Swarm  │
                        │  (ECC 5-Stage & MoA Loop) │
                        └───────────────────────────┘
```

---

## 3. The 4 Plugin Tiers (Execution Engines)

### Tier A: WebAssembly (WASI / Extism) — *Zero-Trust & Polyglot*
* **Runtime:** Extism Rust SDK / Wasmtime.
* **Characteristics:** Compiled `.wasm` binaries. Polyglot source languages (Rust, Go, C, Zig, Swift).
* **Isolation:** Strict linear memory sandboxing. No system access (filesystem, network, environment) without explicit capability grants from `tgs`.
* **Latency:** Startup latency $< 1\text{ ms}$; near-native execution throughput.

### Tier B: Bun / TypeScript Plugins — *Ecosystem Speed & Rapid DX*
* **Runtime:** Tagisan's native Bun runtime (`src/bun/runtime.rs`) and worker pool.
* **Characteristics:** Native `.ts` and `.js` execution with full npm ecosystem compatibility.
* **Strengths:** Rapid prototyping, instant hot-reloading during development, zero-compilation iteration.
* **Isolation:** Executed within isolated Bun worker threads with restricted global scopes.

### Tier C: Model Context Protocol (MCP) Connectors — *Open Standards*
* **Runtime:** Tagisan's native JSON-RPC 2.0 stdio / SSE transport.
* **Characteristics:** Instant compatibility with the universal Anthropic / open-source MCP specification.
* **Strengths:** Instantly connects `tgs` to hundreds of existing community tools (PostgreSQL, GitHub, Docker, Slack, Puppeteer).
* **Integration:** Auto-namespaced as `<plugin_name>__<tool_name>` in Tagisan's `ToolRegistry`.

### Tier D: Dynamic Native Shared Libraries (`cdylib` / `.so` / `.dylib`) — *Bare Metal*
* **Runtime:** `libloading` with C-ABI / Rust FFI.
* **Characteristics:** Sub-microsecond execution for high-performance computing, direct GPU tensor operations, local llama.cpp/candle bindings, or industrial hardware/SCADA protocols.
* **Isolation:** High-privilege tier; requires explicit `--allow-native-plugins` security flag.

---

## 4. Plugin Manifest Specification (`tgs-plugin.toml`)

Every plugin contains a root declarative manifest:

```toml
[plugin]
name = "postgres-auditor"
version = "1.0.0"
author = "Charle Gutierrez <charle@example.com>"
description = "Inspect schemas, explain query execution plans, and audit indexes on PostgreSQL"
homepage = "https://github.com/CharleGutierrez/tgs-plugin-postgres"
license = "MIT OR Apache-2.0"

# Target Runtime: "wasm" | "bun" | "mcp" | "native"
runtime = "bun"
entrypoint = "dist/index.js"

[tools]
enabled = ["inspect_schema", "explain_query", "find_missing_indexes"]

[skills]
# Automatic registration into tgs SkillDispatcher
catalog_dir = "skills/"

[permissions]
# Strict capability-based permissions enforced by AgentShield
network = ["db.prod.internal:5432", "10.0.0.*:5432"]
fs_read = ["./sql/**/*.sql", "./migrations/**/*.sql"]
fs_write = []                  # Read-only tool; writes strictly forbidden
env = ["DATABASE_URL"]         # Masked & injected securely
subprocesses = false           # Cannot spawn arbitrary shell commands
```

---

## 5. Cognitive Lifecycle Extension Hooks

Plugins can intercept and enrich every phase of Tagisan's cognitive loop:

| Extension Hook | Trait / Interface | Function in Tagisan |
| :--- | :--- | :--- |
| **`ToolProvider`** | `pub trait PluginToolProvider` | Registers agent-callable tools into `ToolRegistry`. Agents invoke them during autonomous reasoning. |
| **`SkillPack`** | `pub trait PluginSkillPack` | Dynamically mounts specialized engineering prompts and triggers into `SkillDispatcher`. |
| **`PipelineStage`** | `pub trait PluginPipelineStage` | Injects custom execution stages into the 5-Stage ECC Pipeline (e.g. `ecc_compliance`, `ecc_fuzz`). |
| **`DebateJudge`** | `pub trait PluginDebateJudge` | Provides alternative consensus algorithms (Borda count, Condorcet, Elo rating) for multi-agent arbitration. |
| **`ShieldInterceptor`** | `pub trait PluginShieldInterceptor` | Extends `AgentShield` with domain-specific guardrails (HIPAA compliance, PII scrub, smart-contract invariants). |
| **`TelemetryExporter`** | `pub trait PluginTelemetryExporter` | Streams execution traces, latency, and token costs to external monitoring platforms. |

---

## 6. Security & Capability-Based Sandboxing

To protect developer systems from malicious or buggy plugins:

1. **Principle of Least Privilege:** Plugins are granted zero ambient authority. All file, network, and environment accesses must be declared in `tgs-plugin.toml`.
2. **AgentShield Enforcement:** Tool invocations pass through `AgentShield` before reaching the plugin runtime.
3. **OS-Level Isolation:**
   - **Linux:** Linux Landlock LSM sandboxing + seccomp filters for native/subprocess runtimes.
   - **WASM:** Complete linear memory isolation via WASI.
4. **Interactive Escalation:** If a plugin attempts an undeclared operation (e.g., connecting to an unknown IP), `tgs` prompts the user with an interactive permission confirmation modal.

---

## 7. Curated GitHub Plugin Catalog for `tgs`

The following open-source projects on GitHub are identified as primary plugin candidates for Tagisan:

### 1. Model Context Protocol (MCP) Standard Ecosystem
- [**`modelcontextprotocol/servers`**](https://github.com/modelcontextprotocol/servers) (Anthropic / Community)
  - *Components:* PostgreSQL, SQLite, GitHub, GitLab, Git, Slack, Puppeteer, Memory servers.
  - *Utility in `tgs`:* Gives agents direct access to databases, issue trackers, and git history without writing custom adapters.
- [**`docker/mcp-server-docker`**](https://github.com/docker/mcp-server-docker) (Docker Inc.)
  - *Utility in `tgs`:* Spins up isolated containers during the `ecc_verify` stage to test compiled Rust/Bun binaries in pristine environments.
- [**`cloudflare/mcp-server-cloudflare`**](https://github.com/cloudflare/mcp-server-cloudflare) (Cloudflare)
  - *Utility in `tgs`:* Deploys and debugs Cloudflare Workers, KV namespaces, and D1 serverless SQL.
- [**`getsentry/mcp-server-sentry`**](https://github.com/getsentry/mcp-server-sentry) (Sentry)
  - *Utility in `tgs`:* Automatically fetches production crash stack traces for `tgs` agents to diagnose and generate regression fixes.
- [**`qdrant/mcp-server-qdrant`**](https://github.com/qdrant/mcp-server-qdrant) (Qdrant)
  - *Utility in `tgs`:* High-performance vector memory backend for cross-session agent recall.

### 2. Code Intelligence & AST Manipulation (Rust Native)
- [**`ast-grep/ast-grep`**](https://github.com/ast-grep/ast-grep) (Herrington Darkholme)
  - *Utility in `tgs`:* Replaces regex search with syntax-tree-aware pattern replacement across 20+ languages.
- [**`tree-sitter/tree-sitter`**](https://github.com/tree-sitter/tree-sitter) (Tree-sitter Team)
  - *Utility in `tgs`:* Real-time incremental parsing for symbol definition lookups and reference graphs with microsecond latency.
- [**`oxc-project/oxc`**](https://github.com/oxc-project/oxc) (Boshen)
  - *Utility in `tgs`:* High-throughput JS/TS parser, linter, and minifier in Rust, complementing the Bun runtime.

### 3. WebAssembly (WASM) Sandbox Engine
- [**`extism/extism`**](https://github.com/extism/extism) & [**`extism/plugins`**](https://github.com/extism/plugins) (Dylibso)
  - *Utility in `tgs`:* The universal WebAssembly plugin system for executing polyglot community plugins safely.

### 4. Headless Browser & UI Automation
- [**`microsoft/playwright`**](https://github.com/microsoft/playwright) / [Puppeteer MCP](https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer)
  - *Utility in `tgs`:* Allows agents to open browsers, click elements, capture visual diffs, and verify web interfaces during `ecc_verify`.
- [**`browserbase/mcp-server-browserbase`**](https://github.com/browserbase/mcp-server-browserbase) (Browserbase)
  - *Utility in `tgs`:* Cloud-hosted headless browser fleet for web scraping without IP bans.

### 5. Security Auditing, SAST & Vulnerability Scanning
- [**`gitleaks/gitleaks`**](https://github.com/gitleaks/gitleaks) (Zachary Rice)
  - *Utility in `tgs`:* Secret and token scanning before git commits.
- [**`rustsec/rustsec`**](https://github.com/rustsec/rustsec) (`cargo-audit`)
  - *Utility in `tgs`:* Dependency vulnerability auditing in `Cargo.lock` during the ECC pipeline.
- [**`semgrep/semgrep`**](https://github.com/semgrep/semgrep) (Semgrep Inc.)
  - *Utility in `tgs`:* Semantic security policy enforcement during `ecc_security`.

---

## 8. CLI Ergonomics & Workflow

```bash
# 1. Search available plugins (registry + GitHub)
tgs plugin search postgres

# 2. Install a plugin directly from GitHub or local path
tgs plugin install github.com/modelcontextprotocol/servers/src/postgres
tgs plugin install ./my-wasm-plugin.wasm

# 3. List installed plugins and inspect capability permissions
tgs plugin list --permissions

# 4. Initialize a new plugin template
tgs plugin new my-custom-tool --template bun-ts
tgs plugin new my-rust-tool --template extism-wasm

# 5. Run an autonomous agent with specific plugins active
tgs agent --plugin postgres --plugin docker "Inspect slow queries and test optimization in container"
```

---

## 9. Concrete Implementation SDK (Rust & TypeScript)

### TypeScript Plugin Example (`plugins/git-blame/index.ts`):
```typescript
import { defineTgsPlugin, ToolResult } from "@tagisan/plugin-sdk";

export default defineTgsPlugin({
  name: "git-blame-analyzer",
  version: "1.0.0",
  tools: [
    {
      name: "analyze_code_ownership",
      description: "Analyze author contribution metrics for a source file",
      parameters: {
        type: "object",
        properties: {
          path: { type: "string", description: "Target source file path" },
        },
        required: ["path"],
      },
      async execute({ path }, context): Promise<ToolResult> {
        const blame = await context.git.blame(path);
        return ToolResult.success(blame);
      }
    }
  ]
});
```

### Rust WASM Plugin Example (`plugins/hasher/src/lib.rs`):
```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct HashInput {
    data: String,
    algorithm: String,
}

#[derive(Serialize)]
struct HashOutput {
    hash: String,
}

#[plugin_fn]
pub fn compute_hash(Json(input): Json<HashInput>) -> FnResult<Json<HashOutput>> {
    let hash = match input.algorithm.as_str() {
        "blake3" => blake3::hash(input.data.as_bytes()).to_hex().to_string(),
        "sha256" => sha256::digest(input.data.as_bytes()),
        _ => return Err(WithReturnCode::from("Unsupported algorithm")),
    };
    Ok(Json(HashOutput { hash }))
}
```

---

## 10. Phased Implementation Roadmap

| Phase | Milestone | Deliverables |
| :--- | :--- | :--- |
| **Phase 1** | **Plugin Core Specification** | Define `tgs-plugin.toml` manifest, `PluginMetadata`, and `PluginLoader` trait in `src/plugins/mod.rs`. |
| **Phase 2** | **MCP Bridge Subsystem** | Direct dynamic bridge mapping external MCP stdio/SSE servers into `ToolRegistry`. |
| **Phase 3** | **Bun / TypeScript Runtime Loader** | Execute `.ts` plugins directly through the embedded Bun runtime and worker pool. |
| **Phase 4** | **WebAssembly (Extism) Runner** | Integrate `extism` crate for sandboxed WASM plugin execution with memory quotas. |
| **Phase 5** | **AgentShield Security Gates** | Implement capability checker verifying network, filesystem, and subprocess limits per call. |
| **Phase 6** | **CLI Tooling & Ecosystem Registry** | Add `tgs plugin` subcommands (`install`, `list`, `new`, `test`) and GitHub release fetching. |

---

*Saved to repository blueprints (`ROADMAP_PLUGINS.md`) for planned future implementation.*
