---
name: qual-duhigg-power-of-habit
description: "The neurological Habit Loop: Cue, Routine, Reward. The golden rule of habit change (substituting routines while maintaining cues and rewards), and keystone habits."
triggers: ["charles-duhigg", "power-of-habit", "habit-loop", "cue-routine-reward", "keystone-habits", "habit-transformation"]
---

# qual-duhigg-power-of-habit
> Based on **The Power of Habit: Why We Do What We Do in Life and Business - Charles Duhigg**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Attach new software routines to existing, deeply established user cues (e.g. 'when a pull request is approved', 'when a build finishes').**
2. **ALWAYS: Deliver immediate, tangible rewards (visual confirmation, automated time saved) upon routine completion.**
3. **NEVER: Expect users to adopt new behaviors without anchoring to an unambiguous environmental or system cue.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Anchor digital workflows to existing environmental cues and deliver immediate closure feedback to solidify productive organizational habits.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Expecting users to remember to manually trigger periodic maintenance scripts.**
- **Omitting success confirmation states when automated jobs finish.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-duhigg-power-of-habit"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
