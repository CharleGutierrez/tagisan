---
name: pp-aiprompt-multimodal-vision-prompts
description: "Passing image and PDF document attachments to multimodal GPT-4o vision models in prompts."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-32"]
triggers: ["multimodal-vision-prompts", "gpt-4o-vision-prompts", "image-document-understanding"]
---

# pp-aiprompt-multimodal-vision-prompts

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Prompt Engineering in AI Builder and Power Platform - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-aiprompt-multimodal-vision-prompts workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-aiprompt-multimodal-vision-prompts processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Prompt Engineering in AI Builder and Power Platform - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-aiprompt-multimodal-vision-prompts protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-aiprompt-multimodal-vision-prompts directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-aiprompt-multimodal-vision-prompts execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("multimodal-vision-prompts", "gpt-4o-vision-prompts", "image-document-understanding") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
