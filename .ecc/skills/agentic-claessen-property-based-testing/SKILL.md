---
name: agentic-claessen-property-based-testing
description: "Property-based testing, universal property specifications, algebraic invariants (associativity, idempotence, round-trip), and automated generative testing."
triggers: ["claessen", "hughes", "quickcheck", "property-based-testing", "algebraic-invariants", "generative-testing", "round-trip-testing"]
---

# agentic-claessen-property-based-testing
> Based on **QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs - Koen Claessen & John Hughes**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Universal Property Invariant: forall x in Domain: Property(x) == true across thousands of randomly generated inputs.**
2. **Round-Trip Property: deserialize(serialize(x)) == x for all valid domain objects x.**
3. **Idempotence Property: f(f(x)) == f(x) for operations like formatting, normalization, and reconciliation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement property-based tests (using proptest, hypothesis, or QuickCheck) for all serialization, parsers, and mathematical state transitions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying solely on 2-3 hardcoded example test cases for complex parsers or data serializers.**
- **Writing property tests with weak assertions that never challenge edge cases.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "claessen"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
