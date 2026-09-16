---
name: copilot-adaptive-card-blast-radius-reports
description: "Designing high-density visual telemetry cards for code changes, blast radius, and test diffs."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Software Architecture: The Hard Parts - Neal Ford & Mark Richards"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["blast-radius-reports", "telemetry-adaptive-cards", "code-change-cards", "visual-diff-cards"]
---

# copilot-adaptive-card-blast-radius-reports
> Based on **Software Architecture: The Hard Parts - Neal Ford & Mark Richards** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Visual Telemetry: Render color-coded impact badges (Red: High, Yellow: Med, Green: Low) on architectural diffs.**
2. **Dense Layout: Use FactSet and ColumnSet elements to present multi-dimensional metrics concisely.**
3. **MANDATORY direct links to code repos and test execution logs.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-adaptive-card-blast-radius-reports.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-adaptive-card-blast-radius-reports.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Generate high-density Adaptive Card reports visualizing architectural blast radius, test results, and deployment metrics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-adaptive-card-blast-radius-reports.**
- **Unmonitored runtime execution without telemetry in copilot-adaptive-card-blast-radius-reports.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "blast-radius-reports"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
