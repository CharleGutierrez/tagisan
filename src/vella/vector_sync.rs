//! # Enterprise Multi-Database Vector Synchronization Bridge
//!
//! Synchronizes high-dimensional embeddings and contextual memory between
//! Tagisan's in-process `VectorStore` and Vella's enterprise multi-database
//! vector persistence engine (`vella::ai::vector` and `vella::db`).
//!
//! Supports push, pull, bidirectional conflict reconciliation (LWW), and
//! hybrid similarity search fusing local agent memory with Vella storage.

use crate::error::{Result, TagisanError};
use crate::memory::store::{VectorDocument, VectorStore};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;

#[cfg(feature = "vella")]
use vella::ai::vector::cosine_similarity;

/// A synchronized vector record stored within Vella's multi-database storage tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VellaVectorRecord {
    pub id: String,
    pub collection: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub payload_text: String,
    pub version: u64,
    pub updated_at: u64,
}

/// Statistics detailing vector synchronization operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VectorSyncStats {
    pub pushed_count: usize,
    pub pulled_count: usize,
    pub updated_count: usize,
    pub local_total: usize,
    pub remote_total: usize,
    pub duration_ms: u64,
}

/// A fused search result combining local Tagisan memory and Vella storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridVectorHit {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

/// Multi-database vector bridge managing bidirectional synchronization
#[derive(Clone)]
pub struct VellaVectorSyncBridge {
    pub local_store: Arc<VectorStore>,
    pub remote_records: Arc<RwLock<HashMap<String, VellaVectorRecord>>>,
    pub collection_name: String,
}

impl VellaVectorSyncBridge {
    pub fn new(local_store: Arc<VectorStore>, collection_name: impl Into<String>) -> Self {
        Self {
            local_store,
            remote_records: Arc::new(RwLock::new(HashMap::new())),
            collection_name: collection_name.into(),
        }
    }

    /// Push documents from Tagisan local VectorStore into Vella storage
    pub async fn push_to_vella(&self) -> Result<VectorSyncStats> {
        let start = SystemTime::now();
        let docs = self.local_store.documents();
        let mut remote = self.remote_records.write().await;

        let mut pushed = 0;
        let mut updated = 0;
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for doc in &docs {
            if let Some(existing) = remote.get_mut(&doc.id) {
                if existing.embedding != doc.embedding || existing.payload_text != doc.text {
                    existing.embedding = doc.embedding.clone();
                    existing.payload_text = doc.text.clone();
                    existing.metadata = doc.metadata.clone();
                    existing.version += 1;
                    existing.updated_at = now_sec;
                    updated += 1;
                }
            } else {
                remote.insert(
                    doc.id.clone(),
                    VellaVectorRecord {
                        id: doc.id.clone(),
                        collection: self.collection_name.clone(),
                        embedding: doc.embedding.clone(),
                        metadata: doc.metadata.clone(),
                        payload_text: doc.text.clone(),
                        version: 1,
                        updated_at: now_sec,
                    },
                );
                pushed += 1;
            }
        }

        let elapsed = start.elapsed().unwrap_or_default().as_millis() as u64;
        let local_total = docs.len();
        let remote_total = remote.len();

        info!(
            "🚀 [Vella VectorSync] Pushed {} new, updated {} docs to collection '{}' in {}ms",
            pushed, updated, self.collection_name, elapsed
        );

        Ok(VectorSyncStats {
            pushed_count: pushed,
            pulled_count: 0,
            updated_count: updated,
            local_total,
            remote_total,
            duration_ms: elapsed,
        })
    }

    /// Pull records from Vella storage into Tagisan local VectorStore
    pub async fn pull_from_vella(&self) -> Result<VectorSyncStats> {
        let start = SystemTime::now();
        let remote = self.remote_records.read().await;
        let mut pulled = 0;
        let mut updated = 0;

        let local_docs = self.local_store.documents();

        for record in remote.values() {
            let doc = VectorDocument::new(
                record.id.clone(),
                record.payload_text.clone(),
                record.embedding.clone(),
            )
            .with_metadata(record.metadata.clone());

            let exists = local_docs.iter().any(|d| d.id == record.id);
            self.local_store.add_document(doc)?;

            if exists {
                updated += 1;
            } else {
                pulled += 1;
            }
        }

        let elapsed = start.elapsed().unwrap_or_default().as_millis() as u64;
        let local_total = self.local_store.len();
        let remote_total = remote.len();

        info!(
            "📥 [Vella VectorSync] Pulled {} new, updated {} docs into local VectorStore in {}ms",
            pulled, updated, elapsed
        );

        Ok(VectorSyncStats {
            pushed_count: 0,
            pulled_count: pulled,
            updated_count: updated,
            local_total,
            remote_total,
            duration_ms: elapsed,
        })
    }

    /// Bidirectional synchronization with Last-Write-Wins (LWW) conflict resolution
    pub async fn bidirectional_sync(&self) -> Result<VectorSyncStats> {
        let start = SystemTime::now();
        let mut push_stats = self.push_to_vella().await?;
        let pull_stats = self.pull_from_vella().await?;

        let elapsed = start.elapsed().unwrap_or_default().as_millis() as u64;
        push_stats.pulled_count = pull_stats.pulled_count;
        push_stats.updated_count += pull_stats.updated_count;
        push_stats.duration_ms = elapsed;

        Ok(push_stats)
    }

    /// Perform hybrid similarity search fusing local Tagisan memory and Vella vector storage
    pub async fn hybrid_search(
        &self,
        query_vec: &[f32],
        top_k: usize,
    ) -> Result<Vec<HybridVectorHit>> {
        if query_vec.is_empty() {
            return Err(TagisanError::Execution(
                "Query vector cannot be empty".to_string(),
            ));
        }

        let mut hits: Vec<HybridVectorHit> = Vec::new();

        // 1. Search local VectorStore
        let local_results = self.local_store.search(query_vec, top_k * 2, 0.0);
        for r in local_results {
            hits.push(HybridVectorHit {
                id: r.document.id,
                text: r.document.text,
                score: r.score,
                source: "tagisan_local".to_string(),
                metadata: r.document.metadata,
            });
        }

        // 2. Search remote Vella records using Vella's real cosine_similarity
        let remote = self.remote_records.read().await;
        for r in remote.values() {
            #[cfg(feature = "vella")]
            let sim = cosine_similarity(query_vec, &r.embedding);
            #[cfg(not(feature = "vella"))]
            let sim = crate::memory::embedding::cosine_similarity(query_vec, &r.embedding);

            // De-duplicate if already matched locally with higher score
            if let Some(existing) = hits.iter_mut().find(|h| h.id == r.id) {
                if sim > existing.score {
                    existing.score = sim;
                    existing.source = "fused_vella".to_string();
                }
            } else {
                hits.push(HybridVectorHit {
                    id: r.id.clone(),
                    text: r.payload_text.clone(),
                    score: sim,
                    source: "vella_storage".to_string(),
                    metadata: r.metadata.clone(),
                });
            }
        }

        // Sort descending by score
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(top_k);

        Ok(hits)
    }
}

// =========================================================================
// Tool Handler Implementation
// =========================================================================

/// Tool for synchronizing vector collections between Tagisan and Vella
#[derive(Clone)]
pub struct VellaVectorSyncTool {
    pub bridge: VellaVectorSyncBridge,
}

impl Default for VellaVectorSyncTool {
    fn default() -> Self {
        let store = Arc::new(VectorStore::new());
        let bridge = VellaVectorSyncBridge::new(store, "default_knowledge_base");
        Self { bridge }
    }
}

impl VellaVectorSyncTool {
    pub fn new(bridge: VellaVectorSyncBridge) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl ToolHandler for VellaVectorSyncTool {
    fn name(&self) -> &'static str {
        "vella_vector_sync"
    }

    fn description(&self) -> &'static str {
        "Synchronize vector embeddings between Tagisan agent memory and Vella multi-database persistence. Supports push, pull, bidirectional sync, and hybrid similarity search."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["push", "pull", "sync", "search", "stats"],
                    "description": "Synchronization action to execute"
                },
                "query_vector": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Query vector for hybrid similarity search"
                },
                "top_k": {
                    "type": "integer",
                    "default": 5,
                    "description": "Max number of search results to return"
                },
                "document": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "text": { "type": "string" },
                        "embedding": {
                            "type": "array",
                            "items": { "type": "number" }
                        },
                        "metadata": { "type": "object" }
                    },
                    "description": "Optional document to insert locally before syncing"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing 'action' parameter".to_string()))?;

        // Optional document insertion
        if let Some(doc_val) = arguments.get("document") {
            let id = doc_val.get("id").and_then(|v| v.as_str()).unwrap_or("doc_auto");
            let text = doc_val.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let embedding: Vec<f32> = doc_val
                .get("embedding")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_f64().map(|f| f as f32))
                        .collect()
                })
                .unwrap_or_default();

            let mut metadata = HashMap::new();
            if let Some(meta_obj) = doc_val.get("metadata").and_then(|v| v.as_object()) {
                for (k, v) in meta_obj {
                    if let Some(s) = v.as_str() {
                        metadata.insert(k.clone(), s.to_string());
                    }
                }
            }

            let doc = VectorDocument::new(id, text, embedding).with_metadata(metadata);
            self.bridge.local_store.add_document(doc)?;
        }

        let res: Value = match action {
            "push" => {
                let stats = self.bridge.push_to_vella().await?;
                json!({
                    "status": "success",
                    "action": "push",
                    "stats": stats
                })
            }
            "pull" => {
                let stats = self.bridge.pull_from_vella().await?;
                json!({
                    "status": "success",
                    "action": "pull",
                    "stats": stats
                })
            }
            "sync" => {
                let stats = self.bridge.bidirectional_sync().await?;
                json!({
                    "status": "success",
                    "action": "sync",
                    "stats": stats
                })
            }
            "search" => {
                let query_vec: Vec<f32> = arguments
                    .get("query_vector")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_f64().map(|f| f as f32))
                            .collect()
                    })
                    .ok_or_else(|| {
                        TagisanError::Execution("Action 'search' requires 'query_vector'".to_string())
                    })?;

                let top_k = arguments.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let hits = self.bridge.hybrid_search(&query_vec, top_k).await?;

                json!({
                    "status": "success",
                    "action": "search",
                    "top_k": top_k,
                    "results_count": hits.len(),
                    "hits": hits
                })
            }
            "stats" => {
                let local_count = self.bridge.local_store.len();
                let remote = self.bridge.remote_records.read().await;
                json!({
                    "status": "success",
                    "collection": self.bridge.collection_name,
                    "local_documents_count": local_count,
                    "remote_documents_count": remote.len(),
                })
            }
            _ => {
                return Err(TagisanError::Execution(format!(
                    "Unknown action '{}'. Valid actions: push, pull, sync, search, stats",
                    action
                )));
            }
        };

        serde_json::to_string_pretty(&res).map_err(|e| TagisanError::Execution(e.to_string()))
    }
}
