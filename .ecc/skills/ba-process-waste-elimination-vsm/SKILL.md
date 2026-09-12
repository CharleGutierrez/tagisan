---
name: ba-process-waste-elimination-vsm
description: "Lean Value Stream Mapping (VSM): Eliminating the 7 Wastes (Mudas: Waiting, Transit, Overprocessing, Inventory, Defects, Motion, Overproduction), Takt Time, PCE ratio."
triggers: ["process-waste-elimination-vsm", "vsm", "value-stream-mapping", "womack-jones", "7-wastes", "muda", "process-cycle-efficiency"]
---

# ba-process-waste-elimination-vsm
> Based on **Lean Thinking - James P. Womack & Daniel T. Jones**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Process Cycle Efficiency (PCE): PCE = (Value-Add Time / Total Lead Time) * 100. Target: Increase PCE by eliminating waiting and transit mudas.**
2. **Takt Time Pacing: Takt Time = Available Production Time / Customer Demand Rate.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map process steps as Value-Added (VA), Business-Value-Added (BVA), or Non-Value-Added (NVA/Waste). Formulate redesigns to eliminate 100% of NVA waste.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Automating unnecessary non-value-added steps instead of removing them.**
- **Measuring local sub-task speed while ignoring massive wait queues.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "process-waste-elimination-vsm"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
