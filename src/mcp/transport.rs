use crate::error::{Result, TagisanError};
use crate::mcp::config::McpServerConfig;
use crate::mcp::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;
use tracing::{debug, warn};

type PendingRequests = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<JsonRpcResponse>>>>>;

/// Transport communicating with an MCP server via stdio JSON-RPC 2.0
pub struct StdioTransport {
    server_name: String,
    stdin: Arc<Mutex<ChildStdin>>,
    pending: PendingRequests,
    child: Arc<Mutex<Option<Child>>>,
    is_closed: Arc<AtomicBool>,
}

impl StdioTransport {
    /// Spawn the child process and start background stdio reader tasks
    pub async fn spawn(server_name: impl Into<String>, config: &McpServerConfig) -> Result<Self> {
        let name = server_name.into();

        let mut cmd = if cfg!(target_os = "windows") {
            // Windows handling: resolve .cmd/.bat if needed (including npx, npm, uvx, pnpm, yarn)
            let prog = &config.command;
            let needs_cmd = prog == "npx"
                || prog == "npm"
                || prog == "uvx"
                || prog == "pnpm"
                || prog == "yarn"
                || prog.ends_with(".cmd")
                || prog.ends_with(".bat");

            if needs_cmd {
                let mut c = Command::new("cmd.exe");
                c.args(["/C", prog]);
                c.args(&config.args);
                c
            } else {
                let mut c = Command::new(prog);
                c.args(&config.args);
                c
            }
        } else {
            let mut c = Command::new(&config.command);
            c.args(&config.args);
            c
        };

        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to spawn MCP server '{}' (command '{}'): {e}",
                name, config.command
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stdin pipe".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stdout pipe".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stderr pipe".to_string()))?;

        let pending: PendingRequests = Arc::new(Mutex::new(HashMap::new()));
        let is_closed = Arc::new(AtomicBool::new(false));

        // 1. Stdout reader task: parses line-delimited JSON-RPC responses
        let pending_clone = pending.clone();
        let name_stdout = name.clone();
        let is_closed_clone = is_closed.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                debug!("[MCP Server '{}' stdout]: {}", name_stdout, trimmed);

                match serde_json::from_str::<serde_json::Value>(trimmed) {
                    Ok(val) => {
                        // Check if this is a server-initiated request or notification (contains "method")
                        if val.get("method").is_some() {
                            debug!(
                                "[MCP '{}'] Server-initiated request or notification ignored: {}",
                                name_stdout, trimmed
                            );
                            continue;
                        }

                        match serde_json::from_value::<JsonRpcResponse>(val) {
                            Ok(resp) => {
                                if let Some(id) = resp.id {
                                    let mut map = pending_clone.lock().await;
                                    if let Some(tx) = map.remove(&id) {
                                        let _ = tx.send(Ok(resp));
                                    } else {
                                        debug!(
                                            "[MCP '{}'] Received response for untracked ID {id}",
                                            name_stdout
                                        );
                                    }
                                }
                            }
                            Err(err) => {
                                debug!(
                                    "[MCP '{}'] Failed to parse as JsonRpcResponse: {err} | Raw line: {trimmed}",
                                    name_stdout
                                );
                            }
                        }
                    }
                    Err(err) => {
                        debug!(
                            "[MCP '{}'] Non-JSON-RPC output or parse error: {err} | Raw line: {trimmed}",
                            name_stdout
                        );
                    }
                }
            }

            is_closed_clone.store(true, Ordering::SeqCst);
            // If reader finishes, abort all remaining pending requests
            let mut map = pending_clone.lock().await;
            for (_id, tx) in map.drain() {
                let _ = tx.send(Err(TagisanError::Execution(format!(
                    "MCP server '{}' closed stdout stream unexpectedly",
                    name_stdout
                ))));
            }
        });

        // 2. Stderr logger task: captures diagnostics and warnings
        let name_stderr = name.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    warn!("[MCP '{}' stderr]: {}", name_stderr, trimmed);
                }
            }
        });

        Ok(Self {
            server_name: name,
            stdin: Arc::new(Mutex::new(stdin)),
            pending,
            child: Arc::new(Mutex::new(Some(child))),
            is_closed,
        })
    }

    /// Send a request and wait for the corresponding response matching `req.id`
    pub async fn send_request(
        &self,
        req: JsonRpcRequest,
        timeout_duration: Duration,
    ) -> Result<JsonRpcResponse> {
        if self.is_closed.load(Ordering::SeqCst) {
            return Err(TagisanError::Execution(format!(
                "Cannot send request: MCP server '{}' connection is closed",
                self.server_name
            )));
        }

        let (tx, rx) = oneshot::channel();
        let req_id = req.id;

        {
            let mut map = self.pending.lock().await;
            map.insert(req_id, tx);
        }

        let serialized = serde_json::to_string(&req).map_err(|e| {
            TagisanError::Serialization(e)
        })?;

        {
            let mut stdin_guard = self.stdin.lock().await;
            stdin_guard
                .write_all(format!("{serialized}\n").as_bytes())
                .await
                .map_err(|e| {
                    TagisanError::Execution(format!(
                        "Failed to write request to MCP server '{}' stdin: {e}",
                        self.server_name
                    ))
                })?;
            stdin_guard.flush().await.map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to flush stdin to MCP server '{}': {e}",
                    self.server_name
                ))
            })?;
        }

        match timeout(timeout_duration, rx).await {
            Ok(Ok(response_result)) => response_result,
            Ok(Err(_oneshot_cancelled)) => Err(TagisanError::Execution(format!(
                "MCP server '{}' response channel was dropped before response was received",
                self.server_name
            ))),
            Err(_elapsed) => {
                // Remove from pending map on timeout
                let mut map = self.pending.lock().await;
                map.remove(&req_id);
                Err(TagisanError::Execution(format!(
                    "MCP server '{}' timed out after {:.1}s waiting for response to request ID {}",
                    self.server_name,
                    timeout_duration.as_secs_f32(),
                    req_id
                )))
            }
        }
    }

    /// Send a notification (fire-and-forget, no response expected)
    pub async fn send_notification(&self, notif: JsonRpcNotification) -> Result<()> {
        let serialized = serde_json::to_string(&notif).map_err(TagisanError::Serialization)?;

        let mut stdin_guard = self.stdin.lock().await;
        stdin_guard
            .write_all(format!("{serialized}\n").as_bytes())
            .await
            .map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to send notification to MCP server '{}': {e}",
                    self.server_name
                ))
            })?;
        stdin_guard.flush().await.map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to flush stdin for MCP server '{}': {e}",
                self.server_name
            ))
        })?;

        Ok(())
    }

    /// Terminate and cleanup the child process
    pub async fn close(&self) {
        self.is_closed.store(true, Ordering::SeqCst);
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        self.is_closed.store(true, Ordering::SeqCst);
    }
}
