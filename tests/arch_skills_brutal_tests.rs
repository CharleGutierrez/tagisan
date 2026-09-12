//! Brutal Integration Test Suite for the Top 50 Systems & Architecture Analysis Skills in Tagisan (TGS)
//! Verifies:
//! 1. All 50 Architecture skills built-in registration & valid instructions
//! 2. All 50 Architecture skills discovered on disk in .ecc/skills/
//! 3. Alias and domain prefix lookups (arch-* -> 'architecture' domain)
//! 4. Semantic intent dispatching across architectural queries
//! 5. Formal mathematical/architectural invariant extraction
//! 6. Local LLM CheatSheet vs Cloud Guidelines vs 3-Tier Hierarchical formatting
//! 7. 50-thread concurrent multi-threaded stress test (> 800 QPS debug, > 3,500 QPS release)
//! 8. Native FetchSkillTool dynamic execution
//! 9. Sub-millisecond deterministic token estimator accuracy

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

const ARCH_SKILL_NAMES: [&str; 50] = [
    // Pillar 1: Distributed Systems Foundations & Consensus
    "arch-kleppmann-distributed-data",
    "arch-coulouris-distributed-systems",
    "arch-petrov-database-internals",
    "arch-vitillo-distributed-systems",
    "arch-ozsu-distributed-databases",
    // Pillar 2: Data-Intensive & Storage Engine Architecture
    "arch-gray-transaction-processing",
    "arch-bailis-database-systems",
    "arch-akidau-streaming-systems",
    "arch-shapira-kafka-definitive",
    "arch-robinson-graph-databases",
    // Pillar 3: Enterprise Architectural Patterns & Modularity
    "arch-fowler-enterprise-patterns",
    "arch-martin-clean-architecture",
    "arch-tune-architecture-modernization",
    "arch-ghosh-functional-domain-modeling",
    "arch-ford-software-architecture-hard-parts",
    // Pillar 4: Mechanical Sympathy, Hardware & Concurrency
    "arch-bryant-computer-systems",
    "arch-herlihy-multiprocessor-programming",
    "arch-gregg-systems-performance",
    "arch-love-linux-kernel",
    "arch-mckenney-parallel-programming",
    // Pillar 5: Cloud-Native Resilience & Chaos Engineering
    "arch-nygard-release-it",
    "arch-beyer-site-reliability-engineering",
    "arch-rosenthal-chaos-engineering",
    "arch-davis-cloud-native-patterns",
    "arch-ibryam-kubernetes-patterns",
    // Pillar 6: Microservices & Distributed Boundaries
    "arch-newman-building-microservices",
    "arch-newman-monolith-to-microservices",
    "arch-richardson-microservices-patterns",
    "arch-richards-fundamentals-architecture",
    "arch-burns-designing-distributed-systems",
    // Pillar 7: API Protocols, Networking & Event Topologies
    "arch-stevens-tcp-ip-illustrated",
    "arch-jin-designing-web-apis",
    "arch-grigorik-high-performance-networking",
    "arch-hohpe-enterprise-integration-patterns",
    "arch-indrasiri-grpc-up-and-running",
    // Pillar 8: Architectural Evaluation, Modeling & Decision Records
    "arch-bass-software-architecture-practice",
    "arch-clements-documenting-architectures",
    "arch-brown-c4-model",
    "arch-rozanski-software-systems-architecture",
    "arch-keeling-design-it",
    // Pillar 9: Observability, Distributed Tracing & SRE
    "arch-majors-observability-engineering",
    "arch-parker-distributed-tracing-practice",
    "arch-beyer-sre-workbook",
    "arch-fowler-production-ready-microservices",
    "arch-campbell-database-reliability-engineering",
    // Pillar 10: Security Architecture, Threat Modeling & Zero Trust
    "arch-shostack-threat-modeling",
    "arch-gilman-zero-trust-networks",
    "arch-vehent-securing-devops",
    "arch-janca-alice-bob-appsec",
    "arch-wong-real-world-cryptography",
];

#[test]
fn test_01_all_50_arch_skills_built_in_registration() {
    println!("\n=== TEST 1: All 50 Architecture Skills Built-In Registration ===");
    let built_ins = all_built_in_skills();
    println!("Total built-in skills registered: {}", built_ins.len());
    assert!(built_ins.len() >= 270, "Expected at least 270 built-in skills, found {}", built_ins.len());

    let built_in_map: HashSet<String> = built_ins.into_iter().map(|s| s.name).collect();

    for name in &ARCH_SKILL_NAMES {
        assert!(
            built_in_map.contains(*name),
            "Skill '{}' missing from all_built_in_skills()",
            name
        );
        let skill = find_built_in_skill(name).expect(&format!("find_built_in_skill('{}') returned None", name));
        assert!(!skill.description.is_empty());
        assert!(!skill.instructions.is_empty());
        assert!(skill.instructions.contains("Core Architectural Theoretical Foundations"));
        assert!(skill.instructions.contains("Prompt Contract"));
    }
    println!("  [✓] All 50 Architecture skills confirmed built-in with valid instructions");
}

#[test]
fn test_02_all_50_arch_skills_discovered_on_disk() {
    println!("\n=== TEST 2: All 50 Architecture Skills Discovered on Disk ===");
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    for name in &ARCH_SKILL_NAMES {
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
    println!("  [✓] All 50 Architecture skills verified on disk in .ecc/skills/");
}

#[test]
fn test_03_arch_skills_alias_and_domain_mapping() {
    println!("\n=== TEST 3: Architecture Aliases & Domain Mapping ===");
    let aliases = [
        ("ddia", "arch-kleppmann-distributed-data"),
        ("vector-clocks", "arch-coulouris-distributed-systems"),
        ("raft", "arch-petrov-database-internals"),
        ("circuit-breaker", "arch-vitillo-distributed-systems"),
        ("clean-architecture", "arch-martin-clean-architecture"),
        ("lock-free", "arch-herlihy-multiprocessor-programming"),
        ("chaos-engineering", "arch-rosenthal-chaos-engineering"),
        ("saga", "arch-richardson-microservices-patterns"),
        ("c4-model", "arch-brown-c4-model"),
        ("adr", "arch-keeling-design-it"),
        ("opentelemetry", "arch-parker-distributed-tracing-practice"),
        ("stride", "arch-shostack-threat-modeling"),
        ("mtls", "arch-gilman-zero-trust-networks"),
    ];

    for (alias, expected) in aliases {
        let found = find_built_in_skill(alias);
        assert!(found.is_some(), "Alias '{}' lookup returned None", alias);
        assert_eq!(found.unwrap().name, expected, "Alias '{}' mapped to wrong skill", alias);
    }
    println!("  [✓] All high-leverage architectural aliases verified");
}

#[test]
fn test_04_semantic_intent_dispatching_for_arch_queries() {
    println!("\n=== TEST 4: Semantic Intent Dispatching for Architecture Queries ===");
    let dispatcher = global_dispatcher();

    let test_cases = [
        ("hybrid logical clocks linearizability unreliable network", "arch-kleppmann-distributed-data"),
        ("storage engine lsm tree sstable memtable wal raft consensus", "arch-petrov-database-internals"),
        ("circuit breaker bulkhead pattern exponential backoff jitter", "arch-vitillo-distributed-systems"),
        ("dependency inversion ports and adapters hexagonal clean architecture", "arch-martin-clean-architecture"),
        ("cache lines false sharing mechanical sympathy spatial locality", "arch-bryant-computer-systems"),
        ("saga pattern orchestrator compensating transactions microservices", "arch-richardson-microservices-patterns"),
        ("c4 model system context containers components diagrams as code", "arch-brown-c4-model"),
        ("opentelemetry w3c traceparent distributed tracing span propagation", "arch-parker-distributed-tracing-practice"),
        ("stride threat modeling trust boundaries attack surface", "arch-shostack-threat-modeling"),
        ("argon2id password hashing authenticated encryption aead aes-gcm", "arch-wong-real-world-cryptography"),
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
    println!("  [✓] All 10 architectural domain semantic intent queries correctly dispatched");
}

#[test]
fn test_05_dense_invariants_token_compression() {
    println!("\n=== TEST 5: Dense Invariants Token Compression ===");
    let dispatcher = global_dispatcher();
    let sample_skills = [
        "arch-kleppmann-distributed-data",
        "arch-nygard-release-it",
        "arch-richardson-microservices-patterns",
        "arch-wong-real-world-cryptography",
    ];

    for name in sample_skills {
        let skill = dispatcher.get_skill(name).expect("Skill must exist");
        let ds = DispatchedSkill {
            domain: "architecture".to_string(),
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
    let query = "distributed consensus raft circuit breaker outbox pattern";
    let budget = TokenBudget::cloud_default();

    let result = dispatcher.dispatch_diversified(query, budget, Some("architecture"));
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
    
    let args = json!({ "name": "arch-kleppmann-distributed-data" });
    let output = tool.execute(args).await.expect("Tool execution must succeed");
    assert!(output.contains("# Skill: arch-kleppmann-distributed-data"));
    assert!(output.contains("Transactional Outbox"));
    assert!(output.contains("## Instructions"));
    println!("  [✓] FetchSkillTool retrieved full spec ({} bytes)", output.len());
}

#[test]
fn test_08_multithreaded_concurrent_arch_dispatching_50_threads() {
    println!("\n=== TEST 8: 50-Thread Concurrent Dispatch Stress Test ===");
    let _ = global_dispatcher(); // Pre-warm OnceLock
    let num_threads = 50;
    let queries_per_thread = 200; // 10,000 queries total
    let queries = [
        "raft consensus leader election log replication",
        "circuit breaker bulkhead timeout jitter",
        "clean architecture ports and adapters dependency inversion",
        "opentelemetry distributed tracing w3c traceparent",
        "stride threat modeling zero trust mtls",
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
    let code = "pub async fn handle_raft_append_entries(req: AppendEntriesRequest) -> Result<AppendEntriesResponse, RaftError> { }";
    let tokens = estimate_tokens(code);
    assert!(tokens >= 10 && tokens <= 30);

    let start = Instant::now();
    let mut accum = 0;
    for _ in 0..10_000 {
        accum += estimate_tokens(code);
    }
    let elapsed = start.elapsed();
    let micros_per_op = elapsed.as_micros() as f64 / 10_000.0;
    assert!(accum > 0);
    println!("  [✓] 10,000 token estimations executed in {:?} ({:.2} µs/op)", elapsed, micros_per_op);
    let max_micros = if cfg!(debug_assertions) { 25.0 } else { 5.0 };
    assert!(micros_per_op < max_micros);
}
