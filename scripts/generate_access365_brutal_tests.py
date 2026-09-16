#!/usr/bin/env python3
"""
scripts/generate_access365_brutal_tests.py

Generates tests/access365_skills_brutal_tests.rs with exhaustive testing:
1. Discovery of all 350 Access 365 skills on disk
2. Registration of all 350 skills in built-in registry (assert >= 1695 total skills)
3. Alias resolution across all 35 clusters
4. Invariants enforcement (ALWAYS, NEVER, MANDATORY)
5. Semantic intent dispatching across clusters
6. 50-thread concurrent stress test (1000 dispatches, zero panic)
"""

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
TESTS_DIR = BASE_DIR / "tests"
TARGET_TEST = TESTS_DIR / "access365_skills_brutal_tests.rs"

sys.path.insert(0, str(BASE_DIR))
from scripts.generate_access365_skills import get_all_skills

def main():
    skills = get_all_skills()
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

    # Pick representative alias tests (1 per cluster)
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
        ("relational-3nf", "acc-schema-relational-theory-3nf"),
        ("storage-pages", "acc-ace-4kb-storage-page-geometry"),
        ("jet-ddl-create", "acc-sql-ddl-create-table-constraints"),
        ("correlated-subqueries", "acc-sql-adv-correlated-subqueries"),
        ("foreign-keys", "acc-rel-foreign-key-referential-integrity"),
        ("option-explicit", "acc-vba-core-modular-code-organization"),
        ("structured-error-handling", "acc-vba-err-structured-error-handlers"),
        ("property-get-let", "acc-vba-oop-property-get-let-set"),
        ("ptrsafe-64bit", "acc-win32-ptrsafe-64bit-declarations"),
        ("vbscript-regexp", "acc-str-json-regex-pattern-matching"),
        ("dao-cursors", "acc-dao-cursor-selection-dynaset-snapshot"),
        ("disconnected-recordsets", "acc-ado-disconnected-recordsets"),
        ("dsn-less-connections", "acc-odbc-dsn-less-connection-strings"),
        ("ace-oledb-provider", "acc-oledb-ace-provider-architecture"),
        ("no-locks-property", "acc-concurr-no-locks-vs-edited-record"),
        ("form-event-sequence", "acc-form-life-event-sequence-architecture"),
        ("linkmasterfields", "acc-subform-master-child-link-fields"),
        ("report-sections", "acc-report-section-layout-mastery"),
        ("customui-xml", "acc-ribbon-customui-xml-schema"),
        ("fluent-design-access", "acc-ui-modern-fluent-design-system"),
        ("ssma-migration", "acc-azure-sql-ssma-migration-architecture"),
        ("dataverse-linked-tables", "acc-dataverse-linking-tables-architecture"),
        ("split-database-architecture", "acc-split-db-fe-be-partitioning"),
        ("postgresql-access", "acc-open-db-postgresql-psqlodbc-integration"),
        ("serverxmlhttp-vba", "acc-rest-api-serverxmlhttp-client"),
        ("excel-com-automation", "acc-excel-com-automation-application"),
        ("outlook-com-automation", "acc-outlook-com-automation-mailitems"),
        ("word-com-automation", "acc-word-com-automation-documents"),
        ("sharepoint-linked-lists", "acc-sharepoint-linked-lists-architecture"),
        ("graph-api-access", "acc-graph-rest-api-access-desktop"),
        ("accdb-encryption", "acc-sec-accdb-database-password-aes"),
        ("access-source-control", "acc-git-rd-source-control-architecture"),
        ("tdd-rubberduck", "acc-test-ci-tdd-rubberduck-testing"),
        ("flow-state-vibe", "acc-vibe-flow-state-desktop-development"),
        ("access-runtime-packaging", "acc-deploy-free-access-runtime-packaging"),
    ]

    query_assertions = []
    for q, expected in query_tests:
        query_assertions.append(f"""        ("{q}", "{expected}"),""")
    query_table = "\n".join(query_assertions)

    rust_code = f"""//! Brutal Integration & Verification Tests for Top 350 Skills in Microsoft Access 365 & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{{AtomicUsize, Ordering}};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
}};

pub const ACCESS365_SKILLS_350: [&str; 350] = [
{skills_array}
];

// =========================================================================
// 1. Discovery of all 350 Access 365 Skills on Disk
// =========================================================================

#[test]
fn test_all_350_access365_skills_discovered_on_disk() {{
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &ACCESS365_SKILLS_350 {{
        assert!(
            loaded_map.contains(*skill),
            "Access 365 skill '{{}}' must be discovered in .ecc/skills directory",
            skill
        );
    }}
}}

// =========================================================================
// 2. Built-in Registration of all 350 Access 365 Skills (assert >= 1695 total skills)
// =========================================================================

#[test]
fn test_all_350_access365_skills_built_in_registration() {{
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 1695,
        "Expected at least 1695 total built-in skills including Access 365 suite, found {{}}",
        all_skills.len()
    );

    for skill_name in &ACCESS365_SKILLS_350 {{
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Access 365 skill '{{}}' must be registered in built-in skills",
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
    for skill_name in &ACCESS365_SKILLS_350 {{
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
        "relational-3nf",
        "storage-pages",
        "jet-ddl-create",
        "correlated-subqueries",
        "foreign-keys",
        "option-explicit",
        "structured-error-handling",
        "property-get-let",
        "ptrsafe-64bit",
        "vbscript-regexp",
        "dao-cursors",
        "disconnected-recordsets",
        "dsn-less-connections",
        "ace-oledb-provider",
        "no-locks-property",
        "form-event-sequence",
        "linkmasterfields",
        "report-sections",
        "customui-xml",
        "fluent-design-access",
        "ssma-migration",
        "dataverse-linked-tables",
        "split-database-architecture",
        "postgresql-access",
        "serverxmlhttp-vba",
        "excel-com-automation",
        "outlook-com-automation",
        "word-com-automation",
        "sharepoint-linked-lists",
        "graph-api-access",
        "accdb-encryption",
        "access-source-control",
        "tdd-rubberduck",
        "flow-state-vibe",
        "access-runtime-packaging",
    ]);

    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    let num_threads = 50;
    let iterations_per_thread = 20;

    let start = Instant::now();

    for t in 0..num_threads {{
        let d = Arc::clone(&dispatcher);
        let q = Arc::clone(&queries);
        let s = Arc::clone(&success_count);

        let handle = thread::spawn(move || {{
            for i in 0..iterations_per_thread {{
                let query = q[(t * iterations_per_thread + i) % q.len()];
                let res = d.dispatch(query, 3, None);
                if !res.is_empty() {{
                    s.fetch_add(1, Ordering::Relaxed);
                }}
            }}
        }});
        handles.push(handle);
    }}

    for handle in handles {{
        handle.join().expect("Worker thread panicked in stress test");
    }}

    let elapsed = start.elapsed();
    let total = success_count.load(Ordering::Relaxed);

    println!(
        "Concurrent stress test: {{}} dispatches across {{}} threads completed in {{:?}} ({{:.2}} us/dispatch)",
        total,
        num_threads,
        elapsed,
        (elapsed.as_micros() as f64) / (total as f64)
    );

    assert_eq!(
        total,
        num_threads * iterations_per_thread,
        "All {{}} dispatches must succeed",
        num_threads * iterations_per_thread
    );
}}
"""

    TARGET_TEST.write_text(rust_code, encoding="utf-8")
    print(f"Successfully generated {TARGET_TEST} with 6 brutal test suites.")

if __name__ == "__main__":
    main()
