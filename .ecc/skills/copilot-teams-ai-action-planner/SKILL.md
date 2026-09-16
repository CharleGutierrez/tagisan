---
name: copilot-teams-ai-action-planner
description: "ActionPlanner configuration, plan generation, tool calling loops, and model parameter tuning."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Action Planners and Augmented LLM Orchestration - Microsoft Engineering"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-ai-action-planner", "actionplanner-config", "tool-calling-loops", "llm-turn-orchestration"]
---

# copilot-teams-ai-action-planner
> Based on **Action Planners and Augmented LLM Orchestration - Microsoft Engineering** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Plan Contract: LLM outputs predicted actions (DO <action> <parameters> or SAY <response>).**
2. **Execution Guardrail: Limit maximum tool loops (default: 5) to prevent infinite planner oscillation.**
3. **MANDATORY validation of action parameters against typed TypeScript interfaces.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-ai-action-planner.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-action-planner.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure `ActionPlanner` with strict prompt templates and registered actions for reliable multi-step agent execution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-action-planner.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-action-planner.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-ai-action-planner"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
