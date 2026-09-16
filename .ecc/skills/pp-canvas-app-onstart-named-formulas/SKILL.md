---
name: pp-canvas-app-onstart-named-formulas
description: "Transitioning from imperative App.OnStart to declarative App.Formulas (Named Formulas) for instantaneous application initialization."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["app-onstart-named-formulas", "app-formulas-power-fx", "instant-app-initialization", "named-formulas-migration"]
---

# pp-canvas-app-onstart-named-formulas

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Declarative Evaluation: App.Formulas are evaluated lazily on demand or concurrently at startup, eliminating sequential App.OnStart blocking bottlenecks.
- Startup Invariant: ALWAYS declare read-only lookup dictionaries, user profiles, and design theme palettes in App.Formulas.
- NEVER execute heavy network I/O or multi-record collection writes inside App.OnStart when App.Formulas can provide immutable computed records.
- MANDATORY zero-delay screen rendering on initial app launch.
- Source Reference: *High-Performance Canvas Apps - Microsoft Press & Tim Leung*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Replace imperative `Set(CurrentUser, User())` and `Set(AppTheme, ...)` in App.OnStart with immutable declarations in App.Formulas: `fxUser = User();` and `fxTheme = { Primary: ColorValue('#0078D4'), Background: ColorValue('#FFFFFF') };`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Writing 500 lines of sequential Set() and ClearCollect() calls in App.OnStart, leading to 10+ second initial splash freezes.
- Modifying global variables in App.OnStart that depend on screen controls not yet instantiated.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("app-onstart-named-formulas", "app-formulas-power-fx", "instant-app-initialization", "named-formulas-migration") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
