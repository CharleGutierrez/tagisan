---
name: qual-klein-sources-of-power
description: "Naturalistic Decision Making (NDM): Recognition-Primed Decision (RPD) model, expert intuition under time pressure, mental simulation, and pre-mortems."
triggers: ["gary-klein", "sources-of-power", "naturalistic-decision-making", "recognition-primed-decision", "rpd-model", "pre-mortem", "mental-simulation"]
---

# qual-klein-sources-of-power
> Based on **Sources of Power: How People Make Decisions - Gary Klein**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Incident-response and operations dashboards must present rich situational cues and a single pre-validated primary action to enable fast mental simulation.**
2. **ALWAYS: Conduct a Pre-Mortem before initiating high-risk architectural deployments or automated migration scripts.**
3. **NEVER: Force operators into analytical multi-criteria comparison modals during high-severity production incidents.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design operational controls that support recognition-primed expert decisions: display clear situational context and singular executable remediation paths.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Forcing an on-call engineer to configure 12 query parameters while production is down.**
- **Skipping pre-mortem risk identification on major releases.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-klein-sources-of-power"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
