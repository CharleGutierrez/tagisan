---
name: agentic-laird-soar-cognitive-architecture
description: "Production system rules, working memory elements (WMEs), subgoaling on impasses, chunking (rule learning), and unified cognitive architectures."
triggers: ["laird", "soar-architecture", "cognitive-architecture", "subgoaling-impasses", "chunking-learning", "working-memory-elements"]
---

# agentic-laird-soar-cognitive-architecture
> Based on **The Soar Cognitive Architecture - John E. Laird**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Decision Cycle: Input -> Elaboration (parallel production firing) -> Operator Proposal -> Operator Selection -> Operator Application -> Output.**
2. **Subgoaling on Impasses: When the agent cannot select an operator (tie, conflict, or no-change impasse), Soar creates a substate to resolve it.**
3. **Chunking Invariant: The system compiles the results of successful substate problem solving into new permanent production rules.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Detect reasoning impasses (e.g. uncertainty between two libraries). Spawn an isolated subagent to resolve the impasse, then cache the resolution rule.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Failing to recognize when an agent is in an impasse, causing endless circular retries.**
- **Discarding the lessons learned from resolved impasses instead of chunking them into memory.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "laird"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
