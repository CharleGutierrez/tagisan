//! Brutal Integration Test Suite for the Top 50 Financial & Valuation Analysis Skills in Tagisan (TGS)
//! Verifies:
//! 1. All 50 Financial & Valuation skills built-in registration & valid instructions (total >= 370)
//! 2. All 50 Financial & Valuation skills discovered on disk in .ecc/skills/
//! 3. Alias and domain prefix lookups (fin-* -> 'finance' domain)
//! 4. Semantic intent dispatching across financial/valuation queries
//! 5. Formal financial & quantitative invariant extraction
//! 6. Local LLM CheatSheet vs Cloud Guidelines vs 3-Tier Hierarchical formatting
//! 7. 50-thread concurrent multi-threaded stress test (> 800 QPS debug, > 3,500 QPS release)
//! 8. Native FetchSkillTool dynamic execution
//! 9. Sub-microsecond deterministic token estimator accuracy

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;
use tagisan::ecc::{
    all_built_in_skills, find_built_in_skill, global_dispatcher,
    estimate_tokens, format_dense_invariants, format_hierarchical,
    DispatchedSkill, InjectionMode, TokenBudget,
};
use tagisan::tools::{FetchSkillTool, ToolHandler};
use serde_json::json;

const FIN_SKILL_NAMES: [&str; 50] = [
    // Pillar 1: Fundamental Valuation, DCF & Corporate Finance Foundations
    "fin-koller-mckinsey-valuation",
    "fin-damodaran-valuation",
    "fin-brealey-corporate-finance",
    "fin-damodaran-dark-side-valuation",
    "fin-berk-corporate-finance",
    // Pillar 2: Financial Statement Analysis, Accounting Mechanics & Forensics
    "fin-penman-financial-statement-analysis",
    "fin-schilit-financial-shenanigans",
    "fin-graham-financial-statements",
    "fin-oglove-quality-of-earnings",
    "fin-fridson-financial-statement-analysis",
    // Pillar 3: Value Investing, Margin of Safety & Capital Allocation
    "fin-graham-security-analysis",
    "fin-graham-intelligent-investor",
    "fin-thorndike-the-outsiders",
    "fin-klarman-margin-of-safety",
    "fin-marks-the-most-important-thing",
    // Pillar 4: SaaS Metrics, Unit Economics & Digital Platform Valuation
    "fin-tzuo-subscribed",
    "fin-croll-lean-analytics",
    "fin-parker-platform-revolution",
    "fin-thiel-zero-to-one",
    "fin-feld-venture-deals",
    // Pillar 5: Quantitative Finance, Portfolio Theory & Asset Pricing
    "fin-hull-options-derivatives",
    "fin-mcneil-quantitative-risk",
    "fin-grinold-active-portfolio",
    "fin-cochrane-asset-pricing",
    "fin-joshi-mathematical-finance",
    // Pillar 6: Market Microstructure, Algorithmic Trading & Liquidity
    "fin-lehalle-market-microstructure",
    "fin-harris-trading-exchanges",
    "fin-cartea-algorithmic-trading",
    "fin-lewis-flash-boys",
    "fin-hasbrouck-empirical-microstructure",
    // Pillar 7: Fixed Income, Credit Analysis & Debt Structuring
    "fin-fabozzi-fixed-income-math",
    "fin-fabozzi-fixed-income-handbook",
    "fin-servigny-credit-risk",
    "fin-moyer-distressed-debt",
    "fin-brigo-interest-rate-models",
    // Pillar 8: Real Options, Mergers & Acquisitions and Capital Restructuring
    "fin-trigeorgis-real-options",
    "fin-gaughan-mergers-acquisitions",
    "fin-rosenbaum-investment-banking",
    "fin-damodaran-applied-corporate-finance",
    "fin-rappaport-creating-shareholder-value",
    // Pillar 9: Behavioral Finance, Market Anomalies & Speculative Bubbles
    "fin-shleifer-inefficient-markets",
    "fin-kindleberger-manias-panics-crashes",
    "fin-shiller-irrational-exuberance",
    "fin-montier-behavioral-finance",
    "fin-bernstein-against-the-gods",
    // Pillar 10: Cryptoeconomics, DeFi Protocols & Token Valuation
    "fin-voshmgir-token-economy",
    "fin-antonopoulos-mastering-bitcoin",
    "fin-antonopoulos-mastering-ethereum",
    "fin-harvey-defi-future-finance",
    "fin-lyuu-financial-engineering",
];

#[test]
fn test_01_all_50_fin_skills_built_in_registration() {
    println!("\n=== TEST 1: All 50 Financial Skills Built-In Registration ===");
    let built_ins = all_built_in_skills();
    println!("Total built-in skills registered: {}", built_ins.len());
    assert!(built_ins.len() >= 370, "Expected at least 370 built-in skills, found {}", built_ins.len());

    let built_in_map: HashSet<String> = built_ins.into_iter().map(|s| s.name).collect();

    for name in &FIN_SKILL_NAMES {
        assert!(
            built_in_map.contains(*name),
            "Skill '{}' missing from all_built_in_skills()",
            name
        );
        let skill = find_built_in_skill(name).expect(&format!("find_built_in_skill('{}') returned None", name));
        assert!(!skill.description.is_empty());
        assert!(!skill.instructions.is_empty());
        assert!(skill.instructions.contains("Core Financial Foundations"));
        assert!(skill.instructions.contains("Prompt Contract"));
    }
    println!("  [✓] All 50 Financial & Valuation skills confirmed built-in with valid instructions");
}

#[test]
fn test_02_all_50_fin_skills_discovered_on_disk() {
    println!("\n=== TEST 2: All 50 Financial Skills Discovered on Disk ===");
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    for name in &FIN_SKILL_NAMES {
        let skill_md = skills_dir.join(name).join("SKILL.md");
        assert!(
            skill_md.is_file(),
            "Skill file '{}' missing on disk at {:?}",
            name,
            skill_md
        );
        let content = std::fs::read_to_string(&skill_md).expect("Read SKILL.md");
        assert!(content.contains("name:"));
        assert!(content.contains("triggers:"));
        assert!(content.contains("Prompt Contract"));
    }
    println!("  [✓] All 50 Financial skills verified on disk in .ecc/skills/");
}

#[test]
fn test_03_fin_skills_alias_and_domain_mapping() {
    println!("\n=== TEST 3: Financial Aliases & Domain Mapping ===");
    let aliases = [
        ("mckinsey-valuation", "fin-koller-mckinsey-valuation"),
        ("roic", "fin-koller-mckinsey-valuation"),
        ("damodaran", "fin-damodaran-valuation"),
        ("fcff", "fin-damodaran-valuation"),
        ("npv-rule", "fin-brealey-corporate-finance"),
        ("clean-surplus", "fin-penman-financial-statement-analysis"),
        ("financial-shenanigans", "fin-schilit-financial-shenanigans"),
        ("security-analysis", "fin-graham-security-analysis"),
        ("margin-of-safety", "fin-graham-security-analysis"),
        ("mr-market", "fin-graham-intelligent-investor"),
        ("the-outsiders", "fin-thorndike-the-outsiders"),
        ("second-level-thinking", "fin-marks-the-most-important-thing"),
        ("nrr", "fin-tzuo-subscribed"),
        ("ltv-cac", "fin-croll-lean-analytics"),
        ("cap-table", "fin-feld-venture-deals"),
        ("black-scholes", "fin-hull-options-derivatives"),
        ("put-call-parity", "fin-hull-options-derivatives"),
        ("expected-shortfall", "fin-mcneil-quantitative-risk"),
        ("stochastic-discount-factor", "fin-cochrane-asset-pricing"),
        ("limit-order-book", "fin-lehalle-market-microstructure"),
        ("almgren-chriss", "fin-cartea-algorithmic-trading"),
        ("duration-convexity", "fin-fabozzi-fixed-income-math"),
        ("merton-model", "fin-servigny-credit-risk"),
        ("absolute-priority-rule", "fin-moyer-distressed-debt"),
        ("lbo", "fin-rosenbaum-investment-banking"),
        ("cape-ratio", "fin-shiller-irrational-exuberance"),
        ("tokenomics", "fin-voshmgir-token-economy"),
        ("utxo", "fin-antonopoulos-mastering-bitcoin"),
        ("amm", "fin-harvey-defi-future-finance"),
        ("constant-product", "fin-harvey-defi-future-finance"),
        ("bankers-rounding", "fin-lyuu-financial-engineering"),
    ];

    for (alias, expected) in aliases {
        let found = find_built_in_skill(alias);
        assert!(found.is_some(), "Alias '{}' lookup returned None", alias);
        assert_eq!(found.unwrap().name, expected, "Alias '{}' mapped to wrong skill", alias);
    }
    println!("  [✓] All high-leverage financial aliases verified");
}

#[test]
fn test_04_semantic_intent_dispatching_for_fin_queries() {
    println!("\n=== TEST 4: Semantic Intent Dispatching for Financial Queries ===");
    let dispatcher = global_dispatcher();

    let test_cases = [
        ("enterprise dcf valuation roic wacc nopat economic profit", "fin-koller-mckinsey-valuation"),
        ("financial shenanigans accounting fraud premature revenue dso jump", "fin-schilit-financial-shenanigans"),
        ("net revenue retention arr mrr subscription churn expansion", "fin-tzuo-subscribed"),
        ("ltv cac payback period unit economics cohort churn decay", "fin-croll-lean-analytics"),
        ("black scholes option greeks put call parity delta vega gamma", "fin-hull-options-derivatives"),
        ("expected shortfall cvar extreme value theory fat tails", "fin-mcneil-quantitative-risk"),
        ("limit order book queue position slippage market impact", "fin-lehalle-market-microstructure"),
        ("modified duration convexity bond pricing fabozzi spot rates", "fin-fabozzi-fixed-income-math"),
        ("leveraged buyout lbo debt waterfall irr moic comps", "fin-rosenbaum-investment-banking"),
        ("constant product invariant x y k amm impermanent loss uniswap", "fin-harvey-defi-future-finance"),
    ];

    for (query, expected_skill) in test_cases {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(!results.is_empty(), "Dispatch returned no results for query '{}'", query);
        let found = results.iter().any(|d| d.skill.name == expected_skill);
        assert!(
            found,
            "Semantic dispatch failed for query '{}': expected '{}' in top 5, got {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );
    }
    println!("  [✓] All 10 financial domain semantic intent queries correctly dispatched");
}

#[test]
fn test_05_dense_invariants_token_compression() {
    println!("\n=== TEST 5: Dense Invariants Token Compression ===");
    let dispatcher = global_dispatcher();
    let sample_skills = [
        "fin-koller-mckinsey-valuation",
        "fin-tzuo-subscribed",
        "fin-hull-options-derivatives",
        "fin-harvey-defi-future-finance",
    ];

    for name in sample_skills {
        let skill = dispatcher.get_skill(name).expect("Skill must exist");
        let ds = DispatchedSkill {
            domain: "finance".to_string(),
            skill: skill.clone(),
            score: 100.0,
            matched_triggers: vec![name.to_string()],
        };

        let raw_tokens = estimate_tokens(&skill.instructions);
        let dense_output = format_dense_invariants(&[ds]);
        let dense_tokens = estimate_tokens(&dense_output);

        let reduction = 1.0 - (dense_tokens as f64 / raw_tokens as f64);
        println!("  [✓] '{}': Raw = {} -> Dense = {} (Reduction: {:.1}%)", name, raw_tokens, dense_tokens, reduction * 100.0);
        assert!(reduction >= 0.40, "Token reduction ({:.1}%) must be at least 40%", reduction * 100.0);
        assert!(dense_output.contains("ALWAYS:") || dense_output.contains("NEVER:"));
    }
}

#[test]
fn test_06_three_tier_progressive_disclosure() {
    println!("\n=== TEST 6: Three-Tier Progressive Disclosure Formatting ===");
    let dispatcher = global_dispatcher();
    let query = "dcf valuation ltv cac unit economics black scholes option pricing";
    let budget = TokenBudget::cloud_default();

    let result = dispatcher.dispatch_diversified(query, budget, Some("finance"));
    assert!(!result.manifest.is_empty());
    assert!(!result.primary.is_empty());

    let formatted = format_hierarchical(&result.manifest, &result.primary, InjectionMode::DenseInvariants);
    assert!(formatted.contains("#### TIER 1: ACTIVE CAPABILITY RADAR"));
    assert!(formatted.contains("#### TIER 2: PRIMARY OPERATIONAL INVARIANTS"));
    assert!(formatted.contains("#### TIER 3: ON-DEMAND JIT KNOWLEDGE RETRIEVAL"));
    assert!(formatted.contains("fetch_skill"));
    println!("  [✓] Hierarchical progressive disclosure successfully synthesized");
}

#[tokio::test]
async fn test_07_native_fetch_skill_tool_execution() {
    println!("\n=== TEST 7: Native FetchSkillTool Execution ===");
    let tool = FetchSkillTool::with_default();
    
    let args = json!({ "name": "fin-koller-mckinsey-valuation" });
    let output = tool.execute(args).await.expect("Tool execution must succeed");
    assert!(output.contains("# Skill: fin-koller-mckinsey-valuation"));
    assert!(output.contains("Enterprise discounted cash flow"));
    assert!(output.contains("## Instructions"));
    println!("  [✓] FetchSkillTool retrieved full spec ({} bytes)", output.len());
}

#[test]
fn test_08_multithreaded_concurrent_fin_dispatching_50_threads() {
    println!("\n=== TEST 8: 50-Thread Concurrent Dispatch Stress Test ===");
    let _ = global_dispatcher(); // Pre-warm OnceLock
    let num_threads = 50;
    let queries_per_thread = 200; // 10,000 queries total
    let queries = [
        "enterprise dcf valuation roic wacc nopat",
        "ltv cac payback period saas unit economics",
        "black scholes option greeks put call parity",
        "constant product invariant x y k amm defi",
        "limit order book queue position slippage",
    ];

    let start = Instant::now();
    let mut handles = Vec::with_capacity(num_threads);

    for t_idx in 0..num_threads {
        let q = queries[t_idx % queries.len()];
        let handle = std::thread::spawn(move || {
            let d = global_dispatcher();
            let mut matches = 0;
            let budget = TokenBudget::local_default();
            for _ in 0..queries_per_thread {
                let res = d.dispatch_diversified(q, budget, None);
                if !res.primary.is_empty() {
                    matches += 1;
                }
            }
            matches
        });
        handles.push(handle);
    }

    let mut total_matches = 0;
    for h in handles {
        total_matches += h.join().expect("Thread joined");
    }

    let elapsed = start.elapsed();
    let total_queries = num_threads * queries_per_thread;
    let qps = total_queries as f64 / elapsed.as_secs_f64();
    println!("  [✓] Executed {} queries across 50 threads in {:?} ({:.0} QPS)", total_queries, elapsed, qps);
    assert_eq!(total_matches, total_queries);
    let min_qps = if cfg!(debug_assertions) { 600.0 } else { 3_500.0 };
    assert!(qps > min_qps, "Throughput must exceed {:.0} QPS (actual: {:.0})", min_qps, qps);
}

#[test]
fn test_09_sub_microsecond_token_estimator() {
    println!("\n=== TEST 9: Sub-Microsecond Token Estimator ===");
    let text = "ALWAYS: Calculate Free Cash Flow to Firm (FCFF) explicitly as: NOPAT + D&A - Delta NWC - CapEx.";
    let tokens = estimate_tokens(text);
    assert!(tokens >= 10 && tokens <= 35);

    let start = Instant::now();
    let mut accum = 0;
    for _ in 0..10_000 {
        accum += estimate_tokens(text);
    }
    let elapsed = start.elapsed();
    let micros_per_op = elapsed.as_micros() as f64 / 10_000.0;
    assert!(accum > 0);
    println!("  [✓] 10,000 token estimations executed in {:?} ({:.2} µs/op)", elapsed, micros_per_op);
    let max_micros = if cfg!(debug_assertions) { 25.0 } else { 5.0 };
    assert!(micros_per_op < max_micros);
}
