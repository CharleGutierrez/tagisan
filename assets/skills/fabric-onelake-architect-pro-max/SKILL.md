---
name: fabric-onelake-architect-pro-max
description: Autonomous Master Engine for Microsoft Fabric, OneLake Medallion Architecture, Delta Lake, and Direct Lake Power BI. Covers Parquet compaction, V-Order optimization, Synapse Spark pipelines, Tabular Model Definition Language (TMDL), and DAX performance tuning. Triggers: fabric, onelake, delta-lake, direct-lake, synapse, dax, powerbi, tmdl, medallion.
version: 1.0.0
tags:
  - fabric
  - onelake
  - delta-lake
  - powerbi
  - dax
compatibility: ">=0.2.0"
---

# Microsoft Fabric & OneLake Architecture Pro Max

## Purpose & Scope
The `fabric-onelake-architect-pro-max` skill provides deep-tech data engineering guidance for Microsoft Fabric, OneLake, Delta Lake 3.x, and enterprise Power BI.

## 1. Core Architectural Pillars

### Pillar 1: OneLake Medallion Lakehouse Architecture
- **Bronze (Raw)**: Ingest external data unmodified in append-only Delta or Parquet format.
- **Silver (Cleaned & Conformed)**: Apply schema enforcement, deduplication, and referential integrity checks.
- **Gold (Curated Business Marts)**: Star schemas with dimensional modeling optimized for high-speed reporting.

### Pillar 2: Delta Lake Optimization & V-Order
- **V-Order Sorting**: Apply Microsoft proprietary in-memory columnar sorting to Parquet files during write time for lightning-fast Direct Lake queries.
- **Compaction & Liquid Clustering**: Use `OPTIMIZE` and `CLUSTER BY` on high-cardinality query keys rather than traditional rigid directory partitioning.
- **Vacuum Governance**: Run `VACUUM` with retention periods balancing time-travel recovery and storage efficiency.

### Pillar 3: Direct Lake Mode & Fallback Prevention
- Avoid Direct Lake fallback to DirectQuery:
  - Do not use complex calculated columns in Power BI; compute them upstream in Lakehouse Delta tables.
  - Ensure column data types match Direct Lake supported primitives.
  - Keep Delta table Parquet file count within recommended limits (< 1,000 files per table).

### Pillar 4: DAX Optimization & TMDL ALM
- Eliminate slow iterative DAX formulas (`FILTER` on large tables without index utilization).
- Author Power BI semantic models in TMDL (Tabular Model Definition Language) for clean Git diffs and branch reviews.

## 2. Strict Invariants
1. **No Duplicate Ingestion**: Always implement change-data-capture (CDC) or timestamp watermarking in Data Factory / Spark notebooks.
2. **Idempotent Pipelines**: All Lakehouse transformations must support safe re-runs without producing duplicate rows.
