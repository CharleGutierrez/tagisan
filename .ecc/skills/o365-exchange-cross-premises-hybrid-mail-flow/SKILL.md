---
name: o365-exchange-cross-premises-hybrid-mail-flow
description: "Inbound/outbound mail flow connectors between Exchange Online and on-prem Exchange."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-28"]
triggers: ["hybrid-mail-flow-connectors", "cross-premises-mail-routing", "exchange-hybrid-connectors"]
---

# o365-exchange-cross-premises-hybrid-mail-flow

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Exchange Online Administration with PowerShell - Tony Redmond'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-exchange-cross-premises-hybrid-mail-flow pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-exchange-cross-premises-hybrid-mail-flow operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Exchange Online Administration with PowerShell - Tony Redmond*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-exchange-cross-premises-hybrid-mail-flow protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-exchange-cross-premises-hybrid-mail-flow directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-exchange-cross-premises-hybrid-mail-flow execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("hybrid-mail-flow-connectors", "cross-premises-mail-routing", "exchange-hybrid-connectors") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
