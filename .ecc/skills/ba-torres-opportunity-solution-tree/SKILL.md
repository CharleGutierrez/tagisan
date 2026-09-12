---
name: ba-torres-opportunity-solution-tree
description: "Opportunity Solution Trees (OST): Desired outcomes, opportunity space exploration, multiple solution candidates, and continuous assumption testing."
triggers: ["torres-opportunity-solution-tree", "opportunity-solution-tree", "ost", "teresa-torres", "continuous-discovery", "assumption-testing"]
---

# ba-torres-opportunity-solution-tree
> Based on **Continuous Discovery Habits - Teresa Torres**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **OST Tree Topology: Desired Business Outcome -> Customer Opportunities (Pain Points/Needs) -> Potential Solutions -> Assumption Tests.**
2. **Never implement a Solution without testing at least 3 distinct Opportunity candidates.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Before writing code, map the Opportunity Solution Tree: Top Outcome -> 2-3 Opportunities -> 2-3 Solutions per Opportunity -> Assumption test cards.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Jumping straight from a metric to code without mapping the customer opportunity space.**
- **Testing solutions as monolithic wholes instead of testing underlying assumptions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "torres-opportunity-solution-tree"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
