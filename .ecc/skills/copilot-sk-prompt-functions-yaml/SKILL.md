---
name: copilot-sk-prompt-functions-yaml
description: "Declarative YAML prompt functions (prompt.yaml), input variables, template engines (Handlebars / Liquid)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Semantic Kernel Handlebars and Liquid Prompt Templating - Microsoft Dev"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-prompt-functions", "sk-yaml-prompts", "prompt-yaml-schema", "handlebars-prompt-templating"]
---

# copilot-sk-prompt-functions-yaml
> Based on **Semantic Kernel Handlebars and Liquid Prompt Templating - Microsoft Dev** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Declarative Spec: `prompt.yaml` defines template, execution settings (temperature, top_p), and input variables.**
2. **Template Engines: Utilize Handlebars template engine for structured loops, conditionals, and sub-function calls.**
3. **MANDATORY schema validation of prompt.yaml before runtime loading.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-sk-prompt-functions-yaml.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-prompt-functions-yaml.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Package prompts as version-controlled YAML assets, decoupling prompt engineering from compiled application binaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-prompt-functions-yaml.**
- **Unmonitored runtime execution without telemetry in copilot-sk-prompt-functions-yaml.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-prompt-functions"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
