---
name: arch-beyer-site-reliability-engineering
description: "Google SRE principles: Service Level Indicators (SLIs), Service Level Objectives (SLOs), Error Budgets, Eliminating Toil, Cascading Failure Mitigation, and Overload Handling."
triggers: ["google-sre", "sli-slo", "error-budgets", "eliminating-toil", "overload-shedding", "cascading-failure-prevention", "site-reliability"]
---

# arch-beyer-site-reliability-engineering
> Based on **Site Reliability Engineering (SRE Book) - Betsy Beyer, Chris Jones, Jennifer Petoff, Niall Richard Murphy**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Quantify system reliability with explicit SLIs (availability, latency) and derive SLOs with Error Budgets that gate feature deployment velocity.**
2. **ALWAYS: Shed load gracefully when approaching capacity limits using priority shedding (drop non-critical background jobs, preserve critical user transactions).**
3. **NEVER: Page on-call engineers for alerts that do not require immediate, manual human intervention.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define SLI/SLO metrics for all API contracts. Implement automated load shedding based on CPU/queue saturation. Freeze deployments when error budget is exhausted.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Alerting on transient CPU spikes instead of user-facing SLO degradation.**
- **Allowing unbounded queue growth under load rather than shedding non-essential requests.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-beyer-site-reliability-engineering"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
