---
name: agentic-hunt-pragmatic-programmer
description: "Don't Repeat Yourself (DRY), orthogonality, tracer bullets, broken windows theory, stone soup, pragmatic paranoia, and engineering craftsmanship."
triggers: ["hunt", "pragmatic-programmer", "dry-principle", "orthogonality", "tracer-bullets", "broken-windows", "pragmatic-paranoia"]
---

# agentic-hunt-pragmatic-programmer
> Based on **The Pragmatic Programmer: Your Journey to Mastery - David Thomas & Andrew Hunt**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **DRY Principle: Every piece of knowledge must have a single, unambiguous, authoritative representation within a system.**
2. **Orthogonality: Eliminate side-effects between unrelated components so modifying module A cannot break module B.**
3. **Tracer Bullets: Implement end-to-end thin vertical slices that connect all architectural layers before fleshing out bulk features.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build vertical tracer bullets to validate end-to-end integration immediately. Never tolerate 'broken windows' (commented-out tests, unaddressed linter warnings).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Copy-pasting duplicate logic across multiple files, violating DRY.**
- **Building elaborate horizontal layers (data models, UI) without ever running end-to-end tracer tests.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hunt"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
