---
name: pp-csgen-topic-level-moderation-filters
description: "Configuring Content Moderation levels (High, Medium, Low) and graceful safety refusals."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-21"]
triggers: ["topic-moderation-filters", "content-moderation-levels", "safety-refusal-handling"]
---

# pp-csgen-topic-level-moderation-filters

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Conversational AI with Microsoft Copilot Studio - Michael Roth'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-csgen-topic-level-moderation-filters workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-csgen-topic-level-moderation-filters processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Conversational AI with Microsoft Copilot Studio - Michael Roth*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-csgen-topic-level-moderation-filters protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-csgen-topic-level-moderation-filters directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-csgen-topic-level-moderation-filters execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("topic-moderation-filters", "content-moderation-levels", "safety-refusal-handling") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
