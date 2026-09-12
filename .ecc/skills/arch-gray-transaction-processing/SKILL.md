---
name: arch-gray-transaction-processing
description: "Formal transaction processing: ACID guarantees, ARIES recovery algorithm (Analysis, Redo, Undo), serializability theory, two-phase locking (2PL), and compensation transactions."
triggers: ["jim-gray", "transaction-processing", "aries-recovery", "acid", "two-phase-locking", "isolation-levels", "undo-redo-log"]
---

# arch-gray-transaction-processing
> Based on **Transaction Processing: Concepts and Techniques - Jim Gray & Andreas Reuter**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Transaction state engine must support ARIES-compliant crash recovery: Analysis phase, Redo of all logged changes, and Undo of uncommitted transactions.**
2. **ALWAYS: Enforce explicit isolation levels (Read Committed, Repeatable Read, Serializable) and provide deterministic retry loops for serialization failures (409 Conflict).**
3. **NEVER: Release locks before the end of the transaction in Two-Phase Locking (2PL) protocols.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify exact ACID transactional boundary. Provide compensation transactions for multi-stage operations. Implement ARIES recovery logging with write-ahead logs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Ignoring database serialization failures without retry logic.**
- **Performing partial rollbacks without compensation logs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-gray-transaction-processing"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
