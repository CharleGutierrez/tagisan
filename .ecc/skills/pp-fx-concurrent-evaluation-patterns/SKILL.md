---
name: pp-fx-concurrent-evaluation-patterns
description: "Concurrent() batch execution, network latency minimization, parallel data hydration, and thread safety across connectors."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["concurrent-evaluation-patterns", "concurrent-function-power-fx", "parallel-data-hydration", "network-latency-reduction"]
---

# pp-fx-concurrent-evaluation-patterns

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Parallel Execution: `Concurrent(Op1, Op2, ...)` fires independent connector calls simultaneously, reducing total elapsed time to MAX(times) instead of SUM(times).
- Independence Invariant: Operations inside `Concurrent()` MUST NOT have read-after-write dependencies on each other.
- NEVER call `Concurrent(ClearCollect(colA, ...), ClearCollect(colB, Filter(colA, ...)))` due to race condition hazards.
- MANDATORY inclusion of independent lookup tables (e.g., Accounts, Categories, Users) within startup Concurrent() blocks.
- Source Reference: *High-Velocity Power Fx Optimization - Todd Baginski*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Hydrate reference tables in parallel: `Concurrent(ClearCollect(colAccounts, Accounts), ClearCollect(colContacts, Contacts), ClearCollect(colSettings, SystemSettings))`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Executing 10 consecutive ClearCollect() statements imperatively on screen load, turning a 500ms network fetch into a 5-second wait.
- Including mutate-then-read operations inside the same Concurrent() invocation.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("concurrent-evaluation-patterns", "concurrent-function-power-fx", "parallel-data-hydration", "network-latency-reduction") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
