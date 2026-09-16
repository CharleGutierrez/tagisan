---
name: pp-pbi-incremental-refresh-range-partitions
description: "Incremental Refresh policy configuration with RangeStart and RangeEnd date parameters."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-14"]
triggers: ["incremental-refresh-pbi", "range-start-range-end", "partition-archiving-pbi"]
---

# pp-pbi-incremental-refresh-range-partitions

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Analyzing Data with Microsoft Power BI - Alberto Ferrari & Marco Russo'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pbi-incremental-refresh-range-partitions workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pbi-incremental-refresh-range-partitions processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Analyzing Data with Microsoft Power BI - Alberto Ferrari & Marco Russo*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pbi-incremental-refresh-range-partitions protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pbi-incremental-refresh-range-partitions directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pbi-incremental-refresh-range-partitions execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("incremental-refresh-pbi", "range-start-range-end", "partition-archiving-pbi") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
