---
name: pp-offline-device-hardware-sensors
description: "Hardware sensor integration: Location.Latitude, Compass, Acceleration, and BarcodeScanner."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-4"]
triggers: ["device-hardware-sensors", "location-gps-power-apps", "hardware-barcodescanner"]
---

# pp-offline-device-hardware-sensors

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Field-Ready Offline Mobile Power Apps - Microsoft Docs'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-offline-device-hardware-sensors workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-offline-device-hardware-sensors processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Field-Ready Offline Mobile Power Apps - Microsoft Docs*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-offline-device-hardware-sensors protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-offline-device-hardware-sensors directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-offline-device-hardware-sensors execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("device-hardware-sensors", "location-gps-power-apps", "hardware-barcodescanner") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
