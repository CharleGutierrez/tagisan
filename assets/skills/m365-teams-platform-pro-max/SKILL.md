---
name: m365-teams-platform-pro-max
description: Autonomous Master Engine for Microsoft 365, Teams Apps, Microsoft Graph API, and SPFx. Covers Graph batching, delta query synchronization, rate limiting & throttling mitigation, Teams AI Library, and Adaptive Card Universal Actions. Triggers: m365, teams, graph-api, spfx, teams-toolkit, adaptive-cards, delta-query, graph-throttling.
version: 1.0.0
tags:
  - m365
  - teams
  - graph-api
  - spfx
  - adaptive-cards
compatibility: ">=0.2.0"
---

# Microsoft 365 & Teams Platform Engineering Pro Max

## Purpose & Scope
The `m365-teams-platform-pro-max` skill provides comprehensive development and reliability patterns for Microsoft 365 applications, Teams bots, and Microsoft Graph integrations.

## 1. Core Architectural Pillars

### Pillar 1: Microsoft Graph API Throttling & Resilience
- **Mitigate HTTP 429**: Parse `Retry-After` headers and apply exponential backoff with full jitter.
- **JSON Batch Requests**: Combine up to 20 individual requests into a single `/v1.0/$batch` POST call to minimize roundtrips and conserve rate limits.
- **Delta Query Engine**: Use `@odata.deltaLink` and `@odata.nextLink` for efficient, real-time incremental state replication.

### Pillar 2: Microsoft Teams Bots & Teams AI Library
- Build conversational agents using the Teams AI Library and Bot Framework SDK.
- Manage conversation state, turn context, and multi-turn dialogs.
- Implement Adaptive Card Universal Actions (`Action.Execute`) with refreshed card payloads.

### Pillar 3: SharePoint Framework (SPFx)
- Author modern SPFx Web Parts and Application Customizers using TypeScript, React, and Fluent UI.
- Secure communication to Microsoft Graph and enterprise REST APIs via `AadHttpClient`.

### Pillar 4: Webhooks & Change Notifications
- Subscribe to Graph change notifications via Azure Event Hubs or HTTPS webhooks with validation handshake verification.

## 2. Strict Invariants
1. **Never Ignore Retry-After**: Bypassing HTTP 429 backoff headers leads to progressive IP and tenant bans.
2. **Validate Webhook Tokens**: Verify client state tokens on all incoming Graph webhook notifications before processing payloads.
