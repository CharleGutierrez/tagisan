---
name: ba-bdd-three-amigos-workshop
description: "Collaborative requirement discovery & Example Mapping: Product, Developer, and Tester alignment using Story, Rule, Example, and Question matrices."
triggers: ["bdd-three-amigos-workshop", "three-amigos", "example-mapping", "dan-north", "liz-keogh", "discovery-cards"]
---

# ba-bdd-three-amigos-workshop
> Based on **Example Mapping - Dan North, Liz Keogh, George Dinwiddie**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Three Amigos Alignment: A story cannot start development without explicit consensus across Business (Why/What), Engineering (How), and QA (What could go wrong).**
2. **Example Mapping Heuristic: If a story has > 3 unresolved Red Questions or > 5 Blue Rules, split the story before coding.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Decompose stories into 4 elements: Yellow Story, Blue Rules (business logic), Green Examples (concrete truth vectors), Red Questions (unresolved ambiguities).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting implementation while Red Questions remain unresolved.**
- **Writing stories without engineering feasibility or QA boundary input.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bdd-three-amigos-workshop"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
