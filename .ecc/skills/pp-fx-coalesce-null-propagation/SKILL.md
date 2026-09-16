---
name: pp-fx-coalesce-null-propagation
description: "Coalesce(), Blank(), IsBlank(), IsEmpty(), default value cascades, and graceful null handling across complex expressions."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["coalesce-null-propagation", "blank-vs-isblank-power-fx", "default-value-fallback-cascades", "null-coalescing-fx"]
---

# pp-fx-coalesce-null-propagation

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Coalesce Semantics: `Coalesce(Val1, Val2, ...)` evaluates arguments in order and returns the first non-blank/non-empty value.
- Distinction Invariant: `IsBlank()` checks for scalar nulls/empty strings; `IsEmpty()` checks for empty tables/collections. They are NOT interchangeable.
- NEVER permit raw unhandled Blanks to propagate into numerical multiplication or date formatting expressions.
- MANDATORY provision of sensible fallback values for nullable database fields.
- Source Reference: *Defensive Formula Design in Power Fx - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Display customer contact name with fallback cascade: `lblContact.Text = Coalesce(ThisItem.PreferredName, ThisItem.FullName, ThisItem.Email, 'Unknown Contact')`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Using `If(IsBlank(x), fallback, x)` chains instead of clean, concise `Coalesce(x, y, fallback)`.
- Calling `IsEmpty()` on a scalar text field, causing runtime evaluation errors.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("coalesce-null-propagation", "blank-vs-isblank-power-fx", "default-value-fallback-cascades", "null-coalescing-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
