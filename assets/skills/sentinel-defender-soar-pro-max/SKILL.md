---
name: sentinel-defender-soar-pro-max
description: Autonomous Master Engine for Microsoft Sentinel, Microsoft Defender XDR, and SOAR Cyber Operations. Covers high-performance KQL threat hunting, ASIM normalization, analytic rule authoring, incident triage, and automated Logic Apps remediation playbooks. Triggers: sentinel, defender, kql, soar, threat-hunting, asim, soc, incident-response, xdr.
version: 1.0.0
tags:
  - sentinel
  - defender
  - kql
  - soar
  - cyber-defense
compatibility: ">=0.2.0"
---

# Microsoft Sentinel & Defender SOAR Pro Max

## Purpose & Scope
The `sentinel-defender-soar-pro-max` skill automates cloud SIEM/XDR threat detection, high-throughput KQL query optimization, and autonomous SOAR incident remediation.

## 1. Core Architectural Pillars

### Pillar 1: High-Performance KQL (Kusto Query Language)
- **Execution Efficiency**:
  - Always place time filters (`TimeGenerated >= ago(24h)`) and strict string filters at the earliest possible stage.
  - Use `where` with `has` or `has_cs` rather than `contains` to leverage indexed term tokens.
  - Optimize joins: Place the smaller dataset on the right side of `join`.
  - Use `summarize arg_max()` for entity state deduplication.

### Pillar 2: ASIM (Advanced Security Information Model)
- Write detection rules against ASIM parsers (e.g., `imAuthentication`, `imNetworkSession`, `imProcessEvent`).
- Ensure schema compliance across heterogeneous multi-cloud and on-premises log sources.

### Pillar 3: Defender XDR Unified Incident Investigation
- Query Microsoft Graph Security API and Advanced Hunting tables (`DeviceInfo`, `DeviceProcessEvents`, `IdentityLogonEvents`, `CloudAppEvents`).
- Map alerts to MITRE ATT&CK enterprise tactics and techniques.

### Pillar 4: Automated SOAR Incident Remediation
- Scaffolds Logic Apps and Azure Event Grid playbooks for:
  - Isolating compromised endpoints via Defender Live Response.
  - Revoking active Entra ID user refresh tokens (`Revoke-AzureADUserAllRefreshToken`).
  - Blocking malicious IP addresses on Azure Firewall and Cloudflare WAF.

## 2. Strict Invariants
1. **Bounded Time Windows**: Every KQL query must enforce an explicit `TimeGenerated` boundary.
2. **Safe Automated Containment**: Critical containment playbooks must support human-in-the-loop approval or automated revert safeguards.
