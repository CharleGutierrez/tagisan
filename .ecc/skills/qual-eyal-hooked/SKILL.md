---
name: qual-eyal-hooked
description: "The Hook Framework: Trigger (external/internal), Action (in anticipation of reward), Variable Reward (tribe, hunt, self), and Investment (loading the next trigger)."
triggers: ["nir-eyal", "hooked", "hook-model", "habit-forming", "variable-reward", "user-investment", "stored-value"]
---

# qual-eyal-hooked
> Based on **Hooked: How to Build Habit-Forming Products - Nir Eyal**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Ensure every user session concludes with an Investment step (saved preference, created artifact, accumulated data) that increases future system value.**
2. **ALWAYS: Transition user reliance from external prompts (push notifications, emails) to internal emotional triggers (competence, clarity).**
3. **NEVER: Deploy variable reward schedules for slot-machine manipulation that does not deliver genuine user utility.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure user interactions to store tangible value with every session, creating positive intrinsic reinforcement loops without manipulative dark patterns.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Endless scrolling feeds with zero stopping cues or stored value.**
- **Sending spammy generic notification triggers with zero personalized contextual relevance.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-eyal-hooked"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
