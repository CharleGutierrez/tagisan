---
name: agentic-fielding-rest-agent-apis
description: "Statelessness, uniform interface, cacheability, layered systems, HATEOAS, and resource-oriented modeling for autonomous agent APIs."
triggers: ["fielding", "rest-architecture", "hateoas", "uniform-interface", "statelessness", "resource-oriented", "web-architecture"]
---

# agentic-fielding-rest-agent-apis
> Based on **Architectural Styles and the Design of Network-based Software Architectures (REST) - Roy Thomas Fielding**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Statelessness Invariant: Each request from client to server must contain all the information necessary to understand and process the request.**
2. **Uniform Interface (HATEOAS): Hypermedia as the Engine of Application State; clients transition through states via hypermedia links in responses.**
3. **Cacheability Constraint: Responses must explicitly define themselves as cacheable or non-cacheable to optimize network efficiency.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design agent-accessible web APIs using strict REST principles. Provide self-descriptive hypermedia links in API responses so agents can discover available actions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Maintaining hidden session state on servers that breaks client agent failover and scalability.**
- **Tunneling arbitrary non-idempotent operations through HTTP GET requests.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fielding"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
