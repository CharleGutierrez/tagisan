---
name: frontier-ast-graph-navigator
description: Autonomous AST codebase graph navigation, transitive blast-radius impact analysis, adversarial dialectical critique (Lakandiwa synthesis), and closed-loop TDD invariant proving for Frontier Cloud LLMs (Claude 3.5 Sonnet, GPT-4o, Gemini 2.0 Pro, DeepSeek-V3/R1). Triggers: query_code_graph, calculate_blast_radius, grounded_inference, blast radius, ast graph, callers, callees, refactoring risk, dialectical critique, closed loop tdd, frontier cloud.
version: 1.0.0
tags:
  - frontier-llm
  - ast-codebase-graph
  - blast-radius
  - dialectical-critique
  - closed-loop-tdd
  - invariant-proving
  - lakandiwa-synthesis
compatibility: ">=0.2.0"
providers:
  - anthropic
  - openai
  - gemini
  - deepseek
---

# Frontier AST Graph Navigator & Transitive Blast-Radius Engine

## Purpose & Architectural Mandate
Frontier Cloud LLMs (Claude 3.5 Sonnet, GPT-4o, Gemini 2.0 Pro, DeepSeek-V3/R1) exhibit massive reasoning capabilities and vast context windows (128k - 2M tokens). However, in large production codebases, unconstrained frontier models suffer from four critical pathologies:
1. **Context Bloat & Needle-in-a-Haystack Degradation**: Blindly dumping dozens of source files into context dilutes attention, inflates latency, explodes token costs, and induces plausible hallucinations.
2. **Untracked Blast Radius**: Refactoring central structs, traits, or public API signatures without traversing transitive dependency graphs leads to broken downstream consumers.
3. **Optimistic Bias & Hallucinated Correctness**: Generating code without rigorous adversarial red-teaming leaves subtle concurrency races, memory leaks, and boundary exceptions undetected.
4. **Open-Loop Speculation**: Emitting code without compiler and sandbox verification violates basic engineering rigor.

This skill operationalizes the **4 Pillars of Frontier Model Transformation**, turning frontier models from ordinary, context-bloated generators into outstanding, surgically precise software architects.

---

## Pillar 1: AST Codebase Knowledge Graph & Transitive Blast-Radius Navigation

### Invariant 1.1: Zero Blind File Dumping
- **FORBIDDEN**: Never dump 10+ whole source files into context or request full project cat/dump.
- **MANDATORY**: Always query the codebase knowledge graph first using targeted tools.
  ```json
  {
    "tool": "query_code_graph",
    "arguments": {
      "query": "SymbolName",
      "direction": "definition"
    }
  }
  ```
- Identify exact file locations, line numbers, and symbol signatures before reading any code.

### Invariant 1.2: Surgical Dependency Traversal
Before refactoring, modifying, or deleting any function, struct, trait, or method:
1. **Query Callers**: Inspect all incoming call and reference sites.
   ```json
   {
     "tool": "query_code_graph",
     "arguments": {
       "query": "execute",
       "direction": "callers"
     }
   }
   ```
2. **Query Callees**: Inspect outgoing calls to understand downstream dependencies.
   ```json
   {
     "tool": "query_code_graph",
     "arguments": {
       "query": "execute",
       "direction": "callees"
     }
   }
   ```

### Invariant 1.3: Mandatory Blast Radius Assessment
Before altering any shared type, trait method, or public API signature, you MUST run `calculate_blast_radius`:
```json
{
  "tool": "calculate_blast_radius",
  "arguments": {
    "target": "TargetSymbolOrType",
    "max_depth": 3
  }
}
```

### Invariant 1.4: Strict Risk-Tier Protocol
The calculated blast radius dictates the mandatory operational action:
- 🟢 **LOW RISK (0–2 Dependents)**:
  - Localized refactoring.
  - Inspect the target file and direct call sites.
  - Run the local unit test suite covering the immediate module.
- 🟡 **MEDIUM RISK (3–5 Dependents)**:
  - Multi-call-site verification across all affected files.
  - Update all parameter signatures and call-site invocations simultaneously.
  - Run integration tests for parent modules.
- 🟠 **HIGH RISK (6–12 Dependents)**:
  - Wide ripple effect across multiple subsystems.
  - Execute a comprehensive regression run across all affected modules.
  - Provide an explicit impact breakdown in the PR/task summary before proceeding.
- 🔴 **CRITICAL RISK (13+ Dependents or Core Interface Hubs)**:
  - Structural architectural hub.
  - **BREAKING CHANGES ARE FORBIDDEN**.
  - Must author a backward-compatible adapter, shim, or phased deprecation path.
  - Require full workspace compile check (`cargo check --all-targets` or equivalent) and exhaustive integration proving.

---

## Pillar 2: Adversarial Dialectical Critique (Lakandiwa Synthesis)

Frontier models must not accept their own first drafts. Every non-trivial architectural change, synchronization primitive, or algorithmic implementation must pass through dialectical red-teaming:

```
    [ THESIS ]          --> Initial implementation proposal / algorithm
        │
        ▼
   [ ANTITHESIS ]       --> Adversarial Red-Team Audit (Race conditions, leaks, edge bounds)
        │
        ▼
  [ SYNTHESIS ]         --> Lakandiwa Synthesis (Hardened, battle-tested production code)
```

### Operational Dialectical Workflow:
1. **Thesis (Proposal)**:
   - Formulate the clean, idiomatic solution meeting the functional requirements.
2. **Antithesis (Adversarial Critique)**:
   - Act as a hostile, adversarial auditor. Specifically evaluate:
     - **Concurrency & Races**: Deadlocks, lock contention, ABA problems, lost wakeups, atomic orderings (`SeqCst` vs `Relaxed`).
     - **Memory & Resource Lifetimes**: Leaks, unbounded buffers, dangling handles, circular references, OOM vulnerability.
     - **Boundary & Malformed Inputs**: Zero-length slices, integer overflow/underflow, NaN/Inf floats, unicode edge cases.
     - **Failure & Recovery**: Poisoned mutexes, network timeouts, partial write failures, error propagation hygiene.
3. **Lakandiwa Synthesis (Resolution)**:
   - Reject vulnerable constructs and integrate bulletproof mitigations.
   - Ground the synthesis in formal invariants and documented invariants.

Alternatively, leverage `grounded_inference` to automatically run multi-pass adversarial critique and verification loops.

---

## Pillar 3: Closed-Loop TDD & Formal Invariant Proving

### Invariant 3.1: Spec / Invariant First
- Define mathematical or behavioral invariants before authoring implementation details:
  - Preconditions ($P$)
  - Postconditions ($Q$)
  - State Invariants ($I$)
- Author test cases asserting these invariants *prior* to modifying production logic.

### Invariant 3.2: Deterministic Compilation & Execution Proving
- **NEVER** claim code works without execution feedback.
- Run the language compiler and test runner:
  - Rust: `cargo test`, `cargo check --tests`
  - TypeScript / JS: `bun test`, `tsc --noEmit`
  - Python: `python -m pytest` or `python_eval`
- Verify that:
  - Exit code is strictly `0`.
  - STDERR contains zero compiler warnings or runtime panics.
  - All invariant assertion statements succeed.

### Invariant 3.3: Self-Healing on Failure
When a test or compilation fails:
1. Deconstruct the compiler error or panic traceback to its exact file, line, and column.
2. Formulate a minimal, surgical patch addressing the root cause without side-effects.
3. Re-run verification until full green pass is achieved.

---

## Pillar 4: Hierarchical Episodic Memory & Dynamic Context Paging (JIT Skills)

### Invariant 4.1: Just-In-Time (JIT) Skill Activation
- Do not pollute prompts with dozens of irrelevant skill texts.
- Query skills dynamically using `search_skills` or trigger terms when stepping into unfamiliar domain territory (e.g., distributed consensus, finance math, compiler AST).

### Invariant 4.2: Episodic Memory Grounding
- Search long-term episodic memory (`search_memory`) for prior architectural decisions, known pitfalls, or project-specific idioms.
- Persist high-leverage architectural learnings and root-cause post-mortems using `save_memory`.

---

## Execution Protocol Quick Reference

| Action | Mandatory Tool / Step | Prohibited Behavior |
| :--- | :--- | :--- |
| **Locating Code** | `query_code_graph(query, "definition")` | Grepping entire tree or reading 20 files |
| **Checking Callers** | `query_code_graph(query, "callers")` | Guessing caller sites from memory |
| **Refactoring Hubs** | `calculate_blast_radius(target, 3)` | Editing public types without risk analysis |
| **Critical Changes** | Deprecation adapter + full test suite | Breaking 13+ dependents silently |
| **Code Verification**| Compile + Test execution (Exit Code 0) | Emitting uncompiled code blocks |
| **Complex Logic** | Thesis $\to$ Antithesis $\to$ Synthesis | Accepting first draft without critique |
