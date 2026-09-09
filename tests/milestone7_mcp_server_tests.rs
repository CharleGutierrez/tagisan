use serde_json::json;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tagisan::agent::WorktreeSandbox;
use tagisan::memory::{EmbeddingProvider, FastHashEmbeddingProvider, VectorDocument, VectorStore};
use tagisan::mcp::protocol::{JsonRpcRequest, McpInitializeResult, McpToolCallResult};
use tagisan::mcp::McpServer;
use tagisan::tools::ToolRegistry;
use tagisan::EngineContext;

// =========================================================================
// 1. MCP Server Protocol Handshake & Notifications
// =========================================================================

#[tokio::test]
async fn test_mcp_server_initialize_handshake() {
    let server = McpServer::default_server();

    let init_req = JsonRpcRequest::new(
        1,
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "claude-desktop",
                "version": "0.1.0"
            }
        })),
    );

    let res = server.handle_request(init_req).await;
    assert!(res.is_some(), "Initialize request must produce a response");
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(1));
    assert!(resp.error.is_none(), "Handshake must not return an error");

    let result: McpInitializeResult =
        serde_json::from_value(resp.result.expect("Expected result object")).expect("Must deserialize into McpInitializeResult");

    assert_eq!(result.protocol_version, "2024-11-05");
    assert!(result.server_info.is_some());
    let server_info = result.server_info.unwrap();
    assert_eq!(server_info.name, "tagisan");
    assert!(server_info.version.is_some());
    assert_eq!(
        result.capabilities.get("tools").and_then(|t| t.get("listChanged")),
        Some(&json!(false))
    );

    // Test notifications/initialized notification (returns None because notifications require no response)
    let notif_req = JsonRpcRequest::new(2, "notifications/initialized", None);
    let notif_res = server.handle_request(notif_req).await;
    assert!(
        notif_res.is_none(),
        "notifications/initialized must not produce a response"
    );
}

// =========================================================================
// 2. Ping Probe
// =========================================================================

#[tokio::test]
async fn test_mcp_server_ping_probe() {
    let server = McpServer::default_server();

    let ping_req = JsonRpcRequest::new(42, "ping", None);
    let res = server.handle_request(ping_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(42));
    assert!(resp.error.is_none());
    assert_eq!(resp.result, Some(json!({})));
}

// =========================================================================
// 3. Tools Catalog (tools/list)
// =========================================================================

#[tokio::test]
async fn test_mcp_server_tools_list_catalog() {
    let server = McpServer::default_server();

    let list_req = JsonRpcRequest::new(10, "tools/list", None);
    let res = server.handle_request(list_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(10));
    assert!(resp.error.is_none());

    let result = resp.result.expect("Must have result");
    let tools = result
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("Result must contain tools array");

    assert!(
        tools.len() >= 10,
        "Server must register 8 core Tagisan tools + built-in tools, found {}",
        tools.len()
    );

    let tool_names: Vec<String> = tools
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .collect();

    // Verify all 8 core Tagisan tools
    let expected_core = [
        "tagisan_debate",
        "tagisan_moa",
        "tagisan_agent",
        "tagisan_workflow_plan",
        "tagisan_ecc_pipeline",
        "tagisan_memory_search",
        "tagisan_memory_index",
        "tagisan_status",
    ];

    for expected in &expected_core {
        assert!(
            tool_names.contains(&expected.to_string()),
            "tools/list missing core tool: {}",
            expected
        );
    }

    // Verify built-in tools
    let expected_builtins = ["calculator", "read_file", "write_file", "run_command", "view_image"];
    for expected in &expected_builtins {
        assert!(
            tool_names.contains(&expected.to_string()),
            "tools/list missing built-in tool: {}",
            expected
        );
    }

    // Check schema correctness on every tool
    for tool in tools {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap();
        assert!(
            tool.get("description").is_some(),
            "Tool '{}' must have a description",
            name
        );
        let schema = tool.get("inputSchema").expect("Must have inputSchema");
        assert_eq!(
            schema.get("type").and_then(|t| t.as_str()),
            Some("object"),
            "Tool '{}' schema type must be 'object'",
            name
        );
    }
}

// =========================================================================
// 4. Executing Status Tool (tagisan_status)
// =========================================================================

#[tokio::test]
async fn test_mcp_server_execute_status_tool() {
    let server = McpServer::default_server();

    let call_req = JsonRpcRequest::new(
        20,
        "tools/call",
        Some(json!({
            "name": "tagisan_status",
            "arguments": {}
        })),
    );

    let res = server.handle_request(call_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(20));
    assert!(resp.error.is_none());

    let call_res: McpToolCallResult =
        serde_json::from_value(resp.result.expect("Result required")).expect("McpToolCallResult");

    assert!(!call_res.is_error);
    let output = call_res.extract_text();
    assert!(
        output.contains("Tagisan Engine Status"),
        "Output must contain header: {output}"
    );
    assert!(
        output.contains("MCP Protocol") && output.contains("2024-11-05"),
        "Output must list protocol: {output}"
    );
    assert!(
        output.contains("Registered Tools:"),
        "Output must report registered tools: {output}"
    );
}

// =========================================================================
// 5. Executing Built-in Calculator Tool via MCP
// =========================================================================

#[tokio::test]
async fn test_mcp_server_execute_builtin_calculator() {
    let server = McpServer::default_server();

    let call_req = JsonRpcRequest::new(
        30,
        "tools/call",
        Some(json!({
            "name": "calculator",
            "arguments": {
                "expression": "10 * 8 + 20"
            }
        })),
    );

    let res = server.handle_request(call_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(30));
    assert!(resp.error.is_none());

    let call_res: McpToolCallResult =
        serde_json::from_value(resp.result.expect("Result required")).expect("McpToolCallResult");

    assert!(!call_res.is_error);
    let text = call_res.extract_text();
    assert!(text.contains("100"), "Calculator output must be 100: {text}");
}

// =========================================================================
// 6. Memory Search Integration via MCP Server
// =========================================================================

#[tokio::test]
async fn test_mcp_server_memory_search_populated() {
    let provider = Arc::new(FastHashEmbeddingProvider::default_256());
    let memory_store = Arc::new(VectorStore::new());

    // Populate a test document into memory store
    let doc_text = "pub fn compute_quicksort<T: Ord>(arr: &mut [T]) { /* quicksort impl */ }";
    let emb = provider.embed_text(doc_text).await.unwrap();

    let mut doc = VectorDocument::new("sort.rs", doc_text, emb);
    doc.metadata.insert("file_path".to_string(), "src/algorithms/sort.rs".to_string());
    doc.metadata.insert("start_line".to_string(), "1".to_string());
    doc.metadata.insert("end_line".to_string(), "10".to_string());
    memory_store.add_document(doc).unwrap();

    let ctx = Arc::new(EngineContext::new(10.0));
    let tools = ToolRegistry::with_builtins();
    let server = McpServer::new(ctx, tools, memory_store, provider);

    let call_req = JsonRpcRequest::new(
        40,
        "tools/call",
        Some(json!({
            "name": "tagisan_memory_search",
            "arguments": {
                "query": "quicksort algorithm implementation",
                "threshold": 0.01,
                "top_k": 3
            }
        })),
    );

    let res = server.handle_request(call_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(40));
    assert!(resp.error.is_none());

    let call_res: McpToolCallResult =
        serde_json::from_value(resp.result.expect("Result required")).expect("McpToolCallResult");

    assert!(!call_res.is_error);
    let output = call_res.extract_text();
    assert!(
        output.contains("sort.rs"),
        "Search output should contain sort.rs: {output}"
    );
    assert!(
        output.contains("compute_quicksort"),
        "Search output should contain snippet: {output}"
    );
}

// =========================================================================
// 7. Unknown Method and Unknown Tool Error Handling
// =========================================================================

#[tokio::test]
async fn test_mcp_server_unknown_method_and_unknown_tool() {
    let server = McpServer::default_server();

    // 1. Unknown Method -> JSON-RPC error -32601
    let unknown_method_req = JsonRpcRequest::new(50, "unsupported/method", None);
    let res = server.handle_request(unknown_method_req).await;
    assert!(res.is_some());
    let resp = res.unwrap();

    assert_eq!(resp.id, Some(50));
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
    let err = resp.error.unwrap();
    assert_eq!(err.code, -32601);
    assert!(err.message.contains("Method not found: 'unsupported/method'"));

    // 2. Unknown Tool -> McpToolCallResult with is_error = true
    let unknown_tool_req = JsonRpcRequest::new(
        51,
        "tools/call",
        Some(json!({
            "name": "non_existent_tool_999",
            "arguments": {}
        })),
    );
    let res2 = server.handle_request(unknown_tool_req).await;
    assert!(res2.is_some());
    let resp2 = res2.unwrap();

    assert_eq!(resp2.id, Some(51));
    assert!(resp2.error.is_none());
    let call_res: McpToolCallResult =
        serde_json::from_value(resp2.result.expect("Result required")).expect("McpToolCallResult");
    assert!(call_res.is_error);
    assert!(call_res.extract_text().contains("Unknown tool 'non_existent_tool_999'"));
}

// =========================================================================
// 8. Git Worktree Sandboxing Lifecycle
// =========================================================================

#[test]
fn test_git_worktree_sandbox_lifecycle() {
    let unique_name = format!(
        "tagisan_test_git_repo_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let temp_repo = std::env::temp_dir().join(unique_name);
    fs::create_dir_all(&temp_repo).unwrap();

    // Initialize clean git repository
    let init = Command::new("git")
        .args(["init"])
        .current_dir(&temp_repo)
        .output();

    if let Ok(out) = init {
        if !out.status.success() {
            let _ = fs::remove_dir_all(&temp_repo);
            eprintln!("Skipping git worktree test: git init failed");
            return;
        }
    } else {
        let _ = fs::remove_dir_all(&temp_repo);
        eprintln!("Skipping git worktree test: git command not available");
        return;
    }

    // Configure user identity for test repository
    let _ = Command::new("git")
        .args(["config", "user.email", "test@tagisan.ai"])
        .current_dir(&temp_repo)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "Tagisan Agent"])
        .current_dir(&temp_repo)
        .output();

    // Create initial commit
    let test_file = temp_repo.join("README.md");
    fs::write(&test_file, "# Tagisan Test Repo\n").unwrap();

    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(&temp_repo)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&temp_repo)
        .output();

    // Provision worktree sandbox
    let branch_name = format!("agent_sandbox_branch_{}", std::process::id());
    let mut sandbox = match WorktreeSandbox::create(&temp_repo, &branch_name) {
        Ok(s) => s,
        Err(e) => {
            let _ = fs::remove_dir_all(&temp_repo);
            panic!("WorktreeSandbox::create failed: {e}");
        }
    };

    assert!(
        sandbox.path().exists(),
        "Worktree sandbox directory must exist"
    );
    assert_eq!(sandbox.branch(), branch_name);

    // Test command execution in sandbox
    let (code, stdout, _) = sandbox
        .run_command("git status")
        .expect("run_command should succeed");
    assert_eq!(code, 0);
    assert!(
        stdout.contains("On branch") || stdout.contains("branch"),
        "git status in sandbox output: {stdout}"
    );

    // Create a new file in worktree
    let new_file = sandbox.path().join("feature.txt");
    fs::write(&new_file, "New isolated feature content\n").unwrap();

    // Commit changes in sandbox
    let commit_sha = sandbox
        .commit_all("feat: add feature in isolated worktree")
        .expect("commit_all should succeed");
    assert_eq!(commit_sha.len(), 40, "Commit SHA must be 40 characters");

    let worktree_path = sandbox.path().to_path_buf();

    // Explicit cleanup
    sandbox.cleanup().expect("Cleanup should succeed");
    assert!(
        !worktree_path.exists(),
        "Worktree directory must be removed after cleanup"
    );

    // Clean up base repository
    let _ = fs::remove_dir_all(&temp_repo);
}
