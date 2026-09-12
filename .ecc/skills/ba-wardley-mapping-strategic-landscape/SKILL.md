---
name: ba-wardley-mapping-strategic-landscape
description: "Wardley Mapping & Strategic Value Chains: Anchor customer need, Value Chain positioning, Evolution axis (Genesis -> Custom-Built -> Product/Rental -> Commodity/Utility)."
triggers: ["wardley-mapping-strategic-landscape", "wardley-maps", "simon-wardley", "value-chain", "evolution-axis", "strategic-mapping"]
---

# ba-wardley-mapping-strategic-landscape
> Based on **Wardley Maps: Topographical Intelligence in Business - Simon Wardley**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Evolution Axis Monotonicity: Components inevitably evolve from Genesis -> Custom -> Product -> Commodity over time under competitive pressure.**
2. **Value Chain Positioning: Components higher on the Y-axis are visible to the user; components lower down are invisible infrastructure dependencies.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map system components on a Wardley Map: Y-axis (User Visibility), X-axis (Evolution: Genesis, Custom, Product, Commodity). Outsource commodity layers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Custom-building components that already exist as commodities.**
- **Treating custom-built proprietary software as if it were a stable commodity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wardley-mapping-strategic-landscape"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
