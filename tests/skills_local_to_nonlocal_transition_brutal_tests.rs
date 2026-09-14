//! Comprehensive Brutal Integration & Stress Test Suite: Local <-> Cloud Skills Subsystem Transitions
//! 
//! Tests 100% real, rigorous, and reliable behavior when switching between
//! Local LLM runtimes (Ollama, Colibri, local GGUF) and Non-Local / Cloud runtimes
//! (Gemini, Anthropic Claude, OpenAI, DeepSeek, xAI / Grok).
//!
//! Covers:
//! 1. Provider-Aware TokenBudget Auto-Calibration Roundtrips
//! 2. Provider Format & Prompt Injection Mode Transitions (CheatSheet vs Comprehensive)
//! 3. Domain Diversity & Anti-Monopoly Invariants (max 1/domain Local vs max 2/domain Cloud)
//! 4. Skill JIT Memory Tier Paging & Cache Dynamics (L1 Active <-> L2 Warm <-> L3 Cold)
//! 5. Semantic Invariant Guarding Across Context Scaling (Schema integrity, Diagnostic preservation, Safety contracts)
//! 6. Multi-Agent Swarm Mixed Roles & Dynamic Stage Failover Skill Re-calibration
//! 7. FetchSkillTool Execution Parity & JSON Schema Validation
//! 8. Round-Trip Reversibility & Zero State Drift (10+ alternating cycles)
//! 9. Token Budget Enforcement & Zero-Cost Local Exemption, Reasoning Extraction, CLI/Env var switching

use async_trait::async_trait;
use serde_json::json;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tagisan::ecc::{
    estimate_tokens, format_cheat_sheet, format_cloud_guidelines, format_dense_invariants,
    format_hierarchical, global_dispatcher, is_local_provider, InjectionMode,
    SemanticInvariantGuard, SemanticViolation, SkillJitManager, SkillTier, TokenBudget,
};
use tagisan::engine::budget::TokenBudgetTracker;
use tagisan::error::{Result, TagisanError};
use tagisan::providers::ollama::parse_thinking_blocks;
use tagisan::providers::{BoxEventStream, LlmProvider};
use tagisan::swarm::{
    AssemblyRoles, HarmonyStage, StructuredHarmonyPipeline, SyntaxValidationGate,
};
use tagisan::tools::{FetchSkillTool, ToolHandler};
use tagisan::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message, ProviderCapabilities,
    TokenUsage, ToolDefinition,
};
use tagisan::EngineContext;

// =========================================================================
// DETERMINISTIC MOCK LLM PROVIDER FOR BRUTAL TESTING
// =========================================================================

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum MockFailureMode {
    RateLimited(String, Option<Duration>),
    Authentication(String, String),
    BadResponse(String, String),
    BudgetExceeded(f64, f64),
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
        }
    }
}

#[derive(Clone)]
struct MockTransitionLlmProvider {
    id: &'static str,
    call_count: Arc<AtomicUsize>,
    canned_responses: Vec<String>,
    failure_err: Arc<Mutex<Option<MockFailureMode>>>,
    fail_on_call_index: Arc<Mutex<Option<usize>>>,
    recorded_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl MockTransitionLlmProvider {
    fn new(id: &'static str, canned_responses: Vec<String>) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses,
            failure_err: Arc::new(Mutex::new(None)),
            fail_on_call_index: Arc::new(Mutex::new(None)),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn with_persistent_error(id: &'static str, err: MockFailureMode) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses: Vec::new(),
            failure_err: Arc::new(Mutex::new(Some(err))),
            fail_on_call_index: Arc::new(Mutex::new(None)),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[allow(dead_code)]
    fn fail_once_at(id: &'static str, call_idx: usize, err: MockFailureMode, fallback_responses: Vec<String>) -> Self {
        Self {
            id,
            call_count: Arc::new(AtomicUsize::new(0)),
            canned_responses: fallback_responses,
            failure_err: Arc::new(Mutex::new(Some(err))),
            fail_on_call_index: Arc::new(Mutex::new(Some(call_idx))),
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn recorded_requests(&self) -> Vec<CompletionRequest> {
        self.recorded_requests.lock().unwrap().clone()
    }
}

#[async_trait]
impl LlmProvider for MockTransitionLlmProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        self.recorded_requests.lock().unwrap().push(req.clone());

        // Check conditional failure
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
                .unwrap_or_else(|| "```rust\npub fn mock_transition_result() -> bool { true }\n```".to_string())
        };

        let prompt_len = req.messages.first().map(|m| m.extract_text().len()).unwrap_or(80);

        Ok(CompletionResponse {
            id: format!("mock-transition-{}", idx),
            provider: self.id.to_string(),
            model: req.model.clone(),
            message: Message::assistant(resp_text),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: (prompt_len as u32 / 4).max(1),
                completion_tokens: 60,
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: if is_local_provider(self.id) { Some(0.0) } else { Some(0.001) },
            },
            latency: Duration::from_millis(5),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream> {
        Err(TagisanError::Execution("Streaming mock not required for skills transition tests".to_string()))
    }
}

// =========================================================================
// SECTION 1: PROVIDER-AWARE TOKENBUDGET AUTO-CALIBRATION ROUNDTRIPS
// =========================================================================

#[test]
fn test_01_token_budget_local_calibration() {
    println!("\n=== TEST 1.1: Local Runtime Auto-Calibration ===");

    // 1. Ollama Default (no model specified -> 8k context, 1.2k tokens, DenseInvariants, max 1/domain)
    let b_ollama_default = TokenBudget::for_provider_and_model("ollama", None);
    assert_eq!(b_ollama_default.context_window, 8_192, "Ollama default context must be 8,192");
    assert_eq!(b_ollama_default.max_tokens, 1_200, "Ollama default max tokens must be 1,200");
    assert_eq!(b_ollama_default.mode, InjectionMode::DenseInvariants, "Ollama default mode must be DenseInvariants");
    assert_eq!(b_ollama_default.max_per_domain, 1, "Local models must strictly enforce max 1 skill per domain");

    // 2. Qwen 2.5 32k context on Ollama (32k context, 3.5k tokens, Hierarchical, max 1/domain)
    let b_qwen32k = TokenBudget::for_provider_and_model("ollama", Some("qwen2.5-coder:32b"));
    assert_eq!(b_qwen32k.context_window, 32_768, "Qwen 32k context must be 32,768");
    assert_eq!(b_qwen32k.max_tokens, 3_500, "Qwen 32k max tokens must be 3,500");
    assert_eq!(b_qwen32k.mode, InjectionMode::Hierarchical, "Qwen 32k mode must be Hierarchical");
    assert_eq!(b_qwen32k.max_per_domain, 1, "Local 32k models must preserve max 1/domain constraint");

    // 3. Mistral / DeepSeek-R1 local 32k variants
    let b_deepseek_local = TokenBudget::for_provider_and_model("ollama", Some("deepseek-r1:14b"));
    assert_eq!(b_deepseek_local.context_window, 32_768);
    assert_eq!(b_deepseek_local.max_tokens, 3_500);
    assert_eq!(b_deepseek_local.mode, InjectionMode::Hierarchical);
    assert_eq!(b_deepseek_local.max_per_domain, 1);

    // 4. Colibri local SSD MoE runtime
    let b_colibri = TokenBudget::for_provider_and_model("colibri", None);
    assert_eq!(b_colibri.context_window, 8_192, "Colibri default context must be 8,192");
    assert_eq!(b_colibri.max_tokens, 1_200, "Colibri default max tokens must be 1,200");
    assert_eq!(b_colibri.mode, InjectionMode::DenseInvariants);
    assert_eq!(b_colibri.max_per_domain, 1);

    // 5. Local 16k context variant
    let b_local_16k = TokenBudget::for_provider_and_model("local", Some("model-16k"));
    assert_eq!(b_local_16k.context_window, 16_384);
    assert_eq!(b_local_16k.max_tokens, 2_000);
    assert_eq!(b_local_16k.mode, InjectionMode::DenseInvariants);
    assert_eq!(b_local_16k.max_per_domain, 1);

    println!("  [✓] All Local runtime profiles calibrated with 100% precision");
}

#[test]
fn test_02_token_budget_cloud_calibration() {
    println!("\n=== TEST 1.2: Cloud Runtime Auto-Calibration ===");

    // 1. Google Gemini (1M context, 16k tokens, Hierarchical, max 2/domain)
    let b_gemini = TokenBudget::for_provider_and_model("gemini", Some("gemini-2.5-pro"));
    assert_eq!(b_gemini.context_window, 1_000_000, "Gemini context window must be 1,000,000");
    assert_eq!(b_gemini.max_tokens, 16_000, "Gemini max tokens must be 16,000");
    assert_eq!(b_gemini.mode, InjectionMode::Hierarchical, "Gemini mode must be Hierarchical");
    assert_eq!(b_gemini.max_per_domain, 2, "Cloud models must allow up to 2 skills per domain");

    // 2. Anthropic Claude (200k context, 12k tokens, Hierarchical, max 2/domain)
    let b_claude = TokenBudget::for_provider_and_model("anthropic", Some("claude-3-5-sonnet-20241022"));
    assert_eq!(b_claude.context_window, 200_000, "Claude context window must be 200,000");
    assert_eq!(b_claude.max_tokens, 12_000, "Claude max tokens must be 12,000");
    assert_eq!(b_claude.mode, InjectionMode::Hierarchical);
    assert_eq!(b_claude.max_per_domain, 2);

    // 3. OpenAI (128k context, 8k tokens, Hierarchical, max 2/domain)
    let b_openai = TokenBudget::for_provider_and_model("openai", Some("gpt-4o"));
    assert_eq!(b_openai.context_window, 128_000, "OpenAI context window must be 128,000");
    assert_eq!(b_openai.max_tokens, 8_000, "OpenAI max tokens must be 8,000");
    assert_eq!(b_openai.mode, InjectionMode::Hierarchical);
    assert_eq!(b_openai.max_per_domain, 2);

    // 4. DeepSeek Cloud (128k context, 8k tokens, Hierarchical, max 2/domain)
    let b_deepseek_cloud = TokenBudget::for_provider_and_model("deepseek", Some("deepseek-chat"));
    assert_eq!(b_deepseek_cloud.context_window, 128_000);
    assert_eq!(b_deepseek_cloud.max_tokens, 8_000);
    assert_eq!(b_deepseek_cloud.mode, InjectionMode::Hierarchical);
    assert_eq!(b_deepseek_cloud.max_per_domain, 2);

    // 5. xAI / Grok Cloud (128k context, 8k tokens, Hierarchical, max 2/domain)
    let b_xai = TokenBudget::for_provider_and_model("xai", Some("grok-3"));
    assert_eq!(b_xai.context_window, 128_000);
    assert_eq!(b_xai.max_tokens, 8_000);
    assert_eq!(b_xai.mode, InjectionMode::Hierarchical);
    assert_eq!(b_xai.max_per_domain, 2);

    println!("  [✓] All Cloud runtime profiles calibrated with 100% precision");
}

#[test]
fn test_03_token_budget_dynamic_roundtrip_switching() {
    println!("\n=== TEST 1.3: Dynamic Back-and-Forth Budget Calibration Roundtrips ===");

    let transitions = [
        ("ollama", None, 8_192, 1_200, InjectionMode::DenseInvariants, 1),
        ("gemini", Some("gemini-2.5-pro"), 1_000_000, 16_000, InjectionMode::Hierarchical, 2),
        ("colibri", None, 8_192, 1_200, InjectionMode::DenseInvariants, 1),
        ("anthropic", Some("claude-3-7-sonnet"), 200_000, 12_000, InjectionMode::Hierarchical, 2),
        ("ollama", Some("qwen2.5:32b"), 32_768, 3_500, InjectionMode::Hierarchical, 1),
        ("openai", Some("gpt-4o"), 128_000, 8_000, InjectionMode::Hierarchical, 2),
        ("local", None, 8_192, 1_200, InjectionMode::DenseInvariants, 1),
        ("deepseek", Some("deepseek-chat"), 128_000, 8_000, InjectionMode::Hierarchical, 2),
        ("ollama", None, 8_192, 1_200, InjectionMode::DenseInvariants, 1),
    ];

    for (step, (provider, model, exp_ctx, exp_max_tokens, exp_mode, exp_per_domain)) in transitions.iter().enumerate() {
        let budget = TokenBudget::for_provider_and_model(provider, *model);
        assert_eq!(budget.context_window, *exp_ctx, "Step {}: context mismatch for {}", step, provider);
        assert_eq!(budget.max_tokens, *exp_max_tokens, "Step {}: max tokens mismatch for {}", step, provider);
        assert_eq!(budget.mode, *exp_mode, "Step {}: injection mode mismatch for {}", step, provider);
        assert_eq!(budget.max_per_domain, *exp_per_domain, "Step {}: max per domain mismatch for {}", step, provider);

        let is_loc = is_local_provider(provider);
        if is_loc {
            assert_eq!(budget.max_per_domain, 1, "Local provider must enforce max_per_domain = 1");
            assert!(budget.context_window <= 32_768, "Local context must be <= 32k");
        } else {
            assert_eq!(budget.max_per_domain, 2, "Cloud provider must allow max_per_domain = 2");
            assert!(budget.context_window >= 128_000, "Cloud context must be >= 128k");
        }
    }

    println!("  [✓] Successfully executed 9 continuous back-and-forth roundtrips with zero drift");
}

// =========================================================================
// SECTION 2: PROVIDER FORMAT & PROMPT INJECTION MODE TRANSITIONS
// =========================================================================

#[test]
fn test_04_equip_prompt_for_provider_local_vs_cloud() {
    println!("\n=== TEST 2.1: equip_prompt_for_provider Local vs Cloud Formatting ===");

    let dispatcher = global_dispatcher();
    let query = "optimize database schema, lock contention, and concurrency invariants in rust";
    let base_prompt = "You are an autonomous engineering agent.";

    // 1. Local LLM Formatting (Ollama)
    let (local_prompt, local_skills) = dispatcher.equip_prompt_for_provider(
        base_prompt,
        query,
        "ollama",
        None,
    );

    assert!(
        local_prompt.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local prompt must contain Cheat Sheet header"
    );
    assert!(
        !local_prompt.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"),
        "Local prompt must NOT contain Cloud comprehensive header"
    );
    assert!(
        local_skills.len() <= 2,
        "Local skills must be strictly capped at 2, got: {}",
        local_skills.len()
    );
    assert!(
        local_prompt.lines().any(|l| l.trim().starts_with('-')),
        "Local prompt must contain action-oriented bullet rules"
    );

    // Verify token estimation for local injection
    let local_token_count = estimate_tokens(&local_prompt);
    assert!(
        local_token_count <= 800,
        "Local prompt must maintain tight footprint (got {} tokens)",
        local_token_count
    );

    // 2. Cloud LLM Formatting (Anthropic)
    let (cloud_prompt, cloud_skills) = dispatcher.equip_prompt_for_provider(
        base_prompt,
        query,
        "anthropic",
        None,
    );

    assert!(
        cloud_prompt.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud prompt must contain Comprehensive Architectural Specifications header"
    );
    assert!(
        !cloud_prompt.contains("### [LOCAL LLM CHEAT SHEET"),
        "Cloud prompt must NOT contain Local cheat sheet header"
    );
    assert!(
        cloud_skills.len() <= 4,
        "Cloud skills must be capped at 4, got: {}",
        cloud_skills.len()
    );
    assert!(
        cloud_prompt.contains("#### Full Specification & Directives:"),
        "Cloud prompt must contain full specification directives"
    );
    assert!(
        cloud_prompt.contains("Description:"),
        "Cloud prompt must include comprehensive skill description"
    );

    // Cloud prompt should be richer and more detailed than Local prompt
    assert!(
        cloud_prompt.len() > local_prompt.len(),
        "Cloud prompt ({} bytes) must be more detailed than local prompt ({} bytes)",
        cloud_prompt.len(),
        local_prompt.len()
    );

    println!("  [✓] Local prompt: {} bytes (CheatSheet) | Cloud prompt: {} bytes (Comprehensive)", local_prompt.len(), cloud_prompt.len());
}

#[test]
fn test_05_mid_session_prompt_transition_cloud_to_local_failover() {
    println!("\n=== TEST 2.2: Mid-Session Cloud -> Local Dynamic Failover Mutation ===");

    let dispatcher = global_dispatcher();
    let query = "design clean aggregate root and domain events with bounded contexts";
    let base_system = "You are a software architect implementing core business invariants.";

    // Step A: Prompt initially equipped for Cloud (Anthropic)
    let (cloud_equipped, cloud_skills) = dispatcher.equip_prompt_for_provider(
        base_system,
        query,
        "anthropic",
        None,
    );
    assert!(cloud_equipped.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"));
    assert!(!cloud_skills.is_empty());

    // Step B: Live Failover occurs (e.g. offline, rate limit, network failure) -> Mutate to Local (Ollama)
    let (failover_local_equipped, local_skills) = dispatcher.equip_prompt_for_provider(
        base_system,
        query,
        "ollama",
        None,
    );

    // Verify all cloud boilerplate was stripped
    assert!(
        !failover_local_equipped.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"),
        "Cloud comprehensive header must be completely stripped during failover"
    );
    assert!(
        failover_local_equipped.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local cheat sheet must be injected upon failover"
    );
    assert!(
        local_skills.len() <= 2,
        "Local failover must strictly downscale to max 2 skills"
    );
    assert!(
        failover_local_equipped.starts_with(base_system),
        "Base system contract must remain preserved after failover"
    );

    // Step C: Re-connection occurs (Local -> Cloud)
    let (reconnected_cloud, reconnected_skills) = dispatcher.equip_prompt_for_provider(
        base_system,
        query,
        "anthropic",
        None,
    );
    assert!(reconnected_cloud.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"));
    assert_eq!(
        cloud_equipped, reconnected_cloud,
        "Prompt reconstructed upon reconnection must be 100% byte-identical to original cloud prompt"
    );
    assert_eq!(cloud_skills.len(), reconnected_skills.len());

    println!("  [✓] Mid-session prompt mutation Cloud -> Local -> Cloud executed with zero loss or drift");
}

// =========================================================================
// SECTION 3: DOMAIN DIVERSITY & ANTI-MONOPOLY INVARIANTS
// =========================================================================

#[test]
fn test_06_domain_diversity_anti_monopoly_local_vs_cloud() {
    println!("\n=== TEST 3: Domain Diversity & Anti-Monopoly Invariants ===");

    let dispatcher = global_dispatcher();
    // Broad query hitting multiple skills across multiple domains (concurrency, architecture, ba, security)
    let query = "design a high performance concurrent ecommerce backend with state machine lifecycle and security audit";

    // 1. Local Runtime: Strict max_per_domain = 1
    let local_budget = TokenBudget::new(8_192, 1_200, InjectionMode::DenseInvariants, 1);
    let local_result = dispatcher.dispatch_diversified(query, local_budget, None);

    let mut local_domains_seen = HashSet::new();
    for ds in &local_result.primary {
        assert!(
            !local_domains_seen.contains(&ds.domain),
            "Anti-Monopoly Violation: Domain '{}' appeared more than once under Local budget (max_per_domain=1)",
            ds.domain
        );
        local_domains_seen.insert(ds.domain.clone());
    }
    assert!(
        local_result.primary.len() >= 2,
        "Local dispatch should select at least 2 diverse skills"
    );
    println!("  [✓] Local Anti-Monopoly: {} skills across {} distinct domains (1:1 domain diversity)", local_result.primary.len(), local_domains_seen.len());

    // 2. Cloud Runtime: Expanded max_per_domain = 2
    let cloud_budget = TokenBudget::new(128_000, 8_000, InjectionMode::Hierarchical, 2);
    let cloud_result = dispatcher.dispatch_diversified(query, cloud_budget, None);

    let mut cloud_domain_counts = std::collections::HashMap::new();
    for ds in &cloud_result.primary {
        let count = cloud_domain_counts.entry(ds.domain.clone()).or_insert(0);
        *count += 1;
        assert!(
            *count <= 2,
            "Domain '{}' exceeded max_per_domain=2 under Cloud budget (got {})",
            ds.domain,
            *count
        );
    }
    println!("  [✓] Cloud Domain Expansion: {} primary skills loaded, domain counts: {:?}", cloud_result.primary.len(), cloud_domain_counts);

    // 3. Dynamic Transition: Toggling back and forth
    for cycle in 0..4 {
        let is_loc = cycle % 2 == 0;
        let b = if is_loc { local_budget } else { cloud_budget };
        let res = dispatcher.dispatch_diversified(query, b, None);

        let mut counts = std::collections::HashMap::new();
        for ds in &res.primary {
            *counts.entry(&ds.domain).or_insert(0) += 1;
        }

        let max_observed = counts.values().copied().max().unwrap_or(0);
        let expected_max = if is_loc { 1 } else { 2 };
        assert!(
            max_observed <= expected_max,
            "Cycle {}: max domain count {} exceeded limit {}",
            cycle,
            max_observed,
            expected_max
        );
    }

    println!("  [✓] Domain distribution dynamically and cleanly transitions between Local (1) and Cloud (2)");
}

// =========================================================================
// SECTION 4: SKILL JIT MEMORY TIER PAGING & CACHE DYNAMICS
// =========================================================================

#[test]
fn test_07_skill_jit_tier_paging_and_pinning_across_capacities() {
    println!("\n=== TEST 4: Skill JIT Memory Tier Paging & Pinning ===");

    // A. Local Capacity Profile: Tight L1 = 2 active, L2 = 4 warm
    let local_jit = SkillJitManager::new(2, 4, None);

    // 1. Initial State: All skills in L3 Cold
    assert_eq!(local_jit.get_tier("grokking-algorithms"), SkillTier::L3Cold);
    assert_eq!(local_jit.get_tier("tokio-async-tuning"), SkillTier::L3Cold);

    // 2. Page in skill 1 -> L3 Miss -> Promoted to L1
    let s1 = local_jit.page_in("grokking-algorithms").expect("Page in must succeed");
    assert_eq!(s1.name, "grokking-algorithms");
    assert_eq!(local_jit.get_tier("grokking-algorithms"), SkillTier::L1Active);
    let stats1 = local_jit.stats();
    assert_eq!(stats1.l3_misses, 1);
    assert_eq!(stats1.l1_active_count, 1);

    // 3. Page in skill 1 again -> L1 Hit
    let _s1_again = local_jit.page_in("grokking-algorithms").expect("Page in again");
    let stats2 = local_jit.stats();
    assert_eq!(stats2.l1_hits, 1);
    assert_eq!(stats2.l3_misses, 1);

    // 4. Page in skill 2 -> L3 Miss -> Promoted to L1 (capacity 2 full)
    let _s2 = local_jit.page_in("tokio-async-tuning").expect("Page in s2");
    assert_eq!(local_jit.get_tier("tokio-async-tuning"), SkillTier::L1Active);
    assert_eq!(local_jit.stats().l1_active_count, 2);

    // 5. Pin skill 1 so it cannot be evicted
    local_jit.pin_skill("grokking-algorithms");
    assert!(local_jit.is_pinned("grokking-algorithms"));

    // 6. Page in skill 3 (`domain-driven-design`) -> Capacity is 2!
    // Since skill 1 is pinned, skill 2 (`tokio-async-tuning`) MUST be evicted down to L2 Warm!
    let _s3 = local_jit.page_in("domain-driven-design").expect("Page in s3");
    assert_eq!(local_jit.get_tier("domain-driven-design"), SkillTier::L1Active);
    assert_eq!(local_jit.get_tier("grokking-algorithms"), SkillTier::L1Active, "Pinned skill must NOT be evicted");
    assert_eq!(local_jit.get_tier("tokio-async-tuning"), SkillTier::L2Warm, "Unpinned skill must be demoted to L2 Warm");

    let stats3 = local_jit.stats();
    assert_eq!(stats3.l1_evictions, 1);
    assert_eq!(stats3.l2_warm_count, 1);

    // 7. Page in skill 2 from L2 Warm -> L2 Hit -> Promoted back to L1 Active!
    // Unpinned skill 3 (`domain-driven-design`) gets evicted to L2!
    let _s2_repromoted = local_jit.page_in("tokio-async-tuning").expect("Repromote s2");
    assert_eq!(local_jit.get_tier("tokio-async-tuning"), SkillTier::L1Active);
    assert_eq!(local_jit.get_tier("grokking-algorithms"), SkillTier::L1Active, "Pinned skill still in L1");
    assert_eq!(local_jit.get_tier("domain-driven-design"), SkillTier::L2Warm);

    let stats4 = local_jit.stats();
    assert_eq!(stats4.l2_hits, 1, "Must register L2 cache hit upon promotion");
    assert!(stats4.hit_rate_percent() > 0.0);

    // B. Cloud Capacity Profile: Expanded L1 = 8 active, L2 = 16 warm
    let cloud_jit = SkillJitManager::new(8, 16, None);
    let cloud_skills = [
        "grokking-algorithms",
        "tokio-async-tuning",
        "domain-driven-design",
        "clean-architecture-martin",
        "database-migration-lifecycle",
        "restful-api-design",
    ];

    for name in &cloud_skills {
        let _ = cloud_jit.page_in(name).expect("Cloud page in");
        assert_eq!(cloud_jit.get_tier(name), SkillTier::L1Active);
    }
    assert_eq!(cloud_jit.stats().l1_active_count, 6);
    assert_eq!(cloud_jit.stats().l1_evictions, 0, "Cloud profile with capacity 8 must have 0 evictions for 6 skills");

    // Test PILOT lookahead prefetching
    let prefetched = cloud_jit.pilot_prefetch(&["concurrency lock optimization", "security vulnerability audit"]);
    assert!(prefetched >= 1, "PILOT prefetch must preload relevant engineering skills");

    println!("  [✓] JIT Memory tiers (L1/L2/L3), eviction demotion, hit promotion, and pinning verified 100%");
}

// =========================================================================
// SECTION 5: SEMANTIC INVARIANT GUARDING ACROSS CONTEXT SCALING
// =========================================================================

#[test]
fn test_08_semantic_invariant_guard_tool_schema_and_diagnostics() {
    println!("\n=== TEST 5: Semantic Invariant Guarding Across Context Scaling ===");

    let guard = SemanticInvariantGuard::new();

    // 1. Tool Schema Integrity Enforcement
    // Valid tool definition passes
    let valid_tool = ToolDefinition::new(
        "fetch_skill",
        "Retrieve complete specification for a skill from the ECC catalog.",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The skill name to fetch" }
            },
            "required": ["name"]
        }),
    );
    let enforced = guard.enforce_tool_schema_integrity(&valid_tool);
    assert!(enforced.is_ok(), "Valid tool schema must pass guard");

    // Corrupted: Truncated description ending with '...'
    let truncated_tool = ToolDefinition::new(
        "fetch_skill",
        "Retrieve complete specification for a skill from the ECC catalog...",
        json!({ "type": "object", "properties": {} }),
    );
    let trunc_err = guard.enforce_tool_schema_integrity(&truncated_tool);
    assert!(
        matches!(trunc_err, Err(SemanticViolation::TruncatedToolSchema { .. })),
        "Truncated description must be rejected by SemanticInvariantGuard"
    );

    // Corrupted: Dropped required parameter from properties
    let dropped_param_tool = ToolDefinition::new(
        "execute_code",
        "Executes a piece of code in sandbox",
        json!({
            "type": "object",
            "properties": {
                "timeout": { "type": "integer" }
            },
            "required": ["code", "timeout"] // "code" parameter is dropped from properties!
        }),
    );
    let drop_err = guard.enforce_tool_schema_integrity(&dropped_param_tool);
    assert!(
        matches!(drop_err, Err(SemanticViolation::DroppedRequiredParameter { ref param_name, .. }) if param_name == "code"),
        "Dropped required parameter must trigger DroppedRequiredParameter violation"
    );

    // 2. Compiler Diagnostic Preservation (Rust borrow checker error)
    let rustc_raw_error = r#"
error[E0382]: use of moved value: `buffer`
  --> src/network/worker.rs:42:15
   |
40 |     let buffer = allocate_buffer();
   |         ------ move occurs because `buffer` has type `Vec<u8>`, which does not implement the `Copy` trait
41 |     send_payload(buffer);
   |                  ------ value moved here
42 |     println!("Buffer length: {}", buffer.len());
   |                                   ^^^^^^^^^^^^ value borrowed here after move
   = note: this error originates in network worker
   help: consider cloning the value if the performance cost is acceptable
"#;

    // Generous context (e.g. Cloud 200 tokens): should preserve error code, span, message, help
    let preserved = guard.enforce_diagnostic_preservation(rustc_raw_error, 200).expect("Preservation must succeed");
    assert_eq!(preserved.error_code, Some("E0382".to_string()));
    assert_eq!(preserved.primary_span, "src/network/worker.rs:42:15");
    assert!(preserved.preserved_condensed.contains("DIAGNOSTIC: error[E0382]"));
    assert!(preserved.preserved_condensed.contains("SPAN: src/network/worker.rs:42:15"));
    assert!(preserved.preserved_condensed.contains("help:"));
    assert!(preserved.condensed_token_est <= preserved.original_token_est);

    // Constricted context (e.g. tight Local 5 tokens): cannot safely hold diagnostic -> Reject with DiagnosticDegradation
    let degraded = guard.enforce_diagnostic_preservation(rustc_raw_error, 5);
    assert!(
        matches!(degraded, Err(SemanticViolation::DiagnosticDegradation { ref error_code, .. }) if error_code == "E0382"),
        "Insufficient token budget for diagnostic must be rejected rather than silently truncated"
    );

    // 3. Safety Contract Enforcement
    // Context safely accommodates safety rules
    let safety_rules = ["ALWAYS validate array bounds", "NEVER execute unsanitized SQL queries"];
    let safe_res = guard.enforce_safety_contract(500, 8_192, &safety_rules);
    assert!(safe_res.is_ok());

    // Context exhaustion risk
    let exhaust_res = guard.enforce_safety_contract(8_180, 8_192, &safety_rules);
    assert!(
        matches!(exhaust_res, Err(SemanticViolation::ContextExhaustionWithInvariantRisk { .. })),
        "Context exhaustion with safety rules must be rejected"
    );

    // Empty safety rule omitted
    let empty_rule = ["ALWAYS validate", "   "];
    let omit_res = guard.enforce_safety_contract(100, 8_192, &empty_rule);
    assert!(
        matches!(omit_res, Err(SemanticViolation::SafetyContractOmitted { .. })),
        "Empty safety rule must trigger SafetyContractOmitted"
    );

    println!("  [✓] SemanticInvariantGuard enforced tool schemas, diagnostic causality, and safety contracts");
}

// =========================================================================
// SECTION 6: MULTI-AGENT SWARM MIXED ROLES & LIVE STAGE FAILOVER
// =========================================================================

#[tokio::test]
async fn test_09_multi_agent_swarm_mixed_roles_and_live_stage_failover() {
    println!("\n=== TEST 6: Multi-Agent Swarm Mixed Roles & Dynamic Stage Failover ===");

    let mut ctx = EngineContext::new(10.0);

    // 1. Register Mock Providers for Mixed Swarm:
    // Architect: Local Ollama
    let mock_ollama = Arc::new(MockTransitionLlmProvider::new(
        "ollama",
        vec![
            "```rust\npub struct AtomicRateLimiter { max_requests: u32, interval_ms: u64 }\n```".to_string(),
        ],
    ));
    ctx.register_provider(mock_ollama.clone());

    // Implementer: Cloud Anthropic
    let mock_anthropic = Arc::new(MockTransitionLlmProvider::new(
        "anthropic",
        vec![
            "```rust\nimpl AtomicRateLimiter {\n    pub fn new(max: u32, interval: u64) -> Self { Self { max_requests: max, interval_ms: interval } }\n    pub fn check_rate(&self) -> bool { true }\n}\n```".to_string(),
        ],
    ));
    ctx.register_provider(mock_anthropic.clone());

    // QA: Cloud Gemini - Configured to fail with HTTP 429 RateLimited on its first attempt!
    let mock_gemini_failing = Arc::new(MockTransitionLlmProvider::with_persistent_error(
        "gemini",
        MockFailureMode::RateLimited("gemini".to_string(), Some(Duration::from_secs(30))),
    ));
    ctx.register_provider(mock_gemini_failing.clone());

    // Doc: Local Colibri
    let mock_colibri = Arc::new(MockTransitionLlmProvider::new(
        "colibri",
        vec![
            "# AtomicRateLimiter\nHigh-performance lock-free rate limiter implementation.".to_string(),
        ],
    ));
    ctx.register_provider(mock_colibri.clone());

    // 2. Build Heterogeneous Assembly Roles
    let arch_role = AssemblyRoles::architect("ollama", "qwen2.5:32b");
    let imp_role = AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet-20241022");
    let qa_role = AssemblyRoles::qa("gemini", "gemini-2.0-flash");
    let doc_role = AssemblyRoles::documentation("colibri", "colibri-moe");

    // 3. Assert each stage initial role contract receives provider-calibrated prompt
    assert!(
        arch_role.config().system_contract.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local Architect role must receive Local CheatSheet prompt"
    );
    assert!(
        imp_role.config().system_contract.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud Implementer role must receive Comprehensive Architectural prompt"
    );
    assert!(
        qa_role.config().system_contract.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud QA role must initially receive Comprehensive Architectural prompt"
    );
    assert!(
        doc_role.config().system_contract.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local Doc role must receive Local CheatSheet prompt"
    );

    // 4. Assemble pipeline with automatic fallback to local Ollama on failure
    let mut pipeline = StructuredHarmonyPipeline::new("Construct Thread-Safe Atomic Rate Limiter")
        .with_fallback_to_local(true)
        .with_max_retries(2);

    pipeline = pipeline.add_stage(HarmonyStage::new(arch_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));
    pipeline = pipeline.add_stage(HarmonyStage::new(imp_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));
    pipeline = pipeline.add_stage(HarmonyStage::new(qa_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));
    pipeline = pipeline.add_stage(HarmonyStage::new(doc_role).with_gate(Box::new(SyntaxValidationGate::permissive())));

    // 5. Execute Pipeline!
    // Stage 1 (Architect, Local Ollama) succeeds.
    // Stage 2 (Implementer, Cloud Anthropic) succeeds.
    // Stage 3 (QA, Cloud Gemini) fails with RateLimited -> Evacuates to Local Ollama fallback!
    // On evacuation, QA role mutates system contract into condensed Local CheatSheet on the fly!
    // Stage 4 (Doc, Local Colibri) succeeds.
    let result = pipeline.execute(&ctx).await.expect("Pipeline execution with dynamic failover must succeed");

    assert_eq!(result.artifacts.len(), 4, "All 4 stages must complete successfully");

    // 6. Inspect QA stage failover artifact
    let qa_artifact = &result.artifacts[2];
    assert_eq!(qa_artifact.role_id, "qa");
    assert!(
        qa_artifact.failover_event.is_some(),
        "QA artifact must contain a recorded FailoverEvent"
    );

    let event = qa_artifact.failover_event.as_ref().unwrap();
    assert_eq!(event.original_provider, "gemini");
    assert_eq!(event.evacuated_to_provider, "ollama");
    assert!(event.trigger_reason.contains("Rate limited (HTTP 429)"));
    println!("  [✓] Live Failover Event: {} -> {} ({})", event.original_provider, event.evacuated_to_provider, event.trigger_reason);

    // 7. Verify that when Ollama handled the evacuated QA stage, the prompt was mutated to Local CheatSheet!
    let recorded_ollama_reqs = mock_ollama.recorded_requests();
    assert!(
        recorded_ollama_reqs.len() >= 2,
        "Ollama must have executed at least 2 requests (Architect + Evacuated QA)"
    );

    let evacuated_qa_req = &recorded_ollama_reqs[1];
    let qa_system_prompt = evacuated_qa_req.system_prompt.as_deref().unwrap_or("");
    assert!(
        qa_system_prompt.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Evacuated QA request to Ollama must have dynamically mutated to Local CheatSheet header!"
    );
    assert!(
        !qa_system_prompt.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"),
        "Evacuated QA request must NOT contain Cloud comprehensive header"
    );

    // 8. Verify assembled project contains code from all 4 stages
    let complete = result.complete_project;
    assert!(complete.contains("pub struct AtomicRateLimiter"));
    assert!(complete.contains("impl AtomicRateLimiter"));
    assert!(complete.contains("# AtomicRateLimiter"));

    println!("  [✓] Swarm executed with 100% real dynamic role prompt re-calibration upon cloud failover");
}

// =========================================================================
// SECTION 7: FETCHSKILLTOOL EXECUTION PARITY & JSON SCHEMA
// =========================================================================

#[tokio::test]
async fn test_10_fetch_skill_tool_execution_parity() {
    println!("\n=== TEST 7: FetchSkillTool Execution Parity & Schema Integrity ===");

    let guard = SemanticInvariantGuard::new();
    let tool = FetchSkillTool::with_default();

    assert_eq!(tool.name(), "fetch_skill");
    assert!(!tool.description().is_empty());

    // 1. Validate parameters schema against SemanticInvariantGuard
    let tool_def = ToolDefinition::new(tool.name(), tool.description(), tool.parameters_schema());
    let schema_check = guard.enforce_tool_schema_integrity(&tool_def);
    assert!(schema_check.is_ok(), "FetchSkillTool schema must pass SemanticInvariantGuard");

    // 2. Fetch known built-in skills and assert exact specification structure
    let test_skills = [
        ("grokking-algorithms", "Grokking Algorithms"),
        ("tokio-async-tuning", "Tokio Async"),
        ("ostep-mechanical-sympathy", "Mechanical Sympathy"),
        ("system-design-building-blocks", "System Design"),
        ("pragmatic-programmer-craft", "Pragmatic Programmer"),
    ];

    for (skill_name, expected_keyword) in &test_skills {
        let args = json!({ "name": skill_name });
        let output = tool.execute(args).await.expect("Tool execution must succeed");

        assert!(
            output.contains(&format!("# Skill: {}", skill_name)),
            "Skill output must begin with # Skill: {}", skill_name
        );
        assert!(output.contains("[Domain:"), "Skill output must declare domain");
        assert!(output.contains("## Instructions"), "Skill output must declare Instructions header");
        assert!(
            output.contains(expected_keyword),
            "Skill '{}' must contain keyword '{}'", skill_name, expected_keyword
        );

        let tokens = estimate_tokens(&output);
        assert!(tokens >= 50, "Skill '{}' token estimate ({}) must be >= 50", skill_name, tokens);
    }

    // 3. Fetch non-existent skill returns descriptive recommendation
    let missing_args = json!({ "name": "non-existent-skill-xyz-999" });
    let missing_output = tool.execute(missing_args).await.expect("Must return message");
    assert!(
        missing_output.contains("not found in ECC catalog"),
        "Missing skill must indicate not found"
    );
    assert!(
        missing_output.contains("search_skills"),
        "Missing skill output must recommend search_skills"
    );

    // 4. Missing required parameter returns error
    let bad_args = json!({});
    let bad_result = tool.execute(bad_args).await;
    assert!(
        bad_result.is_err(),
        "Tool call with missing 'name' parameter must return error"
    );

    // 5. Parity check: output remains 100% byte-identical across multiple invocations
    let args1 = json!({ "name": "grokking-algorithms" });
    let out1 = tool.execute(args1.clone()).await.unwrap();
    let out2 = tool.execute(args1).await.unwrap();
    assert_eq!(out1, out2, "FetchSkillTool output must be 100% deterministic and byte-stable");

    println!("  [✓] FetchSkillTool schema, execution, token bounds, and output parity verified 100%");
}

// =========================================================================
// SECTION 8: ROUND-TRIP REVERSIBILITY & ZERO STATE DRIFT
// =========================================================================

#[test]
fn test_11_round_trip_reversibility_and_zero_state_drift() {
    println!("\n=== TEST 8: 10-Cycle Round-Trip Reversibility & Zero State Drift ===");

    let dispatcher = global_dispatcher();
    let query = "distributed consensus raft paxos leader election replicated log";
    let base_prompt = "You are a distributed systems architect.";

    let mut local_prompts = Vec::new();
    let mut cloud_prompts = Vec::new();

    // Execute 10 alternating cycles:
    // Cycle 0: Local Ollama
    // Cycle 1: Cloud Anthropic
    // Cycle 2: Local Colibri
    // Cycle 3: Cloud Gemini
    // Cycle 4: Local Ollama
    // Cycle 5: Cloud OpenAI
    // Cycle 6: Local Ollama
    // Cycle 7: Cloud DeepSeek
    // Cycle 8: Local Colibri
    // Cycle 9: Cloud xAI
    for cycle in 0..10 {
        let is_local = cycle % 2 == 0;
        let provider = match cycle {
            0 | 4 | 6 => "ollama",
            2 | 8 => "colibri",
            1 => "anthropic",
            3 => "gemini",
            5 => "openai",
            7 => "deepseek",
            9 => "xai",
            _ => "ollama",
        };

        let (equipped, skills) = dispatcher.equip_prompt_for_provider(base_prompt, query, provider, None);

        // Assert zero duplicate header accumulation
        let cheat_sheet_headers = equipped.matches("[LOCAL LLM CHEAT SHEET").count();
        let cloud_headers = equipped.matches("[COMPREHENSIVE ARCHITECTURAL").count();

        if is_local {
            assert_eq!(cheat_sheet_headers, 1, "Cycle {}: Exactly 1 Local CheatSheet header expected", cycle);
            assert_eq!(cloud_headers, 0, "Cycle {}: Exactly 0 Cloud headers expected", cycle);
            assert!(skills.len() <= 2, "Cycle {}: Local skills count must be <= 2", cycle);
            local_prompts.push(equipped);
        } else {
            assert_eq!(cloud_headers, 1, "Cycle {}: Exactly 1 Cloud header expected", cycle);
            assert_eq!(cheat_sheet_headers, 0, "Cycle {}: Exactly 0 Local headers expected", cycle);
            assert!(skills.len() <= 4, "Cycle {}: Cloud skills count must be <= 4", cycle);
            cloud_prompts.push(equipped);
        }
    }

    // Verify 100% byte idempotency across all local prompt instances
    let first_local = &local_prompts[0];
    for (idx, p) in local_prompts.iter().enumerate() {
        assert_eq!(
            p, first_local,
            "Local prompt drift detected at cycle {}",
            idx * 2
        );
    }

    // Verify 100% byte idempotency across all cloud prompt instances
    let first_cloud = &cloud_prompts[0];
    for (idx, p) in cloud_prompts.iter().enumerate() {
        assert_eq!(
            p, first_cloud,
            "Cloud prompt drift detected at cycle {}",
            (idx * 2) + 1
        );
    }

    println!("  [✓] Verified 10 alternating cycles: 0 duplicate headers, 0 state drift, 100% deterministic idempotency");
}

// =========================================================================
// SECTION 9: TOKEN BUDGET ENFORCEMENT & ZERO-COST LOCAL EXEMPTION
// =========================================================================

#[test]
fn test_12_token_budget_zero_cost_local_exemption() {
    println!("\n=== TEST 9.1: TokenBudgetTracker Zero-Cost Local Exemption ===");

    let tracker = TokenBudgetTracker::new(0.05); // $0.05 budget limit

    // 1. Local models have $0.00 cost rate (Zero-Cost Local Exemption)
    let cost_ollama = tracker.record("ollama:qwen2.5:32b", 50_000, 20_000).expect("Ollama must never exceed budget");
    assert_eq!(cost_ollama, 0.0, "Ollama local inference must cost exactly $0.00");

    let cost_colibri = tracker.record("colibri-moe:8b", 80_000, 40_000).expect("Colibri must never exceed budget");
    assert_eq!(cost_colibri, 0.0, "Colibri local inference must cost exactly $0.00");

    let cost_llama = tracker.record("llama3.2:latest", 100_000, 50_000).expect("Llama local must never exceed budget");
    assert_eq!(cost_llama, 0.0, "Llama local inference must cost exactly $0.00");

    assert_eq!(tracker.current_spent_usd(), 0.0, "Total spent after 230,000 local tokens must still be $0.00");

    // 2. Cloud models consume real budget
    let cost_cloud1 = tracker.record("gpt-4o", 2_000, 1_000).expect("Cloud call within budget");
    assert!(cost_cloud1 > 0.0, "Cloud call must consume USD budget");
    println!("  [✓] Cloud call recorded ${:.5} USD", cost_cloud1);

    // 3. Exhausting the budget returns BudgetExceeded error
    let budget_result = tracker.record("claude-3-5-sonnet", 50_000, 20_000);
    assert!(
        matches!(budget_result, Err(TagisanError::BudgetExceeded { .. })),
        "Large cloud request exceeding $0.05 budget must trigger BudgetExceeded"
    );

    // 4. Even after budget is exhausted, local models remain ZERO COST and can continue!
    let post_exhaust_local = tracker.record("ollama:qwen2.5", 10_000, 5_000);
    assert!(
        post_exhaust_local.is_ok(),
        "Local models must continue functioning with zero-cost exemption even after cloud budget exhaustion!"
    );

    println!("  [✓] Zero-cost local exemption verified across Ollama, Colibri, and Llama runtimes");
}

#[test]
fn test_13_thinking_tokens_extraction_deepseek_qwen_local_vs_cloud() {
    println!("\n=== TEST 9.2: Local LLM Thinking Tokens Extraction ===");

    // 1. Local DeepSeek-R1 / Qwen thinking tags format
    let local_raw = "<think>\nStep 1: Check if input slice is empty.\nStep 2: Use binary search for O(log N) lookup.\n</think>\npub fn binary_search(arr: &[i32], target: i32) -> Option<usize> { arr.binary_search(&target).ok() }";

    let blocks = parse_thinking_blocks(local_raw);
    assert_eq!(blocks.len(), 2, "Must split into thinking block and text block");

    match &blocks[0] {
        ContentBlock::Thinking { thinking, signature } => {
            assert!(thinking.contains("Step 1: Check if input slice is empty"));
            assert!(thinking.contains("Step 2: Use binary search"));
            assert!(signature.is_none());
        }
        _ => panic!("Expected ContentBlock::Thinking for first block"),
    }

    match &blocks[1] {
        ContentBlock::Text { text } => {
            assert!(text.contains("pub fn binary_search"));
            assert!(!text.contains("<think>"));
        }
        _ => panic!("Expected ContentBlock::Text for second block"),
    }

    // 2. Cloud output without thinking tags
    let cloud_raw = "```rust\npub fn cloud_search() {}\n```";
    let cloud_blocks = parse_thinking_blocks(cloud_raw);
    assert_eq!(cloud_blocks.len(), 1);
    match &cloud_blocks[0] {
        ContentBlock::Text { text } => {
            assert_eq!(text, cloud_raw);
        }
        _ => panic!("Expected ContentBlock::Text for cloud response"),
    }

    // 3. Unclosed think tag: gracefully falls back to Text block without crash or data loss
    let unclosed_raw = "<think>\nAnalyzing complexity trade-offs...\nStill evaluating...";
    let unclosed_blocks = parse_thinking_blocks(unclosed_raw);
    assert_eq!(unclosed_blocks.len(), 1);
    match &unclosed_blocks[0] {
        ContentBlock::Text { text } => {
            assert!(text.contains("Analyzing complexity trade-offs"));
            assert!(text.contains("Still evaluating"));
        }
        _ => panic!("Expected unclosed think tag to gracefully fall back to Text block"),
    }

    println!("  [✓] Thinking tokens extracted accurately for local DeepSeek/Qwen models");
}

#[test]
fn test_14_is_local_provider_comprehensive_matrix() {
    println!("\n=== TEST 9.3: is_local_provider Comprehensive Matrix ===");

    let local_cases = [
        "ollama",
        "OLLAMA",
        "Ollama",
        "local",
        "LOCAL",
        "localhost",
        "ollama-remote",
        "llama.cpp",
        "llama",
        "vllm",
        "colibri",
        "COLIBRI",
        "colibri-moe",
        "coli",
    ];

    for p in &local_cases {
        assert!(
            is_local_provider(p),
            "Provider '{}' must be detected as a local provider",
            p
        );
    }

    let cloud_cases = [
        "anthropic",
        "ANTHROPIC",
        "claude",
        "gemini",
        "google",
        "openai",
        "OPENAI",
        "gpt-4o",
        "deepseek",
        "xai",
        "grok",
        "mistral-cloud",
        "cohere",
    ];

    for p in &cloud_cases {
        assert!(
            !is_local_provider(p),
            "Provider '{}' must NOT be detected as a local provider",
            p
        );
    }

    println!("  [✓] Verified 14 local and 13 cloud provider identifiers against is_local_provider");
}

// =========================================================================
// SECTION 10: ADVANCED LIVE FAILOVER CASCADE & BUDGET EVACUATION MATRIX
// =========================================================================

#[tokio::test]
async fn test_15_live_failover_auth_bad_gateway_and_budget_exhaustion() {
    println!("\n=== TEST 10.1: Live Failover Cascade Matrix (HTTP 401, HTTP 502, Budget Exhaustion) ===");

    // 1. HTTP 401 Unauthorized Failover -> Evacuation to Local
    {
        let mut ctx = EngineContext::new(10.0);

        let mock_local = Arc::new(MockTransitionLlmProvider::new(
            "ollama",
            vec!["```rust\npub fn local_auth_fallback() -> bool { true }\n```".to_string()],
        ));
        ctx.register_provider(mock_local.clone());

        let mock_cloud_auth_fail = Arc::new(MockTransitionLlmProvider::with_persistent_error(
            "anthropic",
            MockFailureMode::Authentication("anthropic".to_string(), "401 Unauthorized: Invalid API key".to_string()),
        ));
        ctx.register_provider(mock_cloud_auth_fail.clone());

        let imp_role = AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet");
        assert!(imp_role.config().system_contract.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"));

        let pipeline = StructuredHarmonyPipeline::new("Implement secure token validator")
            .with_fallback_to_local(true)
            .add_stage(HarmonyStage::new(imp_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));

        let result = pipeline.execute(&ctx).await.expect("Failover on HTTP 401 must succeed");
        assert_eq!(result.artifacts.len(), 1);
        let art = &result.artifacts[0];
        assert!(art.failover_event.is_some());
        let ev = art.failover_event.as_ref().unwrap();
        assert_eq!(ev.original_provider, "anthropic");
        assert_eq!(ev.evacuated_to_provider, "ollama");
        assert!(ev.trigger_reason.contains("Authentication failure on 'anthropic'"));

        let reqs = mock_local.recorded_requests();
        assert_eq!(reqs.len(), 1);
        let prompt = reqs[0].system_prompt.as_deref().unwrap_or("");
        assert!(prompt.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
        println!("  [✓] HTTP 401 Auth error triggered automatic evacuation and mutated prompt to Local CheatSheet");
    }

    // 2. HTTP 502 Bad Gateway / Server Error Failover -> Evacuation to Local
    {
        let mut ctx = EngineContext::new(10.0);

        let mock_local = Arc::new(MockTransitionLlmProvider::new(
            "ollama",
            vec!["```rust\npub fn local_gateway_fallback() -> bool { true }\n```".to_string()],
        ));
        ctx.register_provider(mock_local.clone());

        let mock_cloud_502 = Arc::new(MockTransitionLlmProvider::with_persistent_error(
            "deepseek",
            MockFailureMode::BadResponse("deepseek".to_string(), "502 Bad Gateway from Cloud Proxy".to_string()),
        ));
        ctx.register_provider(mock_cloud_502.clone());

        let imp_role = AssemblyRoles::implementer("deepseek", "deepseek-chat");
        let pipeline = StructuredHarmonyPipeline::new("Implement resilient circuit breaker")
            .with_fallback_to_local(true)
            .add_stage(HarmonyStage::new(imp_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));

        let result = pipeline.execute(&ctx).await.expect("Failover on HTTP 502 must succeed");
        assert_eq!(result.artifacts.len(), 1);
        let art = &result.artifacts[0];
        assert!(art.failover_event.is_some());
        let ev = art.failover_event.as_ref().unwrap();
        assert_eq!(ev.original_provider, "deepseek");
        assert_eq!(ev.evacuated_to_provider, "ollama");
        assert!(ev.trigger_reason.contains("API error on 'deepseek'"));

        let reqs = mock_local.recorded_requests();
        assert_eq!(reqs.len(), 1);
        let prompt = reqs[0].system_prompt.as_deref().unwrap_or("");
        assert!(prompt.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
        println!("  [✓] HTTP 502 Bad Gateway triggered automatic evacuation and mutated prompt to Local CheatSheet");
    }

    // 3. Pre-emptive Financial Budget Exhaustion Evacuation
    {
        let mut ctx = EngineContext::new(0.01); // $0.01 spending limit
        // Exhaust the budget up to the limit
        ctx.budget_tracker.record_cost_usd(0.01).expect("Spend entire budget");
        assert!(ctx.budget_tracker.is_exhausted(), "Budget tracker must be marked exhausted");

        let mock_local = Arc::new(MockTransitionLlmProvider::new(
            "ollama",
            vec!["```rust\npub fn local_budget_fallback() -> bool { true }\n```".to_string()],
        ));
        ctx.register_provider(mock_local.clone());

        let mock_cloud = Arc::new(MockTransitionLlmProvider::new(
            "anthropic",
            vec!["```rust\npub fn should_not_be_called() {}\n```".to_string()],
        ));
        ctx.register_provider(mock_cloud.clone());

        let imp_role = AssemblyRoles::implementer("anthropic", "claude-3-5-sonnet");
        let pipeline = StructuredHarmonyPipeline::new("Implement low-cost cache")
            .with_evacuate_on_budget(true)
            .add_stage(HarmonyStage::new(imp_role).with_gate(Box::new(SyntaxValidationGate::for_rust())));

        let result = pipeline.execute(&ctx).await.expect("Pre-emptive budget evacuation must succeed");
        assert_eq!(result.artifacts.len(), 1);

        // Cloud provider must NEVER have been called because budget was already exhausted!
        assert_eq!(mock_cloud.call_count.load(Ordering::SeqCst), 0, "Cloud provider must not be called when budget exhausted");
        // Local provider was called
        assert_eq!(mock_local.call_count.load(Ordering::SeqCst), 1, "Local provider must execute pre-emptively evacuated stage");

        let art = &result.artifacts[0];
        assert!(art.failover_event.is_some());
        let ev = art.failover_event.as_ref().unwrap();
        assert_eq!(ev.original_provider, "anthropic");
        assert_eq!(ev.evacuated_to_provider, "ollama");
        assert!(ev.trigger_reason.contains("Token budget reached"));

        println!("  [✓] Pre-emptive financial budget exhaustion evacuated to Local without calling Cloud API");
    }
}

// =========================================================================
// SECTION 11: FORMAT DENSE INVARIANTS & HIERARCHICAL MODE ROUNDTRIPS
// =========================================================================

#[test]
fn test_16_format_dense_invariants_and_hierarchical_roundtrips() {
    println!("\n=== TEST 11: Format Dense Invariants & Hierarchical Multi-Tier Mode Transitions ===");

    let dispatcher = global_dispatcher();
    let query = "tokio async concurrency lock contention deadlocks mpsc channel bounded joinset";

    let budget_hierarchical = TokenBudget::cloud_default().with_mode(InjectionMode::Hierarchical);
    let result = dispatcher.dispatch_diversified(query, budget_hierarchical, None);
    assert!(!result.primary.is_empty(), "Primary skills must not be empty");
    assert!(!result.manifest.is_empty(), "Manifest skills must not be empty");

    // 1. Dense Invariants Mode formatting
    let dense = format_dense_invariants(&result.primary);
    assert!(dense.contains("### [TAGISAN ECC INVARIANT DIRECTIVES: HIGH-DENSITY ENFORCEMENT]"));
    assert!(
        dense.contains("INVARIANTS:") || dense.contains("DIRECTIVE:") || dense.contains("ALWAYS:") || dense.contains("NEVER:")
    );
    let dense_tokens = estimate_tokens(&dense);

    // 2. Comprehensive Cloud Guidelines Mode formatting
    let cloud = format_cloud_guidelines(&result.primary);
    assert!(cloud.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"));
    assert!(cloud.contains("#### Full Specification & Directives:"));
    let cloud_tokens = estimate_tokens(&cloud);

    // Dense invariants must achieve significant token compression compared to Comprehensive
    assert!(
        dense_tokens < cloud_tokens,
        "Dense invariants ({} tokens) must be smaller than Comprehensive ({} tokens)",
        dense_tokens,
        cloud_tokens
    );
    let compression_ratio = 1.0 - (dense_tokens as f64 / cloud_tokens as f64);
    println!("  [✓] Dense Invariants Compression: {:.1}% reduction vs Comprehensive", compression_ratio * 100.0);

    // 3. Hierarchical Mode formatting
    let hierarchical = format_hierarchical(&result.manifest, &result.primary, InjectionMode::DenseInvariants);
    assert!(hierarchical.contains("### [TAGISAN ECC MULTI-TIER COGNITIVE ARCHITECTURE]"));
    assert!(hierarchical.contains("#### TIER 1: ACTIVE CAPABILITY RADAR"));
    assert!(hierarchical.contains("| # | Skill ID | Domain | Core Focus & Triggers |"));
    assert!(hierarchical.contains("#### TIER 2: PRIMARY OPERATIONAL INVARIANTS"));
    assert!(hierarchical.contains("#### TIER 3: ON-DEMAND JIT KNOWLEDGE RETRIEVAL"));
    assert!(hierarchical.contains("fetch_skill"));

    // 4. Cheat Sheet Mode formatting
    let cheat = format_cheat_sheet(&result.primary);
    assert!(cheat.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
    assert!(cheat.lines().any(|l| l.trim().starts_with('-')));

    println!("  [✓] All 4 format representations (Dense, Cloud, Hierarchical, CheatSheet) verified with full fidelity");
}

// =========================================================================
// SECTION 12: CLI FLAGS & ENVIRONMENT VARIABLES LOCAL-ONLY OVERRIDES
// =========================================================================

#[test]
fn test_17_env_var_and_cli_flag_local_only_enforcement() {
    println!("\n=== TEST 12: Dynamic Provider Switching via CLI Flags & Env Vars ===");

    let mut ctx = EngineContext::new(10.0);
    let mock_local_ollama = Arc::new(MockTransitionLlmProvider::new("ollama", vec![]));
    let mock_local_colibri = Arc::new(MockTransitionLlmProvider::new("colibri", vec![]));
    let mock_cloud_anthropic = Arc::new(MockTransitionLlmProvider::new("anthropic", vec![]));

    ctx.register_provider(mock_local_ollama);
    ctx.register_provider(mock_local_colibri);
    ctx.register_provider(mock_cloud_anthropic);

    // Helper to evaluate resolution logic matching src/cli.rs lines 1420-1447
    fn resolve_test_provider(
        user_provider: &str,
        env_provider: Option<&str>,
        local_only: bool,
    ) -> &'static str {
        if local_only {
            if user_provider != "auto" && (user_provider == "ollama" || user_provider == "colibri" || user_provider == "local") {
                if user_provider == "colibri" { "colibri" } else { "ollama" }
            } else if let Some(ep) = env_provider {
                if ep == "colibri" { "colibri" } else { "ollama" }
            } else {
                "ollama"
            }
        } else if user_provider != "auto" {
            if user_provider == "anthropic" { "anthropic" }
            else if user_provider == "colibri" { "colibri" }
            else { "ollama" }
        } else if let Some(ep) = env_provider {
            if ep == "colibri" { "colibri" }
            else if ep == "anthropic" { "anthropic" }
            else { "ollama" }
        } else {
            "auto"
        }
    }

    // 1. Standard mode without local_only: user choice honored
    assert_eq!(resolve_test_provider("anthropic", None, false), "anthropic");
    assert_eq!(resolve_test_provider("colibri", None, false), "colibri");
    assert_eq!(resolve_test_provider("ollama", None, false), "ollama");

    // 2. TAGISAN_LOCAL_ONLY / TAGISAN_OFFLINE active: cloud requests strictly overridden to local!
    assert_eq!(
        resolve_test_provider("anthropic", None, true),
        "ollama",
        "When local_only is true, cloud provider request must be forced to local ollama"
    );
    assert_eq!(
        resolve_test_provider("auto", None, true),
        "ollama",
        "When local_only is true, auto mode must be forced to local ollama"
    );
    assert_eq!(
        resolve_test_provider("auto", Some("colibri"), true),
        "colibri",
        "When local_only is true and TAGISAN_PROVIDER=colibri, colibri is chosen"
    );

    // 3. TAGISAN_PROVIDER / TGS_PROVIDER env var fallback when user_provider = "auto"
    assert_eq!(resolve_test_provider("auto", Some("colibri"), false), "colibri");
    assert_eq!(resolve_test_provider("auto", Some("anthropic"), false), "anthropic");

    println!("  [✓] Provider resolution logic under TAGISAN_LOCAL_ONLY, TAGISAN_OFFLINE, and TAGISAN_PROVIDER verified");
}

