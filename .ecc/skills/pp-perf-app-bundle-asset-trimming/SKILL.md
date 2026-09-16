---
name: pp-perf-app-bundle-asset-trimming
description: "Image compression, SVG substitution, and unreferenced media pruning for small bundle footprints."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-5"]
triggers: ["app-bundle-trimming", "canvas-asset-compression", "unreferenced-media-pruning"]
---

# pp-perf-app-bundle-asset-trimming

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-perf-app-bundle-asset-trimming workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-perf-app-bundle-asset-trimming processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Ultra-Fast Power Apps: Delegation, Caching & Profiling - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-perf-app-bundle-asset-trimming protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-perf-app-bundle-asset-trimming directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-perf-app-bundle-asset-trimming execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("app-bundle-trimming", "canvas-asset-compression", "unreferenced-media-pruning") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
