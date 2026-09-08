# 🇵🇭 Tagisan (`tagisan-rs`)
> **Tagisan ng Talino:** High-Performance Multi-LLM Collaboration, Adversarial Debate & Mixture-of-Agents Engine in Rust.

---

## 🌟 Overview
**Tagisan** is an asynchronous, zero-cost abstraction engine written in Rust that orchestrates heterogeneous Large Language Models—**Anthropic Claude, xAI Grok, Google Gemini, OpenAI, DeepSeek, and local Ollama**—into a collaborative intelligence swarm.

Instead of relying on a single AI model (which can hallucinate), Tagisan enables models to **debate, critique, cross-verify, and aggregate** their outputs to generate rock-solid, audited code and architectures.

---

## 🚀 Key Collaboration Strategies

### 1. ⚔️ Dialectical Debate (*Tagisan ng Talino / Balagtasan*)
- **Round 1 (Thesis):** Proponent model (e.g. Claude 3.5 Sonnet) writes the initial solution.
- **Round 2 (Antithesis):** Adversary model (e.g. DeepSeek R1 / Grok 3) ruthlessly probes for logical flaws, edge cases, and vulnerabilities.
- **Round 3 (Synthesis / Lakandiwa):** Chief Adjudicator (e.g. Google Gemini 1.5 Pro / GPT-4o) evaluates both sides and synthesizes the verified master verdict.

### 2. 🛖 Mixture-of-Agents (MoA)
- **Layer 1 (Parallel Proposers):** Grok, Gemini, and DeepSeek generate candidate drafts concurrently in milliseconds using Tokio async channels.
- **Layer 2 (Master Aggregator):** Claude 3.5 Sonnet analyzes, filters, and combines the proposals into a definitive solution.

---

## 📦 Project Structure

```
tagisan/
├── Cargo.toml
├── .env.example              # Template for API keys
├── README.md
└── src/
    ├── main.rs               # CLI Application with colored outputs
    ├── lib.rs                # Library exports
    ├── error.rs              # TagisanError & Result types
    ├── types/                # Role, ContentBlock, Message, TokenUsage, CompletionRequest/Response
    │   └── mod.rs
    ├── providers/            # LLM API Adapters
    │   ├── mod.rs            # LlmProvider trait
    │   ├── anthropic.rs      # Claude 3.5 Sonnet / Opus
    │   ├── openai_compat.rs  # OpenAI, xAI (Grok), DeepSeek (R1 / V3)
    │   ├── gemini.rs         # Google Gemini 1.5 Pro / 2.0 Flash
    │   └── ollama.rs         # Local offline inference (localhost:11434)
    ├── strategies/           # Multi-LLM Collaboration Algorithms
    │   ├── mod.rs            # CollaborationStrategy trait
    │   ├── moa.rs            # Mixture-of-Agents parallel runner
    │   └── debate.rs         # Dialectical Debate (Tagisan ng Talino)
    └── engine/
        ├── mod.rs            # EngineContext & Orchestrator
        └── budget.rs         # Atomic USD Token Cost Tracker
```

---

## 🛠️ Quick Start

### 1. Set Up API Keys
Copy `.env.example` to `.env` in the `tagisan` directory:

```bash
cp .env.example .env
```

Add any keys you have available:
```env
ANTHROPIC_API_KEY=sk-ant-...
XAI_API_KEY=xai-...
GEMINI_API_KEY=AIzaSy...
DEEPSEEK_API_KEY=sk-...
OPENAI_API_KEY=sk-...
```

*(Note: If no API keys are provided, Tagisan falls back automatically to local Ollama on `localhost:11434`)*

---

### 2. Check System Status
```bash
cargo run -- status
```

---

### 3. Run Dialectical Debate (`debate`)
```bash
cargo run -- debate "Should a high-throughput payment engine use an Event-Sourced architecture or CRUD with Postgres?"
```

---

### 4. Run Mixture-of-Agents (`moa`)
```bash
cargo run -- moa "Design a zero-downtime database migration strategy for 100M active records in Rust"
```

---

## 🛡️ Built-in Cost Protection
Tagisan includes a real-time atomic micro-USD budget tracker. By default, it terminates executions if the session cost exceeds `$5.00` USD. You can customize this threshold with `--max-budget`:

```bash
cargo run -- --max-budget 1.50 moa "Your prompt here"
```
