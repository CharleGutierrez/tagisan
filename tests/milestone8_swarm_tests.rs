use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tagisan::{
    AgentReview, AutonomousAgent, BoxEventStream, CompletionRequest, CompletionResponse,
    ContentBlock, EngineContext, FinishReason, InteractiveRepl, LlmProvider, Message,
    ProviderCapabilities, ReplCommand, SessionRecord, SessionStore, SwarmCoordinator, SwarmMember,
    TagisanError, TeamConsensusEngine, TokenUsage, ToolRegistry, VotingRule,
};

// =========================================================================
// Mock Scripted Provider for Swarm Testing
// =========================================================================

struct MockSwarmProvider {
    id: &'static str,
    responses: tokio::sync::Mutex<Vec<Result<CompletionResponse, TagisanError>>>,
    call_count: Arc<AtomicUsize>,
}

impl MockSwarmProvider {
    fn new(id: &'static str, script: Vec<Result<CompletionResponse, TagisanError>>) -> Self {
        Self {
            id,
            responses: tokio::sync::Mutex::new(script),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl LlmProvider for MockSwarmProvider {
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
                message: Message::assistant("Default mock response"),
                finish_reason: FinishReason::Stop,
                usage: TokenUsage {
                    prompt_tokens: 15,
                    completion_tokens: 25,
                    ..Default::default()
                },
                latency: Duration::from_millis(5),
            })
        } else {
            script.remove(0)
        }
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::Execution(format!("Streaming unsupported for {}", self.id)))
    }
}

// =========================================================================
// 1. Swarm Coordinator Lead Delegation Test
// =========================================================================

#[tokio::test]
async fn test_swarm_coordinator_lead_delegation() {
    let ctx = EngineContext::new(10.0);

    // Lead Agent Script:
    // Turn 1: Invokes tool `delegate_task` with target_agent: "security_specialist"
    // Turn 2: Synthesizes final answer after receiving observation
    let lead_resp_1 = CompletionResponse {
        id: "lead_1".to_string(),
        provider: "mock".to_string(),
        model: "mock-lead".to_string(),
        message: Message {
            role: tagisan::Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: "call_delegate_1".to_string(),
                name: "delegate_task".to_string(),
                arguments: json!({
                    "target_agent": "security_specialist",
                    "task": "Audit authentication token handling for timing attacks"
                }),
            }],
            name: None,
            metadata: HashMap::new(),
        },
        finish_reason: FinishReason::ToolCalls,
        usage: TokenUsage { prompt_tokens: 30, completion_tokens: 20, ..Default::default() },
        latency: Duration::from_millis(10),
    };

    let lead_resp_2 = CompletionResponse {
        id: "lead_2".to_string(),
        provider: "mock".to_string(),
        model: "mock-lead".to_string(),
        message: Message::assistant(
            "Based on the security specialist's audit, constant-time comparison is recommended to prevent timing attacks."
        ),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage { prompt_tokens: 50, completion_tokens: 30, ..Default::default() },
        latency: Duration::from_millis(10),
    };

    let lead_provider = Arc::new(MockSwarmProvider::new("lead_prov", vec![Ok(lead_resp_1), Ok(lead_resp_2)]));

    // Specialist Agent Script:
    let specialist_resp = CompletionResponse {
        id: "spec_1".to_string(),
        provider: "mock".to_string(),
        model: "mock-sec".to_string(),
        message: Message::assistant(
            "Vulnerability Identified: `==` string equality leaks timing info. Recommend `subtle::ConstantTimeEq`."
        ),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage { prompt_tokens: 25, completion_tokens: 35, ..Default::default() },
        latency: Duration::from_millis(10),
    };
    let specialist_provider = Arc::new(MockSwarmProvider::new("sec_prov", vec![Ok(specialist_resp)]));

    let lead_member = SwarmMember::new("architect", "Lead System Architect", "mock-lead", lead_provider)
        .with_system_prompt("You are the lead architect.");

    let specialist_member = SwarmMember::new(
        "security_specialist",
        "Cryptographic Security Auditor",
        "mock-sec",
        specialist_provider,
    )
    .with_system_prompt("You are a specialized security auditor.");

    let coordinator = SwarmCoordinator::new()
        .add_member(lead_member)
        .add_member(specialist_member)
        .with_lead("architect");

    assert_eq!(coordinator.lead_name(), Some("architect"));
    assert_eq!(coordinator.members().len(), 2);

    let res = coordinator
        .run_lead("Design and audit our authentication service", &ctx)
        .await
        .expect("Lead orchestration should succeed");

    assert!(res.final_answer.contains("constant-time comparison"));
    assert_eq!(res.iterations, 2);
    assert_eq!(res.steps.len(), 1); // Turn 1 had tool call
}

// =========================================================================
// 2. Swarm Coordinator Sequential Pipeline Test
// =========================================================================

#[tokio::test]
async fn test_swarm_coordinator_sequential_pipeline() {
    let ctx = EngineContext::new(10.0);

    let prov1 = Arc::new(MockSwarmProvider::new("p1", vec![Ok(CompletionResponse {
        id: "resp1".to_string(),
        provider: "p1".to_string(),
        model: "m1".to_string(),
        message: Message::assistant("Stage 1 Plan: Created architecture specification."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(5),
    })]));

    let prov2 = Arc::new(MockSwarmProvider::new("p2", vec![Ok(CompletionResponse {
        id: "resp2".to_string(),
        provider: "p2".to_string(),
        model: "m2".to_string(),
        message: Message::assistant("Stage 2 Code: Implemented Rust structs and traits based on plan."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(5),
    })]));

    let prov3 = Arc::new(MockSwarmProvider::new("p3", vec![Ok(CompletionResponse {
        id: "resp3".to_string(),
        provider: "p3".to_string(),
        model: "m3".to_string(),
        message: Message::assistant("Stage 3 Tests: All unit and fuzz tests passed cleanly."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(5),
    })]));

    let m1 = SwarmMember::new("planner", "Planner", "m1", prov1);
    let m2 = SwarmMember::new("coder", "Coder", "m2", prov2);
    let m3 = SwarmMember::new("tester", "Tester", "m3", prov3);

    let coordinator = SwarmCoordinator::new()
        .add_member(m1)
        .add_member(m2)
        .add_member(m3);

    let stage_names = ["planner", "coder", "tester"];
    let pipeline_res = coordinator
        .execute_pipeline(&stage_names, "Build a high-performance LRU cache", &ctx)
        .await
        .expect("Pipeline execution should succeed");

    assert_eq!(pipeline_res.stages.len(), 3);
    assert_eq!(pipeline_res.stages[0].agent_name, "planner");
    assert_eq!(pipeline_res.stages[1].agent_name, "coder");
    assert_eq!(pipeline_res.stages[2].agent_name, "tester");
    assert!(pipeline_res.final_answer.contains("tests passed cleanly"));
}

// =========================================================================
// 3. Swarm Coordinator Concurrent Broadcast Test
// =========================================================================

#[tokio::test]
async fn test_swarm_coordinator_concurrent_broadcast() {
    let ctx = EngineContext::new(10.0);

    let prov_a = Arc::new(MockSwarmProvider::new("pa", vec![Ok(CompletionResponse {
        id: "ra".to_string(),
        provider: "pa".to_string(),
        model: "ma".to_string(),
        message: Message::assistant("Reviewer A: No deadlock risks observed."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(15),
    })]));

    let prov_b = Arc::new(MockSwarmProvider::new("pb", vec![Ok(CompletionResponse {
        id: "rb".to_string(),
        provider: "pb".to_string(),
        model: "mb".to_string(),
        message: Message::assistant("Reviewer B: Suggest bounding memory growth."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage::default(),
        latency: Duration::from_millis(10),
    })]));

    let coordinator = SwarmCoordinator::new()
        .add_member(SwarmMember::new("rev_a", "Reviewer A", "ma", prov_a))
        .add_member(SwarmMember::new("rev_b", "Reviewer B", "mb", prov_b));

    let results = coordinator
        .execute_broadcast("Analyze memory safety in concurrency loop", &ctx)
        .await
        .expect("Broadcast execution should succeed");

    assert_eq!(results.len(), 2);
    assert!(results.contains_key("rev_a"));
    assert!(results.contains_key("rev_b"));
    assert!(results["rev_a"].final_answer.contains("No deadlock risks"));
    assert!(results["rev_b"].final_answer.contains("bounding memory growth"));
}

// =========================================================================
// 4. Team Consensus Engine Voting Rules Test
// =========================================================================

#[test]
fn test_team_consensus_majority_and_unanimous() {
    let engine = TeamConsensusEngine::new();

    let r1 = AgentReview::new("architect", true, 9)
        .with_score("correctness", 9)
        .with_score("security", 8);

    let r2 = AgentReview::new("security_auditor", true, 8)
        .with_score("correctness", 8)
        .with_score("security", 9);

    let r3 = AgentReview::new("code_reviewer", false, 4)
        .with_score("correctness", 5)
        .with_score("security", 4)
        .with_feedback("Needs better error handling for network timeouts")
        .with_risk("Network timeout unhandled");

    let reviews = [r1, r2, r3];

    // Majority: 2/3 = 66.7% > 50% -> Approved
    let verdict_maj = engine.evaluate(&VotingRule::Majority, &reviews);
    assert!(verdict_maj.approved);
    assert_eq!(verdict_maj.voters_count, 3);
    assert!((verdict_maj.approval_ratio - 0.666).abs() < 0.01);
    assert_eq!(verdict_maj.action_items.len(), 2); // 1 risk + 1 feedback

    // Unanimous: 2/3 != 3/3 -> Rejected
    let verdict_unan = engine.evaluate(&VotingRule::Unanimous, &reviews);
    assert!(!verdict_unan.approved);

    // SuperMajority(0.75): 66.7% < 75% -> Rejected
    let verdict_super = engine.evaluate(&VotingRule::SuperMajority(0.75), &reviews);
    assert!(!verdict_super.approved);

    // SuperMajority(0.60): 66.7% >= 60% -> Approved
    let verdict_super_pass = engine.evaluate(&VotingRule::SuperMajority(0.60), &reviews);
    assert!(verdict_super_pass.approved);
}

#[test]
fn test_team_consensus_borda_count_and_criteria_averages() {
    let engine = TeamConsensusEngine::new();

    let r1 = AgentReview::new("voter_1", true, 8)
        .with_score("correctness", 9)
        .with_score("performance", 8)
        .with_ranked_options(vec!["OptionA".to_string(), "OptionB".to_string(), "OptionC".to_string()]);

    let r2 = AgentReview::new("voter_2", true, 7)
        .with_score("correctness", 7)
        .with_score("performance", 6)
        .with_ranked_options(vec!["OptionB".to_string(), "OptionA".to_string(), "OptionC".to_string()]);

    let r3 = AgentReview::new("voter_3", true, 9)
        .with_score("correctness", 8)
        .with_score("performance", 10)
        .with_ranked_options(vec!["OptionA".to_string(), "OptionC".to_string(), "OptionB".to_string()]);

    let reviews = [r1, r2, r3];

    let verdict = engine.evaluate(&VotingRule::WeightedBorda, &reviews);
    assert!(verdict.approved);
    assert_eq!(verdict.winning_option, Some("OptionA".to_string()));

    // Verify criterion averages
    let corr_avg = verdict.criterion_averages.get("correctness").copied().unwrap_or(0.0);
    assert!((corr_avg - 8.0).abs() < 0.01); // (9 + 7 + 8) / 3 = 8.0

    let perf_avg = verdict.criterion_averages.get("performance").copied().unwrap_or(0.0);
    assert!((perf_avg - 8.0).abs() < 0.01); // (8 + 6 + 10) / 3 = 8.0
}

// =========================================================================
// 5. Session Store Atomic Lifecycle Test
// =========================================================================

#[test]
fn test_session_store_atomic_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_session_{}", std::process::id()));
    let store = SessionStore::with_dir(&temp_dir);

    let session_id = "test-session-alpha";
    let mut record = SessionRecord::new(session_id, "Test Architecture Debate", "claude-3-5-sonnet")
        .with_persona("architect")
        .with_system_prompt("You are a system architect.");

    record.add_message(Message::user("How should we shard our database?"));
    record.add_message(Message::assistant("Consider consistent hashing based on tenant ID."));
    record.total_usage = TokenUsage {
        prompt_tokens: 120,
        completion_tokens: 85,
        ..Default::default()
    };
    record.total_cost_usd = 0.0018;

    // 1. Save session atomically
    store.save(&record).expect("Session should save successfully");

    // 2. List sessions
    let list = store.list().expect("Should list sessions");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, session_id);
    assert_eq!(list[0].message_count, 2);
    assert_eq!(list[0].model, "claude-3-5-sonnet");

    // 3. Load session
    let loaded = store.load(session_id).expect("Should load session");
    assert_eq!(loaded.id, session_id);
    assert_eq!(loaded.title, "Test Architecture Debate");
    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.total_usage.prompt_tokens, 120);
    assert_eq!(loaded.total_cost_usd, 0.0018);

    // 4. Export markdown
    let md = store.export_markdown(session_id).expect("Should export markdown transcript");
    assert!(md.contains("# Session Transcript: Test Architecture Debate"));
    assert!(md.contains("👤 **User**"));
    assert!(md.contains("🤖 **Assistant**"));
    assert!(md.contains("How should we shard our database?"));

    // 5. Delete session
    let deleted = store.delete(session_id).expect("Should delete session");
    assert!(deleted);

    let list_after = store.list().expect("Should list sessions after deletion");
    assert!(list_after.is_empty());

    // Clean up directory
    let _ = std::fs::remove_dir_all(&temp_dir);
}

// =========================================================================
// 6. Interactive REPL Command Parsing & Execution Test
// =========================================================================

#[tokio::test]
async fn test_repl_command_parsing_and_execution() {
    // 1. Test Command Parsing
    assert_eq!(InteractiveRepl::parse_command("/help"), ReplCommand::Help);
    assert_eq!(InteractiveRepl::parse_command("/h"), ReplCommand::Help);
    assert_eq!(InteractiveRepl::parse_command("/agent architect"), ReplCommand::Agent("architect".to_string()));
    assert_eq!(InteractiveRepl::parse_command("/skill tdd-workflow"), ReplCommand::Skill("tdd-workflow".to_string()));
    assert_eq!(InteractiveRepl::parse_command("/model gpt-4o"), ReplCommand::Model("gpt-4o".to_string()));
    assert_eq!(InteractiveRepl::parse_command("/tools"), ReplCommand::Tools);
    assert_eq!(InteractiveRepl::parse_command("/memory"), ReplCommand::Memory);
    assert_eq!(InteractiveRepl::parse_command("/sandbox"), ReplCommand::Sandbox);
    assert_eq!(InteractiveRepl::parse_command("/save check1"), ReplCommand::Save(Some("check1".to_string())));
    assert_eq!(InteractiveRepl::parse_command("/load check1"), ReplCommand::Load("check1".to_string()));
    assert_eq!(InteractiveRepl::parse_command("/clear"), ReplCommand::Clear);
    assert_eq!(InteractiveRepl::parse_command("/budget"), ReplCommand::Budget);
    assert_eq!(InteractiveRepl::parse_command("/history"), ReplCommand::History);
    assert_eq!(InteractiveRepl::parse_command("/exit"), ReplCommand::Exit);
    assert_eq!(
        InteractiveRepl::parse_command("What is our latency budget?"),
        ReplCommand::UserPrompt("What is our latency budget?".to_string())
    );

    // 2. Test Execution State Updates
    let ctx = EngineContext::new(5.0);
    let mock_provider = Arc::new(MockSwarmProvider::new("mock", vec![Ok(CompletionResponse {
        id: "repl_1".to_string(),
        provider: "mock".to_string(),
        model: "test-model".to_string(),
        message: Message::assistant("Latency budget is 50ms p99."),
        finish_reason: FinishReason::Stop,
        usage: TokenUsage { prompt_tokens: 10, completion_tokens: 15, ..Default::default() },
        latency: Duration::from_millis(5),
    })]));

    let agent = AutonomousAgent::new(mock_provider, "test-model", ToolRegistry::with_builtins());
    let mut repl = InteractiveRepl::new(agent, "test-repl-session", "test-model", ctx);

    // Test /agent architect switch
    let out_agent = repl.execute_command(ReplCommand::Agent("architect".to_string())).await.unwrap().unwrap();
    assert!(out_agent.contains("Switched persona to"));
    assert!(out_agent.contains("architect"));
    assert!(repl.agent.system_prompt.is_some());

    // Test /model switch
    let out_model = repl.execute_command(ReplCommand::Model("gemini-2.0-flash".to_string())).await.unwrap().unwrap();
    assert!(out_model.contains("Switched active model to"));
    assert!(out_model.contains("gemini-2.0-flash"));
    assert_eq!(repl.agent.model, "gemini-2.0-flash");

    // Test user prompt turn
    let out_prompt = repl.execute_command(ReplCommand::UserPrompt("Check latency".to_string())).await.unwrap();
    assert!(out_prompt.unwrap().contains("Latency budget is 50ms p99."));
    assert_eq!(repl.session_record.messages.len(), 2); // 1 User + 1 Assistant

    // Test /clear
    let _ = repl.execute_command(ReplCommand::Clear).await.unwrap();
    assert!(repl.session_record.messages.is_empty());
}
