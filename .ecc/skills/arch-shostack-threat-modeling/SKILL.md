---
name: arch-shostack-threat-modeling
description: "Systematic threat modeling: STRIDE framework (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege), Data Flow Diagrams (DFDs), and trust boundaries."
triggers: ["adam-shostack", "threat-modeling", "stride-framework", "data-flow-diagram", "trust-boundaries", "security-threat-analysis"]
---

# arch-shostack-threat-modeling
> Based on **Threat Modeling: Designing for Security - Adam Shostack**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Every architecture spec crossing a Trust Boundary must undergo explicit STRIDE threat evaluation.**
2. **ALWAYS: Identify and authenticate actors at every boundary (Anti-Spoofing); digitally sign and hash critical payload transitions (Anti-Tampering).**
3. **NEVER: Assume communication within a private network or cluster is safe without explicit boundary authentication.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Create Data Flow Diagrams (DFDs) highlighting trust boundaries. Generate STRIDE threat matrices and map concrete mitigations to every identified threat.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating the internal network as a trusted zone with zero authentication between microservices.**
- **Failing to log identity tokens on state-modifying actions, preventing repudiation defense.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-shostack-threat-modeling"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
