---
name: pp-dax-semi-additive-measures-inventory
description: "Semi-additive measures for balance sheets and inventory using CLOSINGBALANCEMONTH and LASTDATE."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-15"]
triggers: ["semi-additive-measures", "inventory-balance-dax", "closingbalancemonth"]
---

# pp-dax-semi-additive-measures-inventory

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'The Definitive Guide to DAX - Marco Russo & Alberto Ferrari'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-dax-semi-additive-measures-inventory workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-dax-semi-additive-measures-inventory processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *The Definitive Guide to DAX - Marco Russo & Alberto Ferrari*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-dax-semi-additive-measures-inventory protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-dax-semi-additive-measures-inventory directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-dax-semi-additive-measures-inventory execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("semi-additive-measures", "inventory-balance-dax", "closingbalancemonth") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
