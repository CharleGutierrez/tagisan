---
name: copilot-sk-auto-function-calling
description: "Automatic function invocation filters, execution loop controls, and max-iteration safeguards."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Autonomous Planning and Execution with Semantic Kernel - John Maeda"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-auto-function-calling", "tool-call-behavior", "function-invocation-loop", "max-iterations-safeguard"]
---

# copilot-sk-auto-function-calling
> Based on **Autonomous Planning and Execution with Semantic Kernel - John Maeda** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Execution Setting: Set `FunctionChoiceBehavior = FunctionChoiceBehavior.Auto()`.**
2. **Loop Guard: Configure `MaximumAutoInvokeAttempts` (e.g. 5) to abort runaway tool loops.**
3. **MANDATORY inspection of function call exceptions during auto-invocation.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-sk-auto-function-calling.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-auto-function-calling.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure automatic tool calling in Semantic Kernel, ensuring robust loop limits and error handling around function invocations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-auto-function-calling.**
- **Unmonitored runtime execution without telemetry in copilot-sk-auto-function-calling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-auto-function-calling"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
