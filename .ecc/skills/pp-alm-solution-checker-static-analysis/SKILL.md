---
name: pp-alm-solution-checker-static-analysis
description: "Running pac solution check for automated static analysis, security rules, and performance warnings."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-34"]
triggers: ["solution-checker-static-analysis", "pac-solution-check-cli", "static-analysis-rules-pp"]
---

# pp-alm-solution-checker-static-analysis

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise ALM with Microsoft Power Platform CLI - Wael Hamze'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-alm-solution-checker-static-analysis workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-alm-solution-checker-static-analysis processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise ALM with Microsoft Power Platform CLI - Wael Hamze*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-alm-solution-checker-static-analysis protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-alm-solution-checker-static-analysis directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-alm-solution-checker-static-analysis execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("solution-checker-static-analysis", "pac-solution-check-cli", "static-analysis-rules-pp") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
