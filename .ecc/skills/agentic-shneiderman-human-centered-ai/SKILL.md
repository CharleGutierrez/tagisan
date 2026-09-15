---
name: agentic-shneiderman-human-centered-ai
description: "High automation and high human control (HCAI matrix), explainable AI, reliable/safe/trustworthy design, and human oversight in agent systems."
triggers: ["shneiderman", "human-centered-ai", "hcai-matrix", "human-in-the-loop", "high-automation-high-control", "explainable-ai"]
---

# agentic-shneiderman-human-centered-ai
> Based on **Human-Centered AI - Ben Shneiderman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **HCAI Two-Dimensional Matrix: Simultaneously maximize Human Control and Computer Automation (avoiding the false choice between total autonomy or manual toil).**
2. **Continuous Oversight Interfaces: Dashboards providing real-time visibility into agent telemetry, pending actions, and override controls.**
3. **Audit Trails & Explainability: Every autonomous action must produce an intelligible audit log explaining why the action was chosen.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Provide transparent audit previews before applying multi-file refactorings. Give the developer instant 1-click override and rollback capabilities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Black-box autonomous modifications with zero explanation or user visibility.**
- **Degrading developer control in the name of full automation, breeding mistrust and rejection.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "shneiderman"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
