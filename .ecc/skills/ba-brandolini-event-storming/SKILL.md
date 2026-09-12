---
name: ba-brandolini-event-storming
description: "EventStorming domain discovery: Domain Events (orange), Commands (blue), Aggregates (yellow), Read Models (green), Policies (pink), and chronological event flows."
triggers: ["brandolini-event-storming", "event-storming", "alberto-brandolini", "domain-events", "commands-aggregates", "event-driven-analysis"]
---

# ba-brandolini-event-storming
> Based on **Introducing EventStorming - Alberto Brandolini**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Event Past-Tense Invariant: Domain Events must always be written in the past tense (e.g., `OrderPlaced`, `CaseRaffled`, `PaymentDeclined`).**
2. **Command-Event Causality: Every Domain Event is triggered by a Command, an External Event, or a Business Policy.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map processes using EventStorming sequence: [COMMAND] -> [AGGREGATE] -> [EVENT] -> [POLICY] -> [NEXT COMMAND].

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using present-tense or imperative verbs for events (e.g., `PlaceOrder`).**
- **Missing the aggregate consistency boundary where commands are validated.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "brandolini-event-storming"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
