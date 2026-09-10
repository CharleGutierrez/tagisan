---
name: swe-bench-decomposition
description: Decomposition of complex real-world software engineering issues into test-first DAG execution plans
---

# SWE-Bench Problem Decomposition & Resolution

## Structured Workflow
1. **Issue Ingestion**: Parse bug reports, stack traces, and reproduction scripts.
2. **Localization**: Use vector search and codebase indexing to identify the minimal set of modified files.
3. **Failing Regression Test**: Author a dedicated unit/integration test reproducing the exact failure.
4. **Surgical Remediation**: Apply targeted modifications with zero side-effects to unrelated subsystems.
5. **Verification Pass**: Run the full test suite and confirm zero regressions.
