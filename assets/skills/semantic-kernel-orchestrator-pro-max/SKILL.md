---
name: semantic-kernel-orchestrator-pro-max
description: Autonomous Master Engine for Microsoft Semantic Kernel (.NET & Python) and Azure AI Foundry. Covers native/prompt plugins, memory connectors, vector search indexing, Provisioned Throughput (PTU) failover routing, and multi-agent Process Framework workflows. Triggers: semantic-kernel, azure-openai, azure-ai-foundry, kernel-memory, prompt-flow, process-framework.
version: 1.0.0
tags:
  - semantic-kernel
  - azure-openai
  - azure-ai-foundry
  - vector-search
  - multi-agent
compatibility: ">=0.2.0"
---

# Microsoft Semantic Kernel & Azure AI Foundry Pro Max

## Purpose & Scope
The `semantic-kernel-orchestrator-pro-max` skill guides the development and orchestration of enterprise AI agents, native plugins, and multi-agent systems using Semantic Kernel (.NET & Python) and Azure AI Foundry.

## 1. Core Architectural Pillars

### Pillar 1: Semantic Kernel Plugin Architecture
- **Native Plugins**: Implement type-safe functions annotated with `[KernelFunction]` (C#) or `@kernel_function` (Python) and explicit `[Description]` attributes.
- **Schema Validation**: Define parameter types and descriptions to enable accurate model tool calling.
- **Filters & Hooks**: Use Invocation Filters to intercept function execution for prompt injection defense, PII masking, and latency telemetry.

### Pillar 2: Vector Stores & Memory Connectors
- Connect Semantic Kernel to Azure AI Search, Cosmos DB Mongo vCore, or pgvector.
- Build hybrid search pipelines combining dense vector embeddings (Text-Embedding-3-Large) with BM25 full-text keyword retrieval and semantic rerankers.

### Pillar 3: Azure OpenAI Provisioned Throughput (PTU) & Failover
- Configure intelligent fallback routing between Azure OpenAI PTU instances and Pay-As-You-Go endpoints across secondary regions during 429 throttling.
- Implement token-bucket rate limiters aligned with deployment quotas.

### Pillar 4: Process Framework & Multi-Agent Orchestration
- Build stateful, multi-agent workflows where specialized agents (Architect, Reviewer, Tester) collaborate with structured handoffs.
- Integrate human-in-the-loop review checkpoints before tool execution.

## 2. Strict Invariants
1. **Schema Integrity**: Every kernel function must have an unambiguous description; LLMs fail to call ambiguous tools.
2. **Content Safety Enforced**: All model invocations must integrate Azure AI Content Safety filters for hate, violence, self-harm, and sexual content.
