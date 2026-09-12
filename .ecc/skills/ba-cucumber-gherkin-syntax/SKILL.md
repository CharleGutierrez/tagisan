---
name: ba-cucumber-gherkin-syntax
description: "Behavior-Driven Development & Gherkin AST Grammar: Given/When/Then, Scenario Outlines, declarative steps, and business-focused acceptance criteria."
triggers: ["cucumber-gherkin-syntax", "gherkin", "bdd", "cucumber", "given-when-then", "scenario-outline", "declarative-testing"]
---

# ba-cucumber-gherkin-syntax
> Based on **The Cucumber Book - Matt Wynne & Aslak Hellesøy**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Strict Gherkin Grammar: Given (context/setup) -> When (action/event) -> Then (observable outcome).**
2. **Declarative over Imperative: Never mention UI widgets in Gherkin; express business intent.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format scenarios in valid Gherkin syntax: Feature, Scenario Outline, Given, When, Then, Examples table. Never mention UI selectors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mixing setup and assertions inside When steps.**
- **Writing UI-driven imperative steps that break on simple CSS refactoring.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cucumber-gherkin-syntax"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
