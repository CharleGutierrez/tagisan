---
name: agentic-ferber-reactive-agents
description: "Situated and reactive agents, stimulus-response architectures, subsumption hierarchy, environmental affordances, and emergence in physical and virtual spaces."
triggers: ["ferber", "reactive-agents", "subsumption-architecture", "situated-agents", "stimulus-response", "environmental-affordances"]
---

# agentic-ferber-reactive-agents
> Based on **Multi-Agent Systems: An Introduction to Distributed Artificial Intelligence - Jacques Ferber**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Stimulus-Response Invariant: Action a = f(s) is computed directly from sensor observation s without complex intermediate deliberative planning.**
2. **Subsumption Layering: Higher-level competence layers subsume (suppress or inhibit) lower-level reactive behaviors without modifying lower layers.**
3. **Affordance Landscape: The environment encodes cues that directly trigger agent actions, minimizing internal state overhead.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build ultra-low-latency agents using layered stimulus-response architectures. Use subsumption to handle reflex responses (e.g. rate limit backoff, syntax error retries) while higher layers execute strategic flows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Over-engineering simple reactive tasks into multi-turn deliberative LLM chains.**
- **Cyclic subsumption inhibition loops resulting in behavioral freeze.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ferber"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
