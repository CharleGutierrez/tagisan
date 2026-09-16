---
name: pp-aiprompt-sentiment-and-intent-triage
description: "Classifying customer sentiment, urgency levels, and underlying intents in a single prompt run."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-32"]
triggers: ["sentiment-and-intent-triage", "urgency-classification-ai", "intent-detection-prompts"]
---

# pp-aiprompt-sentiment-and-intent-triage

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Prompt Engineering in AI Builder and Power Platform - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-aiprompt-sentiment-and-intent-triage workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-aiprompt-sentiment-and-intent-triage processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Prompt Engineering in AI Builder and Power Platform - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-aiprompt-sentiment-and-intent-triage protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-aiprompt-sentiment-and-intent-triage directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-aiprompt-sentiment-and-intent-triage execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("sentiment-and-intent-triage", "urgency-classification-ai", "intent-detection-prompts") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
