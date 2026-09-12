---
name: fin-lewis-flash-boys
description: "Electronic market plumbing: High-frequency trading (HFT) latency arbitrage, colocation, payment for order flow (PFOF), and dark pool mechanics."
triggers: ["michael-lewis", "flash-boys", "latency-arbitrage", "hft", "colocation", "pfof", "dark-pools", "smart-order-router", "speed-bump"]
---

# fin-lewis-flash-boys
> Based on **Flash Boys: A Wall Street Revolt - Michael Lewis**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Protect client order routing from front-running and adverse latency arbitrage by using randomized delays or Smart Order Routing (SOR).**
2. **ALWAYS: Measure and audit execution quality across routing venues (fill rate, price improvement, effective spread).**
3. **NEVER: Route unhedged limit orders to dark pools without continuous fill-rate monitoring.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design smart order routers (SOR) with multi-venue execution quality checks, slippage benchmarking, and latency arbitrage protections.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Routing orders to venues based on rebate payments rather than client price improvement.**
- **Ignoring latency arbitrage front-running.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-lewis-flash-boys"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
