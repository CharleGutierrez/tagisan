---
name: copilot-power-pages-ai-site-generation
description: "Vibe building Power Pages portals, liquid templates, and Web API integration for Copilot."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building External Web Portals with Power Pages and Copilot - Colin Vermander"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-pages-copilot", "power-pages-ai", "liquid-templates-copilot", "portal-web-api"]
---

# copilot-power-pages-ai-site-generation
> Based on **Building External Web Portals with Power Pages and Copilot - Colin Vermander** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **AI Site Generation: Generate page layouts, forms, and Liquid code snippets via natural language Copilot prompts.**
2. **Table Permissions Invariant: External portal users MUST possess explicit Table Permissions to access Dataverse data.**
3. **ALWAYS enable CSRF token verification on Power Pages Web API endpoints.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-pages-ai-site-generation.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-power-pages-ai-site-generation actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Rapidly prototype and deploy external-facing Power Pages portals with grounded Copilot AI components.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-pages-ai-site-generation.**
- **Unmonitored runtime execution without telemetry in copilot-power-pages-ai-site-generation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-pages-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
