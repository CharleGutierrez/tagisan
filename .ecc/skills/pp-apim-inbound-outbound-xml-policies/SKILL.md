---
name: pp-apim-inbound-outbound-xml-policies
description: "Crafting APIM XML policies: rate-limit-by-key, ip-filter, rewrite-uri, and validate-jwt."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-30"]
triggers: ["inbound-outbound-xml-policies", "apim-policy-authoring", "rate-limiting-rewrite-uri"]
---

# pp-apim-inbound-outbound-xml-policies

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-apim-inbound-outbound-xml-policies workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-apim-inbound-outbound-xml-policies processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Integration with Azure API Management and Power Platform - Massimo Crippa*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-apim-inbound-outbound-xml-policies protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-apim-inbound-outbound-xml-policies directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-apim-inbound-outbound-xml-policies execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("inbound-outbound-xml-policies", "apim-policy-authoring", "rate-limiting-rewrite-uri") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
