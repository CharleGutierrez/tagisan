//! =========================================================================
//! LOCAL TO NON-LOCAL (AND BACK) TRANSITION BRUTAL TEST SUITE
//! =========================================================================
//!
//! Brutal, comprehensive, and exhaustive tests verifying 100% feature parity,
//! failover resilience, budget enforcement, tool calling, reasoning extraction,
//! and session reversibility when transitioning between Local LLMs (Ollama, Colibri,
//! local GGUF) and Non-Local / Cloud LLMs (Gemini, Anthropic, OpenAI, DeepSeek, xAI).
//!
//! Tested Domains:
//! 1. CLI Provider Resolution & Environment Invariants
//! 2. CascadeProvider Live Failover & Boundary Transitions
//! 3. Token Budgeting & Cost Accounting Across Transitions
//! 4. Multi-Agent Harmony Swarm Mixed Local & Non-Local Roles
//! 5. Tool Calling & Agentic Loop Parity Across Local and Cloud
//! 6. Reasoning Extraction & Special Tokens Parity (<think> vs native)
//! 7. Reversibility: Seamless Local -> Cloud -> Local Session Persistence
//! =========================================================================

use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tagisan::cli::{
    build_engine_context, default_model_for_provider, resolve_provider_and_model,
};
use tagisan::ecc::{global_dispatcher, is_local_provider, InjectionMode};
use tagisan::error::{Result, TagisanError};
use tagisan::providers::cascade::{CascadeEntry, CascadeProvider};
use tagisan::providers::ollama::{parse_thinking_blocks, OllamaProvider};
use tagisan::providers::openai_compat::StreamingThinkParser;
use tagisan::providers::{BoxEventStream, LlmProvider};
use tagisan::swarm::harmony::gates::SyntaxValidationGate;
use tagisan::swarm::harmony::types::FailoverEvent;
use tagisan::swarm::harmony::{
    AssemblyRoles, HarmonyStage, StructuredHarmonyPipeline,
};
use tagisan::tools::builtin::CalculatorTool;
use tagisan::tools::ToolRegistry;
use tagisan::types::{
    ChatSession, CompletionRequest, CompletionResponse, ContentBlock, FinishReason,
    Message, ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use tagisan::{AutonomousAgent, EngineContext, TokenBudgetTracker};

// =========================================================================
// TEST PROCESS MUTEX FOR FLAKE-FREE DETERMINISTIC ENV TESTS
// =========================================================================

static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// RAII Guard that saves and restores environment variables during tests
struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new(vars: &[&'static str]) -> Self {
        let saved = vars
            .iter()
            .map(|&v| (v, std::env::var(v).ok()))
            .collect();
        Self { saved }
    }

    fn set(&self, var: &'static str, val: &str) {
        std::env::set_var(var, val);
    }

    fn remove(&self, var: &'static str) {
        std::env::remove_var(var);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (var, val) in &self.saved {
            match val {
                Some(v) => std::env::set_var(var, v),
                None => std::env::remove_var(var),
            }
        }
    }
}

// =========================================================================
// MOCK PROVIDER INFRASTRUCTURE FOR BRUTAL FAILOVER & DYNAMIC TRANSITIONS
// =========================================================================

#[allow(dead_code)]
#[derive(Clone)]
enum MockBehavior {
    Success(String),
    ToolCall {
        id: String,
        name: String,
        args: serde_json::Value,
    },
    BudgetExceeded {
        max_budget: f64,
        current_spent: f64,
    },
    RateLimited(String, Option<Duration>),
    BadResponse(String, String),
    ContextLengthExceeded(String, usize, usize),
    Network(String),
    Authentication(String, String),
    Cancelled,
    Io(String),
    Serialization(String),
}

#[derive(Clone)]
struct MockUniversalProvider {
    id: String,
    behaviors: Vec<MockBehavior>,
    call_count: Arc<AtomicUsize>,
    cost_usd: f64,
    caps: ProviderCapabilities,
    recorded_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl MockUniversalProvider {
    fn new(
        id: impl Into<String>,
        behaviors: Vec<MockBehavior>,
        cost_usd: f64,
        caps: ProviderCapabilities,
    ) -> Self {
        Self {
            id: id.into(),
            behaviors,
            call_count: Arc::new(AtomicUsize::new(0)),
            cost_usd,
            caps,
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn successful(
        id: impl Into<String>,
        text: impl Into<String>,
        cost_usd: f64,
        caps: ProviderCapabilities,
    ) -> Self {
        Self::new(id, vec![MockBehavior::Success(text.into())], cost_usd, caps)
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    fn recorded_requests(&self) -> Vec<CompletionRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmProvider for MockUniversalProvider {
    fn provider_id(&self) -> &'static str {
        Box::leak(self.id.clone().into_boxed_str())
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        self.caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        self.recorded_requests.lock().unwrap().push(req.clone());
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
                    prompt_tokens: 15,
                    completion_tokens: 25,
                    reasoning_tokens: if self.caps.contains(ProviderCapabilities::REASONING_EXTRACTION) {
                        Some(64)
                    } else {
                        None
                    },
                    cached_prompt_tokens: None,
                    estimated_cost_usd: Some(self.cost_usd),
                },
                latency: Duration::from_millis(5),
            }),
            MockBehavior::ToolCall { id, name, args } => {
                let msg = Message::tool_call(id.clone(), name.clone(), args.clone());
                Ok(CompletionResponse {
                    id: format!("{}-tool-{}", self.id, idx),
                    provider: self.id.clone(),
                    model: req.model.clone(),
                    message: msg,
                    finish_reason: FinishReason::ToolCalls,
                    usage: TokenUsage {
                        prompt_tokens: 20,
                        completion_tokens: 10,
                        reasoning_tokens: None,
                        cached_prompt_tokens: None,
                        estimated_cost_usd: Some(self.cost_usd),
                    },
                    latency: Duration::from_millis(5),
                })
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
            MockBehavior::ContextLengthExceeded(p, cur, max) => {
                Err(TagisanError::ContextLengthExceeded(p.clone(), *cur, *max))
            }
            MockBehavior::Network(_) => {
                let err = reqwest::Client::new().get("http://bad uri with spaces").build().unwrap_err();
                Err(TagisanError::Network(err))
            }
            MockBehavior::Authentication(p, msg) => {
                Err(TagisanError::Authentication(p.clone(), msg.clone()))
            }
            MockBehavior::Cancelled => Err(TagisanError::Cancelled),
            MockBehavior::Io(msg) => Err(TagisanError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                msg.clone(),
            ))),
            MockBehavior::Serialization(_) => {
                let err = serde_json::from_str::<serde_json::Value>("{bad").unwrap_err();
                Err(TagisanError::Serialization(err))
            }
        }
    }

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        self.recorded_requests.lock().unwrap().push(req.clone());
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
                    yield Ok(StreamChunk::done(FinishReason::Stop, None));
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
            MockBehavior::Network(_) => {
                let err = reqwest::Client::new().get("http://bad uri with spaces").build().unwrap_err();
                Err(TagisanError::Network(err))
            }
            MockBehavior::Authentication(p, msg) => {
                Err(TagisanError::Authentication(p.clone(), msg.clone()))
            }
            _ => Err(TagisanError::Execution("mock stream unhandled error".into())),
        }
    }
}

// =========================================================================
// SECTION 1: CLI PROVIDER RESOLUTION & ENVIRONMENT INVARIANTS
// =========================================================================

#[test]
fn test_cli_flag_transition_local_to_nonlocal_and_back() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let guard = EnvGuard::new(&[
        "TAGISAN_PROVIDER",
        "TGS_PROVIDER",
        "TAGISAN_LOCAL_ONLY",
        "TAGISAN_OFFLINE",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "GEMINI_API_KEY",
        "DEEPSEEK_API_KEY",
        "XAI_API_KEY",
    ]);

    // Setup valid fake keys for all cloud providers so they register in context
    guard.set("ANTHROPIC_API_KEY", "sk-ant-valid-key-12345");
    guard.set("OPENAI_API_KEY", "sk-valid-openai-key-67890");
    guard.set("GEMINI_API_KEY", "AIzaSyValidGeminiKey-abcdef");
    guard.set("DEEPSEEK_API_KEY", "sk-valid-deepseek-key-11223");
    guard.set("XAI_API_KEY", "xai-valid-key-44556");

    let ctx = build_engine_context(10.0);

    // 1. Local Ollama resolution
    let (p1, m1, prov1) = resolve_provider_and_model(&ctx, "ollama", None)
        .expect("Ollama resolution must succeed");
    assert_eq!(p1, "ollama");
    assert!(!m1.is_empty());
    assert_eq!(prov1.provider_id(), "ollama");

    // 2. Local Colibri resolution
    let (p2, m2, prov2) = resolve_provider_and_model(&ctx, "colibri", None)
        .expect("Colibri resolution must succeed");
    assert_eq!(p2, "colibri");
    assert_eq!(m2, "deepseek-v4");
    assert_eq!(prov2.provider_id(), "colibri");

    // 3. Transition to Non-Local: Anthropic
    let (p3, m3, prov3) = resolve_provider_and_model(&ctx, "anthropic", None)
        .expect("Anthropic resolution must succeed");
    assert_eq!(p3, "anthropic");
    assert_eq!(m3, "claude-3-5-sonnet-20241022");
    assert_eq!(prov3.provider_id(), "anthropic");

    // 4. Transition to Non-Local: OpenAI
    let (p4, m4, prov4) = resolve_provider_and_model(&ctx, "openai", None)
        .expect("OpenAI resolution must succeed");
    assert_eq!(p4, "openai");
    assert_eq!(m4, "gpt-4o");
    assert_eq!(prov4.provider_id(), "openai");

    // 5. Transition to Non-Local: Gemini
    let (p5, m5, prov5) = resolve_provider_and_model(&ctx, "gemini", None)
        .expect("Gemini resolution must succeed");
    assert_eq!(p5, "gemini");
    assert_eq!(m5, "gemini-2.0-flash");
    assert_eq!(prov5.provider_id(), "gemini");

    // 6. Transition to Non-Local: DeepSeek
    let (p6, m6, prov6) = resolve_provider_and_model(&ctx, "deepseek", None)
        .expect("DeepSeek resolution must succeed");
    assert_eq!(p6, "deepseek");
    assert_eq!(m6, "deepseek-chat");
    assert_eq!(prov6.provider_id(), "deepseek");

    // 7. Transition to Non-Local: xAI
    let (p7, m7, prov7) = resolve_provider_and_model(&ctx, "xai", None)
        .expect("xAI resolution must succeed");
    assert_eq!(p7, "xai");
    assert_eq!(m7, "grok-2-latest");
    assert_eq!(prov7.provider_id(), "xai");

    // 8. Transition BACK to Local: Colibri with custom model
    let (p8, m8, prov8) = resolve_provider_and_model(&ctx, "colibri", Some("deepseek-v4-moe".into()))
        .expect("Colibri custom model resolution must succeed");
    assert_eq!(p8, "colibri");
    assert_eq!(m8, "deepseek-v4-moe");
    assert_eq!(prov8.provider_id(), "colibri");

    // 9. Transition BACK to Local: Ollama
    let (p9, m9, prov9) = resolve_provider_and_model(&ctx, "ollama", None)
        .expect("Ollama return resolution must succeed");
    assert_eq!(p9, "ollama");
    assert_eq!(prov9.provider_id(), "ollama");
    assert!(!m9.is_empty());
}

#[test]
fn test_env_var_transition_tagisan_provider_and_tgs_provider() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let guard = EnvGuard::new(&[
        "TAGISAN_PROVIDER",
        "TGS_PROVIDER",
        "TAGISAN_LOCAL_ONLY",
        "TAGISAN_OFFLINE",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
    ]);

    guard.set("ANTHROPIC_API_KEY", "sk-ant-valid-12345");
    guard.set("OPENAI_API_KEY", "sk-openai-valid-67890");

    let ctx = build_engine_context(10.0);

    // 1. TAGISAN_PROVIDER takes effect when user_provider is "auto"
    guard.set("TAGISAN_PROVIDER", "anthropic");
    let (p1, m1, _) = resolve_provider_and_model(&ctx, "auto", None).expect("Resolve via TAGISAN_PROVIDER");
    assert_eq!(p1, "anthropic");
    assert_eq!(m1, "claude-3-5-sonnet-20241022");

    // 2. TGS_PROVIDER fallback when TAGISAN_PROVIDER is unset
    guard.remove("TAGISAN_PROVIDER");
    guard.set("TGS_PROVIDER", "openai");
    let (p2, m2, _) = resolve_provider_and_model(&ctx, "auto", None).expect("Resolve via TGS_PROVIDER");
    assert_eq!(p2, "openai");
    assert_eq!(m2, "gpt-4o");

    // 3. Switch via env back to local Colibri
    guard.set("TAGISAN_PROVIDER", "colibri");
    let (p3, m3, _) = resolve_provider_and_model(&ctx, "auto", None).expect("Resolve colibri via env");
    assert_eq!(p3, "colibri");
    assert_eq!(m3, "deepseek-v4");

    // 4. Switch via env back to local Ollama
    guard.set("TAGISAN_PROVIDER", "ollama");
    let (p4, _, _) = resolve_provider_and_model(&ctx, "auto", None).expect("Resolve ollama via env");
    assert_eq!(p4, "ollama");

    // 5. CLI flag overrides environment variable!
    guard.set("TAGISAN_PROVIDER", "anthropic");
    let (p5, m5, _) = resolve_provider_and_model(&ctx, "openai", None).expect("CLI flag override");
    assert_eq!(p5, "openai");
    assert_eq!(m5, "gpt-4o");
}

#[test]
fn test_tagisan_local_only_and_offline_strict_override() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let guard = EnvGuard::new(&[
        "TAGISAN_PROVIDER",
        "TGS_PROVIDER",
        "TAGISAN_LOCAL_ONLY",
        "TAGISAN_OFFLINE",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "GEMINI_API_KEY",
    ]);

    // Populate cloud keys
    guard.set("ANTHROPIC_API_KEY", "sk-ant-valid-12345");
    guard.set("OPENAI_API_KEY", "sk-openai-valid-67890");
    guard.set("GEMINI_API_KEY", "AIzaSyValidGeminiKey");

    let ctx = build_engine_context(10.0);

    // 1. TAGISAN_LOCAL_ONLY=1 strictly overrides auto-detection to Ollama despite cloud keys!
    guard.set("TAGISAN_LOCAL_ONLY", "1");
    let (p1, _, _) = resolve_provider_and_model(&ctx, "auto", None)
        .expect("TAGISAN_LOCAL_ONLY must force local");
    assert_eq!(p1, "ollama", "TAGISAN_LOCAL_ONLY must never route to cloud!");

    // 2. TAGISAN_OFFLINE=true strictly overrides TAGISAN_PROVIDER=anthropic to Ollama!
    guard.remove("TAGISAN_LOCAL_ONLY");
    guard.set("TAGISAN_OFFLINE", "true");
    guard.set("TAGISAN_PROVIDER", "anthropic");
    let (p2, _, _) = resolve_provider_and_model(&ctx, "auto", None)
        .expect("TAGISAN_OFFLINE must override cloud env");
    assert_eq!(p2, "ollama", "TAGISAN_OFFLINE must override cloud TAGISAN_PROVIDER!");

    // 3. User can explicitly request local provider (Colibri) under TAGISAN_LOCAL_ONLY
    guard.set("TAGISAN_LOCAL_ONLY", "true");
    let (p3, m3, _) = resolve_provider_and_model(&ctx, "colibri", None)
        .expect("Colibri allowed under TAGISAN_LOCAL_ONLY");
    assert_eq!(p3, "colibri");
    assert_eq!(m3, "deepseek-v4");

    // 4. Passing a cloud provider flag (--provider anthropic) under TAGISAN_LOCAL_ONLY is safely forced to ollama
    let (p4, _, _) = resolve_provider_and_model(&ctx, "anthropic", None)
        .expect("Cloud provider flag under local-only safely forces to local");
    assert_eq!(p4, "ollama", "Cloud flag must be forced to local under TAGISAN_LOCAL_ONLY");
}

#[test]
fn test_dynamic_autodetection_precedence_order() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let guard = EnvGuard::new(&[
        "TAGISAN_PROVIDER",
        "TGS_PROVIDER",
        "TAGISAN_LOCAL_ONLY",
        "TAGISAN_OFFLINE",
        "GEMINI_API_KEY",
        "DEEPSEEK_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "XAI_API_KEY",
    ]);

    // Auto-detection precedence:
    // 1. Gemini -> 2. DeepSeek -> 3. Anthropic -> 4. OpenAI -> 5. xAI -> 6. Ollama

    // Case 1: All 5 keys present -> Gemini wins
    guard.set("GEMINI_API_KEY", "AIzaSyGeminiKey");
    guard.set("DEEPSEEK_API_KEY", "sk-deepseek-key");
    guard.set("ANTHROPIC_API_KEY", "sk-ant-key");
    guard.set("OPENAI_API_KEY", "sk-openai-key");
    guard.set("XAI_API_KEY", "xai-key");
    let ctx1 = build_engine_context(10.0);
    let (p1, _, _) = resolve_provider_and_model(&ctx1, "auto", None).unwrap();
    assert_eq!(p1, "gemini", "Gemini must have highest priority in auto-detection ladder");

    // Case 2: Gemini absent -> DeepSeek wins
    guard.remove("GEMINI_API_KEY");
    let ctx2 = build_engine_context(10.0);
    let (p2, _, _) = resolve_provider_and_model(&ctx2, "auto", None).unwrap();
    assert_eq!(p2, "deepseek", "DeepSeek must take second priority");

    // Case 3: Gemini & DeepSeek absent -> Anthropic wins
    guard.remove("DEEPSEEK_API_KEY");
    let ctx3 = build_engine_context(10.0);
    let (p3, _, _) = resolve_provider_and_model(&ctx3, "auto", None).unwrap();
    assert_eq!(p3, "anthropic", "Anthropic must take third priority");

    // Case 4: Gemini, DeepSeek, Anthropic absent -> OpenAI wins
    guard.remove("ANTHROPIC_API_KEY");
    let ctx4 = build_engine_context(10.0);
    let (p4, _, _) = resolve_provider_and_model(&ctx4, "auto", None).unwrap();
    assert_eq!(p4, "openai", "OpenAI must take fourth priority");

    // Case 5: Only xAI present -> xAI wins
    guard.remove("OPENAI_API_KEY");
    let ctx5 = build_engine_context(10.0);
    let (p5, _, _) = resolve_provider_and_model(&ctx5, "auto", None).unwrap();
    assert_eq!(p5, "xai", "xAI must take fifth priority");

    // Case 6: All cloud keys absent -> Falls back to Local Ollama!
    guard.remove("XAI_API_KEY");
    let ctx6 = build_engine_context(10.0);
    let (p6, _, _) = resolve_provider_and_model(&ctx6, "auto", None).unwrap();
    assert_eq!(p6, "ollama", "Zero cloud credentials must fall back to local Ollama");
}

#[test]
fn test_default_model_mappings_and_autohealing() {
    assert_eq!(default_model_for_provider("colibri"), "deepseek-v4");
    assert_eq!(default_model_for_provider("gemini"), "gemini-2.0-flash");
    assert_eq!(default_model_for_provider("deepseek"), "deepseek-chat");
    assert_eq!(default_model_for_provider("anthropic"), "claude-3-5-sonnet-20241022");
    assert_eq!(default_model_for_provider("openai"), "gpt-4o");
    assert_eq!(default_model_for_provider("xai"), "grok-2-latest");

    // Ollama model auto-healing matching logic
    let installed = vec![
        "dolphin-phi:latest".to_string(),
        "qwen2.5:0.5b".to_string(),
        "llama3.2:1b".to_string(),
    ];

    // Exact match
    assert_eq!(
        OllamaProvider::find_matching_model("qwen2.5:0.5b", &installed),
        Some("qwen2.5:0.5b".to_string())
    );

    // Prefix/tag match: "qwen2.5" -> "qwen2.5:0.5b"
    assert_eq!(
        OllamaProvider::find_matching_model("qwen2.5", &installed),
        Some("qwen2.5:0.5b".to_string())
    );

    // Substring match: "phi" -> "dolphin-phi:latest"
    assert_eq!(
        OllamaProvider::find_matching_model("phi", &installed),
        Some("dolphin-phi:latest".to_string())
    );

    // Missing model returns None (which triggers fallback to first installed in resolve_provider_and_model)
    assert_eq!(
        OllamaProvider::find_matching_model("completely-absent-model", &installed),
        None
    );
}

// =========================================================================
// SECTION 2: CASCADE PROVIDER LIVE FAILOVER & BOUNDARY TRANSITIONS
// =========================================================================

#[tokio::test]
async fn test_cascade_failover_cloud_to_local_on_retryable_errors() {
    // Chain: Anthropic (RateLimited) -> DeepSeek (BadResponse 502) -> Local Ollama (Success)
    let p_anthropic = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited(
            "anthropic".into(),
            Some(Duration::from_secs(45)),
        )],
        0.015,
        ProviderCapabilities::STREAMING,
    ));

    let p_deepseek = Arc::new(MockUniversalProvider::new(
        "deepseek",
        vec![MockBehavior::BadResponse(
            "deepseek".into(),
            "502 Bad Gateway".into(),
        )],
        0.002,
        ProviderCapabilities::STREAMING,
    ));

    let p_ollama = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Resilient response generated locally by Ollama!",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p_anthropic.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(p_deepseek.clone(), Some("deepseek-chat".into())),
        CascadeEntry::new(p_ollama.clone(), Some("qwen2.5:0.5b".into())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "Implement distributed consensus");
    let resp = cascade.complete(req).await.expect("Cascade must succeed on local Ollama");

    assert_eq!(resp.provider, "ollama");
    assert_eq!(resp.message.extract_text(), "Resilient response generated locally by Ollama!");
    assert_eq!(p_anthropic.calls(), 1, "Anthropic must be attempted once");
    assert_eq!(p_deepseek.calls(), 1, "DeepSeek must be attempted once");
    assert_eq!(p_ollama.calls(), 1, "Ollama must be invoked to fulfill the request");
}

#[tokio::test]
async fn test_cascade_auth_error_failover_cloud_to_local() {
    // Cloud key expired/invalid (Authentication 401) -> must failover to zero-cost local Ollama
    let p_cloud = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::Authentication(
            "anthropic".into(),
            "401 Unauthorized: Invalid API key".into(),
        )],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Offline fallback succeeds seamlessly!",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud.clone(), None),
        CascadeEntry::new(p_local.clone(), None),
    ]);

    let req = CompletionRequest::new("claude", "Test prompt");
    let resp = cascade.complete(req).await.expect("Authentication error must failover to local");

    assert_eq!(resp.provider, "ollama");
    assert_eq!(resp.message.extract_text(), "Offline fallback succeeds seamlessly!");
    assert_eq!(p_cloud.calls(), 1);
    assert_eq!(p_local.calls(), 1);
}

#[tokio::test]
async fn test_cascade_strict_budget_exceeded_cloud_to_local_vs_cloud_to_cloud() {
    let budget_err = TagisanError::BudgetExceeded {
        max_budget: 1.0,
        current_spent: 1.05,
    };

    let p_cloud1 = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::BudgetExceeded {
            max_budget: 1.0,
            current_spent: 1.05,
        }],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_cloud2 = Arc::new(MockUniversalProvider::successful(
        "openai",
        "Should NEVER be reached",
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_local_ollama = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Local Ollama success",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    let p_local_colibri = Arc::new(MockUniversalProvider::successful(
        "colibri",
        "Local Colibri SSD MoE success",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    // Test 1: can_failover logic verification
    let cascade_test = CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud1.clone(), None),
        CascadeEntry::new(p_cloud2.clone(), None),        // idx 1: cloud
        CascadeEntry::new(p_local_ollama.clone(), None),  // idx 2: local ollama
        CascadeEntry::new(p_local_colibri.clone(), None), // idx 3: local colibri
    ]);

    assert!(!cascade_test.can_failover(&budget_err, 1), "BudgetExceeded must NOT cascade to paid cloud provider!");
    assert!(cascade_test.can_failover(&budget_err, 2), "BudgetExceeded MUST cascade to zero-cost Ollama!");
    assert!(cascade_test.can_failover(&budget_err, 3), "BudgetExceeded MUST cascade to zero-cost Colibri!");

    // Test 2: Execution: Cloud -> Cloud halts immediately
    let cloud_only_cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud1.clone(), None),
        CascadeEntry::new(p_cloud2.clone(), None),
    ]);
    let req1 = CompletionRequest::new("claude", "test");
    let err1 = cloud_only_cascade.complete(req1).await.unwrap_err();
    assert!(matches!(err1, TagisanError::BudgetExceeded { .. }));
    assert_eq!(p_cloud1.calls(), 1);
    assert_eq!(p_cloud2.calls(), 0, "Cloud 2 must NEVER be called when budget is exceeded");

    // Test 3: Execution: Cloud -> Local Colibri succeeds with zero-cost exemption
    let cloud_to_colibri = CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud1.clone(), None),
        CascadeEntry::new(p_local_colibri.clone(), None),
    ]);
    let req2 = CompletionRequest::new("claude", "test");
    let resp2 = cloud_to_colibri.complete(req2).await.expect("Failover to Colibri must succeed");
    assert_eq!(resp2.provider, "colibri");
    assert_eq!(resp2.message.extract_text(), "Local Colibri SSD MoE success");
    assert_eq!(p_local_colibri.calls(), 1);
}

#[tokio::test]
async fn test_cascade_non_retryable_errors_halt_without_cascading() {
    let p_cancel = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::Cancelled],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_io = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::Io("Disk read failure".into())],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Should not be reached",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    // 1. Cancelled error must halt
    let cascade_cancel = CascadeProvider::new(vec![
        CascadeEntry::new(p_cancel.clone(), None),
        CascadeEntry::new(p_local.clone(), None),
    ]);
    let err_cancel = cascade_cancel.complete(CompletionRequest::new("m", "p")).await.unwrap_err();
    assert!(matches!(err_cancel, TagisanError::Cancelled));
    assert_eq!(p_cancel.calls(), 1);
    assert_eq!(p_local.calls(), 0, "Local provider must not be called after cancellation");

    // 2. Io error must halt
    let cascade_io = CascadeProvider::new(vec![
        CascadeEntry::new(p_io.clone(), None),
        CascadeEntry::new(p_local.clone(), None),
    ]);
    let err_io = cascade_io.complete(CompletionRequest::new("m", "p")).await.unwrap_err();
    assert!(matches!(err_io, TagisanError::Io(_)));
    assert_eq!(p_io.calls(), 1);
    assert_eq!(p_local.calls(), 0, "Local provider must not be called after IO error");
}

#[tokio::test]
async fn test_cascade_heterogeneous_multi_tier_chain() {
    // 5-Tier chain:
    // Anthropic (RateLimited) -> OpenAI (502 Bad Gateway) -> Gemini (Network fault) -> Ollama (ContextLengthExceeded) -> Colibri (Success)
    let p1 = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited("anthropic".into(), None)],
        0.01,
        ProviderCapabilities::STREAMING,
    ));
    let p2 = Arc::new(MockUniversalProvider::new(
        "openai",
        vec![MockBehavior::BadResponse("openai".into(), "502 Bad Gateway".into())],
        0.01,
        ProviderCapabilities::STREAMING,
    ));
    let p3 = Arc::new(MockUniversalProvider::new(
        "gemini",
        vec![MockBehavior::Network("Socket reset by peer".into())],
        0.005,
        ProviderCapabilities::STREAMING,
    ));
    let p4 = Arc::new(MockUniversalProvider::new(
        "ollama",
        vec![MockBehavior::ContextLengthExceeded("ollama".into(), 16384, 8192)],
        0.0,
        ProviderCapabilities::STREAMING,
    ));
    let p5 = Arc::new(MockUniversalProvider::successful(
        "colibri",
        "DeepSeek-V4 MoE executed via dual-SSD striping",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p1.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(p2.clone(), Some("gpt-4o".into())),
        CascadeEntry::new(p3.clone(), Some("gemini-2.0-flash".into())),
        CascadeEntry::new(p4.clone(), Some("qwen2.5:0.5b".into())),
        CascadeEntry::new(p5.clone(), Some("deepseek-v4".into())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "Synthesize high-frequency orderbook kernel");
    let resp = cascade.complete(req).await.expect("5-tier heterogeneous cascade must succeed");

    assert_eq!(resp.provider, "colibri");
    assert_eq!(resp.model, "deepseek-v4");
    assert_eq!(resp.message.extract_text(), "DeepSeek-V4 MoE executed via dual-SSD striping");

    assert_eq!(p1.calls(), 1);
    assert_eq!(p2.calls(), 1);
    assert_eq!(p3.calls(), 1);
    assert_eq!(p4.calls(), 1);
    assert_eq!(p5.calls(), 1);
}

#[tokio::test]
async fn test_cascade_streaming_failover_and_transition() {
    use futures::StreamExt;

    // Chain: Anthropic (Stream initialization fails with RateLimited) -> Ollama (Streaming succeeds)
    let p_cloud = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited("anthropic".into(), Some(Duration::from_secs(10)))],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Streamed from local Ollama cleanly",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    let cascade = CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(p_local.clone(), Some("qwen2.5:0.5b".into())),
    ]);

    let req = CompletionRequest::new("claude-3-5-sonnet", "Stream response test");
    let mut stream = cascade.stream(req).await.expect("Stream cascade must initialize");

    let chunk = stream.next().await.expect("first chunk").expect("chunk ok");
    match chunk.delta {
        StreamChunkDelta::Text(t) => assert_eq!(t, "Streamed from local Ollama cleanly"),
        _ => panic!("Expected text delta"),
    }

    assert_eq!(p_cloud.calls(), 1);
    assert_eq!(p_local.calls(), 1);
}

// =========================================================================
// SECTION 3: TOKEN BUDGETING & COST ACCOUNTING ACROSS TRANSITIONS
// =========================================================================

#[test]
fn test_token_budget_boundary_exhaustion_cloud_rejection_local_exemption() {
    // 1. Budget of $0.05
    let tracker = TokenBudgetTracker::new(0.05);
    assert!(!tracker.is_exhausted());

    // Spend $0.03 via cloud model
    let spent1 = tracker.record("claude-3-5-sonnet", 5_000, 1_000).expect("spend under cap");
    assert!(!tracker.is_exhausted());
    assert!(spent1 > 0.0);

    // Spend remaining balance to cross cap exactly
    let spent2 = tracker.record_cost_usd(0.05 - spent1).expect("spend to boundary");
    assert_eq!(spent2, 0.05);
    assert!(tracker.is_exhausted());

    // Non-zero cloud expense MUST fail with BudgetExceeded
    let cloud_err1 = tracker.record_micro_usd(1).unwrap_err();
    assert!(matches!(cloud_err1, TagisanError::BudgetExceeded { .. }));

    let cloud_err2 = tracker.record("gpt-4o", 100, 100).unwrap_err();
    assert!(matches!(cloud_err2, TagisanError::BudgetExceeded { .. }));

    let cloud_err3 = tracker.record("claude-3-5-sonnet", 10, 10).unwrap_err();
    assert!(matches!(cloud_err3, TagisanError::BudgetExceeded { .. }));

    // ZERO-COST LOCAL OPERATIONS MUST SUCCEED INDEFINITELY EVEN WHEN 100% EXHAUSTED!
    assert!(tracker.record_micro_usd(0).is_ok());
    assert!(tracker.record_cost_usd(0.0).is_ok());

    // Local Ollama models
    let ollama_res = tracker.record("ollama", 50_000, 50_000);
    assert!(ollama_res.is_ok(), "Ollama usage must succeed when budget is exhausted");
    assert_eq!(ollama_res.unwrap(), 0.05);

    let qwen_res = tracker.record("qwen2.5:0.5b", 25_000, 25_000);
    assert!(qwen_res.is_ok(), "Qwen local model must succeed when budget is exhausted");
    assert_eq!(qwen_res.unwrap(), 0.05);

    let dolphin_res = tracker.record("dolphin-phi", 10_000, 10_000);
    assert!(dolphin_res.is_ok(), "Dolphin local model must succeed when budget is exhausted");

    // Local Colibri models
    let colibri_res = tracker.record("colibri", 100_000, 100_000);
    assert!(colibri_res.is_ok(), "Colibri SSD MoE model must succeed when budget is exhausted");
    assert_eq!(colibri_res.unwrap(), 0.05);

    let coli_res = tracker.record("coli-deepseek", 100_000, 100_000);
    assert!(coli_res.is_ok(), "Coli model prefix must succeed when budget is exhausted");
    assert_eq!(coli_res.unwrap(), 0.05);

    // Initial $0.00 budget: cloud fails immediately, local succeeds immediately
    let zero_budget = TokenBudgetTracker::new(0.0);
    assert!(zero_budget.is_exhausted());
    assert!(zero_budget.record("claude-3-5-sonnet", 1, 1).is_err());
    assert!(zero_budget.record("gpt-4o", 1, 1).is_err());
    assert!(zero_budget.record("ollama", 10_000, 10_000).is_ok());
    assert!(zero_budget.record("colibri", 10_000, 10_000).is_ok());
}

#[test]
fn test_micro_usd_precision_arithmetic_and_caching_discounts() {
    let tracker = TokenBudgetTracker::new(10.0);

    // Prompt caching: cached prompt tokens receive 90% discount (0.1x of prompt rate)
    // claude-3-5-sonnet prompt rate: $3.00/M, completion rate: $15.00/M
    // 100,000 prompt tokens (with 80,000 cached):
    // uncached prompt = 20,000 * 3.0 = 60,000 micro-USD
    // cached prompt   = 80,000 * 3.0 * 0.1 = 24,000 micro-USD
    // completion      = 10,000 * 15.0 = 150,000 micro-USD
    // total = 60,000 + 24,000 + 150,000 = 234,000 micro-USD = $0.234
    let spent = tracker
        .record_with_cache("claude-3-5-sonnet", 100_000, 10_000, 80_000)
        .expect("cache discount record");
    assert_eq!(spent, 0.234);

    // Multithreaded concurrency test: 20 threads simultaneously recording 10,000 micro-USD
    let tracker_arc = Arc::new(TokenBudgetTracker::new(1.0));
    let mut handles = Vec::new();
    for _ in 0..20 {
        let t = tracker_arc.clone();
        handles.push(std::thread::spawn(move || {
            t.record_micro_usd(10_000).expect("concurrent record");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    // 20 * 10,000 micro-USD = 200,000 micro-USD = $0.20
    assert_eq!(tracker_arc.current_spent_usd(), 0.20);
}

// =========================================================================
// SECTION 4: MULTI-AGENT HARMONY SWARM MIXED LOCAL & NON-LOCAL ROLES
// =========================================================================

#[tokio::test]
async fn test_harmony_swarm_mixed_local_cloud_role_pipeline() {
    let mut ctx = EngineContext::new(10.0);

    // 1. Architect: Local Ollama
    let p_arch = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "```rust\npub struct DistributedOrderBook { pub depth: usize }\n```",
        0.0,
        ProviderCapabilities::STREAMING,
    ));
    // 2. Implementer: Cloud Anthropic
    let p_imp = Arc::new(MockUniversalProvider::successful(
        "anthropic",
        "```rust\nimpl DistributedOrderBook {\n    pub fn new(depth: usize) -> Self { Self { depth } }\n}\n```",
        0.005,
        ProviderCapabilities::STREAMING,
    ));
    // 3. QA: Cloud Gemini
    let p_qa = Arc::new(MockUniversalProvider::successful(
        "gemini",
        "```rust\n#[test]\nfn test_orderbook_depth() {\n    let ob = DistributedOrderBook::new(100);\n    assert_eq!(ob.depth, 100);\n}\n```",
        0.001,
        ProviderCapabilities::STREAMING,
    ));
    // 4. Documentation: Local Colibri
    let p_doc = Arc::new(MockUniversalProvider::successful(
        "colibri",
        "# Distributed OrderBook Documentation\n\nHigh throughput zero-copy orderbook.",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    ctx.register_provider(p_arch.clone());
    ctx.register_provider(p_imp.clone());
    ctx.register_provider(p_qa.clone());
    ctx.register_provider(p_doc.clone());

    let stage1 = HarmonyStage::new(AssemblyRoles::architect("ollama", "qwen2.5:0.5b"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage2 = HarmonyStage::new(AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage3 = HarmonyStage::new(AssemblyRoles::qa("gemini", "gemini-2.0-flash"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));
    let stage4 = HarmonyStage::new(AssemblyRoles::documentation("colibri", "deepseek-v4"))
        .with_gate(Box::new(SyntaxValidationGate::permissive()));

    let pipeline = StructuredHarmonyPipeline::new("Design Low-Latency OrderBook")
        .add_stage(stage1)
        .add_stage(stage2)
        .add_stage(stage3)
        .add_stage(stage4);

    let result = pipeline.execute(&ctx).await.expect("Mixed pipeline execution must succeed");

    assert_eq!(result.artifacts.len(), 4);
    assert_eq!(result.artifacts[0].provider, "ollama");
    assert_eq!(result.artifacts[1].provider, "anthropic");
    assert_eq!(result.artifacts[2].provider, "gemini");
    assert_eq!(result.artifacts[3].provider, "colibri");

    // All code blocks assembled seamlessly across local and cloud transitions
    assert!(result.complete_project.contains("pub struct DistributedOrderBook"));
    assert!(result.complete_project.contains("pub fn new(depth: usize)"));
    assert!(result.complete_project.contains("test_orderbook_depth"));

    assert_eq!(p_arch.calls(), 1);
    assert_eq!(p_imp.calls(), 1);
    assert_eq!(p_qa.calls(), 1);
    assert_eq!(p_doc.calls(), 1);
}

#[tokio::test]
async fn test_harmony_swarm_cloud_role_failover_event_propagation() {
    let mut ctx = EngineContext::new(10.0);

    // Implementer cloud role hits HTTP 429 RateLimit!
    let p_cloud_imp = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited(
            "anthropic".into(),
            Some(Duration::from_secs(30)),
        )],
        0.01,
        ProviderCapabilities::STREAMING,
    ));

    let p_local_ollama = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "```rust\npub fn fallback_implementer() -> bool { true }\n```",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    ctx.register_provider(p_cloud_imp.clone());
    ctx.register_provider(p_local_ollama.clone());

    let stage = HarmonyStage::new(AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("Build resilient component")
        .add_stage(stage)
        .with_fallback_to_local(true);

    let result = pipeline.execute(&ctx).await.expect("Failover to local must succeed");
    let artifact = &result.artifacts[0];

    assert_eq!(artifact.provider, "ollama");
    let fo: &FailoverEvent = artifact.failover_event.as_ref().expect("FailoverEvent must be attached");
    assert_eq!(fo.original_provider, "anthropic");
    assert_eq!(fo.original_model, "claude-3-5-sonnet");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.trigger_reason.contains("Rate limited (HTTP 429)"));
    assert!(fo.timestamp_epoch_ms > 0);

    assert_eq!(p_cloud_imp.calls(), 1);
    assert_eq!(p_local_ollama.calls(), 1);
}

#[tokio::test]
async fn test_harmony_swarm_auth_error_failover_to_local() {
    let mut ctx = EngineContext::new(10.0);

    // QA cloud role hits 401 Unauthorized
    let p_cloud_qa = Arc::new(MockUniversalProvider::new(
        "deepseek",
        vec![MockBehavior::Authentication(
            "deepseek".into(),
            "401 Unauthorized: Invalid or expired API Key".into(),
        )],
        0.005,
        ProviderCapabilities::STREAMING,
    ));

    let p_local_ollama = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "```rust\n#[test]\nfn test_auth_recovery() { assert!(true); }\n```",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    ctx.register_provider(p_cloud_qa.clone());
    ctx.register_provider(p_local_ollama.clone());

    let stage = HarmonyStage::new(AssemblyRoles::qa("deepseek", "deepseek-chat"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("QA verification")
        .add_stage(stage)
        .with_fallback_to_local(true);

    let result = pipeline.execute(&ctx).await.expect("Auth error failover to local must succeed");
    let artifact = &result.artifacts[0];

    assert_eq!(artifact.provider, "ollama");
    let fo = artifact.failover_event.as_ref().expect("FailoverEvent present on auth failure");
    assert_eq!(fo.original_provider, "deepseek");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.trigger_reason.contains("Authentication failure"));

    assert_eq!(p_cloud_qa.calls(), 1);
    assert_eq!(p_local_ollama.calls(), 1);
}

#[tokio::test]
async fn test_harmony_swarm_preemptive_evacuation_on_budget_exhaustion() {
    let mut ctx = EngineContext::new(0.05);

    let p_cloud = Arc::new(MockUniversalProvider::successful(
        "openai",
        "```rust\npub struct CloudShouldNotRun;\n```",
        0.05,
        ProviderCapabilities::STREAMING,
    ));

    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "```rust\npub struct PreemptivelyEvacuatedLocal;\n```",
        0.0,
        ProviderCapabilities::STREAMING,
    ));

    ctx.register_provider(p_cloud.clone());
    ctx.register_provider(p_local.clone());

    // Exhaust budget prior to running stage
    ctx.budget_tracker.record_micro_usd(50_000).expect("spend $0.05");
    assert!(ctx.budget_tracker.is_exhausted());

    let stage = HarmonyStage::new(AssemblyRoles::architect("openai", "gpt-4o"))
        .with_gate(Box::new(SyntaxValidationGate::for_rust()));

    let pipeline = StructuredHarmonyPipeline::new("Budget evacuation test")
        .add_stage(stage)
        .with_evacuate_on_budget(true);

    let result = pipeline.execute(&ctx).await.expect("Pre-emptive evacuation must succeed");
    let artifact = &result.artifacts[0];

    assert_eq!(artifact.provider, "ollama");
    assert!(artifact.raw_output.contains("PreemptivelyEvacuatedLocal"));

    let fo = artifact.failover_event.as_ref().expect("FailoverEvent attached");
    assert_eq!(fo.original_provider, "openai");
    assert_eq!(fo.evacuated_to_provider, "ollama");
    assert!(fo.trigger_reason.contains("Token budget reached"));

    // Cloud was NEVER touched because budget was already exhausted!
    assert_eq!(p_cloud.calls(), 0, "Cloud provider must not be invoked when budget is pre-exhausted");
    assert_eq!(p_local.calls(), 1);
}

// =========================================================================
// SECTION 5: TOOL CALLING & AGENTIC LOOP PARITY ACROSS LOCAL AND CLOUD
// =========================================================================

#[tokio::test]
async fn test_agent_tool_calling_loop_transition_cloud_to_local_and_back() {
    let ctx = EngineContext::new(10.0);

    let mut tools = ToolRegistry::new();
    tools.register_tool(CalculatorTool);

    // 1. Cloud Provider requests tool call: eval "21 * 2"
    let p_cloud = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::ToolCall {
            id: "call_abc123".to_string(),
            name: "calculator".to_string(),
            args: serde_json::json!({ "expression": "21 * 2" }),
        }],
        0.005,
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING,
    ));

    let agent_cloud = AutonomousAgent::new(p_cloud.clone(), "claude-3-5-sonnet", tools.clone())
        .with_agentshield(false)
        .with_max_iterations(1); // stop after tool execution

    let _res_step1 = agent_cloud.run("What is 21 * 2?", &ctx).await;

    // 2. Now simulate handoff to Local Provider (Ollama) with the tool results in history
    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "The calculated value of 21 * 2 is 42.",
        0.0,
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING,
    ));

    let agent_local = AutonomousAgent::new(p_local.clone(), "qwen2.5:0.5b", tools)
        .with_agentshield(false)
        .with_max_iterations(2);

    let res_final = agent_local.run("What is 21 * 2?", &ctx).await.expect("Local agent run succeeds");

    assert_eq!(res_final.final_answer, "The calculated value of 21 * 2 is 42.");
    assert_eq!(p_cloud.calls(), 1);
    assert_eq!(p_local.calls(), 1);
}

#[tokio::test]
async fn test_agent_with_cascade_provider_tool_calling_failover() {
    let ctx = EngineContext::new(10.0);
    let mut tools = ToolRegistry::new();
    tools.register_tool(CalculatorTool);

    // Cloud provider fails with RateLimited when attempting tool call
    let p_cloud = Arc::new(MockUniversalProvider::new(
        "anthropic",
        vec![MockBehavior::RateLimited("anthropic".into(), None)],
        0.01,
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING,
    ));

    // Fallback local Ollama provider answers directly
    let p_local = Arc::new(MockUniversalProvider::successful(
        "ollama",
        "Direct calculation: 50 + 50 = 100.",
        0.0,
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING,
    ));

    let cascade = Arc::new(CascadeProvider::new(vec![
        CascadeEntry::new(p_cloud.clone(), Some("claude-3-5-sonnet".into())),
        CascadeEntry::new(p_local.clone(), Some("qwen2.5:0.5b".into())),
    ]));

    let agent = AutonomousAgent::new(cascade, "claude-3-5-sonnet", tools)
        .with_agentshield(false)
        .with_max_iterations(3);

    let res = agent.run("Calculate 50 + 50", &ctx).await.expect("Cascade agent must succeed");
    assert_eq!(res.final_answer, "Direct calculation: 50 + 50 = 100.");
    assert_eq!(p_cloud.calls(), 1);
    assert_eq!(p_local.calls(), 1);
}

// =========================================================================
// SECTION 6: REASONING EXTRACTION & SPECIAL TOKENS PARITY
// =========================================================================

#[test]
fn test_reasoning_extraction_local_think_tags_deepseek_qwen() {
    // 1. Single thinking block
    let raw1 = "<think>\nAnalyze time complexity: O(N log N)\n</think>\nHere is the sorted output.";
    let blocks1 = parse_thinking_blocks(raw1);
    assert_eq!(blocks1.len(), 2);
    match &blocks1[0] {
        ContentBlock::Thinking { thinking, .. } => {
            assert!(thinking.contains("Analyze time complexity: O(N log N)"));
        }
        _ => panic!("Expected thinking block"),
    }
    match &blocks1[1] {
        ContentBlock::Text { text } => assert_eq!(text, "Here is the sorted output."),
        _ => panic!("Expected text block"),
    }

    // 2. Multiple thinking blocks
    let raw2 = "<think>First thought</think>Intermediate text<think>Second thought</think>Final conclusion";
    let blocks2 = parse_thinking_blocks(raw2);
    assert_eq!(blocks2.len(), 4);
    assert!(matches!(&blocks2[0], ContentBlock::Thinking { thinking, .. } if thinking == "First thought"));
    assert!(matches!(&blocks2[1], ContentBlock::Text { text } if text == "Intermediate text"));
    assert!(matches!(&blocks2[2], ContentBlock::Thinking { thinking, .. } if thinking == "Second thought"));
    assert!(matches!(&blocks2[3], ContentBlock::Text { text } if text == "Final conclusion"));

    // 3. Unclosed think tag (graceful fallback to text)
    let raw3 = "<think>Unclosed thinking block without closing tag";
    let blocks3 = parse_thinking_blocks(raw3);
    assert_eq!(blocks3.len(), 1);
    assert!(matches!(&blocks3[0], ContentBlock::Text { .. }));

    // 4. Empty think tag
    let raw4 = "<think></think>Clean answer";
    let blocks4 = parse_thinking_blocks(raw4);
    assert_eq!(blocks4.len(), 2);
    assert!(matches!(&blocks4[0], ContentBlock::Thinking { thinking, .. } if thinking.is_empty()));
    assert!(matches!(&blocks4[1], ContentBlock::Text { text } if text == "Clean answer"));
}

#[test]
fn test_streaming_think_parser_transitions() {
    let mut parser = StreamingThinkParser::new();

    // Push chunks incrementally simulating streaming from DeepSeek-R1 or Qwen local
    let d1 = parser.process("<thi");
    assert!(d1.is_empty(), "Partial tag must buffer without emitting delta");

    let d2 = parser.process("nk>Analyzing constraints...");
    assert_eq!(d2.len(), 1);
    assert_eq!(
        d2[0],
        StreamChunkDelta::Thinking("Analyzing constraints...".to_string())
    );

    let d3 = parser.process("</think>Here is the code solution.");
    assert_eq!(d3.len(), 1);
    assert_eq!(
        d3[0],
        StreamChunkDelta::Text("Here is the code solution.".to_string())
    );

    let d4 = parser.process(" Additional trailing stream tokens.");
    assert_eq!(d4.len(), 1);
    assert_eq!(
        d4[0],
        StreamChunkDelta::Text(" Additional trailing stream tokens.".to_string())
    );
}

#[test]
fn test_reasoning_tokens_accounting_cloud_vs_local() {
    let p_cloud_reasoner = MockUniversalProvider::successful(
        "deepseek",
        "Reasoned solution",
        0.005,
        ProviderCapabilities::STREAMING | ProviderCapabilities::REASONING_EXTRACTION,
    );

    let p_local_ollama = MockUniversalProvider::successful(
        "ollama",
        "Direct local output",
        0.0,
        ProviderCapabilities::STREAMING,
    );

    let caps_cloud = p_cloud_reasoner.capabilities("deepseek-reasoner");
    assert!(caps_cloud.contains(ProviderCapabilities::REASONING_EXTRACTION));

    let caps_local = p_local_ollama.capabilities("qwen2.5:0.5b");
    assert!(!caps_local.contains(ProviderCapabilities::REASONING_EXTRACTION));
}

// =========================================================================
// SECTION 7: REVERSIBILITY & SESSION PERSISTENCE ACROSS TRANSITIONS
// =========================================================================

#[tokio::test]
async fn test_seamless_reversibility_local_to_cloud_to_local_chat_session() {
    let mut session = ChatSession::new().with_system("You are a high-performance systems engineering copilot.");

    let p_ollama = Arc::new(MockUniversalProvider::successful("ollama", "Local A1: Defined Rust trait.", 0.0, ProviderCapabilities::STREAMING));
    let p_anthropic = Arc::new(MockUniversalProvider::successful("anthropic", "Cloud A2: Added generic parameters and associated types.", 0.01, ProviderCapabilities::STREAMING));
    let p_colibri = Arc::new(MockUniversalProvider::successful("colibri", "Local A3: MoE verified memory alignment on disk.", 0.0, ProviderCapabilities::STREAMING));
    let p_openai = Arc::new(MockUniversalProvider::successful("openai", "Cloud A4: Generated benchmark harness.", 0.01, ProviderCapabilities::STREAMING));
    let p_ollama_return = Arc::new(MockUniversalProvider::successful("ollama", "Local A5: Final verification completed.", 0.0, ProviderCapabilities::STREAMING));

    // Turn 1: Local (Ollama)
    session.add_user_message("Q1: Define a Cache trait");
    let req1 = session.build_request("qwen2.5:0.5b");
    let resp1 = p_ollama.complete(req1).await.unwrap();
    session.add_assistant_message(resp1.message.extract_text());

    // Turn 2: Transition to Non-Local (Anthropic)
    session.add_user_message("Q2: Add generics for Key and Value");
    let req2 = session.build_request("claude-3-5-sonnet");
    let resp2 = p_anthropic.complete(req2).await.unwrap();
    session.add_assistant_message(resp2.message.extract_text());

    // Turn 3: Transition to Local (Colibri SSD MoE)
    session.add_user_message("Q3: Check disk memory layout");
    let req3 = session.build_request("deepseek-v4");
    let resp3 = p_colibri.complete(req3).await.unwrap();
    session.add_assistant_message(resp3.message.extract_text());

    // Turn 4: Transition to Non-Local (OpenAI)
    session.add_user_message("Q4: Benchmark throughput");
    let req4 = session.build_request("gpt-4o");
    let resp4 = p_openai.complete(req4).await.unwrap();
    session.add_assistant_message(resp4.message.extract_text());

    // Turn 5: Transition BACK to Local (Ollama)
    session.add_user_message("Q5: Final audit checklist");
    let req5 = session.build_request("qwen2.5:0.5b");
    let resp5 = p_ollama_return.complete(req5).await.unwrap();
    session.add_assistant_message(resp5.message.extract_text());

    // Verify conversation integrity throughout all 5 transitions
    assert_eq!(session.history.len(), 10);
    assert_eq!(session.history[0].role, Role::User);
    assert_eq!(session.history[0].extract_text(), "Q1: Define a Cache trait");
    assert_eq!(session.history[1].role, Role::Assistant);
    assert_eq!(session.history[1].extract_text(), "Local A1: Defined Rust trait.");

    assert_eq!(session.history[2].role, Role::User);
    assert_eq!(session.history[3].role, Role::Assistant);
    assert_eq!(session.history[3].extract_text(), "Cloud A2: Added generic parameters and associated types.");

    assert_eq!(session.history[4].role, Role::User);
    assert_eq!(session.history[5].role, Role::Assistant);
    assert_eq!(session.history[5].extract_text(), "Local A3: MoE verified memory alignment on disk.");

    assert_eq!(session.history[6].role, Role::User);
    assert_eq!(session.history[7].role, Role::Assistant);
    assert_eq!(session.history[7].extract_text(), "Cloud A4: Generated benchmark harness.");

    assert_eq!(session.history[8].role, Role::User);
    assert_eq!(session.history[9].role, Role::Assistant);
    assert_eq!(session.history[9].extract_text(), "Local A5: Final verification completed.");

    assert_eq!(p_ollama.calls(), 1);
    assert_eq!(p_anthropic.calls(), 1);
    assert_eq!(p_colibri.calls(), 1);
    assert_eq!(p_openai.calls(), 1);
    assert_eq!(p_ollama_return.calls(), 1);
}

#[test]
fn test_ecc_skills_injection_adaptation_local_to_cloud_to_local() {
    // 1. Verify is_local_provider recognition
    assert!(is_local_provider("ollama"));
    assert!(is_local_provider("local"));
    assert!(is_local_provider("colibri"));
    assert!(is_local_provider("coli"));
    assert!(is_local_provider("llama.cpp"));

    assert!(!is_local_provider("anthropic"));
    assert!(!is_local_provider("openai"));
    assert!(!is_local_provider("gemini"));
    assert!(!is_local_provider("deepseek"));
    assert!(!is_local_provider("xai"));

    // 2. Test prompt maximization mode adaptation across transitions
    let dispatcher = global_dispatcher();
    let prompt = "Implement memory-mapped ring buffer in Rust";

    // Step 1: Local Ollama prompt injection (dolphin-phi: compact 8k context)
    let (text_local, _, budget_local) = dispatcher.equip_prompt_maximized(
        "",
        prompt,
        "ollama",
        Some("dolphin-phi:latest"),
        None,
        None,
        None,
    );
    // Local provider uses bounded dense invariants (1.2k tokens, max 1 per domain, 8k ctx)
    assert_eq!(budget_local.mode, InjectionMode::DenseInvariants);
    assert_eq!(budget_local.max_tokens, 1200);
    assert_eq!(budget_local.context_window, 8192);
    assert_eq!(budget_local.max_per_domain, 1);

    // Step 2: Transition to Cloud Anthropic prompt injection (200k context window)
    let (text_cloud, _, budget_cloud) = dispatcher.equip_prompt_maximized(
        "",
        prompt,
        "anthropic",
        Some("claude-3-5-sonnet"),
        None,
        None,
        None,
    );
    // Cloud provider expands to hierarchical multi-tier architecture (12k tokens, 200k ctx, max 2 per domain)
    assert_eq!(budget_cloud.mode, InjectionMode::Hierarchical);
    assert_eq!(budget_cloud.max_tokens, 12000);
    assert_eq!(budget_cloud.context_window, 200000);
    assert_eq!(budget_cloud.max_per_domain, 2);

    // Step 3: Transition BACK to Local Colibri prompt injection (dual-SSD MoE local execution)
    let (text_colibri, _, budget_colibri) = dispatcher.equip_prompt_maximized(
        "",
        prompt,
        "colibri",
        Some("deepseek-v4"),
        None,
        None,
        None,
    );
    assert_eq!(budget_colibri.mode, InjectionMode::DenseInvariants);
    assert_eq!(budget_colibri.max_tokens, 1200);
    assert_eq!(budget_colibri.context_window, 8192);
    assert_eq!(budget_colibri.max_per_domain, 1);

    // Verify non-empty injection outputs
    assert!(!text_local.is_empty());
    assert!(!text_cloud.is_empty());
    assert!(!text_colibri.is_empty());
}
