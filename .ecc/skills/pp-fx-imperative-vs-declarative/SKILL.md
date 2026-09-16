---
name: pp-fx-imperative-vs-declarative
description: "Declarative With(), Sequence(), and Set vs UpdateContext scoping, pure functional composition, and side-effect minimization."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["power-fx-imperative-vs-declarative", "with-function-power-fx", "updatecontext-vs-set", "pure-functional-power-fx"]
---

# pp-fx-imperative-vs-declarative

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Declarative Paradigm: Power Fx expressions compute values dynamically based on reactive graph dependencies without imperative mutations.
- Scope Invariant: ALWAYS prefer `UpdateContext()` for screen-scoped temporary state and `Set()` solely for tenant-wide global variables.
- NEVER mutate global collections inside gallery Item properties or visual calculation pipelines.
- MANDATORY use of `With({ scopeVar: expression }, ...)` to eliminate redundant sub-formula evaluations.
- Source Reference: *Power Fx Formula Reference & Advanced Patterns - Greg Lindhorst*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Refactor nested repetitive lookups into `With({ activeAccount: LookUp(Accounts, ID = selectedId) }, activeAccount.Revenue * activeAccount.TaxRate)`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Using Set() inside a Gallery OnSelect to set 15 separate global variables when a single record variable suffices.
- Creating circular formula dependencies causing infinite re-render loops in Power Apps Studio.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("power-fx-imperative-vs-declarative", "with-function-power-fx", "updatecontext-vs-set", "pure-functional-power-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
