---
name: pp-canvas-multi-form-wizard-state
description: "Step-by-step onboarding wizards, progressive form validation guards, draft auto-saving, and unified submission transactions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["multi-form-wizard-state", "step-by-step-onboarding", "progressive-form-validation", "draft-auto-save-canvas"]
---

# pp-canvas-multi-form-wizard-state

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- State Machine: Wizard steps are tracked via integer variable `varCurrentStep` (1..N). Navigation buttons guard transition with `varStepIsValid` predicates.
- Data Integrity Invariant: ALWAYS validate current step fields before incrementing step pointer.
- NEVER submit partial records to production Dataverse tables without draft status flagging.
- MANDATORY unified Patch transaction or multi-step Patch on final wizard confirmation.
- Source Reference: *Complex Multi-Step Form Workflows in Canvas Apps - Brian Dang*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Maintain step state via `UpdateContext({ varStep: varStep + 1 })`. Guard next button: `DisplayMode: If(And(!IsBlank(txtEmail.Text), IsMatch(txtEmail.Text, Match.Email)), DisplayMode.Edit, DisplayMode.Disabled)`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Allowing users to skip mandatory validation steps by directly clicking subsequent wizard tabs without gating.
- Committing irreversible database mutations on step 2 of a 5-step wizard.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("multi-form-wizard-state", "step-by-step-onboarding", "progressive-form-validation", "draft-auto-save-canvas") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
