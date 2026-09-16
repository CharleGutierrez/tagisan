---
name: copilot-sap-s4hana-odata-connector
description: "Integrating SAP S/4HANA OData services (BAPI / CDS views) via Graph Connector and OpenAPI."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "SAP S/4HANA OData Integration with Microsoft 365 - Bjarne Berg"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sap-odata-copilot", "s4hana-connector", "bapi-cds-views", "sap-enterprise-integration"]
---

# copilot-sap-s4hana-odata-connector
> Based on **SAP S/4HANA OData Integration with Microsoft 365 - Bjarne Berg** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OData v4 Protocol: Connect to SAP NetWeaver / S/4HANA OData services via Secure Agent or Azure API Management.**
2. **Principal Propagation: Propagate Entra ID user identity to SAP backend via SAML/OAuth2 bearer assertion.**
3. **ALWAYS validate SAP transaction locks and commit scopes before mutation.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sap-s4hana-odata-connector.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-sap-s4hana-odata-connector actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Connect Microsoft Copilot to SAP S/4HANA systems, enabling users to query purchase orders, inventory, and invoices.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sap-s4hana-odata-connector.**
- **Unmonitored runtime execution without telemetry in copilot-sap-s4hana-odata-connector.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sap-odata-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
