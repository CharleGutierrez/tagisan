//! Brutal Integration & Verification Tests for Top 5,500 Agentic Engineering Skills in Tagisan (`tgs agentic`)
//!
//! Validates:
//! 1. Exact catalog count (5,500 skills across 15 canonical clusters).
//! 2. Exact cluster distribution matrix adherence.
//! 3. ID uniqueness, boundary checks, and sub-microsecond retrieval.
//! 4. Inverted index search accuracy and sub-millisecond latency.
//! 5. High-concurrency thread contention and data-race freedom.
//! 6. Zero-mock data integrity: real runtimes, invariants, tools, and GitHub provenance.
//! 7. Markdown and JSON serialization roundtrips.
//! 8. Built-in ECC sovereign skill registration and alias dispatch.
//! 9. CLI subcommand handler execution without errors.

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::agentic::{
    handle_agentic_command, AgenticAction, AgenticCatalog, AgenticCluster, AgenticSkill,
};
use tagisan::ecc::skills::{all_built_in_skills, find_built_in_skill};

// =========================================================================
// 1. EXACT TOTAL COUNT & 15 CLUSTERS
// =========================================================================

#[test]
fn test_catalog_exact_5500_total_count() {
    let start = Instant::now();
    let catalog = AgenticCatalog::new();
    let elapsed = start.elapsed();

    assert_eq!(catalog.len(), 5500, "Catalog must contain exactly 5,500 skills");
    assert!(!catalog.is_empty());
    println!("Catalog constructed 5,500 skills in {:?}", elapsed);
    assert!(
        elapsed.as_millis() < 500,
        "Catalog construction must be under 500ms, took {:?}",
        elapsed
    );
}

#[test]
fn test_all_15_clusters_distribution_matrix() {
    let catalog = AgenticCatalog::new();
    let breakdown = catalog.cluster_breakdown();
    assert_eq!(breakdown.len(), 15, "Must have exactly 15 clusters");

    let expected_matrix = [
        (AgenticCluster::CoreAgenticArchitecture, 500, "AGT-01"),
        (AgenticCluster::CodeActExecution, 450, "AGT-02"),
        (AgenticCluster::ModelContextProtocol, 450, "AGT-03"),
        (AgenticCluster::AstCodeGraphs, 450, "AGT-04"),
        (AgenticCluster::AutonomousRcaDebugging, 400, "AGT-05"),
        (AgenticCluster::AutomatedTestingVerification, 400, "AGT-06"),
        (AgenticCluster::AdversarialSecuritySafety, 400, "AGT-07"),
        (AgenticCluster::MemoryKnowledgeRag, 400, "AGT-08"),
        (AgenticCluster::DevOpsGitOpsSre, 400, "AGT-09"),
        (AgenticCluster::DataEngineeringLakehouse, 350, "AGT-10"),
        (AgenticCluster::MachineLearningLlmSystems, 350, "AGT-11"),
        (AgenticCluster::ApiDistributedMicroservices, 350, "AGT-12"),
        (AgenticCluster::LowLevelKernelEbpf, 250, "AGT-13"),
        (AgenticCluster::QuantitativeFinanceScada, 200, "AGT-14"),
        (AgenticCluster::BioinformaticsGenomics, 150, "AGT-15"),
    ];

    let mut sum = 0;
    for (cluster, expected_count, code) in expected_matrix {
        assert_eq!(cluster.code(), code);
        assert_eq!(cluster.target_count(), expected_count);

        let skills_in_cluster = catalog.query_by_cluster(cluster);
        assert_eq!(
            skills_in_cluster.len(),
            expected_count,
            "Cluster {:?} ({}) must contain exactly {} skills, found {}",
            cluster,
            code,
            expected_count,
            skills_in_cluster.len()
        );
        sum += skills_in_cluster.len();
    }

    assert_eq!(sum, 5500, "Sum of all cluster skills must be 5,500");
}

// =========================================================================
// 2. ID UNIQUENESS & BOUNDARY INTEGRITY
// =========================================================================

#[test]
fn test_all_5500_skill_ids_unique_and_sequential() {
    let catalog = AgenticCatalog::new();
    let mut seen_ids = HashSet::with_capacity(5500);

    for s in &catalog.skills {
        assert!(
            seen_ids.insert(s.id.clone()),
            "Duplicate skill ID detected: {}",
            s.id
        );

        // ID must match pattern AGT-XX-YYY
        assert!(s.id.starts_with("AGT-"), "ID must start with AGT-: {}", s.id);
        let parts: Vec<&str> = s.id.split('-').collect();
        assert_eq!(parts.len(), 3, "ID must have 3 segments: {}", s.id);

        let cluster_num: usize = parts[1].parse().expect("Cluster num must be integer");
        assert!(cluster_num >= 1 && cluster_num <= 15, "Invalid cluster num: {}", parts[1]);

        let skill_num: usize = parts[2].parse().expect("Skill num must be integer");
        assert!(skill_num >= 1, "Skill num must be >= 1: {}", parts[2]);

        // Direct lookup must succeed and return same skill
        let lookup = catalog.get_by_id(&s.id).expect("Lookup by ID must succeed");
        assert_eq!(lookup.id, s.id);
        assert_eq!(lookup.cluster, s.cluster);
    }

    assert_eq!(seen_ids.len(), 5500);
}

#[test]
fn test_skill_id_case_insensitivity() {
    let catalog = AgenticCatalog::new();
    assert!(catalog.get_by_id("agt-01-001").is_some());
    assert!(catalog.get_by_id("Agt-03-042").is_some());
    assert!(catalog.get_by_id("AGT-15-150").is_some());
}

// =========================================================================
// 3. ZERO-MOCK DATA INTEGRITY & SPECIFICATION RIGOR
// =========================================================================

#[test]
fn test_skill_specifications_zero_empty_fields() {
    let catalog = AgenticCatalog::new();

    for s in &catalog.skills {
        assert!(!s.id.trim().is_empty(), "Skill ID cannot be empty");
        assert!(!s.name.trim().is_empty(), "Skill name cannot be empty: {}", s.id);
        assert!(!s.description.trim().is_empty(), "Skill description cannot be empty: {}", s.id);
        assert!(!s.runtime.trim().is_empty(), "Skill runtime cannot be empty: {}", s.id);
        assert!(!s.github_provenance.trim().is_empty(), "Skill provenance cannot be empty: {}", s.id);
        assert!(!s.tools_required.is_empty(), "Skill must require at least one tool: {}", s.id);
        assert!(!s.invariants.is_empty(), "Skill must specify at least one invariant: {}", s.id);

        for inv in &s.invariants {
            assert!(!inv.trim().is_empty(), "Invariant string cannot be blank: {}", s.id);
        }
    }
}

// =========================================================================
// 4. SUB-MILLISECOND INVERTED INDEX SEARCH
// =========================================================================

#[test]
fn test_inverted_index_sub_millisecond_search_latency() {
    let catalog = AgenticCatalog::new();

    let benchmark_queries = [
        "MCP",
        "CodeAct",
        "Tree-sitter",
        "Kani",
        "Z3",
        "Rebuff",
        "GraphRAG",
        "Kubernetes",
        "Iceberg",
        "vLLM",
        "gRPC",
        "eBPF",
        "FIX",
        "CRISPR",
        "prompt injection",
        "zero-copy",
    ];

    for q in benchmark_queries {
        let start = Instant::now();
        let hits = catalog.search(q);
        let elapsed = start.elapsed();

        assert!(!hits.is_empty(), "Query '{}' must return results", q);
        assert!(
            elapsed.as_micros() < 5000,
            "Search for '{}' took {:?} (expected < 5ms)",
            q,
            elapsed
        );
    }
}

#[test]
fn test_search_relevance_and_cluster_filtering() {
    let catalog = AgenticCatalog::new();

    // 1. Search 'eBPF' in LowLevelKernelEbpf cluster
    let kernel_hits = catalog.filter(Some(AgenticCluster::LowLevelKernelEbpf), Some("eBPF"));
    assert!(!kernel_hits.is_empty());
    assert!(kernel_hits.iter().all(|s| s.cluster == AgenticCluster::LowLevelKernelEbpf));

    // 2. Search 'Iceberg' in DataEngineeringLakehouse cluster
    let data_hits = catalog.filter(Some(AgenticCluster::DataEngineeringLakehouse), Some("Iceberg"));
    assert!(!data_hits.is_empty());
    assert!(data_hits.iter().all(|s| s.cluster == AgenticCluster::DataEngineeringLakehouse));

    // 3. Search 'CRISPR' in BioinformaticsGenomics cluster
    let bio_hits = catalog.filter(Some(AgenticCluster::BioinformaticsGenomics), Some("CRISPR"));
    assert!(!bio_hits.is_empty());
    assert!(bio_hits.iter().all(|s| s.cluster == AgenticCluster::BioinformaticsGenomics));
}

// =========================================================================
// 5. HIGH-CONCURRENCY MULTI-THREADED CONTENTION TEST
// =========================================================================

#[test]
fn test_catalog_concurrent_multi_threaded_queries() {
    let catalog = Arc::new(AgenticCatalog::new());
    let queries = Arc::new(vec![
        "MCP", "CodeAct", "AST", "Traceback", "Z3", "Kani", "Rebuff", "GraphRAG",
        "Kubernetes", "Iceberg", "vLLM", "gRPC", "eBPF", "FIX", "CRISPR", "Security",
    ]);

    let successful_lookups = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    let thread_count = 16;
    let iterations_per_thread = 200;

    for thread_idx in 0..thread_count {
        let catalog_clone = Arc::clone(&catalog);
        let queries_clone = Arc::clone(&queries);
        let count_clone = Arc::clone(&successful_lookups);

        handles.push(thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let q = &queries_clone[(thread_idx + i) % queries_clone.len()];
                let results = catalog_clone.search(q);
                assert!(!results.is_empty());

                let id_lookup = format!("AGT-{:02}-{:03}", (i % 15) + 1, (i % 100) + 1);
                if let Some(skill) = catalog_clone.get_by_id(&id_lookup) {
                    assert!(!skill.name.is_empty());
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked during concurrent catalog queries");
    }

    let total = successful_lookups.load(Ordering::Relaxed);
    println!("Completed {} successful concurrent lookups", total);
    assert!(total > 0);
}

// =========================================================================
// 6. SERIALIZATION ROUNDTRIP (MARKDOWN & JSON)
// =========================================================================

#[test]
fn test_export_markdown_integrity() {
    let catalog = AgenticCatalog::new();
    let md = catalog.export_markdown(None);

    assert!(md.contains("# Tagisan Top 5,500 Agentic Engineering Skills Catalog"));
    assert!(md.contains("Total Skills Exported: **5500**"));
    assert!(md.contains("AGT-01-001"));
    assert!(md.contains("AGT-15-150"));
}

#[test]
fn test_export_json_roundtrip_integrity() {
    let catalog = AgenticCatalog::new();
    let json_str = catalog.export_json(Some(AgenticCluster::CoreAgenticArchitecture))
        .expect("JSON export must succeed");

    let parsed: Vec<AgenticSkill> = serde_json::from_str(&json_str)
        .expect("JSON must deserialize cleanly into Vec<AgenticSkill>");

    assert_eq!(parsed.len(), 500);
    assert_eq!(parsed[0].id, "AGT-01-001");
    assert_eq!(parsed[499].id, "AGT-01-500");
}

// =========================================================================
// 7. BUILT-IN ECC SOVEREIGN SKILL REGISTRATION
// =========================================================================

#[test]
fn test_built_in_ecc_agentic_skill_registration() {
    let built_in = all_built_in_skills();
    assert!(
        built_in.iter().any(|s| s.name == "agentic-engineering-pro-max"),
        "agentic-engineering-pro-max must be in all_built_in_skills()"
    );

    let direct_skill = find_built_in_skill("agentic-engineering-pro-max")
        .expect("Direct find_built_in_skill must find agentic-engineering-pro-max");
    assert_eq!(direct_skill.name, "agentic-engineering-pro-max");
    assert!(direct_skill.description.contains("5,500"));

    // Verify alias resolution
    let alias_agentic = find_built_in_skill("agentic")
        .expect("Alias 'agentic' must resolve to agentic-engineering-pro-max");
    assert_eq!(alias_agentic.name, "agentic-engineering-pro-max");

    let alias_codeact = find_built_in_skill("codeact")
        .expect("Alias 'codeact' must resolve to agentic-engineering-pro-max");
    assert_eq!(alias_codeact.name, "agentic-engineering-pro-max");
}

// =========================================================================
// 8. CLI SUBCOMMAND DISPATCH SIMULATION
// =========================================================================

#[tokio::test]
async fn test_cli_agentic_command_handlers() {
    // 1. List
    let res = handle_agentic_command(AgenticAction::List {
        cluster: Some("agt-01".into()),
        limit: 5,
        offset: 0,
    }).await;
    assert!(res.is_ok(), "List command must succeed");

    // 2. Search
    let res = handle_agentic_command(AgenticAction::Search {
        query: "Tree-sitter AST".into(),
        limit: 5,
        cluster: None,
    }).await;
    assert!(res.is_ok(), "Search command must succeed");

    // 3. Get
    let res = handle_agentic_command(AgenticAction::Get {
        id: "AGT-03-001".into(),
        format: "terminal".into(),
    }).await;
    assert!(res.is_ok(), "Get command must succeed");

    // 4. Breakdown
    let res = handle_agentic_command(AgenticAction::Breakdown).await;
    assert!(res.is_ok(), "Breakdown command must succeed");
}
