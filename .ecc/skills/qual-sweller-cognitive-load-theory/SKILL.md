---
name: qual-sweller-cognitive-load-theory
description: "Working memory limits and cognitive ergonomics: 4±1 item capacity, Extraneous load reduction, Germane schema formation, and Intrinsic task chunking."
triggers: ["sweller", "cognitive-load-theory", "working-memory", "extraneous-load", "split-attention-effect", "modality-effect", "progressive-disclosure"]
---

# qual-sweller-cognitive-load-theory
> Based on **Cognitive Load Theory - John Sweller, Paul Ayres, Slava Kalyuga**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Eliminate extraneous cognitive load by removing visual clutter, redundant text, and split-attention layouts.**
2. **ALWAYS: Bound visible decision choices per viewport to 4 ± 1 items; chunk complex multi-factor forms into progressive stages.**
3. **NEVER: Force users to hold disconnected reference information in working memory across disparate tabs or modal dialogs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure user interfaces to preserve working memory limits. Group related attributes into cohesive chunks and reveal complexity progressively.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Displaying 30 un-grouped form fields on a single sprawling screen.**
- **Forcing users to memorize codes from one screen to enter on another.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-sweller-cognitive-load-theory"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
