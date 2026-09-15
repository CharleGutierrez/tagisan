---
name: agentic-jurafsky-slp-language-models
description: "Autoregressive language modeling, perplexity, beam search decoding, temperature and top-p sampling, semantic parsing, and linguistic foundations."
triggers: ["jurafsky", "martin", "speech-language-processing", "beam-search", "nucleus-sampling", "semantic-parsing", "perplexity"]
---

# agentic-jurafsky-slp-language-models
> Based on **Speech and Language Processing (3rd ed) - Daniel Jurafsky & James H. Martin**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Autoregressive Probability Chain: P(w_1, ..., w_n) = prod_{i=1}^n P(w_i | w_1, ..., w_{i-1}).**
2. **Nucleus (Top-p) Sampling: Restricts generation candidate set to smallest subset V^(p) where sum_{w in V^(p)} P(w) >= p.**
3. **Perplexity (PPL): PPL(W) = exp(- 1/N * sum_{i=1}^N ln P(w_i | w_{<i})), measuring model uncertainty.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Set temperature to 0.0 for deterministic code generation and property verification. Tune top-p to 0.95 for exploratory architecture brainstorming.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **High temperature (> 0.7) during syntax-critical refactoring, inducing hallucinated import statements.**
- **Using greedy search when multi-candidate beam search is required for complex constraint satisfaction.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jurafsky"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
