---
name: qual-nielsen-usability-engineering
description: "Canonical usability engineering: Nielsen's 10 Usability Heuristics, heuristic evaluation methods, discount usability testing, and severity ratings."
triggers: ["jakob-nielsen", "usability-engineering", "nielsen-heuristics", "heuristic-evaluation", "severity-ratings", "discount-usability", "error-prevention"]
---

# qual-nielsen-usability-engineering
> Based on **Usability Engineering - Jakob Nielsen**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Enforce visibility of system status with continuous progress indicators updating in < 100ms.**
2. **ALWAYS: Provide explicit Emergency Exits: universal `Escape`, `Cancel`, and `Undo` support across all state changes.**
3. **NEVER: Display unformatted system exceptions or cryptic error codes without human-readable recovery guidance.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Comply strictly with Nielsen's 10 Usability Heuristics: maintain continuous status visibility, support universal undo, and provide actionable error recovery.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Modal dialogs with no cancel or close button.**
- **Error popups displaying raw database stack traces to end users.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-nielsen-usability-engineering"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
