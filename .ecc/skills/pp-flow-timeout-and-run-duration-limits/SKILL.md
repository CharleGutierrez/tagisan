---
name: pp-flow-timeout-and-run-duration-limits
description: "Action timeout customization, PT1H syntax, and managing the 30-day asynchronous flow limit."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-9"]
triggers: ["flow-timeout-limits", "action-run-duration", "30-day-asynchronous-limit"]
---

# pp-flow-timeout-and-run-duration-limits

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Automating Enterprise Workflows with Power Automate - Serge Luca'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-flow-timeout-and-run-duration-limits workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-flow-timeout-and-run-duration-limits processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Automating Enterprise Workflows with Power Automate - Serge Luca*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-flow-timeout-and-run-duration-limits protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-flow-timeout-and-run-duration-limits directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-flow-timeout-and-run-duration-limits execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("flow-timeout-limits", "action-run-duration", "30-day-asynchronous-limit") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
