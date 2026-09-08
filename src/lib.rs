pub mod agent;
pub mod dag;
pub mod ecc;
pub mod engine;
pub mod error;
pub mod providers;
pub mod strategies;
pub mod tools;
pub mod tui;
pub mod types;

pub use ecc::{
    all_presets as all_ecc_presets, build_ecc_pipeline, find_preset as find_ecc_preset,
    load_agents_from_dir as load_ecc_agents_from_dir, resolve_agent as resolve_ecc_agent,
    EccAgent, EccAuditDebate,
};

pub use agent::{AgentResult, AgentStep, AutonomousAgent};
pub use dag::{
    extract_json_block, interpolate_prompt, DagScheduler, PlannedTask, PlannedWorkflow,
    RetryPolicy, TaskNode, TaskOutput, TaskStatus, WorkflowEvent, WorkflowGraph, WorkflowPlanner,
    WorkflowResult, WorkflowRunner, PLANNER_SYSTEM_PROMPT,
};
pub use engine::budget::TokenBudgetTracker;
pub use engine::EngineContext;
pub use error::{Result, TagisanError};
pub use providers::anthropic::AnthropicProvider;
pub use providers::cascade::{CascadeEntry, CascadeProvider};
pub use providers::gemini::GeminiProvider;
pub use providers::ollama::OllamaProvider;
pub use providers::openai_compat::{OpenAiCompatibleProvider, StreamingThinkParser};
pub use providers::{BoxEventStream, LlmProvider};
pub use strategies::debate::DialecticalDebateStrategy;
pub use strategies::moa::MixtureOfAgentsStrategy;
pub use strategies::{CollaborationStrategy, IntermediateStep, StrategyInput, StrategyOutput};
pub use tools::builtin::{CalculatorTool, ReadFileTool, RunCommandTool, WriteFileTool};
pub use tools::{ToolHandler, ToolRegistry};
pub use tui::run_debate_tui;
pub use types::{
    ChatSession, CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage, ToolDefinition,
};
