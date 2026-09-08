use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    AutonomousAgent, BoxEventStream, CompletionRequest, CompletionResponse, DagScheduler,
    EngineContext, FinishReason, LlmProvider, Message,
    ProviderCapabilities, RetryPolicy, TagisanError, TaskNode, TaskStatus, TokenUsage,
    ToolRegistry, WorkflowEvent, WorkflowGraph,
};

// =========================================================================
// Mock Providers for Remediation Verification
// =========================================================================

struct CountingFlakyProvider {
    id: &'static str,
    calls: Arc<AtomicUsize>,
    fail_until_attempt: usize,
    tokens_per_call: (u32, u32),
}

impl CountingFlakyProvider {
    fn new(fail_until_attempt: usize, prompt_toks: u32, comp_toks: u32) -> (Self, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            Self {
                id: "counting_flaky",
                calls: calls.clone(),
                fail_until_attempt,
                tokens_per_call: (prompt_toks, comp_toks),
            },
            calls,
        )
    }
}

#[async_trait]
impl LlmProvider for CountingFlakyProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let call_idx = self.calls.fetch_add(1, Ordering::SeqCst) + 1;

        if call_idx <= self.fail_until_attempt {
            return Err(TagisanError::BadResponse(
                self.id.to_string(),
                format!("Simulated transient 503 on call {}", call_idx),
            ));
        }

        Ok(CompletionResponse {
            id: format!("resp_{}", call_idx),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(format!("Output from attempt {}", call_idx)),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: self.tokens_per_call.0,
                completion_tokens: self.tokens_per_call.1,
                ..Default::default()
            },
            latency: Duration::from_millis(1),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse(self.id.to_string(), "No stream".into()))
    }
}

struct BudgetExceededTriggerProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl LlmProvider for BudgetExceededTriggerProvider {
    fn provider_id(&self) -> &'static str {
        "budget_trigger"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(TagisanError::BudgetExceeded {
            max_budget: 1.0,
            current_spent: 1.05,
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse("budget_trigger".into(), "No stream".into()))
    }
}

struct EchoMockProvider {
    id: &'static str,
}

impl EchoMockProvider {
    fn new(id: &'static str) -> Self {
        Self { id }
    }
}

#[async_trait]
impl LlmProvider for EchoMockProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let text = req.messages.last().map(|m| m.extract_text()).unwrap_or_default();
        Ok(CompletionResponse {
            id: "echo_resp".to_string(),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(format!("Executed: {}", text)),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 15,
                completion_tokens: 20,
                ..Default::default()
            },
            latency: Duration::from_millis(2),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::BadResponse(self.id.to_string(), "No stream".into()))
    }
}

// =========================================================================
// REMEDIATION 1: extract_json_block handles nested markdown code fences
// =========================================================================
#[test]
fn test_remediation_extract_json_block_handles_nested_code_fences() {
    let raw_llm_output = r#"Here is the generated workflow plan:
```json
{
  "workflow_name": "Code Generator Workflow",
  "workflow_description": "Generates and reviews code",
  "tasks": [
    {
      "id": "write_fn",
      "name": "Write Function",
      "prompt_template": "Implement the following function:\n```rust\nfn calculate(x: i32) -> i32 {\n    x * 2\n}\n```\nMake sure tests pass.",
      "tools": [],
      "dependencies": []
    }
  ]
}
```
"#;

    // extract_json_block must extract the complete valid JSON without being truncated by inner code fences
    let extracted = tagisan::extract_json_block(raw_llm_output).unwrap();
    let parse_result: std::result::Result<tagisan::PlannedWorkflow, _> = serde_json::from_str(&extracted);

    assert!(
        parse_result.is_ok(),
        "Remediation verified: extract_json_block handles nested code fences and parses valid PlannedWorkflow"
    );
    let plan = parse_result.unwrap();
    assert_eq!(plan.tasks.len(), 1);
    assert!(plan.tasks[0].prompt_template.contains("fn calculate"));
}

// =========================================================================
// REMEDIATION 2: DAG Scheduler prevents unrelated context leakage
// =========================================================================
#[tokio::test]
async fn test_remediation_dag_scheduler_filters_upstream_context_leakage() {
    // Task A: Sensitive HR data extraction (independent branch)
    // Task B: Market research (independent branch)
    // Task C: Market summary (depends ONLY on Task B)
    let prov = Arc::new(EchoMockProvider::new("echo_p"));

    let mut graph = WorkflowGraph::new();

    let node_a = TaskNode::new("task_a_hr", "HR Task", "CONFIDENTIAL: CEO salary is $500,000")
        .with_agent(AutonomousAgent::new(prov.clone(), "m1", ToolRegistry::new()));

    let node_b = TaskNode::new("task_b_market", "Market Task", "Market growth is 12% YoY")
        .with_agent(AutonomousAgent::new(prov.clone(), "m2", ToolRegistry::new()));

    // Task C has a generic prompt without explicit placeholders and depends ONLY on Task B
    let node_c = TaskNode::new("task_c_summary", "Summary Task", "Summarize findings.")
        .with_agent(AutonomousAgent::new(prov.clone(), "m3", ToolRegistry::new()));

    graph.add_task(node_a).unwrap();
    graph.add_task(node_b).unwrap();
    graph.add_task(node_c).unwrap();

    // Dependency: ONLY B -> C
    graph.add_dependency("task_b_market", "task_c_summary").unwrap();

    let ctx = EngineContext::new(10.0);
    let scheduler = DagScheduler::new();

    let res = scheduler.run(&mut graph, &ctx).await.unwrap();

    let task_c_output = &res.task_outputs["task_c_summary"].text;

    // Verify Task C received ONLY Task B's context and NEVER received Task A's confidential data
    assert!(
        task_c_output.contains("Market growth is 12% YoY"),
        "Task C should receive its direct upstream dependency output"
    );
    assert!(
        !task_c_output.contains("CEO salary"),
        "Remediation verified: Unrelated parallel branch data was NOT leaked to Task C"
    );
}

// =========================================================================
// REMEDIATION 3: Graph Status and Outputs Persisted on Workflow Failure
// =========================================================================
#[tokio::test]
async fn test_remediation_graph_status_and_outputs_persisted_on_failure() {
    let (prov1, _) = CountingFlakyProvider::new(0, 10, 10); // always succeeds
    let (prov2, _) = CountingFlakyProvider::new(10, 10, 10); // always fails

    let mut graph = WorkflowGraph::new();

    let node1 = TaskNode::new("task_success", "Success Task", "Do step 1")
        .with_agent(AutonomousAgent::new(Arc::new(prov1), "m1", ToolRegistry::new()));

    let node2 = TaskNode::new("task_failure", "Failing Task", "Do step 2")
        .with_agent(AutonomousAgent::new(Arc::new(prov2), "m2", ToolRegistry::new()));

    graph.add_task(node1).unwrap();
    graph.add_task(node2).unwrap();
    graph.add_dependency("task_success", "task_failure").unwrap();

    let ctx = EngineContext::new(5.0);
    let scheduler = DagScheduler::new();

    let res = scheduler.run(&mut graph, &ctx).await;
    assert!(res.is_err(), "Workflow should fail on task 2");

    // Succeeded task must be marked Completed and have output recorded
    let task1_node = graph.get_task("task_success").unwrap();
    assert_eq!(
        task1_node.status,
        TaskStatus::Completed,
        "Remediation verified: Completed task status is preserved in graph"
    );
    assert!(
        task1_node.output.is_some(),
        "Remediation verified: Completed task output is stored in graph"
    );

    // Failed task must be marked Failed
    let task2_node = graph.get_task("task_failure").unwrap();
    assert_eq!(
        task2_node.status,
        TaskStatus::Failed,
        "Remediation verified: Failed task status is recorded in graph"
    );
}

// =========================================================================
// REMEDIATION 4: Token Usage Correctly Accumulates Across Retries
// =========================================================================
#[tokio::test]
async fn test_remediation_token_usage_accumulates_across_retries() {
    // Fails on attempt 1 (25 prompt + 25 comp tokens) and succeeds on attempt 2 (25 prompt + 25 comp tokens)
    let (provider, call_counter) = CountingFlakyProvider::new(1, 25, 25);
    let provider_arc: Arc<dyn LlmProvider> = Arc::new(provider);

    let mut graph = WorkflowGraph::new();
    let node = TaskNode::new("flaky_node", "Flaky Task", "Compute")
        .with_agent(AutonomousAgent::new(provider_arc, "test_model", ToolRegistry::new()))
        .with_retry_policy(RetryPolicy::linear(2, Duration::from_millis(5)));

    graph.add_task(node).unwrap();

    let ctx = EngineContext::new(5.0);
    let scheduler = DagScheduler::new();

    let res = scheduler.run(&mut graph, &ctx).await.unwrap();

    assert_eq!(call_counter.load(Ordering::SeqCst), 2);
    assert_eq!(res.completed_tasks, 1);

    // Attempt 1 failed at HTTP layer (0 tokens generated) and attempt 2 succeeded with 25+25 = 50 tokens
    let total_reported_tokens = res.total_usage.prompt_tokens + res.total_usage.completion_tokens;
    assert_eq!(
        total_reported_tokens, 50,
        "Remediation verified: Successful attempt token usage is properly recorded in WorkflowResult"
    );
}

// =========================================================================
// REMEDIATION 5: Fast Fail on Fatal Non-Retryable Errors (BudgetExceeded)
// =========================================================================
#[tokio::test]
async fn test_remediation_fast_fail_on_fatal_budget_exceeded() {
    let call_counter = Arc::new(AtomicUsize::new(0));
    let provider = Arc::new(BudgetExceededTriggerProvider {
        calls: call_counter.clone(),
    });

    let mut graph = WorkflowGraph::new();
    let node = TaskNode::new("budget_fail_task", "Budget Fail Task", "Compute")
        .with_agent(AutonomousAgent::new(provider, "test_model", ToolRegistry::new()))
        .with_retry_policy(RetryPolicy::linear(3, Duration::from_millis(5))); // max_retries = 3

    graph.add_task(node).unwrap();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let scheduler = DagScheduler::new().with_event_sender(event_tx);
    let ctx = EngineContext::new(5.0);

    let res = scheduler.run(&mut graph, &ctx).await;
    assert!(res.is_err());

    // Non-retryable BudgetExceeded must fail fast on attempt 1 with 0 retries
    let total_attempts = call_counter.load(Ordering::SeqCst);
    assert_eq!(
        total_attempts, 1,
        "Remediation verified: Non-retryable BudgetExceeded error failed immediately on attempt 1"
    );

    let mut retries_emitted = 0;
    while let Ok(evt) = event_rx.try_recv() {
        if let WorkflowEvent::TaskRetry { .. } = evt {
            retries_emitted += 1;
        }
    }
    assert_eq!(
        retries_emitted, 0,
        "Remediation verified: No retry events emitted for fatal error"
    );
}

// =========================================================================
// REMEDIATION 6: Registered Non-Default Providers are Executed Correctly
// =========================================================================
#[tokio::test]
async fn test_remediation_execution_with_registered_providers() {
    let mut ctx = EngineContext::new(5.0);
    ctx.register_provider(Arc::new(EchoMockProvider::new("custom_llm")));

    let mut graph = WorkflowGraph::new();
    let node = TaskNode::new("unattached_task", "Unattached Task", "What is 2 + 2?");
    graph.add_task(node).unwrap();

    let scheduler = DagScheduler::new();
    let res = scheduler.run(&mut graph, &ctx).await.unwrap();

    // Default provider executes the prompt rather than returning raw unexecuted string
    assert_eq!(
        res.task_outputs["unattached_task"].text, "Executed: What is 2 + 2?",
        "Remediation verified: Registered provider was automatically invoked for unattached task"
    );
    assert!(res.task_outputs["unattached_task"].usage.prompt_tokens > 0);
}

// =========================================================================
// REMEDIATION 7: Multi-Leaf Fan-Out Output Collection
// =========================================================================
#[tokio::test]
async fn test_remediation_multi_leaf_output_collection() {
    let prov = Arc::new(EchoMockProvider::new("echo_prov"));

    let mut graph = WorkflowGraph::new();

    let node_a = TaskNode::new("root", "Root", "Start")
        .with_agent(AutonomousAgent::new(prov.clone(), "m1", ToolRegistry::new()));
    let node_b = TaskNode::new("leaf_b", "Leaf B", "Analyze B")
        .with_agent(AutonomousAgent::new(prov.clone(), "m2", ToolRegistry::new()));
    let node_c = TaskNode::new("leaf_c", "Leaf C", "Analyze C")
        .with_agent(AutonomousAgent::new(prov.clone(), "m3", ToolRegistry::new()));

    graph.add_task(node_a).unwrap();
    graph.add_task(node_b).unwrap();
    graph.add_task(node_c).unwrap();

    graph.add_dependency("root", "leaf_b").unwrap();
    graph.add_dependency("root", "leaf_c").unwrap();

    let ctx = EngineContext::new(5.0);
    let scheduler = DagScheduler::new();

    let res = scheduler.run(&mut graph, &ctx).await.unwrap();

    assert_eq!(res.completed_tasks, 3);
    assert_eq!(res.leaf_outputs.len(), 2);
    assert!(res.leaf_outputs.contains_key("leaf_b"));
    assert!(res.leaf_outputs.contains_key("leaf_c"));
    assert!(res.leaf_outputs["leaf_b"].text.contains("Analyze B"));
    assert!(res.leaf_outputs["leaf_c"].text.contains("Analyze C"));
}
