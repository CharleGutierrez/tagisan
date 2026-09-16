---
name: pp-gov-connector-action-control
description: "Granular connector action control: Permitting read operations while blocking write/delete actions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-35"]
triggers: ["connector-action-control", "block-connector-actions", "read-only-connector-rules"]
---

# pp-gov-connector-action-control

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Governance, Security, and Compliance in Power Platform - Manuela Pichler'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-gov-connector-action-control workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-gov-connector-action-control processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Governance, Security, and Compliance in Power Platform - Manuela Pichler*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-gov-connector-action-control protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-gov-connector-action-control directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-gov-connector-action-control execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("connector-action-control", "block-connector-actions", "read-only-connector-rules") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
