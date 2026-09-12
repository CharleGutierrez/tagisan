---
name: arch-fowler-production-ready-microservices
description: "Production readiness standards: Stability, reliability, scalability, fault tolerance, catastrophe preparedness, performance monitoring, security hardening, and documentation."
triggers: ["production-ready-microservices", "production-readiness-checklist", "stability-standards", "graceful-shutdown", "catastrophe-preparedness"]
---

# arch-fowler-production-ready-microservices
> Based on **Production-Ready Microservices - Susan J. Fowler**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Microservices must implement graceful shutdown handling: listen for SIGTERM, stop accepting new traffic, drain in-flight requests, and flush telemetry before exit.**
2. **ALWAYS: Validate every production microservice against an automated readiness checklist (SLOs, runbooks, dashboard links, backup procedures).**
3. **NEVER: Terminate a production container abruptly without a graceful connection drain period.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Fulfill all 8 pillars of the Production Readiness Review. Implement SIGTERM drain loops (e.g. 30s grace period). Publish automated service runbooks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Containers terminating instantly with SIGKILL, severing in-flight HTTP connections and dropping data.**
- **Deploying services with zero runbooks, monitoring dashboards, or on-call alerts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-fowler-production-ready-microservices"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
