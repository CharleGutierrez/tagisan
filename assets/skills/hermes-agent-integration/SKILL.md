---
name: hermes-agent-integration
description: Native Nous Hermes Agent Integration for Tagisan (TGS). Provides full XML grammar parsing and formatting (<tools>, <thought>, <tool_call>, <tool_response>), Hybrid Mixture-of-Agents (MoA) local workhorse routing with cost tracking, and unconstrained adversarial red-team code auditing with zero corporate refusal friction. Triggers: hermes, hermes-agent, hermes-redteam, hermes-workhorse, nous-research, xml-tools, hybrid-tier, redteam-audit, moa-cost-tracker.
version: 1.0.0
tags:
  - hermes
  - nous-research
  - open-weights
  - function-calling
  - xml-protocol
  - hybrid-moa
  - red-team
  - cost-optimization
compatibility: ">=0.2.0"
---

# Hermes Agent Integration: Sovereign Open-Weights Engine & XML Grammar

## Executive Overview
The **Nous Research Hermes series** (Hermes 2 Pro, Hermes 3) represents the vanguard of open-weights autonomous agent engineering. Unlike corporate cloud models bound by hypersensitive refusal boundaries and proprietary JSON schema conventions, Hermes models adhere to an elegant, deterministic XML function-calling grammar:
- `<tools>`: Tool declarations embedded directly in the system prompt.
- `<thought>`: Transparent internal chain-of-thought scratchpad reasoning.
- `<tool_call>`: Deterministic tool invocations supporting multiple parallel or sequential calls.
- `<tool_response>`: High-fidelity tool output reflection for iterative execution.

Tagisan (`TGS`) natively integrates the complete Hermes agent lifecycle, providing:
1. **`HermesXmlProtocol`**: A high-performance parser and serializer in Rust handling multi-call parsing, raw JSON extraction, and markdown block stripping.
2. **`HermesHybridTier`**: An intelligent Mixture-of-Agents (MoA) router combining zero-marginal-cost local Hermes workhorses with frontier cloud arbiters, including exact dollar savings telemetry.
3. **`HermesRedTeamAuditor`**: An unconstrained adversarial auditor that relentlessly probes codebases for concurrency races, TOCTOU bugs, memory safety violations, and timing side-channels without corporate refusal friction.

---

## 1. Hermes XML Grammar Specification

### System Tool Declaration (`<tools>`)
When tools are provisioned to Hermes, they are injected into the system prompt formatted as JSON schemas enclosed in `<tools>...</tools>` tags:
```xml
<tools>
{"type": "function", "function": {"name": "read_file", "description": "Read file contents", "parameters": {"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]}}}
{"type": "function", "function": {"name": "run_command", "description": "Execute shell command", "parameters": {"type": "object", "properties": {"cmd": {"type": "string"}}, "required": ["cmd"]}}}
</tools>
```

### Model Generation with Scratchpad Reasoning & Tool Invocations
Hermes models emit scratchpad reasoning in `<thought>` blocks before dispatching tool calls:
```xml
<thought>
The user requested an analysis of authentication tokens. I should inspect `src/auth.rs` to audit cryptographic key handling.
</thought>
<tool_call>
{"name": "read_file", "arguments": {"path": "src/auth.rs"}}
</tool_call>
```

Multiple tool calls can be emitted concurrently in a single turn:
```xml
<tool_call>
{"name": "read_file", "arguments": {"path": "src/lib.rs"}}
</tool_call>
<tool_call>
{"name": "read_file", "arguments": {"path": "Cargo.toml"}}
</tool_call>
```

### Tool Response Injection (`<tool_response>`)
Outputs returned from executed tools are fed back into the conversational context wrapped in `<tool_response>` tags:
```xml
<tool_response>
{"name": "read_file", "content": "pub mod auth;\npub mod hermes;"}
</tool_response>
```

---

## 2. Hybrid Mixture-of-Agents (MoA) Tier

Running all tasks on frontier cloud models (e.g. Claude 3.5 Sonnet, GPT-4o) introduces prohibitive latency and cloud API costs. Conversely, running solely on 8B local models can compromise strategic system design.

The **Tagisan Hermes Hybrid Tier** dynamically bifurcates tasks:
- **Local Hermes Workhorse (`hermes3:8b`)**:
  - Zero marginal cost ($0.00 / 1M tokens)
  - Sub-50ms local inference on modern silicon (Ollama / vLLM / GGUF)
  - Responsibilities: Routine file edits, compiler syntax autofix, unit test generation, tool execution formatting, and aggressive red-team code probing.
- **Frontier Cloud Orchestrator (`claude-3-5-sonnet` / `gemini-2.5-pro`)**:
  - Responsibilities: Planetary-scale architectural boundaries, dialectical debate arbitration, and high-stakes judicial synthesis.

### Cost Savings Telemetry
`HermesHybridTier` tracks token consumption across both tiers in real-time, computing counterfactual spend (what the workload would have cost if 100% cloud-routed) and net dollar savings.

---

## 3. Adversarial Red-Team Code Auditing

Mainstream cloud models frequently exhibit "refusal friction"—halting analysis when asked to produce attack payloads or analyze vulnerable patterns. `HermesRedTeamAuditor` harnesses Hermes 3's unfiltered open-weights capabilities to conduct offensive security auditing across 6 core attack surfaces:
1. **Happens-Before & Race Hazards**: TOCTOU race conditions, lock order inversions, deadlocks, and atomicity breaches.
2. **Cryptographic & Side-Channel Timing**: Non-constant-time comparisons (`memcmp`, `==`), PRNG predictability, and nonce reuse.
3. **Memory Safety & Unsafe FFI Bounds**: Raw pointer arithmetic, slice off-by-one errors, and C ABI lifetime leaks.
4. **Untrusted Input & Injection**: Path traversal, shell interpolation, and deserialization gadgets.
5. **Resource Exhaustion & Algorithmic DoS**: Catastrophic regex backtracking (ReDoS), memory leaks, and CPU spin loops.
6. **Authorization & State Tampering**: State machine bypasses, privilege escalation, and token replay bugs.

---

## 4. Built-in ECC Presets

Tagisan exposes three dedicated Hermes presets in `src/ecc/presets.rs`:

| Preset Name | Target Model | Included Tools | Core Specialization |
|---|---|---|---|
| `hermes-agent` | `hermes3:latest` | `read_file`, `write_file`, `run_command`, `calculator`, `search_skills` | Full autonomous XML tool-calling and scratchpad reasoning agent. |
| `hermes-redteam` | `hermes3:70b` | `read_file`, `run_command`, `calculator`, `search_skills` | Uncompromising adversarial security probe generator and exploit PoC synthesizer. |
| `hermes-workhorse` | `hermes3:8b` | `read_file`, `write_file`, `run_command` | High-speed, zero-cost local workhorse for routine refactoring and test authoring. |

---

## 5. Interactive REPL Slash Command (`/hermes`)

Within the interactive agent REPL (`tgs repl`), Hermes features are managed via `/hermes`:
```bash
/hermes status               # Display hybrid tier configuration, model bindings, and cost savings
/hermes redteam <path>       # Run adversarial red-team audit probes against targeted code
/hermes hybrid               # Display local vs cloud token telemetry table
/hermes parse <raw_xml>      # Test XML tool call and thought parsing interactively
/hermes help                 # Display Hermes command documentation
```
Tab completion is fully enabled for `/hermes` and its subcommands.
