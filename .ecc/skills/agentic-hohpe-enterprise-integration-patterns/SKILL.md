---
name: agentic-hohpe-enterprise-integration-patterns
description: "Message channels, pipes and filters, content-based router, scatter-gather, message translator, idempotent receiver, and pub/sub architectures."
triggers: ["hohpe", "woolf", "enterprise-integration", "pipes-and-filters", "content-based-router", "scatter-gather", "message-translator", "idempotent-receiver"]
---

# agentic-hohpe-enterprise-integration-patterns
> Based on **Enterprise Integration Patterns: Designing, Building, and Deploying Messaging Solutions - Gregor Hohpe & Bobby Woolf**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pipes and Filters Architecture: Decomposing complex data processing into independent, reusable processing stages connected by message pipes.**
2. **Content-Based Router: Inspecting message payload attributes to dynamically direct messages to the appropriate downstream agent specialist.**
3. **Scatter-Gather Pattern: Broadcasting a query to multiple agent workers and aggregating/ranking their responses into a single composite output.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design multi-agent processing pipelines using Enterprise Integration Patterns: Scatter-Gather for parallel research, Content-Based Routers for language dispatch.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Direct point-to-point spaghetti coupling between agents without message channels.**
- **Non-idempotent message consumers that corrupt state upon receiving duplicate delivery.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hohpe"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
