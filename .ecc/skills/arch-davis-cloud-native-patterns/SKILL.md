---
name: arch-davis-cloud-native-patterns
description: "Cloud-native architectural invariants: Stateless application nodes, declarative configuration, ephemeral compute, distributed coordination, and event-driven choreography."
triggers: ["cornelia-davis", "cloud-native-patterns", "stateless-services", "ephemeral-compute", "declarative-configuration", "twelve-factor", "change-tolerant"]
---

# arch-davis-cloud-native-patterns
> Based on **Cloud Native Patterns: Designing change-tolerant software - Cornelia Davis**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Application compute containers must be strictly stateless; all durable state is stored in external distributed datastores.**
2. **ALWAYS: Software instances must be designed for instant termination (SIGTERM shutdown drain) and zero-downtime rolling upgrades.**
3. **NEVER: Store user session state or uploaded files on the container's local ephemeral filesystem.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Adhere strictly to Twelve-Factor methodology. Externalize configuration via environment variables. Design nodes for arbitrary kill/restart lifecycle.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing uploaded files to local disk paths that disappear on container restart.**
- **Maintaining in-memory session caches that break under multi-instance horizontal scaling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-davis-cloud-native-patterns"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
