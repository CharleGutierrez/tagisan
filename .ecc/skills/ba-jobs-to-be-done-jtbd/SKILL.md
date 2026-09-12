---
name: ba-jobs-to-be-done-jtbd
description: "Jobs to Be Done (JTBD) Theory: Customer progress models, Job Statements, Outcome-Driven Innovation, and the Four Forces of Progress (Push, Pull, Habit, Anxiety)."
triggers: ["jobs-to-be-done-jtbd", "jtbd", "clayton-christensen", "anthony-ulwick", "job-story", "forces-of-progress", "outcome-driven-innovation"]
---

# ba-jobs-to-be-done-jtbd
> Based on **Competing Against Chance - Clayton M. Christensen & Anthony Ulwick**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Job Statement Grammar: 'When [struggling situation/context], I want to [motivation/progress], so I can [desired outcome/transformation].'**
2. **Forces of Progress Balance: A new solution is adopted only when (Push of current situation + Pull of new solution) > (Habit of current solution + Anxiety of new solution).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format feature specifications as Job Stories: When [context], I want to [action], so I can [expected outcome]. Address Push, Pull, Habit, and Anxiety.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Defining products around user demographics rather than the functional/emotional Job to be Done.**
- **Ignoring the friction of user Habits when introducing new software.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jobs-to-be-done-jtbd"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
