//! Local in-process vector backend that delegates to the existing VectorStore.

use super::{BackendSearchResult, BackendVectorDocument, VectorStoreBackend};
use crate::error::Result;
use crate::memory::store::{VectorDocument as StoreDoc, VectorStore};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// An in-process vector backend backed by a VectorStore per collection.
#[derive(Debug, Clone)]
pub struct LocalVectorBackend {
    collections: Arc<RwLock<HashMap<String, VectorStore>>>,
}

impl LocalVectorBackend {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn from_store(collection_name: impl Into<String>, store: VectorStore) -> Self {
        let mut map = HashMap::new();
        map.insert(collection_name.into(), store);
        Self {
            collections: Arc::new(RwLock::new(map)),
        }
    }

    fn to_store_doc(doc: &BackendVectorDocument) -> StoreDoc {
        let meta: HashMap<String, String> = match &doc.metadata {
            serde_json::Value::Object(map) => map
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    (k.clone(), val)
                })
                .collect(),
            _ => HashMap::new(),
        };
        StoreDoc::new(doc.id.clone(), doc.content.clone(), doc.embedding.clone())
            .with_metadata(meta)
    }
}

impl Default for LocalVectorBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStoreBackend for LocalVectorBackend {
    fn name(&self) -> &str {
        "local"
    }

    async fn insert(&self, collection: &str, docs: &[BackendVectorDocument]) -> Result<()> {
        let mut guard = self.collections.write().await;
        let store = guard
            .entry(collection.to_string())
            .or_insert_with(VectorStore::new);
        let store_docs: Vec<StoreDoc> = docs.iter().map(Self::to_store_doc).collect();
        store.add_documents(store_docs)?;
        debug!(
            backend = "local",
            collection = collection,
            count = docs.len(),
            "Upserted documents"
        );
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<BackendSearchResult>> {
        let guard = self.collections.read().await;
        let store = match guard.get(collection) {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };
        let raw = store.search(query_vector, top_k, threshold);
        Ok(raw
            .into_iter()
            .map(|hit| {
                let meta_obj: serde_json::Map<String, serde_json::Value> = hit
                    .document
                    .metadata
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect();
                BackendSearchResult {
                    id: hit.document.id,
                    content: hit.document.text,
                    score: hit.score,
                    metadata: serde_json::Value::Object(meta_obj),
                }
            })
            .collect())
    }

    async fn delete(&self, collection: &str, ids: &[String]) -> Result<()> {
        let guard = self.collections.read().await;
        let store = match guard.get(collection) {
            Some(s) => s,
            None => return Ok(()),
        };
        let id_set: std::collections::HashSet<&String> = ids.iter().collect();
        let remaining: Vec<StoreDoc> = store
            .documents()
            .into_iter()
            .filter(|d| !id_set.contains(&d.id))
            .collect();
        store.clear()?;
        store.add_documents(remaining)?;
        debug!(
            backend = "local",
            collection = collection,
            deleted = ids.len(),
            "Deleted documents"
        );
        Ok(())
    }

    async fn ping(&self) -> Result<bool> {
        Ok(true)
    }
}

impl From<VectorStore> for LocalVectorBackend {
    fn from(store: VectorStore) -> Self {
        Self::from_store("default", store)
    }
}
