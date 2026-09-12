---
name: arch-keeling-design-it
description: "Actionable architecture practices: Architecture Decision Records (ADRs), Risk-Driven Architecture, Architecture Katas, and collaborative design facilitation."
triggers: ["keeling-design-it", "adr", "architecture-decision-records", "risk-driven-architecture", "architecture-katas", "design-mindset"]
---

# arch-keeling-design-it
> Based on **Design It!: From Programmer to Software Architect - Michael Keeling**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Every non-trivial structural decision, database selection, or protocol change must be recorded as an immutable Architecture Decision Record (ADR).**
2. **ALWAYS: ADR format must strictly include: Title, Status (Proposed/Accepted/Superseded), Context, Decision, and Consequences (positive, negative, neutral).**
3. **NEVER: Overrule or reverse an accepted ADR without committing a new superseding ADR documenting the altered context.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain an `/adrs` directory recording every significant architectural choice. Prioritize engineering work using Risk-Driven Architecture matrices.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Making major architectural changes via chat or meetings without committing an ADR to the repository.**
- **Failing to document negative consequences and trade-offs of architectural decisions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-keeling-design-it"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
