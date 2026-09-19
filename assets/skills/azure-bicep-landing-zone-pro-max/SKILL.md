---
name: azure-bicep-landing-zone-pro-max
description: Autonomous Master Engine for Azure Bicep, Azure Verified Modules (AVM), Enterprise-Scale Landing Zones (ALZ), and Resource Graph FinOps governance. Covers parameterized IaC modules, Azure Policy as Code, private networking, and cost optimization. Triggers: bicep, landing-zone, avm, azure-policy, resource-graph, finops, arm-template.
version: 1.0.0
tags:
  - bicep
  - landing-zone
  - azure-policy
  - finops
  - iac
compatibility: ">=0.2.0"
---

# Azure Bicep & Enterprise Landing Zone Pro Max

## Purpose & Scope
The `azure-bicep-landing-zone-pro-max` skill provides comprehensive Infrastructure as Code (IaC) generation, validation, and cost governance using Azure Bicep and Azure Verified Modules.

## 1. Core Architectural Pillars

### Pillar 1: Modular Azure Bicep Design
- Author reusable Bicep modules adhering to Azure Verified Modules (AVM) standards.
- Use strongly typed parameters with user-defined types (`type myType = { ... }`), decorators (`@description`, `@allowed`), and default expressions.
- Secure secrets using `@secure()` parameters backed by Key Vault references.

### Pillar 2: Enterprise-Scale Landing Zones (ALZ)
- Structure subscriptions across Management Group hierarchies: `Root -> Platform (Management, Connectivity, Identity) -> Landing Zones (Online, Corp) -> Decommissioned / Sandbox`.
- Enforce hub-and-spoke virtual networking, Azure Firewall, and central Log Analytics workspaces.

### Pillar 3: Azure Policy as Code
- Implement custom Azure Policy definitions and initiatives in Bicep/JSON.
- Apply `Deny` and `Modify` effects for tagging governance, location restrictions, and disabling public network access on PaaS resources.

### Pillar 4: Azure Resource Graph (ARG) & FinOps Optimization
- Execute KQL-powered ARG queries to detect:
  - Unattached Managed Disks and orphan Public IPs.
  - Overprovisioned compute instances with <5% average CPU.
  - Unused App Service Plans and idle Cosmos DB throughput.

## 2. Strict Invariants
1. **Zero Hardcoded Secrets**: All credentials must be sourced from Key Vault or Managed Identities.
2. **Private Endpoints Required**: All database, storage, and key vault deployments must disable public ingress.
