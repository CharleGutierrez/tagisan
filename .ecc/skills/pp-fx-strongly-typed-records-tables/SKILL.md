---
name: pp-fx-strongly-typed-records-tables
description: "Schema enforcement, Table(), Record(), Type(), Match(), and schema preservation across complex data transformations."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["strongly-typed-records-tables", "power-fx-types", "table-record-constructors", "schema-preservation-fx"]
---

# pp-fx-strongly-typed-records-tables

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Type System: Power Fx is a strongly typed declarative language with strict static typing for Text, Number, Boolean, Date, Time, Record, and Table.
- Type Safety Invariant: ALWAYS match schema keys and value types precisely when constructing in-memory tables via `Table({ Col1: 'Val1' })`.
- NEVER mix heterogeneous value types (e.g., text and numbers) in the same column of a synthetic collection.
- MANDATORY explicit casting when converting between numbers and strings via `Value()` and `Text()`.
- Source Reference: *Pro Power Fx: Strongly Typed Functional Logic - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Construct strongly typed lookup tables: `ClearCollect(colStatusTypes, Table({ ID: 1, Name: 'Draft' }, { ID: 2, Name: 'Approved' }))`. Verify column types in the Collections viewer.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Relying on implicit type coercion in comparisons (`'10' = 10`), causing subtle evaluation failures across different locales.
- Constructing tables with misspelled property names in alternating rows, creating null columns.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("strongly-typed-records-tables", "power-fx-types", "table-record-constructors", "schema-preservation-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
