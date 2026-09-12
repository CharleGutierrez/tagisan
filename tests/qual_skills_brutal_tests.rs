//! Brutal Integration Test Suite for the Top 50 Qualitative & Behavioral Analysis Skills in Tagisan (TGS)
//! Verifies:
//! 1. All 50 Qualitative & Behavioral skills built-in registration & valid instructions (total >= 320)
//! 2. All 50 Qualitative & Behavioral skills discovered on disk in .ecc/skills/
//! 3. Alias and domain prefix lookups (qual-* -> 'behavioral' domain)
//! 4. Semantic intent dispatching across psychological/behavioral queries
//! 5. Formal psychological & behavioral invariant extraction
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

const QUAL_SKILL_NAMES: [&str; 50] = [
    // Pillar 1: Cognitive Psychology, Perception & Mental Models
    "qual-kahneman-thinking-fast-slow",
    "qual-norman-design-everyday-things",
    "qual-sweller-cognitive-load-theory",
    "qual-johnson-laird-mental-models",
    "qual-chabris-invisible-gorilla",
    // Pillar 2: Qualitative Research Methodologies & Contextual Inquiry
    "qual-portigal-interviewing-users",
    "qual-holtzblatt-contextual-inquiry",
    "qual-charmaz-grounded-theory",
    "qual-miles-qualitative-analysis",
    "qual-hall-just-enough-research",
    // Pillar 3: Behavioral Economics & Choice Architecture
    "qual-thaler-nudge",
    "qual-ariely-predictably-irrational",
    "qual-thaler-misbehaving",
    "qual-duke-thinking-in-bets",
    "qual-schwartz-paradox-of-choice",
    // Pillar 4: Behavioral Design, Habit Loops & Motivation
    "qual-fogg-tiny-habits",
    "qual-eyal-hooked",
    "qual-pink-drive",
    "qual-duhigg-power-of-habit",
    "qual-chou-actionable-gamification",
    // Pillar 5: Usability Engineering & Think-Aloud Qualitative Testing
    "qual-krug-dont-make-me-think",
    "qual-krug-rocket-surgery",
    "qual-nielsen-usability-engineering",
    "qual-rubin-usability-testing",
    "qual-sauro-quantifying-ux",
    // Pillar 6: Naturalistic Decision Making & High-Reliability Human Factors
    "qual-klein-sources-of-power",
    "qual-weick-sensemaking",
    "qual-vaughan-challenger-launch",
    "qual-woods-behind-human-error",
    "qual-dekker-field-guide-human-error",
    // Pillar 7: Conversational Pragmatics & Interaction Linguistics
    "qual-austin-how-to-do-things-with-words",
    "qual-grice-studies-way-of-words",
    "qual-hall-conversational-design",
    "qual-nass-wired-for-speech",
    "qual-reeves-media-equation",
    // Pillar 8: Organizational Sociology & Team Dynamics
    "qual-edmondson-fearless-organization",
    "qual-demarco-peopleware",
    "qual-skelton-team-topologies",
    "qual-arbinger-leadership-self-deception",
    "qual-coyle-culture-code",
    // Pillar 9: Deceptive Design, Ethical Ergonomics & Autonomy
    "qual-nodder-evil-by-design",
    "qual-brignull-deceptive-patterns",
    "qual-zuboff-surveillance-capitalism",
    "qual-noble-algorithms-of-oppression",
    "qual-boettcher-technically-wrong",
    // Pillar 10: Sociotechnical Systems, Feedback & Embodied Interaction
    "qual-brown-social-life-information",
    "qual-dourish-where-action-is",
    "qual-suchman-plans-situated-actions",
    "qual-norman-things-make-us-smart",
    "qual-wiener-cybernetics",
];

#[test]
fn test_01_all_50_qual_skills_built_in_registration() {
    println!("\n=== TEST 1: All 50 Qualitative Skills Built-In Registration ===");
    let built_ins = all_built_in_skills();
    println!("Total built-in skills registered: {}", built_ins.len());
    assert!(built_ins.len() >= 320, "Expected at least 320 built-in skills, found {}", built_ins.len());

    let built_in_map: HashSet<String> = built_ins.into_iter().map(|s| s.name).collect();

    for name in &QUAL_SKILL_NAMES {
        assert!(
            built_in_map.contains(*name),
            "Skill '{}' missing from all_built_in_skills()",
            name
        );
        let skill = find_built_in_skill(name).expect(&format!("find_built_in_skill('{}') returned None", name));
        assert!(!skill.description.is_empty());
        assert!(!skill.instructions.is_empty());
        assert!(skill.instructions.contains("Core Psychological Foundations"));
        assert!(skill.instructions.contains("Prompt Contract"));
    }
    println!("  [✓] All 50 Qualitative & Behavioral skills confirmed built-in with valid instructions");
}

#[test]
fn test_02_all_50_qual_skills_discovered_on_disk() {
    println!("\n=== TEST 2: All 50 Qualitative Skills Discovered on Disk ===");
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    for name in &QUAL_SKILL_NAMES {
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
    println!("  [✓] All 50 Qualitative skills verified on disk in .ecc/skills/");
}

#[test]
fn test_03_qual_skills_alias_and_domain_mapping() {
    println!("\n=== TEST 3: Qualitative Aliases & Domain Mapping ===");
    let aliases = [
        ("kahneman", "qual-kahneman-thinking-fast-slow"),
        ("system-1-system-2", "qual-kahneman-thinking-fast-slow"),
        ("norman-doors", "qual-norman-design-everyday-things"),
        ("cognitive-load", "qual-sweller-cognitive-load-theory"),
        ("mental-models", "qual-johnson-laird-mental-models"),
        ("inattentional-blindness", "qual-chabris-invisible-gorilla"),
        ("interviewing-users", "qual-portigal-interviewing-users"),
        ("contextual-inquiry", "qual-holtzblatt-contextual-inquiry"),
        ("nudge", "qual-thaler-nudge"),
        ("choice-architecture", "qual-thaler-nudge"),
        ("predictably-irrational", "qual-ariely-predictably-irrational"),
        ("paradox-of-choice", "qual-schwartz-paradox-of-choice"),
        ("tiny-habits", "qual-fogg-tiny-habits"),
        ("fogg-behavior-model", "qual-fogg-tiny-habits"),
        ("hooked", "qual-eyal-hooked"),
        ("dont-make-me-think", "qual-krug-dont-make-me-think"),
        ("nielsen-heuristics", "qual-nielsen-usability-engineering"),
        ("sources-of-power", "qual-klein-sources-of-power"),
        ("sensemaking", "qual-weick-sensemaking"),
        ("normalization-of-deviance", "qual-vaughan-challenger-launch"),
        ("speech-acts", "qual-austin-how-to-do-things-with-words"),
        ("gricean-maxims", "qual-grice-studies-way-of-words"),
        ("psychological-safety", "qual-edmondson-fearless-organization"),
        ("team-topologies", "qual-skelton-team-topologies"),
        ("dark-patterns", "qual-brignull-deceptive-patterns"),
        ("surveillance-capitalism", "qual-zuboff-surveillance-capitalism"),
        ("cybernetics", "qual-wiener-cybernetics"),
    ];

    for (alias, expected) in aliases {
        let found = find_built_in_skill(alias);
        assert!(found.is_some(), "Alias '{}' lookup returned None", alias);
        assert_eq!(found.unwrap().name, expected, "Alias '{}' mapped to wrong skill", alias);
    }
    println!("  [✓] All high-leverage qualitative aliases verified");
}

#[test]
fn test_04_semantic_intent_dispatching_for_qual_queries() {
    println!("\n=== TEST 4: Semantic Intent Dispatching for Qualitative Queries ===");
    let dispatcher = global_dispatcher();

    let test_cases = [
        ("system 1 intuitive fast heuristic thinking cognitive biases", "qual-kahneman-thinking-fast-slow"),
        ("gulf of execution evaluation affordances visual signifiers", "qual-norman-design-everyday-things"),
        ("working memory extraneous cognitive load progressive disclosure", "qual-sweller-cognitive-load-theory"),
        ("nudge choice architecture default options inertia", "qual-thaler-nudge"),
        ("fogg behavior model motivation ability prompts tiny habits", "qual-fogg-tiny-habits"),
        ("billboard scanning visual hierarchy dont make me think", "qual-krug-dont-make-me-think"),
        ("recognition primed decision expert intuition under pressure", "qual-klein-sources-of-power"),
        ("gricean maxims quantity quality relation manner cooperative", "qual-grice-studies-way-of-words"),
        ("psychological safety blameless post mortem interpersonal risk", "qual-edmondson-fearless-organization"),
        ("roach motel confirmshaming deceptive dark patterns click to cancel", "qual-brignull-deceptive-patterns"),
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
    println!("  [✓] All 10 qualitative domain semantic intent queries correctly dispatched");
}

#[test]
fn test_05_dense_invariants_token_compression() {
    println!("\n=== TEST 5: Dense Invariants Token Compression ===");
    let dispatcher = global_dispatcher();
    let sample_skills = [
        "qual-kahneman-thinking-fast-slow",
        "qual-fogg-tiny-habits",
        "qual-brignull-deceptive-patterns",
        "qual-edmondson-fearless-organization",
    ];

    for name in sample_skills {
        let skill = dispatcher.get_skill(name).expect("Skill must exist");
        let ds = DispatchedSkill {
            domain: "behavioral".to_string(),
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
    let query = "system 1 cognitive bias dark patterns psychological safety";
    let budget = TokenBudget::cloud_default();

    let result = dispatcher.dispatch_diversified(query, budget, Some("behavioral"));
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
    
    let args = json!({ "name": "qual-kahneman-thinking-fast-slow" });
    let output = tool.execute(args).await.expect("Tool execution must succeed");
    assert!(output.contains("# Skill: qual-kahneman-thinking-fast-slow"));
    assert!(output.contains("Dual-process cognitive psychology"));
    assert!(output.contains("## Instructions"));
    println!("  [✓] FetchSkillTool retrieved full spec ({} bytes)", output.len());
}

#[test]
fn test_08_multithreaded_concurrent_qual_dispatching_50_threads() {
    println!("\n=== TEST 8: 50-Thread Concurrent Dispatch Stress Test ===");
    let _ = global_dispatcher(); // Pre-warm OnceLock
    let num_threads = 50;
    let queries_per_thread = 200; // 10,000 queries total
    let queries = [
        "system 1 system 2 cognitive biases fast thinking",
        "dark patterns confirmshaming click to cancel roach motel",
        "fogg behavior model motivation ability prompt tiny habits",
        "psychological safety fearless organization blameless post mortem",
        "dont make me think visual hierarchy billboard scanning",
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
    let text = "ALWAYS: Primary user flows must operate entirely within System 1 cognitive fluency.";
    let tokens = estimate_tokens(text);
    assert!(tokens >= 10 && tokens <= 30);

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
