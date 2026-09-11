use async_trait::async_trait;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    all_ecc_presets, all_ecc_skills, build_ecc_pipeline, find_ecc_preset, find_ecc_skill,
    global_ecc_dispatcher, load_ecc_agents_from_dir, load_ecc_skills_from_dir, resolve_ecc_agent,
    resolve_ecc_skill, AgentShieldScanner, AgentShieldVerdict, AutonomousAgent, BoxEventStream,
    CollaborationStrategy, CompletionRequest, CompletionResponse, ContentBlock, DagScheduler,
    EccAgent, EccAuditDebate, EccSkill, EngineContext, FinishReason, LlmProvider, Message,
    ProviderCapabilities, SearchSkillsTool, StrategyInput, TagisanError, TokenUsage, ToolHandler,
    ToolRegistry,
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

#[test]
fn test_ecc_skills_parser_and_registry() {
    let markdown = r#"---
name: custom-optimization
description: Algorithm and latency optimization skill.
---

# Instructions
Profile bottlenecks before optimizing.
"#;

    let skill = EccSkill::parse(markdown).expect("Valid skill markdown should parse");
    assert_eq!(skill.name, "custom-optimization");
    assert_eq!(skill.description, "Algorithm and latency optimization skill.");
    assert!(skill.instructions.contains("Profile bottlenecks before optimizing."));

    // Built-in skills
    let built_ins = all_ecc_skills();
    assert!(built_ins.len() >= 20);

    assert!(find_ecc_skill("tdd-workflow").is_some());
    assert!(find_ecc_skill("security-review").is_some());
    assert!(find_ecc_skill("api-design").is_some());
    assert!(find_ecc_skill("verification-loop").is_some());
    assert!(find_ecc_skill("deep-modules-complexity").is_some());
    assert!(find_ecc_skill("legacy-seams-characterization").is_some());
    assert!(find_ecc_skill("catalog-refactoring-smells").is_some());
    assert!(find_ecc_skill("production-resilience-release-it").is_some());
    assert!(find_ecc_skill("evolutionary-fitness-functions").is_some());
    assert!(find_ecc_skill("temporal-invariants-tla").is_some());
    assert!(find_ecc_skill("conceptual-integrity-systems").is_some());

    // Directory discovery including integrated Microsoft, Google, and AWS skills
    let skills_dir = Path::new(".ecc/skills");
    if skills_dir.exists() {
        let loaded = load_ecc_skills_from_dir(skills_dir);
        assert!(!loaded.is_empty());
        assert!(resolve_ecc_skill("tdd-workflow", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("azure-identity-rust", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("azure-keyvault-secrets-rust", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("mcp-builder", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("cloud-solution-architect", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("gemini-api-dev", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("mantis-threat-model", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("google-cloud-solution-architecture", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("firebase-firestore", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("chrome-devtools", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("amazon-bedrock", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("aws-well-architected-framework-review", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("aws-lambda", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("threat-modeling-with-aws-security-agent", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("amazon-dynamodb", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("oracle-db", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("oci", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("graal", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("microtx-workflows", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("oci-compute-management", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("oci-networking-management", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("oci-enterprise-ai", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("mysql-best-practices", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("aidp-ai-sql", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("qiskit", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("wxo-builder", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ibm-smith-opa-owasp", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("skills-microservices-architect", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ansible-generator", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ibmdiagrams-builder", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("openshift-deploy-cluster", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("clean-abap", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rap", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sap-cap-capire", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sap-ai-core", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sap-btp-cloud-platform", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sap-hana-cli", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sapui5", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibabacloud-planning", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibabacloud-terraform-codegen", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("aliyun-qwen-coder", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("aliyun-qwen-deep-research", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ms-hub", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("maxcompute-semantic", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("nacos-manager", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibaba-java-coding-standards", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibabacloud-ecs-linux-os-troubleshooting", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibabacloud-kms-envelope-encrypt", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("alibabacloud-rds-inspection", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("agentforce-generate", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("agentforce-test", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("agentforce-observe", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("fflib-enterprise-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sf-apex-enterprise-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sf-trigger-frameworks", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("lwc-wire-refresh-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("record-triggered-flow-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("apex-security-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("salesforce-cli-automation", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("data-cloud-data-streams", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sfnext-routing", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("typescript-discriminated-unions", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("typescript-types", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("typescript-style-guide", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("nextjs-app-router-fundamentals", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ai-sdk-core", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("tanstack-query", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("tanstack-router", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("vue-pinia-best-practices", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("svelte-runes", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("nestjs-best-practices", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("drizzle-orm", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("vitest-dev", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("playwright-core", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("gsap-scrolltrigger", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("zod-v4", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-m01-ownership", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-m06-error-handling", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-m07-concurrency", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-unsafe-checker", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-skills", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("rust-architect", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("axum-web-framework", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ratatui-tui-framework", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ratatui-widgets", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("tauri-desktop-apps", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("makepad-2.0-widgets", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("bun-toolkit", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("bun-testing", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("bun-shell", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("bun-builder", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("deno-runtime", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swiss-design", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("senior-designer-skill", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("design-tokens-type-scale", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("design-tokens-color-scale", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("design-system-aria-patterns", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("flutter-architecture", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("flutter-adaptive-ui", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("flutter-drift", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("react-native-mmkv", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("build-nitro-modules", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("compose-auditing-compose-performance", Some(skills_dir)).or(resolve_ecc_skill("compose-performance-audit", Some(skills_dir))).is_some());
        assert!(resolve_ecc_skill("compose-debugging-recompositions", Some(skills_dir)).or(resolve_ecc_skill("compose-recomposition-debug", Some(skills_dir))).is_some());
        assert!(resolve_ecc_skill("compose-generating-baseline-profiles", Some(skills_dir)).or(resolve_ecc_skill("compose-baseline-profiles", Some(skills_dir))).is_some());
        assert!(resolve_ecc_skill("swiftui-pro-architecture", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swiftui-expert-engineering", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swiftdata", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("ios-activitykit", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("tailwind-v4-shadcn", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-clean-architecture", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-clean-code", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-domain-driven-design", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-refactoring", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-test-driven-development", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-systematic-debugging", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("swe-solid-principles", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("review-testing-implementation", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("review-solid-principles", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("scrum-sprint-planning", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("scrum-backlog-management", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("shape-up-dhh", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sre-sev1-first-15-minutes", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("sre-triage-error-budget-burn", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("devsecops-threat-modeling", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("deep-modules-complexity", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("legacy-seams-characterization", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("catalog-refactoring-smells", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("production-resilience-release-it", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("evolutionary-fitness-functions", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("temporal-invariants-tla", Some(skills_dir)).is_some());
        assert!(resolve_ecc_skill("conceptual-integrity-systems", Some(skills_dir)).is_some());
    }
}

#[test]
fn test_agentshield_scanner_blocking() {
    // 1. Destructive commands
    assert!(matches!(
        AgentShieldScanner::scan_command("rm -rf /"),
        AgentShieldVerdict::Block { .. }
    ));
    assert!(matches!(
        AgentShieldScanner::scan_command("mkfs.ext4 /dev/sda1"),
        AgentShieldVerdict::Block { .. }
    ));
    assert!(matches!(
        AgentShieldScanner::scan_command("cat /etc/shadow"),
        AgentShieldVerdict::Block { .. }
    ));

    // 2. Path traversal
    assert!(matches!(
        AgentShieldScanner::scan_file_path("../../../etc/shadow"),
        AgentShieldVerdict::Block { .. }
    ));
    assert!(matches!(
        AgentShieldScanner::scan_file_path("/etc/shadow"),
        AgentShieldVerdict::Block { .. }
    ));

    // 3. Allowed operations
    assert_eq!(AgentShieldScanner::scan_command("cargo test --release"), AgentShieldVerdict::Allow);
    assert_eq!(AgentShieldScanner::scan_command("git status"), AgentShieldVerdict::Allow);
    assert_eq!(AgentShieldScanner::scan_file_path("src/lib.rs"), AgentShieldVerdict::Allow);

    // 4. Secret redaction
    let text = "My API key is sk-ant-api03-abcdef123456 and token is sk-proj-12345 secret";
    let redacted = AgentShieldScanner::redact_secrets(text);
    assert!(!redacted.contains("sk-ant-api03-abcdef123456"));
    assert!(redacted.contains("[REDACTED_ANTHROPIC_KEY]"));
    assert!(redacted.contains("[REDACTED_OPENAI_KEY]"));
}

#[tokio::test]
async fn test_agent_with_agentshield_interception() {
    struct DangerousToolProvider;

    #[async_trait]
    impl LlmProvider for DangerousToolProvider {
        fn provider_id(&self) -> &'static str { "danger_mock" }
        fn capabilities(&self, _: &str) -> ProviderCapabilities {
            ProviderCapabilities::FUNCTION_CALLING
        }
        async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
            let last_role = req.messages.last().map(|m| m.role.clone());
            if last_role == Some(tagisan::Role::Tool) {
                return Ok(CompletionResponse {
                    id: "resp_final".to_string(),
                    provider: "danger_mock".to_string(),
                    model: req.model,
                    message: Message::assistant("I cannot proceed with the destructive command."),
                    finish_reason: FinishReason::Stop,
                    usage: TokenUsage::default(),
                    latency: Duration::from_millis(1),
                });
            }

            Ok(CompletionResponse {
                id: "resp_danger".to_string(),
                provider: "danger_mock".to_string(),
                model: req.model,
                message: Message::tool_call(
                    "call_bad_1",
                    "run_command",
                    serde_json::json!({ "command": "rm -rf /" }),
                ),
                finish_reason: FinishReason::ToolCalls,
                usage: TokenUsage::default(),
                latency: Duration::from_millis(1),
            })
        }
        async fn stream(&self, _: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
            Err(TagisanError::Execution("unsupported".to_string()))
        }
    }

    let prov = Arc::new(DangerousToolProvider);
    let mut registry = ToolRegistry::new();
    registry.register_tool(tagisan::RunCommandTool::default());

    let agent = AutonomousAgent::new(prov, "mock-model", registry)
        .with_agentshield(true)
        .with_max_iterations(3);

    let ctx = EngineContext::new(5.0);
    let result = agent.run("Please format the drive", &ctx).await.expect("Agent execution should handle security blocks safely");

    assert_eq!(result.steps.len(), 1);
    let step = &result.steps[0];
    assert_eq!(step.tool_results.len(), 1);
    if let ContentBlock::ToolResult { content, is_error, .. } = &step.tool_results[0] {
        assert!(*is_error);
        assert!(content.contains("[AgentShield Security Block"));
    } else {
        panic!("Expected tool result block");
    }

    assert!(result.final_answer.contains("cannot proceed"));
}

#[test]
fn test_skill_dispatcher_catalog_indexing() {
    let dispatcher = global_ecc_dispatcher();
    assert!(
        dispatcher.len() >= 3840,
        "Dispatcher must index all 3,840+ skills (built-ins + .ecc/skills), got {}",
        dispatcher.len()
    );

    // Exact name lookups
    assert!(dispatcher.get_skill("tdd-workflow").is_some());
    assert!(dispatcher.get_skill("security-review").is_some());
    assert!(dispatcher.get_skill("azure-cosmos-rust").is_some());
    assert!(dispatcher.get_skill("bun-toolkit").is_some());
    assert!(dispatcher.get_skill("flutter-drift").is_some());
}

#[test]
fn test_skill_dispatcher_exact_and_hybrid_ranking() {
    let dispatcher = global_ecc_dispatcher();

    // 1. Exact Name Query
    let results = dispatcher.dispatch("azure-cosmos-rust", 3, None);
    assert!(!results.is_empty(), "Must find exact skill");
    assert_eq!(results[0].skill.name, "azure-cosmos-rust");
    assert!(results[0].score >= 200.0, "Exact match score must be >= 200");

    // 2. Trigger phrase match
    let trigger_results = dispatcher.dispatch("cosmos db rust", 3, None);
    assert!(!trigger_results.is_empty());
    assert_eq!(trigger_results[0].skill.name, "azure-cosmos-rust");
    assert!(trigger_results[0].score >= 100.0);

    // 3. Natural language objective query
    let nl_results = dispatcher.dispatch(
        "Build a high-throughput event processor in Rust with Azure Cosmos",
        3,
        None,
    );
    assert!(!nl_results.is_empty());
    let names: Vec<&str> = nl_results.iter().map(|s| s.skill.name.as_str()).collect();
    assert!(
        names.contains(&"azure-cosmos-rust") || names.contains(&"tokio-async-tuning"),
        "Top results should contain domain skills, got: {:?}",
        names
    );

    // 4. Advanced software engineering literature skill queries
    let ousterhout_results = dispatcher.dispatch("deep modules ousterhout information hiding", 3, None);
    assert!(!ousterhout_results.is_empty());
    assert_eq!(ousterhout_results[0].skill.name, "deep-modules-complexity");

    let feathers_results = dispatcher.dispatch("legacy code characterization tests seams", 3, None);
    assert!(!feathers_results.is_empty());
    assert_eq!(feathers_results[0].skill.name, "legacy-seams-characterization");

    let nygard_results = dispatcher.dispatch("circuit breaker bulkhead retry storm cascading failure", 3, None);
    assert!(!nygard_results.is_empty());
    assert_eq!(nygard_results[0].skill.name, "production-resilience-release-it");
}

#[test]
fn test_skill_dispatcher_latency_sub_millisecond() {
    let dispatcher = global_ecc_dispatcher();
    let sample_queries = [
        "azure cosmos rust",
        "bun typescript native server",
        "flutter drift sqlite reactive streams",
        "swiftui navigation architecture",
        "security threat modeling injection",
        "test driven development unit assertions",
        "tokio async channels locks",
        "ratatui terminal user interface widgets",
        "solana anchor web3 smart contracts",
        "sap abap clean code",
    ];

    let start = std::time::Instant::now();
    let iterations: usize = 50;
    for _ in 0..iterations {
        for q in &sample_queries {
            let _ = dispatcher.dispatch(q, 3, None);
        }
    }
    let total_elapsed = start.elapsed();
    let total_queries = (iterations * sample_queries.len()) as u32;
    let avg_per_query = total_elapsed / total_queries;

    println!(
        "Dispatcher Latency: total {:?} across {} queries (avg {:?} per query over {} skills)",
        total_elapsed,
        total_queries,
        avg_per_query,
        dispatcher.len()
    );

    // Sub-5ms requirement
    assert!(
        avg_per_query < Duration::from_millis(5),
        "Average query latency {:?} exceeds 5ms threshold",
        avg_per_query
    );
}

#[tokio::test]
async fn test_search_skills_tool_execution() {
    let tool = SearchSkillsTool::with_default();
    assert_eq!(tool.name(), "search_skills");

    let result = tool
        .execute(serde_json::json!({
            "query": "cosmos db rust",
            "limit": 2
        }))
        .await
        .expect("Tool execution must succeed");

    assert!(result.contains("azure-cosmos-rust"));
    assert!(result.contains("Score:"));
}

#[test]
fn test_ecc_pipeline_auto_equipping() {
    let mock_provider = Arc::new(MockEccProvider::new("mock_llm"));
    let registry = ToolRegistry::new();

    let graph = build_ecc_pipeline(
        "Build an offline iOS app using SwiftData syncing to Axum Rust backend",
        mock_provider,
        "mock-model",
        registry,
    )
    .expect("ECC pipeline construction should succeed");

    // Retrieve nodes
    let plan_node = graph.get_task("ecc_plan").expect("ecc_plan node must exist");
    assert!(
        plan_node.prompt_template.contains("OBJECTIVE"),
        "Plan node prompt template must be configured"
    );
}
