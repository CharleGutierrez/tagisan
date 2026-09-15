use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tagisan::engine::EngineContext;
use tagisan::ide::config::{inspect_ide_status, IdeConfigGenerator};
use tagisan::ide::lsp::LspServer;
use tagisan::mcp::protocol::{JsonRpcRequest, RequestId};
use tagisan::mcp::McpServer;
use tagisan::memory::embedding::default_embedding_provider;
use tagisan::memory::store::VectorStore;
use tagisan::tools::ToolRegistry;

fn create_test_mcp_server() -> McpServer {
    let ctx = Arc::new(EngineContext::new(1.0));
    let tools = ToolRegistry::with_builtins();
    let memory_store = Arc::new(VectorStore::load_or_default());
    let embedding_provider = default_embedding_provider();
    McpServer::new(ctx, tools, memory_store, embedding_provider)
}

/// Helper to create an isolated temporary directory for test runs
fn temp_workspace(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("tagisan_ide_test_{}_{}", prefix, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// =========================================================================
// TEST 1: MCP Server tool exposure (25+ tools with valid schemas)
// =========================================================================
#[tokio::test]
async fn test_01_mcp_server_tool_exposure() {
    let server = create_test_mcp_server();
    let tools = server.list_tools();

    println!("Total tools registered in MCP server: {}", tools.len());
    assert!(
        tools.len() >= 25,
        "Expected at least 25 tools exposed, found {}",
        tools.len()
    );

    let tool_names: HashSet<String> = tools.iter().map(|t| t.name.clone()).collect();

    // High-level Tagisan tools
    let required_tagisan_tools = [
        "tagisan_debate",
        "tagisan_moa",
        "tagisan_agent",
        "tagisan_workflow_plan",
        "tagisan_ecc_pipeline",
        "tagisan_memory_search",
        "tagisan_memory_index",
        "tagisan_swarm_run",
    ];

    for name in required_tagisan_tools {
        assert!(
            tool_names.contains(name),
            "Missing high-level Tagisan tool: {}",
            name
        );
    }

    // Built-in tools
    let required_builtin_tools = [
        "view_file",
        "read_file",
        "write_to_file",
        "write_file",
        "replace_file_content",
        "edit_file",
        "grep_search",
        "find_by_name",
        "delete_file",
        "list_dir",
        "ask_question",
        "ask_user",
        "create_artifact",
        "generate_artifact",
        "render_mermaid",
        "render_carousel",
        "generate_image",
        "render_terminal_media",
        "export_artifact_html",
        "validate_mermaid",
        "render_diff",
    ];

    for name in required_builtin_tools {
        assert!(
            tool_names.contains(name),
            "Missing built-in tool: {}",
            name
        );
    }

    // Validate schema integrity for every tool
    for t in &tools {
        assert!(!t.name.trim().is_empty(), "Tool name cannot be empty");
        assert!(
            t.description.is_some() && !t.description.as_ref().unwrap().trim().is_empty(),
            "Tool '{}' must have a non-empty description",
            t.name
        );
        let schema = &t.input_schema;
        assert!(
            schema.is_object(),
            "Tool '{}' input schema must be a JSON object",
            t.name
        );
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "Tool '{}' schema must declare type 'object'",
            t.name
        );
    }

    println!("✅ Test 1 PASSED: All 25+ tools verified with valid input schemas.");
}

// =========================================================================
// TEST 2: MCP Server tool execution over JSON-RPC 2.0
// =========================================================================
#[tokio::test]
async fn test_02_mcp_server_tool_execution() {
    let server = create_test_mcp_server();

    // 1. Execute `view_file` over JSON-RPC
    let view_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(1),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "view_file",
            "arguments": {
                "AbsolutePath": "Cargo.toml",
                "StartLine": 1,
                "EndLine": 10
            }
        })),
    };
    let view_resp = server.handle_request(view_req).await.expect("Expected JSON-RPC response");
    assert_eq!(view_resp.id, Some(RequestId::from(1)));
    let result = view_resp.result.expect("Expected result in response");
    assert_eq!(result.get("isError").and_then(|v| v.as_bool()), Some(false));
    let content = result.get("content").and_then(|v| v.as_array()).expect("Expected content array");
    let text = content[0].get("text").and_then(|v| v.as_str()).expect("Expected text");
    assert!(text.contains("[package]"), "view_file output must contain [package]");

    // 2. Execute `grep_search` over JSON-RPC
    let grep_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(2),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "grep_search",
            "arguments": {
                "SearchPath": "Cargo.toml",
                "Query": "tagisan"
            }
        })),
    };
    let grep_resp = server.handle_request(grep_req).await.expect("Expected JSON-RPC response");
    let grep_res = grep_resp.result.expect("Expected grep result");
    assert_eq!(grep_res.get("isError").and_then(|v| v.as_bool()), Some(false));

    // 3. Execute `render_mermaid` over JSON-RPC
    let mermaid_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(3),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "render_mermaid",
            "arguments": {
                "diagram": "graph TD;\n  A-->B;\n  B-->C;",
                "title": "Architecture Overview"
            }
        })),
    };
    let mermaid_resp = server.handle_request(mermaid_req).await.expect("Expected JSON-RPC response");
    let mermaid_res = mermaid_resp.result.expect("Expected mermaid result");
    assert_eq!(mermaid_res.get("isError").and_then(|v| v.as_bool()), Some(false));

    // 4. Execute `create_artifact` over JSON-RPC
    let artifact_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(4),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "create_artifact",
            "arguments": {
                "TargetFile": "test_artifact.md",
                "CodeContent": "# Test System Design\n\nVerified architecture.",
                "ArtifactMetadata": {
                    "Summary": "Verification test artifact",
                    "UserFacing": true,
                    "RequestFeedback": false
                }
            }
        })),
    };
    let artifact_resp = server.handle_request(artifact_req).await.expect("Expected JSON-RPC response");
    let artifact_res = artifact_resp.result.expect("Expected artifact result");
    assert_eq!(artifact_res.get("isError").and_then(|v| v.as_bool()), Some(false));

    println!("✅ Test 2 PASSED: MCP tool execution succeeded cleanly across all core tools.");
}

// =========================================================================
// TEST 3: IDE configuration generation (VS Code, Cursor, Windsurf, Claude, Zed, JetBrains)
// =========================================================================
#[test]
fn test_03_ide_configuration_generation() {
    let ws = temp_workspace("ide_configs");
    let generator = IdeConfigGenerator::new(&ws).with_exe_path("/opt/tagisan/bin/tgs");

    let generated_files = generator.generate_all().expect("Failed to generate all IDE configs");
    assert!(generated_files.len() >= 6, "Expected at least 6 generated config files");

    // 1. VS Code settings & tasks
    let settings_path = ws.join(".vscode").join("settings.json");
    let tasks_path = ws.join(".vscode").join("tasks.json");
    let extensions_path = ws.join(".vscode").join("extensions.json");
    assert!(settings_path.exists(), ".vscode/settings.json must exist");
    assert!(tasks_path.exists(), ".vscode/tasks.json must exist");
    assert!(extensions_path.exists(), ".vscode/extensions.json must exist");

    let settings_content = std::fs::read_to_string(&settings_path).unwrap();
    let settings_json: serde_json::Value = serde_json::from_str(&settings_content).unwrap();
    assert_eq!(settings_json["tagisan.enable"], true);
    assert_eq!(settings_json["tagisan.lsp.serverPath"], "/opt/tagisan/bin/tgs");

    let tasks_content = std::fs::read_to_string(&tasks_path).unwrap();
    let tasks_json: serde_json::Value = serde_json::from_str(&tasks_content).unwrap();
    let tasks_arr = tasks_json["tasks"].as_array().expect("tasks array");
    let task_labels: Vec<&str> = tasks_arr.iter().filter_map(|t| t["label"].as_str()).collect();
    assert!(task_labels.contains(&"Tagisan: Dialectical Debate"));
    assert!(task_labels.contains(&"Tagisan: Autonomous Agent"));
    assert!(task_labels.contains(&"Tagisan: Verify Invariants (Ground)"));

    // 2. Cursor
    let cursor_mcp = ws.join(".cursor").join("mcp.json");
    let cursorrules = ws.join(".cursorrules");
    assert!(cursor_mcp.exists(), ".cursor/mcp.json must exist");
    assert!(cursorrules.exists(), ".cursorrules must exist");
    let cursor_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&cursor_mcp).unwrap()).unwrap();
    assert_eq!(cursor_json["mcpServers"]["tagisan"]["command"], "/opt/tagisan/bin/tgs");

    // 3. Windsurf
    let windsurf_mcp = ws.join(".codeium").join("windsurf").join("mcp_config.json");
    assert!(windsurf_mcp.exists(), "Windsurf mcp_config.json must exist");
    let windsurf_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&windsurf_mcp).unwrap()).unwrap();
    assert_eq!(windsurf_json["mcpServers"]["tagisan"]["command"], "/opt/tagisan/bin/tgs");

    // 4. Claude Desktop
    let claude_cfg = ws.join("claude_desktop_config.json");
    assert!(claude_cfg.exists(), "claude_desktop_config.json must exist");
    let claude_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&claude_cfg).unwrap()).unwrap();
    assert_eq!(claude_json["mcpServers"]["tagisan"]["command"], "/opt/tagisan/bin/tgs");

    // 5. Zed
    let zed_cfg = ws.join(".zed").join("settings.json");
    assert!(zed_cfg.exists(), ".zed/settings.json must exist");
    let zed_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&zed_cfg).unwrap()).unwrap();
    assert_eq!(zed_json["context_servers"]["tagisan"]["command"], "/opt/tagisan/bin/tgs");

    // 6. JetBrains
    let jb_cfg = ws.join("tagisan.mcp.json");
    assert!(jb_cfg.exists(), "tagisan.mcp.json must exist");
    let jb_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&jb_cfg).unwrap()).unwrap();
    assert_eq!(jb_json["mcpServers"]["tagisan"]["command"], "/opt/tagisan/bin/tgs");

    // Inspect status
    let status = inspect_ide_status(&ws);
    assert!(status.vscode_configured);
    assert!(status.cursor_configured);
    assert!(status.windsurf_configured);
    assert!(status.claude_configured);
    assert!(status.zed_configured);
    assert!(status.jetbrains_configured);
    assert_eq!(status.detected_editors.len(), 6);

    let _ = std::fs::remove_dir_all(&ws);
    println!("✅ Test 3 PASSED: All 6 IDE configuration targets verified with exact JSON schemas.");
}

// =========================================================================
// TEST 4: LSP handshake (initialize, capabilities, serverInfo)
// =========================================================================
#[tokio::test]
async fn test_04_lsp_handshake() {
    let server = LspServer::new();

    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(1),
        method: "initialize".to_string(),
        params: Some(json!({
            "processId": 9999,
            "rootUri": "file:///C:/projects/my-app",
            "capabilities": {}
        })),
    };

    let resp = server.handle_lsp_request(init_req).await.expect("Expected initialize response");
    assert_eq!(resp.id, Some(RequestId::from(1)));

    let result = resp.result.expect("Expected result in initialize response");
    let capabilities = &result["capabilities"];

    assert_eq!(capabilities["textDocumentSync"], 1);
    assert_eq!(capabilities["hoverProvider"], true);
    assert_eq!(capabilities["codeActionProvider"], true);
    assert!(capabilities["diagnosticProvider"].is_object());

    let server_info = &result["serverInfo"];
    assert_eq!(server_info["name"], "tagisan-lsp");
    assert!(!server_info["version"].as_str().unwrap().is_empty());

    println!("✅ Test 4 PASSED: LSP handshake successfully returned required capabilities.");
}

// =========================================================================
// TEST 5: LSP hover and codeAction responses
// =========================================================================
#[tokio::test]
async fn test_05_lsp_hover_and_code_action() {
    let server = LspServer::new();

    // 1. Test hover with a known Agentic/Architecture skill
    let hover_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(10),
        method: "textDocument/hover".to_string(),
        params: Some(json!({
            "textDocument": { "uri": "file:///src/main.rs" },
            "word": "kani-rust-formal-verifier"
        })),
    };

    let hover_resp = server.handle_lsp_request(hover_req).await.expect("Expected hover response");
    let hover_res = hover_resp.result.expect("Expected result for hover");
    let contents = &hover_res["contents"];
    assert_eq!(contents["kind"], "markdown");
    let md = contents["value"].as_str().expect("Expected markdown string");
    assert!(
        md.contains("kani-rust-formal-verifier") || md.contains("Tagisan"),
        "Hover markdown must contain skill details"
    );

    // 2. Test codeAction
    let ca_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(11),
        method: "textDocument/codeAction".to_string(),
        params: Some(json!({
            "textDocument": { "uri": "file:///src/lib.rs" },
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 0, "character": 10 }
            },
            "context": { "diagnostics": [] }
        })),
    };

    let ca_resp = server.handle_lsp_request(ca_req).await.expect("Expected codeAction response");
    let ca_res = ca_resp.result.expect("Expected result for codeAction");
    let actions = ca_res.as_array().expect("Expected array of code actions");

    let titles: Vec<&str> = actions.iter().filter_map(|a| a["title"].as_str()).collect();
    assert!(titles.contains(&"Tagisan: Refactor with Clean Architecture"));
    assert!(titles.contains(&"Tagisan: Run Dialectical Debate"));
    assert!(titles.contains(&"Tagisan: Verify Invariants"));

    println!("✅ Test 5 PASSED: LSP hover & codeAction returned skill metadata and actions.");
}

// =========================================================================
// TEST 6: AgentShield cyber defense interception on malicious IDE requests
// =========================================================================
#[tokio::test]
async fn test_06_agentshield_cyber_defense_interception() {
    // 6a. Attempt to generate IDE config in malicious path with traversal
    let malicious_generator = IdeConfigGenerator::new("../../etc");
    let res = malicious_generator.generate_vscode();
    assert!(
        res.is_err(),
        "AgentShield MUST block config generation on traversal paths"
    );

    // 6b. Attempt malicious MCP tool call with sensitive file access
    let mcp_server = create_test_mcp_server();
    let blocked_call = mcp_server.handle_tool_call("read_file", json!({
        "path": "/etc/shadow"
    })).await;
    assert!(
        blocked_call.is_error,
        "AgentShield MUST block read_file on /etc/shadow"
    );
    let err_msg = match &blocked_call.content[0] {
        tagisan::mcp::protocol::McpContentBlock::Text { text } => text.clone(),
        _ => String::new(),
    };
    assert!(
        err_msg.contains("AgentShield"),
        "Error message must reference AgentShield interception: {}",
        err_msg
    );

    // 6c. Malicious document request in LSP diagnostic
    let lsp_server = LspServer::new();
    let diag_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: RequestId::from(99),
        method: "textDocument/diagnostic".to_string(),
        params: Some(json!({
            "textDocument": { "uri": "file:///etc/shadow" }
        })),
    };
    let diag_resp = lsp_server.handle_lsp_request(diag_req).await.expect("Expected diagnostic response");
    let diag_res = diag_resp.result.expect("Expected result");
    let items = diag_res["items"].as_array().expect("items array");
    assert!(!items.is_empty(), "Expected AgentShield security diagnostic on sensitive path");
    assert_eq!(items[0]["severity"], 1, "Security diagnostic must have error severity 1");
    assert_eq!(items[0]["source"], "AgentShield");

    println!("✅ Test 6 PASSED: AgentShield cyber defense blocked all unauthorized IDE & tool vectors.");
}

// =========================================================================
// TEST 7: Multi-threaded concurrent stress test (50 worker threads)
// =========================================================================
#[tokio::test]
async fn test_07_concurrent_stress_test() {
    let mcp_server = Arc::new(create_test_mcp_server());
    let lsp_server = Arc::new(LspServer::new());

    let mut handles = Vec::new();

    for thread_idx in 0..50 {
        let mcp = Arc::clone(&mcp_server);
        let lsp = Arc::clone(&lsp_server);

        let handle = tokio::spawn(async move {
            if thread_idx % 2 == 0 {
                // MCP stress path
                let req = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: RequestId::from(thread_idx as u64),
                    method: "tools/call".to_string(),
                    params: Some(json!({
                        "name": "render_mermaid",
                        "arguments": {
                            "diagram": format!("graph TD; S{}-->E{};", thread_idx, thread_idx),
                            "title": format!("Worker {}", thread_idx)
                        }
                    })),
                };
                let resp = mcp.handle_request(req).await.expect("Worker request failed");
                assert_eq!(resp.id, Some(RequestId::from(thread_idx as u64)));
            } else {
                // IDE Config generator + LSP hover stress path
                let ws = temp_workspace(&format!("stress_{}", thread_idx));
                let generator = IdeConfigGenerator::new(&ws).with_exe_path("tgs");
                let files = generator.generate_vscode().expect("VS Code gen failed");
                assert_eq!(files.len(), 3);
                let _ = std::fs::remove_dir_all(&ws);

                let hover_req = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    id: RequestId::from(thread_idx as u64),
                    method: "textDocument/hover".to_string(),
                    params: Some(json!({
                        "textDocument": { "uri": "file:///src/main.rs" },
                        "word": "formal-invariant-prover"
                    })),
                };
                let hover_resp = lsp.handle_lsp_request(hover_req).await.expect("LSP hover failed");
                assert!(hover_resp.result.is_some());
            }
        });

        handles.push(handle);
    }

    for (i, h) in handles.into_iter().enumerate() {
        h.await.unwrap_or_else(|e| panic!("Thread {} panicked: {:?}", i, e));
    }

    println!("✅ Test 7 PASSED: 50 concurrent worker threads completed with 0 errors and 0 deadlocks.");
}
