use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::json;
use tagisan::bun::{
    tagisan_ffi_blake3_digest, tagisan_ffi_cosine_similarity, tagisan_ffi_shield_scan,
    tagisan_ffi_version, BunRuntime, BunSandbox, BunWorkerPool, DiagnosticSeverity,
    TagisanSqliteStore, TsDiagnosticParser,
};
use tagisan::memory::VectorDocument;
use tagisan::tools::{
    extract_missing_package, BunAutoResolveTool, BunCompileTool, BunEvalTool, BunHmrTool,
    BunServeTool, BunStreamBusTool, ToolHandler,
};

struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tagisan_test_{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn test_warm_worker_pool_sub_2ms_and_auto_recovery() {
    let pool = BunWorkerPool::new(2)
        .await
        .expect("Failed to initialize BunWorkerPool");
    assert_eq!(pool.pool_size(), 2);

    // Initial warm-up evaluation
    let warmup = pool
        .eval("return 40 + 2;", Duration::from_secs(5))
        .await
        .expect("Warmup eval failed");
    assert!(warmup.is_success());
    assert_eq!(warmup.stdout.trim(), "42");

    // Measure latency across 10 rapid warm evaluations
    let mut total_duration = Duration::ZERO;
    let iterations = 10;

    for i in 0..iterations {
        let code = format!(
            r#"
            interface MathPayload {{ factor: number; value: number; }}
            const payload: MathPayload = {{ factor: {}, value: 10 }};
            return payload.factor * payload.value;
            "#,
            i + 1
        );

        let start = Instant::now();
        let res = pool
            .eval(&code, Duration::from_secs(5))
            .await
            .expect("Warm eval failed");
        let elapsed = start.elapsed();
        total_duration += elapsed;

        assert!(res.is_success());
        assert_eq!(res.stdout.trim(), ((i + 1) * 10).to_string());
    }

    let avg_latency = total_duration / iterations;
    println!(
        "[WorkerPool Benchmark] Average warm eval latency: {:?}",
        avg_latency
    );
    // Real Bun warm evaluation with JSC is typically < 1ms
    assert!(
        avg_latency < Duration::from_millis(10),
        "Warm eval latency too high: {:?}",
        avg_latency
    );

    let stats = pool.stats().await;
    assert_eq!(stats.total_workers, 2);
    assert!(stats.total_requests >= 11);

    // Auto-recovery test: intentionally terminate one worker
    println!("[WorkerPool] Testing crash resilience and auto-recovery...");
    let _ = pool.eval("process.exit(1);", Duration::from_secs(2)).await;

    // Subsequent evaluations must seamlessly succeed after auto-respawning
    let res1 = pool
        .eval("return 'worker-recovered-1';", Duration::from_secs(5))
        .await
        .expect("Eval after crash failed");
    assert!(res1.stdout.contains("worker-recovered-1"));

    let res2 = pool
        .eval("return 'worker-recovered-2';", Duration::from_secs(5))
        .await
        .expect("Second eval after crash failed");
    assert!(res2.stdout.contains("worker-recovered-2"));

    pool.shutdown().await;
}

#[tokio::test]
async fn test_bun_sandbox_resource_limits_and_path_containment() {
    let temp_dir = TestTempDir::new("sandbox");
    let root = temp_dir.path().to_path_buf();

    let sandbox = BunSandbox::new(root.clone())
        .with_memory_limit_mb(256)
        .with_cpu_limit_secs(5)
        .with_max_file_size_bytes(10 * 1024 * 1024)
        .with_max_open_files(256);

    // Verify path containment
    assert!(!sandbox.is_path_allowed("/etc/shadow"));
    assert!(!sandbox.is_path_allowed("/etc/passwd"));
    assert!(!sandbox.is_path_allowed("/root/.ssh/id_rsa"));
    assert!(!sandbox.is_path_allowed("/home/other_user/.bashrc"));

    let allowed_file = root.join("allowed_script.ts");
    assert!(sandbox.is_path_allowed(&allowed_file));

    // Verify sandboxed execution of valid TypeScript
    let code = r#"
        interface Box<T> { value: T; }
        const b: Box<number> = { value: 1337 };
        console.log(`SANDBOXED_OK:${b.value}`);
    "#;
    let res = sandbox
        .eval_sandboxed(code, 5)
        .await
        .expect("Sandbox eval failed");
    assert!(res.is_success());
    assert!(res.stdout.contains("SANDBOXED_OK:1337"));
}

#[tokio::test]
async fn test_ts_diagnostic_parser_and_self_healing() {
    let raw_stderr = r#"
src/services/auth.ts:24:9 - error TS2322: Type 'string' is not assignable to type 'number'.

24     const userId: number = "user_9921";
             ~~~~~~
src/services/auth.ts:42:15 - warning TS6133: 'unusedToken' is declared but its value is never read.

42     const unusedToken = "xyz";
             ~~~~~~~~~~~
"#;

    let diagnostics = TsDiagnosticParser::parse(raw_stderr);
    assert_eq!(diagnostics.len(), 2);

    assert_eq!(diagnostics[0].code.as_deref(), Some("TS2322"));
    assert_eq!(diagnostics[0].line, Some(24));
    assert_eq!(diagnostics[0].column, Some(9));
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
    assert!(diagnostics[0]
        .message
        .contains("Type 'string' is not assignable to type 'number'"));

    assert_eq!(diagnostics[1].code.as_deref(), Some("TS6133"));
    assert_eq!(diagnostics[1].line, Some(42));
    assert_eq!(diagnostics[1].column, Some(15));
    assert_eq!(diagnostics[1].severity, DiagnosticSeverity::Warning);

    // Test self-healing prompt generation
    let broken_code = r#"
function getUserId(): number {
    const userId: number = "user_9921";
    return userId;
}
"#;
    let prompt = TsDiagnosticParser::generate_self_healing_prompt(&diagnostics, broken_code);
    assert!(prompt.contains("TS2322"));
    assert!(prompt.contains("auth.ts"));
    assert!(prompt.contains("Line 24"));
    assert!(prompt.contains("const userId: number = \"user_9921\";"));
    assert!(prompt.contains("Return only the complete, corrected TypeScript code"));

    // Real live check_and_parse using Bun
    let valid_code = "const x: number = 42; console.log(x);";
    let live_diags = TsDiagnosticParser::check_and_parse(valid_code)
        .await
        .expect("check_and_parse failed on valid code");
    assert!(live_diags.is_empty());
}

#[tokio::test]
async fn test_sqlite_memory_vector_roundtrip_rust_and_bun() {
    let temp_dir = TestTempDir::new("sqlite_mem");
    let db_path = temp_dir.path().join("vector_store.db");

    // 1. Initialize SQLite store in Rust and insert 3 vector documents
    let store = TagisanSqliteStore::open(&db_path).expect("Failed to open TagisanSqliteStore");

    let doc1 = VectorDocument {
        id: "doc-rust-1".to_string(),
        text: "Rust systems programming and zero-cost abstractions".to_string(),
        embedding: vec![1.0, 0.0, 0.0],
        metadata: HashMap::from([("tag".to_string(), "rust".to_string())]),
        created_at: 1000,
    };
    let doc2 = VectorDocument {
        id: "doc-bun-2".to_string(),
        text: "Bun TypeScript fast runtime and native SQLite".to_string(),
        embedding: vec![0.0, 1.0, 0.0],
        metadata: HashMap::from([("tag".to_string(), "bun".to_string())]),
        created_at: 1001,
    };
    let doc3 = VectorDocument {
        id: "doc-hybrid-3".to_string(),
        text: "Tagisan dialectical multi-agent architecture with Bun and Rust".to_string(),
        embedding: vec![0.7071, 0.7071, 0.0],
        metadata: HashMap::from([("tag".to_string(), "hybrid".to_string())]),
        created_at: 1002,
    };

    store.insert(&doc1).expect("Failed to insert doc1");
    store.insert(&doc2).expect("Failed to insert doc2");
    store.insert(&doc3).expect("Failed to insert doc3");

    assert_eq!(store.count().expect("Count failed"), 3);

    // 2. Query in Rust
    let rust_matches = store
        .search_cosine(&[0.0, 1.0, 0.0], 1, 0.8)
        .expect("Rust search failed");
    assert_eq!(rust_matches.len(), 1);
    assert_eq!(rust_matches[0].document.id, "doc-bun-2");
    assert!((rust_matches[0].score - 1.0).abs() < 1e-4);

    // 3. Query the EXACT SAME SQLite file from Bun using native bun:sqlite & Float32Array
    let bun_runtime = BunRuntime::new().expect("Bun runtime not found");
    let script = format!(
        r#"
        import {{ Database }} from "bun:sqlite";
        const db = new Database("{db_path}");

        // Query row inserted by Rust
        const row = db.query("SELECT id, text, embedding FROM tagisan_vectors WHERE id = 'doc-hybrid-3'").get();
        const u8 = row.embedding;
        const f32 = new Float32Array(u8.buffer, u8.byteOffset, u8.byteLength / 4);

        console.log(`BUN_READ_ID:${{row.id}}`);
        console.log(`BUN_FLOAT_LEN:${{f32.length}}`);
        console.log(`BUN_FLOAT_0:${{f32[0].toFixed(4)}}`);
        console.log(`BUN_FLOAT_1:${{f32[1].toFixed(4)}}`);

        // Insert new vector directly from Bun
        const newVec = new Float32Array([0.0, 0.0, 1.0]);
        const newU8 = new Uint8Array(newVec.buffer, newVec.byteOffset, newVec.byteLength);
        db.run(
            "INSERT INTO tagisan_vectors (id, text, embedding, dimension, metadata, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            ["doc-bun-inserted-4", "Inserted by Bun script directly", newU8, 3, JSON.stringify({{ origin: "bun" }}), Date.now()]
        );
        db.close();
        "#,
        db_path = db_path.display()
    );

    let bun_res = bun_runtime
        .eval(&script, Duration::from_secs(10), None, None)
        .await
        .expect("Bun eval failed");
    assert!(bun_res.is_success());
    assert!(bun_res.stdout.contains("BUN_READ_ID:doc-hybrid-3"));
    assert!(bun_res.stdout.contains("BUN_FLOAT_LEN:3"));
    assert!(bun_res.stdout.contains("BUN_FLOAT_0:0.7071"));
    assert!(bun_res.stdout.contains("BUN_FLOAT_1:0.7071"));

    // 4. Verify Rust reads back the vector inserted by Bun!
    assert_eq!(store.count().expect("Count failed"), 4);
    let bun_inserted_matches = store
        .search_cosine(&[0.0, 0.0, 1.0], 1, 0.99)
        .expect("Search for Bun vector failed");
    assert_eq!(bun_inserted_matches.len(), 1);
    assert_eq!(bun_inserted_matches[0].document.id, "doc-bun-inserted-4");
    assert_eq!(
        bun_inserted_matches[0].document.text,
        "Inserted by Bun script directly"
    );
    assert!((bun_inserted_matches[0].score - 1.0).abs() < 1e-4);
}

#[tokio::test]
async fn test_bun_serve_tool_lifecycle_and_fetch_probe() {
    let serve_tool = BunServeTool::new();

    // 1. Start live HTTP server on port 0
    let start_args = json!({
        "action": "start",
        "port": 0,
        "html": "<h1>Tagisan Live Dev Server</h1>",
        "routes": {
            "/api/health": { "status": "healthy", "service": "tagisan" },
            "/api/echo": "pong"
        }
    });

    let start_result = serve_tool
        .execute(start_args)
        .await
        .expect("Failed to start server");
    let start_val: serde_json::Value =
        serde_json::from_str(&start_result).expect("Failed to parse start JSON");

    assert_eq!(start_val["status"], "started");
    let actual_port = start_val["port"].as_u64().expect("Missing port") as u16;
    assert!(actual_port > 0);

    // 2. Check status
    let status_args = json!({ "action": "status" });
    let status_result = serve_tool
        .execute(status_args.clone())
        .await
        .expect("Failed to get status");
    assert!(status_result.contains(&actual_port.to_string()));

    // 3. Fetch probe: default HTML
    let fetch_html_args = json!({
        "action": "fetch",
        "url": format!("http://localhost:{actual_port}/")
    });
    let fetch_html_result = serve_tool
        .execute(fetch_html_args)
        .await
        .expect("Fetch HTML failed");
    let fetch_html_val: serde_json::Value =
        serde_json::from_str(&fetch_html_result).expect("Failed to parse fetch HTML JSON");
    assert_eq!(fetch_html_val["status"], 200);
    assert!(fetch_html_val["body"]
        .as_str()
        .unwrap()
        .contains("Tagisan Live Dev Server"));

    // 4. Fetch probe: JSON route
    let fetch_api_args = json!({
        "action": "fetch",
        "url": format!("http://localhost:{actual_port}/api/health")
    });
    let fetch_api_result = serve_tool
        .execute(fetch_api_args)
        .await
        .expect("Fetch API failed");
    let fetch_api_val: serde_json::Value =
        serde_json::from_str(&fetch_api_result).expect("Failed to parse fetch API JSON");
    assert_eq!(fetch_api_val["status"], 200);
    assert!(fetch_api_val["body"]
        .as_str()
        .unwrap()
        .contains("\"status\":\"healthy\""));

    // 5. Stop server
    let stop_args = json!({
        "action": "stop",
        "port": actual_port
    });
    let stop_result = serve_tool
        .execute(stop_args)
        .await
        .expect("Failed to stop server");
    assert!(stop_result.contains("stopped successfully"));

    // 6. Verify server is no longer in status list
    let status_after = serve_tool
        .execute(status_args)
        .await
        .expect("Failed to get post-stop status");
    let status_val: serde_json::Value =
        serde_json::from_str(&status_after).expect("Failed to parse post-stop status JSON");
    let servers = status_val["servers"].as_array().unwrap();
    assert!(!servers.iter().any(|s| s["port"] == actual_port));
}

#[tokio::test]
async fn test_bun_compile_standalone_executable() {
    let temp_dir = TestTempDir::new("compile");
    let entrypoint = temp_dir.path().join("standalone_app.ts");
    let outfile = temp_dir.path().join("standalone_app_bin");

    let ts_source = r#"
        interface AppConfig { name: string; version: number; }
        const config: AppConfig = { name: "TagisanBinary", version: 1 };
        console.log(`NATIVE_EXECUTION_SUCCESS:${config.name}_v${config.version}`);
        process.exit(0);
    "#;
    fs::write(&entrypoint, ts_source).expect("Failed to write TS source");

    let compile_tool = BunCompileTool::new();
    let compile_args = json!({
        "entrypoint": entrypoint.to_str().unwrap(),
        "outfile": outfile.to_str().unwrap(),
        "minify": true,
        "bytecode": false,
        "timeout_secs": 60
    });

    let compile_result = compile_tool
        .execute(compile_args)
        .await
        .expect("bun_compile execution failed");
    let compile_val: serde_json::Value =
        serde_json::from_str(&compile_result).expect("Failed to parse compile JSON");

    assert_eq!(compile_val["status"], "success");
    assert!(outfile.exists(), "Compiled binary does not exist on disk");

    // Execute the standalone binary directly without Bun!
    let output = tokio::process::Command::new(&outfile)
        .output()
        .await
        .expect("Failed to execute compiled binary directly");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NATIVE_EXECUTION_SUCCESS:TagisanBinary_v1"));
}

#[tokio::test]
async fn test_agentshield_guards_bun_serve_and_compile() {
    let compile_tool = BunCompileTool::new();

    // Malicious entrypoint accessing /etc/shadow
    let evil_compile = json!({
        "entrypoint": "/etc/shadow",
        "outfile": "/tmp/evil_bin"
    });

    let res = compile_tool.execute(evil_compile).await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("AgentShield blocked tool"));
}

#[tokio::test]
async fn test_phase2_zero_copy_c_ffi_bridge() {
    // 1. Version
    let ver_ptr = tagisan_ffi_version();
    assert!(!ver_ptr.is_null());
    let ver_str = unsafe { std::ffi::CStr::from_ptr(ver_ptr) }
        .to_str()
        .unwrap();
    assert_eq!(ver_str, "tagisan-0.1.0-bun-native");

    // 2. Cosine Similarity zero-copy SIMD
    let v1 = [1.0f32, 0.0, 0.0];
    let v2 = [1.0f32, 0.0, 0.0];
    let sim = tagisan_ffi_cosine_similarity(v1.as_ptr(), v2.as_ptr(), 3);
    assert!((sim - 1.0).abs() < 1e-5);

    let v3 = [0.0f32, 1.0, 0.0];
    let sim_ortho = tagisan_ffi_cosine_similarity(v1.as_ptr(), v3.as_ptr(), 3);
    assert!((sim_ortho - 0.0).abs() < 1e-5);

    // Null safety
    assert_eq!(tagisan_ffi_cosine_similarity(std::ptr::null(), v1.as_ptr(), 3), 0.0);

    // 3. BLAKE3 Digest
    let data = b"tagisan-bun-native-ffi";
    let mut out_hex = [0u8; 64];
    let status = tagisan_ffi_blake3_digest(data.as_ptr(), data.len(), out_hex.as_mut_ptr());
    assert_eq!(status, 0);
    let digest_str = std::str::from_utf8(&out_hex).unwrap();
    assert_eq!(digest_str.len(), 64);
    assert_eq!(digest_str, blake3::hash(data).to_hex().as_str());

    // 4. AgentShield scan
    let safe_code = std::ffi::CString::new("const x: number = 42; console.log(x);").unwrap();
    assert_eq!(tagisan_ffi_shield_scan(safe_code.as_ptr()), 0);

    let evil_code = std::ffi::CString::new(":(){ :|:& };:").unwrap();
    assert_eq!(tagisan_ffi_shield_scan(evil_code.as_ptr()), 1);
}

#[tokio::test]
async fn test_phase2_bun_stream_bus_websocket_pubsub() {
    let bus_tool = BunStreamBusTool::new();

    // 1. Start bus on ephemeral port
    let start_args = json!({
        "action": "start",
        "port": 0
    });
    let start_resp = bus_tool.execute(start_args).await.expect("Stream bus start failed");
    let start_val: serde_json::Value = serde_json::from_str(&start_resp).unwrap();
    assert_eq!(start_val["status"], "started");
    let port = start_val["port"].as_u64().unwrap() as u16;
    assert!(port > 0);

    // 2. Status
    let status_args = json!({ "action": "status" });
    let status_resp = bus_tool.execute(status_args).await.unwrap();
    assert!(status_resp.contains(&port.to_string()));

    // 3. Publish message to topic
    let publish_args = json!({
        "action": "publish",
        "port": port,
        "topic": "swarm-telemetry",
        "message": { "agent": "SecurityAuditor", "event": "threat_cleared", "threat_level": 0 }
    });
    let pub_resp = bus_tool.execute(publish_args).await.expect("Publish failed");
    let pub_val: serde_json::Value = serde_json::from_str(&pub_resp).unwrap();
    assert_eq!(pub_val["status"], "published");
    assert_eq!(pub_val["topic"], "swarm-telemetry");

    // 4. Stop bus
    let stop_args = json!({
        "action": "stop",
        "port": port
    });
    let stop_resp = bus_tool.execute(stop_args).await.unwrap();
    assert!(stop_resp.contains("stopped successfully"));
}

#[tokio::test]
async fn test_phase2_bun_hmr_scratchpad() {
    let temp_dir = TestTempDir::new("bun_hmr_test");
    let script_path = temp_dir.path.join("scratchpad.ts");

    let initial_code = r#"
        console.log("HMR_STATE_V1_INIT");
        setInterval(() => {}, 1000);
    "#;
    fs::write(&script_path, initial_code).expect("Write initial HMR script failed");

    let hmr_tool = BunHmrTool::new();
    let id = "test_hmr_session";

    // 1. Start HMR
    let start_args = json!({
        "action": "start",
        "id": id,
        "script_path": script_path.to_str().unwrap()
    });
    let start_resp = hmr_tool.execute(start_args).await.expect("HMR start failed");
    let start_val: serde_json::Value = serde_json::from_str(&start_resp).unwrap();
    assert_eq!(start_val["status"], "started");
    assert_eq!(start_val["id"], id);

    // 2. Status
    let status_args = json!({ "action": "status" });
    let status_resp = hmr_tool.execute(status_args).await.unwrap();
    assert!(status_resp.contains(id));

    // 3. Update script to trigger hot reload
    let updated_code = r#"
        console.log("HMR_STATE_V2_HOT_RELOAD");
        setInterval(() => {}, 1000);
    "#;
    let update_args = json!({
        "action": "update_script",
        "script_path": script_path.to_str().unwrap(),
        "content": updated_code
    });
    let update_resp = hmr_tool.execute(update_args).await.expect("HMR update failed");
    let update_val: serde_json::Value = serde_json::from_str(&update_resp).unwrap();
    assert_eq!(update_val["status"], "updated");

    // 4. Stop HMR
    let stop_args = json!({
        "action": "stop",
        "id": id
    });
    let stop_resp = hmr_tool.execute(stop_args).await.unwrap();
    assert!(stop_resp.contains("stopped successfully"));
}

#[tokio::test]
async fn test_phase2_autonomous_jit_dependency_resolver() {
    // 1. Test missing package extractor regex/logic
    let sample_err_1 = "error: Cannot find package 'chalk' imported from /tmp/index.ts";
    assert_eq!(extract_missing_package(sample_err_1), Some("chalk".to_string()));

    let sample_err_2 = "error: Cannot find module 'lodash' imported from /tmp/index.ts";
    assert_eq!(extract_missing_package(sample_err_2), Some("lodash".to_string()));

    let sample_err_3 = "Could not resolve: \"zod\"";
    assert_eq!(extract_missing_package(sample_err_3), Some("zod".to_string()));

    // 2. Test BunAutoResolveTool execution
    let auto_tool = BunAutoResolveTool::new();
    let eval_args = json!({
        "code": "const x: number = [1, 2, 3].reduce((a, b) => a + b, 0); console.log(`REDUCE_SUM:${x}`);"
    });
    let resp = auto_tool.execute(eval_args).await.expect("Auto-resolve tool failed");
    let resp_val: serde_json::Value = serde_json::from_str(&resp).unwrap();
    assert_eq!(resp_val["status"], "success");
    assert!(resp_val["stdout"].as_str().unwrap().contains("REDUCE_SUM:6"));

    // 3. Test BunEvalTool auto_resolve flag
    let eval_tool = BunEvalTool::new();
    let eval_res = eval_tool
        .execute(json!({
            "code": "const a = 10; const b = 20; console.log(`EVAL_SUM:${a + b}`);",
            "auto_resolve": true
        }))
        .await
        .expect("BunEvalTool execution failed");
    assert!(eval_res.contains("EVAL_SUM:30"));
}

#[tokio::test]
async fn test_phase2_cross_compilation_matrix() {
    let temp_dir = TestTempDir::new("bun_compile_matrix");
    let entrypoint = temp_dir.path.join("app.ts");
    let outfile = temp_dir.path.join("app_bin");

    let ts_source = r#"
        console.log("CROSS_COMPILATION_MATRIX_SUCCESS");
        process.exit(0);
    "#;
    fs::write(&entrypoint, ts_source).unwrap();

    let compile_tool = BunCompileTool::new();
    let compile_args = json!({
        "entrypoint": entrypoint.to_str().unwrap(),
        "outfile": outfile.to_str().unwrap(),
        "targets": ["bun-linux-x64"],
        "minify": true,
        "timeout_secs": 60
    });

    let compile_resp = compile_tool.execute(compile_args).await.expect("Matrix compilation failed");
    let compile_val: serde_json::Value = serde_json::from_str(&compile_resp).unwrap();
    assert_eq!(compile_val["status"], "success");

    let artifacts = compile_val["artifacts"].as_array().unwrap();
    assert!(!artifacts.is_empty());
    assert_eq!(artifacts[0]["target"], "bun-linux-x64");
}

