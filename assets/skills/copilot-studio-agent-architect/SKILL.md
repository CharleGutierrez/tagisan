---
name: copilot-studio-agent-architect
description: Autonomous Master Engine for Microsoft Copilot Studio & Microsoft 365 Copilot Declarative Agents. Covers declarative agent manifest authoring, API plugins with OpenAPI 3.0 specs, Adaptive Cards 1.5 templating, enterprise grounding with SharePoint/Dataverse, and On-Behalf-Of (OBO) token exchange. Triggers: copilot-studio, declarative-agent, m365-copilot, teams-plugin, adaptive-cards, api-plugin, conversational-boosting.
version: 1.0.0
tags:
  - copilot-studio
  - declarative-agent
  - m365-copilot
  - adaptive-cards
  - teams
compatibility: ">=0.2.0"
---

# Microsoft Copilot Studio & Declarative Agent Architect

## Purpose & Scope
The `copilot-studio-agent-architect` skill automates the creation, validation, and packaging of enterprise AI agents for Microsoft Copilot Studio and Microsoft 365 Copilot.

## 1. Core Architectural Pillars

### Pillar 1: Declarative Agent Manifest (`declarativeAgent.json`)
- Author schema-compliant manifests with version `v1.0` or `v1.1`.
- Configure `instructions` with unambiguous system prompts, tone, tool prioritization, and safety guardrails.
- Bind enterprise capabilities:
  - `OneDriveAndSharePoint`: Define exact site URLs and folder paths.
  - `MicrosoftGraphConnectors`: Ground on ingested enterprise index items.

### Pillar 2: API Plugins & OpenAPI 3.0 Optimization
- **Plugin Manifest (`plugin.json`)**: Specify auth schemes (OAuth2, API Key) and runtime endpoints.
- **OpenAPI Schema Tuning**:
  - Annotate state-mutating operations with `"x-openai-isConsequential": true`.
  - Provide rich, descriptive summaries for every operation and parameter to enable accurate LLM function calling.
  - Include precise response models with sample schemas.

### Pillar 3: Adaptive Cards 1.5 UX Engineering
- Create interactive cards with `Action.Execute` and verb dispatching.
- Implement data binding expressions (`${propertyName}`) and conditional visibility.
- Support dark/light enterprise themes and mobile responsiveness.

### Pillar 4: On-Behalf-Of (OBO) Flow & Security
- Exchange M365 user tokens for downstream API tokens securely.
- Respect Purview sensitivity classifications on all grounded search responses.

## 2. Strict Invariants
1. **Explicit Confirmation on Consequential Actions**: Any action modifying corporate data (deletes, updates, payments) must present an Adaptive Card confirmation modal.
2. **Grounded Citations**: AI responses must contain hyperlinked references to source documents.
