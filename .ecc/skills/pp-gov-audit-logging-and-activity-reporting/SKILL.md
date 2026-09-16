---
name: pp-gov-audit-logging-and-activity-reporting
description: "Streaming Microsoft 365 / Dataverse audit logs into Azure Log Analytics and Microsoft Sentinel."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-35"]
triggers: ["audit-logging-reporting", "sentinel-power-platform-logs", "tenant-activity-reporting"]
---

# pp-gov-audit-logging-and-activity-reporting

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Governance, Security, and Compliance in Power Platform - Manuela Pichler'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-gov-audit-logging-and-activity-reporting workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-gov-audit-logging-and-activity-reporting processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Governance, Security, and Compliance in Power Platform - Manuela Pichler*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-gov-audit-logging-and-activity-reporting protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-gov-audit-logging-and-activity-reporting directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-gov-audit-logging-and-activity-reporting execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("audit-logging-reporting", "sentinel-power-platform-logs", "tenant-activity-reporting") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
