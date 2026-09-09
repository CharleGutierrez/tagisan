use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::memory::embedding::EmbeddingProvider;
use crate::memory::store::VectorStore;
use crate::providers::LlmProvider;
use crate::tools::builtin::{SaveMemoryTool, SearchMemoryTool};
use crate::tools::ToolRegistry;
use crate::types::{ChatSession, CompletionRequest, ContentBlock, Message, TokenUsage};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

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
    pub memory: Option<Arc<VectorStore>>,
    pub embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
}

impl AutonomousAgent {
    /// Create a new AutonomousAgent instance (AgentShield security enabled by default)
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
            memory: None,
            embedding_provider: None,
        }
    }

    /// Enable or disable AgentShield security scanner for tool calls and secret redaction
    pub fn with_agentshield(mut self, enabled: bool) -> Self {
        self.agentshield_enabled = enabled;
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

        session.add_user_message_with_blocks(final_content);

        self.execute_session(&mut session, ctx).await
    }

    /// Execute the autonomous loop on an existing chat session
    pub async fn execute_session(
        &self,
        session: &mut ChatSession,
        ctx: &EngineContext,
    ) -> Result<AgentResult> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut total_usage = TokenUsage::default();
        let mut steps = Vec::new();
        let mut iteration = 0;
        let tool_definitions = self.tools.definitions();

        while iteration < self.max_iterations {
            iteration += 1;

            if ctx.cancellation_token.is_cancelled() {
                return Err(TagisanError::Cancelled);
            }

            debug!("Agent iteration {iteration}/{}", self.max_iterations);

            // Construct completion request with active tools
            let mut req = CompletionRequest::new(self.model.clone(), "")
                .with_messages(session.history.clone())
                .with_tools(tool_definitions.clone())
                .with_cancellation(ctx.cancellation_token.clone());

            if let Some(sys) = session.system_prompt.as_deref().or(self.system_prompt.as_deref()) {
                req = req.with_system(sys);
            }

            if let Some(temp) = self.temperature {
                req = req.with_temperature(temp);
            }

            // Call LLM provider
            let response = self.provider.complete(req).await?;

            // Record budget with prompt caching discount if cached tokens present
            if let Some(cached) = response.usage.cached_prompt_tokens {
                ctx.budget_tracker.record_with_cache(
                    &self.model,
                    response.usage.prompt_tokens,
                    response.usage.completion_tokens,
                    cached,
                )?;
            } else {
                ctx.budget_tracker.record(
                    &self.model,
                    response.usage.prompt_tokens,
                    response.usage.completion_tokens,
                )?;
            }

            // Accumulate usage
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            if let Some(rt) = response.usage.reasoning_tokens {
                total_usage.reasoning_tokens = Some(total_usage.reasoning_tokens.unwrap_or(0) + rt);
            }
            if let Some(cached) = response.usage.cached_prompt_tokens {
                total_usage.cached_prompt_tokens = Some(total_usage.cached_prompt_tokens.unwrap_or(0) + cached);
            }

            // Redact assistant message if AgentShield is enabled
            let assistant_msg = if self.agentshield_enabled {
                Self::sanitize_message(response.message.clone())
            } else {
                response.message.clone()
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
