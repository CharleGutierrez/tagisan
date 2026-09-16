---
name: pp-fx-error-handling-iferror-isblank
description: "IfError(), IsError(), Notify(), App.OnError global error traps, and transactional safety in Dataverse Patch mutations."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-2"]
triggers: ["error-handling-iferror", "app-onerror-global-trap", "notify-error-banners", "defensive-patch-power-fx"]
---

# pp-fx-error-handling-iferror-isblank

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Defensive Evaluation: `IfError(TargetOperation, FallbackValue)` intercepts runtime faults (network timeouts, constraint violations) gracefully.
- Telemetry Invariant: ALWAYS implement `App.OnError` to log unhandled client-side runtime errors to Application Insights or Dataverse.
- NEVER suppress critical database write errors silently without notifying the end user.
- MANDATORY validation of `Errors(DataSource, Record)` after high-stakes Patch() calls.
- Source Reference: *Resilient Power Apps: Defensive Coding and Fault Tolerance - Shane Young*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
Wrap transactional updates defensively: `IfError(Patch(Orders, Defaults(Orders), { OrderNumber: '1001' }), Notify("Order creation failed: " & FirstError.Message, NotificationType.Error), Notify("Order submitted successfully!", NotificationType.Success))`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Ignoring return errors from Patch(), giving users the false impression that data was saved when the database rejected it.
- Using empty Notify() messages that leave users bewildered when network errors strike.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("error-handling-iferror", "app-onerror-global-trap", "notify-error-banners", "defensive-patch-power-fx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
