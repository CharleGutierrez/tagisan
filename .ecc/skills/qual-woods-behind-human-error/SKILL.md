---
name: qual-woods-behind-human-error
description: "Human error as symptom: Brittle automation, clumsy systems, conflicting goals, cognitive vulnerability, and the demands of operational complexity."
triggers: ["david-woods", "sidney-dekker", "richard-cook", "behind-human-error", "clumsy-automation", "blameless-culture", "systemic-vulnerability"]
---

# qual-woods-behind-human-error
> Based on **Behind Human Error - David D. Woods, Sidney Dekker, Richard Cook, Leila Johannesen**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Treat operator errors as symptoms of deeper interface ambiguity, brittle automation, or contradictory organizational goals.**
2. **ALWAYS: Provide confirmation dialogs that display the exact blast radius and entity impact before executing irreversible operations.**
3. **NEVER: Blame 'human error' or 'operator negligence' in post-mortems or system documentation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Eliminate clumsy automation: design safety envelopes, blast-radius previews, and resilient recovery mechanisms that treat user errors as design failures.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Closing post-mortems with 'action item: retrain the operator'.**
- **Silent automated scripts that execute massive destructive actions without confirmation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-woods-behind-human-error"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
