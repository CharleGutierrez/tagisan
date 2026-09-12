---
name: arch-parker-distributed-tracing-practice
description: "Distributed tracing fundamentals: OpenTelemetry (OTel), W3C Trace Context (`traceparent`, `tracestate`), Baggage propagation, Spans, and Critical Path analysis."
triggers: ["distributed-tracing", "opentelemetry", "otel", "w3c-trace-context", "traceparent", "span-hierarchy", "critical-path-analysis"]
---

# arch-parker-distributed-tracing-practice
> Based on **Distributed Tracing in Practice - Austin Parker, Daniel Spoonhower, Jonathan Mace, Ben Sigelman, Rebecca Isaacs**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Extract and propagate W3C `traceparent` headers across every HTTP, gRPC, and asynchronous message broker hop.**
2. **ALWAYS: Model distributed operations as a hierarchy of Spans representing start time, end time, status, and semantic attributes.**
3. **NEVER: Break the trace context chain when spawning background tasks or worker threads.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement OpenTelemetry SDK instrumentation. Inject and extract W3C `traceparent` across all network boundaries. Propagate baggage context for tenant/environment tracking.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Generating new trace IDs inside downstream microservices, breaking the distributed trace graph.**
- **Dropping trace headers when enqueuing messages into Kafka or RabbitMQ.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-parker-distributed-tracing-practice"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
