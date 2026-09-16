---
name: pp-csknow-knowledge-refresh-cycle-cadence
description: "Automating document re-indexing schedules, handling deletions, and purging stale vector chunks."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-23"]
triggers: ["knowledge-refresh-cadence", "re-indexing-schedules", "stale-chunk-purging"]
---

# pp-csknow-knowledge-refresh-cycle-cadence

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Knowledge Grounding with Microsoft Copilot Studio - J. Peter Bruzzese'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-csknow-knowledge-refresh-cycle-cadence workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-csknow-knowledge-refresh-cycle-cadence processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Knowledge Grounding with Microsoft Copilot Studio - J. Peter Bruzzese*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-csknow-knowledge-refresh-cycle-cadence protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-csknow-knowledge-refresh-cycle-cadence directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-csknow-knowledge-refresh-cycle-cadence execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("knowledge-refresh-cadence", "re-indexing-schedules", "stale-chunk-purging") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
