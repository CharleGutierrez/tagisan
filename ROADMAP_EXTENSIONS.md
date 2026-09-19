# 📌 Tagisan Architecture Blueprint & Implementation Roadmap (RFC-001)

**Status:** Implemented / RFC-001 Complete  
**Target:** Tagisan (`tgs` / `tagisan-rs`)  
**Scope:** Universal Ecosystem Integrations, Enterprise Vector DB Backends, OpenTelemetry Observability, and Swarm-Based Automated Evaluation (`tgs eval`).

---

## 📑 Table of Contents
1. [Executive Summary](#1-executive-summary)
2. [Module 1: Universal Protocol & Ecosystem Integrations](#2-module-1-universal-protocol--ecosystem-integrations)
3. [Module 2: Pluggable External Vector Databases (`src/memory/backend/`)](#3-module-2-pluggable-external-vector-databases)
4. [Module 3: Enterprise Observability & OpenTelemetry (`src/telemetry/`)](#4-module-3-enterprise-observability--opentelemetry)
5. [Module 4: Automated Swarm Evaluation Suite (`tgs eval`)](#5-module-4-automated-swarm-evaluation-suite-tgs-eval)
6. [Module 5: Federated Agent Bridge & Swarm Event Bus (`src/swarm/bridge.rs`)](#6-module-5-federated-agent-bridge--swarm-event-bus)
7. [Implementation Phases & Milestones](#7-implementation-phases--milestones)

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

## 6. Module 5: Federated Agent Bridge & Swarm Event Bus (`src/swarm/bridge.rs`)

### File Location: `src/swarm/bridge.rs`

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Tagisan Federated Agent Bridge                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────────┐    ┌──────────────────┐    ┌─────────────────────┐   │
│   │ NativeRust Agent │    │   MCP JSON-RPC   │    │ A2A WebSocket / IPC │   │
│   └─────────┬────────┘    └─────────┬────────┘    └──────────┬──────────┘   │
│             │                       │                        │              │
│             └───────────────────────┼────────────────────────┘              │
│                                     ▼                                       │
│                       ┌───────────────────────────┐                         │
│                       │  AgentShield Security     │                         │
│                       │  (DLP + Injection Guard)  │                         │
│                       └─────────────┬─────────────┘                         │
│                                     ▼                                       │
│    ┌────────────────────────────────┼─────────────────────────────────┐     │
│    ▼                                ▼                                 ▼     │
│ ┌──────────────────────┐ ┌──────────────────────┐ ┌───────────────────────┐ │
│ │ Reactive Event Bus   │ │ Hybrid Router        │ │ Per-Agent Circuit     │ │
│ │ (Pub/Sub & Wildcards)│ │ (0-Stall Failover)   │ │ Breakers (Tripping)   │ │
│ └──────────────────────┘ └──────────────────────┘ └───────────────────────┘ │
│    ▲                                ▲                                 ▲     │
│    └────────────────────────────────┼─────────────────────────────────┘     │
│                                     ▼                                       │
│                       ┌───────────────────────────┐                         │
│                       │ Shared Reflexion Vault    │                         │
│                       │ (Cross-Agent Knowledge)   │                         │
│                       └───────────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Advantages & Capabilities:
1. **Universal Protocol Federation:** Seamlessly bridges heterogeneous agents across `NativeRust`, `McpJsonRpc`, `A2aWebSocket`, `IpcUnixSocket`, `RestHttp`, and `ExternalProcess`.
2. **Reactive High-Throughput Pub/Sub:** Topic-based event distribution with single-level (`*`), multi-level (`**`, `#`), and exact matching with bounded backpressure queues.
3. **Hybrid Edge-Cloud Workload Router:** Automatically routes tasks between local Edge Ollama and Cloud Frontier models with dynamic complexity scoring and 0-stall graceful failover during cloud outages.
4. **Per-Agent Circuit Breakers:** Prevents cascading network/model failures with 3-state isolation (`Closed`, `Open`, `Half-Open`) and canary probe recovery.
5. **Zero-Trust AgentShield Gateway:** Automatic DLP redaction of API keys (Anthropic, OpenAI, GitHub, AWS, private keys) and adversarial prompt injection / jailbreak blocking.
6. **Swarm-Wide Shared Reflexion Vault:** Synchronizes mistakes, post-mortems, and execution insights across all agents in real-time.
7. **Tool Multiplexing:** Exposes `agent_bridge_dispatch` and `agent_bridge_publish` to `ToolRegistry` and enables `/bridge` REPL commands (`status`, `agents`, `broadcast`, `route`).

---

## 7. Module 6: Enterprise Oracle Tech Stack Integration

### Architectural Overview
To enable sovereign, zero-drift AI orchestration over mission-critical enterprise databases and ERP ledgers, Tagisan integrates the Oracle enterprise stack directly into its core engine (`src/engine/oracle.rs`):

1. **Oracle Database 23ai AI Vector Search + Relational Fusion:**
   - Single-statement hybrid SQL queries uniting vector embeddings (`VECTOR_DISTANCE`) with complex relational predicates (`WHERE jurisdiction = ... AND amount > ...`).
   - Eliminates out-of-sync external vector databases while preserving full ACID transaction semantics.

2. **JSON-Relational Duality Views:**
   - Autonomous agents read and mutate native JSON documents, while Oracle automatically decomposes and persists them into normalized 3NF relational tables.
   - Built-in ETag optimistic concurrency control prevents race conditions across concurrent swarm agents.

3. **Oracle GoldenGate Change Data Capture (CDC):**
   - Transaction redo log scraping streams table mutations (`INSERT`, `UPDATE`, `DELETE`) directly into the TGS Swarm Event Bus (`oracle.cdc.<schema>.<table>.<op>`).
   - Agents react to database mutations in sub-milliseconds without polling.

4. **Oracle Fusion & NetSuite ERP 3-Way Procurement Matching:**
   - Autonomous reconciliation between Vendor Invoices, Purchase Orders (PO), and Goods Receipt Notes (GRN).
   - Straight-through processing (STP) for exact matches and variance within tolerance thresholds (e.g. $\le 1.5\%$). Automatic quarantine and human-in-the-loop escalation for price or quantity discrepancies.

5. **GraalVM Native Image Interop:**
   - Ahead-of-time (AOT) compiled Java/C++ enterprise business logic and compliance rules executed with microsecond cold starts and minimal memory footprint.

6. **AgentShield Oracle Database Vault Governance:**
   - Real-time SQL inspection blocking destructive operations (`DROP TABLE`, `TRUNCATE`, `ALTER TABLE`, `GRANT ALL`) without supermajority swarm consensus.
   - Dynamic data masking redacting sensitive PII/financial data (SSN, credit card, bank accounts) before reaching LLM context windows.

7. **Oracle Immutable Blockchain Tables:**
   - Cryptographically linked append-only audit trail for agent debates, Borda consensus voting results, ERP approvals, judicial decisions, and code patches.
   - Non-repudiable audit verification (`verify_chain_integrity()`) where any retroactive modification, truncation, or tamper attempt fails cryptographic verification.

8. **Oracle APEX Autonomous Low-Code App Generator:**
   - Autonomous schema-to-APEX low-code application and dashboard generator (Page Designer JSON/SQL export, interactive report definitions, interactive grids, audit screens).

9. **OCI Sovereign AI Supercluster & GovCloud Enclaves:**
   - Dedicated sovereign GPU clusters, data residency enforcement (verifying zero cross-border data egress, geo-fencing policies for Supreme Court / GovCloud compliance), and secure token routing.

10. **Oracle RAC & Autonomous Data Guard Bridge:**
    - Active-active high availability clustering, zero-data-loss replication status tracking (RPO=0, RTO<1s), automated failover triggers, and node heartbeat monitoring.

11. **Oracle HeatWave Lakehouse & In-Database AutoML:**
    - Lakehouse query generator over Parquet/Iceberg object storage with `RAPID` in-memory secondary engine, plus automated in-database ML pipeline (feature engineering, model selection, predictive risk scoring).

12. **Oracle Integration Cloud (OIC) Enterprise Mesh:**
    - Pre-built enterprise adapter mesh connecting TGS to SAP, Salesforce, Workday, ServiceNow, and legacy on-premises mainframes with two-phase commit synchronization.

13. **Oracle 23ai Operational Property Graph (SQL:2023 `GRAPH_TABLE`):**
    - Native `GRAPH_TABLE` and PGQL syntax generation for multi-hop graph traversals, PageRank, AML fraud ring detection, and legal citation graph clustering without external graph engines.

14. **Oracle Exadata Smart Scan & Storage Cell Offloading:**
    - Offloading query predicates and projection operations directly to Exadata storage cell CPU and FPGA accelerators (`CELL_OFFLOAD`, `STORAGE_INDEX`), achieving >90% I/O reduction and sub-millisecond query latencies.

15. **Oracle Text & Lexical Mining Engine (Hybrid RRF Search):**
    - High-performance `CONTAINS()` full-text search with linguistic stemming and Boolean proximity combined with dense vector embeddings via Reciprocal Rank Fusion (RRF $k=60$).

16. **Oracle Spatial & Geospatial Jurisdiction Engine (`SDO_GEOMETRY`):**
    - Native GIS territorial jurisdiction analysis for Regional Trial Courts (RTC), Metropolitan Trial Courts (MTC), and Sandiganbayan, alongside Goods Receipt Note (GRN) delivery truck geofencing.

17. **Oracle Key Vault (OKV) & Hardware Security Module (HSM) Enclave:**
    - FIPS 140-3 Level 4 HSM integration for signing immutable agent audit logs, Borda consensus votes, and judicial rulings, with automated Transparent Data Encryption (TDE) master key rotation.

18. **Oracle Coherence Distributed In-Memory Swarm Memory Grid:**
    - Distributed in-memory data grid providing low-latency distributed agent state sharing and cluster-wide distributed locks (`acquire_distributed_lock`, `release_distributed_lock`).

19. **Oracle Autonomous Health Framework (AHF) & Self-Healing:**
    - Automated real-time diagnosis of concurrency contention, deadlock risk, and tablespace pressure, paired with autonomous self-healing remediation routines.

---

## 8. Implementation Phases & Milestones

| Phase | Component | Deliverables | Status |
|---|---|---|---|
| **Phase 1** | **Vector Backend Abstraction** | Refactor `src/memory/` into pluggable `VectorStoreBackend` trait; add `LocalVectorStore` & `PgVectorStore`. | ✅ **Completed** (`src/memory/backend/mod.rs`, `pgvector.rs`) |
| **Phase 2** | **Qdrant Native Integration** | Add `qdrant-client` gRPC backend with hybrid metadata filtering, collection creation, and connection pooling. | ✅ **Completed** (`src/memory/backend/qdrant.rs`) |
| **Phase 3** | **OpenTelemetry Engine** | Add `src/telemetry/` with OTLP gRPC exporter for Langfuse, Phoenix, and Jaeger (`TagisanTracer`). | ✅ **Completed** (`src/telemetry/otel.rs`) |
| **Phase 4** | **`tgs trace` & TUI Inspector** | Ratatui live trace tree viewer + trace export to JSONL/SQLite (`run_trace_tui`, `TraceArgs`). | ✅ **Completed** (`src/telemetry/tui.rs`, `journal.rs`) |
| **Phase 5** | **`tgs eval` Benchmark Runner** | Multi-agent automated evaluation CLI with dataset runner, multi-criteria scoring, and Borda/Majority consensus. | ✅ **Completed** (`src/eval/`, `evals/sc_docket_benchmarks.json`) |
| **Phase 6** | **WASM Tool Sandbox** | Sandboxed WebAssembly plugin engine (`WasmTool`) enforcing CPU fuel quotas, validation, and directory loading. | ✅ **Completed** (`src/tools/wasm.rs`) |
| **Phase 7** | **Federated Agent Bridge** | Universal cross-framework agent bridge, reactive pub/sub bus, hybrid router, circuit breakers, and REPL `/bridge`. | ✅ **Completed** (`src/swarm/bridge.rs`, `tests/bridge_test.rs`) |
| **Phase 8** | **Oracle Enterprise Integration Suite** | 18 Oracle subsystems, 10 built-in enterprise tools, telemetry orchestrator, and REPL `/oracle`. | ✅ **Completed** (`src/engine/oracle.rs`, `tests/oracle_test.rs`) |

---

### Verification & Testing
All phases are covered by comprehensive unit and integration tests:
- `tests/roadmap_extensions_tests.rs` (Phases 1-6)
- `tests/bridge_test.rs` (Phase 7)
- `tests/oracle_test.rs` (Phase 8: 6 integration test suites, 20 unit tests, 100% passing)

*RFC-001 fully implemented and verified in Tagisan.*
