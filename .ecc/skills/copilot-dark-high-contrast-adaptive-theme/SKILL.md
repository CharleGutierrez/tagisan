---
name: copilot-dark-high-contrast-adaptive-theme
description: "Adapting cards and UI components to Teams default, dark, and high-contrast themes."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Fluent Design System: Principles and UI Engineering - Microsoft Design"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["dark-theme-cards", "high-contrast-adaptation", "adaptive-color-styling", "teams-theming"]
---

# copilot-dark-high-contrast-adaptive-theme
> Based on **Fluent Design System: Principles and UI Engineering - Microsoft Design** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Adaptive Styling: Use semantic card colors (`default`, `accent`, `good`, `warning`, `attention`) instead of hardcoded hex codes.**
2. **Theme Detection: Read client theme from context and apply corresponding Fluent UI theme provider.**
3. **ALWAYS verify legibility in Teams High Contrast mode.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-dark-high-contrast-adaptive-theme.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-dark-high-contrast-adaptive-theme actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ensure all Copilot UI components and cards adapt dynamically to Teams default, dark, and high-contrast themes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-dark-high-contrast-adaptive-theme.**
- **Unmonitored runtime execution without telemetry in copilot-dark-high-contrast-adaptive-theme.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "dark-theme-cards"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
