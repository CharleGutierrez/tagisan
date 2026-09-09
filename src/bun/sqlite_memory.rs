use crate::error::{Result, TagisanError};
use crate::memory::embedding::cosine_similarity;
use crate::memory::store::{SearchResult, VectorDocument, VectorStore};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// SQLite-backed persistent vector store synchronized with Tagisan and Bun TypeScript runtimes
#[derive(Clone)]
pub struct TagisanSqliteStore {
    path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl TagisanSqliteStore {
    /// Opens or creates SQLite database for vector storage
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        if let Some(parent) = path_buf.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let conn = Connection::open(&path_buf)
            .map_err(|e| TagisanError::Execution(format!("Failed to open SQLite database: {e}")))?;

        let store = Self {
            path: path_buf,
            conn: Arc::new(Mutex::new(conn)),
        };

        store.init_schema()?;
        Ok(store)
    }

    /// Initializes schema with table and indices for vector queries
    pub fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tagisan_vectors (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tagisan_vectors_created ON tagisan_vectors(created_at);
            "#,
        )
        .map_err(|e| TagisanError::Execution(format!("Failed to initialize SQLite schema: {e}")))?;

        Ok(())
    }

    /// Packs f32 slice into little-endian bytes matching Bun's Float32Array
    pub fn pack_embedding(floats: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(floats.len() * 4);
        for &f in floats {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        bytes
    }

    /// Unpacks little-endian bytes into Vec<f32>
    pub fn unpack_embedding(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }

    /// Inserts or replaces a vector document
    pub fn insert(&self, doc: &VectorDocument) -> Result<()> {
        let blob = Self::pack_embedding(&doc.embedding);
        let metadata_json = serde_json::to_string(&doc.metadata)
            .map_err(|e| TagisanError::Serialization(e))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tagisan_vectors (id, text, embedding, dimension, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                doc.id,
                doc.text,
                blob,
                doc.embedding.len() as i64,
                metadata_json,
                doc.created_at as i64,
            ],
        )
        .map_err(|e| TagisanError::Execution(format!("Failed to insert vector doc: {e}")))?;

        Ok(())
    }

    /// Inserts a batch of documents within a single transaction
    pub fn insert_batch(&self, docs: &[VectorDocument]) -> Result<usize> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| TagisanError::Execution(format!("Failed to begin transaction: {e}")))?;

        for doc in docs {
            let blob = Self::pack_embedding(&doc.embedding);
            let metadata_json = serde_json::to_string(&doc.metadata)
                .map_err(|e| TagisanError::Serialization(e))?;

            tx.execute(
                "INSERT OR REPLACE INTO tagisan_vectors (id, text, embedding, dimension, metadata, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    doc.id,
                    doc.text,
                    blob,
                    doc.embedding.len() as i64,
                    metadata_json,
                    doc.created_at as i64,
                ],
            )
            .map_err(|e| TagisanError::Execution(format!("Failed to insert in batch: {e}")))?;
        }

        tx.commit()
            .map_err(|e| TagisanError::Execution(format!("Failed to commit batch transaction: {e}")))?;

        Ok(docs.len())
    }

    /// Synchronizes from Tagisan's in-memory VectorStore into SQLite
    pub fn sync_from_vector_store(&self, store: &VectorStore) -> Result<usize> {
        let docs = store.documents();
        self.insert_batch(&docs)
    }

    /// Loads all documents from SQLite into a Tagisan in-memory VectorStore
    pub fn sync_to_vector_store(&self, store: &mut VectorStore) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, text, embedding, metadata, created_at FROM tagisan_vectors")
            .map_err(|e| TagisanError::Execution(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let text: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                let meta_str: String = row.get(3)?;
                let created_at: i64 = row.get(4)?;

                let embedding = TagisanSqliteStore::unpack_embedding(&blob);
                let metadata: HashMap<String, String> =
                    serde_json::from_str(&meta_str).unwrap_or_default();

                Ok(VectorDocument {
                    id,
                    text,
                    embedding,
                    metadata,
                    created_at: created_at as u64,
                })
            })
            .map_err(|e| TagisanError::Execution(format!("Query failed: {e}")))?;

        let mut count = 0;
        for r in rows {
            if let Ok(doc) = r {
                let _ = store.add_document(doc);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Performs exact cosine similarity search over stored embeddings
    pub fn search_cosine(&self, query_embedding: &[f32], top_k: usize, threshold: f32) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, text, embedding, metadata, created_at FROM tagisan_vectors")
            .map_err(|e| TagisanError::Execution(format!("Prepare search failed: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let text: String = row.get(1)?;
                let blob: Vec<u8> = row.get(2)?;
                let meta_str: String = row.get(3)?;
                let created_at: i64 = row.get(4)?;

                let embedding = TagisanSqliteStore::unpack_embedding(&blob);
                let metadata: HashMap<String, String> =
                    serde_json::from_str(&meta_str).unwrap_or_default();

                Ok(VectorDocument {
                    id,
                    text,
                    embedding,
                    metadata,
                    created_at: created_at as u64,
                })
            })
            .map_err(|e| TagisanError::Execution(format!("Query map failed: {e}")))?;

        let mut scored: Vec<SearchResult> = Vec::new();
        for r in rows {
            if let Ok(doc) = r {
                let score = cosine_similarity(query_embedding, &doc.embedding);
                if score >= threshold {
                    scored.push(SearchResult { document: doc, score });
                }
            }
        }

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        Ok(scored)
    }

    /// Returns total document count
    pub fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tagisan_vectors", [], |r| r.get(0))
            .map_err(|e| TagisanError::Execution(format!("Count query failed: {e}")))?;
        Ok(count as usize)
    }

    /// Database file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}
