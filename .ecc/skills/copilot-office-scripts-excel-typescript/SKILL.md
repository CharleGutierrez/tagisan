---
name: copilot-office-scripts-excel-typescript
description: "Writing Office Scripts in TypeScript, manipulating ranges, tables, pivot tables, and conditional formats."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Excel Office Scripts with TypeScript - Yutaka Hiraoka"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-scripts-excel", "excel-typescript-scripts", "workbook-range-mutation", "office-scripts-automation"]
---

# copilot-office-scripts-excel-typescript
> Based on **Excel Office Scripts with TypeScript - Yutaka Hiraoka** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Execution Sandbox: Office Scripts run within a dedicated web worker sandbox with strict API boundaries.**
2. **Zero External Network: Office Scripts CANNOT make arbitrary `fetch()` requests directly; use Power Automate for I/O.**
3. **ALWAYS batch operations inside `main(workbook: ExcelScript.Workbook)`.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-office-scripts-excel-typescript.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-office-scripts-excel-typescript actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Author high-performance TypeScript Office Scripts to automate complex workbook mutations and calculations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-scripts-excel-typescript.**
- **Unmonitored runtime execution without telemetry in copilot-office-scripts-excel-typescript.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-scripts-excel"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
