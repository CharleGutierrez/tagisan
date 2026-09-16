---
name: pp-pbicop-narrative-visual-summaries
description: "Configuring Smart Narrative visuals and Copilot dynamic text summaries bound to active slicers."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-17"]
triggers: ["narrative-visual-summaries", "smart-narrative-copilot", "dynamic-slicer-summaries"]
---

# pp-pbicop-narrative-visual-summaries

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Generative AI and Copilot in Microsoft Power BI - Microsoft Learn'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pbicop-narrative-visual-summaries workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pbicop-narrative-visual-summaries processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Generative AI and Copilot in Microsoft Power BI - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pbicop-narrative-visual-summaries protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pbicop-narrative-visual-summaries directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pbicop-narrative-visual-summaries execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("narrative-visual-summaries", "smart-narrative-copilot", "dynamic-slicer-summaries") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
