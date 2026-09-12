---
name: qual-charmaz-grounded-theory
description: "Constructivist Grounded Theory: Inductive coding, theoretical sampling, constant comparative method, and emergent schema category generation from empirical field data."
triggers: ["charmaz", "grounded-theory", "inductive-coding", "theoretical-sampling", "constant-comparative", "emergent-taxonomy", "verbatim-codes"]
---

# qual-charmaz-grounded-theory
> Based on **Constructing Grounded Theory - Kathy Charmaz**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Derive database enums, tags, and workflow categories inductively from verbatim user terminology and observed empirical field data.**
2. **ALWAYS: Test schema abstractions against raw qualitative field data through continuous comparative analysis.**
3. **NEVER: Impose rigid top-down data classifications that force users to miscategorize their real-world artifacts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ground entity definitions and taxonomy schemas directly in empirical user terminology and qualitative observations, avoiding synthetic abstractions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Hardcoding rigid status enums that don't match the client's actual lifecycle stages.**
- **Inventing developer jargon for domain concepts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-charmaz-grounded-theory"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
