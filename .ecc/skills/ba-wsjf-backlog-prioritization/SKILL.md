---
name: ba-wsjf-backlog-prioritization
description: "Weighted Shortest Job First (WSJF) & Cost of Delay (CoD): User-business value, time criticality, risk reduction/opportunity enablement, and economic job sizing."
triggers: ["wsjf-backlog-prioritization", "wsjf", "reinertsen", "cost-of-delay", "economic-prioritization", "job-duration"]
---

# ba-wsjf-backlog-prioritization
> Based on **The Principles of Product Development Flow - Donald G. Reinertsen**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **WSJF Formula: WSJF = Cost of Delay / Job Duration = (User Value + Time Criticality + RR/OE) / Size.**
2. **Economic Priority: Always schedule items with the highest WSJF score first to maximize economic throughput.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Calculate WSJF for all backlog items: CoD = (User-Business Value + Time Criticality + Risk Reduction) / Job Duration. Sort backlog descending by WSJF.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Prioritizing items based on executive loudness (HiPPO) rather than Cost of Delay.**
- **Ignoring job size/duration and scheduling large low-value tasks first.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wsjf-backlog-prioritization"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
