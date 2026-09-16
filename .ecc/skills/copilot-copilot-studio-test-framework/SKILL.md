---
name: copilot-copilot-studio-test-framework
description: "Power Platform CLI / Copilot Studio test framework for automated topic conversation regression."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-studio-test-framework", "topic-regression-testing", "automated-conversation-tests", "pac-test-copilot"]
---

# copilot-copilot-studio-test-framework
> Based on **Application Lifecycle Management for Copilot Studio - Microsoft Power CAT** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Regression Suite: Execute pre-recorded multi-turn conversation scripts against Copilot Studio bots.**
2. **Intent Assertion: Assert that specific user utterances trigger expected topics and return correct variable values.**
3. **ALWAYS execute regression suite before publishing bot updates to production.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-copilot-studio-test-framework.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-copilot-studio-test-framework actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate conversation regression testing for Copilot Studio bots using Power Platform CLI and testing frameworks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-copilot-studio-test-framework.**
- **Unmonitored runtime execution without telemetry in copilot-copilot-studio-test-framework.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-studio-test-framework"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
