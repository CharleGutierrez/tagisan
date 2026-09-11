//! Comprehensive Brutal Stress & Integration Tests for Structured Role-Based Harmony Swarm (`tgs harmony`)
//! Verifies RFC-003 specification across Local and Non-Local LLM architectures.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tagisan::{
    build_standard_harmony_pipeline, extract_markdown_code_blocks, parse_provider_and_model,
    resolve_harmony_models, AgentShieldSecurityGate, AssemblyRoles, BoxEventStream,
    CollaborationStrategy, CompletionRequest, CompletionResponse, EngineContext,
    ExtractedCodeBlock, GateResult, HarmonyStage, LlmProvider, ProviderCapabilities,
    RoleArtifact, RoleModelOverrides, StrategyInput, StructuredHarmonyPipeline,
    StructuredHarmonyStrategy, SwarmBlackboard, SyntaxValidationGate, TagisanError, TokenUsage,
    ValidationGate,
};

// =========================================================================
// MOCK PROVIDER FOR DETERMINISTIC TESTING
// =========================================================================
struct MockLlmProvider {
    id: &'static str,
    call_count: Arc<AtomicUsize>,
    canned_responses: Vec<String>,
}

impl MockLlmProvider {
    fn new(id: &'static str, canned_responses: Vec<String>) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses,
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> tagisan::Result<CompletionResponse> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp_text = if idx < self.canned_responses.len() {
            self.canned_responses[idx].clone()
        } else {
            self.canned_responses.last().cloned().unwrap_or_else(|| "Default mock response".to_string())
        };

        let prompt_len = req.messages.first().map(|m| m.extract_text().len()).unwrap_or(50);

        Ok(CompletionResponse {
            id: format!("mock-{}", idx),
            provider: self.id.to_string(),
            model: req.model.clone(),
            message: tagisan::Message::assistant(resp_text),
            finish_reason: tagisan::FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: (prompt_len as u32 / 4).max(1),
                completion_tokens: 50,
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: Some(0.0001),
            },
            latency: Duration::from_millis(15),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> tagisan::Result<BoxEventStream> {
        Err(TagisanError::Execution("Stream not implemented for mock".to_string()))
    }
}

// =========================================================================
// TEST 1: Blackboard Thread-Safety & Concurrent Appends
// =========================================================================
#[test]
fn test_01_blackboard_concurrency_and_assembly() {
    let blackboard = Arc::new(SwarmBlackboard::new("Concurrent Rate Limiter Task"));
    let mut handles = Vec::new();

    for thread_idx in 0..16 {
        let bb = blackboard.clone();
        handles.push(std::thread::spawn(move || {
            let artifact = RoleArtifact {
                role_id: format!("role_{thread_idx}"),
                role_title: format!("Worker Role {thread_idx}"),
                provider: "mock".to_string(),
                model: "model_v1".to_string(),
                raw_output: format!("Output from thread {thread_idx}"),
                code_blocks: vec![ExtractedCodeBlock {
                    language: "rust".to_string(),
                    code: format!("pub fn worker_{thread_idx}() -> u32 {{ {thread_idx} }}"),
                }],
                latency_secs: 0.1,
                tokens_used: 100,
            };
            bb.append_artifact(artifact);
            bb.set_metadata(format!("key_{thread_idx}"), format!("val_{thread_idx}"));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(blackboard.len(), 16);
    assert!(!blackboard.is_empty());

    let rust_code = blackboard.get_latest_code_by_language("rust").expect("Must find rust code");
    assert!(rust_code.contains("pub fn worker_"));

    let assembled = blackboard.assemble_complete_project();
    assert!(assembled.contains("PROJECT OBJECTIVE: Concurrent Rate Limiter Task"));
    assert!(assembled.contains("STAGE: ROLE_"));
}

// =========================================================================
// TEST 2: Syntax & AgentShield Validation Gates
// =========================================================================
#[test]
fn test_02_validation_gates_behavior() {
    let syntax_gate = SyntaxValidationGate::for_rust();
    let security_gate = AgentShieldSecurityGate::new();

    // 1. Missing code block -> Reject with critique
    let empty_artifact = RoleArtifact {
        role_id: "architect".to_string(),
        role_title: "Architect".to_string(),
        provider: "mock".to_string(),
        model: "m1".to_string(),
        raw_output: "Here is what I think we should do... (no code blocks)".to_string(),
        code_blocks: vec![],
        latency_secs: 0.1,
        tokens_used: 20,
    };
    match syntax_gate.validate(&empty_artifact) {
        GateResult::RetryWithCritique { critique } => {
            assert!(critique.contains("No code block was detected"));
        }
        _ => panic!("Expected RetryWithCritique for missing code blocks"),
    }

    // 2. Unbalanced braces -> Reject with critique
    let unbalanced_artifact = RoleArtifact {
        role_id: "implementer".to_string(),
        role_title: "Implementer".to_string(),
        provider: "mock".to_string(),
        model: "m1".to_string(),
        raw_output: "```rust\npub fn broken() {\n```".to_string(),
        code_blocks: vec![ExtractedCodeBlock {
            language: "rust".to_string(),
            code: "pub fn broken() {".to_string(),
        }],
        latency_secs: 0.1,
        tokens_used: 30,
    };
    match syntax_gate.validate(&unbalanced_artifact) {
        GateResult::RetryWithCritique { critique } => {
            assert!(critique.contains("Unbalanced curly braces"));
        }
        _ => panic!("Expected RetryWithCritique for unbalanced braces"),
    }

    // 3. Clean Rust code -> Pass
    let valid_artifact = RoleArtifact {
        role_id: "qa".to_string(),
        role_title: "QA".to_string(),
        provider: "mock".to_string(),
        model: "m1".to_string(),
        raw_output: "```rust\n#[test]\nfn test_valid() { assert_eq!(1, 1); }\n```".to_string(),
        code_blocks: vec![ExtractedCodeBlock {
            language: "rust".to_string(),
            code: "#[test]\nfn test_valid() { assert_eq!(1, 1); }".to_string(),
        }],
        latency_secs: 0.1,
        tokens_used: 40,
    };
    assert_eq!(syntax_gate.validate(&valid_artifact), GateResult::Pass);

    // 4. Malicious AgentShield threat -> Block & Critique
    let malicious_artifact = RoleArtifact {
        role_id: "implementer".to_string(),
        role_title: "Implementer".to_string(),
        provider: "mock".to_string(),
        model: "m1".to_string(),
        raw_output: "```rust\nfn hack() { std::process::Command::new(\"rm\").arg(\"-rf\").arg(\"/\").output(); }\n```".to_string(),
        code_blocks: vec![ExtractedCodeBlock {
            language: "rust".to_string(),
            code: "fn hack() { std::process::Command::new(\"rm\").arg(\"-rf\").arg(\"/\").output(); }".to_string(),
        }],
        latency_secs: 0.1,
        tokens_used: 50,
    };
    match security_gate.validate(&malicious_artifact) {
        GateResult::RetryWithCritique { critique } => {
            assert!(critique.contains("AgentShield Security Rule Triggered"));
        }
        _ => panic!("Expected AgentShield to block destructive rm -rf command"),
    }
}

// =========================================================================
// TEST 3: Markdown Code Block Extractor
// =========================================================================
#[test]
fn test_03_markdown_code_block_extraction() {
    let input = r#"
Here is an overview:
```rust
pub struct TokenBucket {
    capacity: u64,
}
```
And here is a configuration file:
```json
{
    "capacity": 100
}
```
"#;

    let blocks = extract_markdown_code_blocks(input);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].language, "rust");
    assert!(blocks[0].code.contains("pub struct TokenBucket"));
    assert_eq!(blocks[1].language, "json");
    assert!(blocks[1].code.contains("\"capacity\": 100"));
}

// =========================================================================
// TEST 4: Model Resolution & Override Parsing (Local & Non-Local)
// =========================================================================
#[test]
fn test_04_model_parsing_and_resolution() {
    let (p1, m1) = parse_provider_and_model("anthropic:claude-3-5-sonnet-20241022", "default");
    assert_eq!(p1, "anthropic");
    assert_eq!(m1, "claude-3-5-sonnet-20241022");

    let (p2, m2) = parse_provider_and_model("qwen2.5:0.5b", "ollama");
    assert_eq!(p2, "ollama");
    assert_eq!(m2, "qwen2.5:0.5b");

    let mut ctx = EngineContext::new(10.0);
    let mock_cloud = Arc::new(MockLlmProvider::new("anthropic", vec!["resp".to_string()]));
    ctx.register_provider(mock_cloud);

    let overrides = RoleModelOverrides {
        architect: Some("openai:gpt-4o".to_string()),
        implementer: None,
        qa: Some("ollama:qwen2.5:0.5b".to_string()),
        doc: None,
    };

    let (arch, imp, qa, doc) = resolve_harmony_models(&ctx, &overrides);
    assert_eq!(arch, ("openai".to_string(), "gpt-4o".to_string()));
    assert_eq!(imp, ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string()));
    assert_eq!(qa, ("ollama".to_string(), "qwen2.5:0.5b".to_string()));
    assert_eq!(doc, ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string()));
}

// =========================================================================
// TEST 5: Complete Assembly Line Pipeline Execution with Mock & Retries
// =========================================================================
#[tokio::test]
async fn test_05_assembly_line_execution_and_retries() {
    let mut ctx = EngineContext::new(10.0);

    // Mock architect: First attempt fails (no code block), second attempt succeeds
    let arch_responses = vec![
        "Here are my thoughts without code fences...".to_string(),
        "```rust\npub struct RateLimiter { capacity: u32 }\n```".to_string(),
    ];
    let mock_arch = Arc::new(MockLlmProvider::new("mock_arch", arch_responses));
    ctx.register_provider(mock_arch);

    // Mock implementer
    let mock_imp = Arc::new(MockLlmProvider::new(
        "mock_imp",
        vec!["```rust\nimpl RateLimiter { pub fn check(&self) -> bool { true } }\n```".to_string()],
    ));
    ctx.register_provider(mock_imp);

    // Mock QA
    let mock_qa = Arc::new(MockLlmProvider::new(
        "mock_qa",
        vec!["```rust\n#[test]\nfn test_check() { let rl = RateLimiter { capacity: 10 }; assert!(rl.check()); }\n```".to_string()],
    ));
    ctx.register_provider(mock_qa);

    // Mock Doc
    let mock_doc = Arc::new(MockLlmProvider::new(
        "mock_doc",
        vec!["# Rate Limiter\n\nUsage documentation and examples.".to_string()],
    ));
    ctx.register_provider(mock_doc);

    // Build pipeline
    let mut pipeline = StructuredHarmonyPipeline::new("Build an in-memory RateLimiter in Rust")
        .with_max_retries(2);

    pipeline = pipeline.add_stage(
        HarmonyStage::new(AssemblyRoles::architect("mock_arch", "m1"))
            .with_gate(Box::new(SyntaxValidationGate::for_rust()))
            .with_gate(Box::new(AgentShieldSecurityGate::new())),
    );
    pipeline = pipeline.add_stage(
        HarmonyStage::new(AssemblyRoles::implementer("mock_imp", "m2"))
            .with_gate(Box::new(SyntaxValidationGate::for_rust()))
            .with_gate(Box::new(AgentShieldSecurityGate::new())),
    );
    pipeline = pipeline.add_stage(
        HarmonyStage::new(AssemblyRoles::qa("mock_qa", "m3"))
            .with_gate(Box::new(SyntaxValidationGate::for_rust())),
    );
    pipeline = pipeline.add_stage(
        HarmonyStage::new(AssemblyRoles::documentation("mock_doc", "m4"))
            .with_gate(Box::new(SyntaxValidationGate::permissive())),
    );

    let result = pipeline.execute(&ctx).await.expect("Pipeline execution must succeed");

    assert_eq!(result.artifacts.len(), 4, "Must have 4 completed stage artifacts");
    assert!(result.complete_project.contains("pub struct RateLimiter"));
    assert!(result.complete_project.contains("impl RateLimiter"));
    assert!(result.complete_project.contains("#[test]"));
    assert!(result.complete_project.contains("# Rate Limiter"));
}

// =========================================================================
// TEST 6: Adversarial Audit Phase Verification
// =========================================================================
#[tokio::test]
async fn test_06_adversarial_audit_phase() {
    let mut ctx = EngineContext::new(10.0);

    let mock_prov = Arc::new(MockLlmProvider::new(
        "mock",
        vec![
            "```rust\npub struct Lock;\n```".to_string(),
            "VERDICT: APPROVED. No deadlock conditions found in Lock struct.".to_string(),
        ],
    ));
    ctx.register_provider(mock_prov);

    let pipeline = StructuredHarmonyPipeline::new("Design Lock")
        .add_stage(
            HarmonyStage::new(AssemblyRoles::architect("mock", "m1"))
                .with_gate(Box::new(SyntaxValidationGate::for_rust())),
        )
        .with_audit(("mock".to_string(), "m1".to_string()));

    let result = pipeline.execute(&ctx).await.expect("Execution must succeed");
    assert!(result.audit_verdict.is_some());
    let verdict = result.audit_verdict.unwrap();
    assert!(verdict.contains("VERDICT: APPROVED"));
}

// =========================================================================
// TEST 7: Strategy Trait Compatibility (CollaborationStrategy)
// =========================================================================
#[tokio::test]
async fn test_07_collaboration_strategy_trait() {
    let mut ctx = EngineContext::new(10.0);
    let mock = Arc::new(MockLlmProvider::new("mock", vec!["```rust\npub fn a() {}\n```".to_string()]));
    ctx.register_provider(mock);

    let overrides = RoleModelOverrides {
        architect: Some("mock:m".to_string()),
        implementer: Some("mock:m".to_string()),
        qa: Some("mock:m".to_string()),
        doc: Some("mock:m".to_string()),
    };

    let strategy = StructuredHarmonyStrategy::new(overrides, false);
    assert_eq!(strategy.name(), "Structured Role-Based Harmony Swarm (Bayanihan)");

    let input = StrategyInput {
        prompt: "Build simple function".to_string(),
        system_instruction: None,
    };

    let output = strategy.execute(input, &ctx).await.expect("Strategy must execute");
    assert_eq!(output.intermediate_steps.len(), 4);
    assert!(output.final_answer.contains("PROJECT OBJECTIVE: Build simple function"));
}

// =========================================================================
// TEST 8: Live Local Ollama Swarm Assembly Line (if daemon online)
// =========================================================================
#[tokio::test]
async fn test_08_live_local_ollama_assembly_line() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1500))
        .build()
        .unwrap();

    let is_ollama_live = client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    if !is_ollama_live {
        println!("Ollama daemon not reachable at http://127.0.0.1:11434. Skipping live inference test.");
        return;
    }

    println!("\n=== LIVE TEST: Local Ollama Structured Harmony Swarm ===");
    let mut ctx = EngineContext::new(10.0);
    ctx.register_provider(Arc::new(tagisan::OllamaProvider::default_local()));

    let overrides = RoleModelOverrides {
        architect: Some("ollama:qwen2.5:0.5b".to_string()),
        implementer: Some("ollama:dolphin-phi:latest".to_string()),
        qa: Some("ollama:qwen2.5:0.5b".to_string()),
        doc: Some("ollama:dolphin-phi:latest".to_string()),
    };

    let pipeline = build_standard_harmony_pipeline(
        "Build a simple in-memory Counter struct in Rust with increment and value methods",
        &ctx,
        &overrides,
        false,
    );

    let result = pipeline.execute(&ctx).await.expect("Live local Ollama pipeline must succeed");

    println!("[✓] Completed in {:.2}s", result.total_latency.as_secs_f64());
    assert_eq!(result.artifacts.len(), 4, "Must have completed all 4 stages");

    for a in &result.artifacts {
        println!("  - Stage: {} [{}/{}] -> ({} tokens, {:.2}s)", a.role_id, a.provider, a.model, a.tokens_used, a.latency_secs);
        assert!(!a.raw_output.trim().is_empty());
    }

    let assembled = result.complete_project;
    println!("\n--- Assembled Code Preview ---\n{}", &assembled[..assembled.len().min(400)]);
    assert!(assembled.contains("Counter") || assembled.contains("counter") || assembled.contains("struct"));
}
