---
name: qual-kahneman-thinking-fast-slow
description: "Dual-process cognitive psychology: System 1 (fast, associative, heuristic-driven) vs. System 2 (slow, analytical, resource-hungry, easily fatigued) and cognitive biases."
triggers: ["kahneman", "thinking-fast-and-slow", "system-1-system-2", "cognitive-bias", "availability-heuristic", "anchoring-bias", "loss-aversion", "framing-effect"]
---

# qual-kahneman-thinking-fast-slow
> Based on **Thinking, Fast and Slow - Daniel Kahneman**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Primary user flows (search, checkout, read, navigate) must operate entirely within System 1 cognitive fluency.**
2. **ALWAYS: Reserve deliberate System 2 friction (confirmation gates, dual-verification, explicit typing) exclusively for high-risk, irreversible operations.**
3. **NEVER: Overload System 2 working memory with complex mental calculations or ambiguous status choices during routine tasks.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize interfaces for intuitive System 1 recognition. Introduce intentional cognitive friction only at irreversible destruction boundaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Requiring multi-step analytical calculation for simple data entry.**
- **Omitting confirmation barriers on destructive batch deletions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-kahneman-thinking-fast-slow"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
