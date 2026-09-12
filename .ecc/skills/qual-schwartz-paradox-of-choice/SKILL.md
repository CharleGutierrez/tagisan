---
name: qual-schwartz-paradox-of-choice
description: "Choice overload and decision paralysis: Maximizers vs. Satisficers, opportunity cost salience, escalation of expectations, and post-decision regret."
triggers: ["barry-schwartz", "paradox-of-choice", "choice-overload", "decision-paralysis", "maximizers-vs-satisficers", "option-curation"]
---

# qual-schwartz-paradox-of-choice
> Based on **The Paradox of Choice: Why More Is Less - Barry Schwartz**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Cap default visible options in menus, templates, and selections to 3 ± 1 high-confidence choices.**
2. **ALWAYS: Provide curated 'Recommended', 'Most Popular', or 'Default' paths to empower fast satisficing decisions.**
3. **NEVER: Confront users with an uncurated, unranked catalog of 50+ choices for a single operational decision.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Eliminate decision paralysis by curating option sets, highlighting recommended defaults, and gating exhaustive lists behind progressive disclosure.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Presenting 40 equally-weighted configuration flags during initial onboarding.**
- **Overwhelming search results without intelligent sorting or faceted filtering.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-schwartz-paradox-of-choice"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
