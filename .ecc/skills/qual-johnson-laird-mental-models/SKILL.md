---
name: qual-johnson-laird-mental-models
description: "Cognitive representation and analog reasoning: How humans construct internal dynamic mental models of system states, causality, and domain metaphors."
triggers: ["johnson-laird", "mental-models", "cognitive-representation", "causal-reasoning", "analog-models", "conceptual-metaphor"]
---

# qual-johnson-laird-mental-models
> Based on **Mental Models: Towards a Cognitive Science of Language, Inference, and Consciousness - Philip Johnson-Laird**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Map software state models (files, carts, channels, boards) 1:1 to users' pre-existing domain mental models and physical analogies.**
2. **ALWAYS: Maintain internal consistency in state transitions; identical actions must produce identical outcomes across the application.**
3. **NEVER: Expose mechanical backend leaks (raw foreign keys, cache invalidation delays, internal error codes) that violate the user's mental model.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Align application interactions with the user's intuitive conceptual model. Preserve spatial and metaphorical fidelity across all state transitions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using database table column names as UI form labels.**
- **Having 'Archive' mean 'permanently delete' in one tab and 'hide' in another.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-johnson-laird-mental-models"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
