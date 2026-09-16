---
name: pp-pcfhard-barcode-scanner-camera-capture
description: "Accessing context.device.captureBarcode and captureImage for mobile camera integration."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-27"]
triggers: ["barcode-scanner-camera-pcf", "context-device-barcode", "mobile-camera-capture"]
---

# pp-pcfhard-barcode-scanner-camera-capture

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Advanced PCF: Native Hardware & Performance Optimization - Microsoft Docs'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pcfhard-barcode-scanner-camera-capture workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pcfhard-barcode-scanner-camera-capture processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Advanced PCF: Native Hardware & Performance Optimization - Microsoft Docs*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pcfhard-barcode-scanner-camera-capture protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pcfhard-barcode-scanner-camera-capture directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pcfhard-barcode-scanner-camera-capture execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("barcode-scanner-camera-pcf", "context-device-barcode", "mobile-camera-capture") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
