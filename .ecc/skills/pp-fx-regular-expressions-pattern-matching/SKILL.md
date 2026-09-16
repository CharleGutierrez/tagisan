---
name: pp-fx-regular-expressions-pattern-matching
description: "IsMatch(), Match(), MatchAll(), regular expression token matching, email/phone/credit card validation, and string sanitization."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["regular-expressions-pattern-matching", "ismatch-power-fx", "regex-validation-canvas", "input-sanitization-fx"]
---

# pp-fx-regular-expressions-pattern-matching

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Regex Engine: `IsMatch(Text, Pattern)` validates string conformance against predefined tokens (`Match.Email`, `Match.PhoneNumber`) or custom PCRE regex strings.
- Sanitization Invariant: ALWAYS validate critical input fields on client before passing to connectors or database tables.
- NEVER trust raw user string input without sanitizing against injection vectors.
- MANDATORY real-time feedback indicator when input fails pattern match.
- Source Reference: *Data Validation and Pattern Matching in Power Apps - Rory Neary*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Validate enterprise employee ID format (2 uppercase letters followed by 6 digits): `IsMatch(txtEmpID.Text, "^[A-Z]{2}\d{6}$", MatchOptions.ContainsName)`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Relying on simple length checks instead of structural pattern verification for emails and postal codes.
- Permitting unbounded string submission that breaks downstream legacy ERP integration.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("regular-expressions-pattern-matching", "ismatch-power-fx", "regex-validation-canvas", "input-sanitization-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
