---
name: arch-campbell-database-reliability-engineering
description: "Database SRE operations: Zero-downtime schema migrations (Expand-Contract pattern), replication topology health, backup verification, RPO/RTO metrics, and capacity planning."
triggers: ["database-reliability", "expand-contract-migration", "zero-downtime-schema", "rpo-rto", "backup-verification", "replication-lag-monitoring"]
---

# arch-campbell-database-reliability-engineering
> Based on **Database Reliability Engineering - Laine Campbell & Charity Majors**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Execute database schema migrations using the Expand-and-Contract (Parallel Run) pattern across multiple releases to eliminate table locks and downtime.**
2. **ALWAYS: Verify database backups through automated scheduled restore drills into isolated test environments; unverified backups are considered non-existent.**
3. **NEVER: Execute raw `ALTER TABLE` operations on multi-million row tables that acquire exclusive DDL locks during peak production traffic.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design all schema changes to be backward-compatible (Expand -> Migrate Data -> Contract). Measure and alert on Replication Lag and RPO/RTO targets. Automate backup restore validation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Running locking DDL migrations that lock production tables for minutes or hours.**
- **Assuming backups work without ever performing automated restore drills.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-campbell-database-reliability-engineering"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
