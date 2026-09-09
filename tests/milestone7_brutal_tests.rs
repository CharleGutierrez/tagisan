use serde_json::json;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tagisan::agent::WorktreeSandbox;
use tagisan::mcp::protocol::{JsonRpcRequest, JsonRpcResponse, RequestId};
use tagisan::mcp::McpServer;
use tagisan::tools::ToolRegistry;

// =========================================================================
// 1. JSON-RPC 2.0 PROTOCOL VERIFICATION: String, Negative, and Polymorphic IDs
// =========================================================================

#[test]
fn test_jsonrpc_request_supports_string_and_negative_ids() {
    let string_id_req = r#"{
        "jsonrpc": "2.0",
        "id": "req-xyz-42",
        "method": "ping",
        "params": {}
    }"#;

    let req: JsonRpcRequest = serde_json::from_str(string_id_req).expect("String ID must parse cleanly");
    assert_eq!(req.id, "req-xyz-42");
    assert_eq!(req.method, "ping");

    let negative_id_req = r#"{
        "jsonrpc": "2.0",
        "id": -1,
        "method": "ping"
    }"#;
    let neg_req: JsonRpcRequest = serde_json::from_str(negative_id_req).expect("Negative ID must parse");
    assert_eq!(neg_req.id, -1i64);

    let u64_id_req = r#"{
        "jsonrpc": "2.0",
        "id": 100,
        "method": "ping"
    }"#;
    let u64_req: JsonRpcRequest = serde_json::from_str(u64_id_req).expect("Numeric ID must parse");
    assert_eq!(u64_req.id, 100u64);
}

#[test]
fn test_jsonrpc_response_echoes_string_ids() {
    let resp = JsonRpcResponse::success("client-req-uuid-999", json!({ "status": "ok" }));
    let serialized = serde_json::to_string(&resp).expect("Serialize response");

    assert!(serialized.contains(r#""id":"client-req-uuid-999""#));

    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).expect("Deserialize response");
    assert_eq!(deserialized.id, Some(RequestId::String("client-req-uuid-999".to_string())));
}

#[tokio::test]
async fn test_run_stdio_preserves_and_echoes_string_ids() {
    let server = McpServer::default_server();

    let client_input = b"{\"jsonrpc\": \"2.0\", \"id\": \"cursor-init-uuid-777\", \"method\": \"ping\"}\n";
    let mut reader = &client_input[..];
    let mut writer = Vec::new();

    let res = server.run_stdio(&mut reader, &mut writer).await;
    assert!(res.is_ok());

    let output_str = String::from_utf8_lossy(&writer);
    assert!(
        !output_str.is_empty(),
        "Server MUST NOT drop requests with string IDs into a blackhole!"
    );
    assert!(
        output_str.contains(r#""id":"cursor-init-uuid-777""#),
        "Response must echo client's string ID: {output_str}"
    );
}

// =========================================================================
// 2. NOTIFICATIONS HANDLING IN STDIO: SILENT & ACCURATE
// =========================================================================

#[tokio::test]
async fn test_run_stdio_notification_produces_zero_output() {
    let server = McpServer::default_server();

    let client_input = b"{\"jsonrpc\": \"2.0\", \"method\": \"notifications/initialized\"}\n";
    let mut reader = &client_input[..];
    let mut writer = Vec::new();

    let res = server.run_stdio(&mut reader, &mut writer).await;
    assert!(res.is_ok());
    assert!(
        writer.is_empty(),
        "Notifications must never emit a response on stdio"
    );
}

// =========================================================================
// 3. GIT WORKTREE SANDBOXING: DIFF AGAINST BASE COMMIT AFTER COMMIT
// =========================================================================

#[test]
fn test_worktree_sandbox_diff_after_commit() {
    let unique_id = format!(
        "tagisan_brutal_diff_repo_{}_{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
    );
    let temp_repo = std::env::temp_dir().join(unique_id);
    fs::create_dir_all(&temp_repo).expect("Create temp repo dir");

    let init_out = Command::new("git").args(["init", "-b", "main"]).current_dir(&temp_repo).output();
    if init_out.is_err() || !init_out.as_ref().unwrap().status.success() {
        let _ = fs::remove_dir_all(&temp_repo);
        return;
    }

    let _ = Command::new("git").args(["config", "user.email", "brutal_test@tagisan.ai"]).current_dir(&temp_repo).output();
    let _ = Command::new("git").args(["config", "user.name", "Tagisan Auditor"]).current_dir(&temp_repo).output();

    let initial_file = temp_repo.join("base.txt");
    fs::write(&initial_file, "Base version 1.0\n").unwrap();
    let _ = Command::new("git").args(["add", "."]).current_dir(&temp_repo).output();
    let _ = Command::new("git").args(["commit", "-m", "Initial base commit"]).current_dir(&temp_repo).output();

    let branch_name = format!("worktree_brutal_branch_{}", std::process::id());
    let mut sandbox = match WorktreeSandbox::create(&temp_repo, &branch_name) {
        Ok(s) => s,
        Err(e) => {
            let _ = fs::remove_dir_all(&temp_repo);
            panic!("Failed to create WorktreeSandbox: {e}");
        }
    };

    assert!(!sandbox.base_commit().is_empty());

    let new_feature_file = sandbox.path().join("feature.rs");
    fs::write(&new_feature_file, "pub fn agent_feature() -> &'static str { \"isolated\" }\n").unwrap();

    let commit_sha = sandbox.commit_all("feat: implement isolated feature").expect("commit_all");
    assert_eq!(commit_sha.len(), 40);

    // Diffs against base commit must contain the feature even after commit!
    let diff_after = sandbox.diff().expect("diff after commit");
    assert!(
        !diff_after.is_empty(),
        "WorktreeSandbox::diff() must NOT return empty diff after commit!"
    );
    assert!(
        diff_after.contains("pub fn agent_feature"),
        "Diff must contain the committed changes: {diff_after}"
    );

    let _ = sandbox.cleanup();
    let _ = fs::remove_dir_all(&temp_repo);
}

// =========================================================================
// 4. GIT WORKTREE SANDBOXING: TOOLS BINDING TO SANDBOX DIRECTORY
// =========================================================================

#[tokio::test]
async fn test_worktree_sandbox_tools_isolation() {
    let temp_sandbox = std::env::temp_dir().join(format!("tagisan_tool_iso_{}", std::process::id()));
    fs::create_dir_all(&temp_sandbox).unwrap();

    let registry = ToolRegistry::with_builtins_in_dir(&temp_sandbox);

    // Write a file through write_file tool
    let write_tool = registry.get("write_file").expect("write_file missing");
    let res = write_tool
        .execute(json!({ "path": "test_output.txt", "content": "Hello Sandboxed World!" }))
        .await
        .expect("write_file failed");
    assert!(res.contains("test_output.txt"));

    // Verify it actually landed in temp_sandbox
    let target_file = temp_sandbox.join("test_output.txt");
    assert!(target_file.exists(), "File must exist in sandbox directory");

    // Read the file through read_file tool with relative path
    let read_tool = registry.get("read_file").expect("read_file missing");
    let read_res = read_tool
        .execute(json!({ "path": "test_output.txt" }))
        .await
        .expect("read_file failed");
    assert_eq!(read_res, "Hello Sandboxed World!");

    let _ = fs::remove_dir_all(&temp_sandbox);
}

// =========================================================================
// 5. MALFORMED JSON AND INVALID REQUESTS ERROR HANDLING
// =========================================================================

#[tokio::test]
async fn test_run_stdio_malformed_json_emits_32700() {
    let server = McpServer::default_server();
    let input = b"{\"jsonrpc\": \"2.0\", not_valid_json\n";
    let mut reader = input.as_slice();
    let mut writer = Vec::new();

    let _ = server.run_stdio(&mut reader, &mut writer).await;
    let output_str = String::from_utf8_lossy(&writer);
    assert!(output_str.contains("-32700") && output_str.contains("Parse error"));
}

#[tokio::test]
async fn test_run_stdio_invalid_request_echoes_client_id() {
    let server = McpServer::default_server();
    // Invalid request missing method, but has string id
    let input = b"{\"jsonrpc\": \"2.0\", \"id\": \"err-req-55\", \"invalid\": true}\n";
    let mut reader = input.as_slice();
    let mut writer = Vec::new();

    let _ = server.run_stdio(&mut reader, &mut writer).await;
    let output_str = String::from_utf8_lossy(&writer);
    assert!(
        output_str.contains(r#""id":"err-req-55""#),
        "Error response must echo client id: {output_str}"
    );
    assert!(output_str.contains("-32600"));
}

// =========================================================================
// 6. HIGH CONCURRENCY STRESS TESTING
// =========================================================================

#[tokio::test]
async fn test_mcp_server_concurrent_requests_stress() {
    let server = Arc::new(McpServer::default_server());
    let concurrency_count = 50;
    let mut handles = Vec::new();

    for i in 0..concurrency_count {
        let srv = server.clone();
        let req_id = format!("req-thread-{i}");

        let handle = tokio::spawn(async move {
            let method = match i % 3 {
                0 => "initialize",
                1 => "ping",
                _ => "tools/list",
            };
            let params = if method == "initialize" {
                Some(json!({ "protocolVersion": "2024-11-05", "capabilities": {} }))
            } else {
                None
            };
            let req = JsonRpcRequest::new(req_id.clone(), method, params);
            (req_id, srv.handle_request(req).await)
        });
        handles.push(handle);
    }

    for handle in handles {
        let (req_id, resp) = handle.await.expect("Task failed");
        let resp = resp.expect("Response expected");
        assert_eq!(resp.id, Some(RequestId::String(req_id)));
        assert!(resp.error.is_none());
    }
}
