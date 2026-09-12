---
name: arch-majors-observability-engineering
description: "High-cardinality observability: Column-level truth, structured events, wide events, eliminating alert fatigue, and answering novel unknown-unknown system queries."
triggers: ["charity-majors", "observability-engineering", "high-cardinality", "structured-wide-events", "unknown-unknowns", "bubbleup-analysis", "telemetry"]
---

# arch-majors-observability-engineering
> Based on **Observability Engineering - Charity Majors, Liz Fong-Jones, George Miranda**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Emit canonical structured Wide JSON Events per request containing high-cardinality metadata (user_id, tenant_id, trace_id, build_sha, latency_ms).**
2. **ALWAYS: Avoid relying on pre-aggregated metrics that destroy high-cardinality dimensions needed for root-cause debugging.**
3. **NEVER: Use unstructured plaintext logging (`println!`, `console.log`) in production server code.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Instrument all requests with structured, wide events carrying rich context and high-cardinality fields. Ensure queries can slice and dice by arbitrary dimensions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Logging unstructured strings without correlation IDs or tenant metadata.**
- **Pre-aggregating metrics at the client, making it impossible to identify which specific tenant experienced errors.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-majors-observability-engineering"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
