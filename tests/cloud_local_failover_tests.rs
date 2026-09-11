use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tagisan::error::{Result, TagisanError};
use tagisan::providers::cascade::{CascadeEntry, CascadeProvider};
use tagisan::providers::{BoxEventStream, LlmProvider};
use tagisan::swarm::harmony::gates::SyntaxValidationGate;
use tagisan::swarm::harmony::types::{
    ExtractedCodeBlock, FailoverEvent, RoleArtifact,
};
use tagisan::swarm::harmony::{
    AssemblyRoles, HarmonyStage, StructuredHarmonyPipeline, SwarmBlackboard,
};
use tagisan::types::{
    CompletionRequest, CompletionResponse, FinishReason, Message,
    ProviderCapabilities, StreamChunk, StreamChunkDelta, TokenUsage,
};
use tagisan::{EngineContext, TokenBudgetTracker};

// =========================================================================
// MOCK PROVIDER FOR FAILOVER TESTING
// =========================================================================

#[allow(dead_code)]
#[derive(Clone)]
enum MockBehavior {
    Success(String),
    BudgetExceeded { max_budget: f64, current_spent: f64 },
    RateLimited(String, Option<Duration>),
    BadResponse(String, String),
    ContextLengthExceeded(String, usize, usize),
    Cancelled,
}

struct MockFailoverProvider {
    id: String,
    behaviors: Vec<MockBehavior>,
    call_count: Arc<AtomicUsize>,
    cost_usd: f64,
}

impl MockFailoverProvider {
    fn new(id: impl Into<String>, behaviors: Vec<MockBehavior>, cost_usd: f64) -> Self {
        Self {
            id: id.into(),
            behaviors,
            call_count: Arc::new(AtomicUsize::new(0)),
            cost_usd,
        }
    }

    fn new_successful(id: impl Into<String>, text: impl Into<String>, cost_usd: f64) -> Self {
        Self::new(id, vec![MockBehavior::Success(text.into())], cost_usd)
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl LlmProvider for MockFailoverProvider {
    fn provider_id(&self) -> &'static str {
        Box::leak(self.id.clone().into_boxed_str())
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        let behavior = if idx < self.behaviors.len() {
            &self.behaviors[idx]
        } else {
            self.behaviors.last().unwrap()
        };

        match behavior {
            MockBehavior::Success(text) => Ok(CompletionResponse {
                id: format!("{}-resp-{}", self.id, idx),
                provider: self.id.clone(),
                model: req.model.clone(),
                message: Message::assistant(text.clone()),
                finish_reason: FinishReason::Stop,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    reasoning_tokens: None,
                    cached_prompt_tokens: None,
                    estimated_cost_usd: Some(self.cost_usd),
                },
                latency: Duration::from_millis(10),
            }),
            MockBehavior::BudgetExceeded {
                max_budget,
                current_spent,
            } => Err(TagisanError::BudgetExceeded {
                max_budget: *max_budget,
                current_spent: *current_spent,
            }),
            MockBehavior::RateLimited(p, dur) => Err(TagisanError::RateLimited(p.clone(), *dur)),
            MockBehavior::BadResponse(p, msg) => {
                Err(TagisanError::BadResponse(p.clone(), msg.clone()))
            }
            MockBehavior::ContextLengthExceeded(p, cur, max) => {
                Err(TagisanError::ContextLengthExceeded(p.clone(), *cur, *max))
            }
            MockBehavior::Cancelled => Err(TagisanError::Cancelled),
        }
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        let behavior = if idx < self.behaviors.len() {
            &self.behaviors[idx]
        } else {
            self.behaviors.last().unwrap()
        };

        match behavior {
            MockBehavior::Success(text) => {
                let chunk_text = text.clone();
                let stream = async_stream::stream! {
                    yield Ok(StreamChunk::text(chunk_text));
                };
                Ok(Box::pin(stream) as BoxEventStream)
            }
            MockBehavior::BudgetExceeded {
                max_budget,
                current_spent,
            } => Err(TagisanError::BudgetExceeded {
                max_budget: *max_budget,
                current_spent: *current_spent,
            }),
            MockBehavior::RateLimited(p, dur) => Err(TagisanError::RateLimited(p.clone(), *dur)),
            MockBehavior::BadResponse(p, msg) => {
                Err(TagisanError::BadResponse(p.clone(), msg.clone()))
            }
            _ => Err(TagisanError::Execution("mock stream error".into())),
        }
    }
}

// =========================================================================
// TEST 1: Zero-Cost Operations & Budget Exhaustion Invariants (RFC-004 Task 1)
// =========================================================================
#[test]
fn test_zero_cost_operations_and_budget_exhaustion() {
    // 1. Budget of $0.05
    let tracker = TokenBudgetTracker::new(0.05);
    assert!(!tracker.is_exhausted());

    // Spend partial budget
    let spent1 = tracker.record_micro_usd(30_000).expect("spend $0.03");
    assert_eq!(spent1, 0.03);
    assert!(!tracker.is_exhausted());

    // Spend remaining budget to reach exact boundary ($0.05)
    let spent2 = tracker.record_micro_usd(20_000).expect("spend $0.02 to hit cap");
    assert_eq!(spent2, 0.05);
    assert!(tracker.is_exhausted());

    // Non-zero cloud cost must now fail with BudgetExceeded
    let cloud_err = tracker.record_micro_usd(1).unwrap_err();
    assert!(matches!(cloud_err, TagisanError::BudgetExceeded { .. }));

    // Zero-cost operations MUST succeed unconditionally even when exhausted
    let zero_micro = tracker.record_micro_usd(0);
    assert!(zero_micro.is_ok(), "record_micro_usd(0) must succeed when budget is exhausted");
    assert_eq!(zero_micro.unwrap(), 0.05);

    let zero_cost = tracker.record_cost_usd(0.0);
    assert!(zero_cost.is_ok(), "record_cost_usd(0.0) must succeed when exhausted");
    assert_eq!(zero_cost.unwrap(), 0.05);

    // Ollama zero-cost model tokens lookup
    let ollama_res = tracker.record("ollama", 5000, 5000);
    assert!(ollama_res.is_ok(), "Ollama record must succeed when budget is exhausted");
    assert_eq!(ollama_res.unwrap(), 0.05);

    let local_res = tracker.record("local", 5000, 5000);
    assert!(local_res.is_ok(), "Local record must succeed when budget is exhausted");
    assert_eq!(local_res.unwrap(), 0.05);

    let cached_local = tracker.record_with_cache("ollama-qwen", 2000, 1000, 500);
    assert!(cached_local.is_ok());

    // 2. Test $0.00 initial budget
    let zero_tracker = TokenBudgetTracker::new(0.0);
    assert!(zero_tracker.is_exhausted());
    assert!(zero_tracker.record_micro_usd(0).is_ok());
    assert!(zero_tracker.record("ollama", 100, 100).is_ok());
    assert!(zero_tracker.record("claude-3-5-sonnet", 10, 10).is_err());
}

// =========================================================================
// TEST 2: CascadeProvider can_failover & is_retryable Logic (RFC-004 Task 2)
// =========================================================================
#[test]
fn test_cascade_can_failover_logic() {
    let cloud_p1 = Arc::new(MockFailoverProvider::new_successful("anthropic", "ok", 0.01));
    let cloud_p2 = Arc::new(MockFailoverProvider::new_successful("openai", "ok", 0.01));
    let local_ollama = Arc::new(MockFailoverProvider::new_successful("ollama", "ok", 0.0));
    let local_custom = Arc::new(MockFailoverProvider::new_successful("local", "ok", 0.0));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_p1.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(cloud_p2.clone(), Some("gpt-4o".into())),
        CascadeEntry::new(local_ollama.clone(), Some("qwen2.5:0.5b".into())),
        CascadeEntry::new(local_custom.clone(), Some("dolphin-phi".into())),
    ]);

    let budget_err = TagisanError::BudgetExceeded {
        max_budget: 1.0,
        current_spent: 1.05,
    };
    let rate_err = TagisanError::RateLimited("anthropic".into(), Some(Duration::from_secs(60)));
    let bad_resp = TagisanError::BadResponse("anthropic".into(), "502 Bad Gateway".into());
    let cancel_err = TagisanError::Cancelled;

    // Index 1 is cloud_p2 ("openai") -> BudgetExceeded cannot failover to paid cloud!
    assert!(!cascade.can_failover(&budget_err, 1));

    // Index 2 is local_ollama ("ollama") -> BudgetExceeded CAN failover to zero-cost Ollama!
    assert!(cascade.can_failover(&budget_err, 2));

    // Index 3 is local_custom ("local") -> BudgetExceeded CAN failover to zero-cost Local!
    assert!(cascade.can_failover(&budget_err, 3));

    // Index 4 is Out of bounds -> Cannot failover
    assert!(!cascade.can_failover(&budget_err, 4));

    // Transient errors can failover to any provider (cloud or local)
    assert!(cascade.can_failover(&rate_err, 1));
    assert!(cascade.can_failover(&rate_err, 2));
    assert!(cascade.can_failover(&bad_resp, 1));

    // Non-retryable error (Cancelled) cannot failover anywhere
    assert!(!cascade.can_failover(&cancel_err, 1));
    assert!(!cascade.can_failover(&cancel_err, 2));

    // is_retryable backward-compatibility check
    assert!(!CascadeProvider::is_retryable(&budget_err));
    assert!(CascadeProvider::is_retryable(&rate_err));
    assert!(CascadeProvider::is_retryable(&bad_resp));
}

// =========================================================================
// TEST 3: CascadeProvider complete & stream Failover to Ollama on BudgetExceeded
// =========================================================================
#[tokio::test]
async fn test_cascade_complete_and_stream_failover_to_ollama() {
    // Chain: Primary Cloud (BudgetExceeded) -> Fallback Ollama (Success)
    let cloud_mock = Arc::new(MockFailoverProvider::new(
        "anthropic",
        vec![MockBehavior::BudgetExceeded {
            max_budget: 0.50,
            current_spent: 0.51,
        }],
        0.01,
    ));
    let ollama_mock = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "Hello from local Ollama!",
        0.0,
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_mock.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(ollama_mock.clone(), Some("qwen2.5:0.5b".into())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "Write an LRU cache");

    // 1. complete() failover
    let resp = cascade.complete(req.clone()).await.expect("Failover to Ollama must succeed");
    assert_eq!(resp.message.extract_text(), "Hello from local Ollama!");
    assert_eq!(cloud_mock.calls(), 1);
    assert_eq!(ollama_mock.calls(), 1);

    // 2. stream() failover
    let mut stream = cascade.stream(req).await.expect("Stream failover to Ollama must succeed");
    use futures::StreamExt;
    let first_chunk = stream.next().await.expect("chunk present").expect("chunk ok");
    match first_chunk.delta {
        StreamChunkDelta::Text(t) => assert_eq!(t, "Hello from local Ollama!"),
        _ => panic!("Expected text delta"),
    }
}

#[tokio::test]
async fn test_cascade_does_not_failover_budget_to_cloud() {
    // Chain: Cloud 1 (BudgetExceeded) -> Cloud 2 (Should NOT be called!)
    let cloud1 = Arc::new(MockFailoverProvider::new(
        "anthropic",
        vec![MockBehavior::BudgetExceeded {
            max_budget: 0.50,
            current_spent: 0.51,
        }],
        0.01,
    ));
    let cloud2 = Arc::new(MockFailoverProvider::new_successful("deepseek", "should not reach", 0.01));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud1.clone(), None),
        CascadeEntry::new(cloud2.clone(), None),
    ]);

    let req = CompletionRequest::new("claude", "test");
    let err = cascade.complete(req).await.unwrap_err();
    assert!(matches!(err, TagisanError::BudgetExceeded { .. }));
    assert_eq!(cloud1.calls(), 1);
    assert_eq!(cloud2.calls(), 0, "Cloud2 must never be called when budget is exceeded");
}

// =========================================================================
// TEST 4: StandardHarmonyRole with_provider_and_model & execute_stage_with_override
// =========================================================================
#[tokio::test]
async fn test_standard_harmony_role_override() {
    let ctx = EngineContext::new(5.0);
    let mock_ollama = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "```rust\npub struct LocalType;\n```",
        0.0,
    ));

    let role = AssemblyRoles::architect("anthropic", "claude-3-5-sonnet");
    assert_eq!(role.config().provider, "anthropic");
    assert_eq!(role.config().model, "claude-3-5-sonnet");

    let blackboard = SwarmBlackboard::new("Design an LRU cache");

    // Execute with override to ollama & qwen2.5:0.5b
    let artifact = role
        .execute_stage_with_override(
            &blackboard,
            mock_ollama,
            Some("qwen2.5:0.5b"),
            &ctx,
            None,
        )
        .await
        .expect("Override execution succeeds");

    assert_eq!(artifact.provider, "ollama");
    assert_eq!(artifact.model, "qwen2.5:0.5b");
    assert!(artifact.raw_output.contains("pub struct LocalType;"));
    assert_eq!(artifact.code_blocks.len(), 1);
    assert_eq!(artifact.failover_event, None);
}

// =========================================================================
// TEST 5: StructuredHarmonyPipeline Evacuation on Budget Exhaustion (RFC-004 Task 5)
// =========================================================================
#[tokio::test]
async fn test_pipeline_evacuate_on_budget_cap() {
    let mut ctx = EngineContext::new(0.05);

    // Primary cloud provider (costs money)
    let cloud_prov = Arc::new(MockFailoverProvider::new_successful(
        "anthropic",
        "```rust\npub struct CloudEngine;\n```",
        0.05,
    ));
    // Local Ollama provider ($0 cost)
    let ollama_prov = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "```rust\npub struct LocalEvacuatedEngine;\n```",
        0.0,
    ));

    ctx.register_provider(cloud_prov.clone());
    ctx.register_provider(ollama_prov.clone());

    // Exhaust budget before running stage
    ctx.budget_tracker.record_micro_usd(50_000).expect("spend $0.05");
    assert!(ctx.budget_tracker.is_exhausted());

    let role = AssemblyRoles::architect("anthropic", "claude-3-5-sonnet");
    let stage = HarmonyStage::new(role).with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("Build resilient system")
        .add_stage(stage)
        .with_evacuate_on_budget(true);

    let res = pipeline.execute(&ctx).await.expect("Pipeline must evacuate and succeed");

    assert_eq!(res.artifacts.len(), 1);
    let artifact = &res.artifacts[0];
    let fo = artifact.failover_event.as_ref().expect("Failover event must be attached");
    assert_eq!(artifact.provider, "ollama");
    assert_eq!(artifact.model, fo.evacuated_to_model);
    assert!(artifact.raw_output.contains("pub struct LocalEvacuatedEngine;"));

    // Verify FailoverEvent telemetry
    assert_eq!(fo.original_provider, "anthropic");
    assert_eq!(fo.original_model, "claude-3-5-sonnet");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.evacuated_to_model.contains("qwen") || fo.evacuated_to_model.contains("dolphin") || !fo.evacuated_to_model.is_empty());
    assert!(fo.trigger_reason.contains("Token budget reached"));
    assert!(fo.cost_at_failover_usd >= 0.05);

    // Cloud was NOT called because pre-emptive evacuation caught it
    assert_eq!(cloud_prov.calls(), 0);
    assert_eq!(ollama_prov.calls(), 1);
}

// =========================================================================
// TEST 6: StructuredHarmonyPipeline Dynamic Hot-Swap on HTTP 429 RateLimit
// =========================================================================
#[tokio::test]
async fn test_pipeline_fallback_to_local_on_ratelimit() {
    let mut ctx = EngineContext::new(5.0);

    // Cloud provider returns RateLimited (HTTP 429)
    let cloud_prov = Arc::new(MockFailoverProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited(
            "anthropic".into(),
            Some(Duration::from_secs(30)),
        )],
        0.01,
    ));
    // Local Ollama provider succeeds
    let ollama_prov = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "```rust\nimpl ResilientService { pub fn ping() -> bool { true } }\n```",
        0.0,
    ));

    ctx.register_provider(cloud_prov.clone());
    ctx.register_provider(ollama_prov.clone());

    let role = AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet");
    let stage = HarmonyStage::new(role).with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("Build resilient service")
        .add_stage(stage)
        .with_fallback_to_local(true);

    let res = pipeline.execute(&ctx).await.expect("Failover to Ollama must succeed");

    assert_eq!(res.artifacts.len(), 1);
    let artifact = &res.artifacts[0];
    let fo = artifact.failover_event.as_ref().expect("Failover event present");
    assert_eq!(artifact.provider, "ollama");
    assert_eq!(artifact.model, fo.evacuated_to_model);
    assert_eq!(fo.original_provider, "anthropic");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.evacuated_to_model.contains("qwen") || fo.evacuated_to_model.contains("dolphin") || !fo.evacuated_to_model.is_empty());
    assert!(fo.trigger_reason.contains("Rate limited (HTTP 429)"));

    // Cloud was attempted once, failed, and local was invoked
    assert_eq!(cloud_prov.calls(), 1);
    assert_eq!(ollama_prov.calls(), 1);
}

// =========================================================================
// TEST 7: StructuredHarmonyPipeline Dynamic Failover on BadResponse & Network
// =========================================================================
#[tokio::test]
async fn test_pipeline_fallback_to_local_on_badresponse() {
    let mut ctx = EngineContext::new(5.0);

    let cloud_prov = Arc::new(MockFailoverProvider::new(
        "deepseek",
        vec![MockBehavior::BadResponse(
            "deepseek".into(),
            "503 Service Unavailable".into(),
        )],
        0.01,
    ));
    let ollama_prov = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "```rust\n#[test]\nfn test_pass() { assert!(true); }\n```",
        0.0,
    ));

    ctx.register_provider(cloud_prov.clone());
    ctx.register_provider(ollama_prov.clone());

    let role = AssemblyRoles::qa("deepseek", "deepseek-chat");
    let stage = HarmonyStage::new(role).with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("Generate QA tests")
        .add_stage(stage)
        .with_fallback_to_local(true);

    let res = pipeline.execute(&ctx).await.expect("BadResponse failover must succeed");
    let fo = res.artifacts[0].failover_event.as_ref().expect("FailoverEvent present");
    assert_eq!(fo.original_provider, "deepseek");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.trigger_reason.contains("503 Service Unavailable"));
}

// =========================================================================
// TEST 8: Pipeline Disables Failover When Flags Are Inactive
// =========================================================================
#[tokio::test]
async fn test_pipeline_no_failover_when_disabled() {
    let mut ctx = EngineContext::new(5.0);

    let cloud_prov = Arc::new(MockFailoverProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited("anthropic".into(), None)],
        0.01,
    ));
    let ollama_prov = Arc::new(MockFailoverProvider::new_successful(
        "ollama",
        "```rust\npub struct WontReach;\n```",
        0.0,
    ));

    ctx.register_provider(cloud_prov.clone());
    ctx.register_provider(ollama_prov.clone());

    let role = AssemblyRoles::architect("anthropic", "claude-3-5-sonnet");
    let stage = HarmonyStage::new(role);

    // fallback_to_local and evacuate_on_budget are false by default
    let pipeline = StructuredHarmonyPipeline::new("Test no failover").add_stage(stage);

    let err = pipeline.execute(&ctx).await.unwrap_err();
    assert!(matches!(err, TagisanError::RateLimited(..)));
    assert_eq!(cloud_prov.calls(), 1);
    assert_eq!(ollama_prov.calls(), 0);
}

// =========================================================================
// TEST 9: FailoverEvent Serialization, Deserialization & Backward Compatibility
// =========================================================================
#[test]
fn test_failover_event_serde_and_backward_compatibility() {
    let event = FailoverEvent {
        original_provider: "anthropic".to_string(),
        original_model: "claude-3-5-sonnet".to_string(),
        trigger_reason: "HTTP 429 TPM Exceeded".to_string(),
        evacuated_to_provider: "ollama".to_string(),
        evacuated_to_model: "qwen2.5:0.5b".to_string(),
        timestamp_epoch_ms: 1726050000000,
        cost_at_failover_usd: 0.045,
    };

    let artifact = RoleArtifact {
        role_id: "architect".to_string(),
        role_title: "Lead Systems Architect".to_string(),
        provider: "ollama".to_string(),
        model: "qwen2.5:0.5b".to_string(),
        raw_output: "```rust\npub struct Evacuated;\n```".to_string(),
        code_blocks: vec![ExtractedCodeBlock {
            language: "rust".to_string(),
            code: "pub struct Evacuated;".to_string(),
        }],
        latency_secs: 0.42,
        tokens_used: 150,
        failover_event: Some(event.clone()),
    };

    // 1. Serde roundtrip with FailoverEvent
    let json_str = serde_json::to_string_pretty(&artifact).expect("serialize artifact");
    let deserialized: RoleArtifact = serde_json::from_str(&json_str).expect("deserialize artifact");
    assert_eq!(artifact, deserialized);
    assert_eq!(deserialized.failover_event.unwrap(), event);

    // 2. Backward compatibility: older JSON without failover_event field
    let legacy_json = r#"{
        "role_id": "implementer",
        "role_title": "Senior Systems Implementer",
        "provider": "openai",
        "model": "gpt-4o",
        "raw_output": "code here",
        "code_blocks": [],
        "latency_secs": 1.2,
        "tokens_used": 500
    }"#;

    let legacy_artifact: RoleArtifact = serde_json::from_str(legacy_json).expect("legacy deserialize");
    assert_eq!(legacy_artifact.role_id, "implementer");
    assert_eq!(legacy_artifact.failover_event, None, "default must be None");
}

// =========================================================================
// TEST 10: Multi-Stage Assembly Line with Parallel Concurrency & Failover Stress
// =========================================================================
#[tokio::test]
async fn test_parallel_assembly_line_with_concurrent_failover() {
    let mut ctx = EngineContext::new(10.0);

    // Stage 1 (Architect) -> Cloud succeeds
    let arch_prov = Arc::new(MockFailoverProvider::new_successful(
        "arch_cloud",
        "```rust\npub struct CacheNode { key: u32 }\n```",
        0.001,
    ));
    ctx.register_provider(arch_prov);

    // Stage 2 (Implementer) -> Cloud succeeds
    let imp_prov = Arc::new(MockFailoverProvider::new_successful(
        "imp_cloud",
        "```rust\nimpl CacheNode { pub fn key(&self) -> u32 { self.key } }\n```",
        0.002,
    ));
    ctx.register_provider(imp_prov);

    // Stage 3 (QA) -> Cloud hits HTTP 429 RateLimit!
    let qa_prov = Arc::new(MockFailoverProvider::new(
        "qa_cloud",
        vec![MockBehavior::RateLimited(
            "qa_cloud".into(),
            Some(Duration::from_secs(10)),
        )],
        0.001,
    ));
    ctx.register_provider(qa_prov);

    // Stage 4 (Doc) -> Cloud hits 502 Bad Gateway!
    let doc_prov = Arc::new(MockFailoverProvider::new(
        "doc_cloud",
        vec![MockBehavior::BadResponse(
            "doc_cloud".into(),
            "502 Bad Gateway".into(),
        )],
        0.001,
    ));
    ctx.register_provider(doc_prov);

    // Ollama Local Provider -> Evacuation succeeds for both concurrent stages
    let ollama_prov = Arc::new(MockFailoverProvider::new(
        "ollama",
        vec![
            MockBehavior::Success("```rust\n#[test]\nfn test_cache() { assert_eq!(1, 1); }\n```".into()),
            MockBehavior::Success("# Documentation\n\nHigh-performance cache implementation.".into()),
        ],
        0.0,
    ));
    ctx.register_provider(ollama_prov);

    let stage1 = HarmonyStage::new(AssemblyRoles::architect("arch_cloud", "m1"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage2 = HarmonyStage::new(AssemblyRoles::implementer("imp_cloud", "m2"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage3 = HarmonyStage::new(AssemblyRoles::qa("qa_cloud", "m3"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage4 = HarmonyStage::new(AssemblyRoles::documentation("doc_cloud", "m4"))
        .with_gate(Box::new(SyntaxValidationGate::permissive()));

    let pipeline = StructuredHarmonyPipeline::new("Build Concurrent Cache")
        .add_stage(stage1)
        .add_stage(stage2)
        .add_stage(stage3)
        .add_stage(stage4)
        .with_parallel(true)
        .with_fallback_to_local(true);

    let res = pipeline.execute(&ctx).await.expect("Assembly line must complete successfully");

    assert_eq!(res.artifacts.len(), 4);

    // Stage 1 & 2: Ran on cloud, no failover
    assert_eq!(res.artifacts[0].provider, "arch_cloud");
    assert!(res.artifacts[0].failover_event.is_none());
    assert_eq!(res.artifacts[1].provider, "imp_cloud");
    assert!(res.artifacts[1].failover_event.is_none());

    // Stage 3 (QA): Evacuated to Ollama due to HTTP 429
    assert_eq!(res.artifacts[2].provider, "ollama");
    let fo3 = res.artifacts[2].failover_event.as_ref().expect("Stage 3 failover");
    assert_eq!(fo3.original_provider, "qa_cloud");
    assert_eq!(fo3.evacuated_to_provider, "ollama");
    assert!(fo3.trigger_reason.contains("Rate limited (HTTP 429)"));

    // Stage 4 (Doc): Evacuated to Ollama due to 502 Bad Gateway
    assert_eq!(res.artifacts[3].provider, "ollama");
    let fo4 = res.artifacts[3].failover_event.as_ref().expect("Stage 4 failover");
    assert_eq!(fo4.original_provider, "doc_cloud");
    assert_eq!(fo4.evacuated_to_provider, "ollama");
    assert!(fo4.trigger_reason.contains("502 Bad Gateway"));

    // Final complete assembled project must contain all artifacts
    assert!(res.complete_project.contains("CacheNode"));
    assert!(res.complete_project.contains("test_cache"));
}
