---
name: harness-dag-lifecycle-telemetry-pro-max
description: Harness Lifecycle, Composition DAG & Telemetry for Tagisan. Multi-harness composition pipelines (piping outputs between synthesized CLIs), SemVer upgrade & deprecation shims, and execution telemetry feedback loop (auto-refining SKILL.md cheat sheets based on agent failure traces). Enforces topological DAG execution, dead-letter queue handling, idempotent execution, and schema backward-compatibility checks.
version: 1.0.0
tags:
  - dag-pipeline
  - harness-lifecycle
  - composition
  - semver-shims
  - deprecation
  - telemetry-feedback
  - self-refining
  - dead-letter-queue
triggers:
  - dag-lifecycle-telemetry
  - harness-dag
  - cli-composition
  - semver-shim
  - deprecation-shim
  - telemetry-refinement
  - skill-feedback-loop
  - dead-letter-queue
  - pipeline-orchestrator
compatibility: ">=0.2.0"
---

# Harness Lifecycle, Composition DAG & Telemetry: Autonomous Multi-CLI Pipelines & Feedback Self-Refinement

## Purpose & Scope
Individual CLI harnesses become exponentially more powerful when composed into directed acyclic graph (DAG) pipelines, where data flows seamlessly between native Rust, Bun, Go, and WASM binaries. However, maintaining multi-harness pipelines over time requires strict SemVer deprecation lifecycles and an autonomous telemetry feedback loop that refines agent instructions based on observed runtime failures.

The `harness-dag-lifecycle-telemetry-pro-max` skill coordinates multi-harness execution, handles deprecation shims, enforces schema compatibility across pipeline stages, and captures execution telemetry to autonomously refine `SKILL.md` cheat sheets.

---

## 1. Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                            DAG LIFECYCLE & TELEMETRY INVARIANTS                                   |
+---------------------------------------------------------------------------------------------------+
|  1. TOPOLOGICAL DAG EXECUTION & CYCLE REJECTION                                                   |
|     Pipelines must be validated via Tarjan's or Kahn's algorithm for topological sorting. Any     |
|     cyclic dependency must be rejected before any execution begins. Parallel nodes execute async. |
+---------------------------------------------------------------------------------------------------+
|  2. INTER-STAGE SCHEMA COMPATIBILITY CONTRACT                                                     |
|     Output schemas of producer nodes must strictly satisfy or exceed input schemas of consumer   |
|     nodes. Field removal or type mutation triggers compile-time pipeline validation errors.      |
+---------------------------------------------------------------------------------------------------+
|  3. DEAD-LETTER QUEUE (DLQ) CAPTURE                                                               |
|     Any stage that fails must dump its complete input arguments, stdin payload, stdout, stderr,   |
|     and exit code to an isolated dead-letter queue fixture for post-mortem debugging.             |
+---------------------------------------------------------------------------------------------------+
|  4. IDEMPOTENT EXECUTION & CHECKPOINTING                                                          |
|     Pipeline stages must support idempotent re-runs via hash-based artifact checkpointing.        |
|     Previously succeeded stages are skipped if their inputs and binaries are bitwise identical.   |
+---------------------------------------------------------------------------------------------------+
|  5. AUTONOMOUS TELEMETRY-DRIVEN SKILL REFINEMENT                                                  |
|     Recurring runtime errors (e.g. exit code 64 arg mismatches, OOM kills) are automatically      |
|     analyzed and synthesized into actionable "Pitfalls & Traps" sections in target SKILL.md files. |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Multi-Harness Composition DAG Architecture

```
+-----------------------------------------------------------------------------------+
|                         TAGISAN COMPOSITION DAG RUNNER                            |
+-----------------------------------------------------------------------------------+
                                        |
               +------------------------+------------------------+
               |                                                 |
               v                                                 v
      +------------------+                              +------------------+
      | Node A (Rust):   |                              | Node B (Go):     |
      | Ingest Binary    |                              | Fetch Remote SDL |
      +------------------+                              +------------------+
               |                                                 |
               +------------------------+------------------------+
                                        |
                                        v
                               +------------------+
                               | Node C (Bun):    |
                               | Transform & Fuzz |
                               +------------------+
                                        |
                                        v
                               +------------------+
                               | Node D (WASM):   |
                               | Verify Invariant |
                               +------------------+
```

### Declarative DAG Pipeline Specification (`pipeline.yaml`)
```yaml
version: "1.0"
name: "telemetry-ingest-verify"
nodes:
  - id: "ingest"
    harness: "tgs-harness-rs"
    command: ["digest", "--algorithm", "blake3", "data/input.bin", "--json"]
    output_key: "digest_info"

  - id: "query_catalog"
    harness: "tgs-harness-go"
    command: ["catalog", "lookup", "--hash", "${ingest.data.digest}", "--json"]
    depends_on: ["ingest"]
    output_key: "catalog_info"

  - id: "validate"
    harness: "tgs-harness-bun"
    command: ["verify", "--expected", "${query_catalog.data.owner}", "--json"]
    depends_on: ["query_catalog"]
    output_key: "validation_result"

dead_letter_queue:
  path: "target/dlq/"
  max_retries: 2
```

---

## 3. SemVer Deprecation & Compatibility Shims

When a CLI harness command or flag is modified, backward compatibility must be preserved across at least one minor release cycle.

```rust
//! SemVer Deprecation Shim Handler
pub fn handle_deprecated_flag(flag: &str, replacement: &str, json_mode: bool) {
    if json_mode {
        // Warning injected into JSON telemetry envelope
    } else {
        eprintln!(
            "\x1b[33m[DEPRECATION WARNING]\x1b[0m Flag '--{}' is deprecated and will be removed in v2.0.0. Please use '--{}' instead.",
            flag, replacement
        );
    }
}
```

---

## 4. Telemetry Feedback Loop: Autonomous `SKILL.md` Refinement

The telemetry feedback engine monitors agent interactions with synthesized harnesses:
1. **Trace Ingestion**: Capture CLI invocation: arguments, exit code, stdout, stderr.
2. **Failure Pattern Clustering**: Group failures by error code (e.g. `ERR_INVALID_ARGUMENT`, `ERR_CONNECTION_REFUSED`).
3. **Hypothesis Generation**: Identify the root cause (e.g. "Agent frequently passes `--file` instead of positional `[path]`").
4. **Skill Patching**: Autonomously inject warning notes and correct usage examples directly into the skill's `SKILL.md` file under a `### Common Pitfalls & Agent Guidance` section.
