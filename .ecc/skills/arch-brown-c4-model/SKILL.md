---
name: arch-brown-c4-model
description: "Hierarchical architecture visualization: Level 1 (System Context), Level 2 (Containers), Level 3 (Components), Level 4 (Code), and diagramming as code."
triggers: ["c4-model", "simon-brown", "system-context", "containers-diagram", "components-diagram", "diagrams-as-code", "hierarchical-architecture"]
---

# arch-brown-c4-model
> Based on **The C4 Model for Software Architecture - Simon Brown**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Architecture specifications must provide explicit C4 Level 1 (System Context) and Level 2 (Containers with protocols/ports) before synthesizing implementation code.**
2. **ALWAYS: Every container and component box must specify its explicit technology, role, and interaction protocol (e.g. `gRPC / TLS`).**
3. **NEVER: Create monolithic unsegmented diagrams containing code-level details alongside cloud-level systems.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure system documentation using the 4 C4 levels: Context (users & software systems), Containers (applications & datastores), Components (modules), and Code (classes).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Skipping container boundaries and trying to explain entire distributed systems with a single diagram.**
- **Omitting protocols, ports, and technologies from architectural boxes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-brown-c4-model"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
