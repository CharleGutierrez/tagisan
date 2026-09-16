---
name: copilot-teams-ai-testing-and-debugging
description: "Teams App Test Tool, Bot Framework Emulator, and automated TurnContext mock test suites."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Testing and Validating Intelligent Agent Plugins - Lisa Crispin"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-ai-testing", "teams-test-tool", "bot-framework-emulator", "turncontext-mock-tests"]
---

# copilot-teams-ai-testing-and-debugging
> Based on **Testing and Validating Intelligent Agent Plugins - Lisa Crispin** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Test Isolation: Run automated integration tests against simulated `TurnContext` without live Teams connections.**
2. **Deterministic Testing: Mock external LLM responses to test planner and action execution deterministically.**
3. **MANDATORY unit test coverage for all custom action handlers.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-ai-testing-and-debugging.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-testing-and-debugging.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Establish comprehensive test suites using Teams App Test Tool and Jest/Vitest to verify agent behavior across multi-turn scenarios.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-testing-and-debugging.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-testing-and-debugging.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-ai-testing"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
