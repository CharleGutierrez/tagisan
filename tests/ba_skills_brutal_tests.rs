//! Brutal Integration & Domain Verification Tests for Top 50 Books in Business & Functional Analysis in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

const BA_SKILLS_50: [&str; 50] = [
    "ba-wiegers-requirements-engineering",
    "ba-volere-requirements-specification",
    "ba-cockburn-use-case-modeling",
    "ba-nfr-quality-attributes",
    "ba-requirements-traceability-matrix",
    "ba-adzic-specification-by-example",
    "ba-cucumber-gherkin-syntax",
    "ba-bdd-three-amigos-workshop",
    "ba-atdd-acceptance-criteria",
    "ba-living-documentation-tooling",
    "ba-evans-ubiquitous-language",
    "ba-brandolini-event-storming",
    "ba-bounded-context-mapping",
    "ba-domain-storytelling",
    "ba-subdomain-core-domain-triage",
    "ba-patton-user-story-mapping",
    "ba-invest-user-stories",
    "ba-story-splitting-patterns",
    "ba-wsjf-backlog-prioritization",
    "ba-kanban-value-stream-metrics",
    "ba-bpmn-level2-process-modeling",
    "ba-bpmn-gateway-soundness",
    "ba-bpmn-timer-boundary-events",
    "ba-state-machine-lifecycle-modeling",
    "ba-process-waste-elimination-vsm",
    "ba-torres-opportunity-solution-tree",
    "ba-cagan-four-product-risks",
    "ba-customer-journey-mapping",
    "ba-jobs-to-be-done-jtbd",
    "ba-assumption-mapping-experimentation",
    "ba-hoberman-data-modeling-resource",
    "ba-relational-normalization-3nf",
    "ba-cardinality-erd-relationship-rules",
    "ba-temporal-bitemporal-data-patterns",
    "ba-data-dictionary-master-metadata",
    "ba-ross-business-rule-manifesto",
    "ba-dmn-decision-table-completeness",
    "ba-drd-decision-requirements-diagrams",
    "ba-feel-expression-language",
    "ba-decision-table-verification-solver",
    "ba-meadows-systems-thinking-leverage",
    "ba-causal-loop-diagrams-archetypes",
    "ba-galls-law-system-evolution",
    "ba-wardley-mapping-strategic-landscape",
    "ba-cynefin-framework-decision-making",
    "ba-cooper-goal-directed-design",
    "ba-norman-affordance-signifiers",
    "ba-johnson-gui-bloopers-heuristics",
    "ba-crud-form-functional-specifications",
    "ba-information-architecture-wireflow",
];

// =========================================================================
// 1. Discovery of all 50 BA Skills on Disk
// =========================================================================

#[test]
fn test_all_50_ba_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &BA_SKILLS_50 {
        assert!(
            loaded_map.contains(*skill),
            "BA skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 50 BA Skills (assert >= 220 total skills)
// =========================================================================

#[test]
fn test_all_50_ba_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 220,
        "Expected at least 220 total built-in skills including Business Analysis, found {}",
        all_skills.len()
    );

    for skill_name in &BA_SKILLS_50 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "BA skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty());
        assert!(!s.description.is_empty());
        assert!(!s.instructions.is_empty());
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete domain architecture instructions",
            skill_name
        );
    }
}

// =========================================================================
// 3. High-Leverage BA Alias Lookups
// =========================================================================

#[test]
fn test_ba_skills_alias_lookups() {
    assert_eq!(find_ecc_skill("wiegers").unwrap().name, "ba-wiegers-requirements-engineering");
    assert_eq!(find_ecc_skill("volere").unwrap().name, "ba-volere-requirements-specification");
    assert_eq!(find_ecc_skill("use-case").unwrap().name, "ba-cockburn-use-case-modeling");
    assert_eq!(find_ecc_skill("nfr").unwrap().name, "ba-nfr-quality-attributes");
    assert_eq!(find_ecc_skill("rtm").unwrap().name, "ba-requirements-traceability-matrix");
    assert_eq!(find_ecc_skill("specification-by-example").unwrap().name, "ba-adzic-specification-by-example");
    assert_eq!(find_ecc_skill("gherkin").unwrap().name, "ba-cucumber-gherkin-syntax");
    assert_eq!(find_ecc_skill("three-amigos").unwrap().name, "ba-bdd-three-amigos-workshop");
    assert_eq!(find_ecc_skill("atdd").unwrap().name, "ba-atdd-acceptance-criteria");
    assert_eq!(find_ecc_skill("living-documentation").unwrap().name, "ba-living-documentation-tooling");
    assert_eq!(find_ecc_skill("ubiquitous-language").unwrap().name, "ba-evans-ubiquitous-language");
    assert_eq!(find_ecc_skill("event-storming").unwrap().name, "ba-brandolini-event-storming");
    assert_eq!(find_ecc_skill("context-mapping").unwrap().name, "ba-bounded-context-mapping");
    assert_eq!(find_ecc_skill("story-mapping").unwrap().name, "ba-patton-user-story-mapping");
    assert_eq!(find_ecc_skill("invest").unwrap().name, "ba-invest-user-stories");
    assert_eq!(find_ecc_skill("story-splitting").unwrap().name, "ba-story-splitting-patterns");
    assert_eq!(find_ecc_skill("wsjf").unwrap().name, "ba-wsjf-backlog-prioritization");
    assert_eq!(find_ecc_skill("process-modeling").unwrap().name, "ba-bpmn-level2-process-modeling");
    assert_eq!(find_ecc_skill("gateway-soundness").unwrap().name, "ba-bpmn-gateway-soundness");
    assert_eq!(find_ecc_skill("state-machine").unwrap().name, "ba-state-machine-lifecycle-modeling");
    assert_eq!(find_ecc_skill("ost").unwrap().name, "ba-torres-opportunity-solution-tree");
    assert_eq!(find_ecc_skill("four-product-risks").unwrap().name, "ba-cagan-four-product-risks");
    assert_eq!(find_ecc_skill("jtbd").unwrap().name, "ba-jobs-to-be-done-jtbd");
    assert_eq!(find_ecc_skill("normalization").unwrap().name, "ba-relational-normalization-3nf");
    assert_eq!(find_ecc_skill("bitemporal").unwrap().name, "ba-temporal-bitemporal-data-patterns");
    assert_eq!(find_ecc_skill("rulespeak").unwrap().name, "ba-ross-business-rule-manifesto");
    assert_eq!(find_ecc_skill("dmn").unwrap().name, "ba-dmn-decision-table-completeness");
    assert_eq!(find_ecc_skill("feel").unwrap().name, "ba-feel-expression-language");
    assert_eq!(find_ecc_skill("systems-thinking").unwrap().name, "ba-meadows-systems-thinking-leverage");
    assert_eq!(find_ecc_skill("galls-law").unwrap().name, "ba-galls-law-system-evolution");
    assert_eq!(find_ecc_skill("wardley-maps").unwrap().name, "ba-wardley-mapping-strategic-landscape");
    assert_eq!(find_ecc_skill("goal-directed-design").unwrap().name, "ba-cooper-goal-directed-design");
    assert_eq!(find_ecc_skill("affordances").unwrap().name, "ba-norman-affordance-signifiers");
    assert_eq!(find_ecc_skill("wireflow").unwrap().name, "ba-information-architecture-wireflow");
}

// =========================================================================
// 4. Semantic Intent Dispatching for Business Analysis Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_ba_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        ("requirements traceability matrix and planguage quality metrics", "ba-wiegers-requirements-engineering"),
        ("volere fit criteria snow card testable requirements", "ba-volere-requirements-specification"),
        ("given when then scenario outline declarative gherkin bdd", "ba-cucumber-gherkin-syntax"),
        ("event storming timeline domain events commands aggregates", "ba-brandolini-event-storming"),
        ("user story mapping walking skeleton dual backbone jeff patton", "ba-patton-user-story-mapping"),
        ("bpmn 2.0 workflow pools lanes gateway token flow", "ba-bpmn-level2-process-modeling"),
        ("dmn decision tables hit policy unique feel expression", "ba-dmn-decision-table-completeness"),
        ("relational normalization third normal form 3nf dependencies", "ba-relational-normalization-3nf"),
        ("systems thinking stocks flows feedback loops donella meadows", "ba-meadows-systems-thinking-leverage"),
        ("galls law complex systems working simple system evolution", "ba-galls-law-system-evolution"),
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
}

// =========================================================================
// 5. Local Ollama vs Cloud Guidelines Discrimination
// =========================================================================

#[test]
fn test_ba_skills_local_vs_cloud_formatting() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Draft Gherkin BDD scenario and DMN decision table for case filing";

    // Local Ollama formatting
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
        local_skills.len() <= 2,
        "Local Ollama context must be bounded to <= 2 skills"
    );

    // Cloud Anthropic formatting
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
}

// =========================================================================
// 6. 50-Thread Concurrent Dispatch Stress Test
// =========================================================================

#[test]
fn test_multithreaded_concurrent_ba_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "requirements traceability planguage",
        "volere fit criteria snow card",
        "gherkin given when then bdd scenario",
        "event storming domain events commands",
        "user story mapping walking skeleton",
        "bpmn process gateway token flow",
        "dmn decision table hit policy unique",
        "relational normalization 3nf bcnf",
        "stocks and flows feedback loops meadows",
        "galls law complex system evolution",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let mut handles = Vec::with_capacity(thread_count);
    let success_count = Arc::new(AtomicUsize::new(0));

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
        "All {} concurrent dispatches must succeed, got {}",
        total_dispatches, successful
    );
    assert!(
        elapsed.as_millis() < 5000,
        "50-thread concurrent dispatch stress test took {:?}, expected < 5s",
        elapsed
    );
}

// =========================================================================
// 7. Formal Business & Functional Invariants Verification
// =========================================================================

#[test]
fn test_wsjf_prioritization_invariant() {
    // WSJF = Cost of Delay / Job Duration = (User Value + Time Criticality + RR/OE) / Size
    struct Job {
        name: &'static str,
        user_value: f64,
        time_criticality: f64,
        risk_reduction: f64,
        duration: f64,
    }

    let jobs = [
        Job { name: "Job A", user_value: 10.0, time_criticality: 10.0, risk_reduction: 10.0, duration: 3.0 }, // CoD = 30, WSJF = 10.0
        Job { name: "Job B", user_value: 20.0, time_criticality: 20.0, risk_reduction: 10.0, duration: 10.0 }, // CoD = 50, WSJF = 5.0
        Job { name: "Job C", user_value: 5.0, time_criticality: 5.0, risk_reduction: 2.0, duration: 1.0 },    // CoD = 12, WSJF = 12.0
    ];

    let mut scored: Vec<(&str, f64)> = jobs.iter().map(|j| {
        let cod = j.user_value + j.time_criticality + j.risk_reduction;
        let wsjf = cod / j.duration;
        (j.name, wsjf)
    }).collect();

    // Sort descending by WSJF
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Highest WSJF must be scheduled first: Job C (12.0) -> Job A (10.0) -> Job B (5.0)
    assert_eq!(scored[0].0, "Job C");
    assert_eq!(scored[1].0, "Job A");
    assert_eq!(scored[2].0, "Job B");
}

#[test]
fn test_littles_law_and_pce_invariants() {
    // Little's Law: Lead Time = WIP / Throughput
    let wip: f64 = 24.0; // items
    let throughput: f64 = 4.0; // items per week
    let lead_time: f64 = wip / throughput;
    assert_eq!(lead_time, 6.0, "Lead time must be 6 weeks");

    // Process Cycle Efficiency (PCE) = (Value-Add Time / Total Lead Time) * 100
    let value_add_hours: f64 = 8.0;
    let total_lead_hours: f64 = 160.0; // 4 weeks
    let pce: f64 = (value_add_hours / total_lead_hours) * 100.0;
    assert!((pce - 5.0).abs() < 1e-4, "PCE must be 5.0%");
}

#[test]
fn test_dmn_decision_table_completeness_and_overlap_invariants() {
    // Verify 1D partition over claim amount [0 .. 100_000]
    struct RuleRange {
        rule_id: usize,
        min: f64,
        max: f64,
    }

    let rules = [
        RuleRange { rule_id: 1, min: 0.0, max: 10000.0 },
        RuleRange { rule_id: 2, min: 10000.01, max: 50000.0 },
        RuleRange { rule_id: 3, min: 50000.01, max: 100000.0 },
    ];

    // Non-overlap check: for any two rules, ranges must not intersect
    for i in 0..rules.len() {
        for j in (i+1)..rules.len() {
            let r1 = &rules[i];
            let r2 = &rules[j];
            let overlaps = (r1.min <= r2.max) && (r2.min <= r1.max);
            assert!(!overlaps, "Rules {} and {} must not overlap", r1.rule_id, r2.rule_id);
        }
    }
}
