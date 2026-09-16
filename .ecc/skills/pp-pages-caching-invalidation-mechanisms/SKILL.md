---
name: pp-pages-caching-invalidation-mechanisms
description: "Portal cache invalidation mechanisms, Web API synchronization, and administrative cache clearing."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-18"]
triggers: ["caching-invalidation-mechanisms", "portal-cache-clear", "web-api-cache-sync"]
---

# pp-pages-caching-invalidation-mechanisms

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Modern Websites with Microsoft Power Pages - Nick Doelman'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pages-caching-invalidation-mechanisms workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pages-caching-invalidation-mechanisms processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Modern Websites with Microsoft Power Pages - Nick Doelman*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pages-caching-invalidation-mechanisms protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pages-caching-invalidation-mechanisms directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pages-caching-invalidation-mechanisms execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("caching-invalidation-mechanisms", "portal-cache-clear", "web-api-cache-sync") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
