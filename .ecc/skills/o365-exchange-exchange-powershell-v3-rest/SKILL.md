---
name: o365-exchange-exchange-powershell-v3-rest
description: "ExchangeOnlineManagement v3 cmdlets, REST-backed execution, token authentication."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-28"]
triggers: ["exchange-powershell-v3", "exchangeonlinemanagement-rest", "modern-exchange-cmdlets"]
---

# o365-exchange-exchange-powershell-v3-rest

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Exchange Online Administration with PowerShell - Tony Redmond'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-exchange-exchange-powershell-v3-rest pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-exchange-exchange-powershell-v3-rest operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Exchange Online Administration with PowerShell - Tony Redmond*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-exchange-exchange-powershell-v3-rest protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-exchange-exchange-powershell-v3-rest directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-exchange-exchange-powershell-v3-rest execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("exchange-powershell-v3", "exchangeonlinemanagement-rest", "modern-exchange-cmdlets") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
