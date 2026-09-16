---
name: pp-pweb-file-attachments-azure-blob-storage
description: "Configuring Azure Blob Storage integration for large file attachments on portal forms."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-19"]
triggers: ["file-attachments-blob-storage", "azure-blob-portal-integration", "portal-attachment-storage"]
---

# pp-pweb-file-attachments-azure-blob-storage

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Secure Portal Development with Power Pages Web API - Colin Vermander'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pweb-file-attachments-azure-blob-storage workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pweb-file-attachments-azure-blob-storage processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Secure Portal Development with Power Pages Web API - Colin Vermander*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pweb-file-attachments-azure-blob-storage protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pweb-file-attachments-azure-blob-storage directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pweb-file-attachments-azure-blob-storage execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("file-attachments-blob-storage", "azure-blob-portal-integration", "portal-attachment-storage") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
