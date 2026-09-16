---
name: pp-canvas-timer-animation-ergonomics
description: "Declarative micro-animations, slide-in navigation panels, timer loops, and non-blocking progressive disclosure transitions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["timer-animation-ergonomics", "slide-in-navigation-timer", "progressive-disclosure-canvas", "smooth-panel-animations"]
---

# pp-canvas-timer-animation-ergonomics

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Timer Calculus: Smooth animations interpolate a coordinate property `X = CurrentX + (TargetX - CurrentX) * (Timer.Value / Timer.Duration)`.
- Performance Invariant: ALWAYS disable Timer.Repeat unless an explicit recurring polling loop is architecturally mandated.
- NEVER run continuous high-frequency timers on low-power mobile devices due to battery drain.
- MANDATORY setting Timer.AutoReset = true and Timer.AutoStart = false when driven by user interactions.
- Source Reference: *Micro-Animations and State Timing in Power Apps - April Dunnam*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Implement smooth drawer opening: set Timer.Duration = 200, Start = varStartTimer. Bind drawer container `X` property to `-(Parent.Width * 0.8) + (Parent.Width * 0.8) * (TimerAnimation.Value / TimerAnimation.Duration)`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Using 5 separate running timers simultaneously for background calculations, spiking client CPU to 100%.
- Leaving timers running when screens are navigated away from.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("timer-animation-ergonomics", "slide-in-navigation-timer", "progressive-disclosure-canvas", "smooth-panel-animations") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
