---
name: pp-pcfhard-audio-recording-microphone
description: "Capturing audio via context.device.captureAudio and encoding voice notes to base64."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-27"]
triggers: ["audio-recording-microphone", "context-device-audio", "voice-note-encoding-pcf"]
---

# pp-pcfhard-audio-recording-microphone

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Advanced PCF: Native Hardware & Performance Optimization - Microsoft Docs'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pcfhard-audio-recording-microphone workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pcfhard-audio-recording-microphone processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Advanced PCF: Native Hardware & Performance Optimization - Microsoft Docs*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pcfhard-audio-recording-microphone protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pcfhard-audio-recording-microphone directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pcfhard-audio-recording-microphone execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("audio-recording-microphone", "context-device-audio", "voice-note-encoding-pcf") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
