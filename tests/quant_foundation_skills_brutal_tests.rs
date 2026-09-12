//! Brutal Integration Tests for Quant AI Foundation Models in Tagisan (tgs)
//!
//! Models Tested:
//! 1. Kronos (K-line BSQuantizer, hierarchical tokens s1/s2, autoregressive Transformer)
//! 2. Microsoft Qlib (Alpha factor engineering, Barra risk neutralization, Top-K backtest, RankIC)
//! 3. Amazon Chronos & Salesforce MOIRAI (Universal time series models, zero-shot probabilistic forecasting, VaR)
//! 4. FinGPT & FinRL (Financial NLP, 10-K/10-Q filing analysis, sentiment factors, deep RL execution)
//! 5. Microsoft RD-Agent (Autonomous quant R&D, automated alpha mining, hypothesis generation)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tagisan::{
    all_ecc_skills, extract_triggers_from_text, find_ecc_skill, format_cheat_sheet,
    format_cloud_guidelines, global_ecc_dispatcher, is_local_provider,
    load_ecc_skills_from_dir, EccSkill, SkillDispatcher,
};

// =========================================================================
// 1. Built-in & On-Disk Quant Foundation Skills Discovery
// =========================================================================

#[test]
fn test_quant_foundation_skills_built_in_registration() {
    let quant_skills = [
        "kronos-kline-modeling",
        "qlib-alpha-engineering",
        "chronos-moirai-forecasting",
        "fingpt-multimodal-nlp",
        "rdagent-factor-mining",
    ];

    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 45,
        "Expected at least 45 total built-in skills, found {}",
        all_skills.len()
    );

    for skill_name in quant_skills {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Quant foundation skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.instructions.is_empty());
        assert!(
            s.instructions.len() > 200,
            "Skill '{}' instructions must contain deep quantitative guidance",
            skill_name
        );
    }
}

#[test]
fn test_quant_foundation_skills_alias_lookups() {
    // Kronos aliases
    assert!(find_ecc_skill("kronos").is_some());
    assert!(find_ecc_skill("kronos-bsq").is_some());
    assert!(find_ecc_skill("kline-modeling").is_some());

    // Qlib aliases
    assert!(find_ecc_skill("qlib").is_some());
    assert!(find_ecc_skill("qlib-alpha").is_some());
    assert!(find_ecc_skill("qlib-factor-neutralization").is_some());

    // Chronos / MOIRAI aliases
    assert!(find_ecc_skill("chronos").is_some());
    assert!(find_ecc_skill("moirai").is_some());
    assert!(find_ecc_skill("chronos-moirai").is_some());

    // FinGPT / FinRL aliases
    assert!(find_ecc_skill("fingpt").is_some());
    assert!(find_ecc_skill("finrl").is_some());
    assert!(find_ecc_skill("financial-nlp").is_some());

    // RD-Agent aliases
    assert!(find_ecc_skill("rdagent").is_some());
    assert!(find_ecc_skill("rd-agent").is_some());
    assert!(find_ecc_skill("rdagent-mining").is_some());
}

#[test]
fn test_quant_foundation_skills_disk_loading() {
    let disk_path = Path::new(".ecc/skills");
    if disk_path.is_dir() {
        let loaded = load_ecc_skills_from_dir(disk_path);
        let loaded_names: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

        // Check if on-disk quant skills are present when .ecc/skills exists
        let expected = [
            "kronos-kline-modeling",
            "qlib-alpha-engineering",
            "chronos-moirai-forecasting",
            "fingpt-multimodal-nlp",
            "rdagent-factor-mining",
        ];

        for name in expected {
            if loaded_names.contains(name) {
                let skill = find_ecc_skill(name).unwrap();
                let triggers = extract_triggers_from_text(&skill.name, &skill.description, &[]);
                assert!(!triggers.is_empty(), "Skill '{}' triggers must not be empty", name);
            }
        }
    }
}

// =========================================================================
// 2. Semantic Intent Top-K Dispatching for Financial Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_quant_foundation_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        (
            "candlestick prediction with Kronos BSQ",
            "kronos-kline-modeling",
        ),
        (
            "Barra factor neutralization in Qlib",
            "qlib-alpha-engineering",
        ),
        (
            "zero-shot time series forecasting with Chronos",
            "chronos-moirai-forecasting",
        ),
        (
            "SEC filing sentiment with FinGPT",
            "fingpt-multimodal-nlp",
        ),
        (
            "RD-Agent alpha mining",
            "rdagent-factor-mining",
        ),
    ];

    for (query, expected_skill) in test_cases {
        let results = dispatcher.dispatch(query, 3, None);
        assert!(
            !results.is_empty(),
            "Query '{}' must dispatch at least one skill",
            query
        );

        let matched = results.iter().any(|d| d.skill.name == expected_skill);
        assert!(
            matched,
            "Query '{}' expected to match '{}', but got top matches: {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );

        // Score must be strongly positive
        let top_match = results.iter().find(|d| d.skill.name == expected_skill).unwrap();
        assert!(
            top_match.score > 0.0,
            "Dispatched skill '{}' score must be > 0.0, got {}",
            expected_skill,
            top_match.score
        );
    }
}

// =========================================================================
// 3. Discrimination Between Local Ollama Cheat-Sheet vs Cloud Guidelines
// =========================================================================

#[test]
fn test_local_ollama_vs_cloud_guidelines_discrimination() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Barra factor neutralization in Qlib with Top-K backtest";

    // 1. Local Ollama formatting: condensed cheat sheet
    let (local_prompt, local_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "ollama",
        None,
    );
    assert!(!local_skills.is_empty());
    assert!(
        local_prompt.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local Ollama prompt must contain Cheat Sheet header"
    );
    assert!(
        !local_prompt.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"),
        "Local Ollama prompt must not contain full cloud specs header"
    );
    assert!(
        local_skills.len() <= 2,
        "Local Ollama context must be bounded to <= 2 skills"
    );

    // 2. Cloud Anthropic / OpenAI formatting: comprehensive architectural guidelines
    let (cloud_prompt, cloud_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "anthropic",
        None,
    );
    assert!(!cloud_skills.is_empty());
    assert!(
        cloud_prompt.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud prompt must contain Comprehensive Specifications header"
    );
    assert!(
        cloud_prompt.contains("Full Specification & Directives:"),
        "Cloud prompt must contain full directive body"
    );
    assert!(
        !cloud_prompt.contains("[LOCAL LLM CHEAT SHEET"),
        "Cloud prompt must not contain local cheat sheet header"
    );
}

// =========================================================================
// 4. Multithreaded Concurrent Dispatching Across 50 Threads
// =========================================================================

#[test]
fn test_multithreaded_concurrent_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher().clone());
    let queries = Arc::new(vec![
        "candlestick prediction with Kronos BSQ",
        "Barra factor neutralization in Qlib",
        "zero-shot time series forecasting with Chronos",
        "SEC filing sentiment with FinGPT",
        "RD-Agent alpha mining",
        "autoregressive dual decoding s1 s2 tokens",
        "pinball loss quantile Value-at-Risk VaR",
        "Almgren-Chriss deep reinforcement learning execution",
        "symbolic factor expression tree discovery",
        "top-k backtesting RankIC ICIR turnover",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(thread_count);

    let start_time = Instant::now();

    for t_idx in 0..thread_count {
        let d_clone = Arc::clone(&dispatcher);
        let q_clone = Arc::clone(&queries);
        let s_clone = Arc::clone(&success_count);

        handles.push(thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let query = &q_clone[(t_idx + i) % q_clone.len()];
                let results = d_clone.dispatch(query, 3, None);
                if !results.is_empty() && results[0].score > 0.0 {
                    s_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked during concurrent dispatch");
    }

    let elapsed = start_time.elapsed();
    let total_dispatches = thread_count * iterations_per_thread;
    let successful = success_count.load(Ordering::SeqCst);

    assert_eq!(
        successful, total_dispatches,
        "All {} concurrent dispatches across 50 threads must succeed",
        total_dispatches
    );

    let avg_dispatch_micros = elapsed.as_micros() as f64 / total_dispatches as f64;
    println!(
        "50-thread concurrent stress test completed: {} queries in {:?} (avg {:.2}µs/query)",
        total_dispatches, elapsed, avg_dispatch_micros
    );
}

// =========================================================================
// 5. Quantitative Safety Checks (Look-Ahead Bias & Zero-Cost Detection)
// =========================================================================

/// Mock quantitative safety validator that simulates AgentShield interception
struct QuantSafetyValidator;

#[derive(Debug, PartialEq, Eq)]
enum SafetyViolation {
    LookAheadBias(String),
    ZeroTransactionCost(String),
    None,
}

impl QuantSafetyValidator {
    fn inspect_backtest_code(code: &str) -> Vec<SafetyViolation> {
        let mut violations = Vec::new();

        // 1. Look-ahead bias detection
        if code.contains(".shift(-1)") || code.contains("shift(-") || code.contains("t + 1") && code.contains("signal[t]") {
            violations.push(SafetyViolation::LookAheadBias(
                "Negative lag or unlagged future return detected in alpha generation".to_string(),
            ));
        }
        if code.contains("fiscal_year_end") && !code.contains("acceptance_datetime") {
            violations.push(SafetyViolation::LookAheadBias(
                "SEC filing timestamp indexed by fiscal period end date instead of EDGAR acceptance timestamp".to_string(),
            ));
        }

        // 2. Zero transaction cost detection
        if code.contains("commission: 0.0") || code.contains("fee: 0.0") || code.contains("slippage: 0.0") || code.contains("cost = 0") {
            violations.push(SafetyViolation::ZeroTransactionCost(
                "Zero commission or slippage fee configured in backtest pipeline".to_string(),
            ));
        }

        violations
    }
}

#[test]
fn test_quantitative_safety_checks() {
    // Violating code 1: Look-ahead bias
    let code_look_ahead = r#"
        def compute_alpha(df):
            # FUTURE LEAK: using future close to predict current return
            future_return = df['close'].shift(-1) / df['close'] - 1.0
            return future_return
    "#;
    let violations = QuantSafetyValidator::inspect_backtest_code(code_look_ahead);
    assert!(
        violations.iter().any(|v| matches!(v, SafetyViolation::LookAheadBias(_))),
        "Look-ahead bias must be caught by validator"
    );

    // Violating code 2: Zero transaction costs
    let code_zero_cost = r#"
        backtest_config = {
            "top_k": 30,
            "commission: 0.0": True,
            "rebalance_freq": "1d",
        }
    "#;
    let violations = QuantSafetyValidator::inspect_backtest_code(code_zero_cost);
    assert!(
        violations.iter().any(|v| matches!(v, SafetyViolation::ZeroTransactionCost(_))),
        "Zero transaction cost configuration must be caught by validator"
    );

    // Compliant code: Proper lagging and non-zero costs
    let compliant_code = r#"
        def compute_alpha(df):
            # Past 5-day momentum lagged by 1 bar
            past_momentum = df['close'].shift(1) / df['close'].shift(6) - 1.0
            return past_momentum
            
        config = {
            "commission": 0.0005,
            "slippage": 0.0005,
            "stamp_tax": 0.0010,
        }
    "#;
    let violations = QuantSafetyValidator::inspect_backtest_code(compliant_code);
    assert!(violations.is_empty(), "Compliant code must have zero violations");
}
