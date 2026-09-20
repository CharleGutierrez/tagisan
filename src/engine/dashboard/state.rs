//! Shared state and telemetry for the Tagisan Swarm & Computer-Use Web Dashboard.

use crate::governor::{HostMemoryGovernor, LinuxMemInfo, MemoryPressureTier};
use crate::tools::builtin::ReflexionEntry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Snapshot of the host system telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostTelemetrySnapshot {
    pub mem_total_mb: u64,
    pub mem_available_mb: u64,
    pub mem_used_pct: f64,
    pub swap_used_pct: f64,
    pub pressure_tier: String,
    pub cpu_cores: usize,
    pub timestamp_ms: u64,
}

impl HostTelemetrySnapshot {
    pub fn current() -> Self {
        let metrics = LinuxMemInfo::read_host();
        let gov = HostMemoryGovernor::new();
        let pressure = gov.evaluate_pressure(&metrics);
        let tier_str = match pressure {
            MemoryPressureTier::GreenNormal => "GreenNormal",
            MemoryPressureTier::YellowWarning => "YellowWarning",
            MemoryPressureTier::RedCritical => "RedCritical",
        };

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            mem_total_mb: metrics.mem_total_kb / 1024,
            mem_available_mb: metrics.mem_available_kb / 1024,
            mem_used_pct: 100.0 - metrics.available_pct(),
            swap_used_pct: metrics.swap_used_pct(),
            pressure_tier: tier_str.to_string(),
            cpu_cores: crate::governor::host_num_cpus(),
            timestamp_ms: now_ms,
        }
    }
}

/// Swarm task node execution status for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNodeStatus {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub duration_ms: u64,
}

/// Central dashboard state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardState {
    pub latest_screen_base64: Option<String>,
    pub latest_screen_timestamp_ms: u64,
    pub active_swarm_nodes: Vec<SwarmNodeStatus>,
    pub recent_reflexions: Vec<ReflexionEntry>,
    pub last_telemetry: Option<HostTelemetrySnapshot>,
}

/// Thread-safe handle to DashboardState
#[derive(Clone)]
pub struct SharedDashboardState {
    inner: Arc<RwLock<DashboardState>>,
}

impl Default for SharedDashboardState {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedDashboardState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(DashboardState::default())),
        }
    }

    pub async fn update_screen_frame(&self, base64_png: String) {
        let mut guard = self.inner.write().await;
        guard.latest_screen_base64 = Some(base64_png);
        guard.latest_screen_timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }

    pub async fn update_telemetry(&self) -> HostTelemetrySnapshot {
        let snap = HostTelemetrySnapshot::current();
        let mut guard = self.inner.write().await;
        guard.last_telemetry = Some(snap.clone());
        snap
    }

    pub async fn set_swarm_nodes(&self, nodes: Vec<SwarmNodeStatus>) {
        let mut guard = self.inner.write().await;
        guard.active_swarm_nodes = nodes;
    }

    pub async fn set_reflexions(&self, reflexions: Vec<ReflexionEntry>) {
        let mut guard = self.inner.write().await;
        guard.recent_reflexions = reflexions;
    }

    pub async fn get_snapshot(&self) -> DashboardState {
        self.inner.read().await.clone()
    }
}
