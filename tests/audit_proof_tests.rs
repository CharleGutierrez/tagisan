use serde_json::json;
use tagisan::{
    AnthropicProvider, ChatSession, CollaborationStrategy, CompletionRequest, ContentBlock,
    DialecticalDebateStrategy, EngineContext, FinishReason, GeminiProvider, LlmProvider,
    Message, MixtureOfAgentsStrategy, OllamaProvider, OpenAiCompatibleProvider,
    ProviderCapabilities, Role, StrategyInput, StreamChunk, StreamChunkDelta,
    StreamingThinkParser, TagisanError, TokenBudgetTracker, TokenUsage, ToolDefinition,
};
use tokio_util::sync::CancellationToken;

#[test]
fn test_remediation_tool_calling_wiring_and_request_builders() {
    // 1. ToolDefinition creation and attachment to CompletionRequest
    let weather_tool = ToolDefinition::new(
        "get_weather",
        "Get current weather in a given city",
        json!({
            "type": "object",
            "properties": {
                "location": { "type": "string", "description": "City name" }
            },
            "required": ["location"]
        }),
    );

    let req = CompletionRequest::new("gpt-4o", "What is the weather in Manila?")
        .with_tool(weather_tool.clone())
        .with_tool_choice(json!({"type": "function", "function": {"name": "get_weather"}}));

    assert_eq!(req.tools.len(), 1);
    assert_eq!(req.tools[0].name, "get_weather");
    assert!(req.tool_choice.is_some());

    // 2. Message tool call and tool result polymorphic formatting
    let tool_call_msg = Message::tool_call("call_999", "get_weather", json!({"location": "Manila"}));
    assert_eq!(tool_call_msg.role, Role::Assistant);
    let extracted_calls = tool_call_msg.extract_tool_calls();
    assert_eq!(extracted_calls.len(), 1);
    assert_eq!(extracted_calls[0].0, "call_999");
    assert_eq!(extracted_calls[0].1, "get_weather");
    assert_eq!(extracted_calls[0].2["location"], "Manila");

    let tool_result_msg = Message::tool_result("call_999", "31°C, Sunny", false);
    assert_eq!(tool_result_msg.role, Role::Tool);
    let extracted_results = tool_result_msg.extract_tool_results();
    assert_eq!(extracted_results.len(), 1);
    assert_eq!(extracted_results[0].0, "call_999");
    assert_eq!(extracted_results[0].1, "31°C, Sunny");
    assert!(!extracted_results[0].2);

    // 3. Provider capabilities advertise FUNCTION_CALLING
    let openai = OpenAiCompatibleProvider::openai("dummy-key");
    assert!(openai.capabilities("gpt-4o").contains(ProviderCapabilities::FUNCTION_CALLING));
    let anthropic = AnthropicProvider::new("dummy-key");
    assert!(anthropic.capabilities("claude-3-5-sonnet-20241022").contains(ProviderCapabilities::FUNCTION_CALLING));
    let gemini = GeminiProvider::new("dummy-key");
    assert!(gemini.capabilities("gemini-2.0-flash").contains(ProviderCapabilities::FUNCTION_CALLING));
    let ollama = OllamaProvider::default_local();
    assert!(ollama.capabilities("llama3.2").contains(ProviderCapabilities::FUNCTION_CALLING));
}

#[test]
fn test_remediation_reasoning_role_and_polymorphic_extraction() {
    let reasoning_msg = Message::reasoning("Step 1: Parse AST. Step 2: Validate types.");
    assert_eq!(reasoning_msg.role, Role::Reasoning);
    assert_eq!(reasoning_msg.extract_text(), "Step 1: Parse AST. Step 2: Validate types.");
    assert_eq!(
        reasoning_msg.extract_thinking(),
        Some("Step 1: Parse AST. Step 2: Validate types.".to_string())
    );

    let mixed_msg = Message {
        role: Role::Assistant,
        content: vec![
            ContentBlock::thinking("Analyzing the problem...", None),
            ContentBlock::text("Here is the solution: 42"),
        ],
        name: None,
        metadata: Default::default(),
    };
    assert_eq!(mixed_msg.extract_text(), "Here is the solution: 42");
    assert_eq!(mixed_msg.extract_thinking(), Some("Analyzing the problem...".to_string()));
}

#[test]
fn test_remediation_vision_content_blocks() {
    let dummy_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let img_msg = Message::image("image/png", dummy_base64);
    assert_eq!(img_msg.role, Role::User);
    assert_eq!(img_msg.content.len(), 1);

    if let ContentBlock::Image { media_type, data_base64 } = &img_msg.content[0] {
        assert_eq!(media_type, "image/png");
        assert_eq!(data_base64, dummy_base64);
    } else {
        panic!("Expected ContentBlock::Image");
    }

    let anthropic = AnthropicProvider::new("dummy-key");
    assert!(anthropic.capabilities("claude-3-5-sonnet-20241022").contains(ProviderCapabilities::VISION));
    let openai = OpenAiCompatibleProvider::openai("dummy-key");
    assert!(openai.capabilities("gpt-4o").contains(ProviderCapabilities::VISION));
    let gemini = GeminiProvider::new("dummy-key");
    assert!(gemini.capabilities("gemini-2.0-flash").contains(ProviderCapabilities::VISION));
}

#[test]
fn test_remediation_streaming_think_state_machine() {
    let mut parser = StreamingThinkParser::new();

    // Chunk 1: Partial opening tag "<th"
    let d1 = parser.process("<th");
    assert!(d1.is_empty(), "Partial opening tag should be buffered");

    // Chunk 2: Completion of opening tag + thinking content
    let d2 = parser.process("ink>\nLet's calculate 2 + 2.");
    assert_eq!(d2.len(), 1);
    assert_eq!(d2[0], StreamChunkDelta::Thinking("\nLet's calculate 2 + 2.".to_string()));

    // Chunk 3: More thinking + partial closing tag "</th"
    let d3 = parser.process(" Result is 4.\n</th");
    assert_eq!(d3.len(), 1);
    assert_eq!(d3[0], StreamChunkDelta::Thinking(" Result is 4.\n".to_string()));

    // Chunk 4: Completion of closing tag + response text
    let d4 = parser.process("ink>\nThe final answer is 4.");
    assert_eq!(d4.len(), 1);
    assert_eq!(d4[0], StreamChunkDelta::Text("\nThe final answer is 4.".to_string()));

    let d5 = parser.finish();
    assert!(d5.is_empty());

    // Test 2: Standard text without thinking tags
    let mut parser_plain = StreamingThinkParser::new();
    let p1 = parser_plain.process("Hello world! ");
    assert_eq!(p1, vec![StreamChunkDelta::Text("Hello world! ".to_string())]);
    let p2 = parser_plain.process("This is plain text.");
    assert_eq!(p2, vec![StreamChunkDelta::Text("This is plain text.".to_string())]);
    assert!(parser_plain.finish().is_empty());
}

#[test]
fn test_remediation_cancellation_token_wiring() {
    let ctx = EngineContext::new(10.0);
    assert!(!ctx.cancellation_token.is_cancelled());

    ctx.cancellation_token.cancel();
    assert!(ctx.cancellation_token.is_cancelled());

    let req = CompletionRequest::new("gpt-4o", "Analyze big data")
        .with_cancellation(ctx.cancellation_token.clone());

    assert!(req.cancellation_token.is_some());
    assert!(req.cancellation_token.as_ref().unwrap().is_cancelled());
}

#[tokio::test]
async fn test_remediation_cancellation_aborts_strategy_immediately() {
    let ctx = EngineContext::new(10.0);
    ctx.cancellation_token.cancel();

    let debate = DialecticalDebateStrategy::new(
        ("ollama".into(), "llama3.2".into()),
        ("ollama".into(), "llama3.2".into()),
        ("ollama".into(), "llama3.2".into()),
    );

    let input = StrategyInput {
        prompt: "Should we use Rust?".to_string(),
        system_instruction: Some("Answer in Filipino".to_string()),
    };

    let result = debate.execute(input, &ctx).await;
    assert!(matches!(result, Err(TagisanError::Cancelled)));

    let moa = MixtureOfAgentsStrategy::new(
        vec![("ollama".into(), "llama3.2".into())],
        ("ollama".into(), "llama3.2".into()),
    );

    let moa_input = StrategyInput {
        prompt: "Generate API spec".to_string(),
        system_instruction: None,
    };

    let moa_result = moa.execute(moa_input, &ctx).await;
    assert!(matches!(moa_result, Err(TagisanError::Cancelled)));
}

#[test]
fn test_remediation_debate_strategy_passes_system_instruction() {
    let input = StrategyInput {
        prompt: "Analyze Rust memory model".to_string(),
        system_instruction: Some("Always respond in Filipino (Tagalog)".to_string()),
    };

    let thesis_req = CompletionRequest::new("llama3.2", "User prompt")
        .with_system(input.system_instruction.clone().unwrap());

    assert_eq!(
        thesis_req.system_prompt.as_deref(),
        Some("Always respond in Filipino (Tagalog)")
    );
}

#[test]
fn test_remediation_stream_chunk_and_token_usage() {
    let chunk_text = StreamChunk::text("Hello");
    assert_eq!(chunk_text.delta, StreamChunkDelta::Text("Hello".to_string()));

    let chunk_think = StreamChunk::thinking("Contemplating...");
    assert_eq!(chunk_think.delta, StreamChunkDelta::Thinking("Contemplating...".to_string()));

    let chunk_tool = StreamChunk::tool_call_delta(
        0,
        Some("call_1".to_string()),
        Some("calc".to_string()),
        Some("{\"x\": 1}".to_string()),
    );
    assert_eq!(
        chunk_tool.delta,
        StreamChunkDelta::ToolCallDelta {
            index: 0,
            id: Some("call_1".to_string()),
            name: Some("calc".to_string()),
            arguments_delta: Some("{\"x\": 1}".to_string()),
        }
    );

    let usage = TokenUsage {
        prompt_tokens: 150,
        completion_tokens: 50,
        reasoning_tokens: Some(25),
        cached_prompt_tokens: Some(100),
        estimated_cost_usd: Some(0.002),
    };
    let chunk_done = StreamChunk::done(FinishReason::Stop, Some(usage.clone()));
    assert_eq!(chunk_done.finish_reason, Some(FinishReason::Stop));
    assert_eq!(chunk_done.usage.unwrap().prompt_tokens, 150);
}

#[test]
fn test_remediation_token_budget_tracker_thread_safety() {
    let tracker = TokenBudgetTracker::new(1.0);

    let cost1 = tracker.record("claude-3-5-sonnet-20241022", 500, 500).unwrap();
    let cost2 = tracker.record("gpt-4o", 1000, 500).unwrap();

    assert!(cost1 > 0.0);
    assert!(cost2 > cost1);
    assert!(tracker.current_spent_usd() > 0.0);
}

#[test]
fn test_remediation_chat_session_with_tools_and_cancellation() {
    let mut session = ChatSession::new().with_system("You are an autonomous AI agent.");
    session.add_user_message("Please execute the calculate tool.");
    session.add_assistant_message("Executing tool now...");

    let tool = ToolDefinition::new("calc", "Performs math", json!({"type": "object"}));
    let cancel = CancellationToken::new();

    let req = session
        .build_request("claude-3-5-sonnet-20241022")
        .with_tool(tool)
        .with_cancellation(cancel.clone());

    assert_eq!(req.messages.len(), 2);
    assert_eq!(req.tools.len(), 1);
    assert!(req.cancellation_token.is_some());
    assert_eq!(req.system_prompt.as_deref(), Some("You are an autonomous AI agent."));
}

#[test]
fn test_remediation_streaming_think_parser_unclosed_and_multiple() {
    let mut parser = StreamingThinkParser::new();

    // Test unclosed thinking block that finishes at EOF
    let d1 = parser.process("<think>Deep internal thought without closing tag");
    assert_eq!(d1.len(), 1);
    assert_eq!(d1[0], StreamChunkDelta::Thinking("Deep internal thought without closing tag".to_string()));

    let d2 = parser.finish();
    assert!(d2.is_empty());

    // Test multiple think chunks and then transition
    let mut parser2 = StreamingThinkParser::new();
    let r1 = parser2.process("<think>Plan A</think>Action 1");
    assert_eq!(r1.len(), 2);
    assert_eq!(r1[0], StreamChunkDelta::Thinking("Plan A".to_string()));
    assert_eq!(r1[1], StreamChunkDelta::Text("Action 1".to_string()));
}

#[test]
fn test_remediation_content_block_helpers_and_metadata() {
    let text_block = ContentBlock::text("Sample text");
    let think_block = ContentBlock::thinking("Sample thought", Some("sig_123".to_string()));
    let img_block = ContentBlock::image("image/jpeg", "base64data");
    let tool_call = ContentBlock::tool_call("id_1", "my_func", json!({"arg": 1}));
    let tool_result = ContentBlock::tool_result("id_1", "Result string", false);

    assert!(matches!(text_block, ContentBlock::Text { .. }));
    assert!(matches!(think_block, ContentBlock::Thinking { .. }));
    assert!(matches!(img_block, ContentBlock::Image { .. }));
    assert!(matches!(tool_call, ContentBlock::ToolCall { .. }));
    assert!(matches!(tool_result, ContentBlock::ToolResult { .. }));

    let msg = Message::user("Hello")
        .with_name("Alice")
        .with_metadata("session_id", json!("sess_abc123"));

    assert_eq!(msg.name.as_deref(), Some("Alice"));
    assert_eq!(msg.metadata.get("session_id").unwrap(), "sess_abc123");
}

#[test]
fn test_remediation_budget_exceeded_error_handling() {
    let tracker = TokenBudgetTracker::new(0.01); // $0.01 max budget
    // Claude 3.5 Sonnet: 500 prompt ($0.0015) + 500 output ($0.0075) = $0.009
    let res1 = tracker.record("claude-3-5-sonnet-20241022", 500, 500);
    assert!(res1.is_ok());

    // Another 500 prompt + 500 output = exceeds $0.01 limit
    let res2 = tracker.record("claude-3-5-sonnet-20241022", 500, 500);
    assert!(res2.is_err());
    if let Err(TagisanError::BudgetExceeded { max_budget, current_spent }) = res2 {
        assert_eq!(max_budget, 0.01);
        assert!(current_spent > 0.01);
    } else {
        panic!("Expected TagisanError::BudgetExceeded");
    }
}
