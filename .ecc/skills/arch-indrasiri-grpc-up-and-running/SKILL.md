---
name: arch-indrasiri-grpc-up-and-running
description: "High-performance binary RPC: Protocol Buffers v3, Unary and Streaming RPCs (Client, Server, Bidirectional), Interceptors, Deadline Propagation, and Name Resolution."
triggers: ["grpc-architecture", "protobuf", "deadline-propagation", "grpc-interceptors", "bidirectional-streaming", "binary-rpc", "schema-evolution"]
---

# arch-indrasiri-grpc-up-and-running
> Based on **gRPC: Up and Running - Kasun Indrasiri & Danesh Kuruppu**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Propagate gRPC context deadlines across all downstream RPC invocations; terminate processing immediately when the deadline expires.**
2. **ALWAYS: Enforce backward and forward compatibility in `.proto` files: never alter existing field tag numbers or remove reserved fields.**
3. **NEVER: Pass unauthenticated gRPC requests across cluster boundaries without TLS and interceptor token validation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define all internal service APIs using Protocol Buffers with numbered fields. Propagate gRPC deadlines across all RPC hops. Implement auth and tracing interceptors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Renumbering or deleting fields in `.proto` files, breaking existing deployed clients.**
- **Continuing expensive downstream database queries after the client gRPC deadline has expired.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-indrasiri-grpc-up-and-running"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
