---
name: ba-dmn-decision-table-completeness
description: "Decision Model and Notation (DMN 1.3/1.4): Decision table Hit Policies (Unique, First, Priority, Any, Collect), completeness checking, and non-overlapping input domains."
triggers: ["dmn-decision-table-completeness", "dmn", "decision-table", "hit-policy", "bruce-silver-dmn", "completeness-checking"]
---

# ba-dmn-decision-table-completeness
> Based on **DMN Method and Style (2nd Edition) - Bruce Silver**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Completeness & Non-Overlap (Hit Policy U): For every possible combination of inputs, exactly one rule evaluates to true.**
2. **Catch-all Fallback: Every decision table must have an explicit default rule to handle edge-case inputs safely.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure complex multi-condition logic into DMN Decision Tables: Hit Policy (U/F/C), Input Clauses, Output Clauses, Rule Rows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overlapping rule conditions in Hit Policy Unique tables.**
- **Leaving numerical boundary gaps where inputs match zero rules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "dmn-decision-table-completeness"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
