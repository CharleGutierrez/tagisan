---
name: qual-noble-algorithms-of-oppression
description: "Algorithmic bias and structural power: How commercial search and ranking algorithms encode, amplify, and normalize historical social biases."
triggers: ["safiya-noble", "algorithms-of-oppression", "algorithmic-bias", "search-bias", "structural-inequity", "representational-fairness", "fair-ranking"]
---

# qual-noble-algorithms-of-oppression
> Based on **Algorithms of Oppression: How Search Engines Reinforce Racism - Safiya Umoja Noble**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Audit search ranking algorithms and recommendation models for representational fairness and bias amplification.**
2. **ALWAYS: Provide transparent explanations of ranking criteria and allow users to override personalized algorithmic feeds.**
3. **NEVER: Assume automated ranking algorithms are neutral or objective simply because they are executed by machine code.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate fairness audits and transparent ranking explanations into all search and recommendation pipelines, actively preventing bias amplification.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying opaque ranking models that amplify historical discriminatory stereotypes.**
- **Hiding the criteria used to rank or filter user search results.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-noble-algorithms-of-oppression"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
