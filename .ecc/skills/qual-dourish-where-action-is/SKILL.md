---
name: qual-dourish-where-action-is
description: "Embodied interaction: Phenomenological human-computer interaction, tangible computing, social computing, and direct manipulation in space and time."
triggers: ["paul-dourish", "where-the-action-is", "embodied-interaction", "tangible-computing", "phenomenology-hci", "direct-manipulation", "spatial-consistency"]
---

# qual-dourish-where-action-is
> Based on **Where the Action Is: The Foundations of Embodied Interaction - Paul Dourish**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Maintain continuous direct manipulation physics (inertial scrolling, tactile drag-and-drop, spatial consistency) in visual UI.**
2. **ALWAYS: Ground software interactions in tangible, physical metaphors that honor human bodily and spatial intuition.**
3. **NEVER: Break spatial consistency when manipulating graphical objects on a digital canvas or workspace.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement direct manipulation physics with sub-16ms render loops, preserving spatial continuity and tactile embodiment.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Jumping or teleporting elements during drag-and-drop interactions.**
- **Inconsistent gesture mappings that violate spatial continuity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-dourish-where-action-is"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
