use std::sync::Arc;
use tagisan::ecc::skill_jit::{SkillJitManager, SkillTier};
use tagisan::ecc::semantic_guard::{SemanticInvariantGuard, SemanticViolation};
use tagisan::providers::colibri::{ColibriConfig, ColibriProvider, StripingMode};
use tagisan::swarm::atlas::{SwarmAtlas, TopicCluster};
use tagisan::swarm::cluster::{
    query_cluster_status, ClusterCoordinator, ClusterWorker, ToolExecutionTask,
};
use tagisan::types::{CompletionRequest, ToolDefinition};

#[tokio::test]
async fn test_skill_jit_paging_and_pilot_prefetch() {
    let jit = SkillJitManager::new(2, 8, None);

    // 1. Initial lookup from L3 -> pages into L1
    assert_eq!(jit.get_tier("tokio-async-tuning"), SkillTier::L3Cold);
    let skill = jit.page_in("tokio-async-tuning").expect("Paging in skill failed");
    assert_eq!(jit.get_tier("tokio-async-tuning"), SkillTier::L1Active);
    assert_eq!(skill.name, "tokio-async-tuning");

    // 2. Second access is an L1 Hit
    let skill2 = jit.page_in("tokio-async-tuning").expect("Paging in cached skill failed");
    assert_eq!(skill2.name, "tokio-async-tuning");
    let stats = jit.stats();
    assert_eq!(stats.l1_hits, 1);

    // 3. Pinning
    jit.pin_skill("tokio-async-tuning");
    assert!(jit.is_pinned("tokio-async-tuning"));

    // 4. PILOT 1-step lookahead prefetching
    let tasks = ["compile and resolve error diagnostics", "audit system security vulnerabilities"];
    let prefetched = jit.pilot_prefetch(&tasks);
    assert!(prefetched >= 1);
    let post_stats = jit.stats();
    assert!(post_stats.pilot_prefetches >= 1);
}

#[test]
fn test_agent_and_skill_atlas_heat_tracking() {
    let mut atlas = SwarmAtlas::new();

    // Record routes
    atlas.record_route("architect", true, Some("strategy"), true, 120, 450);
    atlas.record_route("architect", true, Some("strategy"), true, 110, 420);
    atlas.record_route("security-auditor", true, Some("security"), true, 200, 800);
    atlas.record_route("rust-tokio-concurrency", false, Some("systems"), true, 15, 120);

    let entry = atlas.get_entry("architect").expect("architect not in atlas");
    assert_eq!(entry.invocations, 2);
    assert_eq!(entry.successes, 2);
    assert!(entry.heat_score > 0.0);

    // Verify cluster affinity
    let cluster = TopicCluster::infer("kernel-memory-guard", "low-level io and kernel memory management", &[]);
    assert_eq!(cluster, TopicCluster::Systems);

    // Render table
    let table = atlas.render_atlas_table();
    assert!(table.contains("TAGISAN SWARM CORTEX"));
    assert!(table.contains("architect"));
}

#[tokio::test]
async fn test_colibri_inference_provider() {
    let mut config = ColibriConfig::default();
    config.vram_budget_mb = 8192;
    config.disk_stream_chunk_kb = 256;
    config.striping_mode = StripingMode::SingleSsd;

    let report = config.validate_paths().expect("Validation failed");
    assert_eq!(report.chunk_size_kb, 256);

    let provider = ColibriProvider::new(config);
    let req = CompletionRequest::new("deepseek-v4", "ping status check");
    let resp = tagisan::providers::LlmProvider::complete(&provider, req).await.expect("Completion failed");

    assert_eq!(resp.provider, "colibri");
    assert!(resp.message.extract_text().contains("Colibrì"));
}

#[tokio::test]
async fn test_p2p_cluster_mesh_loopback() {
    let port = 18765;
    let bind_addr = format!("127.0.0.1:{}", port);

    let coordinator = Arc::new(ClusterCoordinator::new(&bind_addr));
    let coord_clone = coordinator.clone();

    // Spawn coordinator server
    let coord_handle = tokio::spawn(async move {
        let _ = coord_clone.run_server().await;
    });

    // Give server time to bind
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Spawn worker
    let worker_id = "test-node-alpha".to_string();
    let worker = Arc::new(ClusterWorker::new(&bind_addr, Some(worker_id.clone())));
    let worker_clone = worker.clone();

    let worker_handle = tokio::spawn(async move {
        let _ = worker_clone.run_worker().await;
    });

    // Wait for handshake
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Query cluster status
    let status = query_cluster_status(&bind_addr).await.expect("Failed to query status");
    assert_eq!(status.total_workers, 1);
    assert_eq!(status.workers[0].worker_id, "test-node-alpha");

    // Dispatch a test task batch
    let tasks = vec![ToolExecutionTask {
        task_id: "calc-task-1".to_string(),
        tool_name: "calculator".to_string(),
        parameters: serde_json::json!({ "expression": "42 * 2" }),
        timeout_ms: 5000,
        working_dir: None,
    }];

    let batch_res = coordinator.dispatch_batch(tasks).await.expect("Batch dispatch failed");
    assert!(batch_res.batch_id.starts_with("batch-"));

    coord_handle.abort();
    worker_handle.abort();
}

#[test]
fn test_semantic_invariant_guard_contract() {
    let guard = SemanticInvariantGuard::new();

    // 1. Valid tool definition passes
    let valid_tool = ToolDefinition::new(
        "run_command",
        "Executes a bash command safely in the user workspace.",
        serde_json::json!({
            "type": "object",
            "properties": {
                "CommandLine": { "type": "string", "description": "The command line string" }
            },
            "required": ["CommandLine"]
        }),
    );
    assert!(guard.enforce_tool_schema_integrity(&valid_tool).is_ok());

    // 2. Truncated description is rejected
    let truncated_tool = ToolDefinition::new(
        "run_command",
        "Executes a bash command...",
        serde_json::json!({
            "type": "object",
            "properties": { "cmd": { "type": "string" } }
        }),
    );
    let err = guard.enforce_tool_schema_integrity(&truncated_tool).unwrap_err();
    match err {
        SemanticViolation::TruncatedToolSchema { .. } => {}
        _ => panic!("Expected TruncatedToolSchema violation"),
    }

    // 3. Dropped required parameter is rejected
    let missing_param_tool = ToolDefinition::new(
        "read_file",
        "Reads file contents.",
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": ["path"]
        }),
    );
    let err = guard.enforce_tool_schema_integrity(&missing_param_tool).unwrap_err();
    match err {
        SemanticViolation::DroppedRequiredParameter { param_name, .. } => {
            assert_eq!(param_name, "path");
        }
        _ => panic!("Expected DroppedRequiredParameter violation"),
    }

    // 4. Compiler diagnostic preservation
    let rustc_error = r#"
error[E0382]: use of moved value: `data`
  --> src/main.rs:42:15
   |
40 |     let data = vec![1, 2, 3];
   |         ---- move occurs because `data` has type `Vec<i32>`
41 |     drop(data);
   |          ---- value moved here
42 |     println!("{:?}", data);
   |                      ^^^^ value used here after move
   |
   = note: this error indicates memory was dropped
    "#;

    let preserved = guard.enforce_diagnostic_preservation(rustc_error, 100).expect("Preservation failed");
    assert_eq!(preserved.error_code.as_deref(), Some("E0382"));
    assert!(preserved.primary_span.contains("src/main.rs:42:15"));
    assert!(preserved.preserved_condensed.contains("DIAGNOSTIC: error[E0382]"));

    // 5. Rejection when budget cannot accommodate minimal semantics
    let tight_budget_err = guard.enforce_diagnostic_preservation(rustc_error, 2).unwrap_err();
    match tight_budget_err {
        SemanticViolation::DiagnosticDegradation { error_code, .. } => {
            assert_eq!(error_code, "E0382");
        }
        _ => panic!("Expected DiagnosticDegradation violation"),
    }
}
