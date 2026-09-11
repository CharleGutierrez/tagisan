//! PostgreSQL pgvector backend — gated behind the `pgvector` feature flag.
//!
//! Required schema:
//! ```sql
//! CREATE EXTENSION IF NOT EXISTS vector;
//! CREATE TABLE IF NOT EXISTS vector_documents (
//!     id        TEXT PRIMARY KEY,
//!     content   TEXT NOT NULL,
//!     embedding VECTOR,
//!     metadata  JSONB NOT NULL DEFAULT '{}'
//! );
//! ```

use super::{BackendSearchResult, BackendVectorDocument, VectorStoreBackend};
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use tracing::{debug, info};

/// VectorStoreBackend over PostgreSQL with the pgvector extension.
#[cfg(feature = "pgvector")]
#[derive(Debug, Clone)]
pub struct PgVectorBackend {
    pool: sqlx::PgPool,
}

#[cfg(feature = "pgvector")]
pub type PgVectorStore = PgVectorBackend;

#[cfg(feature = "pgvector")]
impl PgVectorBackend {
    /// Connect using a Postgres connection string (e.g. `postgres://user:pw@host/db`).
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(connection_string)
            .await
            .map_err(|e| TagisanError::Execution(format!("pgvector connect error: {e}")))?;
        info!(backend = "pgvector", "Connected to PostgreSQL");
        Ok(Self { pool })
    }

    /// Build a Postgres vector literal from a float slice: `[0.1,0.2,...]`.
    fn vec_to_pg_literal(v: &[f32]) -> String {
        let inner: Vec<String> = v.iter().map(|x| x.to_string()).collect();
        format!("[{}]", inner.join(","))
    }
}

#[cfg(feature = "pgvector")]
#[async_trait]
impl VectorStoreBackend for PgVectorBackend {
    fn name(&self) -> &str {
        "pgvector"
    }

    async fn insert(&self, _collection: &str, docs: &[BackendVectorDocument]) -> Result<()> {
        for doc in docs {
            let emb_literal = Self::vec_to_pg_literal(&doc.embedding);
            let meta_str =
                serde_json::to_string(&doc.metadata).map_err(TagisanError::Serialization)?;

            sqlx::query(
                r#"
                INSERT INTO vector_documents (id, content, embedding, metadata)
                VALUES ($1, $2, $3::vector, $4::jsonb)
                ON CONFLICT (id) DO UPDATE
                    SET content   = EXCLUDED.content,
                        embedding = EXCLUDED.embedding,
                        metadata  = EXCLUDED.metadata
                "#,
            )
            .bind(&doc.id)
            .bind(&doc.content)
            .bind(emb_literal)
            .bind(meta_str)
            .execute(&self.pool)
            .await
            .map_err(|e| TagisanError::Execution(format!("pgvector insert error: {e}")))?;
        }
        debug!(backend = "pgvector", count = docs.len(), "Upserted documents");
        Ok(())
    }

    async fn search(
        &self,
        _collection: &str,
        query_vector: &[f32],
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<BackendSearchResult>> {
        let emb_literal = Self::vec_to_pg_literal(query_vector);

        // pgvector <=> is cosine distance; cosine similarity = 1 - distance
        let rows = sqlx::query(
            r#"
            SELECT id, content, metadata,
                   (1.0 - (embedding <=> $1::vector)) AS score
            FROM vector_documents
            WHERE (1.0 - (embedding <=> $1::vector)) >= $2
            ORDER BY score DESC
            LIMIT $3
            "#,
        )
        .bind(emb_literal)
        .bind(threshold as f64)
        .bind(top_k as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TagisanError::Execution(format!("pgvector search error: {e}")))?;

        use sqlx::Row;
        let results = rows
            .iter()
            .map(|row| {
                let id: String = row.get("id");
                let content: String = row.get("content");
                let score: f64 = row.get("score");
                let metadata: serde_json::Value = row
                    .try_get::<serde_json::Value, _>("metadata")
                    .unwrap_or(serde_json::Value::Null);
                BackendSearchResult {
                    id,
                    content,
                    score: score as f32,
                    metadata,
                }
            })
            .collect();

        Ok(results)
    }

    async fn delete(&self, _collection: &str, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM vector_documents WHERE id = ANY($1)")
            .bind(ids)
            .execute(&self.pool)
            .await
            .map_err(|e| TagisanError::Execution(format!("pgvector delete error: {e}")))?;
        debug!(backend = "pgvector", deleted = ids.len(), "Deleted documents");
        Ok(())
    }

    async fn ping(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| TagisanError::Execution(format!("pgvector ping error: {e}")))?;
        Ok(true)
    }
}
