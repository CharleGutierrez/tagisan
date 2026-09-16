---
name: copilot-mobile-teams-card-ergonomics
description: "Optimizing card layouts for mobile Teams clients (single-column reflow, touch targets)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Don't Make Me Think: Conversational UI Usability - Steve Krug"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["mobile-teams-ergonomics", "card-mobile-optimization", "touch-targets", "responsive-cards"]
---

# copilot-mobile-teams-card-ergonomics
> Based on **Don't Make Me Think: Conversational UI Usability - Steve Krug** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Responsive Reflow: Use ColumnSet with `wrap: true` or single-column stacks to ensure clean mobile rendering.**
2. **Touch Target Invariant: Interactive buttons must provide minimum 48x48 pixel touch targets.**
3. **NEVER use wide multi-column tables that require horizontal scrolling on mobile.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-mobile-teams-card-ergonomics.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-mobile-teams-card-ergonomics actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design responsive Adaptive Cards that render flawlessly across mobile, tablet, and desktop Teams clients.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-mobile-teams-card-ergonomics.**
- **Unmonitored runtime execution without telemetry in copilot-mobile-teams-card-ergonomics.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "mobile-teams-ergonomics"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
