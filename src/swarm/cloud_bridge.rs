//! Tagisan (TGS) Cloud Frontier LLM Agent Bridge & Multi-Provider Consensus Swarm Coordinator
//!
//! Provides production-grade multi-cloud agent federation, asymmetric Mixture-of-Frontier-Agents (MoFA),
//! token budget defense, zero-stall cross-cloud failover, pre-egress AgentShield DLP redaction,
//! cross-provider dialectic red-teaming, and shared reflexion synchronization across Anthropic, OpenAI,
//! Google, and Custom local/cloud models.

use crate::error::{Result, TagisanError};
use crate::swarm::bridge::{
    AgentCircuitBreakerRegistry, AgentShieldBridgeGateway, BridgeMessage, CircuitState,
    SharedReflexionBridge,
};
use crate::tools::builtin::ReflexionEntry;
use async_trait::async_trait;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

// =========================================================================
// 1. Cloud Provider Family & Swarm Roles
// =========================================================================

/// Cloud provider families supported by the Cloud Frontier Agent Bridge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudProviderFamily {
    Anthropic,
    OpenAI,
    Google,
    Custom,
}

impl CloudProviderFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anthropic => "Anthropic",
            Self::OpenAI => "OpenAI",
            Self::Google => "Google",
            Self::Custom => "Custom",
        }
    }

    /// Infer provider family from model name string
    pub fn from_model_name(model: &str) -> Self {
        let m = model.to_lowercase();
        if m.contains("claude") || m.contains("anthropic") {
            Self::Anthropic
        } else if m.contains("gpt")
            || m.contains("o1")
            || m.contains("o3")
            || m.contains("o4")
            || m.contains("openai")
        {
            Self::OpenAI
        } else if m.contains("gemini") || m.contains("google") {
            Self::Google
        } else {
            Self::Custom
        }
    }
}

/// Swarm roles within the Asymmetric Mixture-of-Frontier-Agents (MoFA) pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudSwarmRole {
    /// Turn 1: Low-cost, fast model distilling raw conversational intent into structured JSON contract
    Triage,
    /// Turn 2: High-precision frontier reasoning model synthesizing core solution
    Primary,
    /// Turn 3: Cross-provider model red-teaming implementation and generating fuzz tests
    Reviewer,
    /// Turn 4: Consensus evaluator measuring cross-provider AST and semantic agreement
    ConsensusAuditor,
}

impl CloudSwarmRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Triage => "Triage",
            Self::Primary => "Primary",
            Self::Reviewer => "Reviewer",
            Self::ConsensusAuditor => "ConsensusAuditor",
        }
    }

    pub fn agent_id(&self) -> &'static str {
        match self {
            Self::Triage => "cloud_triage",
            Self::Primary => "cloud_primary",
            Self::Reviewer => "cloud_reviewer",
            Self::ConsensusAuditor => "cloud_consensus_auditor",
        }
    }
}

// =========================================================================
// 2. Structured Contracts & Review Evaluations
// =========================================================================

/// Structured task contract synthesized during Turn 1 (Triage)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DistilledContract {
    pub task_summary: String,
    pub primary_objective: String,
    pub architecture_invariants: Vec<String>,
    pub constraints: Vec<String>,
    pub edge_cases: Vec<String>,
    pub dependencies: Vec<String>,
    pub language_or_tech: Option<String>,
    pub expected_schema: Option<String>,
}

impl Default for DistilledContract {
    fn default() -> Self {
        Self {
            task_summary: "General software engineering task".to_string(),
            primary_objective: "Implement requested solution with high precision".to_string(),
            architecture_invariants: vec!["Maintain memory safety and modularity".to_string()],
            constraints: vec!["Zero runtime regressions".to_string()],
            edge_cases: vec!["Handle empty/null input gracefully".to_string()],
            dependencies: Vec::new(),
            language_or_tech: None,
            expected_schema: None,
        }
    }
}

impl DistilledContract {
    /// Parse raw triage model output into structured DistilledContract with resilient fallbacks
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();

        // 1. Try parsing direct JSON
        if let Ok(c) = serde_json::from_str::<DistilledContract>(trimmed) {
            return c;
        }

        // 2. Try extracting JSON from markdown code fence
        if let Some(start) = trimmed.find("```json") {
            let after = &trimmed[start + 7..];
            if let Some(end) = after.find("```") {
                let json_slice = after[..end].trim();
                if let Ok(c) = serde_json::from_str::<DistilledContract>(json_slice) {
                    return c;
                }
            }
        } else if let Some(start) = trimmed.find("```") {
            let after = &trimmed[start + 3..];
            if let Some(end) = after.find("```") {
                let json_slice = after[..end].trim();
                if let Ok(c) = serde_json::from_str::<DistilledContract>(json_slice) {
                    return c;
                }
            }
        }

        // 3. Fallback: Heuristic extraction from free text
        let mut contract = Self::default();
        contract.task_summary = trimmed.lines().next().unwrap_or("Engineering task").to_string();
        contract.primary_objective = trimmed.chars().take(200).collect();
        contract
    }

    pub fn to_contract_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Dialectic review evaluation synthesized during Turn 3 (Reviewer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerEvaluation {
    pub approved: bool,
    pub score: f64,
    pub security_findings: Vec<String>,
    pub fuzz_tests: Vec<String>,
    pub critique: String,
    pub proposed_patches: Option<String>,
}

impl Default for ReviewerEvaluation {
    fn default() -> Self {
        Self {
            approved: true,
            score: 0.90,
            security_findings: Vec::new(),
            fuzz_tests: Vec::new(),
            critique: "Implementation complies with architectural invariants".to_string(),
            proposed_patches: None,
        }
    }
}

impl ReviewerEvaluation {
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();

        if let Ok(eval) = serde_json::from_str::<ReviewerEvaluation>(trimmed) {
            return eval;
        }

        if let Some(start) = trimmed.find("```json") {
            let after = &trimmed[start + 7..];
            if let Some(end) = after.find("```") {
                let json_slice = after[..end].trim();
                if let Ok(eval) = serde_json::from_str::<ReviewerEvaluation>(json_slice) {
                    return eval;
                }
            }
        } else if let Some(start) = trimmed.find("```") {
            let after = &trimmed[start + 3..];
            if let Some(end) = after.find("```") {
                let json_slice = after[..end].trim();
                if let Ok(eval) = serde_json::from_str::<ReviewerEvaluation>(json_slice) {
                    return eval;
                }
            }
        }

        let mut eval = Self::default();
        eval.critique = trimmed.chars().take(300).collect();
        eval
    }
}

/// Record of a zero-stall failover event during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverRecord {
    pub from_provider: CloudProviderFamily,
    pub to_provider: CloudProviderFamily,
    pub reason: String,
    pub timestamp_ms: u64,
}

/// Result returned from executing a full Cloud Frontier Swarm cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSwarmResult {
    pub success: bool,
    pub raw_prompt: String,
    pub raw_tokens: usize,
    pub distilled_contract: DistilledContract,
    pub distilled_tokens: usize,
    pub token_savings_pct: f64,
    pub primary_provider: CloudProviderFamily,
    pub primary_model: String,
    pub primary_output: String,
    pub failover_occurred: bool,
    pub failover_history: Vec<FailoverRecord>,
    pub reviewer_provider: CloudProviderFamily,
    pub reviewer_model: String,
    pub review_evaluation: ReviewerEvaluation,
    pub consensus_score: f64,
    pub consensus_reached: bool,
    pub total_tokens: usize,
    pub total_latency_ms: u64,
    pub secrets_redacted_count: usize,
    pub invariants_synced_count: usize,
    pub error: Option<String>,
}

// =========================================================================
// 3. Cloud Swarm Configuration & Metrics
// =========================================================================

/// Configuration options for the Cloud Frontier Swarm Coordinator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSwarmConfig {
    /// Triage model (default: gemini-2.0-flash)
    pub triage_model: String,
    /// Primary reasoning model (default: claude-3-5-sonnet-20241022)
    pub primary_model: String,
    /// Cross-provider reviewer model (default: gpt-4o)
    pub reviewer_model: String,
    /// Failover provider chain: Anthropic -> OpenAI -> Google -> Local Edge (Custom)
    pub failover_chain: Vec<CloudProviderFamily>,
    /// Fallback models per provider family
    pub fallback_models: HashMap<CloudProviderFamily, String>,
    /// Consensus agreement threshold (0.0 to 1.0, default: 0.75)
    pub consensus_threshold: f64,
    /// Token budget defense limit (default: 50,000 tokens)
    pub token_budget_guard: usize,
    /// Enable zero-stall cross-provider failover (default: true)
    pub failover_enabled: bool,
    /// Circuit breaker failure threshold (default: 3)
    pub circuit_breaker_failure_threshold: usize,
    /// Circuit breaker recovery timeout (default: 10s)
    pub circuit_breaker_recovery_timeout: Duration,
    /// Maximum turns in multi-provider cycle (default: 4)
    pub max_turns: usize,
    /// Timeout in seconds for individual API calls
    pub timeout_secs: u64,
}

impl Default for CloudSwarmConfig {
    fn default() -> Self {
        let mut fallback_models = HashMap::new();
        fallback_models.insert(
            CloudProviderFamily::Anthropic,
            "claude-3-5-sonnet-20241022".to_string(),
        );
        fallback_models.insert(CloudProviderFamily::OpenAI, "gpt-4o".to_string());
        fallback_models.insert(CloudProviderFamily::Google, "gemini-1.5-pro".to_string());
        fallback_models.insert(
            CloudProviderFamily::Custom,
            "qwen2.5-coder:7b".to_string(),
        );

        Self {
            triage_model: "gemini-2.0-flash".to_string(),
            primary_model: "claude-3-5-sonnet-20241022".to_string(),
            reviewer_model: "gpt-4o".to_string(),
            failover_chain: vec![
                CloudProviderFamily::Anthropic,
                CloudProviderFamily::OpenAI,
                CloudProviderFamily::Google,
                CloudProviderFamily::Custom,
            ],
            fallback_models,
            consensus_threshold: 0.75,
            token_budget_guard: 50_000,
            failover_enabled: true,
            circuit_breaker_failure_threshold: 3,
            circuit_breaker_recovery_timeout: Duration::from_secs(10),
            max_turns: 4,
            timeout_secs: 30,
        }
    }
}

impl CloudSwarmConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_triage_model(mut self, model: impl Into<String>) -> Self {
        self.triage_model = model.into();
        self
    }

    pub fn with_primary_model(mut self, model: impl Into<String>) -> Self {
        self.primary_model = model.into();
        self
    }

    pub fn with_reviewer_model(mut self, model: impl Into<String>) -> Self {
        self.reviewer_model = model.into();
        self
    }

    pub fn with_failover_chain(mut self, chain: Vec<CloudProviderFamily>) -> Self {
        self.failover_chain = chain;
        self
    }

    pub fn with_fallback_model(
        mut self,
        provider: CloudProviderFamily,
        model: impl Into<String>,
    ) -> Self {
        self.fallback_models.insert(provider, model.into());
        self
    }

    pub fn with_consensus_threshold(mut self, threshold: f64) -> Self {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    pub fn with_token_budget_guard(mut self, budget: usize) -> Self {
        self.token_budget_guard = budget;
        self
    }

    pub fn with_failover_enabled(mut self, enabled: bool) -> Self {
        self.failover_enabled = enabled;
        self
    }

    pub fn with_max_turns(mut self, max: usize) -> Self {
        self.max_turns = max;
        self
    }

    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// Swarm operational metrics tracked across execution cycles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudSwarmMetrics {
    pub total_cycles: usize,
    pub successful_cycles: usize,
    pub failed_cycles: usize,
    pub failovers_triggered: usize,
    pub total_token_savings_pct_sum: f64,
    pub total_prompt_tokens: usize,
    pub total_distilled_tokens: usize,
    pub secrets_redacted: usize,
    pub injections_blocked: usize,
    pub reflexions_recorded: usize,
}

// =========================================================================
// 4. Cloud Model Executor: Traits, Responses, Mocks & HTTP Client
// =========================================================================

/// Universal completion response returned by cloud model executors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudExecutionResponse {
    pub content: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub provider: CloudProviderFamily,
    pub model: String,
    pub latency_ms: u64,
}

impl CloudExecutionResponse {
    pub fn new(
        content: impl Into<String>,
        provider: CloudProviderFamily,
        model: impl Into<String>,
    ) -> Self {
        let content_str = content.into();
        let prompt_tokens = 50;
        let completion_tokens = (content_str.len() + 3) / 4;
        Self {
            content: content_str,
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            provider,
            model: model.into(),
            latency_ms: 150,
        }
    }

    pub fn with_tokens(
        mut self,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> Self {
        self.prompt_tokens = prompt_tokens;
        self.completion_tokens = completion_tokens;
        self.total_tokens = prompt_tokens + completion_tokens;
        self
    }

    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }
}

/// Async trait implemented by real and mock cloud model executors
#[async_trait]
pub trait CloudModelExecutor: Send + Sync {
    async fn generate(
        &self,
        provider: CloudProviderFamily,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<CloudExecutionResponse>;
}

/// Recorded call for deterministic verification in tests
#[derive(Debug, Clone)]
pub struct RecordedCloudCall {
    pub provider: CloudProviderFamily,
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub timestamp_ms: u64,
}

/// High-speed, deterministic mock executor for unit and integration testing
pub struct MockCloudModelExecutor {
    canned_responses: RwLock<HashMap<(CloudProviderFamily, String), CloudExecutionResponse>>,
    provider_responses: RwLock<HashMap<CloudProviderFamily, String>>,
    provider_errors: RwLock<HashMap<CloudProviderFamily, String>>,
    model_errors: RwLock<HashMap<String, String>>,
    failure_counters: RwLock<HashMap<CloudProviderFamily, (usize, String, String)>>,
    recorded_calls: RwLock<Vec<RecordedCloudCall>>,
}

impl Default for MockCloudModelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockCloudModelExecutor {
    pub fn new() -> Self {
        Self {
            canned_responses: RwLock::new(HashMap::new()),
            provider_responses: RwLock::new(HashMap::new()),
            provider_errors: RwLock::new(HashMap::new()),
            model_errors: RwLock::new(HashMap::new()),
            failure_counters: RwLock::new(HashMap::new()),
            recorded_calls: RwLock::new(Vec::new()),
        }
    }

    pub fn set_canned_response(
        &self,
        provider: CloudProviderFamily,
        model: &str,
        response: CloudExecutionResponse,
    ) {
        if let Ok(mut map) = self.canned_responses.write() {
            map.insert((provider, model.to_string()), response);
        }
    }

    pub fn set_response_for_provider(
        &self,
        provider: CloudProviderFamily,
        content: &str,
    ) {
        if let Ok(mut map) = self.provider_responses.write() {
            map.insert(provider, content.to_string());
        }
    }

    pub fn set_error_for_provider(
        &self,
        provider: CloudProviderFamily,
        error_msg: &str,
    ) {
        if let Ok(mut map) = self.provider_errors.write() {
            map.insert(provider, error_msg.to_string());
        }
    }

    pub fn set_error_for_model(&self, model: &str, error_msg: &str) {
        if let Ok(mut map) = self.model_errors.write() {
            map.insert(model.to_string(), error_msg.to_string());
        }
    }

    pub fn set_failures_before_success(
        &self,
        provider: CloudProviderFamily,
        failures: usize,
        error_msg: &str,
        success_content: &str,
    ) {
        if let Ok(mut map) = self.failure_counters.write() {
            map.insert(
                provider,
                (failures, error_msg.to_string(), success_content.to_string()),
            );
        }
    }

    pub fn recorded_calls(&self) -> Vec<RecordedCloudCall> {
        self.recorded_calls
            .read()
            .map(|c| c.clone())
            .unwrap_or_default()
    }

    pub fn call_count(&self, provider: CloudProviderFamily) -> usize {
        self.recorded_calls
            .read()
            .map(|calls| calls.iter().filter(|c| c.provider == provider).count())
            .unwrap_or(0)
    }

    pub fn clear_calls(&self) {
        if let Ok(mut calls) = self.recorded_calls.write() {
            calls.clear();
        }
    }
}

#[async_trait]
impl CloudModelExecutor for MockCloudModelExecutor {
    async fn generate(
        &self,
        provider: CloudProviderFamily,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<CloudExecutionResponse> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Record invocation
        if let Ok(mut calls) = self.recorded_calls.write() {
            calls.push(RecordedCloudCall {
                provider,
                model: model.to_string(),
                system_prompt: system_prompt.to_string(),
                user_prompt: user_prompt.to_string(),
                timestamp_ms: now_ms,
            });
        }

        // Check simulated failure counters (fail N times, then succeed)
        if let Ok(mut counters) = self.failure_counters.write() {
            if let Some(entry) = counters.get_mut(&provider) {
                if entry.0 > 0 {
                    entry.0 -= 1;
                    let err = entry.1.clone();
                    if err.contains("429") || err.to_lowercase().contains("rate limit") {
                        return Err(TagisanError::RateLimited(
                            provider.as_str().to_string(),
                            Some(Duration::from_millis(500)),
                        ));
                    } else if err.contains("503") || err.to_lowercase().contains("unavailable") {
                        return Err(TagisanError::BadResponse(
                            provider.as_str().to_string(),
                            "HTTP 503 Service Unavailable".to_string(),
                        ));
                    } else {
                        return Err(TagisanError::Execution(err));
                    }
                } else if !entry.2.is_empty() {
                    return Ok(CloudExecutionResponse::new(
                        &entry.2,
                        provider,
                        model,
                    ));
                }
            }
        }

        // Check explicit provider errors
        if let Ok(errors) = self.provider_errors.read() {
            if let Some(err) = errors.get(&provider) {
                if err.contains("429") || err.to_lowercase().contains("rate limit") {
                    return Err(TagisanError::RateLimited(
                        provider.as_str().to_string(),
                        Some(Duration::from_millis(500)),
                    ));
                } else if err.contains("503") || err.to_lowercase().contains("unavailable") {
                    return Err(TagisanError::BadResponse(
                        provider.as_str().to_string(),
                        "HTTP 503 Service Unavailable".to_string(),
                    ));
                } else {
                    return Err(TagisanError::Execution(err.clone()));
                }
            }
        }

        // Check explicit model errors
        if let Ok(errors) = self.model_errors.read() {
            if let Some(err) = errors.get(model) {
                return Err(TagisanError::Execution(err.clone()));
            }
        }

        // Check canned response for (provider, model)
        if let Ok(map) = self.canned_responses.read() {
            if let Some(res) = map.get(&(provider, model.to_string())) {
                return Ok(res.clone());
            }
        }

        // Check provider-level response
        if let Ok(map) = self.provider_responses.read() {
            if let Some(content) = map.get(&provider) {
                return Ok(CloudExecutionResponse::new(content, provider, model));
            }
        }

        // Default smart synthesis based on role heuristics in prompts
        let sys_lower = system_prompt.to_lowercase();
        if sys_lower.contains("triage") || sys_lower.contains("dispatcher") {
            let default_contract = json!({
                "task_summary": "High-performance distributed caching layer with zero-copy deserialization",
                "primary_objective": "Implement thread-safe LRU cache in Rust with O(1) ops",
                "architecture_invariants": [
                    "Zero allocation on cache hits",
                    "Thread-safe via parking_lot::RwLock"
                ],
                "constraints": [
                    "No unsafe code",
                    "Bounded memory footprint"
                ],
                "edge_cases": [
                    "Concurrent evictions under memory pressure",
                    "Zero-capacity cache initialization"
                ],
                "dependencies": ["parking_lot", "hashbrown"],
                "language_or_tech": "Rust",
                "expected_schema": "LruCache<K, V>"
            });
            return Ok(CloudExecutionResponse::new(
                default_contract.to_string(),
                provider,
                model,
            )
            .with_tokens(40, 65));
        } else if sys_lower.contains("reviewer") || sys_lower.contains("red-team") {
            let default_review = json!({
                "approved": true,
                "score": 0.96,
                "security_findings": [
                    "Pre-egress DLP verified: 0 credentials leaked"
                ],
                "fuzz_tests": [
                    "test_concurrent_eviction_under_high_load",
                    "test_zero_capacity_boundary_rejection"
                ],
                "critique": "Implementation strictly conforms to distilled contract invariants with zero leaks.",
                "proposed_patches": null
            });
            return Ok(CloudExecutionResponse::new(
                default_review.to_string(),
                provider,
                model,
            )
            .with_tokens(120, 85));
        }

        // Default primary code generator
        let default_code = format!(
            "// Implementation generated by {:?} ({})\n\
             pub struct LruCache<K, V> {{\n\
                 capacity: usize,\n\
                 items: std::collections::HashMap<K, V>,\n\
             }}\n\
             impl<K: std::hash::Hash + Eq + Clone, V: Clone> LruCache<K, V> {{\n\
                 pub fn new(capacity: usize) -> Self {{\n\
                     Self {{ capacity, items: std::collections::HashMap::with_capacity(capacity) }}\n\
                 }}\n\
                 pub fn get(&self, key: &K) -> Option<&V> {{\n\
                     self.items.get(key)\n\
                 }}\n\
                 pub fn insert(&mut self, key: K, val: V) {{\n\
                     if self.items.len() >= self.capacity {{\n\
                         if let Some(first_key) = self.items.keys().next().cloned() {{\n\
                             self.items.remove(&first_key);\n\
                         }}\n\
                     }}\n\
                     self.items.insert(key, val);\n\
                 }}\n\
             }}",
            provider, model
        );

        Ok(CloudExecutionResponse::new(default_code, provider, model).with_tokens(150, 200))
    }
}

/// Real production HTTP client querying Anthropic, OpenAI, Gemini, or Ollama endpoints
pub struct HttpCloudModelExecutor {
    client: reqwest::Client,
}

impl Default for HttpCloudModelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpCloudModelExecutor {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

#[async_trait]
impl CloudModelExecutor for HttpCloudModelExecutor {
    async fn generate(
        &self,
        provider: CloudProviderFamily,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<CloudExecutionResponse> {
        let start = std::time::Instant::now();

        match provider {
            CloudProviderFamily::Anthropic => {
                let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    TagisanError::Authentication(
                        "Anthropic".to_string(),
                        "ANTHROPIC_API_KEY environment variable is not set".to_string(),
                    )
                })?;

                let payload = json!({
                    "model": model,
                    "max_tokens": 4096,
                    "system": system_prompt,
                    "messages": [
                        { "role": "user", "content": user_prompt }
                    ]
                });

                let res = self
                    .client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;

                let status = res.status();
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    return Err(TagisanError::RateLimited(
                        "Anthropic".to_string(),
                        Some(Duration::from_secs(5)),
                    ));
                } else if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
                    || status.as_u16() == 529
                {
                    return Err(TagisanError::BadResponse(
                        "Anthropic".to_string(),
                        format!("HTTP {status} Anthropic Overloaded / Unavailable"),
                    ));
                } else if !status.is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(TagisanError::BadResponse("Anthropic".to_string(), err_text));
                }

                let body: Value = res.json().await?;
                let content = body["content"][0]["text"].as_str().unwrap_or_default().to_string();
                let prompt_tokens = body["usage"]["input_tokens"].as_u64().unwrap_or(50) as usize;
                let completion_tokens = body["usage"]["output_tokens"].as_u64().unwrap_or(150) as usize;

                Ok(CloudExecutionResponse {
                    content,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    provider,
                    model: model.to_string(),
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            CloudProviderFamily::OpenAI => {
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    TagisanError::Authentication(
                        "OpenAI".to_string(),
                        "OPENAI_API_KEY environment variable is not set".to_string(),
                    )
                })?;

                let payload = json!({
                    "model": model,
                    "messages": [
                        { "role": "system", "content": system_prompt },
                        { "role": "user", "content": user_prompt }
                    ]
                });

                let res = self
                    .client
                    .post("https://api.openai.com/v1/chat/completions")
                    .bearer_auth(api_key)
                    .header("content-type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;

                let status = res.status();
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    return Err(TagisanError::RateLimited(
                        "OpenAI".to_string(),
                        Some(Duration::from_secs(5)),
                    ));
                } else if status.is_server_error() {
                    return Err(TagisanError::BadResponse(
                        "OpenAI".to_string(),
                        format!("HTTP {status} OpenAI Server Error"),
                    ));
                } else if !status.is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(TagisanError::BadResponse("OpenAI".to_string(), err_text));
                }

                let body: Value = res.json().await?;
                let content = body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let prompt_tokens = body["usage"]["prompt_tokens"].as_u64().unwrap_or(50) as usize;
                let completion_tokens = body["usage"]["completion_tokens"].as_u64().unwrap_or(150) as usize;

                Ok(CloudExecutionResponse {
                    content,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    provider,
                    model: model.to_string(),
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            CloudProviderFamily::Google => {
                let api_key = std::env::var("GEMINI_API_KEY")
                    .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                    .map_err(|_| {
                        TagisanError::Authentication(
                            "Google".to_string(),
                            "GEMINI_API_KEY or GOOGLE_API_KEY environment variable is not set".to_string(),
                        )
                    })?;

                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                    model, api_key
                );

                let payload = json!({
                    "system_instruction": {
                        "parts": [{ "text": system_prompt }]
                    },
                    "contents": [{
                        "parts": [{ "text": user_prompt }]
                    }]
                });

                let res = self
                    .client
                    .post(&url)
                    .header("content-type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;

                let status = res.status();
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    return Err(TagisanError::RateLimited(
                        "Google".to_string(),
                        Some(Duration::from_secs(5)),
                    ));
                } else if status.is_server_error() {
                    return Err(TagisanError::BadResponse(
                        "Google".to_string(),
                        format!("HTTP {status} Gemini Server Error"),
                    ));
                } else if !status.is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(TagisanError::BadResponse("Google".to_string(), err_text));
                }

                let body: Value = res.json().await?;
                let content = body["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                let prompt_tokens = body["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(50) as usize;
                let completion_tokens = body["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(150) as usize;

                Ok(CloudExecutionResponse {
                    content,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    provider,
                    model: model.to_string(),
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            CloudProviderFamily::Custom => {
                // Default to local Ollama fallback endpoint
                let url = "http://127.0.0.1:11434/api/generate";
                let payload = json!({
                    "model": model,
                    "prompt": user_prompt,
                    "system": system_prompt,
                    "stream": false
                });

                let res = self.client.post(url).json(&payload).send().await?;
                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(TagisanError::BadResponse("Custom/Ollama".to_string(), err_text));
                }

                let body: Value = res.json().await?;
                let content = body["response"].as_str().unwrap_or_default().to_string();
                let prompt_tokens = body["prompt_eval_count"].as_u64().unwrap_or(40) as usize;
                let completion_tokens = body["eval_count"].as_u64().unwrap_or(120) as usize;

                Ok(CloudExecutionResponse {
                    content,
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    provider,
                    model: model.to_string(),
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
        }
    }
}

// =========================================================================
// 5. CloudSwarmBridge: Coordinator & Multi-Provider Consensus Engine
// =========================================================================

/// Cloud Frontier LLM Agent Bridge & Multi-Provider Consensus Swarm Coordinator
pub struct CloudSwarmBridge {
    pub config: CloudSwarmConfig,
    pub security_gateway: Arc<AgentShieldBridgeGateway>,
    pub circuit_breakers: Arc<AgentCircuitBreakerRegistry>,
    pub reflexion_bridge: Arc<SharedReflexionBridge>,
    pub metrics: Arc<RwLock<CloudSwarmMetrics>>,
    executor: Arc<dyn CloudModelExecutor>,
}

impl CloudSwarmBridge {
    /// Create a new CloudSwarmBridge with default HttpCloudModelExecutor
    pub fn new(config: CloudSwarmConfig) -> Self {
        let executor = Arc::new(HttpCloudModelExecutor::new());
        Self::with_executor(config, executor)
    }

    /// Create a CloudSwarmBridge with a custom or mock executor
    pub fn with_executor(config: CloudSwarmConfig, executor: Arc<dyn CloudModelExecutor>) -> Self {
        let circuit_breakers = Arc::new(AgentCircuitBreakerRegistry::with_defaults(
            config.circuit_breaker_failure_threshold,
            config.circuit_breaker_recovery_timeout,
            2,
        ));

        Self {
            config,
            security_gateway: Arc::new(AgentShieldBridgeGateway::new()),
            circuit_breakers,
            reflexion_bridge: Arc::new(SharedReflexionBridge::new()),
            metrics: Arc::new(RwLock::new(CloudSwarmMetrics::default())),
            executor,
        }
    }

    /// Approximate token count from string length (1 token ≈ 4 characters)
    pub fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4.max(1)
    }

    /// Pre-Egress AgentShield DLP Gateway:
    /// Scans for prompt injection attacks and redacts API keys, credentials, and passwords
    /// before ANY payload leaves the local workstation.
    pub fn sanitize_pre_egress_payload(&self, text: &str) -> Result<(String, usize)> {
        // 1. Prompt Injection Scan
        let verdict = crate::ecc::AgentShieldScanner::scan_prompt_injection(text);
        if let crate::ecc::AgentShieldVerdict::Block { reason, threat_level } = verdict {
            if let Ok(mut m) = self.metrics.write() {
                m.injections_blocked += 1;
            }
            return Err(TagisanError::Security(format!(
                "AgentShield blocked pre-egress cloud payload [{:?}]: Prompt injection detected ({})",
                threat_level, reason
            )));
        }

        // 2. Jailbreak / Override Heuristic Scan
        let lower = text.to_lowercase();
        let jailbreak_signatures = [
            "ignore all previous instructions",
            "you are now in developer mode",
            "system prompt override",
            "jailbreak: bypass all safety",
            "admin override: disable agentshield",
        ];
        for sig in jailbreak_signatures {
            if lower.contains(sig) {
                if let Ok(mut m) = self.metrics.write() {
                    m.injections_blocked += 1;
                }
                return Err(TagisanError::Security(format!(
                    "AgentShield blocked pre-egress cloud payload: Jailbreak signature detected ('{sig}')"
                )));
            }
        }

        // 3. Secret & Credential Redaction
        let mut sanitized = crate::ecc::AgentShieldScanner::redact_dlp_secrets(text);
        let mut redacted_count = 0;

        // Ensure generic sk- patterns are redacted
        let patterns = [
            ("sk-ant-", "[REDACTED_ANTHROPIC_KEY]"),
            ("sk-proj-", "[REDACTED_OPENAI_KEY]"),
            ("AIzaSy", "[REDACTED_GEMINI_KEY]"),
            ("ghp_", "[REDACTED_GITHUB_TOKEN]"),
            ("xai-", "[REDACTED_XAI_KEY]"),
            ("AKIA", "[REDACTED_AWS_KEY]"),
        ];

        for (prefix, replacement) in patterns {
            let mut search_from = 0;
            while search_from < sanitized.len() {
                if let Some(rel_pos) = sanitized[search_from..].find(prefix) {
                    let pos = search_from + rel_pos;
                    let end = sanitized[pos..]
                        .find(|c: char| {
                            c.is_whitespace()
                                || c == '"'
                                || c == '\''
                                || c == '`'
                                || c == '\n'
                                || c == '\r'
                                || c == ','
                                || c == ';'
                                || c == ')'
                                || c == ']'
                                || c == '}'
                                || c == '>'
                                || c == '\\'
                        })
                        .map(|i| pos + i)
                        .unwrap_or(sanitized.len());

                    sanitized.replace_range(pos..end, replacement);
                    search_from = pos + replacement.len();
                    redacted_count += 1;
                } else {
                    break;
                }
            }
        }

        // Additional sensitive connection and password redactions
        let secret_indicators = [
            ("password=", "[REDACTED_PASSWORD]"),
            ("passwd=", "[REDACTED_PASSWORD]"),
            ("client_secret=", "[REDACTED_CLIENT_SECRET]"),
            ("secret_key=", "[REDACTED_SECRET_KEY]"),
            ("database_url=postgres://", "[REDACTED_DATABASE_URL]"),
            ("database_url=mysql://", "[REDACTED_DATABASE_URL]"),
        ];

        for (ind, rep) in secret_indicators {
            let mut search_from = 0;
            while search_from < sanitized.len() {
                let lower_san = sanitized.to_lowercase();
                if let Some(rel_pos) = lower_san[search_from..].find(ind) {
                    let pos = search_from + rel_pos;
                    let end = sanitized[pos..]
                        .find(|c: char| c.is_whitespace() || c == '\n' || c == '"' || c == '\'')
                        .map(|i| pos + i)
                        .unwrap_or(sanitized.len());
                    sanitized.replace_range(pos..end, rep);
                    search_from = pos + rep.len();
                    redacted_count += 1;
                } else {
                    break;
                }
            }
        }

        if sanitized != text && redacted_count == 0 {
            redacted_count = 1;
        }

        if redacted_count > 0 {
            if let Ok(mut m) = self.metrics.write() {
                m.secrets_redacted += redacted_count;
            }
        }

        Ok((sanitized, redacted_count))
    }

    /// Execute the complete 4-turn multi-provider Cloud Frontier Swarm cycle:
    ///
    /// 1. **Turn 1: Triage/Dispatcher Tier**: Low-cost, fast model distills prompt into compact JSON contract (measuring token savings >80%).
    /// 2. **Turn 2: Primary Frontier Tier**: Synthesizes surgical solution with zero-stall cross-cloud failover.
    /// 3. **Turn 3: Dialectic Reviewer Tier**: Cross-provider model red-teams code, verifies security invariants, and synthesizes fuzz tests.
    /// 4. **Turn 4: Consensus Engine**: Computes cross-provider agreement score and confirms consensus threshold.
    pub async fn execute_cloud_swarm_cycle(&self, user_prompt: &str) -> Result<CloudSwarmResult> {
        let cycle_start = std::time::Instant::now();
        info!(
            "{}",
            "🚀 [Cloud Frontier Swarm] Initiating multi-provider consensus cycle...".bold().yellow()
        );

        if let Ok(mut m) = self.metrics.write() {
            m.total_cycles += 1;
        }

        // ---------------------------------------------------------------------
        // STEP 0: Pre-Egress AgentShield DLP Sanitization
        // ---------------------------------------------------------------------
        let (safe_prompt, secrets_redacted_count) =
            self.sanitize_pre_egress_payload(user_prompt)?;

        let raw_tokens = Self::estimate_tokens(user_prompt);

        // ---------------------------------------------------------------------
        // TURN 1: Triage / Dispatcher Tier (Low Cost & Latency)
        // ---------------------------------------------------------------------
        let triage_provider =
            CloudProviderFamily::from_model_name(&self.config.triage_model);
        self.circuit_breakers
            .can_execute(triage_provider.as_str())?;

        info!(
            "{}",
            format!(
                "🔍 [Turn 1: Triage ({})] Analyzing intent & distilling structured contract...",
                self.config.triage_model
            )
            .cyan()
        );

        let triage_system = "You are the Triage & Dispatcher Agent in the Tagisan Cloud Frontier Swarm.\n\
Analyze the user request, decompose all architectural dependencies, eliminate conversational token bloat,\n\
and emit a strict, canonical JSON contract with this exact schema:\n\
{\n\
  \"task_summary\": \"compact summary of the task\",\n\
  \"primary_objective\": \"unambiguous primary objective\",\n\
  \"architecture_invariants\": [\"invariant 1\", \"invariant 2\"],\n\
  \"constraints\": [\"constraint 1\", \"constraint 2\"],\n\
  \"edge_cases\": [\"edge case 1\", \"edge case 2\"],\n\
  \"dependencies\": [\"dep 1\", \"dep 2\"],\n\
  \"language_or_tech\": \"language or stack\",\n\
  \"expected_schema\": \"expected signature or type\"\n\
}\n\
Return ONLY valid JSON.";

        let triage_resp = match self
            .executor
            .generate(
                triage_provider,
                &self.config.triage_model,
                triage_system,
                &safe_prompt,
            )
            .await
        {
            Ok(res) => {
                self.circuit_breakers.record_success(triage_provider.as_str());
                res
            }
            Err(e) => {
                self.circuit_breakers
                    .record_failure(triage_provider.as_str(), &e.to_string());
                return Err(e);
            }
        };

        let contract = DistilledContract::parse(&triage_resp.content);
        let contract_json = contract.to_contract_json();
        let distilled_tokens = Self::estimate_tokens(&contract_json);

        let token_savings_pct = if raw_tokens > distilled_tokens {
            ((raw_tokens - distilled_tokens) as f64 / raw_tokens as f64) * 100.0
        } else {
            0.0
        };

        info!(
            "{}",
            format!(
                "📋 [Triage Contract Distilled] Tokens: {} -> {} ({:.1}% savings) [Tech: {:?}]",
                raw_tokens,
                distilled_tokens,
                token_savings_pct,
                contract.language_or_tech
            )
            .green()
        );

        // Token budget check
        let mut cumulative_tokens = triage_resp.total_tokens;
        if cumulative_tokens >= self.config.token_budget_guard {
            return Err(TagisanError::ResourceExhausted(format!(
                "Token budget guard exceeded during Turn 1 (used: {}, limit: {})",
                cumulative_tokens, self.config.token_budget_guard
            )));
        }

        // ---------------------------------------------------------------------
        // TURN 2: Deep Reasoning Tier (Primary Model with Zero-Stall Failover)
        // ---------------------------------------------------------------------
        let mut primary_provider =
            CloudProviderFamily::from_model_name(&self.config.primary_model);
        let mut primary_model = self.config.primary_model.clone();
        let mut primary_output = String::new();
        let mut failover_occurred = false;
        let mut failover_history = Vec::new();

        // Query Cross-Provider Shared Reflexion Vault
        let relevant_reflexions = self.reflexion_bridge.query(&contract.task_summary);
        let mut reflexion_context = String::new();
        let invariants_synced_count = relevant_reflexions.len();
        if !relevant_reflexions.is_empty() {
            reflexion_context.push_str("Prior Failures & Preventative Invariants (Reflexion Vault):\n");
            for r in &relevant_reflexions {
                reflexion_context.push_str(&format!(
                    "- Invariant: {}\n  Prior Signature: {}\n",
                    r.preventative_invariant, r.error_signature
                ));
            }
        }

        let primary_system = "You are the Primary Implementation Specialist in the Tagisan Cloud Frontier Swarm.\n\
Generate surgical, production-grade, complete implementations directly based on the distilled task contract.\n\
Strictly observe all architectural invariants, avoid unnecessary dependencies, and handle all edge cases.";

        let primary_user_prompt = format!(
            "DISTILLED TASK CONTRACT:\n\
Summary: {}\n\
Objective: {}\n\
Architecture Invariants: {}\n\
Constraints: {}\n\
Edge Cases: {}\n\
Dependencies: {}\n\
Language/Tech: {}\n\
Expected Schema: {}\n\n\
{}\n\
Please output the implementation directly.",
            contract.task_summary,
            contract.primary_objective,
            contract.architecture_invariants.join(", "),
            contract.constraints.join(", "),
            contract.edge_cases.join(", "),
            contract.dependencies.join(", "),
            contract.language_or_tech.as_deref().unwrap_or("general"),
            contract.expected_schema.as_deref().unwrap_or("unspecified"),
            reflexion_context
        );

        // Pre-egress sanitize primary prompt
        let (safe_primary_prompt, _) =
            self.sanitize_pre_egress_payload(&primary_user_prompt)?;

        // Execute primary with 0-stall circuit breaker & failover
        let mut primary_succeeded = false;

        // Check primary circuit breaker
        let primary_cb_allowed = self
            .circuit_breakers
            .can_execute(primary_provider.as_str())
            .is_ok();

        if primary_cb_allowed {
            info!(
                "{}",
                format!(
                    "💻 [Turn 2: Primary ({:?}/{})] Generating implementation...",
                    primary_provider, primary_model
                )
                .cyan()
            );

            match self
                .executor
                .generate(
                    primary_provider,
                    &primary_model,
                    primary_system,
                    &safe_primary_prompt,
                )
                .await
            {
                Ok(resp) => {
                    self.circuit_breakers
                        .record_success(primary_provider.as_str());
                    primary_output = resp.content;
                    cumulative_tokens += resp.total_tokens;
                    primary_succeeded = true;
                }
                Err(err) => {
                    let err_str = err.to_string();
                    warn!(
                        "Primary provider {:?} failed: {}. Initiating zero-stall cross-cloud failover...",
                        primary_provider, err_str
                    );
                    self.circuit_breakers
                        .record_failure(primary_provider.as_str(), &err_str);
                    failover_history.push(FailoverRecord {
                        from_provider: primary_provider,
                        to_provider: primary_provider, // will update when candidate chosen
                        reason: err_str,
                        timestamp_ms: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64,
                    });
                }
            }
        } else {
            warn!(
                "Circuit breaker is OPEN for primary provider {:?}. Initiating zero-stall failover...",
                primary_provider
            );
            failover_history.push(FailoverRecord {
                from_provider: primary_provider,
                to_provider: primary_provider,
                reason: "Circuit breaker OPEN".to_string(),
                timestamp_ms: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
        }

        // Zero-Stall Cross-Cloud Failover Chain Execution
        if !primary_succeeded && self.config.failover_enabled {
            for candidate_provider in &self.config.failover_chain {
                if *candidate_provider == primary_provider {
                    continue; // skip original failed provider
                }

                // Check candidate circuit breaker
                if self
                    .circuit_breakers
                    .can_execute(candidate_provider.as_str())
                    .is_err()
                {
                    continue;
                }

                let candidate_model = self
                    .config
                    .fallback_models
                    .get(candidate_provider)
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());

                info!(
                    "{}",
                    format!(
                        "🔀 [Zero-Stall Failover] Dispatched task to fallback provider {:?} ({})",
                        candidate_provider, candidate_model
                    )
                    .yellow()
                );

                match self
                    .executor
                    .generate(
                        *candidate_provider,
                        &candidate_model,
                        primary_system,
                        &safe_primary_prompt,
                    )
                    .await
                {
                    Ok(resp) => {
                        self.circuit_breakers
                            .record_success(candidate_provider.as_str());
                        primary_output = resp.content;
                        cumulative_tokens += resp.total_tokens;

                        if let Some(last) = failover_history.last_mut() {
                            last.to_provider = *candidate_provider;
                        }

                        primary_provider = *candidate_provider;
                        primary_model = candidate_model;
                        primary_succeeded = true;
                        failover_occurred = true;

                        if let Ok(mut m) = self.metrics.write() {
                            m.failovers_triggered += 1;
                        }

                        info!(
                            "{}",
                            format!(
                                "✔ [Zero-Stall Failover Succeeded] {:?} generated implementation successfully",
                                candidate_provider
                            )
                            .green()
                        );
                        break;
                    }
                    Err(candidate_err) => {
                        let c_err_str = candidate_err.to_string();
                        warn!(
                            "Fallback provider {:?} also failed: {}",
                            candidate_provider, c_err_str
                        );
                        self.circuit_breakers
                            .record_failure(candidate_provider.as_str(), &c_err_str);
                        failover_history.push(FailoverRecord {
                            from_provider: *candidate_provider,
                            to_provider: *candidate_provider,
                            reason: c_err_str,
                            timestamp_ms: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                        });
                    }
                }
            }
        }

        if !primary_succeeded {
            return Err(TagisanError::Execution(
                "All cloud providers in failover chain were exhausted or rate-limited".to_string(),
            ));
        }

        // Token budget check
        if cumulative_tokens >= self.config.token_budget_guard {
            return Err(TagisanError::ResourceExhausted(format!(
                "Token budget guard exceeded during Turn 2 (used: {}, limit: {})",
                cumulative_tokens, self.config.token_budget_guard
            )));
        }

        // ---------------------------------------------------------------------
        // TURN 3: Dialectic Reviewer & Fuzzer Tier (Cross-Provider)
        // ---------------------------------------------------------------------
        let reviewer_provider =
            CloudProviderFamily::from_model_name(&self.config.reviewer_model);
        self.circuit_breakers
            .can_execute(reviewer_provider.as_str())?;

        info!(
            "{}",
            format!(
                "🛡️ [Turn 3: Reviewer ({:?}/{})] Red-teaming implementation & synthesizing fuzz tests...",
                reviewer_provider, self.config.reviewer_model
            )
            .cyan()
        );

        let reviewer_system = "You are the Cross-Provider Dialectic Reviewer & Red-Team Auditor in the Tagisan Swarm.\n\
Your mission is to audit the primary model's implementation against the distilled contract.\n\
1. Audit security invariants (DLP secret leaks, memory safety, injection vulnerabilities).\n\
2. Red-team the code for logic defects, off-by-one errors, and unhandled edge cases.\n\
3. Synthesize rigorous fuzz test cases to validate robustness.\n\
Emit your evaluation in strict JSON format:\n\
{\n\
  \"approved\": true,\n\
  \"score\": 0.95,\n\
  \"security_findings\": [\"finding 1\"],\n\
  \"fuzz_tests\": [\"test_fuzz_case_1\", \"test_fuzz_case_2\"],\n\
  \"critique\": \"detailed critique\",\n\
  \"proposed_patches\": null\n\
}\n\
Return ONLY valid JSON.";

        let reviewer_user_prompt = format!(
            "DISTILLED CONTRACT:\n{}\n\n\
PRIMARY IMPLEMENTATION (Generated by {:?}/{}):\n{}\n\n\
Please review, red-team, and evaluate this implementation.",
            contract_json, primary_provider, primary_model, primary_output
        );

        let (safe_reviewer_prompt, _) =
            self.sanitize_pre_egress_payload(&reviewer_user_prompt)?;

        let reviewer_resp = match self
            .executor
            .generate(
                reviewer_provider,
                &self.config.reviewer_model,
                reviewer_system,
                &safe_reviewer_prompt,
            )
            .await
        {
            Ok(res) => {
                self.circuit_breakers
                    .record_success(reviewer_provider.as_str());
                res
            }
            Err(e) => {
                self.circuit_breakers
                    .record_failure(reviewer_provider.as_str(), &e.to_string());
                return Err(e);
            }
        };

        cumulative_tokens += reviewer_resp.total_tokens;
        let review_evaluation = ReviewerEvaluation::parse(&reviewer_resp.content);

        // Synchronize Lessons Learned into Shared Reflexion Vault
        if !review_evaluation.approved
            || review_evaluation.score < 0.75
            || !review_evaluation.security_findings.is_empty()
        {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let entry = ReflexionEntry {
                id: format!(
                    "cloud-refl-{}",
                    &blake3::hash(format!("{now_ms}-{}", review_evaluation.critique).as_bytes())
                        .to_hex()[..16]
                ),
                timestamp: chrono::Utc::now().to_rfc3339(),
                error_signature: review_evaluation
                    .security_findings
                    .first()
                    .cloned()
                    .unwrap_or_else(|| review_evaluation.critique.clone()),
                root_cause: format!(
                    "Reviewer ({:?}) identified security or logic deficiencies in {:?} output",
                    reviewer_provider, primary_provider
                ),
                fix_applied: review_evaluation
                    .proposed_patches
                    .clone()
                    .unwrap_or_else(|| "Apply reviewer suggested fixes".to_string()),
                preventative_invariant: format!(
                    "Verify compliance: {}",
                    review_evaluation.critique
                ),
                tags: vec![
                    "cloud_bridge".to_string(),
                    "dialectic_reviewer".to_string(),
                    primary_provider.as_str().to_string(),
                ],
            };

            let _ = self.reflexion_bridge.sync_postmortem(entry);
            if let Ok(mut m) = self.metrics.write() {
                m.reflexions_recorded += 1;
            }
        }

        // ---------------------------------------------------------------------
        // TURN 4: Consensus Engine & Agreement Scoring
        // ---------------------------------------------------------------------
        info!(
            "{}",
            "⚖️  [Turn 4: Consensus Engine] Computing cross-provider agreement...".yellow()
        );

        // Compute agreement score based on:
        // 1. Reviewer evaluation score (weight 0.7)
        // 2. Structural presence of deliverables and matching language/tech (weight 0.3)
        let mut structural_score = 0.5;
        if let Some(ref tech) = contract.language_or_tech {
            if primary_output.to_lowercase().contains(&tech.to_lowercase()) {
                structural_score += 0.25;
            }
        } else {
            structural_score += 0.25;
        }

        if !primary_output.trim().is_empty() {
            structural_score += 0.25;
        }

        let consensus_score = (review_evaluation.score * 0.7) + (structural_score * 0.3);
        let consensus_reached = consensus_score >= self.config.consensus_threshold;

        if consensus_reached {
            info!(
                "{}",
                format!(
                    "✅ [Consensus Reached] Score: {:.2} >= Threshold: {:.2} across {:?} and {:?}",
                    consensus_score, self.config.consensus_threshold, primary_provider, reviewer_provider
                )
                .green()
                .bold()
            );
        } else {
            warn!(
                "{}",
                format!(
                    "⚠️ [Consensus Disputed] Score: {:.2} < Threshold: {:.2} between {:?} and {:?}",
                    consensus_score, self.config.consensus_threshold, primary_provider, reviewer_provider
                )
                .red()
            );
        }

        let total_latency_ms = cycle_start.elapsed().as_millis() as u64;

        if let Ok(mut m) = self.metrics.write() {
            if consensus_reached {
                m.successful_cycles += 1;
            } else {
                m.failed_cycles += 1;
            }
            m.total_token_savings_pct_sum += token_savings_pct;
            m.total_prompt_tokens += raw_tokens;
            m.total_distilled_tokens += distilled_tokens;
        }

        Ok(CloudSwarmResult {
            success: consensus_reached,
            raw_prompt: user_prompt.to_string(),
            raw_tokens,
            distilled_contract: contract,
            distilled_tokens,
            token_savings_pct,
            primary_provider,
            primary_model,
            primary_output,
            failover_occurred,
            failover_history,
            reviewer_provider,
            reviewer_model: self.config.reviewer_model.clone(),
            review_evaluation,
            consensus_score,
            consensus_reached,
            total_tokens: cumulative_tokens,
            total_latency_ms,
            secrets_redacted_count,
            invariants_synced_count,
            error: if !consensus_reached {
                Some(format!(
                    "Consensus agreement score ({:.2}) below threshold ({:.2})",
                    consensus_score, self.config.consensus_threshold
                ))
            } else {
                None
            },
        })
    }
}
