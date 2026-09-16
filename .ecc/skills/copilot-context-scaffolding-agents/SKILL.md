---
name: copilot-context-scaffolding-agents
description: "Pre-loading conversation context with architectural decision records, schemas, and linting rules."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["context-scaffolding", "prompt-context-preloading", "adr-context-injection", "architectural-rules"]
---

# copilot-context-scaffolding-agents
> Based on **A Philosophy of Software Design: Deep Agentic Modules - John Ousterhout** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Scaffolding Protocol: Maintain a `.gemini/` or `.ecc/` directory of project rules and schemas.**
2. **Progressive Disclosure: Inject only the specific schemas and invariants relevant to the active sub-task.**
3. **ALWAYS keep project architecture guidelines versioned alongside codebase.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-context-scaffolding-agents.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-context-scaffolding-agents actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Scaffold Copilot coding sessions with concise project rules, domain schemas, and invariants to prevent architectural drift.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-context-scaffolding-agents.**
- **Unmonitored runtime execution without telemetry in copilot-context-scaffolding-agents.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "context-scaffolding"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
