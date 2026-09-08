use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
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
}

impl AutonomousAgent {
    /// Create a new AutonomousAgent instance
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
        }
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

    /// Execute the autonomous agent on a single user prompt
    pub async fn run(&self, prompt: &str, ctx: &EngineContext) -> Result<AgentResult> {
        let mut session = ChatSession::new();
        if let Some(ref sys) = self.system_prompt {
            session = session.with_system(sys.clone());
        }
        session.add_user_message(prompt);

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

            // Record budget
            ctx.budget_tracker.record(
                &self.model,
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
            )?;

            // Accumulate usage
            total_usage.prompt_tokens += response.usage.prompt_tokens;
            total_usage.completion_tokens += response.usage.completion_tokens;
            if let Some(rt) = response.usage.reasoning_tokens {
                total_usage.reasoning_tokens = Some(total_usage.reasoning_tokens.unwrap_or(0) + rt);
            }

            let assistant_msg = response.message.clone();
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

                debug!("Executing tool call '{}' with ID '{}'", name, id);
                let result_block = self.tools.execute_call(id, name, args).await;
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
        let last_answer = session
            .history
            .iter()
            .rev()
            .find(|m| m.role == crate::types::Role::Assistant)
            .map(|m| m.extract_text())
            .unwrap_or_else(|| "Maximum tool iterations reached.".to_string());

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
