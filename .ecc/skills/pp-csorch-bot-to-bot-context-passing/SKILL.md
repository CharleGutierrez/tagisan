---
name: pp-csorch-bot-to-bot-context-passing
description: "Passing conversation history, authenticated user tokens, and variable state between agents."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-24"]
triggers: ["bot-to-bot-context-passing", "agent-context-transfer", "token-passing-multi-agent"]
---

# pp-csorch-bot-to-bot-context-passing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Multi-Agent Systems and Omnichannel Handoff - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-csorch-bot-to-bot-context-passing workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-csorch-bot-to-bot-context-passing processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Multi-Agent Systems and Omnichannel Handoff - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-csorch-bot-to-bot-context-passing protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-csorch-bot-to-bot-context-passing directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-csorch-bot-to-bot-context-passing execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("bot-to-bot-context-passing", "agent-context-transfer", "token-passing-multi-agent") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
