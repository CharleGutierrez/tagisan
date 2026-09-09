use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tracing::debug;

/// Trait defining a pluggable text embedding model provider
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Identifier of the provider (e.g. "fasthash", "ollama", "openai", "gemini")
    fn provider_id(&self) -> &'static str;

    /// Output vector dimensions produced by this provider
    fn dimensions(&self) -> usize;

    /// Generate an embedding vector for a single text chunk
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embedding vectors for a batch of text chunks
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed_text(t).await?);
        }
        Ok(results)
    }
}

// =========================================================================
// Native Vector Mathematics
// =========================================================================

/// Compute the Euclidean (L2) norm of a vector
pub fn l2_norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Compute the dot product between two vectors
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Compute the cosine similarity between two vectors, returning a score in [-1.0, 1.0]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let norm_a = l2_norm(a);
    let norm_b = l2_norm(b);

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    let dot = dot_product(a, b);
    (dot / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Normalize a vector in-place to unit L2 length
pub fn normalize_vector(v: &mut [f32]) {
    let norm = l2_norm(v);
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

// =========================================================================
// 1. FastHashEmbeddingProvider (100% Offline, Deterministic, Zero-Dependency)
// =========================================================================

/// Production-grade deterministic n-gram and word frequency embedding generator
/// that operates 100% locally with zero network calls, zero API keys, and microsecond latency.
#[derive(Debug, Clone)]
pub struct FastHashEmbeddingProvider {
    dimensions: usize,
}

impl FastHashEmbeddingProvider {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }

    /// Default 256-dimensional fast hash embedding
    pub fn default_256() -> Self {
        Self::new(256)
    }

    /// 384-dimensional fast hash embedding (matches MiniLM dimensions)
    pub fn default_384() -> Self {
        Self::new(384)
    }

    /// 64-bit FNV-1a hash implementation
    #[inline]
    fn fnv1a_hash(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for &byte in data {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for FastHashEmbeddingProvider {
    fn default() -> Self {
        Self::default_256()
    }
}

#[async_trait]
impl EmbeddingProvider for FastHashEmbeddingProvider {
    fn provider_id(&self) -> &'static str {
        "fasthash"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let mut vector = vec![0.0f32; self.dimensions];
        if text.trim().is_empty() {
            return Ok(vector);
        }

        let lower = text.to_lowercase();
        let words: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric() && c != '_').filter(|s| !s.is_empty()).collect();

        // 1. Word tokens (weight = 2.0)
        for word in &words {
            let h = Self::fnv1a_hash(word.as_bytes());
            let idx = (h as usize) % self.dimensions;
            let sign = if (h >> 31) & 1 == 1 { 1.0 } else { -1.0 };
            vector[idx] += sign * 2.0;
        }

        // 2. Character 3-grams across the text (weight = 1.0)
        let chars: Vec<char> = lower.chars().collect();
        if chars.len() >= 3 {
            for window in chars.windows(3) {
                let s: String = window.iter().collect();
                let h = Self::fnv1a_hash(s.as_bytes());
                let idx = (h as usize) % self.dimensions;
                let sign = if (h >> 31) & 1 == 1 { 1.0 } else { -1.0 };
                vector[idx] += sign * 1.0;
            }
        }

        // 3. Word bigrams (weight = 1.5)
        if words.len() >= 2 {
            for pair in words.windows(2) {
                let s = format!("{}_{}", pair[0], pair[1]);
                let h = Self::fnv1a_hash(s.as_bytes());
                let idx = (h as usize) % self.dimensions;
                let sign = if (h >> 31) & 1 == 1 { 1.0 } else { -1.0 };
                vector[idx] += sign * 1.5;
            }
        }

        // Normalize to unit L2 length
        normalize_vector(&mut vector);
        Ok(vector)
    }
}

// =========================================================================
// 2. OllamaEmbeddingProvider (Local Neural Embeddings)
// =========================================================================

/// Local neural embeddings via Ollama (e.g. nomic-embed-text or all-minilm)
#[derive(Debug, Clone)]
pub struct OllamaEmbeddingProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
    dimensions: usize,
}

#[derive(Serialize)]
struct OllamaEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

impl OllamaEmbeddingProvider {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>, dimensions: usize) -> Self {
        Self {
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            model: model.into(),
            client: reqwest::Client::new(),
            dimensions,
        }
    }

    pub fn default_nomic() -> Self {
        let endpoint = env::var("OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://localhost:11434".to_string());
        Self::new(endpoint, "nomic-embed-text", 768)
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.endpoint);
        let req_body = OllamaEmbedRequest {
            model: &self.model,
            prompt: text,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse(
                "ollama_embed".to_string(),
                format!("Ollama returned HTTP {status}: {err_text}"),
            ));
        }

        let parsed: OllamaEmbedResponse = resp.json().await.map_err(TagisanError::Network)?;
        let mut emb = parsed.embedding;
        normalize_vector(&mut emb);
        Ok(emb)
    }
}

// =========================================================================
// 3. OpenAiEmbeddingProvider (OpenAI text-embedding-3-small)
// =========================================================================

/// Cloud neural embeddings via OpenAI text-embedding-3-small
#[derive(Debug, Clone)]
pub struct OpenAiEmbeddingProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
    dimensions: usize,
}

#[derive(Serialize)]
struct OpenAiEmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct OpenAiEmbedItem {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OpenAiEmbedResponse {
    data: Vec<OpenAiEmbedItem>,
}

impl OpenAiEmbeddingProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let model_str = model.into();
        let dims = if model_str.contains("large") { 3072 } else { 1536 };
        Self {
            api_key: api_key.into(),
            model: model_str,
            client: reqwest::Client::new(),
            dimensions: dims,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let url = "https://api.openai.com/v1/embeddings";
        let req_body = OpenAiEmbedRequest {
            model: &self.model,
            input: text,
        };

        let resp = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&req_body)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse(
                "openai_embed".to_string(),
                format!("OpenAI returned HTTP {status}: {err_text}"),
            ));
        }

        let parsed: OpenAiEmbedResponse = resp.json().await.map_err(TagisanError::Network)?;
        let first = parsed.data.into_iter().next().ok_or_else(|| {
            TagisanError::BadResponse("openai_embed".to_string(), "Empty embedding array returned".to_string())
        })?;

        let mut emb = first.embedding;
        normalize_vector(&mut emb);
        Ok(emb)
    }
}

// =========================================================================
// 4. GeminiEmbeddingProvider (Google text-embedding-004)
// =========================================================================

/// Cloud neural embeddings via Google Gemini text-embedding-004
#[derive(Debug, Clone)]
pub struct GeminiEmbeddingProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
    dimensions: usize,
}

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiEmbedRequest<'a> {
    content: GeminiContent<'a>,
}

#[derive(Deserialize)]
struct GeminiEmbeddingValues {
    values: Vec<f32>,
}

#[derive(Deserialize)]
struct GeminiEmbedResponse {
    embedding: GeminiEmbeddingValues,
}

impl GeminiEmbeddingProvider {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: reqwest::Client::new(),
            dimensions: 768,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
    fn provider_id(&self) -> &'static str {
        "gemini"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
            self.model, self.api_key
        );

        let req_body = GeminiEmbedRequest {
            content: GeminiContent {
                parts: vec![GeminiPart { text }],
            },
        };

        let resp = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse(
                "gemini_embed".to_string(),
                format!("Gemini returned HTTP {status}: {err_text}"),
            ));
        }

        let parsed: GeminiEmbedResponse = resp.json().await.map_err(TagisanError::Network)?;
        let mut emb = parsed.embedding.values;
        normalize_vector(&mut emb);
        Ok(emb)
    }
}

// =========================================================================
// Factory Function
// =========================================================================

/// Resolve the best available embedding provider:
/// 1. OpenAI (if `OPENAI_API_KEY` is set)
/// 2. Gemini (if `GEMINI_API_KEY` is set)
/// 3. Offline FastHash (reliable fallback with zero external dependencies)
pub fn default_embedding_provider() -> Arc<dyn EmbeddingProvider> {
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        if !key.trim().is_empty() {
            debug!("Selected OpenAI text-embedding-3-small as default embedding provider");
            return Arc::new(OpenAiEmbeddingProvider::new(key, "text-embedding-3-small"));
        }
    }

    if let Ok(key) = env::var("GEMINI_API_KEY") {
        if !key.trim().is_empty() {
            debug!("Selected Gemini text-embedding-004 as default embedding provider");
            return Arc::new(GeminiEmbeddingProvider::new(key, "text-embedding-004"));
        }
    }

    debug!("Falling back to FastHashEmbeddingProvider (100% offline, zero-dependency)");
    Arc::new(FastHashEmbeddingProvider::default())
}
