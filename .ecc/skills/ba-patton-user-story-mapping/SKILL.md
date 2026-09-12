---
name: ba-patton-user-story-mapping
description: "User Story Mapping: Dual-backbone 2D grid, user activities and tasks, horizontal slicing, walking skeletons, and incremental release framing."
triggers: ["patton-user-story-mapping", "user-story-mapping", "jeff-patton", "story-mapping", "walking-skeleton", "narrative-backbone"]
---

# ba-patton-user-story-mapping
> Based on **User Story Mapping - Jeff Patton**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Story Map Topology: Horizontal axis represents narrative time (User Activities -> User Tasks); Vertical axis represents release priority slices (MVP -> Release 2).**
2. **Walking Skeleton Invariant: The MVP slice must constitute an end-to-end functional path through the entire user journey, even if implemented with bare-bones technology.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure project roadmaps as a 2D Story Map: Activities across the top, tasks below, sliced horizontally into Walking Skeleton, MVP, and Future Releases.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building 100% of Module 1 before building any of Module 2 or 3.**
- **Writing isolated user stories that have no clear place in the overall narrative journey.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "patton-user-story-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
