---
name: copilot-sk-agent-framework-chat-completion
description: "ChatCompletionAgent, AgentGroupChat, and multi-agent coordination strategies in SK."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Chat Completion and Multi-Agent Orchestration in Semantic Kernel - Shawn Henry"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-agent-framework", "chat-completion-agent", "agent-group-chat", "sk-multi-agent"]
---

# copilot-sk-agent-framework-chat-completion
> Based on **Chat Completion and Multi-Agent Orchestration in Semantic Kernel - Shawn Henry** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Agent Abstraction: `ChatCompletionAgent` encapsulates persona, instructions, and dedicated plugin subset.**
2. **Group Chat Coordination: `AgentGroupChat` manages multi-agent turns with custom selection and termination strategies.**
3. **MANDATORY termination strategy to prevent infinite conversational chatter.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-sk-agent-framework-chat-completion.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-agent-framework-chat-completion.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Orchestrate multi-agent collaborations (Reviewer, Coder, Verifier) in Semantic Kernel using structured group chat strategies.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-agent-framework-chat-completion.**
- **Unmonitored runtime execution without telemetry in copilot-sk-agent-framework-chat-completion.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-agent-framework"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
