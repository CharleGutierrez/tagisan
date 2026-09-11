---
name: conceptual-integrity-systems
description: "Conceptual Integrity and Software Engineering at Scale (Fred Brooks and Titus Winters): unified architectural vision, second-system syndrome avoidance, Hyrum's law, and sustainability over time."
triggers: ["conceptual integrity", "fred brooks", "mythical man month", "hyrum's law", "software engineering at google", "titus winters", "second system effect", "architectural cohesion", "programming over time"]
---

# Conceptual Integrity and Engineering Over Time (Brooks and Winters)

This skill equips the agent with the high-level governance and sustainability disciplines articulated by Fred Brooks (*The Mythical Man-Month*) and Titus Winters (*Software Engineering at Google*).

## 1. Conceptual Integrity (Fred Brooks)
1. **The Central Virtue of System Design**:
   - 'Conceptual integrity is the most important consideration in system design. It is better to have a system omit certain anomalous features and improvements, but to reflect one set of design ideas, than to have one which contains many good but independent and uncoordinated ideas.'
2. **Guarding Against AI Feature Accretion**:
   - When building with AI agents, each prompt can pull the architecture in a disparate direction. The engineer must enforce a singular design dialect:
     - Unified naming conventions.
     - Unified error handling strategy across all crates/modules.
     - Unified asynchronous and streaming abstractions.
3. **Resisting the Second-System Effect**:
   - Avoid over-engineering a replacement or extension by loading it with all the frustrated ambitions from the first system. Keep abstractions lean and focused.

## 2. Software Engineering vs. Programming (Titus Winters)
1. **The Time Dimension**:
   - Software engineering is programming integrated over time. Code must not merely function today; it must be maintainable, upgradeable, and resilient to dependency decay over years.
2. **Hyrum's Law**:
   - 'With a sufficient number of users of an API, it does not matter what you promise on the contract: all observable behaviors of your system will be depended upon by somebody.'
   - Therefore:
     - Explicitly define and hide non-contractual details (such as iteration order, timing, and internal error string formats).
     - Do not expose internal structs directly if internal fields may change.
3. **Shift-Left Testing and Readability Discipline**:
   - Find defects as close to their introduction as possible (compiler type checks > unit tests > integration tests > production alerts).
   - Prioritize reader ease over writer ease: code is read 100 times more often than it is written.
