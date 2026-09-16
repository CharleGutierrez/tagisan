---
name: o365-excel-scripts-range-batch-read-write
description: "Bulk getValues() and setValues() batching, eliminating cell-by-cell mutation latency."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-2"]
triggers: ["range-batch-read-write", "office-scripts-batching", "bulk-range-setvalues"]
---

# o365-excel-scripts-range-batch-read-write

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Automating Excel with Office Scripts - Yevgen Rogozin & Sudhi Ramamurthy'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-excel-scripts-range-batch-read-write pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-excel-scripts-range-batch-read-write operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Automating Excel with Office Scripts - Yevgen Rogozin & Sudhi Ramamurthy*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-excel-scripts-range-batch-read-write protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-excel-scripts-range-batch-read-write directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-excel-scripts-range-batch-read-write execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("range-batch-read-write", "office-scripts-batching", "bulk-range-setvalues") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
