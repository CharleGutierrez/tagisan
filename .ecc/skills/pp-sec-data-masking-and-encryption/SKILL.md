---
name: pp-sec-data-masking-and-encryption
description: "Customer-Managed Keys (CMK), transparent data encryption, and synthetic column regex masking."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-8"]
triggers: ["data-masking-encryption", "customer-managed-keys-dv", "transparent-data-encryption"]
---

# pp-sec-data-masking-and-encryption

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Security, Governance, and Compliance in Dataverse - Microsoft Learn'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-sec-data-masking-and-encryption workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-sec-data-masking-and-encryption processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Security, Governance, and Compliance in Dataverse - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-sec-data-masking-and-encryption protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-sec-data-masking-and-encryption directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-sec-data-masking-and-encryption execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("data-masking-encryption", "customer-managed-keys-dv", "transparent-data-encryption") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
