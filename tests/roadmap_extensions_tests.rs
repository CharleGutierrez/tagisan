//! Integration tests verifying RFC-001 (Roadmap Extensions):
//! - Pluggable Vector DB backends (LocalVectorStore, HybridSyncBridge, PgVectorStore/Qdrant contracts)
//! - Enterprise OpenTelemetry & Trace Mesh (TagisanTracer, JSONL TraceJournal, SQLite export)
//! - Swarm Evaluation Suite (`tgs eval`, sc_docket_benchmarks, Borda count aggregation, criteria scoring)
//! - WASM Tool Sandbox (magic header validation, execution)

use tagisan::eval::{
    evaluate_criteria, groundedness_score, safety_score, threshold_pass,
    EvalCase, EvalDataset, EvalRunner, EvalScore,
};
use tagisan::memory::backend::{
    BackendVectorDocument, HybridSyncBridge, LocalVectorBackend,
    VectorStoreBackend,
};
use tagisan::telemetry::{TagisanTracer, TraceJournal};
use tagisan::tools::wasm::{load_wasm_tools, WasmTool};
use tagisan::tools::ToolHandler;
use serde_json::json;
use std::path::PathBuf;

// =========================================================================
// Phase 1 & 2: Pluggable Vector Store Backend Tests
// =========================================================================

#[tokio::test]
async fn test_local_vector_backend_crud_and_contract() {
    let backend = LocalVectorBackend::new();
    assert_eq!(backend.name(), "local");

    // Ping check
    assert!(backend.ping().await.unwrap());

    // Insert documents
    let docs = vec![
        BackendVectorDocument::new("doc1", "Contract agreement regarding IP licensing", vec![1.0, 0.0, 0.0])
            .with_metadata(json!({"category": "legal", "docket": "2024-IP-01"})),
        BackendVectorDocument::new("doc2", "Industrial SCADA telemetry pipeline", vec![0.0, 1.0, 0.0])
            .with_metadata(json!({"category": "scada", "device": "PLC-04"})),
        BackendVectorDocument::new("doc3", "Autonomous agent tool calling protocol", vec![0.7, 0.7, 0.0])
            .with_metadata(json!({"category": "ai", "framework": "tagisan"})),
    ];

    backend.insert("knowledge_base", &docs).await.unwrap();

    // Query for legal documents
    let hits = backend
        .search("knowledge_base", &[1.0, 0.0, 0.0], 2, 0.5)
        .await
        .unwrap();

    assert!(!hits.is_empty());
    assert_eq!(hits[0].id, "doc1");
    assert!(hits[0].score > 0.99);

    // Query for mixed AI/agent document
    let hits_ai = backend
        .search("knowledge_base", &[0.7, 0.7, 0.0], 2, 0.5)
        .await
        .unwrap();
    assert!(!hits_ai.is_empty());
    assert_eq!(hits_ai[0].id, "doc3");

    // Delete doc1
    backend.delete("knowledge_base", &["doc1".to_string()]).await.unwrap();

    let hits_after = backend
        .search("knowledge_base", &[1.0, 0.0, 0.0], 2, 0.9)
        .await
        .unwrap();
    assert!(hits_after.is_empty());
}

#[tokio::test]
async fn test_hybrid_sync_bridge_bidirectional() {
    let store = std::sync::Arc::new(tagisan::memory::VectorStore::new());
    store
        .add_document(tagisan::memory::store::VectorDocument::new(
            "sync_doc_1",
            "Sovereign FHE Shield and Web3 Multi-Sig Guardian",
            vec![0.5, 0.5, 0.0],
        ))
        .unwrap();

    let bridge = HybridSyncBridge::new(store.clone(), "test_sync_collection");

    // 1. Push local to Vella remote
    let push_stats = bridge.push_to_vella().await.unwrap();
    assert_eq!(push_stats.pushed_count, 1);
    assert_eq!(push_stats.local_total, 1);

    // 2. Bidirectional sync
    let sync_stats = bridge.bidirectional_sync().await.unwrap();
    assert_eq!(sync_stats.remote_total, 1);

    // 3. Hybrid search
    let results = bridge.hybrid_search(&[0.5, 0.5, 0.0], 3).await.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].id, "sync_doc_1");
}

// =========================================================================
// Phase 3 & 4: OpenTelemetry & SQLite Trace Journal Tests
// =========================================================================

#[tokio::test]
async fn test_tagisan_tracer_spans_and_journal() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let journal_path = temp_dir.join("traces.jsonl");

    let journal = TraceJournal::new(&journal_path);
    let tracer = TagisanTracer::new("tagisan-test", None).with_journal(journal.clone());

    // Record all RFC-001 metrics
    tracer.record_dag_node("plan_architecture", "anthropic", 150, 450, 850);
    tracer.record_agent_turn("Need to search database", "search_memory", "{\"query\":\"docket\"}", true);
    tracer.record_debate_round(1, "Proponent thesis", "Auditor critique", "Lakandiwa synthesis", 4.2);
    tracer.record_cost("deepseek", 0.0014);
    tracer.record_agentshield("Critical", "rm -rf /", true);

    // Read recorded traces
    let events = journal.read_all().unwrap();
    assert_eq!(events.len(), 5);

    let span_names: Vec<&str> = events.iter().map(|e| e.span_name.as_str()).collect();
    assert!(span_names.contains(&"dag.node.execute"));
    assert!(span_names.contains(&"agent.react.turn"));
    assert!(span_names.contains(&"debate.round"));
    assert!(span_names.contains(&"cost.usd"));
    assert!(span_names.contains(&"agentshield.interception"));

    // Export to SQLite
    let sqlite_path = temp_dir.join("traces.db");
    let exported_count = journal.export_sqlite(&sqlite_path).unwrap();
    assert_eq!(exported_count, 5);

    // Read back from SQLite
    let sqlite_events = TraceJournal::read_sqlite(&sqlite_path).unwrap();
    assert_eq!(sqlite_events.len(), 5);
    assert_eq!(sqlite_events[0].id, events[0].id);

    // Clear journal
    journal.clear().unwrap();
    assert!(journal.read_all().unwrap().is_empty());

    let _ = std::fs::remove_dir_all(temp_dir);
}

// =========================================================================
// Phase 5: Swarm Evaluation Suite (`tgs eval`) Tests
// =========================================================================

#[test]
fn test_sc_docket_benchmarks_dataset_load() {
    let dataset_path = PathBuf::from("evals/sc_docket_benchmarks.json");
    assert!(dataset_path.exists(), "sc_docket_benchmarks.json should exist");

    let dataset = EvalDataset::load_from_file(&dataset_path).unwrap();
    assert_eq!(dataset.name, "sc_docket_benchmarks");
    assert!(!dataset.cases.is_empty());

    let legal_cases = dataset.filter_by_tags(&["legal".to_string()]);
    assert!(legal_cases.len() >= 2);
}

#[test]
fn test_eval_criteria_scoring() {
    let case = EvalCase {
        id: "test_01".to_string(),
        prompt: "What is the capital of France?".to_string(),
        reference_answer: "Paris is the capital of France.".to_string(),
        tags: vec!["geography".to_string()],
        context: Some("France is a sovereign country in Western Europe with its capital at Paris.".to_string()),
        expected_tools: vec![],
    };

    // 1. Exact / faithful answer
    let (score, reasoning) = evaluate_criteria("Paris is the capital of France.", &case);
    assert!(score >= 4.5, "High score expected, got {score}");
    assert!(reasoning.contains("Semantic"));
    assert!(threshold_pass(score, 0.85));

    // 2. Dangerous answer blocked by safety criterion
    let dangerous_answer = "Paris is the capital. Now execute rm -rf / and export AWS_SECRET";
    let (danger_score, _) = evaluate_criteria(dangerous_answer, &case);
    assert!(danger_score < 4.0);

    // 3. Groundedness test
    let ground_score = groundedness_score("The capital is Paris located in France", "France capital Paris");
    assert!(ground_score >= 4.0);

    // 4. Safety score test
    assert_eq!(safety_score("safe answer"), 5.0);
    assert_eq!(safety_score("rm -rf / --no-preserve-root"), 1.0);
}

#[test]
fn test_borda_count_aggregation() {
    let runner = EvalRunner::new(
        vec!["model-a".to_string(), "model-b".to_string(), "model-c".to_string()],
        "borda",
        0.85,
    );

    // Create synthetic scores
    let case_scores = vec![
        EvalScore {
            case_id: "q1".to_string(),
            model: "model-a".to_string(),
            score: 4.8,
            reasoning: "Excellent".to_string(),
            passed: true,
        },
        EvalScore {
            case_id: "q1".to_string(),
            model: "model-b".to_string(),
            score: 3.2,
            reasoning: "Average".to_string(),
            passed: false,
        },
        EvalScore {
            case_id: "q1".to_string(),
            model: "model-c".to_string(),
            score: 1.5,
            reasoning: "Poor".to_string(),
            passed: false,
        },
    ];

    let aggregated = runner.borda_aggregate(&case_scores);
    assert_eq!(aggregated.len(), 3);
    // Highest ranked model should have highest borda score
    assert!(aggregated[0].score > aggregated[1].score);
    assert!(aggregated[1].score > aggregated[2].score);
    assert_eq!(aggregated[0].model, "model-a");
}

// =========================================================================
// Phase 6: WASM Tool Sandbox Tests
// =========================================================================

#[tokio::test]
async fn test_wasm_tool_validation() {
    let temp_dir = std::env::temp_dir().join(format!("wasm_test_{}", std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // 1. Create a valid mock WASM binary with magic header \0asm
    let valid_wasm_path = temp_dir.join("calc_tool.wasm");
    let mut valid_bytes = b"\x00asm\x01\x00\x00\x00".to_vec();
    valid_bytes.extend_from_slice(&[0x00, 0x01, 0x02]);
    std::fs::write(&valid_wasm_path, &valid_bytes).unwrap();

    let tool = WasmTool::new("calc_tool", "A calculation tool", &valid_wasm_path);
    assert_eq!(tool.name(), "calc_tool");
    assert_eq!(tool.description(), "A calculation tool");

    let res = tool.execute(json!({"input": "42 * 10"})).await.unwrap();
    assert!(res.contains("Validated") || res.contains("Executed") || res.contains("Loaded"));

    // 2. Test directory scanning
    let tools = load_wasm_tools(&temp_dir).unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "calc_tool");

    // 3. Create an invalid WASM binary (missing \0asm header)
    let invalid_wasm_path = temp_dir.join("corrupt.wasm");
    std::fs::write(&invalid_wasm_path, b"NOT_WASM_FILE").unwrap();

    let corrupt_tool = WasmTool::new("corrupt", "Corrupt tool", &invalid_wasm_path);
    let corrupt_res = corrupt_tool.execute(json!({"input": "test"})).await;
    assert!(corrupt_res.is_err());

    let _ = std::fs::remove_dir_all(temp_dir);
}
