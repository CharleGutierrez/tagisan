pub mod agent;
pub mod bun;
pub mod cli;
pub mod dag;
pub mod ecc;
pub mod engine;
pub mod error;
pub mod mcp;
pub mod memory;
pub mod providers;
pub mod strategies;
pub mod swarm;
pub mod tools;
pub mod tui;
pub mod types;
pub mod vella;

pub use bun::{
    tagisan_ffi_blake3_digest, tagisan_ffi_cosine_similarity, tagisan_ffi_shield_scan,
    tagisan_ffi_version, BunExecutionResult, BunRuntime, BunSandbox, BunSandboxConfig, BunWorker,
    BunWorkerPool, DiagnosticSeverity, TagisanSqliteStore, TsDiagnostic, TsDiagnosticParser,
    WorkerPoolStats,
};
pub use ecc::{
    all_built_in_skills as all_ecc_skills, all_presets as all_ecc_presets, build_ecc_pipeline,
    find_built_in_skill as find_ecc_skill, find_preset as find_ecc_preset,
    global_dispatcher as global_ecc_dispatcher, load_agents_from_dir as load_ecc_agents_from_dir,
    load_skills_from_dir as load_ecc_skills_from_dir, resolve_agent as resolve_ecc_agent,
    resolve_skill as resolve_ecc_skill, AgentShieldScanner, AgentShieldVerdict, DispatchedSkill,
    EccAgent, EccAuditDebate, EccSkill, SkillDispatcher, SkillMetadata,
    ThreatLevel as EccThreatLevel,
};

pub use agent::{AgentResult, AgentStep, AutonomousAgent, WorktreeSandbox};
pub use dag::{
    extract_json_block, interpolate_prompt, DagScheduler, PlannedTask, PlannedWorkflow,
    RetryPolicy, TaskNode, TaskOutput, TaskStatus, WorkflowEvent, WorkflowGraph, WorkflowPlanner,
    WorkflowResult, WorkflowRunner, PLANNER_SYSTEM_PROMPT,
};
pub use engine::budget::TokenBudgetTracker;
pub use engine::EngineContext;
pub use error::{Result, TagisanError};
pub use memory::{
    cosine_similarity, default_embedding_provider, dot_product, l2_norm, normalize_vector,
    Chunk, ChunkMetadata, CodeChunker, CodebaseIndexer, EmbeddingProvider, EpisodicMemory,
    FastHashEmbeddingProvider, GeminiEmbeddingProvider, MemoryStats, OllamaEmbeddingProvider,
    OpenAiEmbeddingProvider, SearchResult, VectorDocument, VectorStore,
};
pub use providers::anthropic::AnthropicProvider;
pub use providers::cascade::{CascadeEntry, CascadeProvider};
pub use providers::gemini::GeminiProvider;
pub use providers::ollama::OllamaProvider;
pub use providers::openai_compat::{OpenAiCompatibleProvider, StreamingThinkParser};
pub use providers::{BoxEventStream, LlmProvider};
pub use strategies::debate::DialecticalDebateStrategy;
pub use strategies::moa::MixtureOfAgentsStrategy;
pub use strategies::{CollaborationStrategy, IntermediateStep, StrategyInput, StrategyOutput};
pub use mcp::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpClient, McpConfig,
    McpContentBlock, McpInitializeResult, McpManager, McpServer, McpServerConfig, McpServerInfo,
    McpToolCallResult, McpToolDefinition, McpToolWrapper, StdioTransport,
};
pub use tools::builtin::{
    CalculatorTool, ReadFileTool, RunCommandTool, SaveMemoryTool, SearchMemoryTool,
    SearchSkillsTool, ViewImageTool, WriteFileTool,
};
pub use tools::bun::{
    extract_missing_package, BunAutoResolveTool, BunBuildTool, BunEvalTool, BunHmrTool,
    BunInstallTool, BunRunTool, BunTestTool,
};
pub use tools::bun_compile::BunCompileTool;
pub use tools::bun_serve::{BunServeTool, BunStreamBusTool};
pub use tools::{ToolHandler, ToolRegistry};
pub use swarm::{
    AgentReview, ConsensusVerdict, DelegateTaskTool, InteractiveRepl, PipelineExecutionResult,
    PipelineStageOutput, ReplCommand, ReviewCriterion, SessionMetadata, SessionRecord, SessionStore,
    SwarmCoordinator, SwarmMember, TeamConsensusEngine, VotingRule,
};
pub use tui::run_debate_tui;
pub use types::{
    ChatSession, CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage, ToolDefinition,
};
pub use vella::{
    DomainActionProposal, VellaAppManager, VellaDebateGovernor, VellaDebateVerdict,
    VellaEventBridgeTool, VellaMedicineTool, VellaPolicyGovernor, VellaRoboticsTool,
    VellaScadaTool, VellaStreamBridge, VellaTradingTool,
};
