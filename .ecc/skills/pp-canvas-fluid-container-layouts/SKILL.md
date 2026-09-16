---
name: pp-canvas-fluid-container-layouts
description: "Auto-layout, Horizontal and Vertical containers, flex-grow, min-width, and dynamic wrapping for enterprise multi-form factor apps."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["fluid-container-layouts", "canvas-responsive-layout", "horizontal-vertical-containers", "flex-grow-min-width"]
---

# pp-canvas-fluid-container-layouts

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Container Mechanics: Horizontal and Vertical layout containers eliminate absolute (X, Y) pixel positioning in favor of responsive CSS-flexbox style flow.
- Layout Invariant: ALWAYS set explicit MinWidth and MinHeight constraints on fluid child controls to prevent layout collapsing on mobile screens.
- NEVER disable container wrapping on multi-column input forms without providing explicit horizontal scroll boundaries.
- MANDATORY utilization of Fill portions (flex-grow) for dynamic width distribution across diverse viewport aspect ratios.
- Source Reference: *Designing Responsive Canvas Apps in Microsoft Power Apps - Matthew Devaney*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS construct responsive screen layouts using hierarchical Auto-layout containers. Bind child Width property to Parent.Width * FillPortion / TotalPortions when fine-grained proportional allocation is required.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Hardcoding fixed X and Y coordinates on child controls inside responsive canvas screens.
- Nesting more than 6 levels of containers, causing excessive layout recalculation latency.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("fluid-container-layouts", "canvas-responsive-layout", "horizontal-vertical-containers", "flex-grow-min-width") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
