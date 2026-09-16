---
name: pp-offline-nfc-tag-reading-rfid
description: "ReadNFC integration, payload decoding, and mobile asset tracking field workflows."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-4"]
triggers: ["nfc-tag-reading", "readnfc-power-apps", "rfid-asset-tracking"]
---

# pp-offline-nfc-tag-reading-rfid

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Field-Ready Offline Mobile Power Apps - Microsoft Docs'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-offline-nfc-tag-reading-rfid workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-offline-nfc-tag-reading-rfid processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Field-Ready Offline Mobile Power Apps - Microsoft Docs*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-offline-nfc-tag-reading-rfid protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-offline-nfc-tag-reading-rfid directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-offline-nfc-tag-reading-rfid execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("nfc-tag-reading", "readnfc-power-apps", "rfid-asset-tracking") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
