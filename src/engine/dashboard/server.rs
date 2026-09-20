//! Embedded HTTP/WebSocket server for the Tagisan Swarm & Computer-Use Web Dashboard.

use super::state::{HostTelemetrySnapshot, SharedDashboardState};
use super::ws::{compute_accept_key, WebSocketBroadcaster};
use crate::error::{Result, TagisanError};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Tagisan Swarm & Computer-Use Dashboard</title>
  <style>
    :root {
      --bg: #090d16;
      --card-bg: #111827;
      --border: #1f2937;
      --accent: #3b82f6;
      --accent-glow: #60a5fa;
      --text: #f3f4f6;
      --text-muted: #9ca3af;
      --green: #10b981;
      --yellow: #f59e0b;
      --red: #ef4444;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }
    header {
      background: var(--card-bg);
      border-bottom: 1px solid var(--border);
      padding: 1rem 2rem;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .brand { font-size: 1.25rem; font-weight: 700; color: var(--accent-glow); letter-spacing: 0.05em; }
    .badge {
      font-size: 0.75rem;
      padding: 0.25rem 0.75rem;
      border-radius: 9999px;
      font-weight: 600;
      background: #1e3a8a;
      color: #bfdbfe;
    }
    main {
      padding: 2rem;
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 1.5rem;
      flex: 1;
    }
    .card {
      background: var(--card-bg);
      border: 1px solid var(--border);
      border-radius: 0.75rem;
      padding: 1.5rem;
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }
    .card-title {
      font-size: 1rem;
      font-weight: 600;
      color: var(--text);
      border-bottom: 1px solid var(--border);
      padding-bottom: 0.5rem;
    }
    .screen-view {
      width: 100%;
      height: 320px;
      background: #000;
      border-radius: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      overflow: hidden;
    }
    .screen-view img { max-width: 100%; max-height: 100%; object-fit: contain; }
    .telemetry-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 1rem;
    }
    .stat-box {
      background: rgba(255,255,255,0.03);
      padding: 1rem;
      border-radius: 0.5rem;
      border: 1px solid var(--border);
    }
    .stat-label { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; }
    .stat-value { font-size: 1.5rem; font-weight: 700; margin-top: 0.25rem; }
    .status-dot {
      display: inline-block;
      width: 8px;
      height: 8px;
      border-radius: 50%;
      margin-right: 6px;
      background: var(--green);
    }
  </style>
</head>
<body>
  <header>
    <div class="brand">TAGISAN // SWARM ASTRA DASHBOARD</div>
    <div id="connection-status" class="badge"><span class="status-dot"></span>CONNECTING...</div>
  </header>
  <main>
    <div class="card">
      <div class="card-title">Astra Live Computer-Use Screen Perception</div>
      <div class="screen-view">
        <img id="screen-feed" alt="Awaiting Astra Screen Frame..." src="">
      </div>
    </div>
    <div class="card">
      <div class="card-title">Host Telemetry & Memory Governor</div>
      <div class="telemetry-grid">
        <div class="stat-box">
          <div class="stat-label">RAM Available</div>
          <div class="stat-value" id="stat-ram">-- MB</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">Memory Pressure Tier</div>
          <div class="stat-value" id="stat-tier">--</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">CPU Cores Active</div>
          <div class="stat-value" id="stat-cpu">--</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">Swap Usage</div>
          <div class="stat-value" id="stat-swap">--%</div>
        </div>
      </div>
    </div>
  </main>
  <script>
    const wsProto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProto}//${location.host}/ws`;
    let ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      document.getElementById('connection-status').innerHTML = '<span class="status-dot" style="background:var(--green)"></span>ONLINE (7420)';
    };

    ws.onclose = () => {
      document.getElementById('connection-status').innerHTML = '<span class="status-dot" style="background:var(--red)"></span>OFFLINE';
      setTimeout(() => { location.reload(); }, 3000);
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'screen_update' && data.base64) {
          document.getElementById('screen-feed').src = 'data:image/png;base64,' + data.base64;
        }
        if (data.type === 'telemetry') {
          document.getElementById('stat-ram').innerText = data.mem_available_mb + ' MB';
          document.getElementById('stat-tier').innerText = data.pressure_tier;
          document.getElementById('stat-cpu').innerText = data.cpu_cores;
          document.getElementById('stat-swap').innerText = data.swap_used_pct.toFixed(1) + '%';
        }
      } catch (e) {}
    };
  </script>
</body>
</html>"#;

/// Running instance of the Dashboard Server
pub struct SwarmDashboardServer {
    pub addr: SocketAddr,
    pub port: u16,
    shutdown_token: CancellationToken,
    broadcaster: WebSocketBroadcaster,
    state: SharedDashboardState,
}

impl SwarmDashboardServer {
    pub async fn start(host: &str, port: u16, state: SharedDashboardState) -> Result<Self> {
        let bind_addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&bind_addr).await.map_err(TagisanError::Io)?;
        let local_addr = listener.local_addr().map_err(TagisanError::Io)?;
        let actual_port = local_addr.port();

        let shutdown_token = CancellationToken::new();
        let broadcaster = WebSocketBroadcaster::new();

        let shutdown_clone = shutdown_token.clone();
        let broadcaster_clone = broadcaster.clone();
        let state_clone = state.clone();

        // 1. Connection accept loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_clone.cancelled() => break,
                    Ok((socket, _)) = listener.accept() => {
                        let state = state_clone.clone();
                        let broadcaster = broadcaster_clone.clone();

                        tokio::spawn(async move {
                            let (read_half, mut write_half) = socket.into_split();
                            let mut reader = BufReader::new(read_half);
                            let mut req_line = String::new();
                            if reader.read_line(&mut req_line).await.is_err() || req_line.is_empty() {
                                return;
                            }

                            let parts: Vec<&str> = req_line.split_whitespace().collect();
                            if parts.len() < 2 {
                                return;
                            }
                            let method = parts[0];
                            let path = parts[1];

                            let mut headers = std::collections::HashMap::new();
                            loop {
                                let mut line = String::new();
                                if reader.read_line(&mut line).await.is_err() || line == "\r\n" || line.is_empty() {
                                    break;
                                }
                                if let Some((k, v)) = line.split_once(':') {
                                    headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
                                }
                            }

                            // WebSocket upgrade
                            if path == "/ws" || headers.get("upgrade").map(|s| s.eq_ignore_ascii_case("websocket")).unwrap_or(false) {
                                if let Some(sec_key) = headers.get("sec-websocket-key") {
                                    let accept_key = compute_accept_key(sec_key);
                                    let handshake = format!(
                                        "HTTP/1.1 101 Switching Protocols\r\n\
                                        Upgrade: websocket\r\n\
                                        Connection: Upgrade\r\n\
                                        Sec-WebSocket-Accept: {}\r\n\r\n",
                                        accept_key
                                    );
                                    if write_half.write_all(handshake.as_bytes()).await.is_ok() && write_half.flush().await.is_ok() {
                                        broadcaster.add_client(write_half).await;
                                    }
                                    return;
                                }
                            }

                            // HTTP endpoints
                            let (status, content_type, body) = match (method, path) {
                                ("GET", "/") => (200, "text/html; charset=utf-8", DASHBOARD_HTML.as_bytes().to_vec()),
                                ("GET", "/api/state") => {
                                    let snap = state.get_snapshot().await;
                                    (200, "application/json", serde_json::to_vec(&snap).unwrap_or_default())
                                }
                                ("GET", "/api/telemetry") => {
                                    let snap = HostTelemetrySnapshot::current();
                                    (200, "application/json", serde_json::to_vec(&snap).unwrap_or_default())
                                }
                                ("GET", "/api/reflexions") => {
                                    let reflexions = std::fs::read_to_string(".tagisan/reflexions.json")
                                        .unwrap_or_else(|_| "[]".to_string());
                                    (200, "application/json", reflexions.into_bytes())
                                }
                                _ => (404, "text/plain", b"Not Found".to_vec()),
                            };

                            let resp = format!(
                                "HTTP/1.1 {}\r\n\
                                Content-Type: {}\r\n\
                                Content-Length: {}\r\n\
                                Connection: close\r\n\r\n",
                                if status == 200 { "200 OK" } else { "404 Not Found" },
                                content_type,
                                body.len()
                            );
                            let _ = write_half.write_all(resp.as_bytes()).await;
                            let _ = write_half.write_all(&body).await;
                            let _ = write_half.flush().await;
                        });
                    }
                }
            }
        });

        // 2. Periodic telemetry broadcast loop (every 2 seconds)
        let broadcaster_bg = broadcaster.clone();
        let shutdown_bg = shutdown_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                tokio::select! {
                    _ = shutdown_bg.cancelled() => break,
                    _ = interval.tick() => {
                        let snap = HostTelemetrySnapshot::current();
                        let msg = json!({
                            "type": "telemetry",
                            "mem_available_mb": snap.mem_available_mb,
                            "pressure_tier": snap.pressure_tier,
                            "cpu_cores": snap.cpu_cores,
                            "swap_used_pct": snap.swap_used_pct,
                            "timestamp_ms": snap.timestamp_ms
                        });
                        broadcaster_bg.broadcast_json(&msg).await;
                    }
                }
            }
        });

        Ok(Self {
            addr: local_addr,
            port: actual_port,
            shutdown_token,
            broadcaster,
            state,
        })
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }

    pub fn broadcaster(&self) -> &WebSocketBroadcaster {
        &self.broadcaster
    }
}
