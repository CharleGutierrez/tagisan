//! Dedicated Brutal Integration Test Suite: Unified User Notification & System Abnormality Alerting
//! across Local <-> Non-Local LLM Transitions.
//!
//! Brutally tests:
//! 1. Unified NotificationHub lifecycle, thread safety, and real-time async subscriber pub/sub.
//! 2. HTTP 429 RateLimited provider failover cascade and user notification.
//! 3. HTTP 401 Authentication error provider failover cascade and user notification.
//! 4. HTTP 502 BadResponse / Gateway error provider failover cascade and user notification.
//! 5. Network drop / connection timeout provider failover cascade and user notification.
//! 6. Pre-emptive & reactive Financial Budget Exhaustion failover to zero-cost local runtime and notification.
//! 7. Streaming failover cascade with event-chunk emission and user notification.
//! 8. Skill subsystem prompt injection mode mutation alerting (Local CheatSheet <-> Cloud Comprehensive).
//! 9. Context window downscaling alerting (e.g. 1M Gemini / 200k Claude ➔ 8k-32k local Ollama/Colibri).
//! 10. Domain diversity anti-monopoly quota enforcement alerting across local (max 1) and cloud (max 2).
//! 11. Semantic Invariant Guard: Truncated / corrupted tool schema rejection and alerting.
//! 12. Semantic Invariant Guard: Dropped required parameter interception and alerting.
//! 13. Semantic Invariant Guard: Diagnostic degradation interception when context is constrained.
//! 14. Semantic Invariant Guard: Safety contract omission and context exhaustion alerts.
//! 15. Model Auto-Healing: Missing local model fallback and alerting.
//! 16. Offline / Local-Only Lock: Air-gapped environment override alerting.
//! 17. Multi-Agent Swarm Structured Harmony Pipeline dynamic stage failover and user alerting.
//! 18. Reversibility & Telemetry Counters across 10 alternating Local <-> Cloud cycles.

use async_stream::stream;
use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tagisan::ecc::{
    global_dispatcher, is_local_provider,
    SemanticInvariantGuard, SemanticViolation, TokenBudget,
};
use tagisan::error::{Result, TagisanError};
use tagisan::notify::{
    clear_history, desktop_delivery_count, get_events_by_category, get_events_by_severity,
    history as get_notification_history, notify as emit_notification,
    notify_context_downscaling, notify_domain_quota, notify_failover, notify_model_auto_healed,
    notify_offline_lock, notify_skill_transition, set_banner_enabled,
    set_desktop_enabled, subscribe as subscribe_notifications, terminal_banner_count,
    NotificationCategory, NotificationEvent, NotificationPayload, NotificationSeverity,
};
use tagisan::providers::cascade::{CascadeEntry, CascadeProvider};
use tagisan::providers::{BoxEventStream, LlmProvider};
use tagisan::swarm::{
    AssemblyRoles, HarmonyStage, StructuredHarmonyPipeline, SyntaxValidationGate,
};
use tagisan::types::{
    CompletionRequest, CompletionResponse, FinishReason, Message,
    ProviderCapabilities, StreamChunk, StreamChunkDelta, TokenUsage, ToolDefinition,
};
use tagisan::EngineContext;

// =========================================================================
// DETERMINISTIC MOCK LLM PROVIDERS FOR BRUTAL NOTIFICATION TESTING
// =========================================================================

#[derive(Clone, Debug)]
enum MockFailureMode {
    RateLimited(String, Option<Duration>),
    Authentication(String, String),
    BadResponse(String, String),
    BudgetExceeded(f64, f64),
    #[allow(dead_code)]
    ContextLengthExceeded(String, usize, usize),
}

impl MockFailureMode {
    fn to_tagisan_error(&self) -> TagisanError {
        match self {
            Self::RateLimited(p, d) => TagisanError::RateLimited(p.clone(), *d),
            Self::Authentication(p, m) => TagisanError::Authentication(p.clone(), m.clone()),
            Self::BadResponse(p, m) => TagisanError::BadResponse(p.clone(), m.clone()),
            Self::BudgetExceeded(max, spent) => TagisanError::BudgetExceeded {
                max_budget: *max,
                current_spent: *spent,
            },
            Self::ContextLengthExceeded(p, req, max) => {
                TagisanError::ContextLengthExceeded(p.clone(), *req, *max)
            }
        }
    }
}

struct MockNetworkDropProvider;

#[async_trait]
impl LlmProvider for MockNetworkDropProvider {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, _req: CompletionRequest) -> Result<CompletionResponse> {
        let err = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(1))
            .build()
            .unwrap()
            .get("http://127.0.0.1:1/nonexistent_socket")
            .send()
            .await
            .unwrap_err();
        Err(TagisanError::Network(err))
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream> {
        let err = reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(1))
            .build()
            .unwrap()
            .get("http://127.0.0.1:1/nonexistent_socket")
            .send()
            .await
            .unwrap_err();
        Err(TagisanError::Network(err))
    }
}

#[derive(Clone)]
struct MockNotificationLlmProvider {
    id: &'static str,
    call_count: Arc<AtomicUsize>,
    stream_call_count: Arc<AtomicUsize>,
    canned_responses: Vec<String>,
    failure_err: Arc<Mutex<Option<MockFailureMode>>>,
    fail_on_call_index: Arc<Mutex<Option<usize>>>,
    fail_on_stream: Arc<Mutex<bool>>,
    recorded_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl MockNotificationLlmProvider {
    fn new(id: &'static str, canned_responses: Vec<String>) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            stream_call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses,
            failure_err: Arc::new(Mutex::new(None)),
            fail_on_call_index: Arc::new(Mutex::new(None)),
            fail_on_stream: Arc::new(Mutex::new(false)),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_persistent_error(id: &'static str, err: MockFailureMode) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            stream_call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses: Vec::new(),
            failure_err: Arc::new(Mutex::new(Some(err))),
            fail_on_call_index: Arc::new(Mutex::new(None)),
            fail_on_stream: Arc::new(Mutex::new(true)),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    fn recorded_requests(&self) -> Vec<CompletionRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmProvider for MockNotificationLlmProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        self.recorded_requests.lock().unwrap().push(req.clone());

        let fail_idx_opt = *self.fail_on_call_index.lock().unwrap();
        if let Some(target_idx) = fail_idx_opt {
            if idx == target_idx {
                if let Some(ref err) = *self.failure_err.lock().unwrap() {
                    return Err(err.to_tagisan_error());
                }
            }
        } else if let Some(ref err) = *self.failure_err.lock().unwrap() {
            return Err(err.to_tagisan_error());
        }

        let resp_text = if idx < self.canned_responses.len() {
            self.canned_responses[idx].clone()
        } else {
            self.canned_responses
                .last()
                .cloned()
                .unwrap_or_else(|| "```rust\npub fn notification_test_ok() -> bool { true }\n```".to_string())
        };

        Ok(CompletionResponse {
            id: format!("mock-notif-{}", idx),
            provider: self.id.to_string(),
            model: req.model.clone(),
            message: Message::assistant(resp_text),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 50,
                completion_tokens: 25,
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: if is_local_provider(self.id) { Some(0.0) } else { Some(0.002) },
            },
            latency: Duration::from_millis(5),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream> {
        let _idx = self.stream_call_count.fetch_add(1, Ordering::SeqCst);

        if *self.fail_on_stream.lock().unwrap() {
            if let Some(ref err) = *self.failure_err.lock().unwrap() {
                return Err(err.to_tagisan_error());
            }
        }

        let s = stream! {
            yield Ok(StreamChunk::text("Hello from stream!"));
            yield Ok(StreamChunk {
                delta: StreamChunkDelta::Text(String::new()),
                finish_reason: Some(FinishReason::Stop),
                usage: Some(TokenUsage {
                    prompt_tokens: 20,
                    completion_tokens: 10,
                    reasoning_tokens: None,
                    cached_prompt_tokens: None,
                    estimated_cost_usd: Some(0.0),
                }),
            });
        };

        Ok(Box::pin(s))
    }
}

// Global serial mutex to protect shared static NotificationHub state across multi-threaded cargo test runs
static TEST_SERIAL_MUTEX: Mutex<()> = Mutex::new(());

fn reset_notification_env() -> MutexGuard<'static, ()> {
    let guard = TEST_SERIAL_MUTEX.lock().unwrap();
    clear_history();
    set_banner_enabled(false); // keep stderr clean during tests, banners verified via counters
    set_desktop_enabled(true);
    std::env::set_var("TAGISAN_TEST_MODE", "1");
    guard
}

// =========================================================================
// TEST SUITE: BRUTAL VERIFICATION
// =========================================================================

#[tokio::test]
async fn test_01_notification_hub_core_and_async_subscriber_stream() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 1: NotificationHub Lifecycle & Subscriber Pub/Sub ===");

    assert_eq!(get_notification_history().len(), 0);

    let mut sub = subscribe_notifications();

    let event1 = NotificationEvent::new(
        NotificationCategory::SystemAlert,
        NotificationSeverity::Info,
        "System Boot",
        "Tagisan engine initialized",
        "Ready to accept workloads",
        NotificationPayload::Generic([("version".to_string(), "0.2.0".to_string())].into_iter().collect()),
    );
    emit_notification(event1);

    let received = sub.recv().await.expect("Subscriber should receive live event");
    assert_eq!(received.title, "System Boot");
    assert_eq!(received.category, NotificationCategory::SystemAlert);
    assert_eq!(received.severity, NotificationSeverity::Info);
    assert!(received.id >= 1);

    // Verify history snapshot
    let hist = get_notification_history();
    assert_eq!(hist.len(), 1);
    assert_eq!(hist[0].title, "System Boot");

    // Test category filter
    let alerts = get_events_by_category(NotificationCategory::SystemAlert);
    assert_eq!(alerts.len(), 1);
    let failovers = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(failovers.len(), 0);

    // Test severity filter
    let infos = get_events_by_severity(NotificationSeverity::Info);
    assert_eq!(infos.len(), 1);
    let criticals = get_events_by_severity(NotificationSeverity::Critical);
    assert_eq!(criticals.len(), 0);

    println!("  [✓] NotificationHub pub/sub, ordering, and filtering verified");
}

#[tokio::test]
async fn test_02_failover_429_rate_limited_cascade_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 2: HTTP 429 RateLimited Cascade & Notification ===");

    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "anthropic",
        MockFailureMode::RateLimited("anthropic".to_string(), Some(Duration::from_secs(45))),
    ));
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec!["```rust\npub fn rate_limit_evacuated() -> bool { true }\n```".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("claude-3-5-sonnet".to_string())),
        CascadeEntry::new(local_prov, Some("qwen2.5:0.5b".to_string())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "Implement task");
    let resp = cascade.complete(req).await.expect("Cascade must succeed on local fallback");
    assert!(resp.message.extract_text().contains("rate_limit_evacuated"));

    // Verify Notification Hub captured the 429 failover event
    let hist = get_notification_history();
    assert_eq!(hist.len(), 1, "Exactly 1 failover event should be recorded");

    let event = &hist[0];
    assert_eq!(event.category, NotificationCategory::ProviderFailover);
    assert_eq!(event.severity, NotificationSeverity::Critical);
    assert!(event.title.contains("anthropic ➔ ollama"));
    assert!(event.message.contains("HTTP 429"));
    assert!(event.message.contains("45s"));

    if let NotificationPayload::Failover(f) = &event.payload {
        assert_eq!(f.original_provider, "anthropic");
        assert_eq!(f.original_model, "claude-3-5-sonnet");
        assert_eq!(f.target_provider, "ollama");
        assert_eq!(f.target_model, "qwen2.5:0.5b");
        assert!(f.cost_delta.contains("+$0.00"));
        assert!(f.action_taken.contains("Evacuating"));
    } else {
        panic!("Expected Failover payload");
    }

    println!("  [✓] HTTP 429 rate limit cascade verified with full notification metadata");
}

#[tokio::test]
async fn test_03_failover_401_authentication_cascade_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 3: HTTP 401 Authentication Error Cascade & Notification ===");

    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "gemini",
        MockFailureMode::Authentication("gemini".to_string(), "401 Unauthorized: Invalid API token".to_string()),
    ));
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec!["```rust\npub fn auth_failover_recovered() -> bool { true }\n```".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("gemini-2.0-flash".to_string())),
        CascadeEntry::new(local_prov, Some("dolphin-phi:latest".to_string())),
    ]);

    let req = CompletionRequest::new("gemini-2.0-flash", "Secure compute");
    let resp = cascade.complete(req).await.expect("Cascade must succeed on local fallback");
    assert!(resp.message.extract_text().contains("auth_failover_recovered"));

    let hist = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(hist.len(), 1);
    let event = &hist[0];
    assert!(event.message.contains("HTTP 401"));
    assert!(event.message.contains("Invalid API token"));

    if let NotificationPayload::Failover(f) = &event.payload {
        assert_eq!(f.original_provider, "gemini");
        assert_eq!(f.target_provider, "ollama");
        assert_eq!(f.target_model, "dolphin-phi:latest");
        assert!(f.cost_delta.contains("Zero incremental cost"));
    } else {
        panic!("Expected Failover payload");
    }

    println!("  [✓] HTTP 401 auth error cascade emitted authentic user alert");
}

#[tokio::test]
async fn test_04_failover_502_bad_gateway_cascade_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 4: HTTP 502 Bad Gateway Cascade & Notification ===");

    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "deepseek",
        MockFailureMode::BadResponse("deepseek".to_string(), "502 Bad Gateway from cloud edge proxy".to_string()),
    ));
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "colibri",
        vec!["```rust\npub fn colibri_healed() -> bool { true }\n```".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("deepseek-chat".to_string())),
        CascadeEntry::new(local_prov, Some("colibri-default".to_string())),
    ]);

    let req = CompletionRequest::new("deepseek-chat", "Reasoning task");
    let resp = cascade.complete(req).await.expect("Cascade must succeed on Colibri local");
    assert!(resp.message.extract_text().contains("colibri_healed"));

    let hist = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(hist.len(), 1);
    assert!(hist[0].message.contains("502 Bad Gateway"));

    if let NotificationPayload::Failover(f) = &hist[0].payload {
        assert_eq!(f.original_provider, "deepseek");
        assert_eq!(f.target_provider, "colibri");
        assert!(f.cost_delta.contains("+$0.00"));
    } else {
        panic!("Expected Failover payload");
    }

    println!("  [✓] HTTP 502 Bad Gateway cascade alert caught and recorded");
}

#[tokio::test]
async fn test_05_failover_network_timeout_cascade_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 5: Network Timeout Drop Cascade & Notification ===");

    let cloud_prov = Arc::new(MockNetworkDropProvider);
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec!["```rust\npub fn offline_safe() -> bool { true }\n```".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("gpt-4o".to_string())),
        CascadeEntry::new(local_prov, Some("mistral:7b".to_string())),
    ]);

    let req = CompletionRequest::new("gpt-4o", "Timeout simulation");
    let resp = cascade.complete(req).await.expect("Cascade must succeed on local");
    assert!(resp.message.extract_text().contains("offline_safe"));

    let hist = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(hist.len(), 1);
    assert!(hist[0].message.contains("Network failure"));

    println!("  [✓] Network timeout failure cascade accurately alerted user");
}

#[tokio::test]
async fn test_06_failover_budget_exhaustion_to_zero_cost_local_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 6: Financial Budget Exhaustion Failover & Notification ===");

    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "anthropic",
        MockFailureMode::BudgetExceeded(5.00, 5.02),
    ));
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec!["```rust\npub fn zero_cost_recovery() -> bool { true }\n```".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("claude-3-5-sonnet".to_string())),
        CascadeEntry::new(local_prov, Some("qwen2.5:0.5b".to_string())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "High budget cost task");
    let resp = cascade.complete(req).await.expect("Cascade must evacuate to zero-cost local");
    assert!(resp.message.extract_text().contains("zero_cost_recovery"));

    let hist = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(hist.len(), 1);
    assert!(hist[0].message.contains("Token budget reached"));
    assert!(hist[0].message.contains("$5.02 spent >= $5.00 max"));

    if let NotificationPayload::Failover(f) = &hist[0].payload {
        assert_eq!(f.original_provider, "anthropic");
        assert_eq!(f.target_provider, "ollama");
        assert!(f.cost_delta.contains("+$0.00"));
    } else {
        panic!("Expected Failover payload");
    }

    println!("  [✓] Budget exhaustion zero-cost failover verified with financial delta alert");
}

#[tokio::test]
async fn test_07_streaming_failover_cascade_and_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 7: Streaming Failover Cascade & Notification ===");

    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "gemini",
        MockFailureMode::RateLimited("gemini".to_string(), Some(Duration::from_secs(10))),
    ));
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec!["Stream fallback text".to_string()],
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(cloud_prov, Some("gemini-2.0-flash".to_string())),
        CascadeEntry::new(local_prov, Some("qwen2.5:0.5b".to_string())),
    ]);

    let req = CompletionRequest::new("gemini-2.0-flash", "Stream me code");
    let stream_res = cascade.stream(req).await;
    assert!(stream_res.is_ok(), "Streaming cascade should succeed on fallback");

    let hist = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(hist.len(), 1);
    assert!(hist[0].message.contains("Rate limited (HTTP 429)"));

    println!("  [✓] Streaming initialization failure cascaded and notified user");
}

#[tokio::test]
async fn test_08_skill_subsystem_transition_local_cheatsheet_vs_cloud_comprehensive() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 8: Skill Subsystem Mode Transition Alerts ===");

    let cloud_budget = TokenBudget::for_provider_and_model("anthropic", Some("claude-3-5-sonnet"));
    assert_eq!(cloud_budget.max_tokens, 12_000);
    assert_eq!(cloud_budget.max_per_domain, 2);

    // Transition Cloud ➔ Local Ollama
    let local_budget = cloud_budget.transition_and_notify(
        "anthropic",
        "ollama",
        Some("dolphin-phi:latest"),
        "Dynamic failover evacuated cloud role to edge Ollama runtime",
    );
    assert_eq!(local_budget.max_tokens, 1_200);
    assert_eq!(local_budget.max_per_domain, 1);

    // Verify notification emitted
    let hist = get_events_by_category(NotificationCategory::SkillTransition);
    assert_eq!(hist.len(), 1);
    let event = &hist[0];
    assert_eq!(event.severity, NotificationSeverity::Info);
    assert!(event.message.contains("anthropic"));
    assert!(event.message.contains("ollama"));

    if let NotificationPayload::SkillTransition(s) = &event.payload {
        assert_eq!(s.from_provider, "anthropic");
        assert_eq!(s.to_provider, "ollama");
        assert_eq!(s.token_budget, 1_200);
        assert_eq!(s.context_window, 8_192);
        assert!(s.action_taken.contains("Mutated prompt injection mode"));
    } else {
        panic!("Expected SkillTransition payload");
    }

    // Transition Local ➔ Cloud Gemini
    let gemini_budget = local_budget.transition_and_notify(
        "ollama",
        "gemini",
        Some("gemini-2.0-flash"),
        "Cloud connectivity restored; restoring full comprehensive capability",
    );
    assert_eq!(gemini_budget.max_tokens, 16_000);

    let hist_after = get_events_by_category(NotificationCategory::SkillTransition);
    assert_eq!(hist_after.len(), 2);
    assert!(hist_after[1].title.contains("DenseInvariants ➔ Hierarchical"));

    println!("  [✓] Local <-> Cloud injection mode transitions accurately notified");
}

#[tokio::test]
async fn test_09_context_downscaling_notification_cloud_to_edge() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 9: Context Window Downscaling Notification ===");

    notify_context_downscaling(
        1_000_000,
        8_192,
        "qwen2.5:0.5b",
        "Edge hardware constraint: context window reduced from Gemini 1M to Local 8k",
        "Scaled down prompt and compacted history to fit edge context",
    );

    let hist = get_events_by_category(NotificationCategory::ContextDownscaling);
    assert_eq!(hist.len(), 1);
    let event = &hist[0];
    assert_eq!(event.severity, NotificationSeverity::Warning);
    assert!(event.title.contains("1000000 ➔ 8192"));

    if let NotificationPayload::ContextDownscale(c) = &event.payload {
        assert_eq!(c.original_context, 1_000_000);
        assert_eq!(c.downscaled_context, 8_192);
        assert_eq!(c.model, "qwen2.5:0.5b");
        assert!(c.reason.contains("Edge hardware constraint"));
    } else {
        panic!("Expected ContextDownscale payload");
    }

    println!("  [✓] Context window downscaling alert recorded with exact token bounds");
}

#[tokio::test]
async fn test_10_domain_diversity_anti_monopoly_quota_alerting_local() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 10: Domain Diversity Anti-Monopoly Quota Alerts ===");

    let dispatcher = global_dispatcher();
    let local_budget = TokenBudget::local_default(); // max_per_domain = 1
    assert_eq!(local_budget.max_per_domain, 1);

    // Query designed to match multiple skills in the "bun" domain
    let result = dispatcher.dispatch_diversified("bun test bun run bun build bun install bun serve", local_budget, None);

    // Local quota ensures max 1 skill per domain
    let mut domain_counts = std::collections::HashMap::new();
    for s in &result.primary {
        *domain_counts.entry(s.domain.clone()).or_insert(0) += 1;
    }
    for (d, count) in domain_counts {
        assert!(count <= 1, "Domain '{}' exceeded local quota of 1: {}", d, count);
    }

    // Check if domain quota notification was emitted
    let quota_events = get_events_by_category(NotificationCategory::DomainDiversityQuota);
    assert!(!quota_events.is_empty(), "Domain diversity quota should alert when candidates are suppressed");

    let first_quota = &quota_events[0];
    assert_eq!(first_quota.severity, NotificationSeverity::Warning);
    assert!(first_quota.title.contains("Domain Diversity Quota"));

    if let NotificationPayload::DomainQuota(q) = &first_quota.payload {
        assert_eq!(q.max_allowed, 1);
        assert_eq!(q.provider, "local");
        assert!(!q.attempted_skill.is_empty());
    } else {
        panic!("Expected DomainQuota payload");
    }

    println!("  [✓] Domain anti-monopoly quotas successfully alerted user for suppressed candidates");
}

#[tokio::test]
async fn test_11_semantic_guard_truncated_tool_schema_alerts() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 11: Semantic Guard Truncated Tool Schema Rejection Alerts ===");

    let guard = SemanticInvariantGuard::new();

    // 1. Empty tool name
    let empty_name_tool = ToolDefinition {
        name: "   ".to_string(),
        description: "Valid description".to_string(),
        parameters: json!({"type": "object"}),
    };
    let res1 = guard.enforce_tool_schema_integrity(&empty_name_tool);
    assert!(matches!(res1, Err(SemanticViolation::TruncatedToolSchema { .. })));

    // 2. Truncated description ending with ellipsis
    let truncated_desc_tool = ToolDefinition {
        name: "execute_sql".to_string(),
        description: "Execute SQL query on database...".to_string(),
        parameters: json!({"type": "object"}),
    };
    let res2 = guard.enforce_tool_schema_integrity(&truncated_desc_tool);
    assert!(matches!(res2, Err(SemanticViolation::TruncatedToolSchema { .. })));

    // 3. Non-object parameters
    let invalid_params_tool = ToolDefinition {
        name: "run_job".to_string(),
        description: "Runs a background job".to_string(),
        parameters: json!("invalid_array"),
    };
    let res3 = guard.enforce_tool_schema_integrity(&invalid_params_tool);
    assert!(matches!(res3, Err(SemanticViolation::TruncatedToolSchema { .. })));

    // Verify all 3 violations generated alerts in NotificationHub
    let events = get_events_by_category(NotificationCategory::SemanticGuard);
    assert_eq!(events.len(), 3);
    for e in events {
        assert_eq!(e.severity, NotificationSeverity::Critical);
        assert!(e.title.contains("TruncatedToolSchema"));
    }

    println!("  [✓] Truncated / malformed tool schemas intercepted and user alerted");
}

#[tokio::test]
async fn test_12_semantic_guard_dropped_required_param_alert() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 12: Semantic Guard Dropped Required Parameter Alerts ===");

    let guard = SemanticInvariantGuard::new();

    let missing_required_tool = ToolDefinition {
        name: "query_database".to_string(),
        description: "Executes a query against the DB".to_string(),
        parameters: json!({
            "type": "object",
            "required": ["query_sql", "target_table"],
            "properties": {
                "target_table": { "type": "string" }
                // query_sql omitted!
            }
        }),
    };

    let res = guard.enforce_tool_schema_integrity(&missing_required_tool);
    assert!(matches!(
        res,
        Err(SemanticViolation::DroppedRequiredParameter { ref param_name, .. }) if param_name == "query_sql"
    ));

    let events = get_events_by_category(NotificationCategory::SemanticGuard);
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert!(event.title.contains("DroppedRequiredParameter"));
    assert!(event.message.contains("query_database.query_sql"));

    if let NotificationPayload::SemanticGuard(g) = &event.payload {
        assert_eq!(g.violation_type, "DroppedRequiredParameter");
        assert_eq!(g.target_item, "query_database.query_sql");
        assert!(g.action_taken.contains("dropped required parameter"));
    } else {
        panic!("Expected SemanticGuard payload");
    }

    println!("  [✓] Dropped required parameter caught and alerted with exact target");
}

#[tokio::test]
async fn test_13_semantic_guard_diagnostic_degradation_alert() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 13: Semantic Guard Diagnostic Degradation Alerts ===");

    let guard = SemanticInvariantGuard::new();

    let compiler_diagnostic = r#"
error[E0382]: borrow of moved value: `pipeline`
  --> src/main.rs:42:15
   |
40 |     let pipeline = build_pipeline();
   |         -------- move occurs because `pipeline` has type `Pipeline`, which does not implement `Copy`
41 |     execute(pipeline);
   |             -------- value moved here
42 |     let _ = pipeline.status();
   |             ^^^^^^^^ value borrowed here after move
help: consider cloning the value
   |
41 |     execute(pipeline.clone());
   |                     ++++++++
"#;

    // Constrain max tokens below minimum required for full semantic preservation
    let res = guard.enforce_diagnostic_preservation(compiler_diagnostic, 5);
    assert!(matches!(res, Err(SemanticViolation::DiagnosticDegradation { ref error_code, .. }) if error_code == "E0382"));

    let events = get_events_by_category(NotificationCategory::SemanticGuard);
    assert_eq!(events.len(), 1);
    assert!(events[0].title.contains("DiagnosticDegradation"));
    assert!(events[0].message.contains("E0382"));

    println!("  [✓] Compiler diagnostic degradation prevented and alerted to user");
}

#[tokio::test]
async fn test_14_semantic_guard_safety_contract_alert() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 14: Semantic Guard Safety Contract Alerts ===");

    let guard = SemanticInvariantGuard::new();

    // 1. Context exhaustion risk
    let res1 = guard.enforce_safety_contract(100, 50, &["Rule 1: Always verify auth"]);
    assert!(matches!(res1, Err(SemanticViolation::ContextExhaustionWithInvariantRisk { .. })));

    // 2. Empty safety rule declared
    let res2 = guard.enforce_safety_contract(10, 500, &["   "]);
    assert!(matches!(res2, Err(SemanticViolation::SafetyContractOmitted { .. })));

    let events = get_events_by_category(NotificationCategory::SemanticGuard);
    assert_eq!(events.len(), 2);
    assert!(events[0].title.contains("ContextExhaustionWithInvariantRisk"));
    assert!(events[1].title.contains("SafetyContractOmitted"));

    println!("  [✓] Safety contract violations cleanly intercepted and alerted");
}

#[tokio::test]
async fn test_15_model_auto_healing_missing_local_model_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 15: Model Auto-Healing Fallback Notification ===");

    notify_model_auto_healed(
        "deepseek-r1:70b",
        "qwen2.5:0.5b",
        "ollama",
        "Specified model 'deepseek-r1:70b' is not installed locally in Ollama catalog",
        "Automatically falling back to 'qwen2.5:0.5b' and healed .env configuration",
    );

    let events = get_events_by_category(NotificationCategory::ModelAutoHeal);
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.severity, NotificationSeverity::Warning);
    assert!(event.title.contains("deepseek-r1:70b"));
    assert!(event.title.contains("qwen2.5:0.5b"));

    if let NotificationPayload::AutoHeal(h) = &event.payload {
        assert_eq!(h.requested_model, "deepseek-r1:70b");
        assert_eq!(h.healed_model, "qwen2.5:0.5b");
        assert_eq!(h.provider, "ollama");
        assert!(h.action_taken.contains("healed .env"));
    } else {
        panic!("Expected AutoHeal payload");
    }

    println!("  [✓] Model auto-healing notification verified with requested & healed models");
}

#[tokio::test]
async fn test_16_offline_local_only_lock_override_notification() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 16: Offline / Local-Only Lock Override Notification ===");

    notify_offline_lock(
        "anthropic",
        "ollama",
        "TAGISAN_LOCAL_ONLY or TAGISAN_OFFLINE environment lock is active",
        "Overriding cloud provider request to local Ollama runtime (air-gapped privacy enforced)",
    );

    let events = get_events_by_category(NotificationCategory::OfflineLockOverride);
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.severity, NotificationSeverity::SecurityAlert);
    assert!(event.title.contains("Disallowed 'anthropic'"));
    assert!(event.action_taken.contains("air-gapped privacy enforced"));
    assert!(event.message.contains("TAGISAN_LOCAL_ONLY"));

    if let NotificationPayload::OfflineLock(l) = &event.payload {
        assert_eq!(l.requested_provider, "anthropic");
        assert_eq!(l.enforced_provider, "ollama");
        assert!(l.lock_reason.contains("TAGISAN_LOCAL_ONLY"));
    } else {
        panic!("Expected OfflineLock payload");
    }

    println!("  [✓] Offline lock override security notification captured accurately");
}

#[tokio::test]
async fn test_17_swarm_harmony_pipeline_multi_stage_failover_notifications() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 17: Swarm Multi-Stage Pipeline Failover Notifications ===");

    let mut ctx = EngineContext::new(10.0);

    // Stage 1: Cloud provider that encounters 502 Bad Gateway
    let cloud_prov = Arc::new(MockNotificationLlmProvider::with_persistent_error(
        "deepseek",
        MockFailureMode::BadResponse("deepseek".to_string(), "502 Bad Gateway from Cloud Proxy".to_string()),
    ));
    // Local provider for fallback and local roles
    let local_prov = Arc::new(MockNotificationLlmProvider::new(
        "ollama",
        vec![
            "```rust\npub fn stage1_recovered_local() -> bool { true }\n```".to_string(),
        ],
    ));

    ctx.register_provider(cloud_prov);
    ctx.register_provider(local_prov.clone());

    let imp_role = AssemblyRoles::implementer("deepseek", "deepseek-chat");

    let pipeline = StructuredHarmonyPipeline::new("Build distributed pipeline")
        .with_fallback_to_local(true)
        .add_stage(HarmonyStage::new(imp_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));

    let result = pipeline.execute(&ctx).await.expect("Pipeline must succeed through failover");

    assert_eq!(result.artifacts.len(), 1);
    assert!(result.artifacts[0].failover_event.is_some());
    let ev = result.artifacts[0].failover_event.as_ref().unwrap();
    assert_eq!(ev.original_provider, "deepseek");
    assert_eq!(ev.evacuated_to_provider, "ollama");
    assert!(ev.trigger_reason.contains("502 Bad Gateway"));

    // Verify NotificationHub captured the failover from Swarm pipeline
    let failover_events = get_events_by_category(NotificationCategory::ProviderFailover);
    assert!(!failover_events.is_empty(), "Failover event should be captured by NotificationHub");

    let event = &failover_events[0];
    assert!(event.message.contains("deepseek"));
    assert!(event.message.contains("ollama"));
    assert!(event.message.contains("502 Bad Gateway"));

    if let NotificationPayload::Failover(f) = &event.payload {
        assert_eq!(f.original_provider, "deepseek");
        assert_eq!(f.target_provider, "ollama");
        assert!(f.action_taken.contains("Evacuating"));
    } else {
        panic!("Expected Failover payload");
    }

    println!("  [✓] Swarm pipeline dynamic stage failover notified with role, stage, and provider delta");
}

#[tokio::test]
async fn test_18_reversibility_and_counter_telemetry_across_10_cycles() {
    let _guard = reset_notification_env();
    println!("\n=== TEST 18: 10-Cycle Reversibility & Telemetry Counters ===");

    set_banner_enabled(true);
    let initial_desktop_count = desktop_delivery_count();
    let initial_banner_count = terminal_banner_count();

    for cycle in 1..=10 {
        // Step A: Local to Cloud
        notify_skill_transition(
            "ollama",
            "anthropic",
            "CheatSheet",
            "Comprehensive",
            12_000,
            200_000,
            &format!("Cycle {} scaling up to Cloud", cycle),
            "Calibrated prompt to comprehensive multi-domain guidelines",
        );

        // Step B: Cloud to Local (Failover)
        notify_failover(
            "anthropic",
            "claude-3-5-sonnet",
            "ollama",
            "qwen2.5:0.5b",
            &format!("Cycle {} simulated transient spike", cycle),
            "+$0.00 (Zero incremental cost on local hardware)",
            "Evacuating to local edge model",
            Some(format!("Cycle {}", cycle)),
        );

        // Step C: Quota alert
        notify_domain_quota(
            "security",
            1,
            "security-audit",
            vec!["security-audit".to_string()],
            "local",
            "Enforcing 1:1 domain diversity invariant on edge runtime",
        );
    }

    let all_history = get_notification_history();
    assert_eq!(all_history.len(), 30, "10 cycles * 3 events = 30 events in history");

    let transitions = get_events_by_category(NotificationCategory::SkillTransition);
    assert_eq!(transitions.len(), 10);

    let failovers = get_events_by_category(NotificationCategory::ProviderFailover);
    assert_eq!(failovers.len(), 10);

    let quotas = get_events_by_category(NotificationCategory::DomainDiversityQuota);
    assert_eq!(quotas.len(), 10);

    // Verify sequential IDs
    for i in 1..all_history.len() {
        assert!(all_history[i].id > all_history[i - 1].id, "Event IDs must be strictly monotonically increasing");
    }

    // Verify desktop delivery and banner counters incremented
    assert!(desktop_delivery_count() >= initial_desktop_count + 30);
    assert!(terminal_banner_count() >= initial_banner_count + 30);

    // Clear history and verify clean reset
    clear_history();
    assert_eq!(get_notification_history().len(), 0);

    println!("  [✓] 10 alternating cycles executed with 100% telemetry fidelity and zero state corruption");
}
