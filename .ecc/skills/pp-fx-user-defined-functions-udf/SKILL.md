---
name: pp-fx-user-defined-functions-udf
description: "App.Formulas reusable helper functions, parameterized functional signatures, pure calculations, and cross-screen DRY principles."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["user-defined-functions-udf", "udf-power-fx", "app-formulas-functions", "reusable-formula-helpers"]
---

# pp-fx-user-defined-functions-udf

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- UDF Architecture: Defined in App.Formulas as pure functions with typed parameters and return types: `FunctionName(Param1: Type, ...): ReturnType = Expression;`.
- Purity Invariant: UDFs in App.Formulas ALWAYS remain pure functions without side effects (no Set, Collect, or Navigate).
- NEVER duplicate identical financial calculation or tax rate formulas across multiple buttons and screens.
- MANDATORY declaration of parameter types (e.g., `rate: Number, amount: Number`).
- Source Reference: *Reusable Logic with User Defined Functions in Power Fx - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Declare tax calculation helper: `CalculateTax(amount: Number, stateCode: Text): Number = amount * LookUp(TaxRates, State = stateCode, Rate);`. Call everywhere: `lblTax.Text = Text(CalculateTax(Value(txtAmount.Text), 'CA'), '$#,##0.00')`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Copy-pasting 30 lines of complex string formatting logic into 15 different label controls.
- Attempting to trigger navigation or variable updates inside a User Defined Function.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("user-defined-functions-udf", "udf-power-fx", "app-formulas-functions", "reusable-formula-helpers") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
