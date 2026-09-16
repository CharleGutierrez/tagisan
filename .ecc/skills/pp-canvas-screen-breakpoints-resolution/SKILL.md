---
name: pp-canvas-screen-breakpoints-resolution
description: "Viewport breakpoint detection using App.Width, App.Height, screen orientation matrices, and adaptive control density."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["canvas-screen-breakpoints", "viewport-orientation", "adaptive-control-density", "app-width-height-breakpoints"]
---

# pp-canvas-screen-breakpoints-resolution

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Breakpoint Matrix: 1 = Mobile Portrait (<600px), 2 = Mobile Landscape/Tablet Portrait (600-900px), 3 = Tablet Landscape/Desktop Small (900-1200px), 4 = Desktop Large (>1200px).
- Resolution Invariant: ALWAYS read viewport metrics from App.ActiveScreen.Width and App.ActiveScreen.Height rather than static parent bounds.
- NEVER force landscape lock in enterprise field worker apps where single-hand portrait operation is ergonomically required.
- MANDATORY adaptation of gallery column counts: 1 column on breakpoint 1, 2 on breakpoint 2, 3-4 on breakpoints 3-4.
- Source Reference: *Power Apps Canvas Design Patterns - Paul Culmsee & Reza Dorrani*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Define global screen breakpoint formulas in App.Formulas: `ScreenSize = Switch(true, App.ActiveScreen.Width < 600, 1, App.ActiveScreen.Width < 900, 2, App.ActiveScreen.Width < 1200, 3, 4)`. Use `ScreenSize` across all responsive visibility properties.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Duplicating screens for mobile vs desktop versions instead of utilizing responsive auto-layout containers.
- Assuming window size never changes during an active browser session.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("canvas-screen-breakpoints", "viewport-orientation", "adaptive-control-density", "app-width-height-breakpoints") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
