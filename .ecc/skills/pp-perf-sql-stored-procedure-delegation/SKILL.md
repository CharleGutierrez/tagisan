---
name: pp-perf-sql-stored-procedure-delegation
description: "Invoking Direct SQL Stored Procedures for multi-million row analytics and heavy aggregations."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-5"]
triggers: ["sql-stored-procedure-delegation", "direct-sproc-execution", "sql-analytics-canvas"]
---

# pp-perf-sql-stored-procedure-delegation

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-perf-sql-stored-procedure-delegation workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-perf-sql-stored-procedure-delegation processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-perf-sql-stored-procedure-delegation protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-perf-sql-stored-procedure-delegation directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-perf-sql-stored-procedure-delegation execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("sql-stored-procedure-delegation", "direct-sproc-execution", "sql-analytics-canvas") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
