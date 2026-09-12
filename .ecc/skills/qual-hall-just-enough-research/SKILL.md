---
name: qual-hall-just-enough-research
description: "Pragmatic design research: Risk reduction, answering the right questions, organizational research, competitive analysis, evaluative vs generative inquiry."
triggers: ["erika-hall", "just-enough-research", "pragmatic-research", "risk-reduction", "generative-research", "evaluative-research", "assumption-testing"]
---

# qual-hall-just-enough-research
> Based on **Just Enough Research - Erika Hall**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Identify and rank the riskiest behavioral assumptions before generating high-fidelity code.**
2. **ALWAYS: Validate assumptions using the lowest-fidelity disposable prototype possible (sketches, paper mocks, simple CLI).**
3. **NEVER: Write thousands of lines of full-stack code to answer a question that could be resolved with a 15-minute user interview.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
De-risk software development by testing critical user assumptions with minimal, lightweight prototypes prior to full architectural implementation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building an entire complex microservice architecture before validating if any user wants the core feature.**
- **Confusing code velocity with user value.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-hall-just-enough-research"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
