---
name: ba-story-splitting-patterns
description: "User Story Splitting Heuristics: 10 vertical splitting patterns (workflow steps, business rules, happy/unhappy, interface variations, simple/complex)."
triggers: ["story-splitting-patterns", "story-splitting", "richard-lawrence", "peter-green", "vertical-slicing", "story-decomposition"]
---

# ba-story-splitting-patterns
> Based on **Patterns for Splitting User Stories - Richard Lawrence & Peter Green**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Vertical Slicing: When splitting a user story, every split sub-story must cut through all architectural layers (UI, Logic, Storage) and produce usable value.**
2. **Split Heuristic 1 (Operations): Split CRUD into individual user-driven capabilities (Create vs Search vs Update vs Archive).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Apply vertical story splitting patterns: Workflow steps, Business rule variations, Happy vs Exception paths, Simple vs Complex data variations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Horizontal slicing (e.g. 'Build backend DB' as Story 1, 'Build React UI' as Story 2).**
- **Splitting stories so small that individual stories deliver no customer value.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "story-splitting-patterns"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
