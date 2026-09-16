---
name: pp-sec-service-principal-app-users
description: "S2S Service Principal registration, Dataverse Application Users, and headless authentication."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-8"]
triggers: ["service-principal-app-users", "s2s-application-users", "headless-api-security"]
---

# pp-sec-service-principal-app-users

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Security, Governance, and Compliance in Dataverse - Microsoft Learn'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-sec-service-principal-app-users workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-sec-service-principal-app-users processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Security, Governance, and Compliance in Dataverse - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-sec-service-principal-app-users protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-sec-service-principal-app-users directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-sec-service-principal-app-users execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("service-principal-app-users", "s2s-application-users", "headless-api-security") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
