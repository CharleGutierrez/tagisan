---
name: pp-appr-guest-user-external-approvals
description: "Routing approvals to external B2B guest users via actionable email messages safely."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-11"]
triggers: ["guest-user-external-approvals", "b2b-actionable-email-approvals", "external-partner-signoff"]
---

# pp-appr-guest-user-external-approvals

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Approvals Architecture in Power Platform - Vivek Bavishi'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-appr-guest-user-external-approvals workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-appr-guest-user-external-approvals processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Approvals Architecture in Power Platform - Vivek Bavishi*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-appr-guest-user-external-approvals protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-appr-guest-user-external-approvals directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-appr-guest-user-external-approvals execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("guest-user-external-approvals", "b2b-actionable-email-approvals", "external-partner-signoff") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
