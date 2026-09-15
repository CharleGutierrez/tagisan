---
name: agentic-henning-rpc-schema-contracts
description: "Interface Definition Languages (IDL), serialization efficiency, binary schemas, backwards compatibility, and strongly typed RPC contracts."
triggers: ["henning", "vinoski", "rpc-contracts", "protocol-buffers", "interface-definition-language", "binary-serialization", "schema-evolution"]
---

# agentic-henning-rpc-schema-contracts
> Based on **Advanced CORBA / Modern RPC & Protocol Buffers - Michi Henning & Steve Vinoski**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **IDL Contract Primacy: The schema is the single source of truth; client and server stubs are mechanically generated from the IDL.**
2. **Binary Wire Efficiency: Protocol Buffers / Cap'n Proto binary serialization delivers order-of-magnitude faster throughput and smaller footprints than JSON.**
3. **Tag-Based Backwards Compatibility: Fields are identified by field numbers/tags; unknown fields are preserved, enabling zero-downtime schema evolution.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use strongly typed schemas (Protocol Buffers, Cap'n Proto, or JSON Schema) for all high-throughput agent-to-agent and tool communications.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Passing loosely typed, undocumented JSON dictionaries between distributed agent services.**
- **Changing field IDs or deleting fields in active schemas without migration paths.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "henning"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
