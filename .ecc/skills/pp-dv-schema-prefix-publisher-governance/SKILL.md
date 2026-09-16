---
name: pp-dv-schema-prefix-publisher-governance
description: "Publisher prefix governance, collision avoidance, and enterprise naming conventions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-6"]
triggers: ["schema-prefix-governance", "solution-publisher-prefix", "naming-conventions-dataverse"]
---

# pp-dv-schema-prefix-publisher-governance

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Data Modeling for Microsoft Dataverse - Marc Gerner'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-dv-schema-prefix-publisher-governance workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-dv-schema-prefix-publisher-governance processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Data Modeling for Microsoft Dataverse - Marc Gerner*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-dv-schema-prefix-publisher-governance protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-dv-schema-prefix-publisher-governance directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-dv-schema-prefix-publisher-governance execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("schema-prefix-governance", "solution-publisher-prefix", "naming-conventions-dataverse") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
