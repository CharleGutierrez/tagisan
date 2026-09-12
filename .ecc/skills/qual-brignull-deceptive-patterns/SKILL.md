---
name: qual-brignull-deceptive-patterns
description: "Taxonomy of deceptive design: Roach Motel, Sneak into Basket, Confirmshaming, Hidden Costs, Forced Continuity, and Obstruction (Sludge)."
triggers: ["harry-brignull", "deceptive-patterns", "dark-patterns", "roach-motel", "confirmshaming", "sneak-into-basket", "forced-continuity", "click-to-cancel"]
---

# qual-brignull-deceptive-patterns
> Based on **Deceptive Patterns: Exposing the Tricks Tech Companies Use to Control You - Harry Brignull**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Enforce Click-to-Cancel equivalence: canceling a subscription or deleting an account must require <= the clicks and steps it took to subscribe.**
2. **ALWAYS: Display full pricing, fees, and renewal terms transparently upfront without hidden surprises at the final checkout step.**
3. **NEVER: Use confirmshaming ('No thanks, I hate saving money') or roach-motel obstacle courses.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Strictly ban all deceptive design patterns: provide 1-click subscription cancellations, transparent pricing, and zero confirmshaming text.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Requiring a phone call to cancel an account created online in 30 seconds.**
- **Confirmshaming opt-out buttons with manipulative copy.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-brignull-deceptive-patterns"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
