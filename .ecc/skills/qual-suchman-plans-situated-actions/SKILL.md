---
name: qual-suchman-plans-situated-actions
description: "Situated action theory: Human action is inherently situated and contingent on immediate circumstances; abstract plans break down in complex real-world interaction."
triggers: ["lucy-suchman", "plans-and-situated-actions", "situated-action", "contingent-behavior", "breakdown-repair", "human-machine-communication"]
---

# qual-suchman-plans-situated-actions
> Based on **Plans and Situated Actions: The Problem of Human-Machine Communication - Lucy Suchman**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Design multi-step workflows to be fluidly adaptable to unexpected real-time changes, supporting out-of-order execution and flexible state branching.**
2. **ALWAYS: Provide robust, immediate breakdown-and-repair mechanisms when user actions diverge from the happy path.**
3. **NEVER: Trap users in rigid linear wizards that prohibit backward navigation or mid-stream situational pivots.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enable flexible, situated workflows: allow users to jump between sections, edit out of sequence, and adapt to changing real-time conditions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Locking wizard steps so users cannot navigate back to edit step 1 after reaching step 3.**
- **Failing when user performs an unanticipated action.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-suchman-plans-situated-actions"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
