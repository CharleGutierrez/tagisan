---
name: qual-boettcher-technically-wrong
description: "Ethical product design: Eliminating toxic tech blind spots, rigid identity models, algorithmic cruelty, and exclusionary form validation."
triggers: ["wachter-boettcher", "technically-wrong", "toxic-tech", "inclusive-design", "algorithmic-cruelty", "flexible-validation", "identity-modeling"]
---

# qual-boettcher-technically-wrong
> Based on **Technically Wrong: Sexist Apps, Biased Algorithms, and Other Threats of Toxic Tech - Sara Wachter-Boettcher**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Ensure form validation schemas accommodate flexible international names (UTF-8, single names, apostrophes, arbitrary lengths).**
2. **ALWAYS: Support inclusive, non-binary, and optional gender/identity attributes in demographic schemas.**
3. **NEVER: Trigger algorithmic cruelty by surfacing traumatic memories or anniversary alerts without sensitive opt-out controls.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design inclusive, compassionate data schemas and notification engines that respect international naming conventions and diverse human identities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Form validation rejecting names like 'O'Connor' or single-character names.**
- **Forcing binary gender selection on a product that doesn't need it.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-boettcher-technically-wrong"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
