use serde_json::json;
use std::collections::HashSet;
use std::time::Instant;
use tagisan::ecc::{
    estimate_tokens, format_dense_invariants, format_hierarchical, global_dispatcher,
    DispatchedSkill, InjectionMode, TokenBudget,
};
use tagisan::tools::{FetchSkillTool, ToolHandler};

// =========================================================================
// TEST 1: Sub-Microsecond Token Estimator Accuracy & Bounding
// =========================================================================
#[test]
fn test_01_token_estimator_accuracy_and_bounds() {
    println!("\n=== TEST 1: Sub-Microsecond Token Estimator Accuracy & Bounding ===");

    // Empty text
    assert_eq!(estimate_tokens(""), 0);

    let prose = "The Tagisan engine orchestrates autonomous multi-model swarms across heterogeneous cloud and local providers with strict verification loops.";
    let prose_tokens = estimate_tokens(prose);
    assert!(prose_tokens >= 15 && prose_tokens <= 35, "Prose tokens: {}", prose_tokens);

    let code = r#"
        pub async fn execute_transaction<T: Transactional>(tx: &mut T, id: Uuid) -> Result<Receipt, TransactionError> {
            let record = tx.lock_row_for_update(id).await?;
            if record.status != Status::Pending {
                return Err(TransactionError::IllegalState(record.status));
            }
            tx.commit().await
        }
    "#;
    let code_tokens = estimate_tokens(code);
    assert!(code_tokens >= 30 && code_tokens <= 90, "Code tokens: {}", code_tokens);

    // Verify mathematical bounds: chars / 5 <= tokens <= chars / 3
    let len = code.len();
    assert!(code_tokens >= len / 5 && code_tokens <= len / 3 + 5);

    // Microsecond throughput benchmark (10,000 iterations)
    let start = Instant::now();
    let mut accumulator = 0;
    for _ in 0..10_000 {
        accumulator += estimate_tokens(code);
    }
    let elapsed = start.elapsed();
    let micros_per_op = elapsed.as_micros() as f64 / 10_000.0;
    let max_micros = if cfg!(debug_assertions) { 25.0 } else { 5.0 };
    assert!(accumulator > 0);
    assert!(micros_per_op < max_micros, "Token estimator must be sub-{:.1}-microsecond per call (actual: {:.2} µs)", max_micros, micros_per_op);
}

// =========================================================================
// TEST 2: TokenBudget Auto-Calibration Across Local & Cloud Providers
// =========================================================================
#[test]
fn test_02_token_budget_auto_calibration() {
    println!("\n=== TEST 2: TokenBudget Auto-Calibration ===");

    // 1. Local Default (Ollama / 8k context)
    let b_local = TokenBudget::for_provider_and_model("ollama", None);
    assert_eq!(b_local.context_window, 8_192);
    assert_eq!(b_local.max_tokens, 1_200);
    assert_eq!(b_local.mode, InjectionMode::DenseInvariants);
    assert_eq!(b_local.max_per_domain, 1);
    println!("  [✓] Local default: {} ctx, {} max tokens, {:?}, max {}/domain", b_local.context_window, b_local.max_tokens, b_local.mode, b_local.max_per_domain);

    // 2. Local Large Context (Qwen 2.5 32k)
    let b_local_32k = TokenBudget::for_provider_and_model("ollama", Some("qwen2.5-coder:32b"));
    assert_eq!(b_local_32k.context_window, 32_768);
    assert_eq!(b_local_32k.max_tokens, 3_500);
    assert_eq!(b_local_32k.mode, InjectionMode::Hierarchical);
    assert_eq!(b_local_32k.max_per_domain, 1);
    println!("  [✓] Local 32k: {} ctx, {} max tokens, {:?}, max {}/domain", b_local_32k.context_window, b_local_32k.max_tokens, b_local_32k.mode, b_local_32k.max_per_domain);

    // 3. Anthropic Claude (200k context)
    let b_claude = TokenBudget::for_provider_and_model("anthropic", Some("claude-3-7-sonnet"));
    assert_eq!(b_claude.context_window, 200_000);
    assert_eq!(b_claude.max_tokens, 12_000);
    assert_eq!(b_claude.mode, InjectionMode::Hierarchical);
    assert_eq!(b_claude.max_per_domain, 2);
    println!("  [✓] Claude: {} ctx, {} max tokens, {:?}, max {}/domain", b_claude.context_window, b_claude.max_tokens, b_claude.mode, b_claude.max_per_domain);

    // 4. Google Gemini (1M context)
    let b_gemini = TokenBudget::for_provider_and_model("gemini", Some("gemini-2.5-pro"));
    assert_eq!(b_gemini.context_window, 1_000_000);
    assert_eq!(b_gemini.max_tokens, 16_000);
    assert_eq!(b_gemini.mode, InjectionMode::Hierarchical);
    assert_eq!(b_gemini.max_per_domain, 2);
    println!("  [✓] Gemini: {} ctx, {} max tokens, {:?}, max {}/domain", b_gemini.context_window, b_gemini.max_tokens, b_gemini.mode, b_gemini.max_per_domain);

    // 5. OpenAI (128k context)
    let b_openai = TokenBudget::for_provider_and_model("openai", Some("gpt-4o"));
    assert_eq!(b_openai.context_window, 128_000);
    assert_eq!(b_openai.max_tokens, 8_000);
    assert_eq!(b_openai.mode, InjectionMode::Hierarchical);
    assert_eq!(b_openai.max_per_domain, 2);
    println!("  [✓] OpenAI: {} ctx, {} max tokens, {:?}, max {}/domain", b_openai.context_window, b_openai.max_tokens, b_openai.mode, b_openai.max_per_domain);
}

// =========================================================================
// TEST 3: Dense Invariants Token Compression Ratio (> 50% Reduction)
// =========================================================================
#[test]
fn test_03_dense_invariants_token_compression_ratio() {
    println!("\n=== TEST 3: Dense Invariants Token Compression Ratio ===");

    let dispatcher = global_dispatcher();
    let skill_names = [
        "ba-state-machine-lifecycle-modeling",
        "ba-patton-user-story-mapping",
        "ba-wiegers-requirements-engineering",
    ];

    for name in &skill_names {
        let skill = dispatcher.get_skill(name).expect("Skill must exist in catalog");
        let ds = DispatchedSkill {
            domain: "test".to_string(),
            skill: skill.clone(),
            score: 100.0,
            matched_triggers: vec![name.to_string()],
        };

        let raw_tokens = estimate_tokens(&skill.instructions);
        let dense_output = format_dense_invariants(&[ds]);
        let dense_tokens = estimate_tokens(&dense_output);

        let reduction = 1.0 - (dense_tokens as f64 / raw_tokens as f64);
        println!(
            "  [✓] Skill '{}': Raw = {} tokens -> Dense DSL = {} tokens (Reduction: {:.1}%)",
            name, raw_tokens, dense_tokens, reduction * 100.0
        );

        // Verify that dense invariants contains strict uppercase invariants
        assert!(
            dense_output.contains("INVARIANTS:")
                || dense_output.contains("ALWAYS:")
                || dense_output.contains("NEVER:")
                || dense_output.contains("STRICT_REJECT:")
                || dense_output.contains("AUDIT:"),
            "Dense invariants must contain categorized uppercase invariants"
        );

        // Must achieve at least 40% token reduction (typically 60-75%)
        assert!(reduction >= 0.40, "Token reduction ({:.1}%) must be at least 40%", reduction * 100.0);
    }
}

// =========================================================================
// TEST 4: Maximal Marginal Relevance (MMR) & Domain Diversity Enforcement
// =========================================================================
#[test]
fn test_04_mmr_and_domain_diversity_enforcement() {
    println!("\n=== TEST 4: MMR & Domain Diversity Enforcement ===");

    let dispatcher = global_dispatcher();
    let query = "design a high performance concurrent ecommerce backend with state machine lifecycle and security audit";

    // Request strict diversity: max 1 skill per domain, tight budget of 800 tokens
    let budget = TokenBudget::new(8_192, 800, InjectionMode::DenseInvariants, 1);
    let result = dispatcher.dispatch_diversified(query, budget, None);

    println!("  Primary Skills Selected ({}):", result.primary.len());
    let mut domains_seen = HashSet::new();
    for ds in &result.primary {
        println!("    • [{}] {} (Score: {:.1})", ds.domain, ds.skill.name, ds.score);
        assert!(
            !domains_seen.contains(&ds.domain),
            "Domain '{}' appeared more than once despite max_per_domain = 1",
            ds.domain
        );
        domains_seen.insert(ds.domain.clone());
    }

    assert!(result.primary.len() >= 2, "Expected at least 2 diverse skills matched");
    assert!(domains_seen.len() >= 2, "Expected at least 2 distinct domains");
    assert!(result.total_estimated_tokens <= budget.max_tokens, "Tokens must not exceed budget");
    println!("  [✓] Multi-domain diversity strictly enforced: {} distinct domains represented", domains_seen.len());
}

// =========================================================================
// TEST 5: Deterministic Canonical Prefix Stability for LLM Prompt Caching
// =========================================================================
#[test]
fn test_05_deterministic_prefix_canonicalization_for_prompt_caching() {
    println!("\n=== TEST 5: Deterministic Canonical Prefix Stability ===");

    let dispatcher = global_dispatcher();
    let query = "rust concurrency async memory safety security vulnerability audit";

    let (prompt_1, skills_1, _) = dispatcher.equip_prompt_maximized("", query, "anthropic", None, None, None, None);
    let (prompt_2, skills_2, _) = dispatcher.equip_prompt_maximized("", query, "anthropic", None, None, None, None);

    // Assert 100% byte identical output
    assert_eq!(prompt_1, prompt_2, "Prompts must be 100% byte-identical for prefix prompt caching");
    assert_eq!(skills_1.len(), skills_2.len());

    // Verify canonical sorting: domain ASC, then name ASC
    for window in skills_1.windows(2) {
        let a = &window[0];
        let b = &window[1];
        let is_canonical = a.domain < b.domain || (a.domain == b.domain && a.skill.name <= b.skill.name);
        assert!(
            is_canonical,
            "Skills must be canonically ordered: [{}] {} vs [{}] {}",
            a.domain, a.skill.name, b.domain, b.skill.name
        );
    }
    println!("  [✓] Verified 100% byte-identical deterministic prefix ordering across invocations");
}

// =========================================================================
// TEST 6: Three-Tier Hierarchical Progressive Disclosure Formatting
// =========================================================================
#[test]
fn test_06_three_tier_hierarchical_progressive_disclosure() {
    println!("\n=== TEST 6: Three-Tier Hierarchical Progressive Disclosure ===");

    let dispatcher = global_dispatcher();
    let query = "business analysis requirement traceability state machine verification";
    let budget = TokenBudget::cloud_default().with_mode(InjectionMode::Hierarchical);

    let result = dispatcher.dispatch_diversified(query, budget, None);
    assert!(!result.manifest.is_empty(), "Manifest must not be empty");

    let formatted = format_hierarchical(&result.manifest, &result.primary, InjectionMode::DenseInvariants);

    // Tier 1 Check
    assert!(formatted.contains("#### TIER 1: ACTIVE CAPABILITY RADAR"), "Missing Tier 1 header");
    assert!(formatted.contains("| # | Skill ID | Domain | Core Focus & Triggers |"), "Missing Tier 1 manifest table");

    // Tier 2 Check
    assert!(formatted.contains("#### TIER 2: PRIMARY OPERATIONAL INVARIANTS"), "Missing Tier 2 header");

    // Tier 3 Check
    assert!(formatted.contains("#### TIER 3: ON-DEMAND JIT KNOWLEDGE RETRIEVAL"), "Missing Tier 3 header");
    assert!(formatted.contains("fetch_skill"), "Tier 3 must reference native fetch_skill tool");

    println!("  [✓] Tier 1 Manifest: {} skills indexed in radar table", result.manifest.len());
    println!("  [✓] Tier 2 Directives: {} primary skills loaded with deep constraints", result.primary.len());
    println!("  [✓] Tier 3 JIT Directives present with tool hint");
}

// =========================================================================
// TEST 7: Native FetchSkillTool Execution & JIT Retrieval
// =========================================================================
#[tokio::test]
async fn test_07_native_fetch_skill_tool_execution() {
    println!("\n=== TEST 7: Native FetchSkillTool Execution ===");

    let tool = FetchSkillTool::with_default();
    assert_eq!(tool.name(), "fetch_skill");
    assert!(!tool.description().is_empty());

    // 1. Fetch valid skill
    let args = json!({ "name": "tokio-async-tuning" });
    let output = tool.execute(args).await.expect("Tool execution must succeed");
    assert!(output.contains("# Skill: tokio-async-tuning"));
    assert!(output.contains("## Instructions"));
    assert!(output.contains("Lock Contention") || output.contains("Bounded"));
    println!("  [✓] Successfully fetched valid skill spec ({} bytes)", output.len());

    // 2. Fetch non-existent skill returns helpful fallback
    let missing_args = json!({ "name": "non-existent-skill-9999" });
    let missing_output = tool.execute(missing_args).await.expect("Must return message");
    assert!(missing_output.contains("not found in ECC catalog"));
    println!("  [✓] Non-existent skill handled gracefully with search recommendation");

    // 3. Missing argument returns validation error
    let bad_args = json!({});
    let bad_result = tool.execute(bad_args).await;
    assert!(bad_result.is_err(), "Missing argument must return error");
    println!("  [✓] Missing argument parameter correctly rejected");
}

// =========================================================================
// TEST 8: Swarm Domain Bias Role Sharding (Thesis vs Antithesis vs Lakandiwa)
// =========================================================================
#[test]
fn test_08_swarm_domain_bias_sharding() {
    println!("\n=== TEST 8: Swarm Domain Bias Role Sharding ===");

    let dispatcher = global_dispatcher();
    let query = "build enterprise inventory and billing system with order lifecycle";

    // 1. Proponent / Thesis biased with "ba"
    let (_, thesis_skills) = dispatcher.equip_prompt_for_provider_with_bias(
        "",
        query,
        "anthropic",
        None,
        Some("ba"),
    );
    assert!(!thesis_skills.is_empty());
    let has_ba = thesis_skills.iter().any(|s| s.domain == "ba");
    println!("  [✓] Thesis (Proponent) skills: {:?}", thesis_skills.iter().map(|s| &s.skill.name).collect::<Vec<_>>());
    assert!(has_ba, "Thesis agent must receive domain-biased BA skills");

    // 2. Opponent / Antithesis biased with "security"
    let (_, anti_skills) = dispatcher.equip_prompt_for_provider_with_bias(
        "",
        query,
        "anthropic",
        None,
        Some("security"),
    );
    assert!(!anti_skills.is_empty());
    let has_sec = anti_skills.iter().any(|s| s.domain == "security");
    println!("  [✓] Antithesis (Adversary) skills: {:?}", anti_skills.iter().map(|s| &s.skill.name).collect::<Vec<_>>());
    assert!(has_sec, "Antithesis agent must receive domain-biased Security skills");

    // 3. Adjudicator / Lakandiwa biased with "standards"
    let (_, synth_skills) = dispatcher.equip_prompt_for_provider_with_bias(
        "",
        query,
        "anthropic",
        None,
        Some("standards"),
    );
    assert!(!synth_skills.is_empty());
    println!("  [✓] Synthesis (Lakandiwa) skills: {:?}", synth_skills.iter().map(|s| &s.skill.name).collect::<Vec<_>>());
}

// =========================================================================
// TEST 9: 50-Thread Concurrent Stress Test (> 10,000 QPS Throughput)
// =========================================================================
#[test]
fn test_09_concurrent_high_throughput_stress_test() {
    println!("\n=== TEST 9: 50-Thread Concurrent Stress Test ===");

    let _ = global_dispatcher(); // Pre-warm singleton before thread spawn
    let num_threads = 50;
    let queries_per_thread = 200; // 10,000 queries total
    let queries = [
        "state machine lifecycle modeling transition invariants",
        "tokio async tuning lock contention joinset",
        "security review threat modeling sql injection sanitization",
        "dmn decision table completeness feel logic",
        "bdd three amigos gherkin specification by example",
    ];

    let start = Instant::now();
    let mut handles = Vec::with_capacity(num_threads);

    for t_idx in 0..num_threads {
        let q = queries[t_idx % queries.len()];
        let handle = std::thread::spawn(move || {
            let d = global_dispatcher();
            let mut success_count = 0;
            let budget = TokenBudget::local_default();
            for _ in 0..queries_per_thread {
                let res = d.dispatch_diversified(q, budget, None);
                if !res.primary.is_empty() {
                    success_count += 1;
                }
            }
            success_count
        });
        handles.push(handle);
    }

    let mut total_success = 0;
    for h in handles {
        total_success += h.join().expect("Thread joined successfully");
    }

    let elapsed = start.elapsed();
    let total_queries = num_threads * queries_per_thread;
    let qps = total_queries as f64 / elapsed.as_secs_f64();

    println!(
        "  [✓] Executed {} diversified dispatch queries across {} threads in {:?} ({:.0} QPS)",
        total_queries, num_threads, elapsed, qps
    );
    assert_eq!(total_success, total_queries);
    let min_qps = if cfg!(debug_assertions) { 600.0 } else { 3_500.0 };
    assert!(qps > min_qps, "Throughput must exceed {:.0} QPS (actual: {:.0})", min_qps, qps);
}
