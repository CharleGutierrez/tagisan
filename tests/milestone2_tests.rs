use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    AutonomousAgent, BoxEventStream, CalculatorTool, CascadeEntry, CascadeProvider,
    CompletionRequest, CompletionResponse, ContentBlock, EngineContext, FinishReason, LlmProvider,
    Message, ProviderCapabilities, ReadFileTool, RunCommandTool, TagisanError,
    TokenUsage, ToolHandler, ToolRegistry, WriteFileTool,
};

// =========================================================================
// Mock Providers for Unit & Integration Testing
// =========================================================================

/// Mock provider that simulates a script of responses or errors
struct MockScriptedProvider {
    id: &'static str,
    responses: tokio::sync::Mutex<Vec<Result<CompletionResponse, TagisanError>>>,
    call_count: Arc<AtomicUsize>,
}

impl MockScriptedProvider {
    fn new(id: &'static str, script: Vec<Result<CompletionResponse, TagisanError>>) -> Self {
        Self {
            id,
            responses: tokio::sync::Mutex::new(script),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl LlmProvider for MockScriptedProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        let mut script = self.responses.lock().await;
        if script.is_empty() {
            Ok(CompletionResponse {
                id: "mock_resp".to_string(),
                provider: self.id.to_string(),
                model: req.model,
                message: Message::assistant("Default mock text"),
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
            "Stream not implemented in mock".to_string(),
        ))
    }
}

// =========================================================================
// 1. Tool Registry & Builtin Tools Tests
// =========================================================================

#[tokio::test]
async fn test_tool_registry_registration_and_execution() {
    let mut registry = ToolRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    registry.register_tool(CalculatorTool::new());
    registry.register_tool(ReadFileTool::new());

    assert_eq!(registry.len(), 2);
    assert!(registry.contains("calculator"));
    assert!(registry.contains("read_file"));
    assert!(!registry.contains("unknown_tool"));

    let defs = registry.definitions();
    assert_eq!(defs.len(), 2);

    // Execute valid tool call
    let result_block = registry
        .execute_call("call_001", "calculator", &json!({"expression": "10 * 5 + 2"}))
        .await;

    if let ContentBlock::ToolResult {
        tool_call_id,
        content,
        is_error,
    } = result_block
    {
        assert_eq!(tool_call_id, "call_001");
        assert_eq!(content, "52");
        assert!(!is_error);
    } else {
        panic!("Expected ContentBlock::ToolResult");
    }

    // Execute unregistered tool call
    let err_block = registry
        .execute_call("call_002", "unregistered_tool", &json!({}))
        .await;

    if let ContentBlock::ToolResult { is_error, .. } = err_block {
        assert!(is_error);
    } else {
        panic!("Expected ContentBlock::ToolResult");
    }
}

#[tokio::test]
async fn test_builtin_read_and_write_file_tools() {
    let temp_dir = std::env::temp_dir().join("tagisan_test_dir");
    let test_file = temp_dir.join("sample.txt");
    let test_file_str = test_file.to_str().unwrap();

    let writer = WriteFileTool::new();
    let write_res = writer
        .execute(json!({
            "path": test_file_str,
            "content": "Tagisan ng Talino Rust Engine"
        }))
        .await;
    assert!(write_res.is_ok());

    let reader = ReadFileTool::new();
    let read_res = reader
        .execute(json!({
            "path": test_file_str
        }))
        .await;

    assert!(read_res.is_ok());
    assert_eq!(read_res.unwrap(), "Tagisan ng Talino Rust Engine");

    // Test non-existent file
    let non_existent = temp_dir.join("non_existent_file.txt");
    let non_exist_res = reader
        .execute(json!({
            "path": non_existent.to_str().unwrap()
        }))
        .await;
    assert!(non_exist_res.is_err());

    // Cleanup
    std::fs::remove_dir_all(temp_dir).ok();
}

#[tokio::test]
async fn test_builtin_run_command_tool() {
    let runner = RunCommandTool::default();

    let cmd_str = if cfg!(target_os = "windows") {
        "echo HelloTagisan"
    } else {
        "echo HelloTagisan"
    };

    let result = runner.execute(json!({"command": cmd_str})).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Exit Code: 0"));
    assert!(output.contains("HelloTagisan"));
}

#[tokio::test]
async fn test_builtin_calculator_tool_expressions() {
    let calc = CalculatorTool::new();

    let cases = vec![
        ("2 + 2", "4"),
        ("3 * (4 + 5)", "27"),
        ("2 ^ 3", "8"),
        ("100 / 4", "25"),
        ("10 % 3", "1"),
        ("sqrt(64)", "8"),
        ("abs(-15)", "15"),
        ("floor(3.9)", "3"),
        ("ceil(3.1)", "4"),
        ("round(3.6)", "4"),
        ("1e3", "1000"),
        ("2.5e-2", "0.025"),
        ("1E+4", "10000"),
        ("sin(0)", "0"),
        ("cos(0)", "1"),
        ("tan(0)", "0"),
        ("asin(0)", "0"),
        ("acos(1)", "0"),
        ("atan(0)", "0"),
    ];

    for (expr, expected) in cases {
        let res = calc.execute(json!({"expression": expr})).await.unwrap();
        assert_eq!(res, expected, "Failed for expr: {expr}");
    }

    // Test error cases
    let div_zero = calc.execute(json!({"expression": "10 / 0"})).await;
    assert!(div_zero.is_err());

    let invalid_syntax = calc.execute(json!({"expression": "2 +* 3"})).await;
    assert!(invalid_syntax.is_err());
}

// =========================================================================
// 2. Autonomous Agent Feedback Loop Tests
// =========================================================================

#[tokio::test]
async fn test_autonomous_agent_multi_turn_tool_calling() {
    let mut registry = ToolRegistry::new();
    registry.register_tool(CalculatorTool::new());

    // Turn 1: Model requests calculator tool call: 42 * 2
    let turn1_resp = CompletionResponse {
        id: "resp_1".to_string(),
        provider: "mock".to_string(),
        model: "mock-model".to_string(),
        message: Message::tool_call(
            "call_abc",
            "calculator",
            json!({"expression": "42 * 2"}),
        ),
        finish_reason: FinishReason::ToolCalls,
        usage: TokenUsage {
            prompt_tokens: 50,
            completion_tokens: 20,
            ..Default::default()
        },
        latency: Duration::from_millis(10),
    };

    // Turn 2: Model uses tool result to output final answer
    let turn2_resp = CompletionResponse {
        id: "resp_2".to_string(),
        provider: "mock".to_string(),
        model: "mock-model".to_string(),
        message: Message::assistant("The calculated result is 84."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage {
            prompt_tokens: 80,
            completion_tokens: 15,
            ..Default::default()
        },
        latency: Duration::from_millis(10),
    };

    let mock_provider = Arc::new(MockScriptedProvider::new(
        "mock_llm",
        vec![Ok(turn1_resp), Ok(turn2_resp)],
    ));

    let agent = AutonomousAgent::new(mock_provider, "mock-model", registry)
        .with_max_iterations(5);

    let ctx = EngineContext::new(1.0);
    let result = agent.run("What is 42 * 2?", &ctx).await.unwrap();

    assert_eq!(result.iterations, 2);
    assert_eq!(result.final_answer, "The calculated result is 84.");
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.steps[0].tool_results.len(), 1);

    if let ContentBlock::ToolResult {
        tool_call_id,
        content,
        is_error,
    } = &result.steps[0].tool_results[0]
    {
        assert_eq!(tool_call_id, "call_abc");
        assert_eq!(content, "84");
        assert!(!is_error);
    } else {
        panic!("Expected ContentBlock::ToolResult");
    }
}

#[tokio::test]
async fn test_autonomous_agent_iteration_limit_enforcement() {
    let mut registry = ToolRegistry::new();
    registry.register_tool(CalculatorTool::new());

    // Infinite tool call responses
    let mut infinite_script = Vec::new();
    for i in 0..10 {
        infinite_script.push(Ok(CompletionResponse {
            id: format!("resp_{i}"),
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            message: Message::tool_call(
                format!("call_{i}"),
                "calculator",
                json!({"expression": "1 + 1"}),
            ),
            finish_reason: FinishReason::ToolCalls,
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 10,
                ..Default::default()
            },
            latency: Duration::from_millis(5),
        }));
    }

    let mock_provider = Arc::new(MockScriptedProvider::new("mock_llm", infinite_script));

    let agent = AutonomousAgent::new(mock_provider, "mock-model", registry)
        .with_max_iterations(3);

    let ctx = EngineContext::new(1.0);
    let result = agent.run("Run infinite loop", &ctx).await.unwrap();

    assert_eq!(result.iterations, 3);
    assert_eq!(result.steps.len(), 3);
}

// =========================================================================
// 3. Cascade Provider Failover Tests
// =========================================================================

#[tokio::test]
async fn test_cascade_provider_seamless_failover() {
    // Provider 1: Returns RateLimited error
    let p1 = Arc::new(MockScriptedProvider::new(
        "primary_claude",
        vec![Err(TagisanError::RateLimited(
            "primary_claude".to_string(),
            Some(Duration::from_secs(60)),
        ))],
    ));

    // Provider 2: Returns BadResponse 502 Bad Gateway
    let p2 = Arc::new(MockScriptedProvider::new(
        "secondary_grok",
        vec![Err(TagisanError::BadResponse(
            "secondary_grok".to_string(),
            "HTTP 502 Bad Gateway".to_string(),
        ))],
    ));

    // Provider 3: Succeeds
    let p3_resp = CompletionResponse {
        id: "p3_resp".to_string(),
        provider: "fallback_deepseek".to_string(),
        model: "deepseek-chat".to_string(),
        message: Message::assistant("Response from fallback DeepSeek"),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage {
            prompt_tokens: 20,
            completion_tokens: 20,
            ..Default::default()
        },
        latency: Duration::from_millis(15),
    };
    let p3 = Arc::new(MockScriptedProvider::new(
        "fallback_deepseek",
        vec![Ok(p3_resp)],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p1.clone(), Some("claude-3-5-sonnet".to_string())),
        CascadeEntry::new(p2.clone(), Some("grok-2".to_string())),
        CascadeEntry::new(p3.clone(), Some("deepseek-chat".to_string())),
    ]);

    let req = CompletionRequest::new("default-model", "Test prompt");
    let resp = cascade.complete(req).await.expect("Cascade should succeed on fallback");

    assert_eq!(resp.provider, "fallback_deepseek");
    assert_eq!(resp.message.extract_text(), "Response from fallback DeepSeek");
    assert_eq!(p1.call_count.load(Ordering::SeqCst), 1);
    assert_eq!(p2.call_count.load(Ordering::SeqCst), 1);
    assert_eq!(p3.call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_cascade_provider_non_retryable_error_does_not_cascade() {
    // Provider 1: Returns BudgetExceeded (Non-retryable)
    let p1 = Arc::new(MockScriptedProvider::new(
        "primary_claude",
        vec![Err(TagisanError::BudgetExceeded {
            max_budget: 1.0,
            current_spent: 1.05,
        })],
    ));

    // Provider 2: Should NOT be reached
    let p2 = Arc::new(MockScriptedProvider::new(
        "secondary_grok",
        vec![Ok(CompletionResponse {
            id: "p2_resp".to_string(),
            provider: "secondary_grok".to_string(),
            model: "grok-2".to_string(),
            message: Message::assistant("Should not run"),
            finish_reason: FinishReason::Stop,
            usage: Default::default(),
            latency: Duration::from_millis(5),
        })],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p1.clone(), None),
        CascadeEntry::new(p2.clone(), None),
    ]);

    let req = CompletionRequest::new("default-model", "Test prompt");
    let err = cascade.complete(req).await.unwrap_err();

    assert!(matches!(err, TagisanError::BudgetExceeded { .. }));
    assert_eq!(p1.call_count.load(Ordering::SeqCst), 1);
    assert_eq!(p2.call_count.load(Ordering::SeqCst), 0, "Provider 2 should not have been called!");
}

#[tokio::test]
async fn test_autonomous_agent_parallel_multi_tool_execution() {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(CalculatorTool));
    registry.register(Arc::new(ReadFileTool::new()));

    // Scripted provider that in Turn 1 calls 2 tools in parallel
    let p = Arc::new(MockScriptedProvider::new(
        "test_agent_prov",
        vec![
            // Turn 1: Assistant calls calculator and another tool simultaneously
            Ok(CompletionResponse {
                id: "step1".to_string(),
                provider: "test_agent_prov".to_string(),
                model: "test_model".to_string(),
                message: Message {
                    role: tagisan::Role::Assistant,
                    content: vec![
                        ContentBlock::ToolCall {
                            id: "call_calc_1".to_string(),
                            name: "calculator".to_string(),
                            arguments: json!({"expression": "100 + 50"}),
                        },
                        ContentBlock::ToolCall {
                            id: "call_calc_2".to_string(),
                            name: "calculator".to_string(),
                            arguments: json!({"expression": "200 * 2"}),
                        },
                    ],
                    name: None,
                    metadata: Default::default(),
                },
                finish_reason: FinishReason::ToolCalls,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    ..Default::default()
                },
                latency: Duration::from_millis(5),
            }),
            // Turn 2: Assistant receives both tool results in ONE unified message turn and finishes
            Ok(CompletionResponse {
                id: "step2".to_string(),
                provider: "test_agent_prov".to_string(),
                model: "test_model".to_string(),
                message: Message::assistant("The results are 150 and 400."),
                finish_reason: FinishReason::Stop,
                usage: TokenUsage {
                    prompt_tokens: 30,
                    completion_tokens: 10,
                    ..Default::default()
                },
                latency: Duration::from_millis(5),
            }),
        ],
    ));

    let agent = AutonomousAgent::new(p, "test_model", registry);
    let ctx = EngineContext::new(5.0);

    let result = agent.run("Calculate both numbers", &ctx).await.unwrap();
    assert_eq!(result.final_answer, "The results are 150 and 400.");
    assert_eq!(result.iterations, 2);

    // Verify history structure: User -> Assistant (2 tool calls) -> Tool (1 merged message with 2 results) -> Assistant (final)
    assert_eq!(result.history.len(), 4);
    assert_eq!(result.history[0].role, tagisan::Role::User);
    assert_eq!(result.history[1].role, tagisan::Role::Assistant);
    assert_eq!(result.history[2].role, tagisan::Role::Tool);
    assert_eq!(result.history[2].content.len(), 2, "Both tool results MUST be in a single turn!");
    assert_eq!(result.history[3].role, tagisan::Role::Assistant);
}

#[test]
fn test_tui_debate_event_streaming_state_machine() {
    use tagisan::tui::{DebateEvent, TuiState};

    let mut state = TuiState::new("Test Architecture Topic");

    // Real-time chunk streaming into Round 1
    state.apply_event(DebateEvent::Round1Chunk("In my opinion, ".to_string()));
    state.apply_event(DebateEvent::Round1Chunk("Rust is the superior choice.".to_string()));
    assert_eq!(state.proponent_text, "In my opinion, Rust is the superior choice.");

    // Real-time chunk streaming into Round 2
    state.apply_event(DebateEvent::Round2Chunk("However, we must consider ".to_string()));
    state.apply_event(DebateEvent::Round2Chunk("compile times.".to_string()));
    assert_eq!(state.adversary_text, "However, we must consider compile times.");

    // Real-time chunk streaming into Round 3
    state.apply_event(DebateEvent::Round3Chunk("Lakandiwa verdict: ".to_string()));
    state.apply_event(DebateEvent::Round3Chunk("Balanced approach wins.".to_string()));
    assert_eq!(state.lakandiwa_text, "Lakandiwa verdict: Balanced approach wins.");
}
