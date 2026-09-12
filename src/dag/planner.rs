use crate::agent::AutonomousAgent;
use crate::dag::graph::WorkflowGraph;
use crate::dag::node::{RetryPolicy, TaskNode};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
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
4. Select appropriate tools for each task from the available tools list (e.g. `read_file`, `write_file`, `run_command`, `calculator`, or namespaced MCP tools).
5. Ensure the graph is strictly acyclic (NO circular dependencies).
6. Provide a clean, robust final synthesis task that aggregates upstream findings.
7. The `dependencies` array MUST ONLY contain exact `id` strings of tasks declared in this JSON. NEVER use generic placeholders like 'all_subtasks' or 'previous_tasks' - explicitly list the real task IDs.

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
      "model": null,
      "provider": null,
      "tools": ["read_file", "calculator"],
      "dependencies": ["<upstream_id_1>"],
      "max_retries": 2
    }
  ]
}

NOTE: Use JSON null (not the string "null") for model and provider unless a specific model or provider is strictly required.
DO NOT output any explanations or text outside the JSON object.
"#;

/// Autonomous planner that decomposes high-level goals into executable DAG workflows
pub struct WorkflowPlanner {
    pub provider: Arc<dyn LlmProvider>,
    pub model: String,
    pub available_tools: ToolRegistry,
}

impl WorkflowPlanner {
    /// Create a new WorkflowPlanner with default built-in tools
    pub fn new(provider: Arc<dyn LlmProvider>, model: impl Into<String>) -> Self {
        let tools = ToolRegistry::with_builtins();

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
        let tool_names = self.available_tools.names();
        let tools_clause = if tool_names.is_empty() {
            String::new()
        } else {
            format!("\n\nAvailable tools that can be assigned to tasks: [{}]", tool_names.join(", "))
        };
        format!(
            "Decompose the following complex user objective into an optimal, parallelizable multi-agent DAG workflow:\n\nObjective: \"{}\"{}",
            goal, tools_clause
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
            let provider = match task.provider.as_deref().map(|s| s.trim()) {
                Some(p) if !p.is_empty()
                    && !p.eq_ignore_ascii_case("null")
                    && !p.eq_ignore_ascii_case("none")
                    && !p.eq_ignore_ascii_case("default")
                    && !p.starts_with('<') => {
                    ctx.get_provider(p)
                        .unwrap_or_else(|_| default_provider.clone().unwrap_or_else(|| self.provider.clone()))
                }
                _ => default_provider.clone().unwrap_or_else(|| self.provider.clone()),
            };

            let model = match task.model.as_deref().map(|s| s.trim()) {
                Some(m) if !m.is_empty()
                    && !m.eq_ignore_ascii_case("null")
                    && !m.eq_ignore_ascii_case("none")
                    && !m.eq_ignore_ascii_case("default")
                    && !m.starts_with('<') => m.to_string(),
                _ => self.model.clone(),
            };

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

        // 2. Add all dependency edges with resilient resolution
        let all_task_ids: Vec<String> = planned.tasks.iter().map(|t| t.id.clone()).collect();

        for (i, task) in planned.tasks.iter().enumerate() {
            for dep in &task.dependencies {
                let d = dep.trim();
                if d.is_empty() {
                    continue;
                }
                let dep_lower = d.to_lowercase();
                if dep_lower == "all_subtasks"
                    || dep_lower == "all"
                    || dep_lower == "previous_tasks"
                    || dep_lower == "all_tasks"
                    || dep_lower == "*"
                {
                    // Automatically connect to all preceding tasks in the workflow
                    for prev in &planned.tasks[..i] {
                        if prev.id != task.id {
                            let _ = graph.add_dependency(&prev.id, &task.id);
                        }
                    }
                } else if all_task_ids.contains(&d.to_string()) {
                    if d != task.id {
                        let _ = graph.add_dependency(d, &task.id);
                    }
                } else {
                    // Try fuzzy matching or substring match against known task IDs
                    let matched = all_task_ids.iter().find(|id| {
                        let id_l = id.to_lowercase();
                        id_l.contains(&dep_lower) || dep_lower.contains(&id_l)
                    });
                    if let Some(real_id) = matched {
                        if real_id != &task.id {
                            let _ = graph.add_dependency(real_id, &task.id);
                        }
                    } else {
                        tracing::warn!(
                            "Ignoring unresolvable dependency '{}' on task '{}'",
                            d,
                            task.id
                        );
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OllamaProvider;

    #[test]
    fn test_all_subtasks_dependency_resolution() {
        let planner = WorkflowPlanner::new(Arc::new(OllamaProvider::default_local()), "test-model");
        let ctx = EngineContext::new(5.0);

        let json = r#"{
            "workflow_name": "Test Workflow",
            "tasks": [
                {
                    "id": "task_a",
                    "name": "Task A",
                    "prompt_template": "Do A",
                    "dependencies": []
                },
                {
                    "id": "task_b",
                    "name": "Task B",
                    "prompt_template": "Do B",
                    "dependencies": []
                },
                {
                    "id": "synthesis",
                    "name": "Final Synthesis",
                    "prompt_template": "Combine",
                    "dependencies": ["all_subtasks"]
                }
            ]
        }"#;

        let graph = planner.parse_plan_json(json, None, &ctx).expect("Should parse and resolve all_subtasks");
        let order = graph.validate().expect("DAG should be valid and acyclic");
        assert_eq!(order.last().unwrap(), "synthesis");
    }

    #[test]
    fn test_parse_plan_with_null_and_placeholder_model_and_provider() {
        let planner = WorkflowPlanner::new(Arc::new(OllamaProvider::default_local()), "test-fallback-model");
        let ctx = EngineContext::new(5.0);

        let json = r#"{
            "workflow_name": "Test Workflow",
            "tasks": [
                {
                    "id": "task_a",
                    "name": "Task A",
                    "prompt_template": "Do A",
                    "model": "null",
                    "provider": "null",
                    "dependencies": []
                },
                {
                    "id": "task_b",
                    "name": "Task B",
                    "prompt_template": "Do B",
                    "model": "<recommended model or null>",
                    "provider": "<recommended provider or null>",
                    "dependencies": []
                }
            ]
        }"#;

        let graph = planner.parse_plan_json(json, None, &ctx).expect("Should parse");
        let task_a = graph.get_task("task_a").unwrap();
        assert_eq!(task_a.agent.as_ref().unwrap().model, "test-fallback-model");
        let task_b = graph.get_task("task_b").unwrap();
        assert_eq!(task_b.agent.as_ref().unwrap().model, "test-fallback-model");
    }
}
