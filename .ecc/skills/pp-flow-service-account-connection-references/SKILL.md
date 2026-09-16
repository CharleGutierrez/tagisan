---
name: pp-flow-service-account-connection-references
description: "Migrating personal user connections to Service Principal Connection References in solutions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-9"]
triggers: ["service-account-connections", "connection-references-alm", "spn-flow-connections"]
---

# pp-flow-service-account-connection-references

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Automating Enterprise Workflows with Power Automate - Serge Luca'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-flow-service-account-connection-references workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-flow-service-account-connection-references processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Automating Enterprise Workflows with Power Automate - Serge Luca*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-flow-service-account-connection-references protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-flow-service-account-connection-references directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-flow-service-account-connection-references execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("service-account-connections", "connection-references-alm", "spn-flow-connections") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
