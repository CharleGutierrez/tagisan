---
name: copilot-teams-ai-library-core-architecture
description: "Teams AI Library (@microsoft/teams-ai), ApplicationBuilder, and TurnContext lifecycle."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Intelligent Bots with Teams AI Library - Microsoft Dev"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-ai-library", "teams-ai-core", "application-builder", "turncontext-lifecycle"]
---

# copilot-teams-ai-library-core-architecture
> Based on **Building Intelligent Bots with Teams AI Library - Microsoft Dev** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Architecture Pattern: `Application` orchestrates activity routing, authentication, and AI turn planner.**
2. **Turn Flow: User Activity -> Middleware -> Auth Handler -> ActionPlanner -> Tool Execution -> Response.**
3. **ALWAYS preserve turn context state during async I/O.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-library-core-architecture.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-ai-library-core-architecture actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build custom engine agents using `@microsoft/teams-ai`, configuring `ApplicationBuilder` with typed state and planners.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-library-core-architecture.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-library-core-architecture.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-ai-library"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
