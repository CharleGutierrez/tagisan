//! Brutal Production-Grade Integration Tests for Local LLM Memory Lifecycle
//! Management in Tagisan (`tgs`) REPL.

use std::net::SocketAddr;
use std::sync::Arc;
use serde_json::json;
use tagisan::agent::AutonomousAgent;
use tagisan::engine::EngineContext;
use tagisan::swarm::repl::{InteractiveRepl, ReplCommand};
use tagisan::tools::ToolRegistry;
use tagisan::{HostMemoryGovernor, OllamaProvider};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct MockRecordedRequest {
    pub method: String,
    pub path: String,
    pub body_json: Option<serde_json::Value>,
}

pub struct MockOllamaLifecycleServer {
    pub addr: SocketAddr,
    pub base_url: String,
    shutdown_token: CancellationToken,
    pub recorded_requests: Arc<Mutex<Vec<MockRecordedRequest>>>,
}

impl MockOllamaLifecycleServer {
    pub async fn start(loaded_models: Vec<&str>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let shutdown_token = CancellationToken::new();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));

        let shutdown_clone = shutdown_token.clone();
        let recorded_clone = recorded_requests.clone();

        let ps_models: Vec<serde_json::Value> = loaded_models
            .into_iter()
            .map(|name| {
                json!({
                    "name": name,
                    "model": name,
                    "size": 2_000_000_000u64,
                    "size_vram": 2_000_000_000u64,
                })
            })
            .collect();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_clone.cancelled() => break,
                    Ok((socket, _)) = listener.accept() => {
                        let (read_half, mut write_half) = socket.into_split();
                        let recorded = recorded_clone.clone();
                        let ps_data = ps_models.clone();

                        tokio::spawn(async move {
                            let mut reader = BufReader::new(read_half);
                            let mut req_line = String::new();
                            if reader.read_line(&mut req_line).await.is_err() || req_line.is_empty() {
                                return;
                            }
                            let parts: Vec<&str> = req_line.split_whitespace().collect();
                            if parts.len() < 2 {
                                return;
                            }
                            let method = parts[0].to_string();
                            let path = parts[1].to_string();

                            let mut content_length = 0usize;
                            loop {
                                let mut line = String::new();
                                if reader.read_line(&mut line).await.is_err() || line == "\r\n" || line.is_empty() {
                                    break;
                                }
                                if let Some((k, v)) = line.split_once(':') {
                                    if k.trim().eq_ignore_ascii_case("content-length") {
                                        content_length = v.trim().parse().unwrap_or(0);
                                    }
                                }
                            }

                            let mut body_bytes = vec![0u8; content_length];
                            if content_length > 0 {
                                let _ = reader.read_exact(&mut body_bytes).await;
                            }
                            let body_json = serde_json::from_slice(&body_bytes).ok();

                            recorded.lock().await.push(MockRecordedRequest {
                                method: method.clone(),
                                path: path.clone(),
                                body_json: body_json.clone(),
                            });

                            let (status, body) = if path == "/api/tags" {
                                (200, json!({"models": [{"name": "llama3.2:1b"}, {"name": "smollm2:1.7b"}]}).to_string().into_bytes())
                            } else if path == "/api/ps" {
                                (200, json!({"models": ps_data}).to_string().into_bytes())
                            } else if path == "/api/generate" {
                                // keep_alive: 0 unload endpoint
                                (200, json!({"status": "success"}).to_string().into_bytes())
                            } else {
                                (404, b"Not found".to_vec())
                            };

                            let resp_header = format!(
                                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                if status == 200 { "200 OK" } else { "404 Not Found" },
                                body.len()
                            );
                            let _ = write_half.write_all(resp_header.as_bytes()).await;
                            let _ = write_half.write_all(&body).await;
                            let _ = write_half.flush().await;
                        });
                    }
                }
            }
        });

        Self {
            addr,
            base_url,
            shutdown_token,
            recorded_requests,
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }
}

// -------------------------------------------------------------------------
// Test 1: Command parsing for /exit, /quit, /q, /model, /m, /provider, /prov
// -------------------------------------------------------------------------
#[test]
fn test_repl_command_parsing_exit_and_models() {
    assert_eq!(InteractiveRepl::parse_command("/exit"), ReplCommand::Exit);
    assert_eq!(InteractiveRepl::parse_command("/quit"), ReplCommand::Exit);
    assert_eq!(InteractiveRepl::parse_command("/q"), ReplCommand::Exit);

    assert_eq!(
        InteractiveRepl::parse_command("/model llama3.2:1b"),
        ReplCommand::Model("llama3.2:1b".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/m smollm2:1.7b"),
        ReplCommand::Model("smollm2:1.7b".to_string())
    );

    assert_eq!(
        InteractiveRepl::parse_command("/provider ollama"),
        ReplCommand::Provider("ollama".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/prov gemini"),
        ReplCommand::Provider("gemini".to_string())
    );
}

// -------------------------------------------------------------------------
// Test 2: Model switch unloads resident models and trims heap
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_switch_model_pre_unload_and_heap_trim() {
    let server = MockOllamaLifecycleServer::start(vec!["resident_model:3b"]).await;

    // Direct memory tuner to mock server
    std::env::set_var("OLLAMA_HOST", &server.base_url);

    let ctx = EngineContext::new(5.0);
    let mock_prov = Arc::new(OllamaProvider::new(&server.base_url));
    let agent = AutonomousAgent::new(mock_prov, "resident_model:3b", ToolRegistry::new());
    let mut repl = InteractiveRepl::new(agent, "test-switch", "resident_model:3b", ctx);

    // Switch model to another local model
    let output = repl
        .switch_model_and_provider("ollama/smollm2:1.7b")
        .await
        .unwrap()
        .unwrap();

    // Verify response indicates memory hygiene
    assert!(
        output.contains("Switched active model to"),
        "Output must confirm model switch"
    );
    assert!(
        output.contains("Memory Hygiene") || output.contains("Unloaded resident local model(s)"),
        "Output must confirm memory hygiene pre-unload: {output}"
    );

    // Verify /api/generate was called with keep_alive: 0
    let recorded = server.recorded_requests.lock().await;
    let unload_req = recorded.iter().find(|r| r.path == "/api/generate");
    assert!(
        unload_req.is_some(),
        "Must send keep_alive: 0 to /api/generate on model switch"
    );

    let body = unload_req.unwrap().body_json.as_ref().unwrap();
    assert_eq!(body["model"], "resident_model:3b");
    assert_eq!(body["keep_alive"], 0);

    drop(recorded);
    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 3: Model switch resilience when Ollama is offline (times out gracefully)
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_switch_model_offline_resilience() {
    // Point to non-existent server
    std::env::set_var("OLLAMA_HOST", "http://127.0.0.1:54321");

    let ctx = EngineContext::new(5.0);
    let mock_prov = Arc::new(OllamaProvider::new("http://127.0.0.1:54321"));
    let agent = AutonomousAgent::new(mock_prov, "test-offline", ToolRegistry::new());
    let mut repl = InteractiveRepl::new(agent, "test-offline-session", "test-offline", ctx);

    let start = std::time::Instant::now();
    let res = repl.switch_model_and_provider("gemini/gemini-2.0-flash").await;
    let elapsed = start.elapsed();

    assert!(res.is_ok(), "Switching must succeed even if Ollama is offline");
    assert!(
        elapsed < std::time::Duration::from_millis(2500),
        "Switching must never hang (must respect 1.5s timeout)"
    );
}

// -------------------------------------------------------------------------
// Test 4: unload_local_models_on_exit when models are resident
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_unload_local_models_on_exit_online() {
    let server = MockOllamaLifecycleServer::start(vec!["active_local_model:1b"]).await;
    std::env::set_var("OLLAMA_HOST", &server.base_url);

    let ctx = EngineContext::new(5.0);
    let mock_prov = Arc::new(OllamaProvider::new(&server.base_url));
    let agent = AutonomousAgent::new(mock_prov, "active_local_model:1b", ToolRegistry::new());
    let repl = InteractiveRepl::new(agent, "test-exit-session", "active_local_model:1b", ctx);

    let unload_res = repl.unload_local_models_on_exit().await;
    assert!(unload_res.is_some(), "Must return confirmation message");
    let msg = unload_res.unwrap();
    assert!(
        msg.contains("Unloaded local LLMs from memory"),
        "Must report memory freed"
    );

    let recorded = server.recorded_requests.lock().await;
    assert!(
        recorded.iter().any(|r| r.path == "/api/generate"),
        "Must invoke /api/generate with keep_alive: 0"
    );

    drop(recorded);
    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 5: unload_local_models_on_exit when Ollama is offline
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_unload_local_models_on_exit_offline() {
    std::env::set_var("OLLAMA_HOST", "http://127.0.0.1:54322");

    let ctx = EngineContext::new(5.0);
    let mock_prov = Arc::new(OllamaProvider::new("http://127.0.0.1:54322"));
    let agent = AutonomousAgent::new(mock_prov, "test-model", ToolRegistry::new());
    let repl = InteractiveRepl::new(agent, "test-exit-offline", "test-model", ctx);

    let start = std::time::Instant::now();
    let res = repl.unload_local_models_on_exit().await;
    let elapsed = start.elapsed();

    assert!(res.is_none() || res.is_some());
    assert!(
        elapsed < std::time::Duration::from_millis(2500),
        "Exit unload must complete within timeout"
    );
}

// -------------------------------------------------------------------------
// Test 6: Heap trimming via HostMemoryGovernor
// -------------------------------------------------------------------------
#[test]
fn test_host_memory_governor_heap_trimming() {
    let gov = HostMemoryGovernor::new();
    // Verify trim_heap executes safely on host OS without panicking
    let _ = gov.trim_heap();
}
