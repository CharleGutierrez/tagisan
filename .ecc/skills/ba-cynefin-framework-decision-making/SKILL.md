---
name: ba-cynefin-framework-decision-making
description: "The Cynefin Sense-Making Framework: Classifying contexts into Clear (Best practice), Complicated (Good practice), Complex (Emergent practice), and Chaotic (Novel practice)."
triggers: ["cynefin-framework-decision-making", "cynefin", "dave-snowden", "sense-making", "complex-adaptive-systems", "decision-framework"]
---

# ba-cynefin-framework-decision-making
> Based on **The Cynefin Framework - Dave Snowden**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Domain Categorization: Clear (Sense-Categorize-Respond), Complicated (Sense-Analyze-Respond), Complex (Probe-Sense-Respond), Chaotic (Act-Sense-Respond).**
2. **Probe-Sense-Respond in Complex Domains: In complex user spaces, conduct small safe-to-fail experiments rather than over-analyzing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Classify project requirements into Cynefin domains. Use standard templates for Clear; analysis for Complicated; safe-to-fail probes for Complex.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Applying rigid 'best practices' to complex emergent problems.**
- **Over-analyzing chaotic situations instead of acting to stabilize flow.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cynefin-framework-decision-making"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
