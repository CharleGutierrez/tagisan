---
name: ba-galls-law-system-evolution
description: "Gall's Law & Complex System Evolution: How complex systems evolve from simple working systems, failure modes of premature complexity, and the functional core."
triggers: ["galls-law-system-evolution", "galls-law", "john-gall", "systemantics", "evolutionary-architecture", "functional-core"]
---

# ba-galls-law-system-evolution
> Based on **The Systems Bible (Systemantics) - John Gall**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Gall's Law: 'A complex system that works is invariably found to have evolved from a simple system that worked. A complex system designed from scratch never works and cannot be made to work.'**
2. **Incremental Complexity Invariant: Never prompt an AI agent to build a multi-tier distributed system in one step; evolve from a verified simple working core.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce Gall's Law: Phase 1: Build minimal working core in-memory; Phase 2: Add persistence; Phase 3: Add rules/auth; Phase 4: Add distributed scaling.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Scaffolding microservices, queues, and distributed consensus before verifying the core domain logic.**
- **Trying to fix an unworking complex system with more complexity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "galls-law-system-evolution"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
