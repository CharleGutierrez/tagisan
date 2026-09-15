---
name: agentic-krug-intuitive-interaction
description: "Cognitive load minimization, visual hierarchy, self-evident interfaces, mindless navigation, and ruthless omission of needless words in developer tools."
triggers: ["krug", "dont-make-me-think", "cognitive-load-minimization", "visual-hierarchy", "self-evident-design", "frictionless-interaction"]
---

# agentic-krug-intuitive-interaction
> Based on **Don't Make Me Think - Steve Krug**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **First Law of Usability: As far as humanly possible, interfaces should be self-evident and obvious without requiring a manual.**
2. **Muddle vs Clarity: Clear visual hierarchy: things that are related visually belong together; primary actions dominate secondary actions.**
3. **Omission of Needless Elements: Strip away boilerplate, extraneous text, and cognitive clutter from developer prompts and terminal outputs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format terminal and chat outputs with clean visual hierarchy, clear headings, and zero useless chatter. Make next steps completely obvious.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Dumping walls of unformatted markdown text that drown key warnings.**
- **Burying critical error remedies in verbose paragraph explanations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "krug"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
