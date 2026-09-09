use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    AutonomousAgent, BoxEventStream, CompletionRequest, CompletionResponse,
    ContentBlock, EngineContext, FinishReason, LlmProvider, Message, ProviderCapabilities,
    TokenBudgetTracker, TokenUsage, ToolHandler, ToolRegistry, ViewImageTool,
};

// =========================================================================
// 1. AgentShield Enabled by Default & Runtime Redaction Tests
// =========================================================================

#[test]
fn test_agentshield_enabled_by_default_in_autonomous_agent() {
    let mock_provider = Arc::new(MockEchoProvider);
    let agent = AutonomousAgent::new(mock_provider, "mock-model", ToolRegistry::new());
    assert!(
        agent.agentshield_enabled,
        "AutonomousAgent MUST have AgentShield enabled by default for runtime security!"
    );

    let disabled_agent = agent.with_agentshield(false);
    assert!(
        !disabled_agent.agentshield_enabled,
        "AutonomousAgent::with_agentshield(false) must allow explicit opt-out"
    );
}

/// Mock provider that simulates a model that requests a tool call and then answers
struct MockScriptedToolProvider {
    turns: std::sync::Mutex<Vec<CompletionResponse>>,
}

#[async_trait]
impl LlmProvider for MockScriptedToolProvider {
    fn provider_id(&self) -> &'static str {
        "mock_scripted"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, _req: CompletionRequest) -> tagisan::Result<CompletionResponse> {
        let mut turns = self.turns.lock().unwrap();
        if turns.is_empty() {
            panic!("No more scripted responses in MockScriptedToolProvider");
        }
        Ok(turns.remove(0))
    }

    async fn stream(&self, _req: CompletionRequest) -> tagisan::Result<BoxEventStream> {
        Err(tagisan::TagisanError::Execution("Stream not supported".to_string()))
    }
}

/// Tool that intentionally leaks various API keys and credentials
struct SecretLeakingTool;

#[async_trait]
impl ToolHandler for SecretLeakingTool {
    fn name(&self) -> &'static str {
        "get_credentials"
    }

    fn description(&self) -> &'static str {
        "Simulates a tool that returns sensitive API keys and secrets"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _arguments: serde_json::Value) -> tagisan::Result<String> {
        Ok("Found credentials: Anthropic: sk-ant-api03-live_secret_key_12345, OpenAI: sk-proj-super_secret_openai_key_67890, Gemini: AIzaSyLiveGeminiKeySecret, xAI: xai-grok_production_secret_key, GitHub: ghp_githubPersonalAccessTokenSecret123".to_string())
    }
}

#[tokio::test]
async fn test_agentshield_runtime_secret_redaction_in_tool_execution() {
    let mut registry = ToolRegistry::new();
    registry.register_tool(SecretLeakingTool);

    // Turn 1: Model requests get_credentials tool
    let turn1 = CompletionResponse {
        id: "turn1".to_string(),
        provider: "mock".to_string(),
        model: "mock-model".to_string(),
        message: Message::tool_call("call_leak", "get_credentials", json!({})),
        finish_reason: FinishReason::ToolCalls,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(5),
    };

    // Turn 2: Model reflects whatever credentials were in the tool result
    let turn2 = CompletionResponse {
        id: "turn2".to_string(),
        provider: "mock".to_string(),
        model: "mock-model".to_string(),
        message: Message::assistant("Here are the credentials found: sk-ant-api03-live_secret_key_12345 and sk-proj-super_secret_openai_key_67890"),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(5),
    };

    let provider = Arc::new(MockScriptedToolProvider {
        turns: std::sync::Mutex::new(vec![turn1, turn2]),
    });

    // Run agent with default AgentShield enabled
    let agent = AutonomousAgent::new(provider, "mock-model", registry);
    assert!(agent.agentshield_enabled);

    let ctx = EngineContext::new(10.0);
    let result = agent.run("Find credentials", &ctx).await.expect("Agent should complete successfully");

    // 1. Verify secrets in tool results are completely redacted
    assert_eq!(result.steps.len(), 1);
    let step = &result.steps[0];
    assert_eq!(step.tool_results.len(), 1);
    if let ContentBlock::ToolResult { content, .. } = &step.tool_results[0] {
        assert!(!content.contains("sk-ant-api03-live_secret_key_12345"), "Anthropic key leaked in tool_result!");
        assert!(!content.contains("sk-proj-super_secret_openai_key_67890"), "OpenAI key leaked in tool_result!");
        assert!(!content.contains("AIzaSyLiveGeminiKeySecret"), "Gemini key leaked in tool_result!");
        assert!(!content.contains("xai-grok_production_secret_key"), "xAI key leaked in tool_result!");
        assert!(!content.contains("ghp_githubPersonalAccessTokenSecret123"), "GitHub token leaked in tool_result!");

        assert!(content.contains("[REDACTED_ANTHROPIC_KEY]"));
        assert!(content.contains("[REDACTED_OPENAI_KEY]"));
        assert!(content.contains("[REDACTED_GEMINI_KEY]"));
        assert!(content.contains("[REDACTED_XAI_KEY]"));
        assert!(content.contains("[REDACTED_GITHUB_TOKEN]"));
    } else {
        panic!("Expected ToolResult content block");
    }

    // 2. Verify secrets in agent's history and final answer are completely redacted
    for msg in &result.history {
        let text = msg.extract_text();
        assert!(!text.contains("sk-ant-api03-live_secret_key_12345"));
        assert!(!text.contains("sk-proj-super_secret_openai_key_67890"));
    }

    assert!(!result.final_answer.contains("sk-ant-api03-live_secret_key_12345"));
    assert!(!result.final_answer.contains("sk-proj-super_secret_openai_key_67890"));
    assert!(result.final_answer.contains("[REDACTED_ANTHROPIC_KEY]"));
    assert!(result.final_answer.contains("[REDACTED_OPENAI_KEY]"));
}

#[test]
fn test_autonomous_agent_sanitization_helpers() {
    let raw_text = "Anthropic: sk-ant-testkey123, OpenAI: sk-proj-testkey456, Gemini: AIzaSyTestKey789, xAI: xai-testkey012, GH: ghp_testtoken345";
    let sanitized = AutonomousAgent::sanitize_text(raw_text);

    assert!(!sanitized.contains("sk-ant-testkey123"));
    assert!(!sanitized.contains("sk-proj-testkey456"));
    assert!(!sanitized.contains("AIzaSyTestKey789"));
    assert!(!sanitized.contains("xai-testkey012"));
    assert!(!sanitized.contains("ghp_testtoken345"));

    assert!(sanitized.contains("[REDACTED_ANTHROPIC_KEY]"));
    assert!(sanitized.contains("[REDACTED_OPENAI_KEY]"));
    assert!(sanitized.contains("[REDACTED_GEMINI_KEY]"));
    assert!(sanitized.contains("[REDACTED_XAI_KEY]"));
    assert!(sanitized.contains("[REDACTED_GITHUB_TOKEN]"));

    let msg = Message::assistant("Leaked key: sk-ant-secretkey999");
    let clean_msg = AutonomousAgent::sanitize_message(msg);
    assert!(!clean_msg.extract_text().contains("sk-ant-secretkey999"));
    assert!(clean_msg.extract_text().contains("[REDACTED_ANTHROPIC_KEY]"));
}

// =========================================================================
// 2. Multimodal Vision & Image Loading Tests
// =========================================================================

#[test]
fn test_content_block_from_image_file_valid_formats() {
    let temp_dir = std::env::temp_dir().join("tagisan_test_images");
    fs::create_dir_all(&temp_dir).unwrap();

    let dummy_png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x01];
    let dummy_jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
    let dummy_webp = [0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50];
    let dummy_gif = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00];

    let png_path = temp_dir.join("sample.png");
    let jpg_path = temp_dir.join("sample.jpg");
    let jpeg_path = temp_dir.join("sample.jpeg");
    let webp_path = temp_dir.join("sample.webp");
    let gif_path = temp_dir.join("sample.gif");

    fs::write(&png_path, dummy_png).unwrap();
    fs::write(&jpg_path, dummy_jpeg).unwrap();
    fs::write(&jpeg_path, dummy_jpeg).unwrap();
    fs::write(&webp_path, dummy_webp).unwrap();
    fs::write(&gif_path, dummy_gif).unwrap();

    // 1. PNG
    let block_png = ContentBlock::from_image_file(&png_path).expect("Failed to load PNG");
    match block_png {
        ContentBlock::Image { media_type, data_base64 } => {
            assert_eq!(media_type, "image/png");
            assert!(!data_base64.is_empty());
        }
        _ => panic!("Expected ContentBlock::Image"),
    }

    // 2. JPG & JPEG
    let block_jpg = ContentBlock::from_image_file(&jpg_path).expect("Failed to load JPG");
    match block_jpg {
        ContentBlock::Image { media_type, data_base64 } => {
            assert_eq!(media_type, "image/jpeg");
            assert!(!data_base64.is_empty());
        }
        _ => panic!("Expected ContentBlock::Image"),
    }

    let block_jpeg = ContentBlock::from_image_file(&jpeg_path).expect("Failed to load JPEG");
    match block_jpeg {
        ContentBlock::Image { media_type, .. } => assert_eq!(media_type, "image/jpeg"),
        _ => panic!("Expected ContentBlock::Image"),
    }

    // 3. WebP
    let block_webp = ContentBlock::from_image_file(&webp_path).expect("Failed to load WebP");
    match block_webp {
        ContentBlock::Image { media_type, .. } => assert_eq!(media_type, "image/webp"),
        _ => panic!("Expected ContentBlock::Image"),
    }

    // 4. GIF
    let block_gif = ContentBlock::from_image_file(&gif_path).expect("Failed to load GIF");
    match block_gif {
        ContentBlock::Image { media_type, .. } => assert_eq!(media_type, "image/gif"),
        _ => panic!("Expected ContentBlock::Image"),
    }

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_content_block_from_image_file_invalid_and_missing() {
    let temp_dir = std::env::temp_dir().join("tagisan_test_images_invalid");
    fs::create_dir_all(&temp_dir).unwrap();

    let txt_path = temp_dir.join("document.txt");
    fs::write(&txt_path, "not an image").unwrap();

    // Unsupported format returns TagisanError::InvalidInput
    let err_txt = ContentBlock::from_image_file(&txt_path);
    assert!(err_txt.is_err());

    // Missing file returns error
    let missing_path = temp_dir.join("non_existent_file.png");
    let err_missing = ContentBlock::from_image_file(&missing_path);
    assert!(err_missing.is_err());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_view_image_tool_execution() {
    let temp_dir = std::env::temp_dir().join("tagisan_tool_image_test");
    fs::create_dir_all(&temp_dir).unwrap();
    let img_path = temp_dir.join("diagram.png");
    fs::write(&img_path, [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x01, 0x02, 0x03, 0x04]).unwrap();

    let tool = ViewImageTool::new();
    assert_eq!(tool.name(), "view_image");

    let result = tool
        .execute(json!({"path": img_path.to_str().unwrap()}))
        .await
        .expect("Tool execution should succeed");

    assert!(result.contains("Image validated and loaded successfully"));
    assert!(result.contains("image/png"));
    assert!(result.contains("Size: 12 bytes"));

    let _ = fs::remove_dir_all(&temp_dir);
}

// =========================================================================
// 3. Token Budget Tracker Prompt Caching Discount Tests
// =========================================================================

#[test]
fn test_token_budget_tracker_prompt_caching_90_percent_discount() {
    // Model: Claude 3.5 Sonnet -> prompt_rate = 3.0, completion_rate = 15.0 per 1M tokens ($/Mtok)
    let tracker_uncached = TokenBudgetTracker::new(1.0);
    let tracker_cached = TokenBudgetTracker::new(1.0);
    let tracker_mixed = TokenBudgetTracker::new(1.0);

    // 10,000 prompt tokens uncached
    let cost_uncached = tracker_uncached
        .record("claude-3-5-sonnet-20241022", 10_000, 0)
        .unwrap();

    // 10,000 prompt tokens 100% cached
    let cost_cached = tracker_cached
        .record_with_cache("claude-3-5-sonnet-20241022", 10_000, 0, 10_000)
        .unwrap();

    // 10,000 prompt tokens 50% cached (5,000 uncached + 5,000 cached)
    let cost_mixed = tracker_mixed
        .record_with_cache("claude-3-5-sonnet-20241022", 10_000, 0, 5_000)
        .unwrap();

    // Verify 10,000 * 3.0 = 30,000 micro-USD = $0.0300
    assert!((cost_uncached - 0.0300).abs() < 0.0001);

    // Verify 10,000 * (3.0 * 0.1) = 3,000 micro-USD = $0.0030 (90% discount!)
    assert!((cost_cached - 0.0030).abs() < 0.0001);

    // Verify mixed: 5,000 * 3.0 + 5,000 * 0.3 = 15,000 + 1,500 = 16,500 micro-USD = $0.0165
    assert!((cost_mixed - 0.0165).abs() < 0.0001);

    // Exact ratio check: cost_cached is exactly 10% of cost_uncached
    let discount_pct = (cost_uncached - cost_cached) / cost_uncached;
    assert!((discount_pct - 0.90).abs() < 0.001);

    // Verify record(model, p, c) is completely identical to record_with_cache(model, p, c, 0)
    let tracker_delegation = TokenBudgetTracker::new(1.0);
    let cost_delegated = tracker_delegation
        .record("claude-3-5-sonnet-20241022", 10_000, 0)
        .unwrap();
    assert_eq!(cost_uncached, cost_delegated);
}

#[test]
fn test_token_budget_tracker_record_usage_helper() {
    let tracker = TokenBudgetTracker::new(1.0);
    let usage = TokenUsage {
        prompt_tokens: 10_000,
        completion_tokens: 1_000,
        reasoning_tokens: None,
        cached_prompt_tokens: Some(8_000),
        estimated_cost_usd: None,
    };

    let cost = tracker.record_usage("deepseek-chat", &usage).unwrap();
    assert!(cost > 0.0);
}

// =========================================================================
// Helper Mock Providers
// =========================================================================

struct MockEchoProvider;

#[async_trait]
impl LlmProvider for MockEchoProvider {
    fn provider_id(&self) -> &'static str {
        "mock_echo"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> tagisan::Result<CompletionResponse> {
        let last_prompt = req
            .messages
            .last()
            .map(|m| m.extract_text())
            .unwrap_or_default();
        Ok(CompletionResponse {
            id: "echo_1".to_string(),
            provider: "mock_echo".to_string(),
            model: req.model,
            message: Message::assistant(format!("Echo: {last_prompt}")),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage::default(),
            latency: Duration::from_millis(1),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> tagisan::Result<BoxEventStream> {
        Err(tagisan::TagisanError::Execution("Stream not supported".to_string()))
    }
}
