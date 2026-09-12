---
name: qual-portigal-interviewing-users
description: "Qualitative user research: Semi-structured interviewing, uncovering tacit mental models, listening for compensating workarounds, and field inquiry."
triggers: ["portigal", "interviewing-users", "user-research", "semi-structured-interview", "field-inquiry", "tacit-knowledge", "compensating-workarounds"]
---

# qual-portigal-interviewing-users
> Based on **Interviewing Users: How to Uncover Compelling Insights - Steve Portigal**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Base feature requirements on observed user workarounds (spreadsheets, copy-pasting, post-it notes) rather than speculative user requests.**
2. **ALWAYS: Frame qualitative discovery around actual past behavior ('Tell me about the last time you...') rather than hypothetical futures.**
3. **NEVER: Ask leading questions or validate roadmap assumptions through speculative user surveys.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design software that eliminates observed human workarounds and compensatory habits. Ground functional workflows in real-world qualitative user transcripts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building features based on speculative user feature voting without behavioral observation.**
- **Designing for hypothetical idealized workflows.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-portigal-interviewing-users"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
