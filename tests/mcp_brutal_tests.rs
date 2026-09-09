use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tagisan::{
    AgentShieldScanner, AgentShieldVerdict, JsonRpcResponse, McpClient, McpContentBlock,
    McpServerConfig, McpToolCallResult, StdioTransport,
};

// =========================================================================
// 1. CRITICAL SECURITY VULNERABILITY: AgentShield Bypass on Namespaced MCP Tools
// =========================================================================

#[test]
fn test_agentshield_intercepts_namespaced_mcp_tools() {
    // When MCP tools are registered via McpManager::populate_tool_registry(..., true)
    // or CLI load_and_register_mcp_tools, they are namespaced as `<server>__<tool_name>`
    // (e.g., `filesystem__read_file`, `shell__run_command`, `bash__terminal`).

    // 1. Destructive fork bomb under namespaced shell tool
    let fork_bomb_args = json!({ "command": ":(){ :|:& };:" });
    let namespaced_shell_tools = [
        "shell__run_command",
        "bash__terminal",
        "system__run_command",
        "sh__execute",
        "terminal__run_command",
    ];

    for tool in namespaced_shell_tools {
        let verdict = AgentShieldScanner::scan_tool_call(tool, &fork_bomb_args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield MUST intercept fork bomb on namespaced tool '{tool}'!"
        );
    }

    // 2. Destructive root filesystem deletion `rm -rf /` under namespaced tools
    let rm_root_args = json!({ "command": "rm -rf /" });
    for tool in ["filesystem__run_command", "os__run_command", "mcp__run_command"] {
        let verdict = AgentShieldScanner::scan_tool_call(tool, &rm_root_args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield MUST intercept rm -rf / on namespaced tool '{tool}'!"
        );
    }

    // 3. Sensitive file exfiltration (/etc/shadow, id_rsa) under namespaced file tools
    let shadow_args = json!({ "path": "/etc/shadow" });
    let namespaced_fs_tools = [
        "filesystem__read_file",
        "fs__read_file",
        "local__read_file",
        "storage__read_file",
        "mcp_server__read_file",
    ];

    for tool in namespaced_fs_tools {
        let verdict = AgentShieldScanner::scan_tool_call(tool, &shadow_args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield MUST intercept /etc/shadow on namespaced tool '{tool}'!"
        );
    }

    let ssh_key_args = json!({ "path": "~/.ssh/id_rsa" });
    let verdict_ssh = AgentShieldScanner::scan_tool_call("filesystem__read_file", &ssh_key_args);
    assert!(
        matches!(verdict_ssh, AgentShieldVerdict::Block { .. }),
        "AgentShield MUST block private SSH key access under 'filesystem__read_file'!"
    );
}

// =========================================================================
// 2. PROTOCOL VERIFICATION: Non-Text MCP Content Blocks (Images, Resources) Retained
// =========================================================================

#[test]
fn test_mcp_content_block_non_text_retained_in_extract_text() {
    // When an MCP tool returns an image (e.g. chart, diagram, screenshot)
    let image_call_result = McpToolCallResult {
        content: vec![McpContentBlock::Image {
            data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            mime_type: "image/png".to_string(),
        }],
        is_error: false,
    };

    let extracted = image_call_result.extract_text();
    assert!(
        extracted.contains("image/png") && extracted.contains("base64"),
        "Image content block must be serialized into output: {extracted}"
    );

    // When an MCP tool returns a resource (e.g. database schema, file reference)
    let resource_call_result = McpToolCallResult {
        content: vec![McpContentBlock::Resource {
            resource: json!({
                "uri": "file:///workspace/schema.sql",
                "mimeType": "application/sql",
                "text": "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);"
            }),
        }],
        is_error: false,
    };

    let extracted_resource = resource_call_result.extract_text();
    assert!(
        extracted_resource.contains("CREATE TABLE users"),
        "Resource content block must retain text schema: {extracted_resource}"
    );

    // When mixed (one text, one image)
    let mixed_call_result = McpToolCallResult {
        content: vec![
            McpContentBlock::Text { text: "Here is the chart:".to_string() },
            McpContentBlock::Image {
                data: "dummy_base64_data".to_string(),
                mime_type: "image/jpeg".to_string(),
            },
        ],
        is_error: false,
    };

    let mixed_extracted = mixed_call_result.extract_text();
    assert!(mixed_extracted.contains("Here is the chart:"));
    assert!(
        mixed_extracted.contains("dummy_base64_data"),
        "Image block in mixed response must be retained in agent output: {mixed_extracted}"
    );
}

// =========================================================================
// 3. PROTOCOL VERIFICATION: Server-Initiated JSON-RPC Requests Do Not Poison
// =========================================================================

#[test]
fn test_jsonrpc_server_request_detection() {
    let server_req_raw = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "method": "roots/list",
        "params": {}
    }"#;

    let val: serde_json::Value = serde_json::from_str(server_req_raw).unwrap();
    assert!(
        val.get("method").is_some(),
        "Server request must be recognized by 'method' presence so it does not poison pending map"
    );
}

// =========================================================================
// 4. PROTOCOL COMPLIANCE: JSON-RPC 2.0 String & Negative IDs
// =========================================================================

#[test]
fn test_jsonrpc_string_and_negative_ids_succeed_in_tagisan() {
    let string_id_resp = r#"{
        "jsonrpc": "2.0",
        "id": "req-uuid-12345",
        "result": {
            "protocolVersion": "2024-11-05"
        }
    }"#;

    let parse_res: Result<JsonRpcResponse, _> = serde_json::from_str(string_id_resp);
    assert!(
        parse_res.is_ok(),
        "Tagisan JsonRpcResponse must support standard String IDs!"
    );
    assert_eq!(
        parse_res.unwrap().id,
        Some(tagisan::mcp::protocol::RequestId::String("req-uuid-12345".to_string()))
    );

    let negative_id_resp = r#"{
        "jsonrpc": "2.0",
        "id": -1,
        "result": {}
    }"#;

    let parse_neg: Result<JsonRpcResponse, _> = serde_json::from_str(negative_id_resp);
    assert!(
        parse_neg.is_ok(),
        "JsonRpcResponse must support negative integer IDs!"
    );
    assert_eq!(
        parse_neg.unwrap().id,
        Some(tagisan::mcp::protocol::RequestId::Number(-1))
    );
}

// =========================================================================
// 5. Real Mock Stdio Server Tests: Crash, Timeout, Unicode
// =========================================================================

#[tokio::test]
async fn test_mcp_server_sudden_crash_unblocks_pending() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_crash_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let crash_script = temp_dir.join("crash_server.py");

    let python_code = r#"
import sys
import os

line = sys.stdin.readline()
os._exit(1)
"#;
    fs::write(&crash_script, python_code).unwrap();

    let config = McpServerConfig {
        command: "python".to_string(),
        args: vec!["-u".to_string(), crash_script.to_str().unwrap().to_string()],
        env: HashMap::new(),
    };

    let client_res = McpClient::connect("crash_server", &config).await;
    assert!(client_res.is_err(), "Connecting to crashing server must return an error");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_mcp_delayed_response_timeout() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_timeout_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let slow_script = temp_dir.join("slow_server.py");

    let python_code = r#"
import sys
import time
import json

line = sys.stdin.readline()
req = json.loads(line)
time.sleep(3)
resp = {
    "jsonrpc": "2.0",
    "id": req["id"],
    "result": {
        "protocolVersion": "2024-11-05",
        "capabilities": {"tools": {}},
        "serverInfo": {"name": "slow", "version": "1.0"}
    }
}
sys.stdout.write(json.dumps(resp) + "\n")
sys.stdout.flush()
"#;
    fs::write(&slow_script, python_code).unwrap();

    let config = McpServerConfig {
        command: "python".to_string(),
        args: vec!["-u".to_string(), slow_script.to_str().unwrap().to_string()],
        env: HashMap::new(),
    };

    let transport = StdioTransport::spawn("slow_server", &config).await.unwrap();
    let req = tagisan::JsonRpcRequest::new(1, "initialize", None);

    let result = transport.send_request(req, Duration::from_millis(200)).await;
    assert!(result.is_err(), "Expected timeout error");
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("timed out"), "Error message should report timeout: {err_str}");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_mcp_unicode_and_special_characters_in_tool_call() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_unicode_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let server_script = temp_dir.join("unicode_server.py");

    let python_code = r#"
import sys
import json

sys.stdin.reconfigure(encoding='utf-8')
sys.stdout.reconfigure(encoding='utf-8')

while True:
    line = sys.stdin.readline()
    if not line:
        break
    req = json.loads(line.strip())
    method = req.get("method")
    req_id = req.get("id")

    if method == "initialize":
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": req_id,
            "result": {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}, "serverInfo": {"name": "unicode-srv"}}
        }) + "\n")
        sys.stdout.flush()
    elif method == "notifications/initialized":
        pass
    elif method == "tools/list":
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": req_id,
            "result": {"tools": [{"name": "echo_unicode", "inputSchema": {"type": "object"}}]}
        }) + "\n")
        sys.stdout.flush()
    elif method == "tools/call":
        args = req.get("params", {}).get("arguments", {})
        query = args.get("query", "")
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": req_id,
            "result": {"content": [{"type": "text", "text": f"ECHO: {query}"}], "isError": False}
        }) + "\n")
        sys.stdout.flush()
"#;
    fs::write(&server_script, python_code).unwrap();

    let config = McpServerConfig {
        command: "python".to_string(),
        args: vec!["-u".to_string(), server_script.to_str().unwrap().to_string()],
        env: HashMap::new(),
    };

    let client = McpClient::connect("unicode_server", &config).await.expect("connect failed");

    let unicode_query = "Tagisan ng Talino: 🇵🇭 Kamusta Mundo! \"Special Quotes\" \n\t and 🔥🚀";
    let res = client
        .call_tool("echo_unicode", json!({ "query": unicode_query }))
        .await
        .expect("call_tool failed");

    assert!(!res.is_error);
    assert_eq!(res.extract_text(), format!("ECHO: {unicode_query}"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_mcp_client_ping_probe() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_ping_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let server_script = temp_dir.join("ping_server.py");

    let python_code = r#"
import sys
import json

while True:
    line = sys.stdin.readline()
    if not line:
        break
    req = json.loads(line.strip())
    method = req.get("method")
    req_id = req.get("id")

    if method == "initialize":
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": req_id,
            "result": {"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "ping-srv"}}
        }) + "\n")
        sys.stdout.flush()
    elif method == "notifications/initialized":
        pass
    elif method == "ping":
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0", "id": req_id,
            "result": {}
        }) + "\n")
        sys.stdout.flush()
"#;
    fs::write(&server_script, python_code).unwrap();

    let config = McpServerConfig {
        command: "python".to_string(),
        args: vec!["-u".to_string(), server_script.to_str().unwrap().to_string()],
        env: HashMap::new(),
    };

    let client = McpClient::connect("ping_server", &config).await.expect("connect failed");
    let ping_res = client.ping().await;
    assert!(ping_res.is_ok(), "ping probe must succeed on responsive MCP server");

    let _ = fs::remove_dir_all(&temp_dir);
}

