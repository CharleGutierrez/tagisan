---
name: pp-appr-ai-summarized-approval-packets
description: "Generating executive summary bullet points inside approval card bodies via AI Builder prompts."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-11"]
triggers: ["ai-summarized-approvals", "copilot-approval-packet", "executive-summary-approval"]
---

# pp-appr-ai-summarized-approval-packets

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Approvals Architecture in Power Platform - Vivek Bavishi'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-appr-ai-summarized-approval-packets workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-appr-ai-summarized-approval-packets processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Approvals Architecture in Power Platform - Vivek Bavishi*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-appr-ai-summarized-approval-packets protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-appr-ai-summarized-approval-packets directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-appr-ai-summarized-approval-packets execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("ai-summarized-approvals", "copilot-approval-packet", "executive-summary-approval") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
