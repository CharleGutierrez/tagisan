---
name: ba-customer-journey-mapping
description: "Customer Journey Mapping: User personas, journey phases, touchpoints, emotional curves, pain point heatmaps, and moment-of-truth interventions."
triggers: ["customer-journey-mapping", "journey-mapping", "jim-kalbach", "touchpoints", "customer-experience", "emotional-arc"]
---

# ba-customer-journey-mapping
> Based on **Mapping Experiences (2nd Edition) - Jim Kalbach**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Touchpoint Completeness: Map all chronological touchpoints across Pre-service, In-service, and Post-service phases.**
2. **Friction Score Quantifiability: Rate customer friction and emotional sentiment at each touchpoint to highlight critical drop-off cliffs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Document the end-to-end customer journey: Persona, Phases, User Actions, Touchpoints, Emotional Sentiment (-5 to +5), Pain Points, Opportunities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mapping internal organization department handoffs instead of the user's actual external experience.**
- **Ignoring the post-service follow-up phase.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "customer-journey-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
