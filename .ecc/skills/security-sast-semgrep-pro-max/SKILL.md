---
name: security-sast-semgrep-pro-max
description: Autonomous Master Engine for Static Application Security Testing (SAST), High-Entropy Secret Scanning, OWASP Top 10 Auditing, Dependency Vulnerability Analysis, and Semgrep Orchestration. Enforces zero-credential leakage, Shannon entropy evaluation, injection pattern detection, supply chain lockfile auditing, and native offline static analysis across Windows, Linux, and macOS. Triggers: sast, security-scan, secret-scanner, vulnerability-scan, semgrep, owasp, dependency-audit, security-sast, cve-audit, injection-defense, sast-pro-max.
version: 1.0.0
tags:
  - sast
  - security
  - secret-scanning
  - semgrep
  - owasp
  - dependencies
  - cve
  - shannon-entropy
  - zero-leakage
compatibility: ">=0.2.0"
---

# Security SAST & Semgrep Pro Max: Autonomous Static Analysis & Credential Protection Engine

## Purpose & Scope
Autonomous agents generating code, refactoring architectures, or integrating third-party dependencies must enforce rigorous security safeguards. Inadvertently committing hardcoded API keys, generating SQL/command injection vulnerabilities, or importing outdated libraries with known CVEs introduces catastrophic security risks into production systems.

The `security-sast-semgrep-pro-max` skill establishes Tagisan's authoritative static security analysis and credential protection architecture. It codifies the 6 architectural pillars governing high-entropy secret scanning, semantic vulnerability pattern detection, software bill of materials (SBOM) and dependency auditing, OWASP Top 10 compliance validation, Semgrep CLI orchestration, and resilient offline static rule evaluation.

---

## Pillar SAST-01: High-Entropy Credential & Secret Scanning

### 1. Multi-Pattern Secret Signatures
The `scan_secrets` action scans workspace source trees against high-fidelity credential patterns:
- **Cloud Provider Keys**:
  - AWS Access Key ID (`\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b`) and Secret Keys
  - Google Cloud API Keys (`\bAIza[0-9A-Za-z\\-_]{35}\b`)
  - Azure Shared Access Signatures and Management Keys
- **VCS & Platform Tokens**:
  - GitHub Personal Access Tokens (`ghp_[A-Za-z0-9_]{36,}`, `github_pat_...`)
  - GitLab Personal Access Tokens (`glpat-...`)
  - Slack Bot/User Tokens (`xox[baprs]-...`), Discord Bot Tokens
- **Private Cryptographic Keys**:
  - PEM/OpenSSH headers (`-----BEGIN RSA PRIVATE KEY-----`, `-----BEGIN OPENSSH PRIVATE KEY-----`, `-----BEGIN EC PRIVATE KEY-----`)
- **Database Connection Strings**:
  - Uniform Resource Identifiers containing embedded passwords (`postgres://user:pass@host`, `mysql://...`, `mongodb+srv://...`)

### 2. Shannon Information Entropy Evaluation
Detects obfuscated and bespoke API keys lacking standard prefixes:
- Calculates Shannon entropy: $H(X) = -\sum_{i=1}^n P(x_i) \log_2 P(x_i)$.
- Evaluates candidate tokens $\ge 20$ characters in assignment contexts (`api_key = "..."`, `secret = "..."`).
- Flags strings with $H \ge 4.5$ bits/symbol as high-probability cryptographic secrets.

### 3. Redaction & Zero-Leakage Invariant
All detected secret payloads are masked (`AKIA****************`) prior to emission into LLM context, preventing secondary credential leakage into logs or model prompts.

---

## Pillar SAST-02: Code Vulnerability & Injection Pattern Detection

### 1. Semantic Anti-Pattern Rules
The `scan_vulnerabilities` action scans source files for dangerous coding patterns:
- **Injection Flaws (CWE-89, CWE-78)**:
  - Raw string concatenation in database queries (`"SELECT * FROM users WHERE id = " + id`).
  - Unsanitized shell invocations (`Command::new("sh").arg("-c").arg(user_input)`, `exec(user_input)`).
- **Path Traversal (CWE-22)**:
  - Direct file access using unsanitized user path variables without canonicalization.
- **Insecure Cryptography (CWE-327, CWE-328)**:
  - Use of MD5, SHA1 for password hashing, DES, or ECB cipher modes.
- **SSRF & Unvalidated Redirects (CWE-918, CWE-601)**:
  - HTTP requests initiated to unvalidated user-supplied URLs.

---

## Pillar SAST-03: Dependency Lockfile & Supply Chain Auditing

### 1. Cross-Ecosystem Lockfile Mining
The `audit_dependencies` action inspects repository lockfiles:
- **Rust**: `Cargo.lock` — checks for yanked crates and known RustSec advisory patterns.
- **Node.js**: `package-lock.json`, `pnpm-lock.yaml`, `bun.lockb` / `bun.lock` — checks dependencies and runs `npm audit` / `bun pm audit` if available.
- **Python**: `requirements.txt`, `poetry.lock`, `Pipfile.lock` — identifies unpinned dependencies (`package>=0.1` without ceiling) and vulnerable packages.

---

## Pillar SAST-04: OWASP Top 10 Compliance Verification

### 1. Categorical Risk Posture Assessment
The `owasp_check` action maps findings across the OWASP Top 10 categories:
- A01: Broken Access Control
- A02: Cryptographic Failures
- A03: Injection
- A04: Insecure Design
- A05: Security Misconfiguration
- A07: Identification & Authentication Failures
- A08: Software & Data Integrity Failures
- A09: Security Logging & Monitoring Failures
- A10: Server-Side Request Forgery (SSRF)
- Outputs an overall security risk score ($0 - 100$) and itemized remediation recommendations.

---

## Pillar SAST-05: Semgrep CLI Orchestration & Native Rule Fallback

### 1. Standard Semgrep Integration
- If `semgrep` CLI is present in host PATH, executes `semgrep scan --json --quiet` with standard security rulesets.
- If `semgrep` is not installed, executes Tagisan's native static analysis rule engine and formats findings into Semgrep-compatible SARIF/JSON structures, ensuring continuous tooling compatibility.

---

## Autonomous Tool-Use Protocols
1. **Pre-Commit Audit**: Prior to committing changes, agent calls `action: "scan_secrets"` and `action: "scan_vulnerabilities"`.
2. **Zero-Warning Gate**: If any critical secret or high-severity injection pattern is detected, commit operation is blocked until remediation is applied.
