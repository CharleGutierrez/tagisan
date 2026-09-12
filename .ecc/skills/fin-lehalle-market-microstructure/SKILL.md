---
name: fin-lehalle-market-microstructure
description: "Limit order book (LOB) mechanics: Queue position priority, order flow toxicity, bid-ask spread decomposition, and market impact models."
triggers: ["charles-lehalle", "market-microstructure", "limit-order-book", "lob", "queue-priority", "order-flow-toxicity", "market-impact", "slippage-model"]
---

# fin-lehalle-market-microstructure
> Based on **Market Microstructure in Practice - Charles-Albert Lehalle & Sophie Laruelle**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model execution latency and queue position priority (Price-Time vs Pro-Rata) in limit order book simulations.**
2. **ALWAYS: Decompose bid-ask spreads into order processing, inventory holding, and adverse selection components.**
3. **NEVER: Assume zero market impact for trade orders exceeding 1% of Average Daily Volume (ADV).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct LOB matching engines with price-time priority queues and realistic slippage modeling based on order book depth.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming instant execution at the touch without order book depth modeling.**
- **Simulating backtests with zero execution slippage.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-lehalle-market-microstructure"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
