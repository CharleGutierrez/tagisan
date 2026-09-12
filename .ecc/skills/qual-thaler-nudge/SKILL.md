---
name: qual-thaler-nudge
description: "Choice architecture & libertarian paternalism: Defaults, feedback, mapping choices to welfare, incentives, structured choices, and error tolerance."
triggers: ["thaler", "sunstein", "nudge", "choice-architecture", "defaults", "libertarian-paternalism", "error-tolerance", "benevolent-defaults"]
---

# qual-thaler-nudge
> Based on **Nudge: Improving Decisions About Health, Wealth, and Happiness - Richard H. Thaler & Cass R. Sunstein**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Configure default settings to the safest, most pro-user, privacy-preserving, and sustainable option (Benevolent Defaults).**
2. **ALWAYS: Provide transparent, single-click mechanisms for users to inspect and modify default behaviors.**
3. **NEVER: Deploy predatory defaults that exploit cognitive inertia (pre-checked subscriptions, hidden marketing opt-ins).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement benevolent choice architecture: pre-select optimal security, privacy, and cost-efficiency defaults while keeping user overrides frictionless.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Pre-selecting recurring payment add-ons during checkout.**
- **Defaulting to public data visibility without explicit user consent.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-thaler-nudge"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
