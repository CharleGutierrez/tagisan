use crate::agent::AutonomousAgent;
use crate::dag::planner::WorkflowPlanner;
use crate::dag::scheduler::DagScheduler;
use crate::ecc::build_ecc_pipeline;
use crate::engine::EngineContext;
use crate::error::Result;
use crate::mcp::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpContentBlock, McpInitializeResult, McpServerInfo,
    McpToolCallResult, McpToolDefinition,
};
use crate::memory::embedding::{default_embedding_provider, EmbeddingProvider};
use crate::memory::index::CodebaseIndexer;
use crate::memory::store::VectorStore;
use crate::providers::LlmProvider;
use crate::strategies::debate::DialecticalDebateStrategy;
use crate::strategies::moa::MixtureOfAgentsStrategy;
use crate::strategies::{CollaborationStrategy, StrategyInput};
use crate::tools::ToolRegistry;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};
use tracing::debug;

pub const SERVER_PROTOCOL_VERSION: &str = "2024-11-05";
pub const SERVER_NAME: &str = "tagisan";
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// High-performance Model Context Protocol (MCP) Server that exposes
/// Tagisan's multi-model consensus, DAG workflows, and codebase RAG
/// as native tools to any external MCP client (Claude Desktop, Cursor, Zed, Windsurf).
pub struct McpServer {
    ctx: Arc<EngineContext>,
    tools: ToolRegistry,
    memory_store: Arc<VectorStore>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl McpServer {
    /// Create a new McpServer with context, tools, and memory
    pub fn new(
        ctx: Arc<EngineContext>,
        tools: ToolRegistry,
        memory_store: Arc<VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            ctx,
            tools,
            memory_store,
            embedding_provider,
        }
    }

    /// Construct a default McpServer initialized with active environment context
    pub fn default_server() -> Self {
        let max_budget = std::env::var("MAX_BUDGET")
            .ok()
            .and_then(|b| b.parse().ok())
            .unwrap_or(5.00);

        let mut ctx = EngineContext::new(max_budget);

        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.trim().is_empty() {
                ctx.register_provider(Arc::new(crate::providers::anthropic::AnthropicProvider::new(key)));
            }
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if !key.trim().is_empty() {
                ctx.register_provider(Arc::new(crate::providers::openai_compat::OpenAiCompatibleProvider::openai(key)));
            }
        }
        if let Ok(key) = std::env::var("XAI_API_KEY") {
            if !key.trim().is_empty() {
                ctx.register_provider(Arc::new(crate::providers::openai_compat::OpenAiCompatibleProvider::xai(key)));
            }
        }
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            if !key.trim().is_empty() {
                ctx.register_provider(Arc::new(crate::providers::openai_compat::OpenAiCompatibleProvider::deepseek(key)));
            }
        }
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.trim().is_empty() {
                ctx.register_provider(Arc::new(crate::providers::gemini::GeminiProvider::new(key)));
            }
        }
        ctx.register_provider(Arc::new(crate::providers::ollama::OllamaProvider::default_local()));

        let tools = ToolRegistry::with_builtins();
        let memory_store = Arc::new(VectorStore::load_or_default());
        let embedding_provider = default_embedding_provider();

        Self::new(Arc::new(ctx), tools, memory_store, embedding_provider)
    }

    /// Return catalog of tools exposed by this MCP server
    pub fn list_tools(&self) -> Vec<McpToolDefinition> {
        let mut defs = vec![
            McpToolDefinition {
                name: "tagisan_debate".to_string(),
                description: Some(
                    "Execute Dialectical Debate (Tagisan ng Talino / Balagtasan) between AI models: Round 1 Thesis, Round 2 Adversarial Critique, Round 3 Lakandiwa Master Synthesis.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The technical decision, architecture question, code bug hypothesis, or problem to debate"
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_moa".to_string(),
                description: Some(
                    "Execute Mixture-of-Agents (MoA) architecture: queries multiple LLMs concurrently and synthesizes candidate responses into a unified definitive answer.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The coding task, prompt, or technical problem to solve"
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_agent".to_string(),
                description: Some(
                    "Run an Autonomous Multi-Turn ReAct Agent with built-in tools (files, bash, math, image view) and optional codebase vector memory.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The task or objective for the autonomous agent"
                        },
                        "memory": {
                            "type": "boolean",
                            "description": "Whether to auto-inject relevant codebase context from vector memory (default: false)"
                        }
                    },
                    "required": ["prompt"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_workflow_plan".to_string(),
                description: Some(
                    "Decompose a complex engineering objective into an optimal, parallelizable Directed Acyclic Graph (DAG) workflow and execute it.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "goal": {
                            "type": "string",
                            "description": "The complex engineering or coding objective to decompose"
                        },
                        "concurrency": {
                            "type": "integer",
                            "description": "Maximum parallel task execution limit (default: 4)"
                        }
                    },
                    "required": ["goal"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_ecc_pipeline".to_string(),
                description: Some(
                    "Execute the 5-stage ECC Autonomous Multi-Agent Engineering Pipeline (Plan -> Test -> Implement -> Review/Security -> Verify).".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "objective": {
                            "type": "string",
                            "description": "The feature, module, or fix to design, test, implement, and verify"
                        }
                    },
                    "required": ["objective"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_memory_search".to_string(),
                description: Some(
                    "Search indexed codebase chunks and persistent episodic memory using fast semantic vector similarity.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Semantic search query or concept"
                        },
                        "top_k": {
                            "type": "integer",
                            "description": "Maximum number of matches to return (default: 5)"
                        },
                        "threshold": {
                            "type": "number",
                            "description": "Minimum similarity score between 0.0 and 1.0 (default: 0.05)"
                        }
                    },
                    "required": ["query"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_memory_index".to_string(),
                description: Some(
                    "Index source files in a workspace directory into Tagisan's persistent vector memory with .gitignore filtering.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path to index (defaults to '.')"
                        }
                    },
                    "required": ["path"]
                }),
            },
            McpToolDefinition {
                name: "tagisan_status".to_string(),
                description: Some(
                    "Inspect available Tagisan LLM providers, registered API keys, and model capability bitflags.".to_string(),
                ),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ];

        // Also expose any registered built-in tools
        for def in self.tools.definitions() {
            defs.push(McpToolDefinition {
                name: def.name,
                description: Some(def.description),
                input_schema: def.parameters,
            });
        }

        defs
    }

    /// Resolve all available providers and default models for multi-agent collaboration
    fn resolve_available_providers(&self) -> Vec<(String, String, Arc<dyn LlmProvider>)> {
        let candidate_keys = [
            ("gemini", "GEMINI_API_KEY", "gemini-2.0-flash"),
            ("deepseek", "DEEPSEEK_API_KEY", "deepseek-chat"),
            ("anthropic", "ANTHROPIC_API_KEY", "claude-3-5-sonnet-20241022"),
            ("openai", "OPENAI_API_KEY", "gpt-4o"),
            ("xai", "XAI_API_KEY", "grok-2-latest"),
        ];

        let mut list = Vec::new();
        for (id, key_var, def_model) in candidate_keys {
            if std::env::var(key_var).is_ok() {
                if let Ok(prov) = self.ctx.get_provider(id) {
                    list.push((id.to_string(), def_model.to_string(), prov));
                }
            }
        }

        if let Ok(prov) = self.ctx.get_provider("ollama") {
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:1.5b".to_string());
            list.push(("ollama".to_string(), model, prov));
        }

        list
    }

    /// Resolve default primary provider for autonomous tools
    fn resolve_primary_provider(&self) -> Result<(String, String, Arc<dyn LlmProvider>)> {
        let available = self.resolve_available_providers();
        if let Some(first) = available.into_iter().next() {
            return Ok(first);
        }

        let prov = self.ctx.get_provider("ollama")?;
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:1.5b".to_string());
        Ok(("ollama".to_string(), model, prov))
    }

    /// Execute an MCP tool call by name
    pub async fn execute_tool(&self, name: &str, arguments: Value) -> McpToolCallResult {
        match name {
            "tagisan_debate" => {
                let prompt = match arguments.get("prompt").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'prompt'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let available = self.resolve_available_providers();
                let (proponent, adversary, lakandiwa) = if available.len() >= 3 {
                    (
                        (available[0].0.clone(), available[0].1.clone()),
                        (available[1].0.clone(), available[1].1.clone()),
                        (available[2].0.clone(), available[2].1.clone()),
                    )
                } else if available.len() == 2 {
                    (
                        (available[0].0.clone(), available[0].1.clone()),
                        (available[1].0.clone(), available[1].1.clone()),
                        (available[0].0.clone(), available[0].1.clone()),
                    )
                } else {
                    let (p_id, p_model, _) = match self.resolve_primary_provider() {
                        Ok(p) => p,
                        Err(e) => {
                            return McpToolCallResult {
                                content: vec![McpContentBlock::Text {
                                    text: format!("Error resolving provider for debate: {e}"),
                                }],
                                is_error: true,
                            };
                        }
                    };
                    (
                        (p_id.clone(), p_model.clone()),
                        (p_id.clone(), p_model.clone()),
                        (p_id, p_model),
                    )
                };

                let strategy = DialecticalDebateStrategy::new(proponent, adversary, lakandiwa);

                let input = StrategyInput {
                    prompt: prompt.to_string(),
                    system_instruction: None,
                };

                match strategy.execute(input, &self.ctx).await {
                    Ok(output) => {
                        let mut text = format!(
                            "## ⚔️ Tagisan Dialectical Debate Verdict\n\n{}\n\n---\n**Rounds Evaluated:**\n",
                            output.final_answer
                        );
                        for step in &output.intermediate_steps {
                            text.push_str(&format!("- **{}** (Model: {})\n", step.step_name, step.model));
                        }
                        text.push_str(&format!("\n*Cost: ${:.4} | Latency: {:.2}s*", output.total_cost_usd, output.total_latency.as_secs_f32()));

                        McpToolCallResult {
                            content: vec![McpContentBlock::Text { text }],
                            is_error: false,
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Debate execution failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_moa" => {
                let prompt = match arguments.get("prompt").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'prompt'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let available = self.resolve_available_providers();
                let (proposers, aggregator) = if available.len() >= 2 {
                    let mut props = Vec::new();
                    for item in &available[..available.len().min(3)] {
                        props.push((item.0.clone(), item.1.clone()));
                    }
                    let agg = (available[0].0.clone(), available[0].1.clone());
                    (props, agg)
                } else {
                    let (p_id, p_model, _) = match self.resolve_primary_provider() {
                        Ok(p) => p,
                        Err(e) => {
                            return McpToolCallResult {
                                content: vec![McpContentBlock::Text {
                                    text: format!("Error resolving provider for MoA: {e}"),
                                }],
                                is_error: true,
                            };
                        }
                    };
                    (
                        vec![(p_id.clone(), p_model.clone()), (p_id.clone(), p_model.clone())],
                        (p_id, p_model),
                    )
                };

                let strategy = MixtureOfAgentsStrategy::new(proposers, aggregator);

                let input = StrategyInput {
                    prompt: prompt.to_string(),
                    system_instruction: None,
                };

                match strategy.execute(input, &self.ctx).await {
                    Ok(output) => {
                        let text = format!(
                            "## 🛖 Mixture-of-Agents (MoA) Synthesis\n\n{}\n\n---\n*Cost: ${:.4} | Latency: {:.2}s*",
                            output.final_answer, output.total_cost_usd, output.total_latency.as_secs_f32()
                        );
                        McpToolCallResult {
                            content: vec![McpContentBlock::Text { text }],
                            is_error: false,
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("MoA execution failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_agent" => {
                let prompt = match arguments.get("prompt").and_then(|v| v.as_str()) {
                    Some(p) => p,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'prompt'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let memory_enabled = arguments.get("memory").and_then(|v| v.as_bool()).unwrap_or(false);

                let (_, p_model, provider) = match self.resolve_primary_provider() {
                    Ok(p) => p,
                    Err(e) => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: format!("Error resolving provider for agent: {e}"),
                            }],
                            is_error: true,
                        };
                    }
                };

                let mut agent = AutonomousAgent::new(provider, p_model, self.tools.clone())
                    .with_max_iterations(10);

                if memory_enabled {
                    agent = agent.with_memory(self.memory_store.clone(), self.embedding_provider.clone());
                }

                match agent.run(prompt, &self.ctx).await {
                    Ok(res) => {
                        let text = format!(
                            "## 🤖 Autonomous Agent Execution Result\n\n{}\n\n---\n*Iterations: {} | Tokens: {} prompt, {} completion | Cost: ${:.4}*",
                            res.final_answer,
                            res.iterations,
                            res.total_usage.prompt_tokens,
                            res.total_usage.completion_tokens,
                            res.total_cost_usd
                        );
                        McpToolCallResult {
                            content: vec![McpContentBlock::Text { text }],
                            is_error: false,
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Autonomous agent failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_workflow_plan" => {
                let goal = match arguments.get("goal").and_then(|v| v.as_str()) {
                    Some(g) => g,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'goal'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let concurrency = arguments.get("concurrency").and_then(|v| v.as_u64()).unwrap_or(4) as usize;

                let (_, p_model, provider) = match self.resolve_primary_provider() {
                    Ok(p) => p,
                    Err(e) => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: format!("Error resolving provider for workflow: {e}"),
                            }],
                            is_error: true,
                        };
                    }
                };

                let planner = WorkflowPlanner::new(provider, p_model).with_tools(self.tools.clone());

                match planner.plan(goal, &self.ctx).await {
                    Ok(mut graph) => {
                        let scheduler = DagScheduler::new().with_concurrency_limit(concurrency);
                        match scheduler.run(&mut graph, &self.ctx).await {
                            Ok(workflow_result) => {
                                let mut summary = format!(
                                    "## 🕸️ DAG Workflow Execution Completed\n\n**Total Tasks:** {}\n\n",
                                    workflow_result.task_outputs.len()
                                );
                                for (task_id, output) in &workflow_result.task_outputs {
                                    summary.push_str(&format!("### Task: `{task_id}`\n{}\n\n", output.text.trim()));
                                }
                                McpToolCallResult {
                                    content: vec![McpContentBlock::Text { text: summary }],
                                    is_error: false,
                                }
                            }
                            Err(e) => McpToolCallResult {
                                content: vec![McpContentBlock::Text {
                                    text: format!("Workflow execution failed: {e}"),
                                }],
                                is_error: true,
                            },
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Workflow planning failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_ecc_pipeline" => {
                let objective = match arguments.get("objective").and_then(|v| v.as_str()) {
                    Some(o) => o,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'objective'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let (_, p_model, provider) = match self.resolve_primary_provider() {
                    Ok(p) => p,
                    Err(e) => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: format!("Error resolving provider for ECC: {e}"),
                            }],
                            is_error: true,
                        };
                    }
                };

                let graph_res = build_ecc_pipeline(objective, provider, &p_model, self.tools.clone());
                match graph_res {
                    Ok(mut graph) => {
                        let scheduler = DagScheduler::new().with_concurrency_limit(4);
                        match scheduler.run(&mut graph, &self.ctx).await {
                            Ok(res) => {
                                let mut summary = format!(
                                    "## 🏛️ ECC 5-Stage Engineering Pipeline Results\n\n**Objective:** \"{}\"\n\n",
                                    objective
                                );
                                for (task_id, output) in &res.task_outputs {
                                    summary.push_str(&format!("### Stage: `{}`\n{}\n\n", task_id, output.text.trim()));
                                }
                                McpToolCallResult {
                                    content: vec![McpContentBlock::Text { text: summary }],
                                    is_error: false,
                                }
                            }
                            Err(e) => McpToolCallResult {
                                content: vec![McpContentBlock::Text {
                                    text: format!("ECC pipeline execution failed: {e}"),
                                }],
                                is_error: true,
                            },
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("ECC pipeline building failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_memory_search" => {
                let query = match arguments.get("query").and_then(|v| v.as_str()) {
                    Some(q) => q,
                    None => {
                        return McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: "Error: Missing required parameter 'query'".to_string(),
                            }],
                            is_error: true,
                        };
                    }
                };

                let top_k = arguments.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let threshold = arguments.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.05) as f32;

                match self.embedding_provider.embed_text(query).await {
                    Ok(emb) => {
                        let hits = self.memory_store.search(&emb, top_k, threshold);
                        if hits.is_empty() {
                            McpToolCallResult {
                                content: vec![McpContentBlock::Text {
                                    text: format!("No memory matches found for query: \"{query}\" above threshold {threshold:.2}"),
                                }],
                                is_error: false,
                            }
                        } else {
                            let mut text = format!("Found {} relevant chunk(s) for \"{}\":\n\n", hits.len(), query);
                            for (i, hit) in hits.iter().enumerate() {
                                let doc = &hit.document;
                                let path = doc.metadata.get("file_path").map(|s| s.as_str()).unwrap_or(&doc.id);
                                let start = doc.metadata.get("start_line").map(|s| s.as_str()).unwrap_or("?");
                                let end = doc.metadata.get("end_line").map(|s| s.as_str()).unwrap_or("?");
                                text.push_str(&format!(
                                    "**{}. {} (Lines {}-{}, Score: {:.3})**\n```\n{}\n```\n\n",
                                    i + 1, path, start, end, hit.score, doc.text.trim()
                                ));
                            }
                            McpToolCallResult {
                                content: vec![McpContentBlock::Text { text }],
                                is_error: false,
                            }
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Embedding generation failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_memory_index" => {
                let path = arguments.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let indexer = CodebaseIndexer::new(self.embedding_provider.clone());

                match indexer.index_directory(path, &self.memory_store).await {
                    Ok(count) => {
                        let default_path = VectorStore::default_path();
                        let _ = self.memory_store.save_to_file(&default_path);
                        McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: format!(
                                    "Successfully indexed {} total codebase chunks from '{}' into persistent vector store ({})",
                                    count, path, default_path.display()
                                ),
                            }],
                            is_error: false,
                        }
                    }
                    Err(e) => McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Codebase indexing failed: {e}"),
                        }],
                        is_error: true,
                    },
                }
            }

            "tagisan_status" => {
                let default_path = VectorStore::default_path();
                let stats = self.memory_store.stats(Some(&default_path));
                let status_text = format!(
                    "## 🇵🇭 Tagisan Engine Status\n\n- **Version:** {}\n- **MCP Protocol:** {}\n- **Active Budget Spent:** ${:.4}\n- **Indexed Memory Documents:** {}\n- **Embedding Dimension:** {}\n- **Registered Tools:** {}\n",
                    SERVER_VERSION,
                    SERVER_PROTOCOL_VERSION,
                    self.ctx.budget_tracker.current_spent_usd(),
                    stats.total_documents,
                    stats.embedding_dimensions,
                    self.tools.len()
                );
                McpToolCallResult {
                    content: vec![McpContentBlock::Text { text: status_text }],
                    is_error: false,
                }
            }

            // Fallback to built-in tools in registry
            other => {
                if let Some(tool) = self.tools.get(other) {
                    match tool.execute(arguments).await {
                        Ok(res) => McpToolCallResult {
                            content: vec![McpContentBlock::Text { text: res }],
                            is_error: false,
                        },
                        Err(e) => McpToolCallResult {
                            content: vec![McpContentBlock::Text {
                                text: format!("Tool '{other}' execution error: {e}"),
                            }],
                            is_error: true,
                        },
                    }
                } else {
                    McpToolCallResult {
                        content: vec![McpContentBlock::Text {
                            text: format!("Unknown tool '{other}'"),
                        }],
                        is_error: true,
                    }
                }
            }
        }
    }

    /// Process a single incoming JSON-RPC 2.0 request and generate response
    pub async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        match req.method.as_str() {
            "initialize" => {
                let result = McpInitializeResult {
                    protocol_version: SERVER_PROTOCOL_VERSION.to_string(),
                    capabilities: json!({
                        "tools": {
                            "listChanged": false
                        }
                    }),
                    server_info: Some(McpServerInfo {
                        name: SERVER_NAME.to_string(),
                        version: Some(SERVER_VERSION.to_string()),
                    }),
                };
                Some(JsonRpcResponse::success(req.id, json!(result)))
            }

            "notifications/initialized" => {
                // Client initialized notification (no response expected)
                None
            }

            "ping" => Some(JsonRpcResponse::success(req.id, json!({}))),

            "tools/list" => {
                let tools = self.list_tools();
                Some(JsonRpcResponse::success(req.id, json!({ "tools": tools })))
            }

            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                let call_result = self.execute_tool(tool_name, arguments).await;
                Some(JsonRpcResponse::success(req.id, json!(call_result)))
            }

            unknown => Some(JsonRpcResponse::error(
                Some(req.id),
                -32601,
                format!("Method not found: '{unknown}'"),
            )),
        }
    }

    /// Run the server loop processing line-delimited JSON-RPC messages from `reader` and writing to `writer`
    pub async fn run_stdio<R, W>(&self, reader: R, mut writer: W) -> Result<()>
    where
        R: AsyncBufRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parsed_json = match serde_json::from_str::<Value>(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    let err_resp = JsonRpcResponse::error(None, -32700, "Parse error");
                    if let Ok(serialized) = serde_json::to_string(&err_resp) {
                        let payload = format!("{}\n", serialized);
                        let _ = writer.write_all(payload.as_bytes()).await;
                        let _ = writer.flush().await;
                    }
                    continue;
                }
            };

            // If it has an "id" field, it is a Request and MUST produce a response
            if let Some(raw_id) = parsed_json.get("id") {
                let req_id: Option<crate::mcp::protocol::RequestId> = serde_json::from_value(raw_id.clone()).ok();
                match serde_json::from_value::<JsonRpcRequest>(parsed_json) {
                    Ok(req) => {
                        if let Some(resp) = self.handle_request(req).await {
                            if let Ok(serialized) = serde_json::to_string(&resp) {
                                let payload = format!("{}\n", serialized);
                                if let Err(e) = writer.write_all(payload.as_bytes()).await {
                                    eprintln!("Failed to write to stdio: {e}");
                                    break;
                                }
                                let _ = writer.flush().await;
                            }
                        }
                    }
                    Err(e) => {
                        let err_resp = JsonRpcResponse::error(req_id, -32600, format!("Invalid Request: {e}"));
                        if let Ok(serialized) = serde_json::to_string(&err_resp) {
                            let payload = format!("{}\n", serialized);
                            let _ = writer.write_all(payload.as_bytes()).await;
                            let _ = writer.flush().await;
                        }
                    }
                }
            } else {
                // No "id" field: standard JSON-RPC 2.0 notification
                if let Ok(notif) = serde_json::from_value::<crate::mcp::protocol::JsonRpcNotification>(parsed_json) {
                    debug!("Received notification: {}", notif.method);
                }
            }
        }

        Ok(())
    }

    /// Run the default standard I/O server loop (reading from stdin, writing to stdout)
    pub async fn run_default_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = tokio::io::BufReader::new(stdin);
        self.run_stdio(reader, stdout).await
    }
}
