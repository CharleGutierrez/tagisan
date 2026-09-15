---
name: agentic-amodei-concrete-ai-safety
description: "Avoiding negative side effects, reward hacking mitigation, scalable oversight, safe exploration, and robustness to distributional shift."
triggers: ["amodei", "ai-safety", "reward-hacking", "negative-side-effects", "scalable-oversight", "safe-exploration", "distributional-shift"]
---

# agentic-amodei-concrete-ai-safety
> Based on **Concrete Problems in AI Safety - Dario Amodei et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Side Effect Invariant: Actions must not cause unintended destructive perturbations to external systems outside the primary objective.**
2. **Reward Hacking Defense: Ensure agent fitness cannot be maximized by trivial shortcuts (e.g. deleting failing tests).**
3. **Safe Exploration: Constraining exploratory actions to verified sandboxes where catastrophic damage is mathematically impossible.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Sandbox all agent filesystem and shell operations. Protect test files from unauthorized tampering and enforce least-privilege security policies.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Permitting agents to edit the very test suites verifying their correctness.**
- **Running unverified agent shell commands directly on host production machines.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "amodei"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
