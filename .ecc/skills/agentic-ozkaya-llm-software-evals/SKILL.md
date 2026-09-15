---
name: agentic-ozkaya-llm-software-evals
description: "SWE-bench decomposition, pass@k metrics, patch verification, test suite execution in sandboxes, and regression prevention in agentic software engineering."
triggers: ["ozkaya", "swe-bench", "software-evals", "pass-at-k", "patch-verification", "regression-prevention", "sandbox-execution"]
---

# agentic-ozkaya-llm-software-evals
> Based on **LLMs in Software Engineering: Evaluation & Verification - Ipek Ozkaya**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pass@k Metric: Probability that at least one of k generated code samples passes all unit tests: pass@k = E[1 - comb(n - c, k) / comb(n, k)].**
2. **Isolated Sandbox Verification: Compiling and executing generated patches inside ephemeral Docker or WebAssembly containers.**
3. **Regression Invariant: A patch is valid if and only if it makes previously failing tests pass without breaking any existing passing tests.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Evaluate agent-generated git patches inside isolated worktrees. Run full regression test suites before presenting solutions to the developer.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Accepting patches that resolve a local bug but introduce silent regressions elsewhere.**
- **Executing generated code directly on the host development machine without sandboxing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ozkaya"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
