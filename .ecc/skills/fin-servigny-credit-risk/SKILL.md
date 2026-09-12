---
name: fin-servigny-credit-risk
description: "Institutional credit risk: Probability of Default (PD), Loss Given Default (LGD), Exposure at Default (EAD), structural default models (Merton), and rating migrations."
triggers: ["arnaud-servigny", "credit-risk", "merton-model", "probability-of-default", "lgd", "ead", "expected-loss", "credit-migration"]
---

# fin-servigny-credit-risk
> Based on **Standard & Poor's Guide to Measuring and Managing Credit Risk - Arnaud de Servigny & Olivier Renault**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate Expected Credit Loss explicitly as: Expected Loss = PD * LGD * EAD.**
2. **ALWAYS: Model corporate credit default probabilities using structural models (Merton model distance-to-default) and credit transition matrices.**
3. **NEVER: Treat credit risk as a binary static variable; model continuous rating migration probabilities.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build credit risk engines using the structural Merton model: calculate distance-to-default and expected loss (EL = PD * LGD * EAD).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming zero loss on investment-grade counterparties.**
- **Ignoring exposure-at-default expansion on committed credit lines.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-servigny-credit-risk"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
