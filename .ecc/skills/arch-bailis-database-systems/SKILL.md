---
name: arch-bailis-database-systems
description: "Modern database systems architecture: Relational query compilation, columnar vectorized execution, shared-nothing vs shared-disk topologies, and HTAP hybridization."
triggers: ["bailis-stonebraker", "red-book", "columnar-execution", "vectorized-query", "shared-nothing", "htap", "oltp-olap-separation"]
---

# arch-bailis-database-systems
> Based on **Readings in Database Systems ('The Red Book') - Peter Bailis, Joseph M. Hellerstein, Michael Stonebraker**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Separate OLTP write workloads from OLAP aggregation workloads; route analytical queries to columnar stores (DuckDB, ClickHouse, BigQuery).**
2. **ALWAYS: Vectorized execution engines must operate on SIMD-aligned contiguous column batches rather than row-at-a-time iterator tuples.**
3. **NEVER: Run full-table analytical scans or aggregations on transactional master primary nodes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design hybrid transactional/analytical processing (HTAP) architecture with physical read/write segregation. Use columnar formats (Parquet, Arrow) for analytical batch pipelines.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Running heavy reporting aggregations directly on production OLTP primary instances.**
- **Using row-oriented stores for scanning billions of analytical log records.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-bailis-database-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
