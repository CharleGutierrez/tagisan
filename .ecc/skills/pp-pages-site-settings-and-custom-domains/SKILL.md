---
name: pp-pages-site-settings-and-custom-domains
description: "Site Settings configuration, custom domain binding, SSL certificates, and URL rewrite rules."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-18"]
triggers: ["site-settings-custom-domains", "ssl-certificates-power-pages", "url-rewrite-rules"]
---

# pp-pages-site-settings-and-custom-domains

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Modern Websites with Microsoft Power Pages - Nick Doelman'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pages-site-settings-and-custom-domains workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pages-site-settings-and-custom-domains processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Modern Websites with Microsoft Power Pages - Nick Doelman*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pages-site-settings-and-custom-domains protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pages-site-settings-and-custom-domains directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pages-site-settings-and-custom-domains execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("site-settings-custom-domains", "ssl-certificates-power-pages", "url-rewrite-rules") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
