---
name: copilot-fail-fast-diagnostic-surfacing
description: "Surfacing actionable runtime and compilation errors immediately in the conversation thread."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["fail-fast-diagnostics", "traceback-surfacing", "actionable-compiler-errors", "error-recovery-vibe"]
---

# copilot-fail-fast-diagnostic-surfacing
> Based on **A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Error Transparency: Capture and format compiler diagnostics and stack traces directly in the conversation.**
2. **Self-Healing: Feed exact error messages back to the model with instruction to propose targeted diffs.**
3. **NEVER hide or swallow exceptions with empty catch blocks.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-fail-fast-diagnostic-surfacing.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-fail-fast-diagnostic-surfacing actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design diagnostic pipelines that surface compilation and API errors with high fidelity, enabling instant conversational self-healing.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-fail-fast-diagnostic-surfacing.**
- **Unmonitored runtime execution without telemetry in copilot-fail-fast-diagnostic-surfacing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fail-fast-diagnostics"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
