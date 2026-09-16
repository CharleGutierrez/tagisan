---
name: pp-fx-in-memory-caching-collections
description: "Collect(), ClearCollect(), Patch(), bulk in-memory table mutations, local caching, and optimistic UI synchronization."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["in-memory-caching-collections", "clearcollect-patch-bulk", "optimistic-ui-updates", "local-collection-cache"]
---

# pp-fx-in-memory-caching-collections

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Collection Architecture: In-memory collections provide microsecond-level responsiveness and temporary scratchpad state before server commits.
- Sync Invariant: ALWAYS maintain synchronization between in-memory cache and server-side Dataverse state following mutation.
- NEVER store more than 10,000 records in client RAM to avoid browser tab memory exhaustion.
- MANDATORY use of `Patch(DataSource, Defaults(DataSource), colStagedRecords)` for high-speed bulk record creation.
- Source Reference: *In-Memory Data Engineering in Canvas Apps - Sancho Harker*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Implement optimistic UI: 1) Update local collection immediately: `Patch(colTasks, LookUp(colTasks, ID = selectedId), { Status: 'Completed' })`. 2) Trigger server sync asynchronously in background: `Patch(Tasks, LookUp(Tasks, ID = selectedId), { Status: 'Completed' })`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Reloading entire 2,000 record collections from the server after modifying a single boolean toggle.
- Mutating collection schemas dynamically mid-session, corrupting gallery data bindings.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("in-memory-caching-collections", "clearcollect-patch-bulk", "optimistic-ui-updates", "local-collection-cache") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
