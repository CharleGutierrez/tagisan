---
name: agentic-tulving-episodic-memory-retrieval
description: "Episodic vs semantic memory, autonoetic consciousness, retrieval cues, temporal tagging, chronesthesia, and agent trajectory retrospection."
triggers: ["tulving", "episodic-memory", "semantic-memory", "autonoetic-consciousness", "temporal-tagging", "retrieval-cues"]
---

# agentic-tulving-episodic-memory-retrieval
> Based on **Elements of Episodic Memory - Endel Tulving**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Episodic-Semantic Distinction: Semantic memory stores general knowledge ('Rust has ownership'); episodic memory stores temporally situated personal experiences ('In step 3 I broke the build').**
2. **Encoding Specificity Principle: Retrieval is successful only if the cues present during retrieval match the information encoded with the memory trace.**
3. **Chronesthesia: Mental time travel allowing an agent to simulate future outcomes by reconstructing past episodic trajectories.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Tag agent tool execution logs with precise temporal and situational metadata. Use encoding specificity to retrieve past debugging sessions that share identical error signatures.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Collapsing all historical actions into generic semantic rules, losing the chronological sequence of why decisions were made.**
- **Querying memory with generic keywords that lack the situational cues present during original failure.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "tulving"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
