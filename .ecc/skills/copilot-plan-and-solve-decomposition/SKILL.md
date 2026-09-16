---
name: copilot-plan-and-solve-decomposition
description: "Hierarchical plan-and-solve prompting for multi-step enterprise business processes."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Autonomous Planning and Execution with Semantic Kernel - John Maeda"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["plan-and-solve", "hierarchical-planning", "task-decomposition-prompts", "step-by-step-execution"]
---

# copilot-plan-and-solve-decomposition
> Based on **Autonomous Planning and Execution with Semantic Kernel - John Maeda** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Two-Stage Prompting: 1) Devise a comprehensive multi-step plan, 2) Execute each step sequentially with verification.**
2. **Interleaved Verification: Validate the output of step N before permitting the model to execute step N+1.**
3. **MANDATORY abort and replan if any intermediate step fails.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-plan-and-solve-decomposition.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-plan-and-solve-decomposition.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure complex business workflows using Plan-and-Solve prompt architectures for deterministic multi-stage execution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-plan-and-solve-decomposition.**
- **Unmonitored runtime execution without telemetry in copilot-plan-and-solve-decomposition.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "plan-and-solve"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
