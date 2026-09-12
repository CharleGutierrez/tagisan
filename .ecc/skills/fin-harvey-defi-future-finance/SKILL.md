---
name: fin-harvey-defi-future-finance
description: "Decentralized financial infrastructure: Automated Market Makers (AMM: x * y = k), constant product liquidity pools, impermanent loss, and flash loans."
triggers: ["campbell-harvey", "defi-future-finance", "automated-market-maker", "amm", "constant-product", "impermanent-loss", "flash-loans", "liquidity-pools"]
---

# fin-harvey-defi-future-finance
> Based on **DeFi and the Future of Finance - Campbell R. Harvey, Ashwin Ramachandran, Joey Santoro**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce the Constant Product Invariant (x * y = k) in AMM pool state transitions.**
2. **ALWAYS: Calculate and disclose Impermanent Loss to liquidity providers: IL = 2 * sqrt(Price_Ratio) / (1 + Price_Ratio) - 1.**
3. **NEVER: Allow flash-loanable state transitions to manipulate spot oracle prices without Time-Weighted Average Price (TWAP) protections.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Simulate AMM mechanics with constant-product invariants (x * y = k): incorporate concentrated liquidity math and TWAP oracle protections.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using spot AMM reserves as a pricing oracle for lending collateral.**
- **Failing to deduct swap fees before verifying invariant k.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-harvey-defi-future-finance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
