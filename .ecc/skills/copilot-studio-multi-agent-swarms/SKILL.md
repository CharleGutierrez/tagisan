---
name: copilot-studio-multi-agent-swarms
description: "Multi-agent delegation, supervisor orchestrators, subagent task routing, and handoffs."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Multi-Agent System Choreography in Copilot Studio - Michael Wooldridge"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-studio-multi-agent", "agent-swarms", "supervisor-orchestrator", "subagent-task-routing"]
---

# copilot-studio-multi-agent-swarms
> Based on **Multi-Agent System Choreography in Copilot Studio - Michael Wooldridge** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Supervisor-Worker Pattern: A primary orchestrator agent decomposes requests and delegates to specialized subagents.**
2. **Context Handoff Contract: Handoff payloads must transfer session state, conversation history, and authenticated user tokens.**
3. **ALWAYS establish a return path from subagents back to the supervisor upon task completion.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-multi-agent-swarms.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-studio-multi-agent-swarms actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect modular agent swarms in Copilot Studio: HR Agent, IT Helpdesk Agent, Finance Agent coordinated by a master Enterprise Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Circular handoffs between subagents causing conversational deadlocks.**
- **Dropping user authentication context during subagent delegation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-studio-multi-agent"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
