#!/usr/bin/env python3
"""
scripts/generate_power_platform_brutal_tests.py

Generates tests/power_platform_skills_brutal_tests.rs with exhaustive testing:
1. Discovery of all 350 skills on disk
2. Registration of all 350 skills in built-in registry (assert >= 995 total skills)
3. Alias resolution across all 35 clusters
4. Invariants enforcement (ALWAYS, NEVER, MANDATORY)
5. Semantic intent dispatching
6. 50-thread concurrent stress test
"""

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
TESTS_DIR = BASE_DIR / "tests"
TARGET_TEST = TESTS_DIR / "power_platform_skills_brutal_tests.rs"

sys.path.insert(0, str(BASE_DIR))
from scripts.generate_power_platform_skills import generate_skills

def main():
    skills = generate_skills()
    assert len(skills) == 350

    # Format 350 skill IDs
    skill_list_lines = []
    current_cluster = None
    for s in skills:
        if s["cluster"] != current_cluster:
            current_cluster = s["cluster"]
            skill_list_lines.append(f"    // Cluster {current_cluster} (Skill {s['id']})")
        skill_list_lines.append(f'    "{s["id"]}",')

    skills_array = "\n".join(skill_list_lines)

    # Pick representative alias tests (at least 1 per cluster)
    alias_tests = []
    for c in range(1, 36):
        c_skills = [s for s in skills if s["cluster"] == c]
        if c_skills:
            rep = c_skills[0]
            if rep["triggers"]:
                alias_tests.append((rep["triggers"][0], rep["id"]))

    alias_assertions = []
    for alias, canonical in alias_tests:
        alias_assertions.append(f"""        ("{alias}", "{canonical}"),""")
    alias_table = "\n".join(alias_assertions)

    # Pick representative query tests for dispatcher
    query_tests = [
        ("fluid-container-layouts", "pp-canvas-fluid-container-layouts"),
        ("power-fx-imperative-vs-declarative", "pp-fx-imperative-vs-declarative"),
        ("component-libraries", "pp-comp-component-libraries-versioning"),
        ("loaddata-savedata", "pp-offline-loaddata-savedata-local"),
        ("delegation-limits", "pp-perf-delegation-limits-workarounds"),
        ("standard-custom-tables", "pp-dv-standard-custom-activity-tables"),
        ("modern-app-designer", "pp-mda-modern-app-designer-sitemaps"),
        ("privilege-depth-matrix", "pp-sec-privilege-depth-matrix"),
        ("flow-trigger-types", "pp-flow-trigger-types-matrix"),
        ("try-catch-finally-scopes", "pp-err-try-catch-finally-scope-patterns"),
        ("approval-types-matrix", "pp-appr-approval-types-matrix"),
        ("odata-filter-query", "pp-odata-filter-query-syntax"),
        ("attended-vs-unattended-rpa", "pp-rpa-attended-vs-unattended-execution"),
        ("star-schema-modeling", "pp-pbi-star-schema-dimensional-modeling"),
        ("row-context-vs-filter-context", "pp-dax-row-context-vs-filter-context"),
        ("query-folding-diagnostics", "pp-m-query-folding-diagnostics"),
        ("linguistic-schema-synonyms", "pp-pbicop-linguistic-schema-synonyms"),
        ("site-architecture-web-templates", "pp-pages-site-architecture-web-templates"),
        ("portal-web-api-client", "pp-pweb-dataverse-web-api-portal-client"),
        ("bootstrap-5-customization", "pp-pstyle-bootstrap-5-customization"),
        ("intent-recognition-triggers", "pp-csgen-intent-recognition-trigger-phrases"),
        ("dynamic-chaining-actions", "pp-cschain-dynamic-chaining-generative-actions"),
        ("dataverse-search-grounding", "pp-csknow-dataverse-search-knowledge-indexing"),
        ("hub-and-spoke-multi-agent", "pp-csorch-hub-and-spoke-agent-architecture"),
        ("control-manifest-schema", "pp-pcf-control-manifest-input-output-schema"),
        ("fluent-ui-v9-pcf", "pp-pcfrx-fluent-ui-v9-react-components"),
        ("barcode-scanner-camera-pcf", "pp-pcfhard-barcode-scanner-camera-capture"),
        ("openapi-swagger-schemas", "pp-conn-openapi-swagger-v2-v3-schemas"),
        ("oauth2-code-grant-flow", "pp-oauth-oauth2-code-grant-flow"),
        ("export-to-power-platform", "pp-apim-export-to-power-platform-native"),
        ("prebuilt-invoice-receipt", "pp-aib-prebuilt-invoice-receipt-processor"),
        ("prompt-engineering-studio", "pp-aiprompt-prompt-engineering-studio"),
        ("conversational-prototyping-canvas", "pp-vibe-conversational-prototyping-canvas"),
        ("pac-cli-core-commands", "pp-alm-pac-cli-core-commands-auth"),
        ("dlp-policy-tiering", "pp-gov-dlp-policy-tiering-business-nonbusiness"),
    ]

    query_assertions = []
    for q, expected in query_tests:
        query_assertions.append(f"""        ("{q}", "{expected}"),""")
    query_table = "\n".join(query_assertions)

    rust_code = f"""//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Power Platform & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{{AtomicUsize, Ordering}};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
}};

pub const POWER_PLATFORM_SKILLS_350: [&str; 350] = [
{skills_array}
];

// =========================================================================
// 1. Discovery of all 350 Power Platform Skills on Disk
// =========================================================================

#[test]
fn test_all_350_power_platform_skills_discovered_on_disk() {{
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &POWER_PLATFORM_SKILLS_350 {{
        assert!(
            loaded_map.contains(*skill),
            "Power Platform skill '{{}}' must be discovered in .ecc/skills directory",
            skill
        );
    }}
}}

// =========================================================================
// 2. Built-in Registration of all 350 Power Platform Skills (assert >= 995 total skills)
// =========================================================================

#[test]
fn test_all_350_power_platform_skills_built_in_registration() {{
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 995,
        "Expected at least 995 total built-in skills including Power Platform suite, found {{}}",
        all_skills.len()
    );

    for skill_name in &POWER_PLATFORM_SKILLS_350 {{
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Power Platform skill '{{}}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty(), "Skill name cannot be empty for '{{}}'", skill_name);
        assert!(!s.description.is_empty(), "Description cannot be empty for '{{}}'", skill_name);
        assert!(!s.instructions.is_empty(), "Instructions cannot be empty for '{{}}'", skill_name);
        assert!(
            s.instructions.len() > 100,
            "Skill '{{}}' must contain concrete engineering instructions, got length {{}}",
            skill_name,
            s.instructions.len()
        );
        assert!(
            s.instructions.contains("## 1. Core Mathematical"),
            "Skill '{{}}' must contain Core Mathematical & Architectural Foundations",
            skill_name
        );
        assert!(
            s.instructions.contains("## 2. Concrete Agent Specification"),
            "Skill '{{}}' must contain Concrete Agent Specification & Prompt Contract",
            skill_name
        );
        assert!(
            s.instructions.contains("## 3. Anti-Patterns"),
            "Skill '{{}}' must contain Anti-Patterns & Hallucination Mitigations",
            skill_name
        );
    }}
}}

// =========================================================================
// 3. High-Leverage Alias Resolution Across All 35 Clusters
// =========================================================================

#[test]
fn test_high_leverage_alias_lookups() {{
    let alias_cases = [
{alias_table}
    ];

    for (alias, canonical) in &alias_cases {{
        let res = find_ecc_skill(alias);
        assert!(
            res.is_some(),
            "Alias '{{}}' must resolve to canonical skill '{{}}'",
            alias,
            canonical
        );
        assert_eq!(
            res.unwrap().name,
            *canonical,
            "Alias '{{}}' resolved to unexpected skill name",
            alias
        );
    }}
}}

// =========================================================================
// 4. Structural Completeness & Operational Invariants
// =========================================================================

#[test]
fn test_structural_completeness_and_operational_invariants() {{
    for skill_name in &POWER_PLATFORM_SKILLS_350 {{
        let skill = find_ecc_skill(skill_name).expect("Skill must be found");

        assert!(
            skill.instructions.contains("ALWAYS"),
            "Skill '{{}}' must enforce explicit ALWAYS directive",
            skill_name
        );
        assert!(
            skill.instructions.contains("NEVER"),
            "Skill '{{}}' must enforce explicit NEVER directive",
            skill_name
        );
        assert!(
            skill.instructions.contains("MANDATORY"),
            "Skill '{{}}' must enforce explicit MANDATORY directive",
            skill_name
        );
    }}
}}

// =========================================================================
// 5. Semantic Intent Dispatching & Routing Across Clusters
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_and_routing() {{
    let dispatcher = global_ecc_dispatcher();

    let query_cases = [
{query_table}
    ];

    for (query, expected_skill) in &query_cases {{
        let results = dispatcher.dispatch(query, 5, None);
        assert!(
            !results.is_empty(),
            "Dispatcher must return matches for query '{{}}'",
            query
        );

        let found = results.iter().any(|d| d.skill.name == *expected_skill);
        assert!(
            found,
            "Dispatcher failed to match query '{{}}' with expected skill '{{}}'. Got top matches: {{:?}}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );
    }}
}}

// =========================================================================
// 6. Concurrent Dispatching Stress Test (50 Threads, Zero Panic)
// =========================================================================

#[test]
fn test_concurrent_dispatching_stress_50_threads() {{
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "fluid-container-layouts",
        "power-fx-imperative-vs-declarative",
        "component-libraries",
        "loaddata-savedata",
        "delegation-limits",
        "standard-custom-tables",
        "modern-app-designer",
        "privilege-depth-matrix",
        "flow-trigger-types",
        "try-catch-finally-scopes",
        "approval-types-matrix",
        "odata-filter-query",
        "attended-vs-unattended-rpa",
        "star-schema-modeling",
        "row-context-vs-filter-context",
        "query-folding-diagnostics",
        "linguistic-schema-synonyms",
        "site-architecture-web-templates",
        "portal-web-api-client",
        "bootstrap-5-customization",
        "intent-recognition-triggers",
        "dynamic-chaining-actions",
        "dataverse-search-grounding",
        "hub-and-spoke-multi-agent",
        "control-manifest-schema",
        "fluent-ui-v9-pcf",
        "barcode-scanner-camera-pcf",
        "openapi-swagger-schemas",
        "oauth2-code-grant-flow",
        "export-to-power-platform",
        "prebuilt-invoice-receipt",
        "prompt-engineering-studio",
        "conversational-prototyping-canvas",
        "pac-cli-core-commands",
        "dlp-policy-tiering",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let completed_queries = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(thread_count);

    let start = Instant::now();

    for t_idx in 0..thread_count {{
        let d = Arc::clone(&dispatcher);
        let q = Arc::clone(&queries);
        let c = Arc::clone(&completed_queries);

        let handle = thread::spawn(move || {{
            for i in 0..iterations_per_thread {{
                let query = q[(t_idx + i) % q.len()];
                let res = d.dispatch(query, 3, None);
                if !res.is_empty() && res[0].score > 0.0 {{
                    c.fetch_add(1, Ordering::Relaxed);
                }}
            }}
        }});
        handles.push(handle);
    }}

    for handle in handles {{
        handle.join().expect("Worker thread panicked during dispatch");
    }}

    let elapsed = start.elapsed();
    let total = thread_count * iterations_per_thread;
    let completed = completed_queries.load(Ordering::SeqCst);
    assert_eq!(completed, total, "Expected all 1000 concurrent dispatches to succeed");
    println!(
        "Concurrent stress test: {{}} dispatches across 50 threads completed in {{:?}} ({{:.2}} us/dispatch)",
        total,
        elapsed,
        (elapsed.as_micros() as f64) / (total as f64)
    );
}}
"""
    TARGET_TEST.write_text(rust_code, encoding="utf-8")
    print(f"Successfully generated {TARGET_TEST} with {len(skills)} skills and 6 brutal tests!")

if __name__ == "__main__":
    main()
