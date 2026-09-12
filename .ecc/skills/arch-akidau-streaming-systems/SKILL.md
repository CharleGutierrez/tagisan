---
name: arch-akidau-streaming-systems
description: "The unified streaming architecture: What (transformations), Where (event-time windowing), When (watermarks), and How (accumulating vs retracting triggers)."
triggers: ["akidau", "streaming-systems", "event-time", "processing-time", "watermarks", "sliding-windows", "session-windows", "stream-triggers"]
---

# arch-akidau-streaming-systems
> Based on **Streaming Systems - Tyler Akidau, Slava Chernyak, Reuven Lax**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Window stream calculations by Event Time (when the event occurred) rather than Processing Time (when the server ingested it).**
2. **ALWAYS: Establish explicit Watermarks representing event-time completeness and define deterministic Late-Data handling (side outputs or retractions).**
3. **NEVER: Assume event arrivals are ordered across distributed producers.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model streaming computations using the 4 dimensions: What (transform), Where (window), When (watermark/trigger), and How (accumulation mode). Handle out-of-order and late data deterministically.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Grouping streaming events by server ingestion clock.**
- **Dropping late-arriving events silently without dead-letter or side-output logging.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-akidau-streaming-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
