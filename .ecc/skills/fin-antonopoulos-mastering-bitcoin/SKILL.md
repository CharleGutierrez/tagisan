---
name: fin-antonopoulos-mastering-bitcoin
description: "Decentralized money architecture: Unspent Transaction Output (UTXO) accounting, proof-of-work difficulty adjustments, script execution, and double-spend protection."
triggers: ["andreas-antonopoulos", "mastering-bitcoin", "utxo", "unspent-transaction-output", "proof-of-work", "double-spend-prevention", "crypto-accounting"]
---

# fin-antonopoulos-mastering-bitcoin
> Based on **Mastering Bitcoin: Programming the Open Blockchain - Andreas M. Antonopoulos**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Maintain strict UTXO balance conservation: Sum(Input Values) = Sum(Output Values) + Miner Fee.**
2. **ALWAYS: Verify cryptographic transaction signatures before mutating local ledger states.**
3. **NEVER: Allow state transitions that reference already-spent transaction outputs (double-spending).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement UTXO ledger accounting: enforce invariant that input values strictly equal output values plus miner fees, rejecting double-spends.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Creating unbacked outputs from thin air without input references.**
- **Ignoring transaction malleability and fee estimation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-antonopoulos-mastering-bitcoin"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
