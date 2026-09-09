use crate::bun::runtime::{BunExecutionResult, BunRuntime};
use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;
use tracing::{debug, warn};

/// Embedded daemon worker script executed by Bun workers
pub const WORKER_DAEMON_SCRIPT: &str = r#"
import { createInterface } from "readline";
const rl = createInterface({ input: process.stdin, terminal: false });
const transpiler = new Bun.Transpiler({ loader: "ts" });
const AsyncFunction = Object.getPrototypeOf(async function(){}).constructor;

rl.on("line", async (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;
  try {
    const msg = JSON.parse(trimmed);
    const { id, method, params } = msg;

    if (method === "ping") {
      process.stdout.write(JSON.stringify({ id, result: "pong" }) + "\n");
    } else if (method === "eval") {
      const { code, is_expression } = params;
      const start = performance.now();
      let stdoutBuffer = "";
      const originalLog = console.log;
      console.log = (...args) => {
        stdoutBuffer += args.map(a => typeof a === "object" ? JSON.stringify(a) : String(a)).join(" ") + "\n";
      };

      try {
        const js = transpiler.transformSync(code);
        const fn = new AsyncFunction(js);
        const evalRes = await fn();
        console.log = originalLog;

        let output = stdoutBuffer;
        if (evalRes !== undefined && (is_expression || !output)) {
          if (output && !output.endsWith("\n")) output += "\n";
          output += (typeof evalRes === "object" ? JSON.stringify(evalRes) : String(evalRes));
        }

        const durationMs = Math.max(1, Math.round(performance.now() - start));
        process.stdout.write(JSON.stringify({
          id,
          result: {
            exit_code: 0,
            stdout: output,
            stderr: "",
            duration_ms: durationMs,
            success: true
          }
        }) + "\n");
      } catch (evalErr) {
        console.log = originalLog;
        const durationMs = Math.max(1, Math.round(performance.now() - start));
        process.stdout.write(JSON.stringify({
          id,
          result: {
            exit_code: 1,
            stdout: stdoutBuffer,
            stderr: String(evalErr?.stack || evalErr),
            duration_ms: durationMs,
            success: false
          }
        }) + "\n");
      }
    }
  } catch (parseErr) {}
});
"#;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<BunExecutionResult>>>>>;

/// Individual persistent warm Bun worker
pub struct BunWorker {
    pub id: usize,
    stdin: Arc<Mutex<ChildStdin>>,
    pending: PendingMap,
    is_alive: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
    request_counter: Arc<AtomicU64>,
}

impl BunWorker {
    /// Spawns a persistent Bun worker with a warm JavaScriptCore runtime
    pub async fn spawn(id: usize, bun_path: &PathBuf, cwd: Option<&PathBuf>) -> Result<Self> {
        let daemon_path = std::env::temp_dir().join(format!("tagisan_worker_daemon_{}.ts", std::process::id()));
        let _ = tokio::fs::write(&daemon_path, WORKER_DAEMON_SCRIPT).await;

        let mut cmd = Command::new(bun_path);
        cmd.arg("run");
        cmd.arg(&daemon_path);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!("Failed to spawn Bun worker #{id}: {e}"))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stdout".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TagisanError::Execution("Failed to acquire child stderr".to_string()))?;

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let is_alive = Arc::new(AtomicBool::new(true));
        let is_alive_clone = is_alive.clone();
        let pending_clone = pending.clone();

        // Stdout reader task processing JSON-RPC responses
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                    let mut map = pending_clone.lock().await;
                    if let Some(tx) = map.remove(&resp.id) {
                        if let Some(res_val) = resp.result {
                            if res_val == "pong" {
                                let _ = tx.send(Ok(BunExecutionResult {
                                    exit_code: 0,
                                    stdout: "pong".to_string(),
                                    stderr: String::new(),
                                    duration_ms: 0,
                                    success: true,
                                }));
                            } else if let Ok(exec_res) = serde_json::from_value::<BunExecutionResult>(res_val) {
                                let _ = tx.send(Ok(exec_res));
                            } else {
                                let _ = tx.send(Err(TagisanError::Execution("Failed to deserialize execution result".to_string())));
                            }
                        } else if let Some(err) = resp.error {
                            let _ = tx.send(Err(TagisanError::Execution(err)));
                        }
                    }
                }
            }

            is_alive_clone.store(false, Ordering::SeqCst);
            // Cancel any remaining requests
            let mut map = pending_clone.lock().await;
            for (_, tx) in map.drain() {
                let _ = tx.send(Err(TagisanError::Execution(format!(
                    "Bun worker #{id} died unexpectedly"
                ))));
            }
        });

        // Stderr logging task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                debug!("[Bun Worker #{} stderr]: {}", id, line);
            }
        });

        let worker = Self {
            id,
            stdin: Arc::new(Mutex::new(stdin)),
            pending,
            is_alive,
            child: Arc::new(Mutex::new(Some(child))),
            request_counter: Arc::new(AtomicU64::new(1)),
        };

        // Quick ping handshake to verify worker readiness
        worker.ping().await?;

        Ok(worker)
    }

    /// Sends a ping probe to verify worker liveness
    pub async fn ping(&self) -> Result<()> {
        let req_id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        let req = serde_json::json!({
            "id": req_id,
            "method": "ping",
            "params": {}
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.pending.lock().await;
            map.insert(req_id, tx);
        }

        let serialized = format!("{req}\n");
        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(serialized.as_bytes())
                .await
                .map_err(|e| TagisanError::Execution(format!("Worker #{}: write failed: {e}", self.id)))?;
            stdin.flush().await.map_err(|e| {
                TagisanError::Execution(format!("Worker #{}: flush failed: {e}", self.id))
            })?;
        }

        match timeout(Duration::from_secs(3), rx).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err(TagisanError::Execution("Worker channel dropped".to_string())),
            Err(_) => {
                let mut map = self.pending.lock().await;
                map.remove(&req_id);
                Err(TagisanError::Execution("Worker ping timed out".to_string()))
            }
        }
    }

    /// Evaluates TypeScript or JavaScript on the warm worker with <2ms latency
    pub async fn eval(&self, code: &str, is_expression: bool, timeout_duration: Duration) -> Result<BunExecutionResult> {
        if !self.is_alive.load(Ordering::SeqCst) {
            return Err(TagisanError::Execution(format!(
                "Bun worker #{} is dead",
                self.id
            )));
        }

        let req_id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        let req = serde_json::json!({
            "id": req_id,
            "method": "eval",
            "params": {
                "code": code,
                "is_expression": is_expression,
            }
        });

        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.pending.lock().await;
            map.insert(req_id, tx);
        }

        let serialized = format!("{req}\n");
        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(serialized.as_bytes())
                .await
                .map_err(|e| TagisanError::Execution(format!("Worker #{}: write failed: {e}", self.id)))?;
            stdin.flush().await.map_err(|e| {
                TagisanError::Execution(format!("Worker #{}: flush failed: {e}", self.id))
            })?;
        }

        match timeout(timeout_duration, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                self.is_alive.store(false, Ordering::SeqCst);
                Err(TagisanError::Execution("Worker response channel dropped".to_string()))
            }
            Err(_) => {
                let mut map = self.pending.lock().await;
                map.remove(&req_id);
                // Mark worker dead on timeout
                self.is_alive.store(false, Ordering::SeqCst);
                let mut child_guard = self.child.lock().await;
                if let Some(mut child) = child_guard.take() {
                    let _ = child.kill().await;
                }
                Err(TagisanError::Execution(format!(
                    "Bun worker #{} timed out after {:.1}s",
                    self.id,
                    timeout_duration.as_secs_f32()
                )))
            }
        }
    }

    /// Checks if worker is currently alive and responsive
    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }

    /// Kills and cleans up worker child process
    pub async fn terminate(&self) {
        self.is_alive.store(false, Ordering::SeqCst);
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }
    }
}

/// Statistics for the Bun worker pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolStats {
    pub total_workers: usize,
    pub active_workers: usize,
    pub total_requests: u64,
    pub avg_latency_ms: f64,
}

/// Thread-safe persistent pool of warm Bun workers with automatic crash recovery
#[derive(Clone)]
pub struct BunWorkerPool {
    bun_path: PathBuf,
    cwd: Option<PathBuf>,
    workers: Arc<Mutex<Vec<Arc<BunWorker>>>>,
    pool_size: usize,
    next_idx: Arc<AtomicU64>,
    total_requests: Arc<AtomicU64>,
    total_latency_ms: Arc<AtomicU64>,
}

impl BunWorkerPool {
    /// Creates and warms up a worker pool of the specified size
    pub async fn new(size: usize) -> Result<Self> {
        let bun_path = BunRuntime::find_bun().ok_or_else(|| {
            TagisanError::Execution("Bun binary not found for worker pool".to_string())
        })?;

        let size = size.max(1);
        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            let worker = BunWorker::spawn(i + 1, &bun_path, None).await?;
            workers.push(Arc::new(worker));
        }

        Ok(Self {
            bun_path,
            cwd: None,
            workers: Arc::new(Mutex::new(workers)),
            pool_size: size,
            next_idx: Arc::new(AtomicU64::new(0)),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_latency_ms: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Returns configured pool size
    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    /// Creates a worker pool bounded to a specific working directory
    pub async fn in_dir(size: usize, cwd: PathBuf) -> Result<Self> {
        let mut pool = Self::new(size).await?;
        pool.cwd = Some(cwd);
        Ok(pool)
    }

    /// Acquires an active worker, automatically respawning dead workers
    async fn acquire_worker(&self) -> Result<Arc<BunWorker>> {
        let mut guard = self.workers.lock().await;

        // 1. Search for an alive worker
        for worker in guard.iter() {
            if worker.is_alive() {
                return Ok(worker.clone());
            }
        }

        // 2. All workers dead, respawn slot
        let idx = (self.next_idx.fetch_add(1, Ordering::SeqCst) as usize) % guard.len();
        warn!("[BunWorkerPool] Worker #{} died, spawning replacement...", idx + 1);
        let replacement = BunWorker::spawn(idx + 1, &self.bun_path, self.cwd.as_ref()).await?;
        let replacement_arc = Arc::new(replacement);
        guard[idx] = replacement_arc.clone();
        Ok(replacement_arc)
    }

    /// Evaluates TypeScript code on a warm worker with ultra-low latency (<2ms)
    pub async fn eval(&self, code: &str, timeout_duration: Duration) -> Result<BunExecutionResult> {
        let trimmed = code.trim();
        let is_expression = !trimmed.contains('\n')
            && !trimmed.contains(';')
            && !trimmed.contains("console.log")
            && !trimmed.contains("return ");

        let code_to_eval = if is_expression && !trimmed.starts_with("return ") {
            format!("return {trimmed};")
        } else {
            trimmed.to_string()
        };

        let start = Instant::now();
        let worker = self.acquire_worker().await?;
        let result = worker.eval(&code_to_eval, is_expression, timeout_duration).await?;

        let elapsed = start.elapsed().as_millis() as u64;
        self.total_requests.fetch_add(1, Ordering::SeqCst);
        self.total_latency_ms.fetch_add(elapsed, Ordering::SeqCst);

        Ok(result)
    }

    /// Retrieves worker pool telemetry
    pub async fn stats(&self) -> WorkerPoolStats {
        let guard = self.workers.lock().await;
        let active = guard.iter().filter(|w| w.is_alive()).count();
        let requests = self.total_requests.load(Ordering::SeqCst);
        let latency_sum = self.total_latency_ms.load(Ordering::SeqCst);
        let avg_latency = if requests > 0 {
            latency_sum as f64 / requests as f64
        } else {
            0.0
        };

        WorkerPoolStats {
            total_workers: guard.len(),
            active_workers: active,
            total_requests: requests,
            avg_latency_ms: avg_latency,
        }
    }

    /// Terminates all pool workers
    pub async fn shutdown(&self) {
        let guard = self.workers.lock().await;
        for w in guard.iter() {
            w.terminate().await;
        }
    }
}
