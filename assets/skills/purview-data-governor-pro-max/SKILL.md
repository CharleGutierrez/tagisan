---
name: purview-data-governor-pro-max
description: Autonomous Master Engine for Microsoft Purview, Data Governance, Sensitivity Labels, and DLP Compliance. Covers sensitivity classification rules, RMS encryption, Unified Audit Log search, and end-to-end data lineage across multi-cloud estates. Triggers: purview, sensitivity-labels, dlp, data-governance, mip-sdk, rms, compliance, lineage.
version: 1.0.0
tags:
  - purview
  - sensitivity-labels
  - dlp
  - compliance
  - data-governance
compatibility: ">=0.2.0"
---

# Microsoft Purview & Data Governance Pro Max

## Purpose & Scope
The `purview-data-governor-pro-max` skill automates enterprise data security, sensitivity classification, DLP policy enforcement, and regulatory compliance across Microsoft 365, Azure, and multi-cloud data lakes.

## 1. Core Architectural Pillars

### Pillar 1: Sensitivity Labeling & RMS Protection
- Define sensitivity label hierarchies (`Public`, `Internal`, `Confidential`, `Highly Confidential`).
- Configure Rights Management Services (RMS) encryption with granular usage rights (VIEW, EDIT, PRINT, FORWARD).
- Handle protected `.pfile` envelopes and encrypted Office OpenXML files safely.

### Pillar 2: Data Loss Prevention (DLP)
- Author DLP policies preventing the unauthorized sharing of sensitive info types (PII, PCI-DSS, HIPAA, secret keys).
- Enforce blocking rules with policy tips across Exchange, Teams, OneDrive, and SharePoint.

### Pillar 3: End-to-End Lineage & Cataloging
- Track automated data lineage from raw SQL/Dataverse ingestion through Azure Data Factory / Fabric pipelines to downstream Power BI datasets.
- Manage enterprise business glossaries and custom classification classifiers.

### Pillar 4: Audit & Compliance Search
- Automate searches across the Microsoft 365 Unified Audit Log (UAL) to investigate data access anomalies, mass downloads, or permission escalations.

## 2. Strict Invariants
1. **Strict Egress Protection**: Documents labeled "Highly Confidential" must never be routed through unencrypted or unmonitored external LLM APIs.
2. **Immutable Audit Trails**: All data access and classification events must be logged to a tamper-proof repository.
