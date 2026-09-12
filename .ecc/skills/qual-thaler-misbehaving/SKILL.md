---
name: qual-thaler-misbehaving
description: "Behavioral economic anomalies: Mental accounting, endowment effect, sunk cost fallacy, perceived fairness, and bounded self-control."
triggers: ["thaler-misbehaving", "mental-accounting", "endowment-effect", "sunk-cost-fallacy", "perceived-fairness", "bounded-rationality"]
---

# qual-thaler-misbehaving
> Based on **Misbehaving: The Making of Behavioral Economics - Richard H. Thaler**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Ensure account billing, downgrades, and refunds respect perceived fairness and provide immediate confirmation.**
2. **ALWAYS: Present financial data categorized according to users' mental accounts (e.g. Operating vs Capital, Personal vs Business).**
3. **NEVER: Leverage the sunk-cost fallacy to manipulate users into continuing unwanted subscriptions or completing bloated forms.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Align application accounting and state transitions with users' psychological mental accounts and transparent standards of transactional fairness.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Refusing prorated refunds on accidental annual renewals.**
- **Hiding the cancel button behind 5 pages of guilt-tripping 'look what you will lose' prompts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-thaler-misbehaving"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
