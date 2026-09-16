---
name: pp-canvas-deep-linking-param-routing
description: "Param('recordId') ingestion, App.StartScreen routing logic, automated entity hydration, and secure cross-app navigational links."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-1"]
triggers: ["deep-linking-param-routing", "startscreen-power-fx", "param-recordid-routing", "canvas-url-navigation"]
---

# pp-canvas-deep-linking-param-routing

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Routing Architecture: App.StartScreen evaluates URL parameters (`Param('screen')`, `Param('id')`) before any visual tree renders.
- Navigation Invariant: ALWAYS validate and sanitize Param() inputs against authorization filters before routing to target screens.
- NEVER navigate users to unauthorized detail screens if GUID parameter is forged or invalid.
- MANDATORY fallback to default home screen when URL parameters are missing or malformed.
- Source Reference: *Enterprise URL Routing and Deep Linking in Power Apps - Reza Dorrani*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
In App.StartScreen, specify: `If(!IsBlank(Param('recordId')), DetailScreen, HomeScreen)`. On DetailScreen.OnVisible, fetch record: `Set(CurrentItem, LookUp(Accounts, Account = GUID(Param('recordId'))))`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Using Navigate() inside App.OnStart (now deprecated and causes runtime warnings/flicker) instead of App.StartScreen.
- Assuming Param('id') always contains a valid GUID without using IsType or Try/Catch validation.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("deep-linking-param-routing", "startscreen-power-fx", "param-recordid-routing", "canvas-url-navigation") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
