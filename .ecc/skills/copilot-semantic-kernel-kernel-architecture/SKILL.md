---
name: copilot-semantic-kernel-kernel-architecture
description: "Semantic Kernel core architecture: Kernel, Service Collection, AI Service registration (Azure OpenAI/OpenAI)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Programming Microsoft Semantic Kernel - Lucas Vogel"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["semantic-kernel-core", "sk-kernel-architecture", "sk-service-collection", "sk-azure-openai"]
---

# copilot-semantic-kernel-kernel-architecture
> Based on **Programming Microsoft Semantic Kernel - Lucas Vogel** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Kernel Lifecycle: `Kernel` acts as the dependency injection and orchestration hub.**
2. **Multi-Service Routing: Register multiple chat completion and embedding services under distinct service IDs.**
3. **ALWAYS configure timeout and retry policies on underlying HTTP client handlers.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-semantic-kernel-kernel-architecture.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-semantic-kernel-kernel-architecture actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Initialize and configure `Kernel` instances with strongly-typed AI connectors and plugin registries in C# and Python.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-semantic-kernel-kernel-architecture.**
- **Unmonitored runtime execution without telemetry in copilot-semantic-kernel-kernel-architecture.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "semantic-kernel-core"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
