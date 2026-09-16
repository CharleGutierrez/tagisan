---
name: copilot-power-automate-error-handling-scopes
description: "Flow run retry policies, Scope-based Try-Catch-Finally, and run-after error escalation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Automating Business Processes with Power Automate and AI Builder - Aaron Murchie"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-automate-error-handling", "flow-try-catch-finally", "retry-policies", "run-after-escalation"]
---

# copilot-power-automate-error-handling-scopes
> Based on **Automating Business Processes with Power Automate and AI Builder - Aaron Murchie** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pattern Implementation: Wrap critical flow actions in Scope_Try, Scope_Catch, Scope_Finally blocks.**
2. **Run-After Configuration: Configure Scope_Catch to run only if Scope_Try has 'has failed', 'has timed out'.**
3. **MANDATORY notification to system administrators on catch execution.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-power-automate-error-handling-scopes.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-automate-error-handling-scopes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement enterprise-grade error handling in Power Automate flows using Scope blocks and automated dead-letter alerts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-automate-error-handling-scopes.**
- **Unmonitored runtime execution without telemetry in copilot-power-automate-error-handling-scopes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-automate-error-handling"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
