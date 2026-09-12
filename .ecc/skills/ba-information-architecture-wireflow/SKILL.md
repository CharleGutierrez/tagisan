---
name: ba-information-architecture-wireflow
description: "Information Architecture & Wireflow Modeling: The 5 Planes (Strategy, Scope, Structure, Skeleton, Surface), Wireflows (wireframe + state machine transition diagram)."
triggers: ["information-architecture-wireflow", "wireflow", "information-architecture", "jesse-james-garrett", "5-planes", "screen-state-flow"]
---

# ba-information-architecture-wireflow
> Based on **The Elements of User Experience - Jesse James Garrett**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Wireflow Graph Completeness: Every screen wireframe must define entry transitions, exit transitions, empty states, loading skeletons, and error toasts.**
2. **Navigational Hierarchy: Users must always know where they are, where they can go, and how to get back to home in <= 3 clicks.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model UI as a Wireflow Graph: Connect screen wireframes with state transition arrows. Specify Empty, Loading, and Error states for every screen.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Designing static wireframes without documenting screen transition triggers.**
- **Forgetting empty states for screens before data is populated.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "information-architecture-wireflow"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
