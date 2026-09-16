---
name: pp-aib-table-extraction-multi-page-docs
description: "Extracting dynamic multi-row tables and repeating line items spanning multi-page documents."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-31"]
triggers: ["table-extraction-multi-page", "repeating-line-items-ai", "multi-page-table-parsing"]
---

# pp-aib-table-extraction-multi-page-docs

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Intelligent Automation with AI Builder and Power Platform - Joe Camp'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-aib-table-extraction-multi-page-docs workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-aib-table-extraction-multi-page-docs processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Intelligent Automation with AI Builder and Power Platform - Joe Camp*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-aib-table-extraction-multi-page-docs protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-aib-table-extraction-multi-page-docs directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-aib-table-extraction-multi-page-docs execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("table-extraction-multi-page", "repeating-line-items-ai", "multi-page-table-parsing") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
