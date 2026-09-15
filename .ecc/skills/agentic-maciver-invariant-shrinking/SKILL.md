---
name: agentic-maciver-invariant-shrinking
description: "Automated minimal test case reduction (shrinking), stateful model-based testing, falsification search, and integration fuzzing."
triggers: ["maciver", "hypothesis", "test-shrinking", "minimal-reproduction", "stateful-testing", "falsification-search"]
---

# agentic-maciver-invariant-shrinking
> Based on **Hypothesis: Modern Property-Based Testing and Invariant Shrinking - David R. MacIver**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Minimal Counterexample Shrinking: When a test fails on complex input X, automatically shrink X down to the smallest minimal failing reproduction.**
2. **Stateful Model-Based Testing: Execute randomized sequences of state-machine actions comparing system state against an abstract reference model.**
3. **Deterministic Replay: Any shrunk counterexample must reproduce identically given its random seed.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
When debugging, have the agent automatically shrink failing test payloads to minimal 1-line reproductions before attempting code fixes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Dumping massive 1000-line failure logs on the developer without isolating the minimal failing input.**
- **Non-deterministic tests that cannot be reliably reproduced from a fixed seed.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "maciver"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
