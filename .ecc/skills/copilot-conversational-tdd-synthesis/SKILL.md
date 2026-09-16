---
name: copilot-conversational-tdd-synthesis
description: "Prompting test-driven red-green-refactor loops directly in conversational pair programming."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Test-Driven Development for AI Agents - Kent Beck"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["conversational-tdd", "red-green-refactor-prompts", "prompt-driven-testing", "tdd-agent-synthesis"]
---

# copilot-conversational-tdd-synthesis
> Based on **Test-Driven Development for AI Agents - Kent Beck** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **TDD Loop: 1) Prompt test creation, 2) Verify failure (Red), 3) Prompt minimal code (Green), 4) Refactor.**
2. **Contract Preservation: Tests serve as immutable contracts that the conversational agent cannot violate.**
3. **ALWAYS run tests automatically on file save during vibe coding sessions.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-conversational-tdd-synthesis.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-conversational-tdd-synthesis actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce conversational TDD: prompt the AI to write comprehensive unit tests before generating application logic.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-conversational-tdd-synthesis.**
- **Unmonitored runtime execution without telemetry in copilot-conversational-tdd-synthesis.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "conversational-tdd"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
