---
name: qual-reeves-media-equation
description: "The Media Equation (Media = Real Life): Human interactions with computers and autonomous agents are fundamentally social and obey real-world psychological rules."
triggers: ["reeves-nass", "media-equation", "social-actors", "politeness-protocol", "reciprocity", "agent-etiquette", "flattery-effects"]
---

# qual-reeves-media-equation
> Based on **The Media Equation: How People Treat Computers, Television, and New Media Like Real People and Places - Byron Reeves & Clifford Nass**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Program AI agents to take ownership of communicative breakdowns (e.g. 'I didn't quite catch that') rather than blaming the user.**
2. **ALWAYS: Observe human social norms of politeness, reciprocity, and respect in agent dialogue.**
3. **NEVER: Insult, condescend to, or scold the user in error messages or automated prompts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design agent communications according to social psychology principles: maintain humility, assume ownership of confusion, and practice conversational etiquette.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **System error messages stating 'User entered invalid data' or 'User failed to configure'.**
- **Condescending or patronizing chatbot responses.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-reeves-media-equation"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
