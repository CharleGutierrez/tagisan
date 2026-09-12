---
name: qual-rubin-usability-testing
description: "Empirical usability test design: Formative vs summative testing, test plan formulation, performance benchmarks (Task Completion, Time on Task), and data moderation."
triggers: ["rubin-chisnell", "usability-testing-handbook", "test-plan", "task-completion-rate", "time-on-task", "formative-testing", "summative-testing"]
---

# qual-rubin-usability-testing
> Based on **Handbook of Usability Testing - Jeffrey Rubin & Dana Chisnell**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Define quantifiable benchmark criteria (Task Completion Rate > 90%, zero critical errors) for core user user journeys.**
2. **ALWAYS: Design end-to-end task flows with unambiguous start and success criteria.**
3. **NEVER: Ship complex user workflows without validating that novice users can complete the primary task unaided.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Establish rigorous task completion and error-free execution benchmarks for all generated user pathways prior to production deployment.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming a feature is usable because the engineer who wrote it can navigate it.**
- **Releasing multi-step workflows with zero automated completion metrics.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-rubin-usability-testing"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
