use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::memory::embedding::EmbeddingProvider;
use crate::memory::store::VectorStore;
use crate::providers::LlmProvider;
use crate::tools::builtin::{SaveMemoryTool, SearchMemoryTool};
use crate::tools::ToolRegistry;
use crate::types::{ChatSession, CompletionRequest, ContentBlock, Message, StreamChunkDelta, TokenUsage, ToolDefinition};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

pub mod sandbox;
pub mod cross_sandbox;
pub use sandbox::WorktreeSandbox;
pub use cross_sandbox::*;

/// Record of an intermediate step in the autonomous execution loop
#[derive(Debug, Clone)]
pub struct AgentStep {
    pub iteration: usize,
    pub assistant_message: Message,
    pub tool_results: Vec<ContentBlock>,
}

/// Comprehensive output produced by an autonomous agent run
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub final_answer: String,
    pub history: Vec<Message>,
    pub iterations: usize,
    pub steps: Vec<AgentStep>,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
    pub total_latency: std::time::Duration,
}

/// Production autonomous agent that executes multi-turn tool-calling loops
#[derive(Clone)]
pub struct AutonomousAgent {
    pub provider: Arc<dyn LlmProvider>,
    pub model: String,
    pub tools: ToolRegistry,
    pub system_prompt: Option<String>,
    pub max_iterations: usize,
    pub temperature: Option<f32>,
    pub agentshield_enabled: bool,
    pub auto_skills_enabled: bool,
    pub memory: Option<Arc<VectorStore>>,
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
    pub working_dir: Option<std::path::PathBuf>,
}

impl AutonomousAgent {
    /// Create a new AutonomousAgent instance (AgentShield security and Auto-Skills enabled by default)
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        model: impl Into<String>,
        tools: ToolRegistry,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            tools,
            system_prompt: None,
            max_iterations: 10,
            temperature: Some(0.7),
            agentshield_enabled: true,
            auto_skills_enabled: true,
            memory: None,
            embedding_provider: None,
            working_dir: None,
        }
    }

    /// Enable or disable automatic skill selection based on prompt semantics (enabled by default)
    pub fn with_auto_skills(mut self, enabled: bool) -> Self {
        self.auto_skills_enabled = enabled;
        self
    }

    /// Enable or disable AgentShield security scanner for tool calls and secret redaction
    pub fn with_agentshield(mut self, enabled: bool) -> Self {
        self.agentshield_enabled = enabled;
        self
    }

    /// Set an isolated working directory / sandbox path for agent execution
    pub fn with_working_dir(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    /// Attach a long-term vector memory store and embedding provider, automatically registering
    /// `search_memory` and `save_memory` tools and enabling semantic context retrieval
    pub fn with_memory(
        mut self,
        memory: Arc<VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        self.tools.register_tool(SearchMemoryTool::new(memory.clone(), embedding_provider.clone()));
        self.tools.register_tool(SaveMemoryTool::new(memory.clone(), embedding_provider.clone()));
        self.memory = Some(memory);
        self.embedding_provider = Some(embedding_provider);
        self
    }

    /// Set system instruction for the agent
    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    /// Set maximum allowed feedback loop iterations
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Set model temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sanitize arbitrary text using AgentShield secret redaction
    pub fn sanitize_text(text: &str) -> String {
        crate::ecc::AgentShieldScanner::redact_secrets(text)
    }

    /// Redact recognized secrets in a single ContentBlock
    pub fn sanitize_content_block(block: ContentBlock) -> ContentBlock {
        match block {
            ContentBlock::Text { text } => ContentBlock::Text {
                text: crate::ecc::AgentShieldScanner::redact_secrets(&text),
            },
            ContentBlock::Thinking { thinking, signature } => ContentBlock::Thinking {
                thinking: crate::ecc::AgentShieldScanner::redact_secrets(&thinking),
                signature,
            },
            ContentBlock::ToolResult { tool_call_id, content, is_error } => ContentBlock::ToolResult {
                tool_call_id,
                content: crate::ecc::AgentShieldScanner::redact_secrets(&content),
                is_error,
            },
            ContentBlock::ToolCall { id, name, arguments } => {
                let args_str = arguments.to_string();
                let redacted_args_str = crate::ecc::AgentShieldScanner::redact_secrets(&args_str);
                let sanitized_args = serde_json::from_str(&redacted_args_str).unwrap_or(arguments);
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments: sanitized_args,
                }
            }
            other => other,
        }
    }

    /// Redact recognized secrets in all content blocks of a Message
    pub fn sanitize_message(mut message: Message) -> Message {
        message.content = message
            .content
            .into_iter()
            .map(Self::sanitize_content_block)
            .collect();
        message
    }

    /// Redact recognized secrets in an AgentStep (assistant message and tool results)
    pub fn sanitize_step(mut step: AgentStep) -> AgentStep {
        step.assistant_message = Self::sanitize_message(step.assistant_message);
        step.tool_results = step
            .tool_results
            .into_iter()
            .map(Self::sanitize_content_block)
            .collect();
        step
    }

    /// Redact secrets in an entire AgentResult
    pub fn sanitize_agent_result(mut result: AgentResult) -> AgentResult {
        result.final_answer = crate::ecc::AgentShieldScanner::redact_secrets(&result.final_answer);
        result.history = result.history.into_iter().map(Self::sanitize_message).collect();
        result.steps = result.steps.into_iter().map(Self::sanitize_step).collect();
        result
    }

    /// Execute the autonomous agent on a single user prompt
    pub async fn run(&self, prompt: &str, ctx: &EngineContext) -> Result<AgentResult> {
        self.run_with_content(vec![ContentBlock::text(prompt)], ctx).await
    }

    /// Execute the autonomous agent with multiple content blocks (e.g. text + vision image)
    pub async fn run_with_content(
        &self,
        content: Vec<ContentBlock>,
        ctx: &EngineContext,
    ) -> Result<AgentResult> {
        let mut session = ChatSession::new();
        if let Some(ref sys) = self.system_prompt {
            session = session.with_system(sys.clone());
        }

        // Contextual memory recall: auto-retrieve top-3 matching items if memory is enabled
        let mut final_content = content;
        if let (Some(memory), Some(provider)) = (&self.memory, &self.embedding_provider) {
            let prompt_text: String = final_content
                .iter()
                .filter_map(|b| {
                    if let ContentBlock::Text { text } = b {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            if !prompt_text.trim().is_empty() {
                if let Ok(emb) = provider.embed_text(&prompt_text).await {
                    let hits = memory.search(&emb, 3, 0.05);
                    if !hits.is_empty() {
                        let mut memory_context = String::from("[LONG-TERM MEMORY & CODEBASE CONTEXT]\nThe following relevant context was retrieved from persistent memory:\n\n");
                        for (i, hit) in hits.iter().enumerate() {
                            let doc = &hit.document;
                            let file_path = doc.metadata.get("file_path").map(|s| s.as_str()).unwrap_or(&doc.id);
                            memory_context.push_str(&format!(
                                "Hit #{}: {} (relevance score: {:.2})\n{}\n\n",
                                i + 1, file_path, hit.score, doc.text.trim()
                            ));
                        }
                        memory_context.push_str("[END CONTEXT]\n\n");
                        final_content.insert(0, ContentBlock::text(memory_context));
                    }
                }
            }
        }

        // AgentShield cyber defense: intercept indirect prompt injection and synthetic tool calls before LLM ingestion
        if self.agentshield_enabled {
            for block in &final_content {
                match block {
                    ContentBlock::Text { ref text } => {
                        let pi_verdict = crate::ecc::AgentShieldScanner::scan_prompt_injection(text);
                        if let crate::ecc::AgentShieldVerdict::Block { ref reason, threat_level } = pi_verdict {
                            return Err(TagisanError::Execution(format!(
                                "[AgentShield Cyber Defense Block: {:?}] Prompt injection blocked: {}",
                                threat_level, reason
                            )));
                        }
                    }
                    ContentBlock::ToolCall { id: _, ref name, ref arguments } => {
                        let verdict = crate::ecc::AgentShieldScanner::scan_tool_call(name, arguments);
                        if let crate::ecc::AgentShieldVerdict::Block { ref reason, threat_level } = verdict {
                            return Err(TagisanError::Execution(format!(
                                "[AgentShield Cyber Defense Block: {:?}] Direct synthetic tool call blocked: {}",
                                threat_level, reason
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }

        session.add_user_message_with_blocks(final_content);

        self.execute_session(&mut session, ctx).await
    }

    /// Dynamically prunes tool definitions for local LLM models (e.g. Ollama)
    /// to reduce prompt evaluation latency by 15x-50x while preserving essential agentic tools
    pub fn prune_tools_for_query(
        tools: &[ToolDefinition],
        history: &[Message],
    ) -> Vec<ToolDefinition> {
        // Collect recent user text in history to detect domain keywords
        let mut combined_text = String::new();
        for msg in history.iter().rev().take(4) {
            combined_text.push_str(&msg.extract_text());
            combined_text.push(' ');
        }
        let query = combined_text.to_ascii_lowercase();

        // Core developer tools that are always retained for agentic tasks
        let core_tool_names = [
            "read_file", "view_file", "write_file", "write_to_file",
            "edit_file", "replace_file_content", "grep_search", "find_by_name",
            "list_dir", "delete_file", "run_command", "web_search",
            "calculator", "ask_question", "ask_user",
        ];

        // Domain keywords
        let wants_bun = query.contains("bun") || query.contains("typescript") || query.contains("ts") || query.contains("javascript") || query.contains("npm");
        let wants_python = query.contains("python") || query.contains("pip") || query.contains("pytest") || query.contains("numpy");
        let wants_perl = query.contains("perl") || query.contains("cpan");
        let wants_vella = query.contains("vella") || query.contains("scada") || query.contains("robot") || query.contains("trading") || query.contains("estop");
        let wants_copilot = query.contains("copilot") || query.contains("teams") || query.contains("sharepoint") || query.contains("excel") || query.contains("outlook") || query.contains("graph") || query.contains("purview") || query.contains("office");
        let wants_visual = query.contains("mermaid") || query.contains("diagram") || query.contains("chart") || query.contains("carousel") || query.contains("image") || query.contains("draw");
        let wants_artifact = query.contains("artifact") || query.contains("html");
        let wants_code_intel = query.contains("graph") || query.contains("blast") || query.contains("radius") || query.contains("ast");
        let wants_systems = query.contains("ebpf") || query.contains("xdp") || query.contains("spdk") || query.contains("verilog") || query.contains("fpga") || query.contains("simd") || query.contains("fuzz") || query.contains("chaos") || query.contains("z3") || query.contains("smt") || query.contains("kani") || query.contains("tla");
        let wants_review = query.contains("review") || query.contains("vibe");
        let wants_skills = query.contains("skill");
        let wants_memory = query.contains("memory") || query.contains("remember") || query.contains("recall");
        let wants_git = query.contains("worktree") || query.contains("branch") || query.contains("commit");

        tools
            .iter()
            .filter(|t| {
                let name = t.name.as_str();
                if core_tool_names.contains(&name) {
                    return true;
                }
                if name.starts_with("bun_") && wants_bun {
                    return true;
                }
                if name.starts_with("python_") && wants_python {
                    return true;
                }
                if name.starts_with("perl_") && wants_perl {
                    return true;
                }
                if name.starts_with("vella_") && wants_vella {
                    return true;
                }
                if name.starts_with("copilot_") && wants_copilot {
                    return true;
                }
                if (name.contains("mermaid") || name.contains("carousel") || name.contains("image") || name.contains("terminal_media")) && wants_visual {
                    return true;
                }
                if name.contains("artifact") && wants_artifact {
                    return true;
                }
                if (name == "query_code_graph" || name == "calculate_blast_radius") && wants_code_intel {
                    return true;
                }
                if (name.starts_with("ebpf_") || name.starts_with("xdp_") || name.starts_with("spdk_") || name.starts_with("fpga_") || name.starts_with("simd_") || name.starts_with("api_contract_") || name.starts_with("chaos_") || name.starts_with("z3_") || name.starts_with("kani_") || name.starts_with("tla_") || name.starts_with("binary_") || name.starts_with("compiler_") || name.starts_with("constant_time_") || name.starts_with("rr_") || name.starts_with("qemu_")) && wants_systems {
                    return true;
                }
                if name == "vibe_code_review" && wants_review {
                    return true;
                }
                if (name == "search_skills" || name == "fetch_skill") && wants_skills {
                    return true;
                }
                if (name == "search_memory" || name == "save_memory") && wants_memory {
                    return true;
                }
                if (name == "git_worktree" || name == "render_diff") && (wants_git || query.contains("diff")) {
                    return true;
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Execute the autonomous loop on an existing chat session
    pub async fn execute_session(
        &self,
        session: &mut ChatSession,
        ctx: &EngineContext,
    ) -> Result<AgentResult> {
        self.execute_session_streaming(session, ctx, |_| {}).await
    }

    /// Execute the autonomous loop on an existing chat session with real-time streaming deltas
    pub async fn execute_session_streaming<F>(
        &self,
        session: &mut ChatSession,
        ctx: &EngineContext,
        mut on_delta: F,
    ) -> Result<AgentResult>
    where
        F: FnMut(&StreamChunkDelta) + Send,
    {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut total_usage = TokenUsage::default();
        let mut steps = Vec::new();
        let mut iteration = 0;

        let all_tool_definitions = self.tools.definitions();
        let supports_tools = self.provider.capabilities(&self.model).contains(crate::types::ProviderCapabilities::FUNCTION_CALLING);
        let is_local = self.provider.provider_id() == "ollama"
            || self.model.to_ascii_lowercase().contains("ollama")
            || self.model.to_ascii_lowercase().contains("llama")
            || self.model.to_ascii_lowercase().contains("qwen")
            || self.model.to_ascii_lowercase().contains("mistral")
            || self.model.to_ascii_lowercase().contains("phi");

        let disable_pruning = std::env::var("TAGISAN_DISABLE_TOOL_PRUNING")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        // Automatic skill selection & semantic injection across all processes
        if self.auto_skills_enabled {
            let last_user_prompt = session
                .history
                .iter()
                .rev()
                .find(|m| m.role == crate::types::Role::User)
                .map(|m| m.extract_text())
                .unwrap_or_default();

            if !last_user_prompt.trim().is_empty() {
                let current_sys = session.system_prompt.as_deref().unwrap_or("");
                if !current_sys.contains("[AUTOMATICALLY SELECTED ENGINEERING SKILLS]") && !current_sys.contains("Skill Invariant:") {
                    let dispatcher = crate::ecc::global_dispatcher();
                    let matched = dispatcher.dispatch(&last_user_prompt, 2, None);
                    if !matched.is_empty() {
                        let mut skills_ctx = String::from("\n[AUTOMATICALLY SELECTED ENGINEERING SKILLS]\nThe following specialized engineering skills were automatically selected based on problem semantics:\n");
                        for item in &matched {
                            skills_ctx.push_str(&format!(
                                "\n--- ⚡ Skill: {} (Relevance: {:.2}) ---\n{}\n",
                                item.skill.name, item.score, item.skill.instructions.trim()
                            ));
                        }
                        skills_ctx.push_str("\n[END AUTODISPATCHED SKILLS]\n\n");
                        let new_sys = format!("{}{}", skills_ctx, current_sys);
                        session.system_prompt = Some(new_sys);
                    }
                }
            }
        }

        while iteration < self.max_iterations {
            iteration += 1;

            if ctx.cancellation_token.is_cancelled() {
                return Err(TagisanError::Cancelled);
            }

            debug!("Agent iteration {iteration}/{}", self.max_iterations);

            // If the model does not support tool calling, do not send any tools
            let active_tools = if !supports_tools {
                Vec::new()
            } else if is_local && !disable_pruning {
                Self::prune_tools_for_query(&all_tool_definitions, &session.history)
            } else {
                all_tool_definitions.clone()
            };

            // Construct completion request with active tools
            let mut req = CompletionRequest::new(self.model.clone(), "")
                .with_messages(session.history.clone())
                .with_tools(active_tools)
                .with_stream(true)
                .with_cancellation(ctx.cancellation_token.clone());

            if let Some(sys) = session.system_prompt.as_deref().or(self.system_prompt.as_deref()) {
                req = req.with_system(sys);
            }

            if let Some(temp) = self.temperature {
                req = req.with_temperature(temp);
            }

            // Attempt streaming from provider, fallback to complete() if unsupported
            let stream_res = self.provider.stream(req.clone()).await;
            let (assistant_msg, stream_usage) = match stream_res {
                Ok(mut stream) => {
                    let mut text_acc = String::new();
                    let mut thinking_acc = String::new();
                    let mut tool_calls_map: std::collections::BTreeMap<usize, (Option<String>, Option<String>, String)> = std::collections::BTreeMap::new();
                    let mut latest_usage: Option<TokenUsage> = None;

                    while let Some(chunk_res) = stream.next().await {
                        if ctx.cancellation_token.is_cancelled() {
                            return Err(TagisanError::Cancelled);
                        }
                        let chunk = chunk_res?;
                        if let Some(u) = chunk.usage {
                            latest_usage = Some(u);
                        }

                        on_delta(&chunk.delta);

                        match chunk.delta {
                            StreamChunkDelta::Text(t) => {
                                text_acc.push_str(&t);
                            }
                            StreamChunkDelta::Thinking(th) => {
                                thinking_acc.push_str(&th);
                            }
                            StreamChunkDelta::ToolCallDelta { index, id, name, arguments_delta } => {
                                let entry = tool_calls_map.entry(index).or_insert_with(|| (None, None, String::new()));
                                if let Some(new_id) = id {
                                    entry.0 = Some(new_id);
                                }
                                if let Some(new_name) = name {
                                    entry.1 = Some(new_name);
                                }
                                if let Some(args) = arguments_delta {
                                    entry.2.push_str(&args);
                                }
                            }
                        }
                    }

                    let mut blocks = Vec::new();
                    if !thinking_acc.is_empty() {
                        blocks.push(ContentBlock::Thinking {
                            thinking: thinking_acc,
                            signature: None,
                        });
                    }
                    if !text_acc.is_empty() {
                        blocks.push(ContentBlock::Text { text: text_acc });
                    }
                    for (idx, (id_opt, name_opt, args_str)) in tool_calls_map {
                        let id = id_opt.unwrap_or_else(|| format!("call_{}_{}", iteration, idx));
                        let name = name_opt.unwrap_or_default();
                        let args = serde_json::from_str(&args_str).unwrap_or(serde_json::Value::Object(Default::default()));
                        blocks.push(ContentBlock::ToolCall {
                            id,
                            name,
                            arguments: args,
                        });
                    }

                    let msg = Message {
                        role: crate::types::Role::Assistant,
                        content: blocks,
                        name: None,
                        metadata: std::collections::HashMap::new(),
                    };

                    (msg, latest_usage)
                }
                Err(stream_err) => {
                    let err_str = stream_err.to_string();
                    let req_fallback = if err_str.contains("does not support tools") {
                        req.with_tools(Vec::new()).with_stream(false)
                    } else {
                        req.with_stream(false)
                    };
                    let resp = self.provider.complete(req_fallback).await?;
                    let text = resp.message.extract_text();
                    if !text.is_empty() {
                        on_delta(&StreamChunkDelta::Text(text));
                    }
                    (resp.message, Some(resp.usage))
                }
            };

            let usage = stream_usage.unwrap_or_else(|| {
                let prompt_tok = (session.history.iter().map(|m| m.extract_text().len()).sum::<usize>() / 4) as u32;
                let comp_tok = (assistant_msg.extract_text().len() / 4) as u32;
                TokenUsage {
                    prompt_tokens: prompt_tok.max(1),
                    completion_tokens: comp_tok.max(1),
                    reasoning_tokens: None,
                    cached_prompt_tokens: None,
                    estimated_cost_usd: Some(0.0),
                }
            });

            // Record budget with prompt caching discount if cached tokens present
            if let Some(cached) = usage.cached_prompt_tokens {
                ctx.budget_tracker.record_with_cache(
                    &self.model,
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    cached,
                )?;
            } else {
                ctx.budget_tracker.record(
                    &self.model,
                    usage.prompt_tokens,
                    usage.completion_tokens,
                )?;
            }

            // Accumulate usage
            total_usage.prompt_tokens += usage.prompt_tokens;
            total_usage.completion_tokens += usage.completion_tokens;
            if let Some(rt) = usage.reasoning_tokens {
                total_usage.reasoning_tokens = Some(total_usage.reasoning_tokens.unwrap_or(0) + rt);
            }
            if let Some(cached) = usage.cached_prompt_tokens {
                total_usage.cached_prompt_tokens = Some(total_usage.cached_prompt_tokens.unwrap_or(0) + cached);
            }

            // Redact assistant message if AgentShield is enabled
            let assistant_msg = if self.agentshield_enabled {
                Self::sanitize_message(assistant_msg)
            } else {
                assistant_msg
            };
            session.add_message(assistant_msg.clone());

            let tool_calls = assistant_msg.extract_tool_calls();

            // If the assistant did not invoke any tools, we reached a final answer
            if tool_calls.is_empty() {
                let final_text = assistant_msg.extract_text();
                info!("Agent concluded in {iteration} iteration(s).");
                return Ok(AgentResult {
                    final_answer: final_text,
                    history: session.history.clone(),
                    iterations: iteration,
                    steps,
                    total_usage,
                    total_cost_usd: ctx.budget_tracker.current_spent_usd(),
                    total_latency: start_time.elapsed(),
                });
            }

            // Execute all requested tool calls
            let mut tool_results = Vec::new();
            for (id, name, args) in tool_calls {
                if ctx.cancellation_token.is_cancelled() {
                    return Err(TagisanError::Cancelled);
                }

                // AgentShield security interception
                if self.agentshield_enabled {
                    if let crate::ecc::AgentShieldVerdict::Block { reason, threat_level } =
                        crate::ecc::AgentShieldScanner::scan_tool_call(name, args)
                    {
                        warn!(
                            "AgentShield security violation blocked tool '{}' (threat level {:?}): {}",
                            name, threat_level, reason
                        );
                        tool_results.push(ContentBlock::tool_result(
                            id,
                            format!(
                                "[AgentShield Security Block: {:?}] Tool execution blocked: {}",
                                threat_level, reason
                            ),
                            true,
                        ));
                        continue;
                    }
                }

                debug!("Executing tool call '{}' with ID '{}'", name, id);
                let mut result_block = self.tools.execute_call(id, name, args).await;

                // AgentShield runtime secret redaction for tool outputs
                if self.agentshield_enabled {
                    result_block = Self::sanitize_content_block(result_block);
                }

                tool_results.push(result_block);
            }

            // Add all tool results for this iteration in a single turn
            if !tool_results.is_empty() {
                session.add_message(Message::tool_results(tool_results.clone()));
            }

            steps.push(AgentStep {
                iteration,
                assistant_message: assistant_msg,
                tool_results,
            });
        }

        warn!(
            "Autonomous agent reached maximum iterations ({}) without completing.",
            self.max_iterations
        );

        // Extract last known text or fallback
        let mut last_answer = session
            .history
            .iter()
            .rev()
            .find(|m| m.role == crate::types::Role::Assistant)
            .map(|m| m.extract_text())
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| format!("Maximum tool iterations ({}) reached without final answer.", self.max_iterations));

        if self.agentshield_enabled {
            last_answer = crate::ecc::AgentShieldScanner::redact_secrets(&last_answer);
        }

        Ok(AgentResult {
            final_answer: last_answer,
            history: session.history.clone(),
            iterations: iteration,
            steps,
            total_usage,
            total_cost_usd: ctx.budget_tracker.current_spent_usd(),
            total_latency: start_time.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;

    #[test]
    fn test_prune_tools_for_general_query() {
        let registry = ToolRegistry::with_builtins();
        let all_defs = registry.definitions();
        assert!(all_defs.len() > 50);

        let history = vec![Message::user("Hello, how are you today?")];
        let pruned = AutonomousAgent::prune_tools_for_query(&all_defs, &history);

        // General conversation should prune down to core tools (~15 tools)
        assert!(pruned.len() <= 20);
        assert!(pruned.iter().any(|t| t.name == "read_file"));
        assert!(pruned.iter().any(|t| t.name == "run_command"));
        assert!(!pruned.iter().any(|t| t.name.starts_with("copilot_")));
        assert!(!pruned.iter().any(|t| t.name.starts_with("vella_")));
    }

    #[test]
    fn test_prune_tools_for_domain_queries() {
        let registry = ToolRegistry::with_builtins();
        let all_defs = registry.definitions();

        // 1. Copilot domain query
        let copilot_history = vec![Message::user("Can you sync this report with Microsoft Copilot and Teams?")];
        let copilot_pruned = AutonomousAgent::prune_tools_for_query(&all_defs, &copilot_history);
        assert!(copilot_pruned.iter().any(|t| t.name.starts_with("copilot_")));

        // 2. Vella domain query
        let vella_history = vec![Message::user("Check the SCADA status and trigger emergency estop if needed.")];
        let vella_pruned = AutonomousAgent::prune_tools_for_query(&all_defs, &vella_history);
        assert!(vella_pruned.iter().any(|t| t.name.starts_with("vella_")));

        // 3. Bun domain query
        let bun_history = vec![Message::user("Run this TypeScript file using bun runtime.")];
        let bun_pruned = AutonomousAgent::prune_tools_for_query(&all_defs, &bun_history);
        assert!(bun_pruned.iter().any(|t| t.name.starts_with("bun_")));
    }
}
