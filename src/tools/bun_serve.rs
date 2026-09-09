use super::ToolHandler;
use crate::bun::runtime::BunRuntime;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

struct ActiveServer {
    child: Child,
    url: String,
    started_at: Instant,
}

/// Tool for spinning up and managing live UI dev servers using `Bun.serve`
#[derive(Clone)]
pub struct BunServeTool {
    servers: Arc<Mutex<HashMap<u16, ActiveServer>>>,
    pub working_dir: Option<PathBuf>,
}

impl Default for BunServeTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunServeTool {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for BunServeTool {
    fn name(&self) -> &'static str {
        "bun_serve"
    }

    fn description(&self) -> &'static str {
        "Start, inspect, query, or stop high-performance live HTTP web servers using Bun.serve. Supports serving static HTML/JS/CSS and custom API routes."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start", "stop", "status", "fetch"],
                    "description": "The action to perform: 'start' server, 'stop' server, 'status' list servers, or 'fetch' to test an endpoint."
                },
                "port": {
                    "type": "integer",
                    "description": "Port to bind (use 0 for auto-assigned ephemeral port)."
                },
                "html": {
                    "type": "string",
                    "description": "Direct HTML content to serve at '/' (for 'start' action)."
                },
                "routes": {
                    "type": "object",
                    "description": "Optional mapping of route paths to response bodies (e.g. {'/api/health': '{\"status\":\"ok\"}'})."
                },
                "url": {
                    "type": "string",
                    "description": "Full URL to query (for 'fetch' action, e.g. 'http://localhost:3000/api/health')."
                },
                "method": {
                    "type": "string",
                    "description": "HTTP method for fetch action (GET, POST, etc., default: GET)."
                },
                "body": {
                    "type": "string",
                    "description": "Optional request body for POST/PUT fetch action."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'action'".to_string()))?;

        match action {
            "start" => {
                let port = arguments.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let html = arguments.get("html").and_then(|v| v.as_str()).unwrap_or("<h1>Tagisan Live UI Dev Server</h1>");
                let routes_val = arguments.get("routes").cloned().unwrap_or(json!({}));

                let bun_path = BunRuntime::find_bun().ok_or_else(|| {
                    TagisanError::Execution("Bun binary not found for live dev server".to_string())
                })?;

                let routes_json = serde_json::to_string(&routes_val).unwrap_or_else(|_| "{}".to_string());
                let html_escaped = serde_json::to_string(html).unwrap_or_else(|_| "\"\"".to_string());

                let serve_script = format!(
                    r#"
                    import {{ serve }} from "bun";
                    const routes = {routes_json};
                    const defaultHtml = {html_escaped};

                    const server = serve({{
                      port: {port},
                      fetch(req) {{
                        const url = new URL(req.url);
                        const path = url.pathname;

                        if (routes[path] !== undefined) {{
                          const val = routes[path];
                          const body = typeof val === "object" ? JSON.stringify(val) : String(val);
                          const isJson = body.startsWith("{{") || body.startsWith("[");
                          return new Response(body, {{
                            headers: {{ "Content-Type": isJson ? "application/json" : "text/plain" }}
                          }});
                        }}

                        return new Response(defaultHtml, {{
                          headers: {{ "Content-Type": "text/html; charset=utf-8" }}
                        }});
                      }}
                    }});

                    console.log(`TAGISAN_SERVER_PORT:${{server.port}}`);
                    "#
                );

                let temp_script = std::env::temp_dir().join(format!(
                    "tagisan_srv_{}_{}_{}.ts",
                    std::process::id(),
                    port,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
                tokio::fs::write(&temp_script, &serve_script)
                    .await
                    .map_err(|e| TagisanError::Execution(format!("Failed to write server script: {e}")))?;

                let mut cmd = Command::new(bun_path);
                cmd.arg("run");
                cmd.arg(&temp_script);
                if let Some(ref dir) = self.working_dir {
                    cmd.current_dir(dir);
                }
                cmd.stdin(Stdio::null());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
                cmd.kill_on_drop(true);

                let mut child = cmd.spawn().map_err(|e| {
                    TagisanError::Execution(format!("Failed to spawn Bun server: {e}"))
                })?;

                let stdout = child.stdout.take().ok_or_else(|| {
                    TagisanError::Execution("Failed to acquire server stdout".to_string())
                })?;

                let mut reader = BufReader::new(stdout).lines();

                let wait_ready = timeout(Duration::from_secs(5), async {
                    while let Ok(Some(line)) = reader.next_line().await {
                        if line.starts_with("TAGISAN_SERVER_PORT:") {
                            if let Ok(p) = line["TAGISAN_SERVER_PORT:".len()..].trim().parse::<u16>() {
                                return Ok(p);
                            }
                        }
                    }
                    Err(TagisanError::Execution("Server did not emit port notification".to_string()))
                })
                .await;

                let actual_port = match wait_ready {
                    Ok(Ok(p)) => p,
                    _ => {
                        let _ = child.kill().await;
                        return Err(TagisanError::Execution("Server startup timed out or failed to bind port".to_string()));
                    }
                };

                let url = format!("http://localhost:{actual_port}");
                {
                    let mut map = self.servers.lock().await;
                    map.insert(
                        actual_port,
                        ActiveServer {
                            child,
                            url: url.clone(),
                            started_at: Instant::now(),
                        },
                    );
                }

                let resp = json!({
                    "status": "started",
                    "url": url,
                    "port": actual_port,
                    "message": "Live UI Dev Server started successfully"
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "status" => {
                let map = self.servers.lock().await;
                let mut list = Vec::new();
                for (port, srv) in map.iter() {
                    let uptime = srv.started_at.elapsed().as_secs();
                    list.push(json!({
                        "port": port,
                        "url": srv.url,
                        "uptime_secs": uptime
                    }));
                }
                let resp = json!({
                    "status": "ok",
                    "servers": list
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "fetch" => {
                let url = arguments
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing required parameter 'url' for fetch".to_string()))?;

                let method = arguments.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                let body = arguments.get("body").and_then(|v| v.as_str());

                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .map_err(|e| TagisanError::Execution(format!("HTTP client error: {e}")))?;

                let req_builder = match method.to_uppercase().as_str() {
                    "POST" => {
                        let mut b = client.post(url);
                        if let Some(d) = body {
                            b = b.body(d.to_string());
                        }
                        b
                    }
                    "PUT" => {
                        let mut b = client.put(url);
                        if let Some(d) = body {
                            b = b.body(d.to_string());
                        }
                        b
                    }
                    "DELETE" => client.delete(url),
                    _ => client.get(url),
                };

                let resp = req_builder
                    .send()
                    .await
                    .map_err(|e| TagisanError::Execution(format!("Fetch failed: {e}")))?;

                let status = resp.status().as_u16();
                let text = resp.text().await.unwrap_or_default();

                let fetch_resp = json!({
                    "status": status,
                    "body": text
                });
                Ok(serde_json::to_string(&fetch_resp).unwrap())
            }

            "stop" => {
                let port_opt = arguments.get("port").and_then(|v| v.as_u64()).map(|p| p as u16);
                let mut map = self.servers.lock().await;

                if let Some(port) = port_opt {
                    if let Some(mut srv) = map.remove(&port) {
                        let _ = srv.child.kill().await;
                        Ok(format!("Server on port {port} stopped successfully."))
                    } else {
                        Ok(format!("No server running on port {port}."))
                    }
                } else {
                    let count = map.len();
                    for (_, mut srv) in map.drain() {
                        let _ = srv.child.kill().await;
                    }
                    Ok(format!("Stopped all {count} active servers."))
                }
            }

            _ => Err(TagisanError::Execution(format!("Unknown action '{action}' for bun_serve"))),
        }
    }
}

// =========================================================================
// BunStreamBusTool (Swarm Telemetry WebSocket & Pub/Sub Bus)
// =========================================================================

/// High-throughput real-time WebSocket telemetry and pub/sub bus for swarm coordination using Bun.serve uWebSockets
#[derive(Clone)]
pub struct BunStreamBusTool {
    servers: Arc<Mutex<HashMap<u16, ActiveServer>>>,
    pub working_dir: Option<PathBuf>,
}

impl Default for BunStreamBusTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunStreamBusTool {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for BunStreamBusTool {
    fn name(&self) -> &'static str {
        "bun_stream_bus"
    }

    fn description(&self) -> &'static str {
        "Manage high-throughput real-time Swarm Telemetry WebSocket & Pub/Sub buses using Bun.serve uWebSockets. Supports multi-topic broadcast and subscription."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start", "publish", "status", "stop"],
                    "description": "Action to perform: 'start' bus, 'publish' message, 'status' list buses, 'stop' terminate bus."
                },
                "port": {
                    "type": "integer",
                    "description": "Port to bind (use 0 for auto-assigned ephemeral port)."
                },
                "topic": {
                    "type": "string",
                    "description": "Pub/Sub topic (default: 'default')."
                },
                "message": {
                    "description": "Message payload to publish (string or JSON object)."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'action'".to_string()))?;

        match action {
            "start" => {
                let port = arguments.get("port").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let bun_path = BunRuntime::find_bun().ok_or_else(|| {
                    TagisanError::Execution("Bun binary not found for stream bus".to_string())
                })?;

                let bus_script = format!(
                    r#"
                    import {{ serve }} from "bun";

                    const server = serve({{
                      port: {port},
                      fetch(req, server) {{
                        const url = new URL(req.url);
                        if (url.pathname === "/ws") {{
                          const topic = url.searchParams.get("topic") || "default";
                          const upgraded = server.upgrade(req, {{ data: {{ topic }} }});
                          if (upgraded) return undefined;
                        }}

                        if (req.method === "POST" && url.pathname === "/publish") {{
                          return (async () => {{
                            try {{
                              const body = await req.json();
                              const topic = body.topic || "default";
                              const payload = typeof body.message === "object" ? JSON.stringify(body.message) : String(body.message);
                              const sent = server.publish(topic, payload);
                              return new Response(JSON.stringify({{
                                status: "published",
                                topic,
                                subscribers: server.subscriberCount(topic),
                                bytes_sent: sent
                              }}), {{
                                headers: {{ "Content-Type": "application/json" }}
                              }});
                            }} catch (e) {{
                              return new Response(JSON.stringify({{ error: String(e) }}), {{ status: 400 }});
                            }}
                          }})();
                        }}

                        if (url.pathname === "/health") {{
                          return new Response(JSON.stringify({{ status: "healthy", bus: true, port: server.port }}), {{
                            headers: {{ "Content-Type": "application/json" }}
                          }});
                        }}

                        return new Response("Tagisan Stream Bus", {{ status: 200 }});
                      }},
                      websocket: {{
                        open(ws) {{
                          const topic = ws.data.topic || "default";
                          ws.subscribe(topic);
                        }},
                        message(ws, message) {{
                          const topic = ws.data.topic || "default";
                          ws.publish(topic, message);
                        }},
                        close(ws) {{
                          const topic = ws.data.topic || "default";
                          ws.unsubscribe(topic);
                        }}
                      }}
                    }});

                    console.log(`TAGISAN_BUS_PORT:${{server.port}}`);
                    "#
                );

                let mut cmd = Command::new(bun_path);
                cmd.args(["run", "-"]);
                if let Some(ref dir) = self.working_dir {
                    cmd.current_dir(dir);
                }
                cmd.stdin(Stdio::piped());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
                cmd.kill_on_drop(true);

                let mut child = cmd.spawn().map_err(|e| {
                    TagisanError::Execution(format!("Failed to spawn Bun stream bus: {e}"))
                })?;

                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(bus_script.as_bytes()).await;
                    let _ = stdin.flush().await;
                }

                let stdout = child.stdout.take().ok_or_else(|| {
                    TagisanError::Execution("Failed to acquire stream bus stdout".to_string())
                })?;

                let mut reader = BufReader::new(stdout).lines();
                let wait_ready = timeout(Duration::from_secs(5), async {
                    while let Ok(Some(line)) = reader.next_line().await {
                        if line.starts_with("TAGISAN_BUS_PORT:") {
                            if let Ok(p) = line["TAGISAN_BUS_PORT:".len()..].trim().parse::<u16>() {
                                return Ok(p);
                            }
                        }
                    }
                    Err(TagisanError::Execution("Stream bus did not emit port notification".to_string()))
                })
                .await;

                let actual_port = match wait_ready {
                    Ok(Ok(p)) => p,
                    _ => {
                        let _ = child.kill().await;
                        return Err(TagisanError::Execution("Stream bus startup timed out or failed to bind port".to_string()));
                    }
                };

                let url = format!("http://localhost:{actual_port}");
                let ws_url = format!("ws://localhost:{actual_port}/ws");
                {
                    let mut map = self.servers.lock().await;
                    map.insert(
                        actual_port,
                        ActiveServer {
                            child,
                            url: url.clone(),
                            started_at: Instant::now(),
                        },
                    );
                }

                let resp = json!({
                    "status": "started",
                    "port": actual_port,
                    "url": url,
                    "ws_url": ws_url
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "publish" => {
                let port = arguments.get("port").and_then(|v| v.as_u64()).map(|p| p as u16);
                let topic = arguments.get("topic").and_then(|v| v.as_str()).unwrap_or("default");
                let message = arguments.get("message").cloned().unwrap_or(json!({}));

                let target_port = match port {
                    Some(p) => p,
                    None => {
                        let map = self.servers.lock().await;
                        map.keys().copied().next().ok_or_else(|| {
                            TagisanError::Execution("No active stream bus available to publish to".to_string())
                        })?
                    }
                };

                let publish_url = format!("http://localhost:{target_port}/publish");
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                    .map_err(|e| TagisanError::Execution(format!("HTTP client error: {e}")))?;

                let resp = client
                    .post(&publish_url)
                    .json(&json!({
                        "topic": topic,
                        "message": message
                    }))
                    .send()
                    .await
                    .map_err(|e| TagisanError::Execution(format!("Publish failed: {e}")))?;

                let text = resp.text().await.unwrap_or_default();
                Ok(text)
            }

            "status" => {
                let map = self.servers.lock().await;
                let mut list = Vec::new();
                for (port, srv) in map.iter() {
                    let uptime = srv.started_at.elapsed().as_secs();
                    list.push(json!({
                        "port": port,
                        "url": srv.url,
                        "ws_url": format!("ws://localhost:{port}/ws"),
                        "uptime_secs": uptime
                    }));
                }
                let resp = json!({
                    "status": "ok",
                    "buses": list
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "stop" => {
                let port_opt = arguments.get("port").and_then(|v| v.as_u64()).map(|p| p as u16);
                let mut map = self.servers.lock().await;

                if let Some(port) = port_opt {
                    if let Some(mut srv) = map.remove(&port) {
                        let _ = srv.child.kill().await;
                        Ok(format!("Stream bus on port {port} stopped successfully."))
                    } else {
                        Ok(format!("No stream bus running on port {port}."))
                    }
                } else {
                    let count = map.len();
                    for (_, mut srv) in map.drain() {
                        let _ = srv.child.kill().await;
                    }
                    Ok(format!("Stopped all {count} active stream buses."))
                }
            }

            _ => Err(TagisanError::Execution(format!("Unknown action '{action}' for bun_stream_bus"))),
        }
    }
}

