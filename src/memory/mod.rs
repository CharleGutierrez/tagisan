pub mod backend;
pub mod chunking;
pub mod embedding;
pub mod episodic;
pub mod index;
pub mod store;

pub use backend::{BackendSearchResult, BackendVectorDocument, VectorStoreBackend};
pub use backend::local::LocalVectorBackend;
pub use chunking::{Chunk, ChunkMetadata, CodeChunker};
pub use embedding::{
    cosine_similarity, default_embedding_provider, dot_product, l2_norm, normalize_vector,
    EmbeddingProvider, FastHashEmbeddingProvider, GeminiEmbeddingProvider, OllamaEmbeddingProvider,
    OpenAiEmbeddingProvider,
};
pub use episodic::EpisodicMemory;
pub use index::CodebaseIndexer;
pub use store::{MemoryStats, SearchResult, VectorDocument, VectorStore};
