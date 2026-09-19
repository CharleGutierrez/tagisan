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
        Self {
            bind_host: "127.0.0.1".to_string(),
            bind_port: DEFAULT_TUNER_PORT,
            ollama_url: DEFAULT_OLLAMA_URL.to_string(),
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
#[derive(Clone)]
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
        let loaded = if ollama_reachable {
            self.query_loaded_models().await.unwrap_or_default()
        } else {
            Vec::new()
        };
        let proxy_addr = format!("{}:{}", self.config.bind_host, self.config.bind_port);

        Ok(MemoryTunerStatus {
            is_running: self.is_running.load(Ordering::Acquire),
            proxy_addr,
            ollama_url: self.config.ollama_url.clone(),
            ollama_reachable,
            idle_duration_secs: self.idle_duration_secs(),
            inactivity_threshold_secs: self.config.inactivity_timeout_secs,
            active_models_in_vram: loaded,
            total_unloads: self.total_unloads.load(Ordering::Relaxed),
            total_proxied_requests: self.total_proxied.load(Ordering::Relaxed),
        })
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
