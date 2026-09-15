---
name: agentic-ravichandiran-llm-engineering
description: "LangChain & LlamaIndex internals, agent chains, tool use integrations, prompt templates, memory buffers, and orchestration pipelines."
triggers: ["ravichandiran", "llm-chains", "tool-integration", "prompt-templates", "agent-orchestration", "memory-buffers"]
---

# agentic-ravichandiran-llm-engineering
> Based on **Getting Started with Large Language Models - Sudharsan Ravichandiran**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Tool Calling Protocol: The LLM emits structured tool invocation tokens which the runtime executes and returns as tool result messages.**
2. **Conversational Buffer Memory: Rolling memory buffers prune historical messages to prevent context exhaustion while retaining core directives.**
3. **Chaining Paradigm: Composable pipeline execution where output of step N feeds input of step N+1 under invariant assertions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct agent workflows as deterministic Directed Acyclic Graphs (DAGs). Isolate tool execution in sandboxed environments with strict timeouts and error handling.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Unbounded memory buffers growing until the context window overflows and triggers runtime crashes.**
- **Allowing tools to execute destructive shell commands without human-in-the-loop verification.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ravichandiran"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
