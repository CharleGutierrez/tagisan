use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    AutonomousAgent, BoxEventStream, CompletionRequest, CompletionResponse, DagScheduler,
    EngineContext, FinishReason, LlmProvider, Message, ProviderCapabilities, RetryPolicy,
    TagisanError, TaskNode, TaskStatus, TokenUsage, ToolRegistry, WorkflowEvent,
    WorkflowGraph, WorkflowPlanner,
};
use tokio::sync::Mutex;

// =========================================================================
// Test Helpers & Mock Providers
// =========================================================================

/// Mock provider that tracks concurrent in-flight requests and simulates latency
struct ConcurrencyTrackingProvider {
    id: &'static str,
    active_count: Arc<AtomicUsize>,
    max_active_observed: Arc<AtomicUsize>,
    delay: Duration,
}

impl ConcurrencyTrackingProvider {
    fn new(id: &'static str, delay: Duration) -> (Self, Arc<AtomicUsize>) {
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        (
            Self {
                id,
                active_count: active,
                max_active_observed: max_active.clone(),
                delay,
            },
            max_active,
        )
    }
}

#[async_trait]
impl LlmProvider for ConcurrencyTrackingProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let current = self.active_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_active_observed.fetch_max(current, Ordering::SeqCst);

        tokio::time::sleep(self.delay).await;

        self.active_count.fetch_sub(1, Ordering::SeqCst);

        let user_prompt = req.messages.last().map(|m| m.extract_text()).unwrap_or_default();

        Ok(CompletionResponse {
            id: format!("resp_{}", req.model),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(format!("Result for prompt: [{user_prompt}]")),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 15,
                completion_tokens: 25,
                ..Default::default()
            },
            latency: self.delay,
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse(
            self.id.to_string(),
            "Stream not used in mock".to_string(),
        ))
    }
}

/// Mock provider that fails a specified number of times before succeeding
struct FlakyFailThenSucceedProvider {
    id: &'static str,
    fail_count: Arc<AtomicUsize>,
    total_calls: Arc<AtomicUsize>,
    response_text: String,
}

impl FlakyFailThenSucceedProvider {
    fn new(id: &'static str, fail_count: usize, response_text: impl Into<String>) -> (Self, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            Self {
                id,
                fail_count: Arc::new(AtomicUsize::new(fail_count)),
                total_calls: calls.clone(),
                response_text: response_text.into(),
            },
            calls,
        )
    }
}

#[async_trait]
impl LlmProvider for FlakyFailThenSucceedProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        self.total_calls.fetch_add(1, Ordering::SeqCst);

        let remaining_fails = self.fail_count.load(Ordering::SeqCst);
        if remaining_fails > 0 {
            self.fail_count.fetch_sub(1, Ordering::SeqCst);
            return Err(TagisanError::BadResponse(
                self.id.to_string(),
                "Transient 503 Service Unavailable".to_string(),
            ));
        }

        Ok(CompletionResponse {
            id: "flaky_ok".to_string(),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(self.response_text.clone()),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 30,
                ..Default::default()
            },
            latency: Duration::from_millis(5),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse(
            self.id.to_string(),
            "Stream not used".to_string(),
        ))
    }
}

/// Scripted mock provider that returns a list of prepared responses
struct ScriptedMockProvider {
    id: &'static str,
    responses: Mutex<Vec<Result<CompletionResponse, TagisanError>>>,
}

impl ScriptedMockProvider {
    fn new(id: &'static str, responses: Vec<Result<CompletionResponse, TagisanError>>) -> Self {
        Self {
            id,
            responses: Mutex::new(responses),
        }
    }
}

#[async_trait]
impl LlmProvider for ScriptedMockProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let mut script = self.responses.lock().await;
        if script.is_empty() {
            Ok(CompletionResponse {
                id: "default_resp".to_string(),
                provider: self.id.to_string(),
                model: req.model,
                message: Message::assistant("Default scripted output"),
                finish_reason: FinishReason::Stop,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 10,
                    ..Default::default()
                },
                latency: Duration::from_millis(5),
            })
        } else {
            script.remove(0)
        }
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse(
            self.id.to_string(),
            "Stream not used".to_string(),
        ))
    }
}

// =========================================================================
// 1. Diamond DAG Parallel Execution Test
// =========================================================================

#[tokio::test]
async fn test_diamond_dag_parallel_execution() {
    // Structure:
    //       A (Root)
    //      / \
    //     B   C (Parallel Branches)
    //      \ /
    //       D (Synthesis)

    let (provider, max_concurrency) =
        ConcurrencyTrackingProvider::new("mock_concurrency", Duration::from_millis(80));
    let provider_arc: Arc<dyn LlmProvider> = Arc::new(provider);

    let mut graph = WorkflowGraph::new();

    let node_a = TaskNode::new("task_a", "Root Task", "Generate Initial Research")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "model_a",
            ToolRegistry::new(),
        ));

    let node_b = TaskNode::new("task_b", "Branch B", "Analyze Tech Angle: {task_a.output}")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "model_b",
            ToolRegistry::new(),
        ));

    let node_c = TaskNode::new("task_c", "Branch C", "Analyze Business Angle: {task_a.output}")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "model_c",
            ToolRegistry::new(),
        ));

    let node_d = TaskNode::new(
        "task_d",
        "Synthesis Task",
        "Synthesize: B=[{task_b.output}] and C=[{task_c.output}]",
    )
    .with_agent(AutonomousAgent::new(
        provider_arc.clone(),
        "model_d",
        ToolRegistry::new(),
    ));

    graph.add_task(node_a).unwrap();
    graph.add_task(node_b).unwrap();
    graph.add_task(node_c).unwrap();
    graph.add_task(node_d).unwrap();

    graph.add_dependency("task_a", "task_b").unwrap();
    graph.add_dependency("task_a", "task_c").unwrap();
    graph.add_dependency("task_b", "task_d").unwrap();
    graph.add_dependency("task_c", "task_d").unwrap();

    let topo = graph.validate().unwrap();
    assert_eq!(topo.len(), 4);
    assert_eq!(topo[0], "task_a");
    assert_eq!(topo[3], "task_d");

    let ctx = EngineContext::new(10.0);
    let scheduler = DagScheduler::new();

    let result = scheduler.run(&mut graph, &ctx).await.unwrap();

    assert_eq!(result.completed_tasks, 4);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.task_outputs.contains_key("task_a"));
    assert!(result.task_outputs.contains_key("task_b"));
    assert!(result.task_outputs.contains_key("task_c"));
    assert!(result.task_outputs.contains_key("task_d"));

    // Verify task D's output contains interpolated results from B and C
    let out_d = &result.task_outputs["task_d"].text;
    assert!(out_d.contains("Analyze Tech Angle"));
    assert!(out_d.contains("Analyze Business Angle"));

    // Verify parallel execution: Branches B and C ran concurrently, so peak concurrent requests >= 2
    let peak = max_concurrency.load(Ordering::SeqCst);
    assert!(
        peak >= 2,
        "Expected peak concurrent requests >= 2 during Diamond DAG execution, but got {}",
        peak
    );

    // Verify graph node statuses were updated
    assert_eq!(graph.get_task("task_a").unwrap().status, TaskStatus::Completed);
    assert_eq!(graph.get_task("task_b").unwrap().status, TaskStatus::Completed);
    assert_eq!(graph.get_task("task_c").unwrap().status, TaskStatus::Completed);
    assert_eq!(graph.get_task("task_d").unwrap().status, TaskStatus::Completed);
}

// =========================================================================
// 2. Cycle Detection Failure Tests
// =========================================================================

#[test]
fn test_cycle_detection_triangular_cycle() {
    // A -> B -> C -> A (Classic 3-node cycle)
    let mut graph = WorkflowGraph::new();

    let node_a = TaskNode::new("task_a", "Task A", "Prompt A");
    let node_b = TaskNode::new("task_b", "Task B", "Prompt B");
    let node_c = TaskNode::new("task_c", "Task C", "Prompt C");

    graph.add_task(node_a).unwrap();
    graph.add_task(node_b).unwrap();
    graph.add_task(node_c).unwrap();

    graph.add_dependency("task_a", "task_b").unwrap();
    graph.add_dependency("task_b", "task_c").unwrap();
    graph.add_dependency("task_c", "task_a").unwrap();

    let validation_result = graph.validate();
    assert!(
        validation_result.is_err(),
        "Validation should fail on cyclic DAG"
    );

    if let Err(TagisanError::Execution(msg)) = validation_result {
        assert!(msg.to_lowercase().contains("cycle"));
    } else {
        panic!("Expected TagisanError::Execution with cycle message");
    }
}

#[test]
fn test_cycle_detection_self_dependency() {
    let mut graph = WorkflowGraph::new();
    let node_a = TaskNode::new("task_a", "Task A", "Prompt A");
    graph.add_task(node_a).unwrap();

    // Adding A -> A should immediately fail
    let res = graph.add_dependency("task_a", "task_a");
    assert!(res.is_err());
}

#[test]
fn test_dangling_dependency_detection() {
    let mut graph = WorkflowGraph::new();
    let node_a = TaskNode::new("task_a", "Task A", "Prompt A");
    graph.add_task(node_a).unwrap();

    let res = graph.add_dependency("task_a", "non_existent_task");
    assert!(res.is_err());
}

// =========================================================================
// 3. Dynamic Upstream Prompt Interpolation Test
// =========================================================================

#[tokio::test]
async fn test_dynamic_prompt_interpolation() {
    let provider: Arc<dyn LlmProvider> = Arc::new(ConcurrencyTrackingProvider::new(
        "mock_prov",
        Duration::from_millis(5),
    ).0);

    let mut graph = WorkflowGraph::new();

    let node1 = TaskNode::new("extract_data", "Data Extractor", "Extract dataset metrics")
        .with_agent(AutonomousAgent::new(
            provider.clone(),
            "model_1",
            ToolRegistry::new(),
        ));

    let node2 = TaskNode::new(
        "format_report",
        "Report Formatter",
        "Generate report using extracted data:\n{extract_data.output}\nEnd of prompt.",
    )
    .with_agent(AutonomousAgent::new(
        provider.clone(),
        "model_2",
        ToolRegistry::new(),
    ));

    graph.add_task(node1).unwrap();
    graph.add_task(node2).unwrap();
    graph.add_dependency("extract_data", "format_report").unwrap();

    let ctx = EngineContext::new(5.0);
    let scheduler = DagScheduler::new();

    let result = scheduler.run(&mut graph, &ctx).await.unwrap();

    let report_output = &result.task_outputs["format_report"].text;
    assert!(report_output.contains("Extract dataset metrics"));
}

// =========================================================================
// 4. Task Retry on Transient Failure Test
// =========================================================================

#[tokio::test]
async fn test_task_retry_transient_failure() {
    // Provider will fail on attempt 1 and 2, and succeed on attempt 3
    let (flaky_provider, call_counter) = FlakyFailThenSucceedProvider::new(
        "flaky_provider",
        2,
        "Success after retrying!",
    );
    let provider_arc: Arc<dyn LlmProvider> = Arc::new(flaky_provider);

    let mut graph = WorkflowGraph::new();

    let retry_policy = RetryPolicy::new(3, Duration::from_millis(10), 1.5);

    let node = TaskNode::new("flaky_task", "Flaky Task", "Do difficult calculation")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "flaky_model",
            ToolRegistry::new(),
        ))
        .with_retry_policy(retry_policy);

    graph.add_task(node).unwrap();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let scheduler = DagScheduler::new().with_event_sender(event_tx);
    let ctx = EngineContext::new(5.0);

    let result = scheduler.run(&mut graph, &ctx).await.unwrap();

    assert_eq!(result.completed_tasks, 1);
    assert_eq!(result.failed_tasks, 0);
    assert_eq!(result.task_outputs["flaky_task"].text, "Success after retrying!");

    // Verify it was called exactly 3 times (2 failures + 1 success)
    assert_eq!(call_counter.load(Ordering::SeqCst), 3);

    // Verify retry events were recorded
    let mut retry_event_count = 0;
    while let Ok(evt) = event_rx.try_recv() {
        if let WorkflowEvent::TaskRetry { attempt, .. } = evt {
            retry_event_count += 1;
            assert!(attempt <= 2);
        }
    }
    assert_eq!(retry_event_count, 2);
}

// =========================================================================
// 5. Cancellation Token Aborting In-Flight Tasks
// =========================================================================

#[tokio::test]
async fn test_cancellation_token_aborts_dag() {
    let (provider, _) =
        ConcurrencyTrackingProvider::new("slow_provider", Duration::from_millis(500));
    let provider_arc: Arc<dyn LlmProvider> = Arc::new(provider);

    let mut graph = WorkflowGraph::new();

    let node_a = TaskNode::new("slow_a", "Slow Task A", "Heavy computational task A")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "slow_model",
            ToolRegistry::new(),
        ));

    let node_b = TaskNode::new("slow_b", "Slow Task B", "Heavy computational task B")
        .with_agent(AutonomousAgent::new(
            provider_arc.clone(),
            "slow_model",
            ToolRegistry::new(),
        ));

    graph.add_task(node_a).unwrap();
    graph.add_task(node_b).unwrap();
    graph.add_dependency("slow_a", "slow_b").unwrap();

    let ctx = EngineContext::new(5.0);
    let cancel_token = ctx.cancellation_token.clone();

    // Trigger cancellation after 50ms while task A is in-flight
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        cancel_token.cancel();
    });

    let scheduler = DagScheduler::new();
    let result = scheduler.run(&mut graph, &ctx).await;

    assert!(result.is_err(), "Workflow should fail due to cancellation");
    match result.unwrap_err() {
        TagisanError::Cancelled => {} // Expected
        other => panic!("Expected TagisanError::Cancelled, got {:?}", other),
    }
}

// =========================================================================
// 6. Autonomous Planner JSON Decomposition Test
// =========================================================================

#[tokio::test]
async fn test_workflow_planner_json_decomposition() {
    let mock_json_plan = r#"```json
{
  "workflow_name": "Autonomous Rust Compiler Optimization",
  "workflow_description": "Decomposes code analysis, profiling, and LLVM codegen tuning into a DAG",
  "tasks": [
    {
      "id": "parse_ast",
      "name": "Parse Syntax Tree",
      "prompt_template": "Parse the input code into an abstract syntax tree: {goal}",
      "system_prompt": "You are an expert compiler parser.",
      "model": "claude-3-5-sonnet",
      "tools": ["read_file"],
      "dependencies": [],
      "max_retries": 1
    },
    {
      "id": "profile_hotspots",
      "name": "Profile CPU Hotspots",
      "prompt_template": "Profile execution hotspots: {parse_ast.output}",
      "system_prompt": "You are a Linux perf profiling engineer.",
      "model": "claude-3-5-sonnet",
      "tools": ["run_command"],
      "dependencies": ["parse_ast"],
      "max_retries": 2
    },
    {
      "id": "optimize_codegen",
      "name": "LLVM IR Optimization",
      "prompt_template": "Generate optimized LLVM IR based on hotspots:\n{profile_hotspots.output}",
      "system_prompt": "You are an LLVM backend optimization expert.",
      "model": "claude-3-5-sonnet",
      "tools": ["calculator"],
      "dependencies": ["profile_hotspots"],
      "max_retries": 1
    }
  ]
}
```"#;

    let scripted_provider = ScriptedMockProvider::new(
        "mock_planner_prov",
        vec![Ok(CompletionResponse {
            id: "plan_resp".to_string(),
            provider: "mock_planner_prov".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            message: Message::assistant(mock_json_plan),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 200,
                ..Default::default()
            },
            latency: Duration::from_millis(20),
        })],
    );

    let provider_arc: Arc<dyn LlmProvider> = Arc::new(scripted_provider);
    let planner = WorkflowPlanner::new(provider_arc.clone(), "claude-3-5-sonnet");

    let ctx = EngineContext::new(10.0);
    let mut graph = planner
        .plan("Optimize Rust matrix multiplication kernel", &ctx)
        .await
        .unwrap();

    assert_eq!(graph.task_count(), 3);
    assert_eq!(graph.edge_count(), 2);

    let topo = graph.validate().unwrap();
    assert_eq!(topo, vec!["parse_ast", "profile_hotspots", "optimize_codegen"]);

    let task1 = graph.get_task("parse_ast").unwrap();
    assert_eq!(task1.name, "Parse Syntax Tree");
    assert!(task1.agent.is_some());

    let task2 = graph.get_task("profile_hotspots").unwrap();
    assert_eq!(task2.retry_policy.max_retries, 2);

    // Now execute the planned graph
    let scheduler = DagScheduler::new();
    let result = scheduler.run(&mut graph, &ctx).await.unwrap();

    assert_eq!(result.completed_tasks, 3);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.task_outputs.contains_key("parse_ast"));
    assert!(result.task_outputs.contains_key("profile_hotspots"));
    assert!(result.task_outputs.contains_key("optimize_codegen"));
}

// =========================================================================
// 7. Pipeline Expression Parsing Test
// =========================================================================

#[test]
fn test_planner_from_pipeline_str() {
    let (provider, _) =
        ConcurrencyTrackingProvider::new("mock_prov", Duration::from_millis(5));
    let provider_arc: Arc<dyn LlmProvider> = Arc::new(provider);
    let planner = WorkflowPlanner::new(provider_arc.clone(), "test_model");

    let graph = planner
        .from_pipeline_str(
            "Fetch Data -> Transform Features -> Train Model -> Evaluate Metrics",
            provider_arc,
            "test_model",
            ToolRegistry::new(),
        )
        .unwrap();

    assert_eq!(graph.task_count(), 4);
    assert_eq!(graph.edge_count(), 3);

    let topo = graph.validate().unwrap();
    assert_eq!(topo.len(), 4);
}
