---
name: pp-rpa-legacy-win32-terminal-emulation
description: "Automating legacy Win32 thick clients, SAP GUI scripting, and 3270/5250 mainframe green screens."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-13"]
triggers: ["legacy-win32-terminal", "sap-gui-scripting", "mainframe-3270-automation"]
---

# pp-rpa-legacy-win32-terminal-emulation

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Robotic Process Automation with Power Automate Desktop - Joe Unwin'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-rpa-legacy-win32-terminal-emulation workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-rpa-legacy-win32-terminal-emulation processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Robotic Process Automation with Power Automate Desktop - Joe Unwin*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-rpa-legacy-win32-terminal-emulation protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-rpa-legacy-win32-terminal-emulation directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-rpa-legacy-win32-terminal-emulation execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("legacy-win32-terminal", "sap-gui-scripting", "mainframe-3270-automation") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
