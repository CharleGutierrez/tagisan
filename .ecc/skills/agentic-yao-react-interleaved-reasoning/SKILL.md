---
name: agentic-yao-react-interleaved-reasoning
description: "Interleaved Thought-Action-Observation loops, external knowledge grounding, hallucination interruption, and dynamic trajectory adjustment in language agents."
triggers: ["yao", "react-framework", "thought-action-observation", "interleaved-reasoning", "tool-grounding", "hallucination-interruption"]
---

# agentic-yao-react-interleaved-reasoning
> Based on **ReAct: Synergizing Reasoning and Acting in Language Models - Shunyu Yao et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **ReAct Triad: Thought_t -> Action_t -> Observation_t sequence where reasoning guides action and observation grounds reasoning.**
2. **External Grounding Invariant: The agent MUST NOT hallucinate the results of tool calls; all state transitions depend strictly on Observation_t.**
3. **Hallucination Interruption: If Observation_t contradicts Thought_t, the agent immediately enters a corrective Thought_{t+1} phase.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Execute all autonomous actions in a strict Thought -> Action -> Observation loop. Never combine multiple unverified actions into a single ungrounded assumption.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Emitting speculative observations instead of awaiting true tool return payloads.**
- **Skipping the Thought phase, devolving into unguided random tool thrashing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "yao"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
