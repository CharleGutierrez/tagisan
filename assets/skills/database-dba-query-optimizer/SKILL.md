---
name: database-dba-query-optimizer
description: Production database query plan optimization, indexing strategy, and zero-downtime schema migrations. Enforces mandatory EXPLAIN ANALYZE checks, optimal index selection (B-Tree, GIN, BRIN), non-blocking DDL (CREATE INDEX CONCURRENTLY), and query rewrite discipline.
version: 1.0.0
tags:
  - database-dba
  - query-optimization
  - explain-analyze
  - index-tuning
  - postgres
  - zero-downtime-ddl
triggers:
  - dba-optimizer
  - query-optimization
  - explain-analyze
  - index-tuning
  - create-index-concurrently
  - zero-downtime-ddl
  - postgres-performance
  - slow-query
compatibility: ">=0.2.0"
---

# Database DBA & Query Plan Optimizer: Zero-Downtime & High Performance

## Purpose & Scope
Sub-optimal SQL queries, missing indexes, unindexed foreign keys, and blocking schema migrations are the leading causes of production database outages, table-lock pileups, and unbounded latency spikes.

This skill equips autonomous agents and backend engineers with rigorous DBA operational discipline. It enforces mandatory `EXPLAIN (ANALYZE, BUFFERS)` inspections, systematic index selection (B-Tree, GIN, BRIN, covering indexes), non-blocking DDL practices (`CREATE INDEX CONCURRENTLY`), and query rewrite optimizations.

---

## 1. Operational Invariants (DBA Discipline Protocol)

### Invariant 1: Mandatory EXPLAIN ANALYZE Verification
- **NEVER**: Ship a database query to production or alter schema definitions without examining the physical query execution plan (`EXPLAIN (ANALYZE, BUFFERS, COSTS, VERBOSE)`).
- **RED FLAGS**:
  - `Seq Scan` on tables with >10,000 rows.
  - `Sort Method: external merge Disk` (indicates insufficient `work_mem` or missing index).
  - High `Buffers: shared read` relative to `shared hit` (cold disk I/O thrashing).
  - Rows estimated vs. rows actual discrepancy > 10x (indicates stale planner statistics; requires `ANALYZE table_name`).

### Invariant 2: Optimal Index Architecture
- Match access patterns to specialized index structures:
  - **B-Tree**: Primary keys, unique constraints, exact lookups (`=`), and range queries (`<`, `<=`, `>`, `>=`, `BETWEEN`). Order composite keys by cardinality: `(most_selective_column, range_column)`.
  - **Covering Index (`INCLUDE`)**: Satisfy index-only scans without table heap lookups (e.g. `CREATE INDEX idx_orders_user ON orders (user_id) INCLUDE (total, status);`).
  - **Partial Indexes**: Index only active or relevant row subsets (e.g. `WHERE status = 'pending'`), saving RAM and write amplification.
  - **GIN / GiST**: Full-text search (`tsvector`), JSONB containment (`@>`), arrays, and geometric data.
  - **BRIN**: Large append-only time-series tables ordered physically on disk by timestamp.

### Invariant 3: Zero-Downtime Non-Blocking DDL
- **NEVER**: Execute blocking DDL on high-traffic production databases:
  - `CREATE INDEX` without `CONCURRENTLY` acquires `SHARE` locks, blocking all table writes.
  - `ALTER TABLE ... ADD COLUMN ... DEFAULT <expression>` in older engines rewrites tables under `ACCESS EXCLUSIVE` lock.
  - Foreign key additions without `NOT VALID` acquire exclusive table locks during full table validation.
- **MANDATORY**:
  1. Always set strict lock timeouts before running DDL:
     ```sql
     SET lock_timeout = '2s';
     ```
  2. Always use `CREATE INDEX CONCURRENTLY` or `DROP INDEX CONCURRENTLY`.
  3. Author multi-phase zero-downtime migrations: add column with nullable default -> backfill in batches -> add `NOT VALID` check constraint -> validate constraint asynchronously.

### Invariant 4: Query Rewrite & Hygiene Discipline
- **NEVER**: Use `SELECT *` in API hot paths or high-throughput queries. Select only required projected columns.
- **Unnest Correlated Subqueries**: Rewrite correlated `WHERE EXISTS (SELECT 1 ...)` or scalar subqueries into efficient `JOIN` / `LEFT JOIN` structures.
- **Pagination**: Replace high-offset `OFFSET 100000 LIMIT 20` (which scans and discards 100,000 rows) with Keyset / Cursor Pagination (`WHERE id > :last_seen_id ORDER BY id ASC LIMIT 20`).
- **CTE Optimization**: In PostgreSQL 12+, CTEs default to inline evaluation, but can be forced when needed: `WITH data AS NOT MATERIALIZED (...)`.

---

## 2. Production DBA Checklists & SQL Patterns

### Zero-Downtime Index Creation
```sql
-- Step 1: Set strict lock timeout to avoid queueing behind transactions
SET lock_timeout = '2s';

-- Step 2: Build index non-blockingly in background
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_active_email
ON users (email)
WHERE is_active = TRUE;
```

### Keyset Cursor Pagination Over Offset
```sql
-- ANTI-PATTERN: O(N) heap page reads
SELECT id, title, created_at FROM events ORDER BY created_at DESC OFFSET 50000 LIMIT 25;

-- PRODUCTION PATTERN: O(1) B-Tree seek
SELECT id, title, created_at FROM events 
WHERE created_at < :last_seen_created_at 
ORDER BY created_at DESC LIMIT 25;
```
