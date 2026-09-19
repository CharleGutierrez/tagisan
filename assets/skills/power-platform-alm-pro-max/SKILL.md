---
name: power-platform-alm-pro-max
description: Autonomous Master Engine for Microsoft Power Platform & Dataverse ALM. Covers Canvas & Model-Driven Apps, Power Apps Component Framework (PCF) with React 18 & Fluent UI v9, Dataverse solution packaging and deployment pipelines, C# plugin execution stages, Power Automate cloud flow optimization, and TDS endpoint querying. Triggers: power-platform, power-apps, dataverse, power-automate, pac-cli, pcf, canvas-app, solution-packager, model-driven.
version: 1.0.0
tags:
  - power-platform
  - dataverse
  - power-automate
  - pcf
  - power-apps
compatibility: ">=0.2.0"
---

# Power Platform ALM & Dataverse Engineering Pro Max

## Purpose & Scope
The `power-platform-alm-pro-max` skill provides autonomous ALM, pro-dev/citizen fusion architecture, and code inspection across Microsoft Power Platform and Microsoft Dataverse.

## 1. Core Architectural Pillars

### Pillar 1: Dataverse Solution Lifecycle & Packager
- **Solution Unpacking/Packing**: Use Microsoft Power Platform CLI (`pac solution unpack` / `pack`) to map raw `.zip` solutions into git-versioned source files.
- **Managed vs. Unmanaged Boundaries**: Development occurs strictly in Unmanaged solutions; production builds compile into Managed solutions with deterministic publisher prefixes.
- **Customizations XML Validation**: Inspect `customizations.xml` to prevent missing dependencies, schema clashes, or corrupted form XML definitions.

### Pillar 2: PCF (Power Apps Component Framework) Controls
- **Modern UI Stack**: Author PCF controls using React 18, TypeScript, and Fluent UI v9.
- **Control Manifest (`ControlManifest.Input.xml`)**: Specify bound and input properties, dataset definitions, external resources, and feature usages (e.g. WebAPI, Device).
- **Zero Memory Leaks**: Ensure all event subscriptions and timers are cleaned up in `destroy()`.

### Pillar 3: C# Dataverse Plugin Architecture
- **Pipeline Execution Stages**:
  - `Pre-Validation (Stage 10)`: Outside database transaction.
  - `Pre-Operation (Stage 20)`: Inside transaction, before write to DB. Ideal for setting default values.
  - `Post-Operation (Stage 40)`: Inside transaction, after write. Ideal for related entity creation.
- **Performance & Sandbox Invariants**:
  - Execution time must never exceed 2 minutes (hard sandbox ceiling); design hot paths under 2 seconds.
  - Avoid thread blocking, external unauthenticated HTTP calls, or statics that retain state across requests.
  - Always check `context.Depth <= 1` to prevent infinite recursion cascades.

### Pillar 4: Power Automate Cloud Flow Optimization
- **Concurrency Control**: Configure trigger concurrency limits for high-throughput webhook events.
- **Error Handling & Resiliency**: Enforce `runAfter` scopes with dedicated catch blocks for all HTTP, Dataverse, and connector actions.
- **Child Flows**: Break monolithic flows (>50 actions) into reusable, authenticated Child Flows with connection references.

## 2. Strict Invariants
1. **Never Hardcode Environment URLs or IDs**: Always use Environment Variables and Connection References.
2. **Deterministic Primary Keys**: In Dataverse, always model alternate keys for external system integrations to enable upsert semantics.
3. **Paging Integrity**: When querying Dataverse via Web API or FetchXML, always respect `@odata.nextLink` or `paging-cookie`.
