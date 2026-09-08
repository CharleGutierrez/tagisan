use crate::agent::AutonomousAgent;
use crate::dag::graph::WorkflowGraph;
use crate::dag::node::{RetryPolicy, TaskNode};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::tools::builtin::{CalculatorTool, ReadFileTool, RunCommandTool, WriteFileTool};
use crate::tools::ToolRegistry;
use crate::types::CompletionRequest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// JSON schema representing an autonomous multi-task workflow plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedWorkflow {
    pub workflow_name: String,
    #[serde(default)]
    pub workflow_description: Option<String>,
    pub tasks: Vec<PlannedTask>,
}

/// JSON schema representing an individual planned task in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    pub id: String,
    pub name: String,
    pub prompt_template: String,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub max_retries: Option<usize>,
}

pub const PLANNER_SYSTEM_PROMPT: &str = r#"You are the Tagisan Autonomous Workflow Architect and Graph Decomposition Specialist.
Your job is to decompose high-level, complex objectives into an optimal Directed Acyclic Graph (DAG) of specialized subtasks.

Decomposition Guidelines:
1. Divide complex problems into modular, parallelizable tasks (e.g. independent research or analysis branches).
2. For downstream tasks that need previous outputs, specify their dependencies in `dependencies` array.
3. In downstream `prompt_template`, reference upstream outputs using `{upstream_task_id.output}` placeholders.
4. Select appropriate tools for each task (e.g. `read_file`, `write_file`, `run_command`, `calculator`).
5. Ensure the graph is strictly acyclic (NO circular dependencies).
6. Provide a clean, robust final synthesis task that aggregates upstream findings.

Output Format:
You MUST respond ONLY with a valid JSON object strictly matching this schema:
{
  "workflow_name": "<Short descriptive name>",
  "workflow_description": "<Overview of workflow architecture>",
  "tasks": [
    {
      "id": "<unique_snake_case_id>",
      "name": "<Human-readable task name>",
      "prompt_template": "<Task prompt template. Can include {upstream_id.output} placeholders>",
      "system_prompt": "<Specialized persona and instructions for this subagent>",
      "model": "<recommended model or null>",
      "provider": "<recommended provider or null>",
      "tools": ["read_file", "calculator"],
      "dependencies": ["<upstream_id_1>"],
      "max_retries": 2
    }
  ]
}

DO NOT output any explanations or text outside the JSON object.
"#;

/// Autonomous planner that decomposes high-level goals into executable DAG workflows
pub struct WorkflowPlanner {
    pub provider: Arc<dyn LlmProvider>,
    pub model: String,
    pub available_tools: ToolRegistry,
}

impl WorkflowPlanner {
    /// Create a new WorkflowPlanner
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        let mut tools = ToolRegistry::new();
        tools.register_tool(ReadFileTool::new());
        tools.register_tool(WriteFileTool::new());
        tools.register_tool(RunCommandTool::default());
        tools.register_tool(CalculatorTool::new());

        Self {
            provider,
            model: model.into(),
            available_tools: tools,
        }
    }

    /// Attach a custom tool registry
    pub fn with_tools(mut self, tools: ToolRegistry) -> Self {
        self.available_tools = tools;
        self
    }

    /// Build the prompt for LLM goal decomposition
    pub fn build_planner_prompt(&self, goal: &str) -> String {
        format!(
            "Decompose the following complex user objective into an optimal, parallelizable multi-agent DAG workflow:\n\nObjective: \"{}\"",
            goal
        )
    }

    /// Decompose a user goal into an executable `WorkflowGraph`
    pub async fn plan(&self, goal: &str, ctx: &EngineContext) -> Result<WorkflowGraph> {
        let prompt = self.build_planner_prompt(goal);
        let req = CompletionRequest::new(self.model.clone(), prompt)
            .with_system(PLANNER_SYSTEM_PROMPT)
            .with_temperature(0.2)
            .with_cancellation(ctx.cancellation_token.clone());

        let resp = self.provider.complete(req).await?;
        ctx.budget_tracker.record(
            &self.model,
            resp.usage.prompt_tokens,
            resp.usage.completion_tokens,
        )?;

        let raw_text = resp.message.extract_text();
        debug!("Planner raw response:\n{}", raw_text);

        self.parse_plan_json(&raw_text, Some(self.provider.clone()), ctx)
    }

    /// Parse a JSON plan string into a validated `WorkflowGraph`
    pub fn parse_plan_json(
        &self,
        json_str: &str,
        default_provider: Option<Arc<dyn LlmProvider>>,
        ctx: &EngineContext,
    ) -> Result<WorkflowGraph> {
        let cleaned_json = extract_json_block(json_str)?;
        let planned: PlannedWorkflow = serde_json::from_str(&cleaned_json).map_err(|e| {
            TagisanError::BadResponse(
                "workflow_planner".to_string(),
                format!("Failed to parse PlannedWorkflow JSON: {e}\nRaw output: {json_str}"),
            )
        })?;

        info!(
            "Parsed planned workflow '{}' with {} tasks",
            planned.workflow_name,
            planned.tasks.len()
        );

        let mut graph = WorkflowGraph::new();

        // 1. Add all tasks
        for task in &planned.tasks {
            let provider = if let Some(ref prov_id) = task.provider {
                ctx.get_provider(prov_id)
                    .unwrap_or_else(|_| default_provider.clone().unwrap_or_else(|| self.provider.clone()))
            } else {
                default_provider.clone().unwrap_or_else(|| self.provider.clone())
            };

            let model = task
                .model
                .clone()
                .unwrap_or_else(|| self.model.clone());

            // Build task-specific tool registry
            let mut task_tools = ToolRegistry::new();
            for tool_name in &task.tools {
                if let Some(tool) = self.available_tools.get(tool_name) {
                    task_tools.register(tool);
                }
            }

            let mut agent = AutonomousAgent::new(provider, model, task_tools);
            if let Some(ref sys) = task.system_prompt {
                agent = agent.with_system_prompt(sys.clone());
            }

            let retries = task.max_retries.unwrap_or(2);
            let retry_policy = RetryPolicy::linear(retries, Duration::from_millis(500));

            let mut task_node = TaskNode::new(task.id.clone(), task.name.clone(), task.prompt_template.clone())
                .with_agent(agent)
                .with_retry_policy(retry_policy);

            if let Some(ref sys) = task.system_prompt {
                task_node = task_node.with_system_prompt(sys.clone());
            }

            graph.add_task(task_node)?;
        }

        // 2. Add all dependency edges
        for task in &planned.tasks {
            for dep in &task.dependencies {
                graph.add_dependency(dep, &task.id)?;
            }
        }

        // 3. Validate graph integrity
        graph.validate()?;

        Ok(graph)
    }

    /// Construct a linear workflow DAG from a simple pipeline string (e.g. "task1 -> task2 -> task3")
    pub fn from_pipeline_str(
        &self,
        pipeline: &str,
        provider: Arc<dyn LlmProvider>,
        model: &str,
        tools: ToolRegistry,
    ) -> Result<WorkflowGraph> {
        let stages: Vec<&str> = pipeline
            .split("->")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if stages.is_empty() {
            return Err(TagisanError::Execution(
                "Pipeline string cannot be empty".to_string(),
            ));
        }

        let mut graph = WorkflowGraph::new();
        let mut prev_id: Option<String> = None;

        for (i, stage) in stages.iter().enumerate() {
            let id = format!("step_{}_{}", i + 1, sanitize_id(stage));
            let name = stage.to_string();

            let prompt_template = if let Some(ref prev) = prev_id {
                format!(
                    "Execute stage '{}' using output from previous stage:\n{{{}.output}}",
                    stage, prev
                )
            } else {
                format!("Execute initial stage: {}", stage)
            };

            let agent = AutonomousAgent::new(provider.clone(), model.to_string(), tools.clone());
            let node = TaskNode::new(id.clone(), name, prompt_template)
                .with_agent(agent)
                .with_retry_policy(RetryPolicy::linear(2, Duration::from_millis(500)));

            graph.add_task(node)?;

            if let Some(ref prev) = prev_id {
                graph.add_dependency(prev, &id)?;
            }

            prev_id = Some(id);
        }

        graph.validate()?;
        Ok(graph)
    }
}

/// Helper to sanitize stage name into a valid snake_case identifier
fn sanitize_id(s: &str) -> String {
    let sanitized: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    sanitized.trim_matches('_').to_string()
}

/// Extract clean JSON string by removing markdown code fences or surrounding text
pub fn extract_json_block(text: &str) -> Result<String> {
    let trimmed = text.trim();

    // 1. Try stripping outermost markdown fence with rfind to preserve inner nested code fences
    if let Some(start_idx) = trimmed.find("```json") {
        let content_after = &trimmed[start_idx + 7..];
        if let Some(end_idx) = content_after.rfind("```") {
            let candidate = content_after[..end_idx].trim().to_string();
            if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
                return Ok(candidate);
            }
        }
    } else if let Some(start_idx) = trimmed.find("```") {
        let content_after = &trimmed[start_idx + 3..];
        if let Some(end_idx) = content_after.rfind("```") {
            let candidate = content_after[..end_idx].trim().to_string();
            if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
                return Ok(candidate);
            }
        }
    }

    // 2. Fallback: look for outermost balanced '{' and '}'
    if let (Some(first_brace), Some(last_brace)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if first_brace <= last_brace {
            let candidate = trimmed[first_brace..=last_brace].trim().to_string();
            if serde_json::from_str::<serde_json::Value>(&candidate).is_ok() {
                return Ok(candidate);
            }
        }
    }

    // 3. Return trimmed text as last attempt
    Ok(trimmed.to_string())
}
