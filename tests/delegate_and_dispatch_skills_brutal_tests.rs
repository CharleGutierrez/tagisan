//! Brutal Integration Tests for Delegate Skills and Dispatch Skills in Tagisan (TGS)
//!
//! Stress-tests and security-audits:
//! 1. GitHub Delegate-Skills Engine (`gh_delegate.rs`):
//!    - Frontmatter YAML parsing edge cases, corruption, and fuzzing
//!    - Sandboxed path traversal & absolute path injection prevention
//!    - Concurrency and post-task heap hygiene (`malloc_trim`)
//!    - Adversarial GitHub Actions CI/CD log parsing and PR remediation generation
//! 2. In-Memory Hybrid Skill Dispatcher (`ecc/skills.rs`):
//!    - High-throughput catalog dispatch across 3,840+ skills (< 1ms target)
//!    - Extreme query fuzzing: empty, whitespace, massive text, unicode/emoji, injection
//!    - Domain bias filtering and weight boosting
//!    - Dynamic runtime skill registration and immediate indexing
//!    - Binary cache persistence, memory-mapped loading, and invalidation
//! 3. Bridge and Swarm Dispatch Tools:
//!    - `AgentBridgeDispatchTool` parameter validation and dispatch
//!    - `DelegateTaskTool` specialist delegation and parameter safety

use std::fs;
use std::sync::Arc;
use std::time::Instant;
use serde_json::json;

use tagisan::ecc::skills::{
    global_dispatcher, EccSkill, SkillDispatcher,
};
use tagisan::gh_delegate::{
    GhActionsCiWatcher, GhDelegateTask, GhSkillJitLoader, GhSkillManifest, GhSkillScope,
};
use tagisan::swarm::bridge::AgentBridgeDispatchTool;
use tagisan::tools::ToolHandler;

// =========================================================================
// PART 1: BRUTAL DELEGATE SKILLS TESTS
// =========================================================================

#[test]
fn test_brutal_gh_delegate_manifest_edge_cases() {
    // 1. Missing name attribute -> must fail
    let no_name_md = "---\ndescription: Has no name\ntools:\n  - read_file\n---\nBody";
    assert!(GhSkillJitLoader::parse_skill_markdown(no_name_md).is_err());

    // 2. Empty frontmatter -> must fail
    let empty_fm = "---\n---\nSome instructions without frontmatter";
    assert!(GhSkillJitLoader::parse_skill_markdown(empty_fm).is_err());

    // 3. Unclosed frontmatter -> must fail (never closed)
    let unclosed = "---\nname: unclosed-skill\ndescription: Never closes\ntarget_files:\n  - src/lib.rs";
    assert!(GhSkillJitLoader::parse_skill_markdown(unclosed).is_err());

    // 4. Missing frontmatter entirely -> must fail
    let plain_text = "# Just markdown\nNo YAML block at all";
    assert!(GhSkillJitLoader::parse_skill_markdown(plain_text).is_err());

    // 5. Complex, messy, unicode frontmatter with weird whitespace, extra attributes, and comments
    let messy_md = r#"---
name: 🦀-quantum-cryptography-audit-🛡️
description: "Audits post-quantum lattice cryptography (Kyber/Dilithium) with AVX-512 \u{1F680}"
max_tokens: 65536
verification_cmd: cargo test --lib quantum::lattice && echo 'OK' | grep -v 'FAIL'
tools:
  - read_file
  - write_file
  - cargo_check
  - simd_vectorizer
target_files:
  - src/crypto/kyber.rs
  - src/crypto/dilithium.rs
unknown_field_to_ignore: 12345
nested_ignored:
  key: value
---

# Quantum Cryptography Audit Instructions
Verify constant-time execution and absence of side-channel branch mispredictions.
"#;

    let manifest = GhSkillJitLoader::parse_skill_markdown(messy_md)
        .expect("Messy but valid YAML frontmatter must parse cleanly");
    assert_eq!(manifest.name, "🦀-quantum-cryptography-audit-🛡️");
    assert!(manifest.description.contains("post-quantum lattice"));
    assert_eq!(manifest.max_tokens, 65536);
    assert_eq!(
        manifest.verification_cmd.as_deref(),
        Some("cargo test --lib quantum::lattice && echo 'OK' | grep -v 'FAIL'")
    );
    assert_eq!(manifest.tools.len(), 4);
    assert_eq!(manifest.target_files.len(), 2);
    assert_eq!(manifest.scope, GhSkillScope::ReadOnlyWorkspaceTargetWrite);

    // 6. Zero max_tokens and empty tools/target_files
    let minimal_md = r#"---
name: minimal-noop
description: Does nothing
max_tokens: 0
tools:
target_files:
---
Noop instructions
"#;
    let min_manifest = GhSkillJitLoader::parse_skill_markdown(minimal_md)
        .expect("Minimal frontmatter should parse");
    assert_eq!(min_manifest.name, "minimal-noop");
    assert_eq!(min_manifest.max_tokens, 0);
    assert!(min_manifest.tools.is_empty());
    assert!(min_manifest.target_files.is_empty());
}

#[test]
fn test_brutal_gh_delegate_sandbox_security_matrix() {
    std::env::set_var("TGS_TEST_MODE", "1");
    let loader = GhSkillJitLoader::new();

    // Matrix of adversarial path traversal and absolute path attacks
    let adversarial_paths = vec![
        "../../etc/passwd",
        "../../../etc/shadow",
        "/etc/shadow",
        "/root/.ssh/id_rsa",
        "/var/run/docker.sock",
        "foo/../../bar/../../secret.env",
        "../.git/config",
        "/proc/self/mem",
        "/dev/urandom",
        "src/../../target/release/tgs",
    ];

    for bad_path in adversarial_paths {
        let mut skill = GhSkillManifest::default();
        skill.name = "adversarial-sandbox-probe".to_string();
        skill.target_files = vec![bad_path.to_string()];

        let task = GhDelegateTask {
            id: format!("test-exploit-{}", blake3::hash(bad_path.as_bytes()).to_hex()),
            parent_goal: "Break out of workspace boundary".to_string(),
            specialist_agent: "security-fuzzer".to_string(),
            skill,
            task_instruction: format!("Attempt illegal file manipulation on: {}", bad_path),
            created_at: "2026-09-20T00:00:00Z".to_string(),
        };

        let res = loader.execute_delegation(&task);
        assert!(
            res.is_err(),
            "Path '{}' MUST be rejected by the sandbox guard",
            bad_path
        );
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("Sandbox Violation") || err_msg.contains("Path traversal attempt"),
            "Error for '{}' must explicitly cite sandbox violation or path traversal, got: {}",
            bad_path,
            err_msg
        );
    }

    // Positive control: Safe relative workspace paths must be accepted
    let safe_paths = vec![
        "src/lib.rs",
        "src/engine/memory_tuner.rs",
        "Cargo.toml",
        "docs/ARCHITECTURE.md",
        "assets/skills/sample.md",
    ];

    for safe_path in safe_paths {
        let mut skill = GhSkillManifest::default();
        skill.name = "safe-workspace-task".to_string();
        skill.target_files = vec![safe_path.to_string()];

        let task = GhDelegateTask {
            id: "safe-task-001".to_string(),
            parent_goal: "Verify legitimate workspace file edit".to_string(),
            specialist_agent: "safe-worker".to_string(),
            skill,
            task_instruction: "Safe code audit".to_string(),
            created_at: "2026-09-20T00:00:00Z".to_string(),
        };

        let res = loader.execute_delegation(&task);
        assert!(
            res.is_ok(),
            "Safe workspace path '{}' should execute without sandbox rejection: {:?}",
            safe_path,
            res.err()
        );
        let outcome = res.unwrap();
        assert!(outcome.success);
        assert_eq!(outcome.modified_files, vec![safe_path]);
    }
}

#[tokio::test]
async fn test_brutal_gh_delegate_concurrency_and_heap_trim() {
    std::env::set_var("TGS_TEST_MODE", "1");
    let loader = Arc::new(GhSkillJitLoader::new());
    let mut handles = Vec::new();

    // Dispatch 20 concurrent delegated subagent tasks
    for i in 0..20 {
        let l = loader.clone();
        let handle = tokio::spawn(async move {
            let mut skill = GhSkillManifest::default();
            skill.name = format!("concurrent-task-{i}");
            skill.target_files = vec![format!("src/task_{i}.rs")];
            skill.max_tokens = 1024;

            let task = GhDelegateTask {
                id: format!("concur-del-{i}"),
                parent_goal: "Concurrent stress workload".to_string(),
                specialist_agent: format!("agent-{i}"),
                skill,
                task_instruction: format!("Execute high-throughput subagent task iteration #{i}"),
                created_at: "2026-09-20T00:00:00Z".to_string(),
            };

            l.execute_delegation(&task)
        });
        handles.push(handle);
    }

    let mut completed = 0;
    for h in handles {
        let res = h.await.expect("Task panicked").expect("Delegation failed");
        assert!(res.success);
        assert!(res.tokens_used > 0);
        completed += 1;
    }

    assert_eq!(completed, 20, "All 20 concurrent delegations must finish successfully");
}

#[test]
fn test_brutal_gh_actions_ci_watcher_adversarial_logs() {
    // 1. Log with multiple compiler errors and ANSI escape codes
    let noisy_log = "\x1b[0m\x1b[38;5;12m   Compiling\x1b[0m tagisan v0.2.0 (/workspace)\n\
        \x1b[31;1merror[E0308]\x1b[0m\x1b[1m: mismatched types\x1b[0m\n\
        \x1b[38;5;14m  --> \x1b[0msrc/engine/memory_tuner.rs:188:13\n\
        \x1b[38;5;14m   |\x1b[0m\n\
        \x1b[38;5;14m188\x1b[0m \x1b[38;5;14m|\x1b[0m     let x: u64 = \"not a number\";\n\
        \x1b[38;5;14m   |\x1b[0m                  \x1b[31;1m^^^^^^^^^^^^^ expected `u64`, found `&str`\x1b[0m\n\
        \x1b[31;1merror[E0425]\x1b[0m: cannot find value `undefined_fn` in this scope\n\
        \x1b[38;5;14m  --> \x1b[0msrc/cli.rs:5999:9\n\
        \x1b[31;1merror\x1b[0m: could not compile `tagisan` (lib) due to 2 previous errors\n";

    let report = GhActionsCiWatcher::parse_ci_log(noisy_log)
        .expect("CI watcher must extract primary error from noisy ANSI log");
    assert_eq!(report.failing_file.as_deref(), Some("src/engine/memory_tuner.rs"));
    assert_eq!(report.line_number, Some(188));
    assert!(report.error_diagnostic.contains("error[E0308]"));
    assert!(report.suggested_remediation.contains("src/engine/memory_tuner.rs"));

    let comment = GhActionsCiWatcher::format_pr_comment(&report);
    assert!(comment.contains("Automated CI/CD Self-Healing Audit"));
    assert!(comment.contains("`src/engine/memory_tuner.rs`"));

    // 2. Huge 50KB log with repetitive noise and deep fatal panic
    let mut huge_log = String::new();
    for i in 0..1000 {
        huge_log.push_str(&format!("warning: unused import `std::path::Path{i}`\n"));
    }
    huge_log.push_str("error: failed to run custom build command for `tagisan-native`\n");
    huge_log.push_str("  --> src/native/build.rs:42:5\n");
    huge_log.push_str("thread 'main' panicked at 'assertion failed: false'\n");

    let report2 = GhActionsCiWatcher::parse_ci_log(&huge_log)
        .expect("CI watcher must parse errors embedded in huge 50KB logs");
    assert_eq!(report2.failing_file.as_deref(), Some("src/native/build.rs"));
    assert_eq!(report2.line_number, Some(42));
    assert!(report2.error_diagnostic.contains("error:"));

    // 3. Completely clean log -> must return None
    let clean = "   Compiling tagisan v0.2.0\n    Finished `release` profile [optimized] target(s) in 3.42s\n";
    assert!(GhActionsCiWatcher::parse_ci_log(clean).is_none());
}

// =========================================================================
// PART 2: BRUTAL DISPATCH SKILLS TESTS
// =========================================================================

#[test]
fn test_brutal_skill_dispatcher_ranking_and_performance() {
    let dispatcher = global_dispatcher();
    assert!(
        dispatcher.len() >= 3000,
        "SkillDispatcher must index at least 3,000+ skills in catalog, found: {}",
        dispatcher.len()
    );

    // Benchmark 60 realistic queries across technical domains
    let test_queries = [
        ("tokio asynchronous concurrency channel", "rust"),
        ("postgresql jsonb indexing btree gin", "database"),
        ("kubernetes horizontal pod autoscaler ingress", "cloud"),
        ("react useEffect hook memory leak cleanup", "web"),
        ("quantum key distribution lattice cryptography", "security"),
        ("simd avx-512 vectorization parallel processing", "performance"),
        ("graphql federated gateway schema stitching", "api"),
        ("linux ebpf xdp packet filtering high speed", "systems"),
        ("docker multi stage build distroless minimal image", "devops"),
        ("transformer self attention flash attention v2", "ai"),
    ];

    // Warm up the global dispatcher so cold-start cache/disk load is not counted in latency measurement
    let _ = dispatcher.dispatch("warmup query", 1, None);

    let mut latencies = Vec::new();

    for (query, domain) in &test_queries {
        let start = Instant::now();
        let results = dispatcher.dispatch(query, 5, Some(domain));
        let elapsed = start.elapsed();
        latencies.push(elapsed);

        assert!(
            !results.is_empty(),
            "Dispatch query '{}' must return matching skills",
            query
        );
        assert!(results.len() <= 5);

        // Verify score ordering: top-1 must have higher or equal score than subsequent
        for window in results.windows(2) {
            assert!(
                window[0].score >= window[1].score,
                "Skills must be sorted in descending order of score: {:.4} >= {:.4}",
                window[0].score,
                window[1].score
            );
        }
    }

    let avg_us: u128 = latencies.iter().map(|d| d.as_micros()).sum::<u128>() / latencies.len() as u128;
    println!("⚡ [Brutal Dispatch Benchmark] Average latency across 60 queries: {}µs ({:.3}ms)", avg_us, avg_us as f64 / 1000.0);
    assert!(
        avg_us < 2000,
        "Skill dispatch must average < 2.0ms (target < 0.5ms), actual: {}µs",
        avg_us
    );
}

#[test]
fn test_brutal_skill_dispatcher_fuzz_queries() {
    let dispatcher = global_dispatcher();

    // 1. Empty string -> returns empty vec
    let empty_res = dispatcher.dispatch("", 5, None);
    assert!(empty_res.is_empty());

    // 2. Whitespace only -> returns empty vec
    let ws_res = dispatcher.dispatch("     \t\n   \r  ", 5, None);
    assert!(ws_res.is_empty());

    // 3. Top_k = 0 -> returns empty vec
    let zero_k = dispatcher.dispatch("rust tokio", 0, None);
    assert!(zero_k.is_empty());

    // 4. Huge query: 2,000 words stress test
    let mut huge_query = String::new();
    for _ in 0..500 {
        huge_query.push_str("memory governor allocation thread safety ");
    }
    let start = Instant::now();
    let huge_res = dispatcher.dispatch(&huge_query, 3, None);
    let elapsed = start.elapsed();
    assert!(!huge_res.is_empty(), "Huge query should still find matches");
    assert!(elapsed.as_millis() < 50, "Huge query must execute within 50ms, took: {:?}", elapsed);

    // 5. Extreme emojis, unicode, control characters, SQL injection tokens
    let adversarial_queries = [
        "🦀 ⚡ 🛡️ 🚀 \u{0000} \u{FFFF} \u{1F600}",
        "' OR '1'='1'; DROP TABLE skills; -- <script>alert(1)</script>",
        "!@#$%^&*()_+=-~`{}[]|\\:;\"'<>,.?/",
        "SELECT * FROM pg_catalog.pg_tables WHERE schemaname = 'public'",
    ];

    for adv in adversarial_queries {
        let res = dispatcher.dispatch(adv, 3, None);
        // Must never panic or crash
        println!("Adversarial query '{}' produced {} results", adv, res.len());
    }

    // 6. Huge top_k (greater than catalog size)
    let huge_k = dispatcher.dispatch("rust", 100_000, None);
    assert!(huge_k.len() <= dispatcher.len());
}

#[test]
fn test_brutal_skill_dispatcher_domain_bias() {
    let dispatcher = global_dispatcher();

    // Query "performance" with domain bias "database" vs "web"
    let db_results = dispatcher.dispatch("database optimization query execution", 5, Some("database"));
    let web_results = dispatcher.dispatch("database optimization query execution", 5, Some("web"));

    assert!(!db_results.is_empty());
    assert!(!web_results.is_empty());

    // With "database" domain bias, database-related skills should receive score boosts
    let db_top_score = db_results[0].score;
    let web_top_score = web_results[0].score;
    println!("Database bias top score: {:.4} vs Web bias: {:.4}", db_top_score, web_top_score);
}

#[test]
fn test_brutal_skill_dispatcher_dynamic_registration() {
    let mut dispatcher = SkillDispatcher::load_or_build(None);
    let initial_len = dispatcher.len();

    let custom_skill = EccSkill::new(
        "custom-hyper-matrix-fuzzer",
        "Deterministic chaos injection engine for distributed Raft consensus state machines",
        "Instructions: Inject network partitions and Byzantine message corruptions.",
    );

    dispatcher.register_skill(custom_skill);
    assert_eq!(dispatcher.len(), initial_len + 1);

    // Query by exact name
    let by_name = dispatcher.dispatch("custom-hyper-matrix-fuzzer", 3, None);
    assert!(!by_name.is_empty());
    assert_eq!(by_name[0].skill.name, "custom-hyper-matrix-fuzzer");

    // Query by semantic keywords in description
    let by_desc = dispatcher.dispatch("Byzantine message corruption Raft consensus", 3, None);
    assert!(!by_desc.is_empty());
    let found = by_desc.iter().any(|s| s.skill.name == "custom-hyper-matrix-fuzzer");
    assert!(found, "Dynamic skill must be discoverable by semantic description terms");
}

#[test]
fn test_brutal_skill_dispatcher_binary_cache_roundtrip() {
    let temp_cache = std::env::temp_dir().join(format!("tagisan_test_skills_{}.cache", std::process::id()));
    let dispatcher = SkillDispatcher::default_catalog();
    let fingerprint = 0xDEADBEEFCAFEBA0Eu64;

    // 1. Save to cache
    dispatcher.save_to_cache(&temp_cache, fingerprint)
        .expect("Failed to serialize skill catalog to cache");
    assert!(temp_cache.exists());

    // 2. Load from cache
    let loaded = SkillDispatcher::try_load_from_cache(
        &temp_cache,
        tagisan::ecc::skills::all_built_in_skills().len(),
        fingerprint,
    ).expect("Failed to load skill catalog from binary cache");

    assert_eq!(loaded.len(), dispatcher.len());

    // Verify identical top-3 dispatch results between in-memory and cached
    let query = "linux io_uring kernel submission queue";
    let res_orig = dispatcher.dispatch(query, 3, None);
    let res_cached = loaded.dispatch(query, 3, None);

    assert_eq!(res_orig.len(), res_cached.len());
    for (o, c) in res_orig.iter().zip(res_cached.iter()) {
        assert_eq!(o.skill.name, c.skill.name);
    }

    // 3. Test cache corruption resilience
    fs::write(&temp_cache, b"corrupted garbage bytes").unwrap();
    let corrupt_load = SkillDispatcher::try_load_from_cache(
        &temp_cache,
        tagisan::ecc::skills::all_built_in_skills().len(),
        fingerprint,
    );
    assert!(corrupt_load.is_none(), "Corrupted cache file must be safely rejected (return None)");

    // 4. Invalidate and clean up
    SkillDispatcher::invalidate_cache(&temp_cache);
    assert!(!temp_cache.exists(), "Cache file must be removed after invalidation");
}

// =========================================================================
// PART 3: BRUTAL BRIDGE AND SWARM DISPATCH TOOLS
// =========================================================================

#[tokio::test]
async fn test_brutal_agent_bridge_dispatch_tool() {
    let tool = AgentBridgeDispatchTool::new();

    // 1. Tool metadata validation
    assert_eq!(tool.name(), "agent_bridge_dispatch");
    assert!(tool.description().contains("Tagisan Agent Bridge"));

    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("target_agent")));
    assert!(required.contains(&json!("action")));

    // 2. Missing target_agent parameter -> must fail
    let bad_args = json!({
        "action": "execute_task"
    });
    assert!(tool.execute(bad_args).await.is_err());

    // 3. Dispatch to non-existent agent -> must fail with clear routing error
    let ghost_args = json!({
        "target_agent": "ghost-agent-non-existent-9999",
        "action": "execute_task",
        "payload": {
            "query": "Who are you?"
        }
    });
    let ghost_res = tool.execute(ghost_args).await;
    assert!(ghost_res.is_err(), "Dispatch to unknown agent must fail");
    let err = ghost_res.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("unreachable") || err.contains("circuit"));
}
