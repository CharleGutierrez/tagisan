---
name: copilot-sk-native-plugins-csharp-python
description: "Authoring native plugins with [KernelFunction] (C#) and @kernel_function (Python) with typed docstrings."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Programming Microsoft Semantic Kernel - Lucas Vogel"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-native-plugins", "kernel-function-decorator", "typed-docstrings", "sk-native-functions"]
---

# copilot-sk-native-plugins-csharp-python
> Based on **Programming Microsoft Semantic Kernel - Lucas Vogel** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Decorator Contract: Mark methods with `[KernelFunction]` (C#) or `@kernel_function` (Python).**
2. **Metadata Annotation: `[Description("...")]` on functions and parameters is MANDATORY for model discovery.**
3. **ALWAYS validate parameter bounds inside native function bodies.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-native-plugins-csharp-python.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Author native Semantic Kernel plugins that expose local computations, file operations, and database access to LLM planners.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-native-plugins-csharp-python.**
- **Unmonitored runtime execution without telemetry in copilot-sk-native-plugins-csharp-python.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-native-plugins"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
