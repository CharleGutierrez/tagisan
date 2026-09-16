---
name: o365-alm-pnp-powershell-site-provisioning
description: "Modern site provisioning, schema templates, bulk permission assignments via PnP PowerShell."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-34"]
triggers: ["pnp-powershell-site-provisioning", "declarative-site-templates", "bulk-permission-assignments-pnp"]
---

# o365-alm-pnp-powershell-site-provisioning

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Mastering CLI for Microsoft 365 - Waldek Mastykarz & Garry Trinder'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-alm-pnp-powershell-site-provisioning pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-alm-pnp-powershell-site-provisioning operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Mastering CLI for Microsoft 365 - Waldek Mastykarz & Garry Trinder*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-alm-pnp-powershell-site-provisioning protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-alm-pnp-powershell-site-provisioning directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-alm-pnp-powershell-site-provisioning execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("pnp-powershell-site-provisioning", "declarative-site-templates", "bulk-permission-assignments-pnp") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
