---
name: pp-canvas-accessible-aria-theming
description: "WCAG 2.1 AA compliance, TabIndex ordering, ScreenReaderLabel semantics, and accessible 4.5:1 color contrast theming."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["accessible-aria-theming", "wcag-power-apps", "tabindex-screenreader", "accessible-color-contrast"]
---

# pp-canvas-accessible-aria-theming

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Accessibility Protocol: Every interactive control requires TabIndex = 0 (or positive sequential index) and a descriptive ScreenReaderLabel.
- Contrast Ratio Invariant: ALWAYS ensure normal text meets minimum 4.5:1 contrast against its background and large text meets 3:1.
- NEVER convey state or validation errors solely through color without secondary iconography or textual hints.
- MANDATORY automated execution of the Power Apps built-in Accessibility Checker before solution export.
- Source Reference: *Accessible Power Platform Solutions - Microsoft Accessibility Engineering*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Audit all controls with the App Checker -> Accessibility tab. Resolve all missing accessible labels, invalid tab orders, and low-contrast warnings. Bind icon AccessibleLabel to action descriptions (e.g., 'Delete customer record').

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Setting TabIndex = -1 on actionable buttons, completely blinding keyboard and screen reader users.
- Using light gray text (#999999) on white background (#FFFFFF) failing WCAG AA contrast thresholds.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("accessible-aria-theming", "wcag-power-apps", "tabindex-screenreader", "accessible-color-contrast") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
