---
name: ba-invest-user-stories
description: "Agile User Stories & INVEST Rubric: Independent, Negotiable, Valuable, Estimable, Small, Testable stories, Card-Conversation-Confirmation (3Cs)."
triggers: ["invest-user-stories", "invest-rubric", "mike-cohn", "bill-wake", "user-stories", "3cs-card-conversation-confirmation"]
---

# ba-invest-user-stories
> Based on **User Stories Applied - Bill Wake & Mike Cohn**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **INVEST Verification: Every story must be Independent, Negotiable, Valuable, Estimable, Small (fits in one prompt/sprint), and Testable.**
2. **Story Syntax: 'As a [specific persona], I want [capability/action] so that [business value/benefit]'.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Validate all backlog items against INVEST checklist before asking AI to code. Ensure every story has concrete Given/When/Then acceptance criteria.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing technical tasks disguised as user stories.**
- **Writing huge epic stories that exceed the AI agent's single-turn context.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "invest-user-stories"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
