use crate::agent::{AgentResult, AutonomousAgent};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::tools::{ToolHandler, ToolRegistry};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Represents a specialized agent member within an autonomous swarm
#[derive(Clone)]
pub struct SwarmMember {
    pub name: String,
    pub role: String,
    pub model: String,
    pub provider: Arc<dyn LlmProvider>,
    pub system_prompt: Option<String>,
    pub tools: ToolRegistry,
    pub max_iterations: usize,
    pub temperature: Option<f32>,
}

impl SwarmMember {
    /// Create a new swarm member with persona, model, and provider
    pub fn new(
        name: impl Into<String>,
        role: impl Into<String>,
        model: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
    ) -> Self {
        Self {
            name: name.into(),
            role: role.into(),
            model: model.into(),
            provider,
            system_prompt: None,
            tools: ToolRegistry::new(),
            max_iterations: 10,
            temperature: Some(0.7),
        }
    }

    /// Set member-specific system instructions
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set member tool registry
    pub fn with_tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = tools;
        self
    }

    /// Register an individual tool for this member
    pub fn with_tool(mut self, tool: impl ToolHandler + 'static) -> Self {
        self.tools.register_tool(tool);
        self
    }

    /// Set maximum tool iterations for this member
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set model temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Build an AutonomousAgent instance representing this member
    pub fn build_agent(&self) -> AutonomousAgent {
        let mut agent = AutonomousAgent::new(self.provider.clone(), self.model.clone(), self.tools.clone())
            .with_max_iterations(self.max_iterations);

        if let Some(ref sys) = self.system_prompt {
            agent = agent.with_system_prompt(sys.clone());
        }

        if let Some(temp) = self.temperature {
            agent = agent.with_temperature(temp);
        }

        agent
    }

    /// Execute the member agent directly on a given prompt
    pub async fn run(&self, prompt: &str, ctx: &EngineContext) -> Result<AgentResult> {
        let agent = self.build_agent();
        agent.run(prompt, ctx).await
    }
}

/// Dynamic delegation tool allowing the Lead Swarm Agent to dispatch subtasks to specialists
#[derive(Clone)]
pub struct DelegateTaskTool {
    members: Arc<HashMap<String, SwarmMember>>,
    ctx: Arc<EngineContext>,
    caller_name: String,
}

impl DelegateTaskTool {
    /// Create a new delegation tool bound to swarm members and execution context
    pub fn new(
        members: Arc<HashMap<String, SwarmMember>>,
        ctx: Arc<EngineContext>,
        caller_name: impl Into<String>,
    ) -> Self {
        Self {
            members,
            ctx,
            caller_name: caller_name.into(),
        }
    }

    fn available_specialists_summary(&self) -> String {
        self.members
            .values()
            .filter(|m| !m.name.eq_ignore_ascii_case(&self.caller_name))
            .map(|m| format!("- `{}`: {}", m.name, m.role))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[async_trait]
impl ToolHandler for DelegateTaskTool {
    fn name(&self) -> &str {
        "delegate_task"
    }

    fn description(&self) -> &str {
        "Delegate a specific subtask to a specialized swarm member agent. Available specialists:\n"
    }

    fn parameters_schema(&self) -> Value {
        let specialist_names: Vec<String> = self
            .members
            .values()
            .filter(|m| !m.name.eq_ignore_ascii_case(&self.caller_name))
            .map(|m| m.name.clone())
            .collect();

        json!({
            "type": "object",
            "properties": {
                "target_agent": {
                    "type": "string",
                    "description": format!("The name of the specialist agent to delegate to. Options: [{}]", specialist_names.join(", "))
                },
                "task": {
                    "type": "string",
                    "description": "The detailed instructions or prompt for the specialist agent"
                }
            },
            "required": ["target_agent", "task"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let target_name = arguments
            .get("target_agent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required 'target_agent' parameter".to_string()))?;

        let task = arguments
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required 'task' parameter".to_string()))?;

        // Case-insensitive member lookup
        let target_member = self
            .members
            .values()
            .find(|m| m.name.eq_ignore_ascii_case(target_name))
            .ok_or_else(|| {
                let available: Vec<&str> = self.members.keys().map(|s| s.as_str()).collect();
                TagisanError::Execution(format!(
                    "Agent '{}' not found in swarm. Available agents: [{}]",
                    target_name,
                    available.join(", ")
                ))
            })?;

        info!(
            "Lead agent '{}' delegating subtask to specialist '{}' ({}): {}",
            self.caller_name, target_member.name, target_member.role, task
        );

        let result = target_member.run(task, &self.ctx).await?;
        debug!(
            "Specialist '{}' completed delegated task ({} iterations, cost: ${:.4})",
            target_member.name, result.iterations, result.total_cost_usd
        );

        Ok(format!(
            "[Response from specialist agent '{}' (Role: {})]:\n{}",
            target_member.name, target_member.role, result.final_answer
        ))
    }
}

/// Record of a single stage execution in a sequential pipeline
#[derive(Debug, Clone)]
pub struct PipelineStageOutput {
    pub stage_index: usize,
    pub agent_name: String,
    pub role: String,
    pub result: AgentResult,
}

/// Comprehensive outcome of a sequential pipeline execution
#[derive(Debug, Clone)]
pub struct PipelineExecutionResult {
    pub initial_input: String,
    pub stages: Vec<PipelineStageOutput>,
    pub final_answer: String,
}

/// Coordinator that orchestrates an autonomous multi-agent swarm
#[derive(Clone, Default)]
pub struct SwarmCoordinator {
    members: HashMap<String, SwarmMember>,
    lead_name: Option<String>,
}

impl SwarmCoordinator {
    /// Create a new empty swarm coordinator
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
            lead_name: None,
        }
    }

    /// Add a member to the swarm
    pub fn add_member(mut self, member: SwarmMember) -> Self {
        self.register_member(member);
        self
    }

    /// Register a member to the coordinator
    pub fn register_member(&mut self, member: SwarmMember) {
        if self.lead_name.is_none() {
            self.lead_name = Some(member.name.clone());
        }
        self.members.insert(member.name.clone(), member);
    }

    /// Set designated lead agent name
    pub fn set_lead(&mut self, name: impl Into<String>) {
        self.lead_name = Some(name.into());
    }

    /// Fluent builder method to set lead agent name
    pub fn with_lead(mut self, name: impl Into<String>) -> Self {
        self.set_lead(name);
        self
    }

    /// Retrieve a member by name
    pub fn get_member(&self, name: &str) -> Option<&SwarmMember> {
        self.members
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    /// Get all registered members
    pub fn members(&self) -> &HashMap<String, SwarmMember> {
        &self.members
    }

    /// Get current lead agent name
    pub fn lead_name(&self) -> Option<&str> {
        self.lead_name.as_deref()
    }

    /// Get current lead member
    pub fn lead(&self) -> Option<&SwarmMember> {
        self.lead_name.as_deref().and_then(|n| self.get_member(n))
    }

    /// Orchestrate execution via the Lead Agent with dynamic delegation tools
    pub async fn run_lead(&self, task: &str, ctx: &EngineContext) -> Result<AgentResult> {
        let lead_member = self.lead().ok_or_else(|| {
            TagisanError::Execution("No lead agent configured in SwarmCoordinator".to_string())
        })?;

        let members_arc = Arc::new(self.members.clone());
        let ctx_arc = Arc::new(ctx.clone());

        // Construct lead tool registry with DelegateTaskTool
        let mut lead_tools = lead_member.tools.clone();
        let delegate_tool = DelegateTaskTool::new(members_arc, ctx_arc, lead_member.name.clone());
        lead_tools.register_tool(delegate_tool.clone());

        // Augment lead agent system prompt to describe swarm capabilities
        let specialists_desc = delegate_tool.available_specialists_summary();

        let base_sys = lead_member
            .system_prompt
            .clone()
            .unwrap_or_else(|| format!("You are the Lead Coordinator '{}'.", lead_member.name));

        let augmented_system_prompt = format!(
            "{}\n\n## 👥 Swarm Orchestration Capabilities\n\
            You are the Lead Swarm Agent. You have access to a specialized tool `delegate_task(target_agent, task)` \
            which lets you dispatch subtasks to domain experts in your team:\n\
            {}\n\
            When handling complex objectives, break down the problem and delegate specialized responsibilities to these agents. \
            Review their outputs, synthesize their contributions, and produce a unified final solution.",
            base_sys.trim(),
            if specialists_desc.is_empty() { "None (working independently)\n".to_string() } else { specialists_desc }
        );

        let lead_agent = AutonomousAgent::new(
            lead_member.provider.clone(),
            lead_member.model.clone(),
            lead_tools,
        )
        .with_system_prompt(augmented_system_prompt)
        .with_max_iterations(lead_member.max_iterations);

        info!(
            "Starting Swarm execution with lead agent '{}' (model: {})",
            lead_member.name, lead_member.model
        );

        lead_agent.run(task, ctx).await
    }

    /// Execute a sequential pipeline where each specialist builds on the previous output
    pub async fn execute_pipeline(
        &self,
        stage_names: &[&str],
        initial_input: &str,
        ctx: &EngineContext,
    ) -> Result<PipelineExecutionResult> {
        let stages_to_run: Vec<&SwarmMember> = if stage_names.is_empty() {
            self.members.values().collect()
        } else {
            let mut resolved = Vec::new();
            for name in stage_names {
                let member = self.get_member(name).ok_or_else(|| {
                    TagisanError::Execution(format!("Pipeline stage agent '{name}' not found in swarm"))
                })?;
                resolved.push(member);
            }
            resolved
        };

        if stages_to_run.is_empty() {
            return Err(TagisanError::Execution(
                "Cannot execute pipeline with zero stages".to_string(),
            ));
        }

        let mut stage_outputs = Vec::new();
        let mut previous_output = String::new();

        for (idx, member) in stages_to_run.iter().enumerate() {
            if ctx.cancellation_token.is_cancelled() {
                return Err(TagisanError::Cancelled);
            }

            let stage_prompt = if idx == 0 {
                initial_input.to_string()
            } else {
                format!(
                    "## Initial Objective\n{}\n\n\
                    ## Output from Previous Stage (Agent: '{}', Role: {})\n{}\n\n\
                    ## Your Task\n\
                    As '{}' ({}), advance, refine, implement, or verify the above deliverable.",
                    initial_input,
                    stages_to_run[idx - 1].name,
                    stages_to_run[idx - 1].role,
                    previous_output.trim(),
                    member.name,
                    member.role
                )
            };

            info!(
                "Executing pipeline stage {}/{} with agent '{}' (role: {})",
                idx + 1,
                stages_to_run.len(),
                member.name,
                member.role
            );

            let res = member.run(&stage_prompt, ctx).await?;
            previous_output = res.final_answer.clone();

            stage_outputs.push(PipelineStageOutput {
                stage_index: idx,
                agent_name: member.name.clone(),
                role: member.role.clone(),
                result: res,
            });
        }

        let final_answer = previous_output;
        Ok(PipelineExecutionResult {
            initial_input: initial_input.to_string(),
            stages: stage_outputs,
            final_answer,
        })
    }

    /// Broadcast prompt to all registered swarm members concurrently via Tokio tasks
    pub async fn execute_broadcast(
        &self,
        prompt: &str,
        ctx: &EngineContext,
    ) -> Result<HashMap<String, AgentResult>> {
        if self.members.is_empty() {
            return Ok(HashMap::new());
        }

        info!(
            "Broadcasting prompt to {} swarm members concurrently",
            self.members.len()
        );

        let mut tasks = Vec::new();
        for member in self.members.values() {
            let member_clone = member.clone();
            let prompt_string = prompt.to_string();
            let ctx_clone = ctx.clone();

            let handle = tokio::spawn(async move {
                let res = member_clone.run(&prompt_string, &ctx_clone).await;
                (member_clone.name, res)
            });
            tasks.push(handle);
        }

        let mut results = HashMap::new();
        for task in tasks {
            let (name, agent_res) = task
                .await
                .map_err(|e| TagisanError::Execution(format!("Tokio broadcast task panicked: {e}")))?;
            let agent_result = agent_res?;
            results.insert(name, agent_result);
        }

        Ok(results)
    }
}
