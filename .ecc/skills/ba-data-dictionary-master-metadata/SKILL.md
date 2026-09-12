---
name: ba-data-dictionary-master-metadata
description: "Data Dictionary & Enterprise Metadata Standards: Data element definitions, ISO 11179 naming conventions, permitted domain values, nullability, and single sources of truth."
triggers: ["data-dictionary-master-metadata", "data-dictionary", "dmbok", "iso-11179", "metadata-standards", "canonical-data-dictionary"]
---

# ba-data-dictionary-master-metadata
> Based on **DAMA-DMBOK2 & ISO/IEC 11179 - Data Management Association**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Data Element Standard: Name, Definition, Data Type, Precision, Valid Values Domain, Mandatory/Optional, Source of Record.**
2. **Canonical Single Definition: Each enterprise business element must have exactly one authoritative data definition shared across all systems.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a project DATA_DICTIONARY.md defining every column, data type, allowed enum values, nullability, and business rule constraints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Leaving column names cryptic or undocumented (e.g. `c_stat_cd`).**
- **Allowing conflicting data types for the same concept across different tables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "data-dictionary-master-metadata"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
