---
name: pp-sec-modern-role-based-app-licensing
description: "Restricting Canvas and Model-Driven App access strictly via Security Role assignment."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-8"]
triggers: ["role-based-app-access", "app-security-role-binding", "licensed-app-protection"]
---

# pp-sec-modern-role-based-app-licensing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Security, Governance, and Compliance in Dataverse - Microsoft Learn'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-sec-modern-role-based-app-licensing workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-sec-modern-role-based-app-licensing processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Security, Governance, and Compliance in Dataverse - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-sec-modern-role-based-app-licensing protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-sec-modern-role-based-app-licensing directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-sec-modern-role-based-app-licensing execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("role-based-app-access", "app-security-role-binding", "licensed-app-protection") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
