use async_trait::async_trait;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    all_ecc_presets, build_ecc_pipeline, find_ecc_preset, load_ecc_agents_from_dir,
    resolve_ecc_agent, BoxEventStream, CompletionRequest, CompletionResponse, DagScheduler,
    EccAgent, EccAuditDebate, EngineContext, FinishReason, LlmProvider, Message,
    ProviderCapabilities, StrategyInput, TagisanError, TokenUsage, ToolRegistry,
};

/// Mock provider for testing ECC workflows deterministically
struct MockEccProvider {
    id: &'static str,
}

impl MockEccProvider {
    fn new(id: &'static str) -> Self {
        Self { id }
    }
}

#[async_trait]
impl LlmProvider for MockEccProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let user_prompt = req.messages.last().map(|m| m.extract_text()).unwrap_or_default();
        let reply = format!("Mock response from {} for prompt: {}", self.id, user_prompt);

        Ok(CompletionResponse {
            id: "mock_ecc_resp_123".to_string(),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(reply),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 25,
                completion_tokens: 50,
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: Some(0.0001),
            },
            latency: Duration::from_millis(5),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::Execution("Streaming not implemented in mock".to_string()))
    }
}

#[test]
fn test_ecc_agent_frontmatter_parser_valid() {
    let markdown = r#"---
name: sample-auditor
description: A security and resilience auditor for microservices.
tools: read_file, run_command
model: claude-3-5-sonnet-20241022
---

# System Prompt
You are an expert security auditor. Probe for vulnerabilities.
"#;

    let agent = EccAgent::parse(markdown).expect("Valid ECC markdown should parse");
    assert_eq!(agent.name, "sample-auditor");
    assert_eq!(agent.description, "A security and resilience auditor for microservices.");
    assert_eq!(agent.tools, vec!["read_file", "run_command"]);
    assert_eq!(agent.recommended_model, Some("claude-3-5-sonnet-20241022".to_string()));
    assert!(agent.system_prompt.contains("Probe for vulnerabilities."));
}

#[test]
fn test_ecc_agent_frontmatter_parser_malformed() {
    // Missing leading delimiter
    let no_delim = "name: bad\n---\nPrompt";
    assert!(EccAgent::parse(no_delim).is_err());

    // Missing closing delimiter
    let unclosed = "---\nname: bad\nPrompt without closing delimiter";
    assert!(EccAgent::parse(unclosed).is_err());

    // Missing name
    let no_name = "---\ndescription: only description\n---\nPrompt";
    assert!(EccAgent::parse(no_name).is_err());
}

#[test]
fn test_ecc_presets_registry() {
    let presets = all_ecc_presets();
    assert_eq!(presets.len(), 5);

    let names: HashSet<String> = presets.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains("architect"));
    assert!(names.contains("tdd-engineer"));
    assert!(names.contains("code-reviewer"));
    assert!(names.contains("security-auditor"));
    assert!(names.contains("build-resolver"));

    // Case-insensitive & hyphen/underscore normalization
    assert!(find_ecc_preset("architect").is_some());
    assert!(find_ecc_preset("ARCHITECT").is_some());
    assert!(find_ecc_preset("tdd_engineer").is_some());
    assert!(find_ecc_preset("tdd-engineer").is_some());
    assert!(find_ecc_preset("nonexistent_preset").is_none());
}

#[test]
fn test_ecc_directory_discovery() {
    let agents_dir = Path::new(".ecc/agents");
    if agents_dir.exists() {
        let loaded = load_ecc_agents_from_dir(agents_dir);
        assert!(!loaded.is_empty(), "Should discover sample agents in .ecc/agents");

        let names: Vec<String> = loaded.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"code-explorer".to_string()) || names.contains(&"debugger".to_string()));

        // Check resolve_ecc_agent
        let resolved = resolve_ecc_agent("code-explorer", Some(agents_dir));
        assert!(resolved.is_some());
    }
}

#[test]
fn test_ecc_5_stage_pipeline_dag_topology() {
    let mock_provider = Arc::new(MockEccProvider::new("mock_llm"));
    let registry = ToolRegistry::new();

    let graph = build_ecc_pipeline(
        "Build an atomic, lock-free token bucket rate limiter in Rust",
        mock_provider,
        "mock-model",
        registry,
    )
    .expect("Pipeline graph construction must succeed");

    // 1. Check all 6 stages exist
    let tasks = graph.task_ids();
    assert_eq!(tasks.len(), 6);
    assert!(tasks.contains(&"ecc_plan".to_string()));
    assert!(tasks.contains(&"ecc_test".to_string()));
    assert!(tasks.contains(&"ecc_implement".to_string()));
    assert!(tasks.contains(&"ecc_review".to_string()));
    assert!(tasks.contains(&"ecc_security".to_string()));
    assert!(tasks.contains(&"ecc_verify".to_string()));

    // 2. Validate topological order & dependencies
    let order = graph.validate().expect("ECC pipeline must be acyclic");

    let idx = |id: &str| order.iter().position(|t| t == id).unwrap();
    assert!(idx("ecc_plan") < idx("ecc_test"));
    assert!(idx("ecc_test") < idx("ecc_implement"));
    assert!(idx("ecc_implement") < idx("ecc_review"));
    assert!(idx("ecc_implement") < idx("ecc_security"));
    assert!(idx("ecc_review") < idx("ecc_verify"));
    assert!(idx("ecc_security") < idx("ecc_verify"));

    // 3. Verify upstream dependencies
    let plan_deps = graph.upstream_dependencies("ecc_plan").unwrap();
    assert!(plan_deps.is_empty());

    let test_deps = graph.upstream_dependencies("ecc_test").unwrap();
    assert_eq!(test_deps, vec!["ecc_plan"]);

    let impl_deps = graph.upstream_dependencies("ecc_implement").unwrap();
    assert_eq!(impl_deps, vec!["ecc_test"]);

    let review_deps = graph.upstream_dependencies("ecc_review").unwrap();
    assert_eq!(review_deps, vec!["ecc_implement"]);

    let sec_deps = graph.upstream_dependencies("ecc_security").unwrap();
    assert_eq!(sec_deps, vec!["ecc_implement"]);

    let verify_deps = graph.upstream_dependencies("ecc_verify").unwrap();
    assert_eq!(verify_deps.len(), 2);
    assert!(verify_deps.contains(&"ecc_review".to_string()));
    assert!(verify_deps.contains(&"ecc_security".to_string()));
}

#[tokio::test]
async fn test_ecc_5_stage_pipeline_execution() {
    let mock_provider = Arc::new(MockEccProvider::new("mock_llm"));
    let mut ctx = EngineContext::new(10.0);
    ctx.register_provider(mock_provider.clone());

    let registry = ToolRegistry::new();
    let mut pipeline = build_ecc_pipeline(
        "Design and implement a zero-downtime schema migrator in Rust",
        mock_provider,
        "mock-model",
        registry,
    )
    .unwrap();

    let scheduler = DagScheduler::new().with_id("test_ecc_run");
    let result = scheduler.run(&mut pipeline, &ctx).await.expect("ECC pipeline execution should succeed");

    assert_eq!(result.completed_tasks, 6);
    assert_eq!(result.failed_tasks, 0);
    assert!(result.final_output.is_some());
    assert!(result.total_usage.prompt_tokens > 0);
}

#[tokio::test]
async fn test_ecc_audit_debate_execution() {
    let mock_provider = Arc::new(MockEccProvider::new("mock_llm"));
    let mut ctx = EngineContext::new(10.0);
    ctx.register_provider(mock_provider);

    let audit = EccAuditDebate::new(
        ("mock_llm".to_string(), "claude-3-5-sonnet".to_string()),
        ("mock_llm".to_string(), "deepseek-reasoner".to_string()),
        ("mock_llm".to_string(), "gemini-1.5-pro".to_string()),
    );

    let input = StrategyInput {
        prompt: "Should high-concurrency payment ledger balance updates use optimistic concurrency control?".to_string(),
        system_instruction: None,
    };

    let output = audit.execute(input, &ctx).await.expect("ECC audit debate should succeed");
    assert_eq!(output.intermediate_steps.len(), 3);
    assert_eq!(output.intermediate_steps[0].step_name, "Round 1: Architectural Proposal (ECC Architect)");
    assert_eq!(output.intermediate_steps[1].step_name, "Round 2: Threat & Vulnerability Audit (ECC Security Auditor)");
    assert_eq!(output.intermediate_steps[2].step_name, "Round 3: Lakandiwa Synthesis & Hardened Verdict");
    assert!(!output.final_answer.is_empty());
}
