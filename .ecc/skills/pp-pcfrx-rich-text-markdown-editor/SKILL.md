---
name: pp-pcfrx-rich-text-markdown-editor
description: "Integrating Slate / TipTap WYSIWYG editors with live Markdown export inside PCF controls."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-26"]
triggers: ["rich-text-markdown-editor", "tiptap-wysiwyg-pcf", "markdown-export-pcf"]
---

# pp-pcfrx-rich-text-markdown-editor

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Modern Front-End Engineering with PCF and Fluent UI - Dian Taylor'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pcfrx-rich-text-markdown-editor workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pcfrx-rich-text-markdown-editor processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Modern Front-End Engineering with PCF and Fluent UI - Dian Taylor*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pcfrx-rich-text-markdown-editor protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pcfrx-rich-text-markdown-editor directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pcfrx-rich-text-markdown-editor execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("rich-text-markdown-editor", "tiptap-wysiwyg-pcf", "markdown-export-pcf") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
