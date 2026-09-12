---
name: qual-wiener-cybernetics
description: "Cybernetics & systems control: Circular causal feedback loops, negative feedback for homeostasis, information entropy, and human-machine symbiotic control."
triggers: ["norbert-wiener", "cybernetics", "feedback-loops", "circular-causality", "negative-feedback", "homeostasis", "closed-loop-systems"]
---

# qual-wiener-cybernetics
> Based on **Cybernetics: Or Control and Communication in the Animal and the Machine - Norbert Wiener**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Close feedback loops immediately: provide visible, continuous sensory feedback on the effects of user actions, enabling real-time error correction.**
2. **ALWAYS: Design asynchronous systems to broadcast live progress percentages and bidirectional error-correction channels.**
3. **NEVER: Leave users in open-loop states (e.g. silent background jobs with zero status or feedback).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain closed-loop cybernetic feedback across all interactive operations: deliver real-time state broadcasts and instant error-correction paths.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Triggering long-running asynchronous operations with no progress bar or cancellation option.**
- **Silent failures with zero user feedback.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-wiener-cybernetics"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
