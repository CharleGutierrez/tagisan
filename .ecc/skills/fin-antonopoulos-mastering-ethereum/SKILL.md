---
name: fin-antonopoulos-mastering-ethereum
description: "EVM state transition economics: Account-based ledger state, gas mechanics, EIP-1559 base fee burning, and re-entrancy attack prevention."
triggers: ["gavin-wood", "mastering-ethereum", "evm", "gas-economics", "eip-1559", "re-entrancy-guard", "checks-effects-interactions", "smart-contract-accounting"]
---

# fin-antonopoulos-mastering-ethereum
> Based on **Mastering Ethereum: Building Smart Contracts and DApps - Andreas M. Antonopoulos & Gavin Wood**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce the Checks-Effects-Interactions pattern across smart contracts to prevent re-entrancy drain attacks.**
2. **ALWAYS: Account for EIP-1559 base fee burning and priority tipping in transaction cost estimation engines.**
3. **NEVER: Mutate external contract state before finalizing internal ledger balance updates.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit smart contract accounting logic against re-entrancy: enforce checks-effects-interactions pattern and account for EIP-1559 gas mechanics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Transferring ETH/tokens to external addresses before updating internal user balances.**
- **Ignoring gas limits in loops resulting in bricked contracts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-antonopoulos-mastering-ethereum"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
