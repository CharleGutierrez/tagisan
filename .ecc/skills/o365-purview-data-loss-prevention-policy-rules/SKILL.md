---
name: o365-purview-data-loss-prevention-policy-rules
description: "Creating DLP policies across Exchange, SharePoint, OneDrive, Teams, and endpoints."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-29"]
triggers: ["dlp-policy-rules-purview", "data-loss-prevention-rules", "endpoint-dlp-blocking"]
---

# o365-purview-data-loss-prevention-policy-rules

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Microsoft Purview Information Protection Architecture - Peter De Tender'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-purview-data-loss-prevention-policy-rules pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-purview-data-loss-prevention-policy-rules operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Microsoft Purview Information Protection Architecture - Peter De Tender*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-purview-data-loss-prevention-policy-rules protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-purview-data-loss-prevention-policy-rules directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-purview-data-loss-prevention-policy-rules execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("dlp-policy-rules-purview", "data-loss-prevention-rules", "endpoint-dlp-blocking") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
