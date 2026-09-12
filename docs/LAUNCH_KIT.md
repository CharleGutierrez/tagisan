# 🚀 Tagisan (`tgs`) GitHub Growth & Launch Kit

This kit contains copy-paste ready promotional copy for launching Tagisan across developer communities to maximize GitHub stars and community adoption.

---

## 1. Hacker News: Show HN

* **Where to Post:** [news.ycombinator.com/submit](https://news.ycombinator.com/submit)
* **Best Timing:** Tuesday or Thursday morning at 08:00 AM – 09:00 AM EST.
* **Title:** `Show HN: Tagisan – High-Performance Multi-LLM Swarm & Mixture-of-Agents in Rust`
* **URL:** `https://github.com/CharleGutierrez/tagisan`
* **Text (if text submission, or first comment):**
```markdown
Hey HN,

I built Tagisan (`tgs`) [https://github.com/CharleGutierrez/tagisan], a zero-dependency, single-binary multi-agent engine written in Rust.

### Why Tagisan?
Existing multi-agent frameworks (CrewAI, AutoGen, MetaGPT, LangGraph) are predominantly written in Python. While they are great for prototyping, in production they suffer from:
1. Slow startup latency (~1.5s+ per invocation).
2. High memory footprint (400MB–1GB+ per worker process).
3. Fragile dependency graphs (pip/conda package breaks).
4. Unchecked conversational loops.

We wanted a native systems tool that treats multi-agent orchestration like Unix pipelines: fast, composable, and resource-efficient.

### Key Capabilities:
- **Single 29 MB Native Binary:** Boots in ~2ms, runs in ~18MB RAM.
- **Dialectical Debate (`tgs debate`):** Proponent (Thesis) vs Adversary (Antithesis) moderated by an independent Lakandiwa synthesis adjudicator.
- **Native Mixture-of-Agents (`tgs moa`):** Concurrently queries heterogeneous models across async Tokio channels and synthesizes them with a Master Aggregator.
- **100% Offline Local Ollama Support:** Dynamic on-disk model discovery, hardware-aware GPU pinning, and sub-400ms rotational model hot-swapping.
- **70 Built-in Engineering Skills:** Sub-millisecond hybrid lexical-semantic dispatcher indexing quantitative foundation models (Kronos, Qlib, Chronos), applied mathematics (The Nature of Code, Lengyel 3D game geometry, Pearl causality), UX, and security audits.
- **Polyglot Agent Sandbox:** Embedded Bun, Python (uv), and Perl with real-time AgentShield defense against shell escapes and malicious writes.

### Quickstart (No API keys needed with Ollama):
```bash
# Install
curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh

# Run a local debate between installed models
tgs debate "PostgreSQL vs MongoDB for high-throughput time series" --local

# Search 70+ built-in quantitative and math skills
tgs ecc skills -q "boids flocking simulation in WebGL"
```

The codebase is open source (MIT/Apache 2.0). I’d love feedback on the architecture and multi-model debate ergonomics!
```

---

## 2. Reddit: `r/LocalLLaMA`

* **Subreddit:** `r/LocalLLaMA`
* **Title:** `[Project] Tagisan: High-performance Multi-LLM Swarm & Mixture-of-Agents in Rust (Runs 100% Offline on Ollama)`
* **Content:**
```markdown
Hey LocalLLaMA!

Most multi-agent frameworks are heavy Python applications that consume hundreds of megabytes of RAM just for their runtime. 

I built **Tagisan (`tgs`)** in Rust to bring high-performance multi-LLM collaboration to local setups:
GitHub: https://github.com/CharleGutierrez/tagisan

### What makes it special for local models:
1. **Dynamic Model Auto-Discovery:** Automatically scans `~/.ollama/models/manifests/` and identifies your installed local models.
2. **Hardware-Aware Memory Prioritization:** Detects host RAM/VRAM and automatically selects optimal models (e.g. prioritizing lightweight models on 8GB hosts, preventing OOM crashes on larger weights).
3. **Sub-400ms Model Swapping:** Uses `keep_alive: "24h"`, `num_gpu: 99`, and memory mapping (`use_mmap: true`) to rotate models cleanly during multi-turn debates.
4. **DeepSeek `<think>` State Machine:** Parses and separates internal chain-of-thought blocks from final outputs.
5. **Single Static Binary:** No Python virtual environments or conflicting pip packages.

Try running a local debate right now:
```bash
curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh
tgs debate "Rust vs Go for high concurrency microservices" --local
```

Feedback and PRs are welcome!
```

---

## 3. Reddit: `r/rust`

* **Subreddit:** `r/rust`
* **Title:** `Tagisan (tgs): High-performance Multi-LLM Swarms & Mixture-of-Agents in Rust`
* **Content:**
```markdown
Hi r/rust!

I wanted to share Tagisan (`tgs`), a multi-agent orchestration engine written in pure Rust:
https://github.com/CharleGutierrez/tagisan

### Tech Stack & Architecture:
- **Tokio & Tokio-Stream:** Concurrent fan-out/fan-in for parallel Mixture-of-Agents (MoA) proposers and streaming token delivery.
- **Petgraph:** 5-stage parallel DAG execution pipelines (`ecc_plan` -> `ecc_test` -> `ecc_implement` -> `ecc_review` -> `ecc_security`).
- **Sub-millisecond Skill Dispatcher:** Hybrid BM25/TF-IDF and semantic trigger matcher indexing 70 built-in skills and 3,900+ on-disk skills in under 2ms.
- **Polyglot Embedded Runtimes:** Sandboxed execution handlers for Bun, Python 3 (uv), and Perl 5 with AST-level security interceptors.

Code and benchmarks are in the repo. Any feedback on our async channel fan-out or dispatch algorithms is appreciated!
```

---

## 4. Twitter / X Launch Thread

```text
1/6 🚀 Introducing Tagisan (tgs): The 100x faster, single-binary Rust alternative to bloated Python multi-agent frameworks.

Run Mixture-of-Agents (MoA), dialectical debates, and 70+ built-in engineering skills in ~2ms with zero Python dependencies.

🔗 https://github.com/CharleGutierrez/tagisan

2/6 🥊 Dialectical Debate (Tagisan ng Talino):
Why settle for single-model hallucination?
`tgs debate` pits two models against each other (Thesis vs. Antithesis), while an independent Lakandiwa adjudicator synthesizes the final mathematically grounded verdict.

3/6 ⚡ Performance & Resource Efficiency:
- Startup: 2.1ms (vs 1,800ms in Python frameworks)
- Idle RAM: 18MB (vs 500MB+)
- Single 29MB static binary. No pip, no virtual environments.

4/6 🦙 100% Offline Local Ollama Engine:
Auto-discovers installed models from ~/.ollama, pins GPU layers, and hot-swaps models dynamically with sub-400ms latency. Run debates between Qwen 2.5 and DeepSeek R1 completely offline.

5/6 📚 70+ Built-in Engineering Skills:
Sub-millisecond semantic dispatcher with quantitative foundation models (Kronos, Qlib, Chronos), applied math (The Nature of Code, Lengyel 3D math, Pearl causality), UX, and SRE audits.

6/6 Get started in 5 seconds:
curl -fsSL https://raw.githubusercontent.com/CharleGutierrez/tagisan/main/install.sh | sh
tgs debate "Postgres vs Mongo" --local

Star the repo if you believe AI tooling belongs in systems-grade Rust! ⭐
```
