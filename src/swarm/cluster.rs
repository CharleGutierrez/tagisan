use crate::error::{Result, TagisanError};
use crate::tools::ToolRegistry;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Wire messages between Cluster Coordinator and Cluster Workers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClusterMessage {
    /// Worker registration payload sent upon connection
    RegisterWorker(WorkerInfo),
    /// Acknowledgment of worker registration
    RegisterAck {
        assigned_id: String,
        coordinator_version: String,
        active_nodes: usize,
    },
    /// Ping message for latency and health verification
    HeartbeatPing {
        seq: u64,
        timestamp_ms: u64,
    },
    /// Pong response for latency and health verification
    HeartbeatPong {
        seq: u64,
        timestamp_ms: u64,
        available_ram_mb: usize,
    },
    /// Batch of tool execution tasks dispatched to a worker
    DispatchTaskBatch(ClusterTaskBatch),
    /// Batch of tool execution results returned by a worker
    TaskBatchResult(ClusterTaskResult),
    /// Status inquiry sent by CLI status client
    StatusQuery,
    /// Status response returned to CLI status client
    StatusResponse(ClusterStatusReport),
}

/// Metadata and specifications of a cluster node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub worker_id: String,
    pub hostname: String,
    pub ip_port: String,
    pub cpu_cores: usize,
    pub total_ram_mb: usize,
    pub available_ram_mb: usize,
    pub supported_tools: Vec<String>,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub last_seen_epoch_secs: u64,
}

impl WorkerInfo {
    pub fn collect_local(worker_id: Option<String>) -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "tgs-node".to_string());

        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let id = worker_id.unwrap_or_else(|| {
            format!(
                "worker-{}-{}",
                hostname,
                &blake3::hash(hostname.as_bytes()).to_hex()[..6]
            )
        });

        let registry = ToolRegistry::with_builtins();
        let supported_tools = registry.names();

        Self {
            worker_id: id,
            hostname,
            ip_port: "local".to_string(),
            cpu_cores,
            total_ram_mb: 32768,
            available_ram_mb: 16384,
            supported_tools,
            active_tasks: 0,
            completed_tasks: 0,
            last_seen_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Individual tool execution task for cluster dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionTask {
    pub task_id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub timeout_ms: u64,
    pub working_dir: Option<String>,
}

/// A batch of tasks dispatched atomically to a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTaskBatch {
    pub batch_id: String,
    pub tasks: Vec<ToolExecutionTask>,
}

/// Result of an individual tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub task_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub latency_ms: u64,
    pub worker_id: String,
}

/// Execution result for an entire batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterTaskResult {
    pub batch_id: String,
    pub results: Vec<ToolExecutionResult>,
}

/// Full cluster status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatusReport {
    pub coordinator_addr: String,
    pub total_workers: usize,
    pub workers: Vec<WorkerInfo>,
    pub total_dispatched_tasks: usize,
    pub total_completed_tasks: usize,
}

/// A connected worker session managed by the coordinator
struct WorkerSession {
    info: WorkerInfo,
    tx: mpsc::Sender<String>,
}

/// Cluster Coordinator managing LAN workers and dispatching batched tool execution tasks
pub struct ClusterCoordinator {
    bind_addr: String,
    workers: Arc<RwLock<HashMap<String, WorkerSession>>>,
    dispatched_tasks_count: Arc<RwLock<usize>>,
    completed_tasks_count: Arc<RwLock<usize>>,
}

impl ClusterCoordinator {
    pub fn new(bind_addr: impl Into<String>) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            dispatched_tasks_count: Arc::new(RwLock::new(0)),
            completed_tasks_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn default_coordinator() -> Self {
        Self::new("0.0.0.0:8765")
    }

    /// Run the persistent TCP coordinator server
    pub async fn run_server(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.bind_addr).await.map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to bind Cluster Coordinator to {}: {}",
                self.bind_addr, e
            ))
        })?;

        info!("🚀 Cluster Coordinator listening on {}", self.bind_addr);
        println!(
            "{} {}",
            "  🌐  Tagisan P2P Cluster Coordinator running on:".bold().bright_green(),
            self.bind_addr.bold().bright_yellow()
        );

        let workers = self.workers.clone();
        let dispatched_count = self.dispatched_tasks_count.clone();
        let completed_count = self.completed_tasks_count.clone();

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let workers_clone = workers.clone();
                    let dispatched_clone = dispatched_count.clone();
                    let completed_clone = completed_count.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            stream,
                            addr,
                            workers_clone,
                            dispatched_clone,
                            completed_clone,
                        )
                        .await
                        {
                            debug!("Connection closed from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        workers: Arc<RwLock<HashMap<String, WorkerSession>>>,
        dispatched_count: Arc<RwLock<usize>>,
        completed_count: Arc<RwLock<usize>>,
    ) -> Result<()> {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let (tx, mut rx) = mpsc::channel::<String>(64);

        // Writer task
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write_half.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                let _ = write_half.flush().await;
            }
        });

        let mut registered_worker_id: Option<String> = None;
        let mut line = String::new();

        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            if let Ok(msg) = serde_json::from_str::<ClusterMessage>(trimmed) {
                match msg {
                    ClusterMessage::RegisterWorker(mut info) => {
                        info.ip_port = addr.to_string();
                        let assigned_id = info.worker_id.clone();
                        registered_worker_id = Some(assigned_id.clone());

                        println!(
                            "  🤝  {} {} (cores: {}, tools: {})",
                            "Worker registered:".bold().bright_green(),
                            assigned_id.bright_cyan(),
                            info.cpu_cores,
                            info.supported_tools.len()
                        );

                        {
                            let mut w = workers.write().await;
                            w.insert(
                                assigned_id.clone(),
                                WorkerSession {
                                    info: info.clone(),
                                    tx: tx.clone(),
                                },
                            );
                        }

                        let active_nodes = workers.read().await.len();
                        let ack = ClusterMessage::RegisterAck {
                            assigned_id,
                            coordinator_version: env!("CARGO_PKG_VERSION").to_string(),
                            active_nodes,
                        };
                        let serialized = format!("{}\n", serde_json::to_string(&ack)?);
                        let _ = tx.send(serialized).await;
                    }
                    ClusterMessage::HeartbeatPong {
                        seq,
                        available_ram_mb,
                        ..
                    } => {
                        if let Some(id) = &registered_worker_id {
                            let mut w = workers.write().await;
                            if let Some(session) = w.get_mut(id) {
                                session.info.available_ram_mb = available_ram_mb;
                                session.info.last_seen_epoch_secs = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                            }
                        }
                    }
                    ClusterMessage::TaskBatchResult(result) => {
                        let completed_len = result.results.len();
                        {
                            let mut c = completed_count.write().await;
                            *c += completed_len;
                        }
                        if let Some(id) = &registered_worker_id {
                            let mut w = workers.write().await;
                            if let Some(session) = w.get_mut(id) {
                                session.info.completed_tasks += completed_len;
                                session.info.active_tasks = session
                                    .info
                                    .active_tasks
                                    .saturating_sub(completed_len);
                            }
                        }
                        println!(
                            "  ✅  Batch {} completed ({} results from {})",
                            result.batch_id.bright_yellow(),
                            completed_len,
                            registered_worker_id.as_deref().unwrap_or("unknown")
                        );
                    }
                    ClusterMessage::StatusQuery => {
                        let w = workers.read().await;
                        let worker_list: Vec<WorkerInfo> =
                            w.values().map(|s| s.info.clone()).collect();
                        let total_dispatched = *dispatched_count.read().await;
                        let total_completed = *completed_count.read().await;

                        let resp = ClusterMessage::StatusResponse(ClusterStatusReport {
                            coordinator_addr: addr.to_string(),
                            total_workers: worker_list.len(),
                            workers: worker_list,
                            total_dispatched_tasks: total_dispatched,
                            total_completed_tasks: total_completed,
                        });
                        let serialized = format!("{}\n", serde_json::to_string(&resp)?);
                        let _ = tx.send(serialized).await;
                    }
                    _ => {}
                }
            }

            line.clear();
        }

        // Cleanup on disconnect
        if let Some(id) = registered_worker_id {
            let mut w = workers.write().await;
            w.remove(&id);
            println!(
                "  🔌  Worker disconnected: {}",
                id.bright_red()
            );
        }

        writer_task.abort();
        Ok(())
    }

    /// Dispatch a batch of tool tasks across available workers
    pub async fn dispatch_batch(&self, tasks: Vec<ToolExecutionTask>) -> Result<ClusterTaskResult> {
        let batch_id = format!("batch-{}", blake3::hash(format!("{:?}", tasks).as_bytes()).to_hex()[..10].to_string());
        let batch = ClusterTaskBatch {
            batch_id: batch_id.clone(),
            tasks: tasks.clone(),
        };

        let workers = self.workers.read().await;
        if workers.is_empty() {
            return Err(TagisanError::Execution("No cluster workers registered to execute task batch".to_string()));
        }

        // Select worker with lowest active tasks
        let (worker_id, session) = workers
            .iter()
            .min_by_key(|(_, s)| s.info.active_tasks)
            .ok_or_else(|| TagisanError::Execution("No available worker node found".to_string()))?;

        {
            let mut dispatched = self.dispatched_tasks_count.write().await;
            *dispatched += tasks.len();
        }

        let envelope = ClusterMessage::DispatchTaskBatch(batch);
        let serialized = format!("{}\n", serde_json::to_string(&envelope)?);
        session.tx.send(serialized).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to transmit task batch to worker {}: {}", worker_id, e))
        })?;

        // Return provisional envelope
        Ok(ClusterTaskResult {
            batch_id,
            results: Vec::new(),
        })
    }
}

/// Cluster Worker executing sandboxed tool operations dispatched by the Coordinator
pub struct ClusterWorker {
    coordinator_addr: String,
    worker_info: WorkerInfo,
    registry: ToolRegistry,
}

impl ClusterWorker {
    pub fn new(coordinator_addr: impl Into<String>, custom_worker_id: Option<String>) -> Self {
        Self {
            coordinator_addr: coordinator_addr.into(),
            worker_info: WorkerInfo::collect_local(custom_worker_id),
            registry: ToolRegistry::with_builtins(),
        }
    }

    pub fn worker_info(&self) -> &WorkerInfo {
        &self.worker_info
    }

    /// Connect to coordinator and start task execution loop
    pub async fn run_worker(&self) -> Result<()> {
        println!(
            "{} {}",
            "  🔌  Connecting to Cluster Coordinator at:".bold().bright_green(),
            self.coordinator_addr.bold().bright_yellow()
        );

        let stream = TcpStream::connect(&self.coordinator_addr).await.map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to connect to Cluster Coordinator at {}: {}",
                self.coordinator_addr, e
            ))
        })?;

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        // Send registration message
        let reg_msg = ClusterMessage::RegisterWorker(self.worker_info.clone());
        let reg_json = format!("{}\n", serde_json::to_string(&reg_msg)?);
        write_half.write_all(reg_json.as_bytes()).await?;
        write_half.flush().await?;

        println!(
            "  ✨  {} as {}",
            "Handshake sent".bold().bright_green(),
            self.worker_info.worker_id.bright_cyan()
        );

        let mut line = String::new();
        while reader.read_line(&mut line).await? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                line.clear();
                continue;
            }

            if let Ok(msg) = serde_json::from_str::<ClusterMessage>(trimmed) {
                match msg {
                    ClusterMessage::RegisterAck {
                        assigned_id,
                        active_nodes,
                        ..
                    } => {
                        println!(
                            "  🎉  Registered successfully as {} (Cluster size: {} nodes)",
                            assigned_id.bright_cyan().bold(),
                            active_nodes
                        );
                    }
                    ClusterMessage::HeartbeatPing { seq, timestamp_ms } => {
                        let pong = ClusterMessage::HeartbeatPong {
                            seq,
                            timestamp_ms,
                            available_ram_mb: self.worker_info.available_ram_mb,
                        };
                        let pong_json = format!("{}\n", serde_json::to_string(&pong)?);
                        write_half.write_all(pong_json.as_bytes()).await?;
                        write_half.flush().await?;
                    }
                    ClusterMessage::DispatchTaskBatch(batch) => {
                        println!(
                            "  📥  Received task batch {} ({} tasks)",
                            batch.batch_id.bright_yellow(),
                            batch.tasks.len()
                        );

                        let mut results = Vec::new();
                        for task in batch.tasks {
                            let start = Instant::now();
                            let (success, output, error_msg) = match self.registry.get(&task.tool_name) {
                                Some(tool) => match tool.execute(task.parameters.clone()).await {
                                    Ok(out) => (true, out, None),
                                    Err(e) => (false, String::new(), Some(e.to_string())),
                                },
                                None => (false, String::new(), Some(format!("Tool '{}' not found in registry", task.tool_name))),
                            };

                            let latency_ms = start.elapsed().as_millis() as u64;
                            results.push(ToolExecutionResult {
                                task_id: task.task_id,
                                tool_name: task.tool_name,
                                success,
                                output,
                                error: error_msg,
                                latency_ms,
                                worker_id: self.worker_info.worker_id.clone(),
                            });
                        }

                        let result_envelope = ClusterMessage::TaskBatchResult(ClusterTaskResult {
                            batch_id: batch.batch_id,
                            results,
                        });

                        let resp_json = format!("{}\n", serde_json::to_string(&result_envelope)?);
                        write_half.write_all(resp_json.as_bytes()).await?;
                        write_half.flush().await?;
                    }
                    _ => {}
                }
            }

            line.clear();
        }

        warn!("Disconnected from Cluster Coordinator");
        Ok(())
    }
}

/// Query cluster status from a running coordinator over TCP
pub async fn query_cluster_status(coordinator_addr: &str) -> Result<ClusterStatusReport> {
    let stream = TcpStream::connect(coordinator_addr).await.map_err(|e| {
        TagisanError::Execution(format!(
            "Failed to reach Cluster Coordinator at {}: {}",
            coordinator_addr, e
        ))
    })?;

    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let query = ClusterMessage::StatusQuery;
    let query_json = format!("{}\n", serde_json::to_string(&query)?);
    write_half.write_all(query_json.as_bytes()).await?;
    write_half.flush().await?;

    let mut line = String::new();
    if reader.read_line(&mut line).await? > 0 {
        if let Ok(ClusterMessage::StatusResponse(report)) = serde_json::from_str(line.trim()) {
            return Ok(report);
        }
    }

    Err(TagisanError::Execution(
        "Invalid or empty status response from Cluster Coordinator".to_string(),
    ))
}
