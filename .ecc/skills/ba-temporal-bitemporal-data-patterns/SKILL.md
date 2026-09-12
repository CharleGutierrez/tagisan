---
name: ba-temporal-bitemporal-data-patterns
description: "Bitemporal Data Modeling: Valid Time (business reality) vs Transaction Time (system audit record), immutable append-only ledgers, and point-in-time state reconstruction."
triggers: ["temporal-bitemporal-data-patterns", "bitemporal", "snodgrass", "valid-time", "transaction-time", "temporal-database", "audit-immutability"]
---

# ba-temporal-bitemporal-data-patterns
> Based on **Developing Time-Oriented Database Applications in SQL - Richard T. Snodgrass**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bitemporal Invariant: Distinguish Valid Time [vt_start, vt_end] (when fact was true in real world) from Transaction Time [tt_start, tt_end] (when recorded in database).**
2. **Immutability of History: Never execute destructive SQL `UPDATE` or `DELETE` on financial or statutory tables; append new temporal records.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model temporal records with `valid_from`, `valid_to`, `recorded_at`, and `recorded_by`. Use temporal ranges for point-in-time reconstruction.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overwriting previous historical data, destroying legal audit trails.**
- **Conflating database system insertion timestamp with the real-world business event date.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "temporal-bitemporal-data-patterns"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
