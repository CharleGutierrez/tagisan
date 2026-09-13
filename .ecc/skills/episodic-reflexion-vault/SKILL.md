---
name: episodic-reflexion-vault
description: Self-curating engineering case law and episodic memory vault for autonomous agents. Captures solved compiler panics, error signatures, root causes, applied fixes, and preventative invariants to preempt redundant debugging cycles.
version: 1.0.0
tags:
  - episodic-memory
  - reflexion-vault
  - case-law
  - root-cause-analysis
  - debugging-invariants
  - post-mortem
triggers:
  - episodic-memory
  - reflexion
  - case-law
  - post-mortem
  - debugging-vault
  - root-cause-analysis
  - compiler-panic
  - incident-learning
  - preventative-invariant
compatibility: ">=0.2.0"
---

# Episodic Reflexion Vault: Self-Curating Engineering Case Law

## Purpose & Scope
Autonomous engineering agents frequently encounter esoteric compiler diagnostics, borrow-checker constraints, subtle race conditions, and runtime panics. Without persistent episodic reflexion, agents repeatedly reinvent failure modes across tasks and sessions.

This skill equips autonomous agents with self-curating engineering case law. By formalizing every solved non-trivial failure into a structured case record (error signature, root cause, applied patch, and preventative invariant), agents turn ephemeral debugging into permanent organizational immunity.

---

## 1. Operational Invariants (Dense Case Law Protocol)

### Invariant 1: Mandatory Pre-Debug Vault Query
- **ALWAYS**: Before attempting multi-step exploratory debugging or making speculative trial-and-error edits when a compiler error or test failure occurs, query the reflexion vault using `reflexion_vault(action: "query", query: <error_signature_or_symptom>)`.
- **STRICT_REJECT**: Never proceed with deep code surgery if an identical or near-neighbor error signature exists in the vault without first inspecting the precedent fix.

### Invariant 2: Atomic Incident Recording
- **ALWAYS**: Upon successfully diagnosing and resolving a non-trivial bug, panic, lifetime violation, or regression, record the incident into the vault via `reflexion_vault(action: "record", ...)`.
- The record must contain:
  1. `error_signature`: Exact compiler error code (e.g. `E0382`, `E0597`), panic message, or failing assertion.
  2. `root_cause`: The precise architectural, lifetime, concurrency, or type mismatch mechanism.
  3. `fix_applied`: The concrete diff or structural pattern change that resolved the failure.
  4. `preventative_invariant`: An actionable, generalized rule (e.g., `ALWAYS wrap shared mutable state in Arc<RwLock<T>> before thread spawn`) to prevent recurrence.
  5. `tags`: Relevant subsystem, language, and technology identifiers.

### Invariant 3: Generalization Over Situational Band-Aids
- **NEVER**: Persist situational or trivial reflexions (such as typographical fixes or transient network timeouts).
- **MANDATORY**: The `preventative_invariant` must be formulated as a universal design principle that can be evaluated during code generation or lint review.

### Invariant 4: No Repetitive Diagnostic Hallucination
- **NEVER**: Formulate hypotheses that contradict recorded case-law in the vault. If precedent establishes that a given pattern triggers deadlocks or undefined behavior, that approach is strictly vetoed.

---

## 2. Structured Case Law Schema

Every entry in the reflexion vault adheres to the following specification:
```json
{
  "id": "refl-<timestamp>-<hash>",
  "timestamp": "2026-09-13T11:56:00Z",
  "error_signature": "cannot borrow `*self` as mutable, as it is also borrowed as immutable (E0502)",
  "root_cause": "Holding an immutable reference from `self.sandboxes.get()` while invoking a mutating helper method on `self`.",
  "fix_applied": "Scoped immutable reference lookup into a block or cloned handle prior to calling mutating receiver method.",
  "preventative_invariant": "ALWAYS scope immutable borrows to minimal lifetimes before invoking mutable methods on parent structs.",
  "tags": ["rust", "borrow-checker", "lifetimes", "concurrency"]
}
```

---

## 3. Workflow & Verification Recipe

### Step 3.1: Querying Past Lessons
```bash
# Query precedent for rust lifetime conflicts
tgs tool execute reflexion_vault '{"action": "query", "query": "borrow as mutable immutable E0502"}'
```

### Step 3.2: Recording a Proven Fix
```bash
# Persist resolved case law into .tagisan/reflexions.json
tgs tool execute reflexion_vault '{
  "action": "record",
  "error_signature": "E0502: cannot borrow *self as mutable",
  "root_cause": "Simultaneous read and write borrow on struct member",
  "fix_applied": "Extracted field clone before mutable mutation block",
  "preventative_invariant": "ALWAYS drop read guards before calling mutable receiver methods",
  "tags": ["rust", "borrowck"]
}'
```
