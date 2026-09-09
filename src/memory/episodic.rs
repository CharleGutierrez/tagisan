use crate::error::Result;
use crate::memory::embedding::EmbeddingProvider;
use crate::memory::store::{SearchResult, VectorDocument, VectorStore};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// High-level episodic memory manager that records agent decisions, architecture notes,
/// debugging sessions, and execution summaries for semantic recall across turns and sessions.
#[derive(Debug, Default, Clone)]
pub struct EpisodicMemory;

impl EpisodicMemory {
    pub fn new() -> Self {
        Self
    }

    /// Record an episodic memory into the vector store
    pub async fn record(
        &self,
        store: &VectorStore,
        provider: &dyn EmbeddingProvider,
        tag: &str,
        summary: &str,
        details: &str,
    ) -> Result<String> {
        let full_text = if details.trim().is_empty() {
            format!("[{tag}] {summary}")
        } else {
            format!("[{tag}] {summary}\n\n{details}")
        };

        let embedding = provider.embed_text(&full_text).await?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        let doc_id = format!("episodic_{}_{}", tag, timestamp);

        let mut metadata = HashMap::new();
        metadata.insert("tag".to_string(), tag.to_string());
        metadata.insert("type".to_string(), "episodic".to_string());
        metadata.insert("summary".to_string(), summary.to_string());

        let doc = VectorDocument::new(&doc_id, full_text, embedding).with_metadata(metadata);
        store.add_document(doc)?;

        info!("Recorded episodic memory '{}' under tag '{}'", doc_id, tag);
        Ok(doc_id)
    }

    /// Retrieve relevant episodic memories matching a semantic search query
    pub async fn recall(
        &self,
        store: &VectorStore,
        provider: &dyn EmbeddingProvider,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = provider.embed_text(query).await?;
        let results = store.search(&query_embedding, top_k, 0.15);
        Ok(results)
    }
}
