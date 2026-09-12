---
name: ba-johnson-gui-bloopers-heuristics
description: "Cognitive Usability & GUI Blooper Prevention: Reducing cognitive load, Hick's Law, Fitts's Law, Nielsen's 10 heuristics, and error prevention."
triggers: ["johnson-gui-bloopers-heuristics", "gui-bloopers", "jeff-johnson", "usability-heuristics", "nielsen-heuristics", "cognitive-load", "hicks-law"]
---

# ba-johnson-gui-bloopers-heuristics
> Based on **GUI Bloopers 2.0 & Usability Heuristics - Jeff Johnson & Jakob Nielsen**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Hick's Law: Decision time increases logarithmically with the number and complexity of choices. Group and minimize choices.**
2. **Error Prevention: Design interfaces to make errors impossible (e.g. disabling invalid options, date pickers) rather than relying on error dialogs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit UI against Nielsen heuristics and Johnson bloopers: Eliminate cognitive clutter, minimize choice counts, and prevent errors through constraints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Presenting 30 unsorted options in a dropdown.**
- **Blaming the user with accusatory error messages when validation fails.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "johnson-gui-bloopers-heuristics"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
