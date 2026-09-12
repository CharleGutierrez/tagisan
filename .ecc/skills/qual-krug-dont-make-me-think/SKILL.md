---
name: qual-krug-dont-make-me-think
description: "Common sense web usability: The First Law of Usability, designing for billboard scanning, visual hierarchy, omission of needless words, and clear street signs."
triggers: ["steve-krug", "dont-make-me-think", "web-usability", "billboard-scanning", "visual-hierarchy", "omission-of-needless-words", "self-evident-design"]
---

# qual-krug-dont-make-me-think
> Based on **Don't Make Me Think, Revisited - Steve Krug**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Make pages and UI components self-evident; a user should grasp what it is and how to use it without reading instructional paragraphs.**
2. **ALWAYS: Create clear visual hierarchies where relative importance corresponds to visual prominence.**
3. **NEVER: Require users to read instructions before performing primary tasks.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Eliminate cognitive overhead: make interface affordances and visual hierarchies so clear that every page is self-evident at a glance.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Placing long blocks of explanatory text above a simple form.**
- **Using ambiguous iconography without accompanying text labels.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-krug-dont-make-me-think"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
