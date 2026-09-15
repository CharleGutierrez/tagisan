---
name: agentic-baddeley-working-memory-buffers
description: "Central executive, phonological loop, visuospatial sketchpad, episodic buffer, capacity limits in context windows, and cognitive load distribution."
triggers: ["baddeley", "working-memory", "central-executive", "episodic-buffer", "cognitive-load", "context-window-management"]
---

# agentic-baddeley-working-memory-buffers
> Based on **Working Memory, Thought, and Action - Alan Baddeley**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Multi-Component Working Memory Model: Central Executive coordinates attention and controls three slave buffers: Phonological Loop, Visuospatial Sketchpad, and Episodic Buffer.**
2. **Capacity Limits (Miller/Cowan): Working memory can reliably manipulate only 4-7 active conceptual chunks simultaneously.**
3. **Episodic Buffer: Binds cross-modal information into coherent chronological episodes available for executive decision-making.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure LLM context windows to emulate Baddeley's working memory: an executive system prompt, a concise episodic buffer of recent turns, and scratchpad space.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overloading the context window with dozens of unorganized code snippets exceeding the LLM's effective attention span.**
- **Failing to clear the working memory buffer after concluding an isolated subtask.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "baddeley"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
