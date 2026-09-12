---
name: ba-domain-storytelling
description: "Domain Storytelling methodology: Visual, actor-centric storytelling using pictographic notations, work objects, activities, and sequence numbers."
triggers: ["domain-storytelling", "stefan-hofer", "henning-schwentner", "domain-stories", "work-objects", "actor-activities"]
---

# ba-domain-storytelling
> Based on **Domain Storytelling - Stefan Hofer & Henning Schwentner**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Grammar of Domain Stories: Actor -> Activity -> Work Object -> Destination Actor. Every step must have a strict integer sequence number (1, 2, 3...).**
2. **Concrete People & Real Objects: Use specific real-world examples and work objects, avoiding abstract data structures.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model business conversations as Domain Stories: (1) Actor A sends Work Object to Actor B; (2) Actor B evaluates Work Object; (3) Actor B creates Output.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Abstracting away the actors and turning stories into generic data flow diagrams.**
- **Skipping sequence numbers and creating ambiguous execution paths.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "domain-storytelling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
