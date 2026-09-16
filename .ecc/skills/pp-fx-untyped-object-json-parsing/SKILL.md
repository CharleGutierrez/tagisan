---
name: pp-fx-untyped-object-json-parsing
description: "ParseJSON(), UntypedObject manipulation, Value(), Text(), Boolean(), and Table() casting from external REST responses."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["untyped-object-json-parsing", "parsejson-power-fx", "untypedobject-casting", "json-deserialization-fx"]
---

# pp-fx-untyped-object-json-parsing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Untyped Object Boundary: `ParseJSON(jsonString)` returns an `UntypedObject` requiring explicit type coercion before consumption.
- Coercion Invariant: ALWAYS wrap UntypedObject fields with explicit type functions: `Text(item.name)`, `Value(item.price)`, `Boolean(item.active)`.
- NEVER pass raw UntypedObject references directly to database Patch() statements without casting.
- MANDATORY use of `Table(untypedArray)` for iterating over JSON arrays in galleries or ForAll loops.
- Source Reference: *Working with JSON and Untyped Objects in Power Fx - Microsoft Docs*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Parse API responses safely: `Set(varData, ParseJSON(apiResponse)); ClearCollect(colItems, ForAll(Table(varData.items), { Title: Text(ThisRecord.Value.title), Count: Value(ThisRecord.Value.count) }))`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Directly binding a Label.Text property to `ParseJSON(payload).title` without calling `Text()`, causing compilation errors.
- Failing to check for null JSON values before type casting, triggering runtime exceptions.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("untyped-object-json-parsing", "parsejson-power-fx", "untypedobject-casting", "json-deserialization-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
