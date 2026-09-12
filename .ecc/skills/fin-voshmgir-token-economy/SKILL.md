---
name: fin-voshmgir-token-economy
description: "Cryptoeconomics and mechanism design: Token engineering, bonding curves, token velocity (MV = PQ), and staking/burn equilibrium sinks."
triggers: ["shermin-voshmgir", "token-economy", "tokenomics", "bonding-curves", "token-velocity", "equation-of-exchange", "staking-sinks", "token-utility"]
---

# fin-voshmgir-token-economy
> Based on **Token Economy: How the Web3 Reinvents the Internet - Shermin Voshmgir**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model token velocity using the equation of exchange (M * V = P * Q); high velocity degrades token price without staking/burn sinks.**
2. **ALWAYS: Ensure cryptoeconomic mechanism designs align individual participant incentives with network stability.**
3. **NEVER: Issue utility tokens whose sole function is transactional medium-of-exchange without value-capture sinks.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Analyze tokenomic designs using equation of exchange (MV = PQ): calculate token velocity and verify existence of durable economic sinks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Launching tokens with 100% circulating velocity and zero staking utility.**
- **Assuming token price will rise simply because transaction count increases.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-voshmgir-token-economy"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
