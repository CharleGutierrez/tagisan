---
name: pp-rpa-desktop-flow-machine-groups
description: "Hosted machine groups, On-premises data gateway clusters, and dynamic load distribution."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-13"]
triggers: ["desktop-machine-groups", "hosted-rpa-bots", "gateway-cluster-balancing"]
---

# pp-rpa-desktop-flow-machine-groups

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Robotic Process Automation with Power Automate Desktop - Joe Unwin'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-rpa-desktop-flow-machine-groups workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-rpa-desktop-flow-machine-groups processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Robotic Process Automation with Power Automate Desktop - Joe Unwin*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-rpa-desktop-flow-machine-groups protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-rpa-desktop-flow-machine-groups directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-rpa-desktop-flow-machine-groups execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("desktop-machine-groups", "hosted-rpa-bots", "gateway-cluster-balancing") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
