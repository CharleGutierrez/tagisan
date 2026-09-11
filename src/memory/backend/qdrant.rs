//! Qdrant REST API backend — gated behind the `qdrant` feature flag.

use super::{BackendSearchResult, BackendVectorDocument, VectorStoreBackend};
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;

/// VectorStoreBackend over the Qdrant vector database REST/gRPC API.
#[cfg(feature = "qdrant")]
#[derive(Debug, Clone)]
pub struct QdrantBackend {
    client: Client,
    base_url: String,
}

#[cfg(feature = "qdrant")]
pub type QdrantStore = QdrantBackend;

#[cfg(feature = "qdrant")]
impl QdrantBackend {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn with_client(base_url: impl Into<String>, client: Client) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:6334")
    }

    fn points_url(&self, collection: &str) -> String {
        format!("{}/collections/{}/points", self.base_url, collection)
    }

    /// Create or ensure a target collection exists with specified vector dimensions and distance metric.
    pub async fn create_collection(&self, collection: &str, vector_size: usize, distance: &str) -> Result<()> {
        let url = format!("{}/collections/{}", self.base_url, collection);
        let body = json!({
            "vectors": {
                "size": vector_size,
                "distance": distance
            }
        });
        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant create_collection error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::Execution(format!(
                "qdrant create_collection HTTP {status}: {text}"
            )));
        }
        Ok(())
    }

    /// Search with hybrid payload metadata filter.
    pub async fn search_with_filter(
        &self,
        collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
        filter: Option<Value>,
    ) -> Result<Vec<BackendSearchResult>> {
        let url = format!("{}/search", self.points_url(collection));
        let mut body = json!({
            "vector": query_vector,
            "limit": top_k,
            "score_threshold": threshold,
            "with_payload": true,
        });
        if let Some(f) = filter {
            body["filter"] = f;
        }

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant search request error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::Execution(format!(
                "qdrant search HTTP {status}: {text}"
            )));
        }

        let parsed: QdrantSearchResponse = resp
            .json()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant search parse error: {e}")))?;

        let results = parsed
            .result
            .into_iter()
            .map(|pt| {
                let id = match &pt.id {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let payload = pt.payload.unwrap_or(Value::Null);
                let content = payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let metadata = payload.get("metadata").cloned().unwrap_or(Value::Null);
                BackendSearchResult {
                    id,
                    content,
                    score: pt.score,
                    metadata,
                }
            })
            .collect();

        Ok(results)
    }
}

#[cfg(feature = "qdrant")]
#[derive(Serialize, Deserialize, Debug)]
struct QdrantScoredPoint {
    id: Value,
    score: f32,
    payload: Option<Value>,
}

#[cfg(feature = "qdrant")]
#[derive(Deserialize, Debug)]
struct QdrantSearchResponse {
    result: Vec<QdrantScoredPoint>,
}

#[cfg(feature = "qdrant")]
#[async_trait]
impl VectorStoreBackend for QdrantBackend {
    fn name(&self) -> &str {
        "qdrant"
    }

    async fn insert(&self, collection: &str, docs: &[BackendVectorDocument]) -> Result<()> {
        let points: Vec<Value> = docs
            .iter()
            .map(|doc| {
                json!({
                    "id": doc.id,
                    "vector": doc.embedding,
                    "payload": {
                        "content": doc.content,
                        "metadata": doc.metadata
                    }
                })
            })
            .collect();

        let url = format!("{}?wait=true", self.points_url(collection));
        let body = json!({ "points": points });

        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant insert request error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::Execution(format!(
                "qdrant insert HTTP {status}: {text}"
            )));
        }

        debug!(backend = "qdrant", collection = collection, count = docs.len(), "Upserted points");
        Ok(())
    }

    async fn search(
        &self,
        collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<BackendSearchResult>> {
        let url = format!("{}/search", self.points_url(collection));
        let body = json!({
            "vector": query_vector,
            "limit": top_k,
            "score_threshold": threshold,
            "with_payload": true,
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant search request error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::Execution(format!(
                "qdrant search HTTP {status}: {text}"
            )));
        }

        let parsed: QdrantSearchResponse = resp
            .json()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant search parse error: {e}")))?;

        let results = parsed
            .result
            .into_iter()
            .map(|pt| {
                let id = match &pt.id {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let payload = pt.payload.unwrap_or(Value::Null);
                let content = payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let metadata = payload.get("metadata").cloned().unwrap_or(Value::Null);
                BackendSearchResult {
                    id,
                    content,
                    score: pt.score,
                    metadata,
                }
            })
            .collect();

        Ok(results)
    }

    async fn delete(&self, collection: &str, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let url = format!("{}/delete?wait=true", self.points_url(collection));
        let body = json!({ "points": ids });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant delete request error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::Execution(format!(
                "qdrant delete HTTP {status}: {text}"
            )));
        }

        debug!(backend = "qdrant", collection = collection, deleted = ids.len(), "Deleted points");
        Ok(())
    }

    async fn ping(&self) -> Result<bool> {
        let url = format!("{}/healthz", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TagisanError::Execution(format!("qdrant ping error: {e}")))?;
        Ok(resp.status().is_success())
    }
}
