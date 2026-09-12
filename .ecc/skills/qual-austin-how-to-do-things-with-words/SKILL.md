---
name: qual-austin-how-to-do-things-with-words
description: "Speech Act Theory: Performative utterances vs constatives; locutionary (meaning), illocutionary (force/action), and perlocutionary (effect) acts."
triggers: ["jl-austin", "how-to-do-things-with-words", "speech-act-theory", "performatives", "illocutionary-force", "perlocutionary-effect", "agent-commitments"]
---

# qual-austin-how-to-do-things-with-words
> Based on **How to Do Things with Words - J. L. Austin**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Treat AI conversational agent messages that mutate database state or authorize external actions as formal illocutionary performative contracts.**
2. **ALWAYS: Require explicit cryptographic or secondary user confirmation before an agent executes irreversible illocutionary side effects.**
3. **NEVER: Conflate informational conversational text with autonomous tool execution commitments.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Separate informational chat output from transactional performative acts, requiring explicit confirmation gates for all autonomous tool executions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Autonomous agents executing money transfers or database drops from unverified conversational intent.**
- **Treating tool execution as plain chat.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-austin-how-to-do-things-with-words"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
