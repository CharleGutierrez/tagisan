//! Ollama Memory Tuner & Inactivity Auto-Unload Daemon
//!
//! Monolithically integrated into Tagisan (TGS) to optimize RAM/VRAM usage
//! on resource-constrained systems (e.g., 8GB/16GB laptops).
//!
//! Features:
//! 1. Inactivity Watchdog: Automatically unloads resident LLMs from VRAM/RAM
//!    after 5 minutes (300s) of inactivity by sending `keep_alive: 0` to Ollama.
//! 2. Auto-Load Reverse Proxy: Listens on a dedicated port (default `11435`),
//!    intercepts client prompts, resets the inactivity timer, and streams responses
//!    back chunk-by-chunk while waking Ollama models on demand.
//! 3. Manual Eviction: Immediate purge of loaded models via CLI or REPL (`/tuner unload`).

use crate::error::{Result, TagisanError};
use crate::governor::{HostMemoryGovernor, LinuxMemInfo, MemoryPressureTier};
use colored::Colorize;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

pub const DEFAULT_TUNER_PORT: u16 = 11435;
pub const DEFAULT_OLLAMA_URL: &str = "http://127.0.0.1:11434";
pub const DEFAULT_INACTIVITY_TIMEOUT_SECS: u64 = 300; // 5 minutes
pub const DEFAULT_WATCHDOG_INTERVAL_SECS: u64 = 5;

/// Safety buffer reserved for OS/UI threads to prevent desktop freeze (512 MB)
pub const ANTI_FREEZE_SAFETY_BUFFER_BYTES: u64 = 512 * 1024 * 1024;

/// Configuration for the Ollama Memory Tuner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTunerConfig {
    pub bind_host: String,
    pub bind_port: u16,
    pub ollama_url: String,
    pub inactivity_timeout_secs: u64,
    pub watchdog_interval_secs: u64,
    pub auto_start_proxy: bool,
}

impl Default for MemoryTunerConfig {
    fn default() -> Self {
        let ollama_url = if let Ok(host) = std::env::var("OLLAMA_HOST") {
            if host.starts_with("http://") || host.starts_with("https://") {
                host
            } else {
                format!("http://{}", host)
            }
        } else if let Ok(url) = std::env::var("OLLAMA_URL") {
            url
        } else {
            DEFAULT_OLLAMA_URL.to_string()
        };

        Self {
            bind_host: "127.0.0.1".to_string(),
            bind_port: DEFAULT_TUNER_PORT,
            ollama_url,
            inactivity_timeout_secs: DEFAULT_INACTIVITY_TIMEOUT_SECS,
            watchdog_interval_secs: DEFAULT_WATCHDOG_INTERVAL_SECS,
            auto_start_proxy: true,
        }
    }
}

static GLOBAL_TUNER: std::sync::OnceLock<Arc<OllamaMemoryTuner>> = std::sync::OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTunerStatus {
    pub is_running: bool,
    pub proxy_addr: String,
    pub ollama_url: String,
    pub ollama_reachable: bool,
    pub idle_duration_secs: u64,
    pub inactivity_threshold_secs: u64,
    pub installed_models: Vec<String>,
    pub active_models_in_vram: Vec<LoadedModelInfo>,
    pub total_unloads: u64,
    pub total_proxied_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedModelInfo {
    pub name: String,
    pub model: String,
    pub size_bytes: u64,
    pub size_vram_bytes: u64,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OllamaPsResponse {
    models: Option<Vec<OllamaPsModel>>,
}

#[derive(Debug, Deserialize)]
struct OllamaPsModel {
    name: String,
    model: String,
    size: Option<u64>,
    size_vram: Option<u64>,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct OllamaUnloadPayload<'a> {
    model: &'a str,
    keep_alive: i32,
}

/// Main Ollama Memory Tuner Engine
#[derive(Clone, Debug)]
pub struct OllamaMemoryTuner {
    pub config: MemoryTunerConfig,
    last_activity_ms: Arc<AtomicU64>,
    total_unloads: Arc<AtomicU64>,
    total_proxied: Arc<AtomicU64>,
    is_running: Arc<AtomicBool>,
    cancellation_token: CancellationToken,
    client: reqwest::Client,
}

impl OllamaMemoryTuner {
    /// Retrieve the shared global instance of the Memory Tuner
    pub fn global() -> Arc<Self> {
        GLOBAL_TUNER.get_or_init(|| Arc::new(Self::default_local())).clone()
    }

    pub fn new(config: MemoryTunerConfig) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let client = reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            config,
            last_activity_ms: Arc::new(AtomicU64::new(now_ms)),
            total_unloads: Arc::new(AtomicU64::new(0)),
            total_proxied: Arc::new(AtomicU64::new(0)),
            is_running: Arc::new(AtomicBool::new(false)),
            cancellation_token: CancellationToken::new(),
            client,
        }
    }

    pub fn default_local() -> Self {
        Self::new(MemoryTunerConfig::default())
    }

    /// Reset the inactivity timer to the current instant
    pub fn touch_activity(&self) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_activity_ms.store(now_ms, Ordering::Release);
    }

    /// Retrieve seconds elapsed since the last incoming activity
    pub fn idle_duration_secs(&self) -> u64 {
        let last_ms = self.last_activity_ms.load(Ordering::Acquire);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        now_ms.saturating_sub(last_ms) / 1000
    }

    /// Check if the tuner background loop is running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    /// Stop all background tasks (watchdog and reverse proxy)
    pub fn stop(&self) {
        self.cancellation_token.cancel();
        self.is_running.store(false, Ordering::Release);
    }

    /// Start the background watchdog and reverse proxy
    pub async fn start(self: Arc<Self>) -> Result<SocketAddr> {
        if self.is_running.load(Ordering::Acquire) {
            let addr: SocketAddr = format!("{}:{}", self.config.bind_host, self.config.bind_port)
                .parse()
                .map_err(|e| TagisanError::Execution(format!("Invalid tuner address: {e}")))?;
            return Ok(addr);
        }

        self.is_running.store(true, Ordering::Release);
        self.touch_activity();

        // 1. Spawn the inactivity watchdog
        let watchdog_self = self.clone();
        tokio::spawn(async move {
            watchdog_self.run_watchdog_loop().await;
        });

        // 2. Bind reverse proxy TCP listener
        let bind_addr = format!("{}:{}", self.config.bind_host, self.config.bind_port);
        let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
            self.is_running.store(false, Ordering::Release);
            TagisanError::Execution(format!(
                "Failed to bind Memory Tuner proxy to {bind_addr}: {e}"
            ))
        })?;

        let local_addr = listener.local_addr().map_err(|e| {
            TagisanError::Execution(format!("Failed to query local socket address: {e}"))
        })?;

        info!(
            "{}",
            format!(
                "🛡️  [Memory Tuner] Reverse Proxy listening on http://{} (upstream: {})",
                local_addr, self.config.ollama_url
            )
            .cyan()
            .bold()
        );

        // 3. Spawn the proxy accept loop
        let proxy_self = self.clone();
        tokio::spawn(async move {
            proxy_self.run_proxy_loop(listener).await;
        });

        Ok(local_addr)
    }

    /// Run the 5-minute inactivity watchdog loop
    async fn run_watchdog_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.watchdog_interval_secs));
        debug!("[Memory Tuner] Inactivity watchdog loop active");

        while !self.cancellation_token.is_cancelled() {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => break,
                _ = interval.tick() => {
                    let idle = self.idle_duration_secs();
                    if idle >= self.config.inactivity_timeout_secs {
                        match self.unload_all_models().await {
                            Ok(unloaded) if !unloaded.is_empty() => {
                                info!(
                                    "{}",
                                    format!(
                                        "⏱️  [Memory Tuner] Inactivity threshold reached (idle {}s >= {}s). Unloaded {} model(s): [{}]",
                                        idle,
                                        self.config.inactivity_timeout_secs,
                                        unloaded.len(),
                                        unloaded.join(", ")
                                    ).yellow().bold()
                                );
                            }
                            Ok(_) => {}
                            Err(e) => {
                                debug!("[Memory Tuner] Watchdog poll error: {e}");
                            }
                        }
                    }
                }
            }
        }
    }

    /// Run the reverse proxy TCP accept loop
    async fn run_proxy_loop(&self, listener: TcpListener) {
        while !self.cancellation_token.is_cancelled() {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => break,
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, peer_addr)) => {
                            let this = self.clone();
                            tokio::spawn(async move {
                                if let Err(e) = this.handle_client_connection(stream, peer_addr).await {
                                    debug!("[Memory Tuner] Client {peer_addr} error: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            debug!("[Memory Tuner] Accept error: {e}");
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                }
            }
        }
    }

    /// Handle an incoming HTTP proxy connection from a client
    async fn handle_client_connection(&self, mut stream: TcpStream, _peer_addr: SocketAddr) -> Result<()> {
        let mut buffer = vec![0u8; 16384];
        let n = stream.read(&mut buffer).await.map_err(|e| {
            TagisanError::Execution(format!("Proxy socket read error: {e}"))
        })?;

        if n == 0 {
            return Ok(());
        }

        // 1. Reset inactivity watchdog
        self.touch_activity();
        self.total_proxied.fetch_add(1, Ordering::Relaxed);

        let request_str = String::from_utf8_lossy(&buffer[..n]);
        let mut lines = request_str.lines();
        let request_line = lines.next().unwrap_or("");
        let mut parts = request_line.split_whitespace();
        let method_str = parts.next().unwrap_or("GET");
        let path = parts.next().unwrap_or("/");

        // CORS preflight
        if method_str == "OPTIONS" {
            let cors_resp = "HTTP/1.1 204 No Content\r\n\
                Access-Control-Allow-Origin: *\r\n\
                Access-Control-Allow-Methods: GET, POST, OPTIONS, HEAD\r\n\
                Access-Control-Allow-Headers: Content-Type, Authorization, X-Tagisan-Client\r\n\
                Content-Length: 0\r\n\r\n";
            stream.write_all(cors_resp.as_bytes()).await.ok();
            return Ok(());
        }

        // Extract body if present
        let (header_part, body_bytes) = if let Some(idx) = buffer[..n].windows(4).position(|w| w == b"\r\n\r\n") {
            (&buffer[..idx], &buffer[idx + 4..n])
        } else {
            (&buffer[..n], &[][..])
        };

        // Parse content length if body was only partially read
        let mut content_length = 0usize;
        for line in String::from_utf8_lossy(header_part).lines() {
            if line.to_ascii_lowercase().starts_with("content-length:") {
                if let Some(val) = line.split(':').nth(1) {
                    content_length = val.trim().parse().unwrap_or(0);
                }
            }
        }

        let mut full_body = body_bytes.to_vec();
        if content_length > full_body.len() {
            let remaining = content_length - full_body.len();
            let mut rest = vec![0u8; remaining];
            if let Ok(read_n) = stream.read_exact(&mut rest).await {
                full_body.extend_from_slice(&rest[..read_n]);
            }
        }

        // 2. Build upstream request to Ollama
        let upstream_url = format!("{}{}", self.config.ollama_url.trim_end_matches('/'), path);
        let method = match method_str.to_uppercase().as_str() {
            "POST" => reqwest::Method::POST,
            "GET" => reqwest::Method::GET,
            "HEAD" => reqwest::Method::HEAD,
            "DELETE" => reqwest::Method::DELETE,
            _ => reqwest::Method::GET,
        };

        let mut req_builder = self.client.request(method, &upstream_url);

        // Forward headers
        for line in String::from_utf8_lossy(header_part).lines().skip(1) {
            if let Some(colon) = line.find(':') {
                let name = line[..colon].trim();
                let val = line[colon + 1..].trim();
                let lower = name.to_ascii_lowercase();
                if lower != "host" && lower != "connection" {
                    req_builder = req_builder.header(name, val);
                }
            }
        }

        if !full_body.is_empty() {
            req_builder = req_builder.body(full_body);
        }

        // 3. Send upstream
        let upstream_resp = match req_builder.send().await {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!(
                    r#"{{"error":"Ollama upstream unreachable at {}: {}"}}"#,
                    self.config.ollama_url, e
                );
                let resp = format!(
                    "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    err_msg.len(), err_msg
                );
                stream.write_all(resp.as_bytes()).await.ok();
                return Ok(());
            }
        };

        // 4. Stream response back to client
        let status = upstream_resp.status();
        let mut header_str = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or("OK"));

        for (k, v) in upstream_resp.headers() {
            let key = k.as_str();
            if key.eq_ignore_ascii_case("connection") {
                continue;
            }
            if let Ok(v_str) = v.to_str() {
                header_str.push_str(&format!("{}: {}\r\n", key, v_str));
            }
        }
        header_str.push_str("X-Tagisan-Memory-Tuner: active\r\n\r\n");
        stream.write_all(header_str.as_bytes()).await.map_err(|e| {
            TagisanError::Execution(format!("Failed to write proxy response headers: {e}"))
        })?;

        let mut byte_stream = upstream_resp.bytes_stream();
        while let Some(chunk_res) = byte_stream.next().await {
            match chunk_res {
                Ok(chunk) => {
                    if let Err(e) = stream.write_all(&chunk).await {
                        debug!("[Memory Tuner] Client disconnected while streaming: {e}");
                        break;
                    }
                    let _ = stream.flush().await;
                }
                Err(e) => {
                    debug!("[Memory Tuner] Upstream stream error: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Query Ollama `/api/ps` for models currently loaded in RAM/VRAM
    pub async fn query_loaded_models(&self) -> Result<Vec<LoadedModelInfo>> {
        let ps_url = format!("{}/api/ps", self.config.ollama_url.trim_end_matches('/'));
        let resp = self.client.get(&ps_url).send().await?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let ps_data = resp.json::<OllamaPsResponse>().await.map_err(|e| {
            TagisanError::Execution(format!("Failed to parse /api/ps JSON response: {e}"))
        })?;

        let models = ps_data
            .models
            .unwrap_or_default()
            .into_iter()
            .map(|m| LoadedModelInfo {
                name: m.name,
                model: m.model,
                size_bytes: m.size.unwrap_or(0),
                size_vram_bytes: m.size_vram.unwrap_or(0),
                expires_at: m.expires_at,
            })
            .collect();

        Ok(models)
    }

    /// Unload a specific model immediately by sending `keep_alive: 0`
    pub async fn unload_model(&self, model_name: &str) -> Result<bool> {
        let unload_url = format!("{}/api/generate", self.config.ollama_url.trim_end_matches('/'));
        let payload = OllamaUnloadPayload {
            model: model_name,
            keep_alive: 0,
        };

        let resp = self.client.post(&unload_url).json(&payload).send().await?;
        if resp.status().is_success() {
            self.total_unloads.fetch_add(1, Ordering::Relaxed);
            Ok(true)
        } else {
            let err_text = resp.text().await.unwrap_or_default();
            Err(TagisanError::BadResponse("ollama".into(), format!("Unload failed: {err_text}")))
        }
    }

    /// Unload all currently resident models from VRAM/RAM
    pub async fn unload_all_models(&self) -> Result<Vec<String>> {
        let loaded = self.query_loaded_models().await?;
        let mut unloaded = Vec::new();

        for m in loaded {
            let target = if !m.model.is_empty() { &m.model } else { &m.name };
            if let Ok(true) = self.unload_model(target).await {
                unloaded.push(target.to_string());
            }
        }

        Ok(unloaded)
    }

    /// Check if upstream Ollama daemon is reachable and responding
    pub async fn is_ollama_reachable(&self) -> bool {
        let url = format!("{}/api/tags", self.config.ollama_url.trim_end_matches('/'));
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Retrieve full status summary of the Memory Tuner
    pub async fn status(&self) -> Result<MemoryTunerStatus> {
        let ollama_reachable = self.is_ollama_reachable().await;
        let (loaded, installed) = if ollama_reachable {
            let l = self.query_loaded_models().await.unwrap_or_default();
            let ins = self.fetch_installed_model_tags().await.map(|tags| {
                tags.into_iter().map(|t| t.name).collect()
            }).unwrap_or_default();
            (l, ins)
        } else {
            (Vec::new(), Vec::new())
        };
        let proxy_addr = format!("{}:{}", self.config.bind_host, self.config.bind_port);

        Ok(MemoryTunerStatus {
            is_running: self.is_running.load(Ordering::Acquire),
            proxy_addr,
            ollama_url: self.config.ollama_url.clone(),
            ollama_reachable,
            idle_duration_secs: self.idle_duration_secs(),
            inactivity_threshold_secs: self.config.inactivity_timeout_secs,
            installed_models: installed,
            active_models_in_vram: loaded,
            total_unloads: self.total_unloads.load(Ordering::Relaxed),
            total_proxied_requests: self.total_proxied.load(Ordering::Relaxed),
        })
    }

    /// Query all installed models from `/api/tags`
    pub async fn fetch_installed_model_tags(&self) -> Result<Vec<OllamaModelTagItem>> {
        let url = format!("{}/api/tags", self.config.ollama_url.trim_end_matches('/'));
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err));
        }
        let tags: OllamaTagsResponse = resp.json().await.map_err(|e| {
            TagisanError::Execution(format!("Failed to parse /api/tags response: {e}"))
        })?;
        Ok(tags.models.unwrap_or_default())
    }

    /// Query the exact byte size of a model from `/api/tags` or heuristic estimation
    pub async fn query_model_size_bytes(&self, model_name: &str) -> u64 {
        // 1. Try querying /api/tags
        if let Ok(models) = self.fetch_installed_model_tags().await {
            let clean = model_name.trim();
            for m in &models {
                if m.name.eq_ignore_ascii_case(clean)
                    || m.model.as_deref().unwrap_or("").eq_ignore_ascii_case(clean)
                    || m.name.split(':').next().unwrap_or("").eq_ignore_ascii_case(clean)
                {
                    if let Some(sz) = m.size {
                        if sz > 0 {
                            return sz;
                        }
                    }
                }
            }
        }

        // 2. Try disk manifests if local
        if let Ok(resolver) = std::panic::catch_unwind(|| crate::engine::OllamaBlobResolver::new(None)) {
            if let Ok(summary) = resolver.resolve(model_name) {
                if summary.size > 0 {
                    return summary.size;
                }
            }
        }

        // 3. Heuristic fallback based on parameter size in name
        Self::estimate_model_size_from_name(model_name)
    }

    /// Heuristic estimation of model size in bytes from name when live APIs are unreachable
    pub fn estimate_model_size_from_name(name: &str) -> u64 {
        let lower = name.to_ascii_lowercase();
        if lower.contains("70b") {
            40 * 1024 * 1024 * 1024 // ~40 GB
        } else if lower.contains("32b") || lower.contains("34b") {
            20 * 1024 * 1024 * 1024 // ~20 GB
        } else if lower.contains("14b") || lower.contains("13b") {
            9 * 1024 * 1024 * 1024 // ~9 GB
        } else if lower.contains("7b") || lower.contains("8b") {
            4800 * 1024 * 1024 // ~4.7 GB
        } else if lower.contains("3b") || lower.contains("4b") {
            2200 * 1024 * 1024 // ~2.2 GB
        } else if lower.contains("1.7b") || lower.contains("1.5b") || lower.contains("2b") {
            1500 * 1024 * 1024 // ~1.5 GB
        } else if lower.contains("1b") || lower.contains("0.5b") {
            1100 * 1024 * 1024 // ~1.1 GB
        } else {
            3800 * 1024 * 1024 // ~3.8 GB conservative default
        }
    }

    /// Proactively evict all currently loaded models from RAM/VRAM
    pub async fn evict_idle_models(&self) -> Result<Vec<String>> {
        self.unload_all_models().await
    }

    /// Pre-flight admission control check
    pub async fn check_admission(&self, model_name: &str, num_ctx: u32) -> Result<AdmissionVerdict> {
        let metrics = LinuxMemInfo::read_host();
        self.check_admission_with_metrics(model_name, num_ctx, &metrics).await
    }

    /// Admission check with explicit metrics (enables deterministic testing)
    pub async fn check_admission_with_metrics(
        &self,
        model_name: &str,
        num_ctx: u32,
        initial_metrics: &LinuxMemInfo,
    ) -> Result<AdmissionVerdict> {
        let model_bytes = self.query_model_size_bytes(model_name).await;
        let kv_cache_bytes = (num_ctx as u64) * 128 * 1024;
        let safety_buffer_bytes = ANTI_FREEZE_SAFETY_BUFFER_BYTES;
        let total_required = model_bytes + kv_cache_bytes + safety_buffer_bytes;

        let mut available_bytes = initial_metrics.mem_available_kb * 1024;
        let mut evicted = Vec::new();

        // 1. If required exceeds available, proactively evict idle models
        if total_required > available_bytes {
            debug!(
                "[AntiFreezeGuardian] Required RAM ({:.2} GB) > Available RAM ({:.2} GB). Proactively evicting resident models...",
                total_required as f64 / 1e9,
                available_bytes as f64 / 1e9
            );
            if let Ok(unloaded) = self.unload_all_models().await {
                if !unloaded.is_empty() {
                    info!(
                        "{}",
                        format!(
                            "🧹 [AntiFreezeGuardian] Proactively evicted {} resident model(s) [{}] to reclaim RAM.",
                            unloaded.len(),
                            unloaded.join(", ")
                        ).yellow().bold()
                    );
                    evicted = unloaded;
                    HostMemoryGovernor::new().trim_heap();
                    let updated = LinuxMemInfo::read_host();
                    if updated.mem_available_kb * 1024 > available_bytes {
                        available_bytes = updated.mem_available_kb * 1024;
                    }
                }
            }
        }

        // 2. Fits safely without swap thrashing?
        if total_required <= available_bytes {
            return Ok(AdmissionVerdict::Admitted {
                model_name: model_name.to_string(),
                required_bytes: total_required,
                available_bytes,
                evicted_models: evicted,
            });
        }

        // 3. Bypass via TAGISAN_FORCE_OLLAMA_LOAD=1
        if OllamaAdmissionController::is_force_load_enabled() {
            warn!(
                "{}",
                format!(
                    "⚠️  [AntiFreezeGuardian] TAGISAN_FORCE_OLLAMA_LOAD=1 bypass active! Model '{}' requires {:.2}GB RAM but only {:.2}GB available. Laptop may freeze due to swap thrashing!",
                    model_name,
                    total_required as f64 / 1e9,
                    available_bytes as f64 / 1e9
                ).red().bold()
            );
            return Ok(AdmissionVerdict::Bypassed {
                model_name: model_name.to_string(),
                required_bytes: total_required,
                available_bytes,
                reason: "TAGISAN_FORCE_OLLAMA_LOAD=1 environment variable set".to_string(),
            });
        }

        // 4. Auto-recover to installed lighter model that fits
        if let Ok(installed_tags) = self.fetch_installed_model_tags().await {
            let mut candidates: Vec<(String, u64)> = Vec::new();
            for item in installed_tags {
                let name = item.name.clone();
                if name.eq_ignore_ascii_case(model_name) {
                    continue;
                }
                let sz = item.size.unwrap_or_else(|| Self::estimate_model_size_from_name(&name));
                let cand_req = sz + kv_cache_bytes + safety_buffer_bytes;
                if cand_req <= available_bytes {
                    candidates.push((name, cand_req));
                }
            }

            candidates.sort_by_key(|c| c.1);

            if let Some((fallback, fb_req)) = candidates.first() {
                if OllamaAdmissionController::is_auto_recover_lighter_enabled() {
                    info!(
                        "{}",
                        format!(
                            "🛡️  [AntiFreezeGuardian Auto-Recovery] Model '{}' ({:.2} GB required) exceeds available RAM ({:.2} GB). Safely switching to installed lighter model '{}' ({:.2} GB required) to eliminate laptop freeze.",
                            model_name,
                            total_required as f64 / 1e9,
                            available_bytes as f64 / 1e9,
                            fallback,
                            *fb_req as f64 / 1e9
                        ).green().bold()
                    );
                    return Ok(AdmissionVerdict::RecoveredToLighterModel {
                        original_model: model_name.to_string(),
                        fallback_model: fallback.clone(),
                        required_bytes: *fb_req,
                        available_bytes,
                        evicted_models: evicted,
                    });
                }
            }
        }

        // 5. Reject with full diagnostic details and pull suggestions
        let suggested = vec![
            "smollm2:1.7b".to_string(),
            "llama3.2:1b".to_string(),
            "llama3.2:3b".to_string(),
        ];

        let diagnostics = format!(
            "ANTI-FREEZE GUARDIAN BLOCKED MODEL LOAD: Model '{}' requires ~{:.2} GB RAM (model: {:.2} GB, KV cache: {:.2} GB, safety buffer: 512 MB), but host only has {:.2} GB available RAM ({:.1}% free).\n\
            Loading this model on your hardware will trigger aggressive Linux kernel swap thrashing (kswapd0 at 100% I/O) and completely freeze your laptop UI, desktop, and terminal.\n\n\
            Recommended Solutions:\n\
              1. Pull and run a lightweight model tailored for your hardware:\n\
                 ▶ ollama pull smollm2:1.7b   (~1.0 GB RAM required - ultra fast)\n\
                 ▶ ollama pull llama3.2:1b    (~1.3 GB RAM required - compact reasoning)\n\
                 ▶ ollama pull llama3.2:3b    (~2.2 GB RAM required - high quality 3B)\n\
              2. Free system RAM by closing browser tabs or background processes.\n\
              3. If you really want to force loading despite freeze risk:\n\
                 ▶ export TAGISAN_FORCE_OLLAMA_LOAD=1",
            model_name,
            total_required as f64 / 1e9,
            model_bytes as f64 / 1e9,
            kv_cache_bytes as f64 / 1e9,
            available_bytes as f64 / 1e9,
            initial_metrics.available_pct()
        );

        Ok(AdmissionVerdict::Rejected {
            model_name: model_name.to_string(),
            model_bytes,
            kv_cache_bytes,
            safety_buffer_bytes,
            total_required_bytes: total_required,
            available_bytes,
            available_pct: initial_metrics.available_pct(),
            suggested_models: suggested,
            diagnostics,
        })
    }

    /// Post-inference memory hygiene:
    /// 1. Immediately call libc::malloc_trim(0) to return heap memory to Linux kernel.
    /// 2. Check if memory is RedCritical. If so, immediately unload the model with keep_alive: 0.
    pub async fn perform_post_inference_hygiene_with_metrics(
        &self,
        model_name: &str,
        metrics: &LinuxMemInfo,
    ) -> bool {
        let gov = HostMemoryGovernor::new();
        gov.trim_heap();

        let pressure = gov.evaluate_pressure(metrics);
        if pressure == MemoryPressureTier::RedCritical {
            warn!(
                "🚨 [AntiFreezeGuardian] Post-inference memory is RedCritical ({:.1}% available). Proactively evicting '{}' via keep_alive: 0...",
                metrics.available_pct(),
                model_name
            );
            let _ = self.unload_model(model_name).await;
            gov.trim_heap();
            true
        } else {
            false
        }
    }

    pub async fn perform_post_inference_hygiene(&self, model_name: &str) -> bool {
        let metrics = LinuxMemInfo::read_host();
        self.perform_post_inference_hygiene_with_metrics(model_name, &metrics).await
    }
}

/// Discovered model metadata from Ollama /api/tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelTagItem {
    pub name: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub details: Option<OllamaTagModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaTagModelDetails {
    #[serde(default)]
    pub parent_model: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub families: Option<Vec<String>>,
    #[serde(default)]
    pub parameter_size: Option<String>,
    #[serde(default)]
    pub quantization_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OllamaTagsResponse {
    pub models: Option<Vec<OllamaModelTagItem>>,
}

/// Result of Pre-Flight Memory Admission Check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdmissionVerdict {
    /// Safe to load and execute model
    Admitted {
        model_name: String,
        required_bytes: u64,
        available_bytes: u64,
        evicted_models: Vec<String>,
    },
    /// Oversized model was safely redirected to an installed lightweight model that fits
    RecoveredToLighterModel {
        original_model: String,
        fallback_model: String,
        required_bytes: u64,
        available_bytes: u64,
        evicted_models: Vec<String>,
    },
    /// User explicitly forced loading via TAGISAN_FORCE_OLLAMA_LOAD=1
    Bypassed {
        model_name: String,
        required_bytes: u64,
        available_bytes: u64,
        reason: String,
    },
    /// Model load rejected to prevent Linux swap thrashing and laptop lockup
    Rejected {
        model_name: String,
        model_bytes: u64,
        kv_cache_bytes: u64,
        safety_buffer_bytes: u64,
        total_required_bytes: u64,
        available_bytes: u64,
        available_pct: f64,
        suggested_models: Vec<String>,
        diagnostics: String,
    },
}

impl AdmissionVerdict {
    pub fn is_admitted(&self) -> bool {
        matches!(
            self,
            Self::Admitted { .. } | Self::RecoveredToLighterModel { .. } | Self::Bypassed { .. }
        )
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected { .. })
    }

    pub fn effective_model(&self) -> &str {
        match self {
            Self::Admitted { model_name, .. } => model_name,
            Self::RecoveredToLighterModel { fallback_model, .. } => fallback_model,
            Self::Bypassed { model_name, .. } => model_name,
            Self::Rejected { model_name, .. } => model_name,
        }
    }

    pub fn ensure_admitted(self) -> Result<String> {
        match self {
            Self::Admitted { model_name, .. } => Ok(model_name),
            Self::RecoveredToLighterModel { fallback_model, .. } => Ok(fallback_model),
            Self::Bypassed { model_name, .. } => Ok(model_name),
            Self::Rejected { diagnostics, .. } => {
                Err(TagisanError::ResourceExhausted(diagnostics))
            }
        }
    }
}

/// Standalone Pre-Flight Admission Controller / Anti-Freeze Guardian
#[derive(Clone, Debug)]
pub struct OllamaAdmissionController {
    tuner: Arc<OllamaMemoryTuner>,
}

pub type AntiFreezeGuardian = OllamaAdmissionController;

impl OllamaAdmissionController {
    pub fn new(tuner: Arc<OllamaMemoryTuner>) -> Self {
        Self { tuner }
    }

    pub fn default_local() -> Self {
        Self::new(OllamaMemoryTuner::global())
    }

    pub async fn check_admission(&self, model_name: &str, num_ctx: u32) -> Result<AdmissionVerdict> {
        self.tuner.check_admission(model_name, num_ctx).await
    }

    pub async fn check_admission_with_metrics(
        &self,
        model_name: &str,
        num_ctx: u32,
        metrics: &LinuxMemInfo,
    ) -> Result<AdmissionVerdict> {
        self.tuner.check_admission_with_metrics(model_name, num_ctx, metrics).await
    }

    pub fn is_force_load_enabled() -> bool {
        std::env::var("TAGISAN_FORCE_OLLAMA_LOAD")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    pub fn is_auto_recover_lighter_enabled() -> bool {
        std::env::var("TAGISAN_AUTO_RECOVER_LIGHTER_MODEL")
            .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tuner_config_defaults() {
        let config = MemoryTunerConfig::default();
        assert_eq!(config.bind_port, 11435);
        assert_eq!(config.inactivity_timeout_secs, 300);
        assert_eq!(config.watchdog_interval_secs, 5);
        assert!(config.ollama_url.contains("11434"));
    }

    #[test]
    fn test_inactivity_timer_tracking() {
        let tuner = OllamaMemoryTuner::default_local();
        assert_eq!(tuner.idle_duration_secs(), 0);

        tuner.touch_activity();
        assert_eq!(tuner.idle_duration_secs(), 0);
    }
}
