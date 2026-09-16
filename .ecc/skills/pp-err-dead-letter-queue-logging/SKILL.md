---
name: pp-err-dead-letter-queue-logging
description: "Routing terminal flow exceptions to Azure Service Bus or Dataverse Dead Letter Queue tables."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-10"]
triggers: ["dead-letter-queue-logging", "flow-dlq-pattern", "terminal-exception-routing"]
---

# pp-err-dead-letter-queue-logging

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-err-dead-letter-queue-logging workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-err-dead-letter-queue-logging processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-err-dead-letter-queue-logging protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-err-dead-letter-queue-logging directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-err-dead-letter-queue-logging execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("dead-letter-queue-logging", "flow-dlq-pattern", "terminal-exception-routing") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
