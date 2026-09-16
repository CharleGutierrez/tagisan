---
name: pp-canvas-modal-dialog-surfacing
description: "Declarative modal popup layers, full-screen background scrims, z-index stack isolation, and keyboard focus trapping."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["modal-dialog-surfacing", "popup-layers-canvas", "scrim-focus-trapping", "z-index-dialog-stack"]
---

# pp-canvas-modal-dialog-surfacing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Z-Index Scrim Architecture: Modal containers sit at maximum screen z-index with a semi-transparent RGBA background (0, 0, 0, 0.4) intercepting all click events.
- Safety Invariant: ALWAYS trap clicks on the backdrop container to prevent accidental background interaction.
- NEVER leave modal state variable unbound; use clear boolean flags (`varShowDeleteConfirm`).
- MANDATORY escape hatch via explicit close icon or cancellation button resetting the modal state.
- Source Reference: *Micro-Interactions and Dialog Design in Power Apps - Sancho Harker*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Implement modal overlays using a dedicated full-screen container (`X: 0, Y: 0, Width: Parent.Width, Height: Parent.Height, Visible: varShowModal`). Place dialog card inside with centered alignment and explicit dismiss handlers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Scattershot visibility formulas on 20 individual controls instead of grouping dialog controls in a single container.
- Permitting destructive background actions while an uncommitted confirmation dialog is visible.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("modal-dialog-surfacing", "popup-layers-canvas", "scrim-focus-trapping", "z-index-dialog-stack") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
