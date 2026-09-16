---
name: pp-m-unpivot-transpose-matrix-wrangling
description: "Table.UnpivotOtherColumns for reshaping cross-tabulated spreadsheets into tidy normalized tables."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-16"]
triggers: ["unpivot-transpose-matrix", "table-unpivotothercolumns", "tidy-data-wrangling-m"]
---

# pp-m-unpivot-transpose-matrix-wrangling

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Collect, Combine, and Transform Data Using Power Query in Excel and Power BI - Gil Raviv'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-m-unpivot-transpose-matrix-wrangling workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-m-unpivot-transpose-matrix-wrangling processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Collect, Combine, and Transform Data Using Power Query in Excel and Power BI - Gil Raviv*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-m-unpivot-transpose-matrix-wrangling protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-m-unpivot-transpose-matrix-wrangling directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-m-unpivot-transpose-matrix-wrangling execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("unpivot-transpose-matrix", "table-unpivotothercolumns", "tidy-data-wrangling-m") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
