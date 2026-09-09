# 📌 Tagisan Architecture Blueprint & Implementation Roadmap (RFC-001)

**Status:** Proposed / Saved for Later Implementation  
**Target:** Tagisan (`tgs` / `tagisan-rs`)  
**Scope:** Universal Ecosystem Integrations, Enterprise Vector DB Backends, OpenTelemetry Observability, and Swarm-Based Automated Evaluation (`tgs eval`).

---

## 📑 Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [Module 1: Universal Protocol & Ecosystem Integrations](#2-module-1-universal-protocol--ecosystem-integrations)
3. [Module 2: Pluggable External Vector Databases (`src/memory/backend/`)](#3-module-2-pluggable-external-vector-databases)
4. [Module 3: Enterprise Observability & OpenTelemetry (`src/telemetry/`)](#4-module-3-enterprise-observability--opentelemetry)
5. [Module 4: Automated Swarm Evaluation Suite (`tgs eval`)](#5-module-4-automated-swarm-evaluation-suite-tgs-eval)
6. [Implementation Phases & Milestones](#6-implementation-phases--milestones)

---

## 1. Executive Summary

While Python-based frameworks (e.g., LangChain, AutoGen) rely on hundreds of fragile, individual wrapper packages, **Tagisan adopts a high-throughput, protocol-driven architecture in Rust**. This blueprint outlines how Tagisan will achieve parity and superiority across:
- **Integrations:** Utilizing the Model Context Protocol (MCP) and Extism WebAssembly (WASM) plugins.
- **Vector DBs:** Providing an asynchronous, pluggable `VectorStoreBackend` trait supporting Qdrant (gRPC), PostgreSQL `pgvector` (`sqlx`), and Pinecone/Weaviate.
- **Observability:** Implementing native OpenTelemetry (OTel) spans exported via OTLP gRPC to Langfuse, Arize Phoenix, Datadog, or Jaeger.
- **Evaluation:** Leveraging Tagisan's Dialectical Debate and Borda count consensus engine for automated model evaluation (`tgs eval`).

---

## 2. Module 1: Universal Protocol & Ecosystem Integrations

### Architectural Overview
Rather than maintaining thousands of brittle Python API wrappers:
1. **Model Context Protocol (MCP) Hub:**
   - Universal JSON-RPC 2.0 stdio/SSE client/server already present in Milestone 5 & 7.
   - Instantly consumes any open-source or commercial MCP server (GitHub, Slack, Jira, Postgres, Docker, Brave Search, etc.) with zero glue code.
   - All MCP tool calls are intercepted and filtered by `AgentShield`.

2. **Sandboxed WebAssembly (WASM) Plugin Engine:**
   - Integrated via `wasmtime` or `extism`.
   - Allows users to write custom tools in TypeScript, Go, Python, or C and compile them to `.wasm` binaries.
   - Enforces strict memory and CPU runtime quotas with isolated file access.

3. **High-Throughput Native Connectors:**
   - `sqlx`: PostgreSQL, MySQL, SQLite with compile-time checked queries.
   - `octocrab`: GitHub API integration.
   - `aws-sdk-rust` / `object_store`: S3, GCS, Azure Blob, MinIO.
   - `rdkafka`: Event stream ingestion and agent triggering.

---

## 3. Module 2: Pluggable External Vector Databases

### File Location: `src/memory/backend/mod.rs`

```rust
use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

#[async_trait]
pub trait VectorStoreBackend: Send + Sync {
    /// Name identifier of the backend (e.g. "qdrant", "pgvector", "local")
    fn name(&self) -> &str;

    /// Insert or upsert vectorized documents into a target collection
    async fn insert(&self, collection: &str, docs: &[VectorDocument]) -> Result<()>;

    /// Perform vector similarity search (cosine, dot product, or euclidean)
    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>>;

    /// Delete vectors by document ID or metadata filter
    async fn delete(&self, collection: &str, ids: &[String]) -> Result<()>;

    /// Health check and connection verification
    async fn ping(&self) -> Result<bool>;
}
```

### Planned Implementations:
1. **`QdrantStore` (`src/memory/backend/qdrant.rs`):**
   - Direct high-speed gRPC client via `qdrant-client`.
   - Supports payload schema indexing, filtered vector queries, and vector quantization.
2. **`PgVectorStore` (`src/memory/backend/pgvector.rs`):**
   - PostgreSQL vector extension integration via `sqlx`.
   - Supports HNSW and IVFFlat index querying with hybrid relational joins (`WHERE` + `<->` cosine distance).
3. **`LocalVectorStore` (`src/memory/store.rs`):**
   - Retains current atomic `.tagisan/memory.json` with FastHash / OpenAI / Gemini embeddings for 100% offline environments.
4. **`HybridSyncBridge` (`src/vella/vector_sync.rs`):**
   - Bidirectional synchronization between local NVMe cache (L1) and remote cloud vector database (L2).

---

## 4. Module 3: Enterprise Observability & OpenTelemetry

### File Location: `src/telemetry/mod.rs`

```
┌─────────────────────────────────────────────────────────────┐
│                    Tagisan Telemetry Mesh                   │
└──────────────┬───────────────────────────────┬──────────────┘
               │                               │
    ┌──────────▼───────────────┐   ┌───────────▼──────────────┐
    │  OpenTelemetry Exporter  │   │  Local SQLite / DuckDB   │
    │  (OTLP gRPC / HTTP)      │   │  Audit Journal           │
    └──────────┬───────────────┘   └───────────┬──────────────┘
               │                               │
    Enterprise APMs / Backends      Local Tools & UI
    • Langfuse (Self-hosted/Cloud)  • `tgs trace --live` (Ratatui TUI)
    • Arize Phoenix                 • Web UI Trace Inspector
    • Datadog / Honeycomb           • `.tagisan/traces.jsonl`
    • Jaeger / OTel Collector
```

### Key Span Metrics Tracked:
- **`dag.node.execute`:** Task name, model provider, input/output tokens, execution latency.
- **`agent.react.turn`:** Thought reasoning chain, tool name, arguments, observation status.
- **`debate.round`:** Thesis, Antithesis, Synthesis adjudications with Borda consensus scores.
- **`cost.usd`:** Exact cost per token calculated atomically per provider.
- **`agentshield.interception`:** Intercepted threats (`ThreatLevel::Safe` to `Critical`).

---

## 5. Module 4: Automated Swarm Evaluation Suite (`tgs eval`)

### File Location: `src/eval/mod.rs`

```bash
# Run automated benchmark evaluation against a test dataset
tgs eval run --dataset ./evals/sc_docket_benchmarks.json \
             --models claude-3-5-sonnet,deepseek-reasoner,gemini-1-5-pro \
             --rule borda \
             --threshold 0.85
```

### Evaluation Criteria:
1. **Groundedness / Faithfulness:** Detects if agent claims contradict retrieved context chunks.
2. **Tool Precision & Safety:** Assesses whether tool inputs adhered to JSON schemas without triggering AgentShield violations.
3. **Semantic Correctness:** Measures cosine similarity and exact AST match against reference answers.
4. **Multi-Model Consensus Scoring:** Runs 3 models in adversarial debate to produce a verified quality score (1.0 - 5.0) with reasoning breakdowns.

---

## 6. Implementation Phases & Milestones

| Phase | Component | Deliverables |
|---|---|---|
| **Phase 1** | **Vector Backend Abstraction** | Refactor `src/memory/` into pluggable `VectorStoreBackend` trait; add `LocalVectorStore` & `PgVectorStore`. |
| **Phase 2** | **Qdrant Native Integration** | Add `qdrant-client` gRPC backend with hybrid metadata filtering and connection pooling. |
| **Phase 3** | **OpenTelemetry Engine** | Add `src/telemetry/` with OTLP gRPC exporter for Langfuse, Phoenix, and Jaeger. |
| **Phase 4** | **`tgs trace` & TUI Inspector** | Ratatui live trace tree viewer + trace export to JSONL/SQLite. |
| **Phase 5** | **`tgs eval` Benchmark Runner** | Multi-agent automated evaluation CLI with dataset runner and Borda scoring. |
| **Phase 6** | **WASM Tool Sandbox** | Extism/Wasmtime plugin loader allowing user tools compiled to WebAssembly. |

---

*Saved to repository roadmap for future implementation.*
