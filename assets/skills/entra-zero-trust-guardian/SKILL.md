---
name: entra-zero-trust-guardian
description: Autonomous Master Engine for Microsoft Entra ID (Azure AD), Zero Trust Architecture, and Identity Governance. Covers Continuous Access Evaluation (CAE), Workload Identity Federation, Privileged Identity Management (PIM), MSAL token exchange, and least-privilege permission auditing. Triggers: entra, azure-ad, conditional-access, workload-identity, pim, msal, cae, zero-trust, managed-identity.
version: 1.0.0
tags:
  - entra
  - zero-trust
  - identity
  - msal
  - security
compatibility: ">=0.2.0"
---

# Microsoft Entra ID & Zero Trust Identity Guardian

## Purpose & Scope
The `entra-zero-trust-guardian` skill enforces cryptographic Zero Trust identity security, workload authentication, and role governance across enterprise Microsoft tenants.

## 1. Core Architectural Pillars

### Pillar 1: Workload Identity Federation (Secretless Auth)
- Eliminate long-lived client secrets.
- Configure OIDC trust between Microsoft Entra ID and external runners (GitHub Actions, Kubernetes service accounts).
- Map subject identifiers (`sub`), audiences (`aud`), and claims cleanly to user-assigned managed identities.

### Pillar 2: Continuous Access Evaluation (CAE)
- Implement CAE-ready token handling in applications.
- Intercept HTTP 401 with `WWW-Authenticate` bearing `error="insufficient_claims"` and re-authenticate with the requested `claims` challenge string.

### Pillar 3: Privileged Identity Management (PIM)
- Eliminate permanent administrative role assignments.
- Enforce eligible role assignments requiring multi-factor authentication, ticketing justification, and time-bounded activation (max 8 hours).

### Pillar 4: Least-Privilege App Registrations & Permissions Audit
- Identify and eliminate overly broad application permissions (e.g. `Directory.ReadWrite.All`, `Mail.ReadWrite`).
- Prefer delegated permissions with user consent or granular application roles.
- Rotate credentials and certificates automatically via Azure Key Vault.

## 2. Strict Invariants
1. **Never Commit Secrets**: Hardcoded client secrets or certificates trigger immediate security rejection.
2. **Enforce Conditional Access**: All interactive administrative logins must require phishing-resistant MFA (FIDO2 / Windows Hello for Business).
