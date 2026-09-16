---
name: pp-perf-odata-delegation-matrix
description: "Delegation compatibility across Dataverse, SharePoint, and Azure SQL data sources."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-5"]
triggers: ["odata-delegation-matrix", "sharepoint-vs-dataverse-delegation", "sql-delegation-support"]
---

# pp-perf-odata-delegation-matrix

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-perf-odata-delegation-matrix workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-perf-odata-delegation-matrix processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-perf-odata-delegation-matrix protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-perf-odata-delegation-matrix directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-perf-odata-delegation-matrix execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("odata-delegation-matrix", "sharepoint-vs-dataverse-delegation", "sql-delegation-support") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
