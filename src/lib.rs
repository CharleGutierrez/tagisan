pub mod agent;
pub mod bun;
pub mod cli;
pub mod dag;
pub mod ecc;
pub mod engine;
pub mod error;
pub mod eval;
pub mod mcp;
pub mod memory;
pub mod perl;
pub mod providers;
pub mod python;
pub mod strategies;
pub mod swarm;
pub mod telemetry;
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
pub use perl::{PerlExecutionResult, PerlRuntime};
pub use python::{PythonExecutionResult, PythonRuntime};
pub use ecc::{
    all_built_in_skills as all_ecc_skills, all_presets as all_ecc_presets, build_ecc_pipeline,
    extract_triggers_from_text, find_built_in_skill as find_ecc_skill,
    find_preset as find_ecc_preset, format_cheat_sheet, format_cloud_guidelines,
    global_dispatcher as global_ecc_dispatcher, is_local_provider,
    load_agents_from_dir as load_ecc_agents_from_dir,
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
pub use providers::ollama::{default_ollama_model, parse_thinking_blocks, OllamaProvider};
pub use providers::openai_compat::{OpenAiCompatibleProvider, StreamingThinkParser};
pub use providers::{BoxEventStream, LlmProvider};
pub use strategies::debate::DialecticalDebateStrategy;
pub use strategies::harmony::StructuredHarmonyStrategy;
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
pub use tools::perl::{PerlEvalTool, PerlRunTool};
pub use tools::python::{PythonEvalTool, PythonRunTool};
pub use tools::wasm::{load_wasm_tools, WasmTool};
pub use tools::{ToolHandler, ToolRegistry};
pub use swarm::{
    build_standard_harmony_pipeline, extract_markdown_code_blocks, parse_provider_and_model,
    resolve_harmony_models, AgentReview, AgentShieldSecurityGate, AssemblyRoles, ConsensusVerdict,
    DelegateTaskTool, ExtractedCodeBlock, FailoverEvent, GateResult, HarmonyExecutionResult, HarmonyRole,
    HarmonyRoleConfig, HarmonyStage, HarmonyTierProfile, InteractiveRepl, PipelineExecutionResult,
    PipelineStageOutput, ReplCommand, ReviewCriterion, RoleArtifact, RoleModelOverrides,
    SessionMetadata, SessionRecord, SessionStore, StandardHarmonyRole, StructuredHarmonyPipeline,
    SwarmBlackboard, SwarmCoordinator, SwarmMember, SyntaxValidationGate, TeamConsensusEngine,
    ValidationGate, VotingRule,
};
pub use tui::{run_debate_tui, Spinner};
pub use types::{
    ChatSession, CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage, ToolDefinition,
};
pub use vella::{
    DomainActionProposal, VellaAppManager, VellaDebateGovernor, VellaDebateVerdict,
    VellaEventBridgeTool, VellaMedicineTool, VellaPolicyGovernor, VellaRoboticsTool,
    VellaScadaTool, VellaStreamBridge, VellaTradingTool,
};
pub use memory::backend::{
    BackendSearchResult, BackendVectorDocument, HybridSyncBridge, LocalVectorBackend,
    LocalVectorStore, VectorStoreBackend,
};
#[cfg(feature = "pgvector")]
pub use memory::backend::PgVectorStore;
#[cfg(feature = "qdrant")]
pub use memory::backend::QdrantStore;

pub use eval::{
    cosine_score, evaluate_criteria, groundedness_score, run_eval_command, safety_score,
    threshold_pass, EvalArgs, EvalCase, EvalDataset, EvalReport, EvalRunner, EvalScore,
};
pub use telemetry::{
    run_trace_command, run_trace_tui, TagisanTracer, TraceArgs, TraceEvent, TraceJournal, TraceSpan,
};
