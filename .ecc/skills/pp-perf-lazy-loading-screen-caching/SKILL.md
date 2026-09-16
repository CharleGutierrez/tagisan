---
name: pp-perf-lazy-loading-screen-caching
description: "Deferring data hydration until screen navigation and gating execution behind OnVisible flags."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-5"]
triggers: ["lazy-loading-screens", "screen-onvisible-gating", "deferred-data-hydration"]
---

# pp-perf-lazy-loading-screen-caching

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-perf-lazy-loading-screen-caching workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-perf-lazy-loading-screen-caching processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-perf-lazy-loading-screen-caching protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-perf-lazy-loading-screen-caching directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-perf-lazy-loading-screen-caching execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("lazy-loading-screens", "screen-onvisible-gating", "deferred-data-hydration") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
