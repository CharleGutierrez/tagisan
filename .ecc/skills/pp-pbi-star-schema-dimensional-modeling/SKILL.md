---
name: pp-pbi-star-schema-dimensional-modeling
description: "Designing optimal Star Schemas: Fact tables, Dimension tables, surrogate keys, and avoiding snowflakes."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-14"]
triggers: ["star-schema-modeling", "dimensional-modeling-pbi", "fact-dimension-tables"]
---

# pp-pbi-star-schema-dimensional-modeling

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Analyzing Data with Microsoft Power BI - Alberto Ferrari & Marco Russo'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pbi-star-schema-dimensional-modeling workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pbi-star-schema-dimensional-modeling processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Analyzing Data with Microsoft Power BI - Alberto Ferrari & Marco Russo*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pbi-star-schema-dimensional-modeling protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pbi-star-schema-dimensional-modeling directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pbi-star-schema-dimensional-modeling execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("star-schema-modeling", "dimensional-modeling-pbi", "fact-dimension-tables") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
