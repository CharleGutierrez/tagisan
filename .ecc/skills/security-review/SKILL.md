---
name: security-review
description: Offensive and defensive security checklist for vulnerability and threat detection.
---

# Security Review Skill

## Philosophy
Assume all external input is untrusted and actively malicious.

## Checks
1. Validate command arguments against injection attacks.
2. Ensure path traversal checks prevent access to sensitive files.
3. Validate memory bounds and avoid unbounded allocation buffers.
4. Verify authentication tokens and secrets are never echoed to standard out.
