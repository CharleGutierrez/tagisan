---
name: pp-cschain-connector-actions-openapi-plugins
description: "Exposing Power Platform connectors directly as generative actions without intermediary flows."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-22"]
triggers: ["connector-actions-openapi", "direct-connector-plugins", "openapi-tool-calling-cs"]
---

# pp-cschain-connector-actions-openapi-plugins

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Autonomous Multi-Action Agents with Copilot Studio - Microsoft Learn'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-cschain-connector-actions-openapi-plugins workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-cschain-connector-actions-openapi-plugins processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Autonomous Multi-Action Agents with Copilot Studio - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-cschain-connector-actions-openapi-plugins protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-cschain-connector-actions-openapi-plugins directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-cschain-connector-actions-openapi-plugins execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("connector-actions-openapi", "direct-connector-plugins", "openapi-tool-calling-cs") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
