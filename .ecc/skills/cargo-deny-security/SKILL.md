---
name: cargo-deny-security
description: Supply chain crate security, license compliance, duplicate dependencies, and cargo-audit scanning
---

# Cargo Supply Chain Security & Dependency Hygiene

## Audit Guidelines
1. **Vulnerability Scanning**: Audit all transitive dependencies with `cargo audit` and RustSec Advisory Database.
2. **License Enforcement**: Reject non-permissive or ambiguous licenses (enforce MIT, Apache-2.0, BSD-3-Clause).
3. **Duplicate Elimination**: Detect and consolidate duplicate versions of common crates (`serde`, `tokio`, `syn`).
4. **Ban Lists**: Block unmaintained or unsound crates with known CVEs or memory safety violations.
