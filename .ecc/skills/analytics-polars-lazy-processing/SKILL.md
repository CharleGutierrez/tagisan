---
name: analytics-polars-lazy-processing
description: Rust-native Polars analytics: LazyFrame query planning, predicate/projection/slice pushdown, parallel expression execution, and streaming engines. Triggers: polars-lazy-processing, polars, polars-rust, lazyframe-optimizer, predicate-pushdown, projection-pushdown, arrow-native.
triggers:
  - polars-lazy-processing
  - polars
  - polars-rust
  - lazyframe-optimizer
  - predicate-pushdown
  - projection-pushdown
  - arrow-native
---

# Analytics Polars Lazy Processing
> Based on **Data Analysis with Polars - Jeroen Janssens**

## 1. Canonical Architecture & Data Modeling (DDL)

```sql
-- Polars Pipeline Execution Trace Log
CREATE TABLE polars_job_metrics (
    job_id UUID PRIMARY KEY,
    rows_scanned BIGINT NOT NULL,
    rows_output BIGINT NOT NULL,
    execution_time_ms BIGINT NOT NULL,
    peak_memory_mb DOUBLE PRECISION NOT NULL
);
```

## 2. Core Mathematical Foundations & Analytical Invariants

### 2.1 Relational Algebra Pushdown Optimization
Let $\sigma_p$ be predicate filter and $\pi_C$ be column projection:
$$\pi_C(\sigma_p(R)) \equiv \pi_C(\sigma_p(\pi_{C \cup \text{Vars}(p)}(R)))$$
Polars reads only columns $C \cup \text{Vars}(p)$ from disk, discarding non-matching rows before materialization.

## 3. Pipeline Flow & State Machine Invariants

```mermaid
graph TD
    LazyCode[LazyFrame: .filter().select().groupby()] --> Plan[Construct Logical Plan]
    Plan --> Optimize[Query Optimizer: Pushdown Predicates & Projections]
    Optimize --> Physical[Construct Physical Plan with Rayon Threads]
    Physical --> Collect[Execute .collect() across CPU Cores]
```

## 4. Production Implementation Guidelines (Rust / SQL / Python)

```sql
import polars as pl

q = (
    pl.scan_parquet("data/*.parquet")
    .filter(pl.col("status") == "COMPLETED")
    .group_by(["cohort", "region"])
    .agg([
        pl.col("revenue").sum().alias("total_rev"),
        pl.col("user_id").n_unique().alias("unique_users")
    ])
    .sort("total_rev", descending=True)
)
# df = q.collect(streaming=True)
```

## 5. Actionable Prompt Recipes

### 5.1 Local Ollama Cheat Sheet (Actionable Constraints & Invariants)
```markdown
- Always use scan_parquet() and LazyFrame; call .collect() only at the final step.
- Polars optimizer automatically re-orders operations: filters and column selections are pushed to source.
- Never use Python loops or apply(); use Polars native expression contexts (select, with_columns).
- Enable streaming=True in .collect() to process datasets that exceed available RAM.
```

### 5.2 Cloud LLM Architectural Prompt (Comprehensive Engineering Blueprint)
```markdown
Design high-performance analytical engines using Polars and Apache Arrow:
1. Formulate lazy query plans utilizing projection and predicate pushdown for optimal I/O throughput.
2. Build multi-threaded aggregation pipelines scaling linearly across CPU cores with Rayon.
3. Deploy Polars streaming mode to execute out-of-core transforms on multi-gigabyte datasets.
```
