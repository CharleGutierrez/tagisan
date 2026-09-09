use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tagisan::memory::{
    cosine_similarity, dot_product, l2_norm, CodeChunker, CodebaseIndexer, EmbeddingProvider,
    FastHashEmbeddingProvider, VectorDocument, VectorStore,
};
use tagisan::tools::builtin::{SaveMemoryTool, SearchMemoryTool};
use tagisan::{
    AutonomousAgent, BoxEventStream, CompletionRequest, CompletionResponse, EngineContext,
    FinishReason, LlmProvider, Message, ProviderCapabilities, TokenUsage, ToolHandler, ToolRegistry,
};

// =========================================================================
// 1. Vector Mathematics Tests
// =========================================================================

#[test]
fn test_vector_math_norm_and_dot_product() {
    let a = [3.0f32, 4.0f32];
    assert!((l2_norm(&a) - 5.0).abs() < 1e-6);

    let b = [1.0f32, 2.0f32];
    let dot = dot_product(&a, &b);
    assert!((dot - 11.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_edge_cases() {
    let zero = [0.0f32, 0.0f32];
    let a = [1.0f32, 0.0f32];
    let b = [0.0f32, 1.0f32];
    let c = [1.0f32, 0.0f32];
    let neg_c = [-1.0f32, 0.0f32];

    // Orthogonal
    assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);

    // Identical
    assert!((cosine_similarity(&a, &c) - 1.0).abs() < 1e-6);

    // Opposite
    assert!((cosine_similarity(&a, &neg_c) - (-1.0)).abs() < 1e-6);

    // Zero vector
    assert_eq!(cosine_similarity(&zero, &a), 0.0);

    // Empty vector
    assert_eq!(cosine_similarity(&[], &a), 0.0);
}

// =========================================================================
// 2. FastHash Embedding Provider Tests
// =========================================================================

#[tokio::test]
async fn test_fasthash_embedding_properties() {
    let provider = FastHashEmbeddingProvider::default_256();
    assert_eq!(provider.dimensions(), 256);
    assert_eq!(provider.provider_id(), "fasthash");

    let text1 = "fn calculate_total(price: f64, tax: f64) -> f64 { price * (1.0 + tax) }";
    let text2 = "fn compute_sum(price: f64, tax: f64) -> f64 { price * (1.0 + tax) }";
    let text3 = "struct DatabaseConnectionPool { host: String, port: u16 }";

    let emb1 = provider.embed_text(text1).await.unwrap();
    let emb2 = provider.embed_text(text2).await.unwrap();
    let emb3 = provider.embed_text(text3).await.unwrap();

    // Unit length check
    assert!((l2_norm(&emb1) - 1.0).abs() < 1e-4);
    assert!((l2_norm(&emb2) - 1.0).abs() < 1e-4);
    assert!((l2_norm(&emb3) - 1.0).abs() < 1e-4);

    // Deterministic check: same text produces identical embedding
    let emb1_repeat = provider.embed_text(text1).await.unwrap();
    assert_eq!(emb1, emb1_repeat);

    // Semantic overlap: text1 and text2 have high similarity, text1 and text3 have lower
    let sim_1_2 = cosine_similarity(&emb1, &emb2);
    let sim_1_3 = cosine_similarity(&emb1, &emb3);
    assert!(
        sim_1_2 > sim_1_3,
        "Similar price calculation code should have higher similarity than DB connection struct: {sim_1_2} vs {sim_1_3}"
    );
}

// =========================================================================
// 3. CodeChunker Tests
// =========================================================================

#[test]
fn test_code_chunker_sliding_window_and_metadata() {
    let chunker = CodeChunker::new(5, 2); // 5 lines per chunk, 2 lines overlap (step = 3)
    let content = (1..=10)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let chunks = chunker.chunk_text("src/math.rs", &content);

    // Chunk 1: Lines 1..=5
    // Chunk 2: Lines 4..=8
    // Chunk 3: Lines 7..=10
    assert_eq!(chunks.len(), 3);

    assert_eq!(chunks[0].metadata.start_line, 1);
    assert_eq!(chunks[0].metadata.end_line, 5);
    assert_eq!(chunks[0].metadata.language, "rust");

    assert_eq!(chunks[1].metadata.start_line, 4);
    assert_eq!(chunks[1].metadata.end_line, 8);

    assert_eq!(chunks[2].metadata.start_line, 7);
    assert_eq!(chunks[2].metadata.end_line, 10);
}

// =========================================================================
// 4. VectorStore Tests (Search, Persistence, Reload)
// =========================================================================

#[tokio::test]
async fn test_vector_store_persistence_and_search() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mem_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let mem_file = temp_dir.join("memory.json");

    let provider = FastHashEmbeddingProvider::default_256();
    let store = VectorStore::new();

    let doc1_text = "fn authenticate_user(token: &str) -> bool { token == \"secret\" }";
    let doc2_text = "fn render_tui(screen: &mut Screen) { screen.clear(); }";

    let emb1 = provider.embed_text(doc1_text).await.unwrap();
    let emb2 = provider.embed_text(doc2_text).await.unwrap();

    store.add_document(VectorDocument::new("doc1", doc1_text, emb1)).unwrap();
    store.add_document(VectorDocument::new("doc2", doc2_text, emb2)).unwrap();

    assert_eq!(store.len(), 2);

    // Save to disk
    store.save_to_file(&mem_file).unwrap();
    assert!(mem_file.exists());

    // Load from disk into new store
    let loaded = VectorStore::load_from_file(&mem_file).unwrap();
    assert_eq!(loaded.len(), 2);

    // Query for authentication
    let query_emb = provider.embed_text("user auth verification").await.unwrap();
    let hits = loaded.search(&query_emb, 2, 0.1);

    assert!(!hits.is_empty());
    assert_eq!(hits[0].document.id, "doc1", "doc1 should rank #1 for user authentication query");

    let _ = fs::remove_dir_all(&temp_dir);
}

// =========================================================================
// 5. CodebaseIndexer Tests
// =========================================================================

#[tokio::test]
async fn test_codebase_indexer_directory_traversal() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_idx_test_{}", std::process::id()));
    fs::create_dir_all(temp_dir.join("src")).unwrap();
    fs::create_dir_all(temp_dir.join("target")).unwrap(); // Should be ignored

    let rs_file = temp_dir.join("src").join("lib.rs");
    let py_file = temp_dir.join("src").join("script.py");
    let bin_file = temp_dir.join("src").join("asset.png");
    let target_file = temp_dir.join("target").join("build.rs");

    fs::write(&rs_file, "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    fs::write(&py_file, "def subtract(a, b):\n    return a - b\n").unwrap();
    fs::write(&bin_file, [0x89, 0x50, 0x4E, 0x47]).unwrap();
    fs::write(&target_file, "// should be ignored\n").unwrap();

    let provider = Arc::new(FastHashEmbeddingProvider::default_256());
    let indexer = CodebaseIndexer::new(provider.clone());
    let store = VectorStore::new();

    let indexed_count = indexer.index_directory(&temp_dir, &store).await.unwrap();
    assert_eq!(indexed_count, 2, "Only src/lib.rs and src/script.py should be indexed");

    let query_emb = provider.embed_text("python subtract").await.unwrap();
    let hits = store.search(&query_emb, 1, 0.1);
    assert!(!hits.is_empty());
    assert!(hits[0].document.text.contains("def subtract"));

    let _ = fs::remove_dir_all(&temp_dir);
}

// =========================================================================
// 6. EpisodicMemory & Memory Tools Tests
// =========================================================================

#[tokio::test]
async fn test_episodic_memory_and_tools_execution() {
    let provider = Arc::new(FastHashEmbeddingProvider::default_256());
    let store = Arc::new(VectorStore::new());

    // 1. SaveMemoryTool
    let save_tool = SaveMemoryTool::new(store.clone(), provider.clone());
    let save_args = json!({
        "tag": "architecture",
        "summary": "Switched to FastHash embedding fallback for offline reliability",
        "details": "FastHash uses 256-dim word and n-gram hashing to guarantee zero network dependency."
    });

    let save_res = save_tool.execute(save_args).await.unwrap();
    assert!(save_res.contains("Successfully saved episodic memory item"));
    assert_eq!(store.len(), 1);

    // 2. SearchMemoryTool
    let search_tool = SearchMemoryTool::new(store.clone(), provider.clone());
    let search_args = json!({
        "query": "offline embedding fallback",
        "top_k": 1
    });

    let search_res = search_tool.execute(search_args).await.unwrap();
    assert!(search_res.contains("FastHash"));
    assert!(search_res.contains("Score:"));
}

// =========================================================================
// 7. AutonomousAgent Integration with Memory
// =========================================================================

struct MockEchoProvider;

#[async_trait]
impl LlmProvider for MockEchoProvider {
    fn provider_id(&self) -> &'static str {
        "mock_echo"
    }
    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }
    async fn complete(&self, req: CompletionRequest) -> tagisan::Result<CompletionResponse> {
        let last_prompt = req.messages.last().map(|m| m.extract_text()).unwrap_or_default();
        Ok(CompletionResponse {
            id: "echo_1".to_string(),
            provider: "mock".to_string(),
            model: req.model,
            message: Message::assistant(format!("Processed prompt with context: {last_prompt}")),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage::default(),
            latency: Duration::from_millis(1),
        })
    }
    async fn stream(&self, _req: CompletionRequest) -> tagisan::Result<BoxEventStream> {
        Err(tagisan::TagisanError::Execution("Stream unsupported".to_string()))
    }
}

#[tokio::test]
async fn test_autonomous_agent_with_memory_auto_context_injection() {
    let memory_store = Arc::new(VectorStore::new());
    let provider = Arc::new(FastHashEmbeddingProvider::default_256());

    let doc = VectorDocument::new(
        "secret_config",
        "DATABASE_PORT = 5432 and SSL_MODE = require",
        provider.embed_text("database port config ssl").await.unwrap(),
    );
    memory_store.add_document(doc).unwrap();

    let mock_llm = Arc::new(MockEchoProvider);
    let agent = AutonomousAgent::new(mock_llm, "mock-model", ToolRegistry::new())
        .with_memory(memory_store.clone(), provider.clone());

    assert!(agent.tools.has_tool("search_memory"));
    assert!(agent.tools.has_tool("save_memory"));

    let ctx = EngineContext::new(10.0);
    let result = agent.run("What is the database port config?", &ctx).await.unwrap();

    assert!(
        result.final_answer.contains("LONG-TERM MEMORY & CODEBASE CONTEXT"),
        "Agent prompt must be augmented with recalled memory context"
    );
    assert!(result.final_answer.contains("DATABASE_PORT = 5432"));
}
