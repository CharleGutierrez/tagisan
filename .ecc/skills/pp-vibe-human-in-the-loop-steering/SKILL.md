---
name: pp-vibe-human-in-the-loop-steering
description: "Steering AI code generation with iterative prompt refinement and targeted architectural guardrails."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-33"]
triggers: ["human-in-the-loop-steering", "iterative-prompt-refinement", "architectural-guardrail-steering"]
---

# pp-vibe-human-in-the-loop-steering

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Vibe Coding: Conversational Development in Power Platform - Andrej Karpathy & Satya Nadella'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-vibe-human-in-the-loop-steering workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-vibe-human-in-the-loop-steering processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Vibe Coding: Conversational Development in Power Platform - Andrej Karpathy & Satya Nadella*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-vibe-human-in-the-loop-steering protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-vibe-human-in-the-loop-steering directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-vibe-human-in-the-loop-steering execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("human-in-the-loop-steering", "iterative-prompt-refinement", "architectural-guardrail-steering") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
