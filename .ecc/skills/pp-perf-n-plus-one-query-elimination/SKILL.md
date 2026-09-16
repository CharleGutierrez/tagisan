---
name: pp-perf-n-plus-one-query-elimination
description: "Eliminating N+1 LookUp loops inside galleries by leveraging collections and single batch queries."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-5"]
triggers: ["n-plus-one-query-elimination", "gallery-lookup-optimization", "batch-query-hydration"]
---

# pp-perf-n-plus-one-query-elimination

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-perf-n-plus-one-query-elimination workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-perf-n-plus-one-query-elimination processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-perf-n-plus-one-query-elimination protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-perf-n-plus-one-query-elimination directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-perf-n-plus-one-query-elimination execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("n-plus-one-query-elimination", "gallery-lookup-optimization", "batch-query-hydration") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
