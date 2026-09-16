---
name: pp-dv-auditing-retention-lifecycle
description: "Field-level auditing configuration, audit partition archiving, and compliance policies."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-6"]
triggers: ["auditing-retention-lifecycle", "field-level-auditing", "audit-partition-management"]
---

# pp-dv-auditing-retention-lifecycle

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Data Modeling for Microsoft Dataverse - Marc Gerner'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-dv-auditing-retention-lifecycle workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-dv-auditing-retention-lifecycle processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Data Modeling for Microsoft Dataverse - Marc Gerner*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-dv-auditing-retention-lifecycle protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-dv-auditing-retention-lifecycle directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-dv-auditing-retention-lifecycle execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("auditing-retention-lifecycle", "field-level-auditing", "audit-partition-management") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
