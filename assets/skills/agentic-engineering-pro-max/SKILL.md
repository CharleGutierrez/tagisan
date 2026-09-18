---
name: agentic-engineering-pro-max
description: Autonomous Master Engine for the Top 5,500 Agentic Engineering Skills found across GitHub. Enforces deterministic CodeAct REPL grounding, Model Context Protocol (MCP) tool contracts, AST semantic call graph indexing, autonomous RCA traceback healing, formal verification with Z3/Kani, and multi-agent swarm orchestration. Triggers: agentic, skills, mcp, codeact, repl, ast, rca, traceback healing, formal verification, guardrails, swarm, graphrag, devops agents.
version: 1.0.0
tags:
  - agentic
  - codeact
  - mcp
  - ast
  - rca
  - formal-verification
  - guardrails
  - swarm
  - graphrag
compatibility: ">=0.2.0"
---

# Agentic Engineering Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Modern autonomous agents require more than speculative natural language generation—they require **deterministic grounding, contract-driven tool execution, and self-verifying operational invariants**.

The `agentic-engineering-pro-max` skill codifies the **5,500 Agentic Engineering Skills** discovered across the global open-source AI frontier on GitHub (Anthropic Cookbook, MCP, LangGraph, AutoGen, CrewAI, SWE-agent, OpenHands, DSPy, and Kani). It provides Tagisan with an industrial-grade catalog, sub-millisecond search index, and CLI automation via `tgs agentic`.

---

## 1. The 15 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Domain Name | Skills | Benchmark Provenance | Core Operational Invariant |
|---|---|---|---|---|
| `AGT-01` | **Core Agentic Architecture & Meta-Cognition** | 500 | `anthropics/anthropic-cookbook`, `langchain-ai/langgraph` | ReAct, ToT/GoT, Reflexion loops, token budget circuit breakers |
| `AGT-02` | **Deterministic CodeAct & Polyglot REPLs** | 450 | `xingyaoww/codeact`, `huggingface/smolagents` | Zero in-weights math; all arithmetic & date deltas executed via REPL |
| `AGT-03` | **Model Context Protocol (MCP) & Inter-Agent RPC** | 450 | `modelcontextprotocol/servers`, `modelcontextprotocol/specification` | Strict JSON-RPC 2.0 stdio/SSE/WS, dynamic tool binding, resource streaming |
| `AGT-04` | **AST, Semantic Code Graphs & Program Analysis** | 450 | `tree-sitter/tree-sitter`, `sourcegraph/cody` | Tree-sitter AST queries, LSP symbol indexing, blast radius calculation |
| `AGT-05` | **Autonomous RCA, Debugging & Traceback Healing** | 400 | `microsoft/SWE-bench`, `rr-debugger/rr` | Stack frame deconstruction, deterministic replay, sanitizer bug repair |
| `AGT-06` | **Automated Software Testing & Formal Verification** | 400 | `proptest-rs/proptest`, `Z3Prover/z3`, `model-checking/kani` | TDD synthesis, property-based fuzzing, SMT constraint solving, Kani proofs |
| `AGT-07` | **Adversarial Security, Red-Teaming & Guardrails** | 400 | `protectai/rebuff`, `trufflesecurity/trufflehog` | Prompt injection interception, Shannon entropy secret gates, seccomp-bpf |
| `AGT-08` | **Memory Architectures, Knowledge Graphs & GraphRAG** | 400 | `noahshalo/reflexion`, `microsoft/graphrag` | Episodic case-law vaults, GraphRAG subgraphs, Reciprocal Rank Fusion |
| `AGT-09` | **DevOps, GitOps, SRE & Cloud Infrastructure** | 400 | `kubernetes/kubernetes`, `hashicorp/terraform` | Operator reconciliation, Terraform drift repair, canary rollback sentinels |
| `AGT-10` | **Data Engineering, Lakehouse & ETL Orchestration** | 350 | `dataform-co/dataform`, `dbt-labs/dbt-core`, `apache/iceberg` | Zero-copy catalog federation, SQLX DAGs, Spark & Dataflow streaming |
| `AGT-11` | **Machine Learning, Alignment & LLM Systems** | 350 | `vllm-project/vllm`, `huggingface/transformers` | DPO/RLHF alignment, KV cache optimization, LoRA adapters, AWQ quantization |
| `AGT-12` | **API Design, Distributed Systems & Microservices** | 350 | `grpc/grpc`, `open-telemetry/opentelemetry-rust` | OpenAPI 3.1 contracts, gRPC streaming, Raft consensus validation |
| `AGT-13` | **Low-Level Systems, Linux Kernel, eBPF & Firmware** | 250 | `iovisor/bcc`, `libbpf/libbpf`, `qemu/qemu` | XDP packet filtering, SPDK NVMe zero-copy, baremetal QEMU emulation |
| `AGT-14` | **Quantitative Finance, Algorithmic Trading & SCADA** | 200 | `quantopian/zipline`, `FreeOpcUa/opcua-asyncio` | FIX/ITCH protocols, Black-Scholes surfaces, OPC-UA industrial alarms |
| `AGT-15` | **Bioinformatics, Genomics & Scientific Computing** | 150 | `biopython/biopython`, `broadinstitute/gatk` | FASTA/FASTQ quality trimming, Smith-Waterman alignment, PDB coordinates |
| **TOTAL** | **15 Clusters** | **5,500** | **Global Open-Source Frontier** | **Deterministic Verification, Zero Hallucination, Autonomous Self-Correction** |

---

## 2. Core Invariant Rules

### Invariant 1: Computational Grounding (CodeAct Protocol)
- Never compute multi-digit multiplication, logarithms, compounding, or date deltas mentally in natural language.
- Always route computations through the sandboxed Python REPL (`python_eval`).

### Invariant 2: Standard Tool Interoperability (MCP Standard)
- All external capabilities must be exposed as Model Context Protocol (MCP) tools with validated JSON Schema parameters.

### Invariant 3: Verification Before Promotion
- Code modifications must undergo blast radius analysis, AST dependency validation, and unit test execution prior to merging or deployment.

### Invariant 4: Episodic Case-Law Retention
- Every resolved failure, traceback deconstruction, or user correction must be indexed into the episodic case-law vault to prevent compound regression.

---

## 3. CLI Commands & Usage

```bash
# 1. List skills across clusters
tgs agentic list --cluster agt-01 --limit 20

# 2. Search all 5,500 skills with keyword or semantic query
tgs agentic search "prompt injection defense"

# 3. Retrieve specific skill definition by ID
tgs agentic get AGT-03-001 --format json

# 4. Show complete cluster breakdown and statistics
tgs agentic breakdown

# 5. Export skill catalog to Markdown or JSON
tgs agentic export --cluster agt-06 --format markdown --output ./tests-catalog.md
```
