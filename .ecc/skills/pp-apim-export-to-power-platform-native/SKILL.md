---
name: pp-apim-export-to-power-platform-native
description: "Exporting APIs directly from Azure API Management to Power Platform as first-class connectors."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-30"]
triggers: ["export-to-power-platform", "apim-to-power-platform", "one-click-connector-export"]
---

# pp-apim-export-to-power-platform-native

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-apim-export-to-power-platform-native workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-apim-export-to-power-platform-native processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-apim-export-to-power-platform-native protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-apim-export-to-power-platform-native directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-apim-export-to-power-platform-native execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("export-to-power-platform", "apim-to-power-platform", "one-click-connector-export") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
