---
name: pp-conn-openapi-swagger-v2-v3-schemas
description: "Authoring OpenAPI / Swagger specifications tailored for Power Automate and Power Apps."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-28"]
triggers: ["openapi-swagger-schemas", "swagger-v2-v3-connectors", "connector-contract-authoring"]
---

# pp-conn-openapi-swagger-v2-v3-schemas

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Custom Connectors for Power Platform - Troy Taylor'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-conn-openapi-swagger-v2-v3-schemas workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-conn-openapi-swagger-v2-v3-schemas processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Custom Connectors for Power Platform - Troy Taylor*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-conn-openapi-swagger-v2-v3-schemas protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-conn-openapi-swagger-v2-v3-schemas directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-conn-openapi-swagger-v2-v3-schemas execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("openapi-swagger-schemas", "swagger-v2-v3-connectors", "connector-contract-authoring") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
