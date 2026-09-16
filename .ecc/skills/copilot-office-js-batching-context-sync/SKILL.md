---
name: copilot-office-js-batching-context-sync
description: "Efficient context.sync() batching, avoiding roundtrips, and property loading (range.load)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Developing Office Add-ins with Office.js - Michael Saunders"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-js-batching", "context-sync-optimization", "property-loading-range", "office-js-performance"]
---

# copilot-office-js-batching-context-sync
> Based on **Developing Office Add-ins with Office.js - Michael Saunders** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Batching Invariant: Queue all reads and writes before invoking `await context.sync()`.**
2. **Explicit Property Loading: ALWAYS invoke `.load('property')` before accessing object properties post-sync.**
3. **NEVER call `context.sync()` inside high-iteration loops.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-office-js-batching-context-sync actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize Office.js add-in performance by batching commands and minimizing client-to-host bridge roundtrips.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-js-batching-context-sync.**
- **Unmonitored runtime execution without telemetry in copilot-office-js-batching-context-sync.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-js-batching"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
