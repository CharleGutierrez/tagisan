---
name: arch-hohpe-enterprise-integration-patterns
description: "Enterprise messaging patterns: Pipes and Filters, Message Router, Splitter, Aggregator, Resequencer, Dead Letter Channel, and Content-Based Router."
triggers: ["enterprise-integration-patterns", "eip", "pipes-and-filters", "message-router", "dead-letter-channel", "splitter-aggregator", "resequencer"]
---

# arch-hohpe-enterprise-integration-patterns
> Based on **Enterprise Integration Patterns (EIP) - Gregor Hohpe & Bobby Woolf**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Poison messages that fail parsing or processing after N retry attempts must be automatically diverted to an isolated Dead Letter Channel (DLC/DLQ).**
2. **ALWAYS: Composed message processing must assemble disparate asynchronous responses using an Aggregator with a correlation ID and completion timeout.**
3. **NEVER: Discard failed messages silently without routing to an inspection dead-letter store.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect event pipelines using canonical EIP primitives: Pipes & Filters, Content-Based Routers, Splitter/Aggregators, and Dead Letter Queues with Correlation IDs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Retrying unparseable malformed messages infinitely, clogging the message broker queue.**
- **Losing asynchronous sub-task responses due to missing correlation IDs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-hohpe-enterprise-integration-patterns"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
