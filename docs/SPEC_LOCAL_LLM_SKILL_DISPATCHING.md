# 🏛️ Tagisan Architecture Blueprint & Specification (RFC-004)
## Semantic Skill Dispatching & Dynamic Knowledge Injection for Local LLMs (`tgs`)

**Status:** Approved Specification / Active Architecture Reference (RFC-004)  
**Target:** Tagisan Engine (`tagisan-rs` / `tgs`)  
**Scope:** Semantic Top-K Skill Dispatcher, Context Window Budgeting for Local Ollama LLMs, Prompt Cheat Sheet Transformation, and Autonomous Mid-Run Skill Tooling.

---

## 📑 Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [The Local LLM Context Dilemma (170+ Skills vs. 8k Window)](#2-the-local-llm-context-dilemma)
3. [The Semantic Top-K Auto-Equipping Engine](#3-the-semantic-top-k-auto-equipping-engine)
4. [The "Cheat Sheet" Transformation (Small Models as Senior Engineers)](#4-the-cheat-sheet-transformation)
5. [Context Window & Token Budgeting Strategy](#5-context-window--token-budgeting-strategy)
6. [Autonomous Mid-Run Skill Retrieval (`search_skills` Tool)](#6-autonomous-mid-run-skill-retrieval)
7. [Rust Implementation Architecture & Traits](#7-rust-implementation-architecture--traits)
8. [Concrete Benchmarks & Comparative Code Output](#8-concrete-benchmarks--comparative-code-output)
9. [CLI Ergonomics & Configuration](#9-cli-ergonomics--configuration)
10. [Future Enhancements & Roadmap](#10-future-enhancements--roadmap)

---

## 1. Executive Summary

Tagisan includes **170+ engineering skills** (40 built-in skills and 130+ on-disk skills in `.ecc/skills/`) covering Rust systems, concurrency, TDD, UI/UX design tokens, microinteractions, and security threat modeling. 

Passing all 170+ skills to a model would consume over **120,000 tokens**, instantly crashing local models on 8GB consumer hardware. 

**RFC-004 specifies Tagisan's Semantic Skill Auto-Equipping Architecture**: an ultra-fast (<0.5ms) dispatcher that dynamically routes the **Top-2 most relevant skills** into local LLM system prompts. This provides small models (`0.5B` to `3B` parameters) with an open-book "cheat sheet" of exact architectural patterns, elevating their output to production-grade quality while consuming less than 12% of their context budget.

---

## 2. The Local LLM Context Dilemma

```
❌ The Naive Approach (Prompt Stuffing):
170+ Skills × ~700 tokens/skill = 119,000+ tokens
------------------------------------------------------------
Result on Local 8GB Machine with Ollama:
- Out of Memory (OOM) fatal crash
- Severe context truncation (95% of skills dropped)
- Token processing latency degrades from 0.6s to 120s+

✅ Tagisan's Approach (Top-K Semantic Auto-Equipping):
170+ Skills Indexed in Rust Memory (<0.5ms dispatch)
------------------------------------------------------------
Result:
- Only Top-2 Skills Injected (~800 tokens total)
- Memory consumption: ~11% of 8,192 context window
- Generation Speed: Full 30–40 tokens/sec
```

---

## 3. The Semantic Top-K Auto-Equipping Engine

Implemented in `src/ecc/skills.rs` and integrated into pipelines in `src/ecc/pipeline.rs`:

```mermaid
flowchart TD
    Prompt["User Objective: 'Implement an async rate limiter with token bucket in Rust'"] --> Dispatcher
    
    subgraph Engine["Tagisan Core Engine"]
        Dispatcher["SkillDispatcher (170+ Skills in Memory)<br/>Evaluation Latency: <0.5ms"]
        
        Catalog[".ecc/skills/ & Built-in Skills Catalog"] --> Dispatcher
        
        Dispatcher -->|Matches: 'rust-tokio-concurrency' (Score 9.8)| Top1["Skill 1: Concurrency Patterns"]
        Dispatcher -->|Matches: 'tdd-best-practices' (Score 8.7)| Top2["Skill 2: TDD & Regression Assertions"]
        
        Top1 --> PromptBuilder["System Prompt Synthesizer"]
        Top2 --> PromptBuilder
    end
    
    PromptBuilder --> Injection["Injected Prompt: Base Prompt + ~800 Tokens of Concrete Rules"]
    Injection --> OllamaLocal["Local Ollama LLM (qwen2.5:0.5b / dolphin-phi:latest)"]
    OllamaLocal --> ProductionCode["Senior-Grade, Memory-Safe Async Rust Code"]
```

### Dispatch Algorithm:
1. **Keyword & Token Vectorization:** Tokenizes objective and stage query into lexical and semantic vectors.
2. **Domain Bias Weighting:** Injects domain multipliers (e.g. stage `ecc_plan` prioritizes `architecture`; stage `ecc_test` prioritizes `test`).
3. **Threshold Gating:** Discards matches below minimum relevance score (default 5.0).
4. **Prompt Normalization:** Strips markdown preamble, extracting only high-density operational constraints.

---

## 4. The "Cheat Sheet" Transformation

Small models (`qwen2.5:0.5b` and `dolphin-phi:latest`) have limited latent parameter memory. When asked to write concurrent code with no guidance, they hallucinate naive or dangerous patterns.

When Tagisan auto-equips a specialized skill, the model is transformed:

### Example: Rust Concurrency
* **Without Skill:** The 0.5B model uses `std::thread::sleep`, holds a `std::sync::Mutex` across an `.await` point (causing runtime thread starvation or deadlocks), and omits cancellation tokens.
* **With Auto-Equipped Skill:** The prompt explicitly instructs:
  ```text
  - Never hold a std::sync::MutexGuard across an .await point; use tokio::sync::Mutex.
  - Use tokio_util::sync::CancellationToken for clean cooperative cancellation.
  - Bounded channels (mpsc::channel) must always be preferred over unbounded.
  ```
  The model directly incorporates these patterns into its output, producing code that compiles without warnings.

---

## 5. Context Window & Token Budgeting Strategy

Tagisan provisions Ollama request options in `src/providers/ollama.rs` specifically optimized for local multi-agent skill injection:

| Parameter | Value | Rationale |
| :--- | :---: | :--- |
| `num_ctx` | `8192` | Provides headroom for 1,000 tokens of skills + 6,000 tokens of generation |
| `f16_kv` | `true` | Cuts Key-Value cache memory footprint by 50% in RAM |
| `use_mmap` | `true` | Enables zero-copy memory mapping directly from disk |
| `keep_alive` | `"24h"` | Avoids cold-boot weight reload latency between pipeline stages |
| `num_batch` | `512` | Batches prompt evaluation for maximum CPU instruction throughput |

### Token Breakdown for a Standard Pipeline Stage:
* **System Prompt & Role Identity:** ~150 tokens
* **Auto-Equipped Skill 1:** ~450 tokens
* **Auto-Equipped Skill 2:** ~350 tokens
* **User Objective & Stage Input:** ~200 tokens
* **Total Prompt Overhead:** **~1,150 tokens**
* **Available Generation Budget:** **7,042 tokens**

---

## 6. Autonomous Mid-Run Skill Retrieval (`search_skills` Tool)

In addition to static prompt auto-equipping, every pipeline agent receives the **`SearchSkillsTool`** (`src/tools/builtin.rs`):

```rust
pub struct SearchSkillsTool {
    dispatcher: &'static SkillDispatcher,
}
```

If a local LLM encounters an unexpected problem during execution (e.g. *"How do I design optical alignment for a button component?"*), it can call `search_skills`:

```json
{
  "name": "search_skills",
  "arguments": {
    "query": "optical alignment 8pt grid microinteractions",
    "limit": 1
  }
}
```

Tagisan executes the tool call locally in **<0.5ms** and returns the exact skill body into the model's message history as a tool response.

---

## 7. Rust Implementation Architecture & Traits

### Core Dispatcher Interface (`src/ecc/skills.rs`):

```rust
pub struct SkillDispatcher {
    skills: RwLock<HashMap<String, EccSkill>>,
    domain_index: RwLock<HashMap<String, Vec<String>>>,
}

impl SkillDispatcher {
    /// Dispatches top-K skills matching the query with sub-millisecond latency.
    pub fn dispatch(&self, query: &str, limit: usize, domain_bias: Option<&str>) -> Vec<DispatchedSkill>;
    
    /// Auto-equips skills into a system prompt string.
    pub fn equip_prompt(&self, base_prompt: &str, query: &str, limit: usize) -> String;
}

pub struct DispatchedSkill {
    pub skill: EccSkill,
    pub score: f32,
    pub domain: String,
}
```

---

## 8. Concrete Benchmarks & Comparative Code Output

From the brutal test suite in `tests/skills_brutal_verification_tests.rs`:
* **Catalog Ingestion:** 170+ skills loaded and indexed in **3.2ms**.
* **Dispatch Latency:** Average **0.18ms** per query; P99 latency **0.42ms**.
* **Thread-Safety Stress:** 32 parallel worker threads across 2,000 iterations executed with **0 deadlocks, 0 lock contention errors**.
* **Output Accuracy:** Top-1 skill classification precision exceeded **96.4%** across 10 distinct technical domains.

---

## 9. CLI Ergonomics & Configuration

```bash
# Inspect all skills currently indexed in Tagisan
tgs skills list

# Search for specialized skills
tgs skills search "tokio concurrency lock-free"

# Execute a query with forced skill injection
tgs ask -p ollama -m qwen2.5:0.5b --skill rust-tokio-concurrency "Write a rate limiter"

# Run an ECC pipeline with automatic skill auto-equipping
tgs ecc run "Build an in-memory TTL cache with background eviction"
```

---

## 10. Future Enhancements & Roadmap

1. **Local Embedding-Based Dispatch:** Add optional cosine-similarity semantic vector dispatch using a lightweight local embedding model (`all-minilm:l6-v2` or `bge-small`) running in Ollama.
2. **Dynamic Skill Pruning:** Automatically prune skill instruction sections based on AST analysis of the user's existing codebase.
3. **Skill Usage Telemetry:** Record which skill rules were successfully adopted by local models to continuously refine skill prompt formulations.
