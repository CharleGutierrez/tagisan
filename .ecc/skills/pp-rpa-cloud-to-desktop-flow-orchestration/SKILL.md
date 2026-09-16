---
name: pp-rpa-cloud-to-desktop-flow-orchestration
description: "Triggering Desktop Flows from Cloud Flows, passing typed inputs, and receiving JSON outputs."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-13"]
triggers: ["cloud-to-desktop-flow", "rpa-hybrid-orchestration", "desktop-flow-parameters"]
---

# pp-rpa-cloud-to-desktop-flow-orchestration

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Robotic Process Automation with Power Automate Desktop - Joe Unwin'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-rpa-cloud-to-desktop-flow-orchestration workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-rpa-cloud-to-desktop-flow-orchestration processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Robotic Process Automation with Power Automate Desktop - Joe Unwin*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-rpa-cloud-to-desktop-flow-orchestration protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-rpa-cloud-to-desktop-flow-orchestration directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-rpa-cloud-to-desktop-flow-orchestration execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("cloud-to-desktop-flow", "rpa-hybrid-orchestration", "desktop-flow-parameters") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
