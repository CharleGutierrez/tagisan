---
name: copilot-rapid-feedback-loops-m365
description: "Sub-second feedback loops with hot-reloading Teams dev tunnels and mock Graph responses."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Extreme Programming Explained: AI Pair Programming - Kent Beck"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["rapid-feedback-loops", "teams-dev-tunnels", "hot-reload-m365", "sub-second-dev-loops"]
---

# copilot-rapid-feedback-loops-m365
> Based on **Extreme Programming Explained: AI Pair Programming - Kent Beck** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Feedback Cadence: Developer changes MUST reflect in the active running application in < 2 seconds.**
2. **Dev Tunnels: Utilize Microsoft Dev Tunnels for secure, low-latency webhook tunneling to localhost.**
3. **ALWAYS provide mock Graph data fixtures for instant offline test iteration.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-rapid-feedback-loops-m365.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-rapid-feedback-loops-m365 actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Set up ultra-fast local development loops with Teams Dev Tunnels and hot-reloading watchers for rapid prototyping.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-rapid-feedback-loops-m365.**
- **Unmonitored runtime execution without telemetry in copilot-rapid-feedback-loops-m365.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "rapid-feedback-loops"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
