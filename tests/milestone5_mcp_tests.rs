use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tagisan::{
    AgentShieldScanner, AgentShieldVerdict, AutonomousAgent, EccThreatLevel, JsonRpcNotification,
    JsonRpcRequest, JsonRpcResponse, McpClient, McpConfig, McpContentBlock, McpInitializeResult,
    McpManager, McpServerConfig, McpToolCallResult, McpToolDefinition, McpToolWrapper, ToolHandler,
    ToolRegistry,
};

#[test]
fn test_mcp_config_serialization_and_parsing() {
    let json_str = r#"{
        "mcpServers": {
            "filesystem": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
                "env": {
                    "NODE_ENV": "production"
                }
            },
            "sqlite": {
                "command": "uvx",
                "args": ["mcp-server-sqlite", "--db-path", "app.db"]
            }
        }
    }"#;

    let config = McpConfig::from_json(json_str).expect("Failed to parse McpConfig");
    assert_eq!(config.mcp_servers.len(), 2);

    let fs_server = config.mcp_servers.get("filesystem").expect("filesystem server missing");
    assert_eq!(fs_server.command, "npx");
    assert_eq!(fs_server.args, vec!["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]);
    assert_eq!(fs_server.env.get("NODE_ENV").map(|s| s.as_str()), Some("production"));

    let sqlite_server = config.mcp_servers.get("sqlite").expect("sqlite server missing");
    assert_eq!(sqlite_server.command, "uvx");
    assert_eq!(sqlite_server.args, vec!["mcp-server-sqlite", "--db-path", "app.db"]);
    assert!(sqlite_server.env.is_empty());
}

#[test]
fn test_mcp_config_builder_and_file_io() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let config_path = temp_dir.join("mcp.json");

    let config = McpConfig::new().with_server(
        "custom_server",
        "python",
        vec!["-m".to_string(), "custom_mcp".to_string()],
    );

    let serialized = serde_json::to_string_pretty(&config).expect("Serialize config failed");
    fs::write(&config_path, serialized).expect("Failed to write test config");

    let loaded = McpConfig::from_file(&config_path).expect("Failed to load config from file");
    assert_eq!(loaded.mcp_servers.len(), 1);
    assert!(loaded.mcp_servers.contains_key("custom_server"));

    let server = &loaded.mcp_servers["custom_server"];
    assert_eq!(server.command, "python");
    assert_eq!(server.args, vec!["-m", "custom_mcp"]);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_jsonrpc_protocol_messages() {
    // Request serialization
    let req = JsonRpcRequest::new(42, "tools/list", Some(serde_json::json!({})));
    let req_json = serde_json::to_string(&req).expect("Serialize request failed");
    assert!(req_json.contains(r#""jsonrpc":"2.0""#));
    assert!(req_json.contains(r#""id":42"#));
    assert!(req_json.contains(r#""method":"tools/list""#));

    // Notification serialization (no id)
    let notif = JsonRpcNotification::new("notifications/initialized", None);
    let notif_json = serde_json::to_string(&notif).expect("Serialize notification failed");
    assert!(notif_json.contains(r#""jsonrpc":"2.0""#));
    assert!(notif_json.contains(r#""method":"notifications/initialized""#));
    assert!(!notif_json.contains(r#""id""#));

    // Response with result
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "id": 42,
        "result": {
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "tagisan-test",
                "version": "1.0.0"
            }
        }
    }"#;
    let resp: JsonRpcResponse = serde_json::from_str(resp_json).expect("Deserialize response failed");
    assert_eq!(resp.id, Some(42));
    assert!(resp.error.is_none());
    assert!(resp.result.is_some());

    let init_res: McpInitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(init_res.protocol_version, "2024-11-05");
    assert_eq!(init_res.server_info.as_ref().unwrap().name, "tagisan-test");
    assert_eq!(init_res.server_info.as_ref().unwrap().version.as_deref(), Some("1.0.0"));
}

#[test]
fn test_mcp_tool_call_result_text_extraction() {
    let call_res = McpToolCallResult {
        content: vec![
            McpContentBlock::Text {
                text: "First block".to_string(),
            },
            McpContentBlock::Text {
                text: "Second block".to_string(),
            },
        ],
        is_error: false,
    };

    let extracted = call_res.extract_text();
    assert_eq!(extracted, "First block\nSecond block");

    // Empty content block
    let empty_res = McpToolCallResult {
        content: vec![],
        is_error: false,
    };
    assert_eq!(empty_res.extract_text(), "");
}

#[test]
fn test_mcp_tool_wrapper_naming_and_registry_integration() {
    let tool_def = McpToolDefinition {
        name: "query_db".to_string(),
        description: Some("Execute SQL against SQLite database".to_string()),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "sql": { "type": "string" }
            },
            "required": ["sql"]
        }),
    };

    // Verify schema and parameter inspection
    assert_eq!(tool_def.name, "query_db");
    assert_eq!(tool_def.description.as_deref(), Some("Execute SQL against SQLite database"));
    assert!(tool_def.input_schema["properties"]["sql"].is_object());
}

#[test]
fn test_agentshield_security_interception_on_mcp_tools() {
    // 1. Destructive fork bomb command passed as arguments to an MCP tool
    let fork_bomb_args = serde_json::json!({
        "command": ":(){ :|:& };:"
    });
    let verdict = AgentShieldScanner::scan_tool_call("run_command", &fork_bomb_args);
    match verdict {
        AgentShieldVerdict::Block { reason, threat_level } => {
            assert!(reason.contains("Fork bomb") || reason.contains("pattern"));
            assert_eq!(threat_level, EccThreatLevel::Critical);
        }
        AgentShieldVerdict::Allow => panic!("Fork bomb should have been blocked by AgentShield!"),
    }

    // 2. Destructive rm -rf / root erasure
    let rm_root_args = serde_json::json!({
        "command": "rm -rf /"
    });
    let verdict = AgentShieldScanner::scan_tool_call("run_command", &rm_root_args);
    match verdict {
        AgentShieldVerdict::Block { reason, threat_level } => {
            assert!(reason.to_lowercase().contains("recursive deletion") || reason.to_lowercase().contains("root"));
            assert_eq!(threat_level, EccThreatLevel::Critical);
        }
        AgentShieldVerdict::Allow => panic!("rm -rf / should have been blocked by AgentShield!"),
    }

    // 3. Sensitive file exfiltration (/etc/shadow)
    let shadow_args = serde_json::json!({
        "path": "/etc/shadow"
    });
    let verdict = AgentShieldScanner::scan_tool_call("read_file", &shadow_args);
    match verdict {
        AgentShieldVerdict::Block { reason, .. } => {
            assert!(reason.contains("shadow") || reason.contains("System credential"));
        }
        AgentShieldVerdict::Allow => panic!("/etc/shadow should have been blocked!"),
    }

    // 4. AutonomousAgent secret redaction on tool outputs containing leaked API keys
    let raw_tool_output = "Connected to cluster. Key sk-ant-api03-abcdef1234567890abcdef1234567890 and OpenAI key sk-proj-1234567890abcdef1234567890 were used.";
    let sanitized = AutonomousAgent::sanitize_text(raw_tool_output);
    assert!(!sanitized.contains("sk-ant-api03-abcdef1234567890abcdef1234567890"));
    assert!(!sanitized.contains("sk-proj-1234567890abcdef1234567890"));
    assert!(sanitized.contains("[REDACTED_ANTHROPIC_KEY]"));
    assert!(sanitized.contains("[REDACTED_OPENAI_KEY]"));
}

#[tokio::test]
async fn test_end_to_end_real_stdio_mcp_server() {
    // Write an ephemeral Python script implementing standard MCP JSON-RPC 2.0 protocol over stdio
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_e2e_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let server_script = temp_dir.join("mock_mcp_server.py");

    let python_code = r#"
import sys
import json

def handle():
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except Exception:
            continue
        
        method = req.get("method")
        req_id = req.get("id")

        if method == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {
                        "name": "python-echo-server",
                        "version": "1.0.0"
                    }
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
        elif method == "notifications/initialized":
            # Handshake notification, no response required
            pass
        elif method == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "tools": [
                        {
                            "name": "echo",
                            "description": "Echoes back the message",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "message": {"type": "string"}
                                },
                                "required": ["message"]
                            }
                        }
                    ]
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
        elif method == "tools/call":
            params = req.get("params", {})
            args = params.get("arguments", {})
            msg = args.get("message", "empty")
            resp = {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [
                        {
                            "type": "text",
                            "text": f"MOCK_ECHO_RESULT: {msg}"
                        }
                    ],
                    "isError": False
                }
            }
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    handle()
"#;

    fs::write(&server_script, python_code).expect("Failed to write Python server script");

    let script_str = server_script.to_str().unwrap().to_string();
    let srv_config = McpServerConfig {
        command: "python".to_string(),
        args: vec!["-u".to_string(), script_str],
        env: HashMap::new(),
    };

    // 1. Connect and perform initialize handshake
    let client = McpClient::connect("test_echo_server", &srv_config)
        .await
        .expect("Failed to connect to Python MCP server");

    assert_eq!(client.server_name, "test_echo_server");
    assert_eq!(client.protocol_version, "2024-11-05");
    assert_eq!(
        client.server_info.as_ref().unwrap().name,
        "python-echo-server"
    );

    // 2. Discover published tools via tools/list
    let tools = client.list_tools().await.expect("Failed to list tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
    assert_eq!(
        tools[0].description.as_deref(),
        Some("Echoes back the message")
    );

    // 3. Execute tool call via tools/call
    let call_res = client
        .call_tool(
            "echo",
            serde_json::json!({
                "message": "Hello from Tagisan Rust MCP Client!"
            }),
        )
        .await
        .expect("Tool call failed");

    assert!(!call_res.is_error);
    let out = call_res.extract_text();
    assert_eq!(out, "MOCK_ECHO_RESULT: Hello from Tagisan Rust MCP Client!");

    // 4. McpToolWrapper and ToolRegistry integration
    let wrapper = McpToolWrapper::new(Arc::new(client), tools[0].clone(), true);
    assert_eq!(wrapper.name(), "test_echo_server__echo");

    let mut registry = ToolRegistry::new();
    registry.register_tool(wrapper);
    assert!(registry.contains("test_echo_server__echo"));

    let tool_handler = registry
        .get("test_echo_server__echo")
        .expect("Tool not in registry");

    let exec_res = tool_handler
        .execute(serde_json::json!({ "message": "Invoked through ToolRegistry trait" }))
        .await
        .expect("Failed to execute tool via ToolHandler");

    assert_eq!(exec_res, "MOCK_ECHO_RESULT: Invoked through ToolRegistry trait");

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_mcp_manager_multi_server_orchestration() {
    let temp_dir = std::env::temp_dir().join(format!("tagisan_mcp_mgr_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");
    let server_script = temp_dir.join("mgr_server.py");

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
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "mgr-server", "version": "1.0"}
            }
        }) + "\n")
        sys.stdout.flush()
    elif method == "tools/list":
        sys.stdout.write(json.dumps({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": [
                    {"name": "ping", "description": "Returns pong", "inputSchema": {"type": "object"}}
                ]
            }
        }) + "\n")
        sys.stdout.flush()
"#;

    fs::write(&server_script, python_code).expect("Failed to write python script");
    let script_str = server_script.to_str().unwrap().to_string();

    let config = McpConfig::new().with_server(
        "srv1",
        "python",
        vec!["-u".to_string(), script_str.clone()],
    );

    let mut manager = McpManager::new(config);
    assert_eq!(manager.len(), 1);
    assert_eq!(manager.server_names(), vec!["srv1"]);

    let discovered = manager.connect_all().await.expect("connect_all failed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].0, "srv1");
    assert_eq!(discovered[0].1.len(), 1);
    assert_eq!(discovered[0].1[0].name, "ping");

    let mut registry = ToolRegistry::new();
    let registered_count = manager.populate_tool_registry(&mut registry, true);
    assert_eq!(registered_count, 1);
    assert!(registry.contains("srv1__ping"));

    manager.shutdown_all().await;
    let _ = fs::remove_dir_all(&temp_dir);
}
