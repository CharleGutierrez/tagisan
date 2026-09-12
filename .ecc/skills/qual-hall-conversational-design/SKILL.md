---
name: qual-hall-conversational-design
description: "Principles of conversational UI: Human conversation as collaborative turn-taking, context retention, progressive disclosure in dialogue, and repair protocols."
triggers: ["erika-hall-conversation", "conversational-design", "turn-taking", "context-retention", "repair-protocol", "conversational-ui", "dialogue-management"]
---

# qual-hall-conversational-design
> Based on **Conversational Design - Erika Hall**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Support graceful conversational repair: allow users to clarify, correct mid-sentence, or pivot intent without resetting conversation history.**
2. **ALWAYS: Retain conversational context and referential pronouns ('it', 'that', 'the previous file') across turns.**
3. **NEVER: Force users into rigid keyword matching or drop context after a minor validation error.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain persistent conversational state and natural repair protocols, enabling seamless turn-taking, clarification, and context resolution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Chatbots that reset the entire session when an unrecognized word is entered.**
- **Requiring users to re-state the entire context every turn.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-hall-conversational-design"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
