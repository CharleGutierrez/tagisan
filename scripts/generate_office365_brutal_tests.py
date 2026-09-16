#!/usr/bin/env python3
"""
scripts/generate_office365_brutal_tests.py

Generates tests/office365_skills_brutal_tests.rs with exhaustive testing:
1. Discovery of all 350 Office 365 skills on disk
2. Registration of all 350 skills in built-in registry (assert >= 1345 total skills)
3. Alias resolution across all 35 clusters
4. Invariants enforcement (ALWAYS, NEVER, MANDATORY)
5. Semantic intent dispatching across clusters
6. 50-thread concurrent stress test (1000 dispatches, zero panic)
"""

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
TESTS_DIR = BASE_DIR / "tests"
TARGET_TEST = TESTS_DIR / "office365_skills_brutal_tests.rs"

sys.path.insert(0, str(BASE_DIR))
from scripts.generate_office365_skills import generate_skills

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

    # Pick representative query tests for dispatcher (1 per cluster)
    query_tests = [
        ("spill-range-hash", "o365-excel-lambda-spill-range-anchoring-hash"),
        ("range-batch-read-write", "o365-excel-scripts-range-batch-read-write"),
        ("context-sync-minimization", "o365-excel-js-context-sync-queue-minimization"),
        ("openxml-document-traversal", "o365-word-openxml-document-part-traversal"),
        ("slide-layout-insertion", "o365-ppt-slide-layout-insertion-matrix"),
        ("read-vs-compose-contexts", "o365-outlook-read-vs-compose-mode-contexts"),
        ("notebook-hierarchy-navigation", "o365-onenote-notebook-section-page-hierarchy"),
        ("teams-js-sdk-v2", "o365-teams-teams-js-sdk-v2-capabilities"),
        ("meeting-lifecycle-stages", "o365-teams-live-meeting-lifecycle-stage-routing"),
        ("viva-ace-card-views", "o365-viva-viva-connections-ace-card-views"),
        ("spfx-yeoman-manifest", "o365-spfx-core-spfx-yeoman-scaffolding-manifest"),
        ("fluent-ui-v9-spfx", "o365-spfx-react-fluent-ui-v9-react-styling"),
        ("application-customizer-placeholders", "o365-spfx-ext-application-customizer-placeholders"),
        ("pnpjs-fluent-client", "o365-sp-data-pnpjs-fluent-client-chaining"),
        ("graph-rest-conventions", "o365-graph-core-graph-rest-endpoint-conventions"),
        ("webhook-subscription-handshake", "o365-graph-events-webhook-subscription-handshake"),
        ("connection-schema-registration", "o365-graph-conn-connection-schema-registration"),
        ("unified-manifest-json", "o365-addin-core-unified-manifest-json-schema"),
        ("entra-id-app-scopes", "o365-addin-auth-entra-id-app-registration-scopes"),
        ("centralized-deployment-admin", "o365-addin-deploy-centralized-deployment-admin-center"),
        ("fluid-dds-fundamentals", "o365-fluid-fluid-distributed-data-structures"),
        ("loop-component-architecture", "o365-loop-loop-component-file-architecture"),
        ("kql-query-syntax", "o365-search-kql-query-syntax-mastery"),
        ("forms-flow-trigger", "o365-forms-forms-power-automate-trigger"),
        ("unified-tasks-graph-model", "o365-tasks-unified-tasks-graph-model"),
        ("bookings-schema-endpoints", "o365-bookings-bookings-schema-and-endpoints"),
        ("json-column-formatting", "o365-lists-json-column-formatting-ast"),
        ("exchange-powershell-v3", "o365-exchange-exchange-powershell-v3-rest"),
        ("sensitivity-label-taxonomy", "o365-purview-sensitivity-label-taxonomy"),
        ("zero-trust-principles", "o365-sec-zero-trust-architecture-principles"),
        ("ediscovery-premium-cases", "o365-compliance-ediscovery-premium-case-management"),
        ("graph-powershell-sdk-core", "o365-admin-microsoft-graph-powershell-sdk-core"),
        ("conversational-vibe-workflow", "o365-vibe-conversational-vibe-coding-workflow"),
        ("m365-cli-automation", "o365-alm-m365-cli-cross-platform-automation"),
        ("multi-tenant-data-partitioning", "o365-saas-multi-tenant-data-partitioning"),
    ]

    query_assertions = []
    for q, expected in query_tests:
        query_assertions.append(f"""        ("{q}", "{expected}"),""")
    query_table = "\n".join(query_assertions)

    rust_code = f"""//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Office 365 & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{{AtomicUsize, Ordering}};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
}};

pub const OFFICE365_SKILLS_350: [&str; 350] = [
{skills_array}
];

// =========================================================================
// 1. Discovery of all 350 Office 365 Skills on Disk
// =========================================================================

#[test]
fn test_all_350_office365_skills_discovered_on_disk() {{
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &OFFICE365_SKILLS_350 {{
        assert!(
            loaded_map.contains(*skill),
            "Office 365 skill '{{}}' must be discovered in .ecc/skills directory",
            skill
        );
    }}
}}

// =========================================================================
// 2. Built-in Registration of all 350 Office 365 Skills (assert >= 1345 total skills)
// =========================================================================

#[test]
fn test_all_350_office365_skills_built_in_registration() {{
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 1345,
        "Expected at least 1345 total built-in skills including Office 365 suite, found {{}}",
        all_skills.len()
    );

    for skill_name in &OFFICE365_SKILLS_350 {{
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Office 365 skill '{{}}' must be registered in built-in skills",
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
    for skill_name in &OFFICE365_SKILLS_350 {{
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
        "spill-range-hash",
        "range-batch-read-write",
        "context-sync-minimization",
        "openxml-document-traversal",
        "slide-layout-insertion",
        "read-vs-compose-contexts",
        "notebook-hierarchy-navigation",
        "teams-js-sdk-v2",
        "meeting-lifecycle-stages",
        "viva-ace-card-views",
        "spfx-yeoman-manifest",
        "fluent-ui-v9-spfx",
        "application-customizer-placeholders",
        "pnpjs-fluent-client",
        "graph-rest-conventions",
        "webhook-subscription-handshake",
        "connection-schema-registration",
        "unified-manifest-json",
        "entra-id-app-scopes",
        "centralized-deployment-admin",
        "fluid-dds-fundamentals",
        "loop-component-architecture",
        "kql-query-syntax",
        "forms-flow-trigger",
        "unified-tasks-graph-model",
        "bookings-schema-endpoints",
        "json-column-formatting",
        "exchange-powershell-v3",
        "sensitivity-label-taxonomy",
        "zero-trust-principles",
        "ediscovery-premium-cases",
        "graph-powershell-sdk-core",
        "conversational-vibe-workflow",
        "m365-cli-automation",
        "multi-tenant-data-partitioning",
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
