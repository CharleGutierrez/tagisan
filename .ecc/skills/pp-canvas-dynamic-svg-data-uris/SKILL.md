---
name: pp-canvas-dynamic-svg-data-uris
description: "In-line SVG rendering via Image controls, data URI encoding, dynamic dashboard KPI progress rings, gauges, and sparklines."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["dynamic-svg-data-uris", "svg-power-apps", "kpi-progress-rings", "sparkline-canvas-visuals"]
---

# pp-canvas-dynamic-svg-data-uris

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Data URI Scheme: SVGs are constructed via Power Fx string interpolation and encoded into `data:image/svg+xml;utf8,...` strings.
- Visual Invariant: ALWAYS URL-encode special characters (e.g., `#` as `%23`) inside SVG color hex values.
- NEVER exceed 64KB SVG payload size in image controls to prevent browser canvas rendering stalls.
- MANDATORY parametric viewBox attributes for seamless responsive vector scaling across device resolutions.
- Source Reference: *Advanced Data Visualization with SVGs in Power Apps - Kristine Kolodziejski*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Bind an Image control's `Image` property to: `"data:image/svg+xml;utf8," & EncodeUrl("<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><circle cx='50' cy='50' r='40' stroke='%230078d4' stroke-width='8' fill='none'/></svg>")`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Omitting EncodeUrl() or forgetting to escape `#` characters, resulting in broken blank image renders.
- Generating complex multi-megabyte vector illustrations dynamically in Power Fx.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("dynamic-svg-data-uris", "svg-power-apps", "kpi-progress-rings", "sparkline-canvas-visuals") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
