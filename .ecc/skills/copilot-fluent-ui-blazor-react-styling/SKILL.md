---
name: copilot-fluent-ui-blazor-react-styling
description: "Fluent UI React v9 / Blazor design tokens, typography, and accessibility (WCAG 2.1 AA)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Fluent Design System: Principles and UI Engineering - Microsoft Design"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["fluent-ui-styling", "fluent-design-system", "fluent-ui-react-v9", "wcag-accessibility"]
---

# copilot-fluent-ui-blazor-react-styling
> Based on **Fluent Design System: Principles and UI Engineering - Microsoft Design** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Design Tokens: Utilize Fluent design tokens (`colorNeutralBackground1`, `fontSizeBase300`) for seamless theme adaptation.**
2. **Accessibility Invariant: Meet WCAG 2.1 AA standards: minimum 4.5:1 contrast ratios and full keyboard focus navigation.**
3. **ALWAYS support dark, light, and high-contrast theme modes.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-fluent-ui-blazor-react-styling.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-fluent-ui-blazor-react-styling actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build modern, accessible web task panes and dialogs conforming strictly to Microsoft Fluent Design System standards.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-fluent-ui-blazor-react-styling.**
- **Unmonitored runtime execution without telemetry in copilot-fluent-ui-blazor-react-styling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fluent-ui-styling"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
