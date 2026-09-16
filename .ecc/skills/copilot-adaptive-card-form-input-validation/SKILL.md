---
name: copilot-adaptive-card-form-input-validation
description: "Input validation: Input.Text, Input.ChoiceSet, regex patterns, and client-side error labels."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["card-input-validation", "adaptive-card-forms", "input-choiceset", "client-side-validation"]
---

# copilot-adaptive-card-form-input-validation
> Based on **Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Client-Side Validation: Define `isRequired: true`, `regex: "..."`, and `errorMessage: "..."` on input elements.**
2. **Server-Side Re-validation: ALWAYS re-validate all submitted form fields on the backend server before processing.**
3. **NEVER trust raw client card submissions without sanitization.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-adaptive-card-form-input-validation actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design bulletproof data entry forms in Adaptive Cards with instant client-side validation and backend verification.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-adaptive-card-form-input-validation.**
- **Unmonitored runtime execution without telemetry in copilot-adaptive-card-form-input-validation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "card-input-validation"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
