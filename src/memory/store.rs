use crate::error::{Result, TagisanError};
use crate::memory::embedding::cosine_similarity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

/// A stored document with its embedding vector and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorDocument {
    pub id: String,
    pub text: String,
    pub embedding: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: u64,
}

impl VectorDocument {
    pub fn new(id: impl Into<String>, text: impl Into<String>, embedding: Vec<f32>) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: id.into(),
            text: text.into(),
            embedding,
            metadata: HashMap::new(),
            created_at,
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A ranked search hit containing the document and its cosine similarity score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub document: VectorDocument,
    pub score: f32,
}

/// Structural statistics for an active or serialized VectorStore
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub embedding_dimensions: usize,
    pub storage_bytes: usize,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorStoreData {
    pub version: u32,
    pub documents: Vec<VectorDocument>,
}

impl Default for VectorStoreData {
    fn default() -> Self {
        Self {
            version: 1,
            documents: Vec::new(),
        }
    }
}

/// Thread-safe, serializable vector repository supporting top-k cosine similarity queries
#[derive(Debug, Clone)]
pub struct VectorStore {
    data: Arc<RwLock<VectorStoreData>>,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(VectorStoreData::default())),
        }
    }

    pub fn with_documents(documents: Vec<VectorDocument>) -> Self {
        Self {
            data: Arc::new(RwLock::new(VectorStoreData {
                version: 1,
                documents,
            })),
        }
    }

    /// Add a single document to the store (replaces if document with same id exists)
    pub fn add_document(&self, doc: VectorDocument) -> Result<()> {
        let mut guard = self.data.write().map_err(|_| {
            TagisanError::Execution("VectorStore lock poisoned on write".to_string())
        })?;

        if let Some(pos) = guard.documents.iter().position(|d| d.id == doc.id) {
            guard.documents[pos] = doc;
        } else {
            guard.documents.push(doc);
        }
        Ok(())
    }

    /// Add multiple documents to the store
    pub fn add_documents(&self, docs: Vec<VectorDocument>) -> Result<()> {
        let mut guard = self.data.write().map_err(|_| {
            TagisanError::Execution("VectorStore lock poisoned on write".to_string())
        })?;

        for doc in docs {
            if let Some(pos) = guard.documents.iter().position(|d| d.id == doc.id) {
                guard.documents[pos] = doc;
            } else {
                guard.documents.push(doc);
            }
        }
        Ok(())
    }

    /// Execute a semantic vector similarity search returning the top-k highest scoring matches
    pub fn search(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Vec<SearchResult> {
        let guard = match self.data.read() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        let mut results: Vec<SearchResult> = guard
            .documents
            .iter()
            .filter_map(|doc| {
                let score = cosine_similarity(&doc.embedding, query_embedding);
                if score >= threshold {
                    Some(SearchResult {
                        document: doc.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort descending by similarity score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        results
    }

    /// Clear all stored documents
    pub fn clear(&self) -> Result<()> {
        let mut guard = self.data.write().map_err(|_| {
            TagisanError::Execution("VectorStore lock poisoned on write".to_string())
        })?;
        guard.documents.clear();
        Ok(())
    }

    /// Get total number of stored vector documents
    pub fn len(&self) -> usize {
        self.data.read().map(|g| g.documents.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return memory statistics
    pub fn stats(&self, file_path: Option<&Path>) -> MemoryStats {
        let guard = self.data.read().ok();
        let total_documents = guard.as_ref().map(|g| g.documents.len()).unwrap_or(0);
        let embedding_dimensions = guard
            .as_ref()
            .and_then(|g| g.documents.first().map(|d| d.embedding.len()))
            .unwrap_or(0);

        let storage_bytes = file_path
            .and_then(|p| fs::metadata(p).ok().map(|m| m.len() as usize))
            .unwrap_or(0);

        MemoryStats {
            total_documents,
            total_chunks: total_documents,
            embedding_dimensions,
            storage_bytes,
            file_path: file_path.map(|p| p.to_string_lossy().to_string()),
        }
    }

    /// Default file path for Tagisan persistent memory (`.tagisan/memory.json`)
    pub fn default_path() -> PathBuf {
        PathBuf::from(".tagisan").join("memory.json")
    }

    /// Persist the vector store to disk as formatted JSON
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TagisanError::Execution(format!("Failed to create memory directory '{:?}': {e}", parent))
            })?;
        }

        let guard = self.data.read().map_err(|_| {
            TagisanError::Execution("VectorStore lock poisoned on read".to_string())
        })?;

        let json_str = serde_json::to_string_pretty(&*guard).map_err(TagisanError::Serialization)?;
        fs::write(path_ref, json_str).map_err(|e| {
            TagisanError::Execution(format!("Failed to write memory file '{:?}': {e}", path_ref))
        })?;

        debug!("Saved VectorStore with {} documents to {:?}", guard.documents.len(), path_ref);
        Ok(())
    }

    /// Load vector store from disk
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(TagisanError::Execution(format!(
                "Memory store file does not exist: {:?}",
                path_ref
            )));
        }

        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!("Failed to read memory file '{:?}': {e}", path_ref))
        })?;

        let data: VectorStoreData = serde_json::from_str(&content).map_err(TagisanError::Serialization)?;
        info!("Loaded VectorStore with {} documents from {:?}", data.documents.len(), path_ref);

        Ok(Self {
            data: Arc::new(RwLock::new(data)),
        })
    }

    /// Load vector store from default `.tagisan/memory.json` or create empty store if absent
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            match Self::load_from_file(&path) {
                Ok(store) => store,
                Err(e) => {
                    debug!("Failed to load memory store from default path {:?}: {e}. Creating new empty store.", path);
                    Self::new()
                }
            }
        } else {
            Self::new()
        }
    }
}
