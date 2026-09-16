---
name: pp-err-compensating-transactions-rollback
description: "Orchestrating compensating transactions to roll back partial writes across distributed systems."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-10"]
triggers: ["compensating-transactions", "flow-rollback-logic", "distributed-write-compensation"]
---

# pp-err-compensating-transactions-rollback

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-err-compensating-transactions-rollback workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-err-compensating-transactions-rollback processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-err-compensating-transactions-rollback protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-err-compensating-transactions-rollback directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-err-compensating-transactions-rollback execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("compensating-transactions", "flow-rollback-logic", "distributed-write-compensation") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
