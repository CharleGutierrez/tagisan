---
name: pp-dv-elastic-tables-azure-cosmos-db
description: "Dataverse Elastic tables powered by Azure Cosmos DB for ultra-high throughput telemetry."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-6"]
triggers: ["elastic-tables-cosmos", "high-volume-telemetry-dv", "json-partition-keys"]
---

# pp-dv-elastic-tables-azure-cosmos-db

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Data Modeling for Microsoft Dataverse - Marc Gerner'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-dv-elastic-tables-azure-cosmos-db workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-dv-elastic-tables-azure-cosmos-db processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Data Modeling for Microsoft Dataverse - Marc Gerner*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-dv-elastic-tables-azure-cosmos-db protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-dv-elastic-tables-azure-cosmos-db directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-dv-elastic-tables-azure-cosmos-db execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("elastic-tables-cosmos", "high-volume-telemetry-dv", "json-partition-keys") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
