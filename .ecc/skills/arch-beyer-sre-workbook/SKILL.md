---
name: arch-beyer-sre-workbook
description: "Practical SRE implementation: Multi-window multi-burn-rate alerting, postmortem culture with actionable follow-ups, canary release gating, and disaster recovery."
triggers: ["sre-workbook", "multi-burn-rate-alerting", "canary-gating", "postmortem-action-items", "disaster-recovery", "slo-engineering"]
---

# arch-beyer-sre-workbook
> Based on **The Site Reliability Workbook - Betsy Beyer, David N. Blank-Edelman et al.**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Implement Multi-Window Multi-Burn-Rate alerting: trigger pages only when error budget is burning rapidly across both short and long windows (e.g. 14.4x in 1h AND 5m).**
2. **ALWAYS: Automate canary release gating: halt deployments immediately when canary error rate deviates from baseline.**
3. **NEVER: Write a blameless postmortem without filing concrete, prioritized engineering tickets to eliminate the underlying root cause.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure multi-burn-rate alert rules against SLOs. Enforce automated canary rollbacks on error budget degradation. Conduct blameless postmortems with verifiable prevention tasks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Alerting on single spike anomalies that self-resolve in 30 seconds.**
- **Holding postmortems that produce no actionable architecture or prevention tasks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-beyer-sre-workbook"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
