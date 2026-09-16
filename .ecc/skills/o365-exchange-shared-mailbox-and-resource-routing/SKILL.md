---
name: o365-exchange-shared-mailbox-and-resource-routing
description: "Programmatically provisioning shared mailboxes, meeting rooms, resource booking."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-28"]
triggers: ["shared-mailbox-provisioning", "meeting-room-resource-booking", "resource-mailbox-routing"]
---

# o365-exchange-shared-mailbox-and-resource-routing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Exchange Online Administration with PowerShell - Tony Redmond'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-exchange-shared-mailbox-and-resource-routing pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-exchange-shared-mailbox-and-resource-routing operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Exchange Online Administration with PowerShell - Tony Redmond*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-exchange-shared-mailbox-and-resource-routing protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-exchange-shared-mailbox-and-resource-routing directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-exchange-shared-mailbox-and-resource-routing execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("shared-mailbox-provisioning", "meeting-room-resource-booking", "resource-mailbox-routing") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
