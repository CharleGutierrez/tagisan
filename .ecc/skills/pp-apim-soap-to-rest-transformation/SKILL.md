---
name: pp-apim-soap-to-rest-transformation
description: "Modernizing legacy SOAP XML enterprise services into clean REST JSON APIs for Power Apps."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-30"]
triggers: ["soap-to-rest-transformation", "legacy-soap-modernization", "soap-xml-to-rest-json"]
---

# pp-apim-soap-to-rest-transformation

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-apim-soap-to-rest-transformation workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-apim-soap-to-rest-transformation processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-apim-soap-to-rest-transformation protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-apim-soap-to-rest-transformation directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-apim-soap-to-rest-transformation execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("soap-to-rest-transformation", "legacy-soap-modernization", "soap-xml-to-rest-json") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
