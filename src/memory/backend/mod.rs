//! Pluggable vector store backend trait and shared types.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod local;

#[cfg(feature = "pgvector")]
pub mod pgvector;

#[cfg(feature = "qdrant")]
pub mod qdrant;

/// A document stored in a vector backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendVectorDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: serde_json::Value,
}

impl BackendVectorDocument {
    pub fn new(id: impl Into<String>, content: impl Into<String>, embedding: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            embedding,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A ranked search hit returned from a similarity query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

impl BackendSearchResult {
    pub fn new(id: impl Into<String>, content: impl Into<String>, score: f32) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            score,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Type alias aligning with RFC-001 naming
pub type VectorDocument = BackendVectorDocument;
/// Type alias aligning with RFC-001 naming
pub type SearchResult = BackendSearchResult;
/// Type alias aligning with RFC-001 naming
pub type LocalVectorStore = local::LocalVectorBackend;

pub use local::LocalVectorBackend;

#[cfg(feature = "pgvector")]
pub use pgvector::{PgVectorBackend, PgVectorBackend as PgVectorStore};

#[cfg(feature = "qdrant")]
pub use qdrant::{QdrantBackend, QdrantBackend as QdrantStore};

pub use crate::vella::vector_sync::VellaVectorSyncBridge as HybridSyncBridge;

/// Async, pluggable interface for any vector storage backend.
#[async_trait]
pub trait VectorStoreBackend: Send + Sync {
    /// Name identifier for this backend (e.g. "local", "pgvector", "qdrant").
    fn name(&self) -> &str;

    /// Insert or upsert vectorized documents into a target collection.
    async fn insert(&self, collection: &str, docs: &[BackendVectorDocument]) -> Result<()>;

    /// Perform vector similarity search (cosine) and return top-k results above threshold.
    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<BackendSearchResult>>;

    /// Delete vectors by document ID list.
    async fn delete(&self, collection: &str, ids: &[String]) -> Result<()>;

    /// Health check and connection verification.
    async fn ping(&self) -> Result<bool>;
}
