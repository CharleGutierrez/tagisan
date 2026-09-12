---
name: ba-kanban-value-stream-metrics
description: "Kanban Value Stream Metrics & Flow Control: Little's Law, Work in Progress (WIP) limits, Cycle Time, Lead Time, and Cumulative Flow Diagrams (CFD)."
triggers: ["kanban-value-stream-metrics", "kanban", "littles-law", "wip-limits", "cycle-time", "lead-time", "cumulative-flow-diagram"]
---

# ba-kanban-value-stream-metrics
> Based on **Kanban: Successful Evolutionary Change - David J. Anderson**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Little's Law Invariant: Average Lead Time = Work In Progress (WIP) / Throughput.**
2. **WIP Restriction: To reduce cycle time and improve delivery predictability, ruthlessly limit active WIP at each stage of the engineering pipeline.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Monitor delivery using Little's Law. Set WIP limits on each state in the development pipeline. Measure and minimize Lead Time and Cycle Time.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting 20 tasks in parallel, causing context switching and blowing out lead times.**
- **Ignoring bottlenecks where tasks pile up indefinitely.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kanban-value-stream-metrics"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
