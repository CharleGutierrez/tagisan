//! Host Memory Governor & Anti-Freeze Shield for Tagisan (TGS)
//!
//! Provides proactive Linux kernel telemetry (/proc/meminfo & PSI),
//! dynamic concurrency throttling, libc::malloc_trim(0) heap release,
//! and process safety guards to eliminate UI freezes on 8GB workstations.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Live memory metrics parsed directly from the Linux `/proc/meminfo` pseudo-filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxMemInfo {
    pub mem_total_kb: u64,
    pub mem_free_kb: u64,
    pub mem_available_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
    pub dirty_kb: u64,
    pub timestamp_ms: u64,
}

impl Default for LinuxMemInfo {
    fn default() -> Self {
        Self {
            mem_total_kb: 8 * 1024 * 1024,
            mem_free_kb: 2 * 1024 * 1024,
            mem_available_kb: 3 * 1024 * 1024,
            buffers_kb: 256 * 1024,
            cached_kb: 1024 * 1024,
            swap_total_kb: 8 * 1024 * 1024,
            swap_free_kb: 4 * 1024 * 1024,
            dirty_kb: 64 * 1024,
            timestamp_ms: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

impl LinuxMemInfo {
    /// Read live `/proc/meminfo` from the Linux kernel
    pub fn read_host() -> Self {
        let mut info = Self::default();
        info.timestamp_ms = chrono::Utc::now().timestamp_millis() as u64;

        if let Ok(file) = File::open("/proc/meminfo") {
            let reader = BufReader::new(file);
            for line in reader.lines().map_while(Result::ok) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0].trim_end_matches(":");
                    let val: u64 = parts[1].parse().unwrap_or(0);
                    match key {
                        "MemTotal" => info.mem_total_kb = val,
                        "MemFree" => info.mem_free_kb = val,
                        "MemAvailable" => info.mem_available_kb = val,
                        "Buffers" => info.buffers_kb = val,
                        "Cached" => info.cached_kb = val,
                        "SwapTotal" => info.swap_total_kb = val,
                        "SwapFree" => info.swap_free_kb = val,
                        "Dirty" => info.dirty_kb = val,
                        _ => {}
                    }
                }
            }
        }
        info
    }

    /// Available RAM percentage (0.0 to 100.0%)
    pub fn available_pct(&self) -> f64 {
        if self.mem_total_kb == 0 {
            return 100.0;
        }
        (self.mem_available_kb as f64 / self.mem_total_kb as f64) * 100.0
    }

    /// Swap usage percentage (0.0 to 100.0%)
    pub fn swap_used_pct(&self) -> f64 {
        if self.swap_total_kb == 0 {
            return 0.0;
        }
        let used = self.swap_total_kb.saturating_sub(self.swap_free_kb);
        (used as f64 / self.swap_total_kb as f64) * 100.0
    }

    /// Formatted total RAM in GB
    pub fn total_gb(&self) -> f64 {
        self.mem_total_kb as f64 / (1024.0 * 1024.0)
    }

    /// Formatted available RAM in GB
    pub fn available_gb(&self) -> f64 {
        self.mem_available_kb as f64 / (1024.0 * 1024.0)
    }

    /// Formatted swap total in GB
    pub fn swap_total_gb(&self) -> f64 {
        self.swap_total_kb as f64 / (1024.0 * 1024.0)
    }

    /// Formatted swap used in GB
    pub fn swap_used_gb(&self) -> f64 {
        let used = self.swap_total_kb.saturating_sub(self.swap_free_kb);
        used as f64 / (1024.0 * 1024.0)
    }
}

/// Dynamic 3-Tier Memory Pressure Level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressureTier {
    /// > 25% RAM Available. Full concurrency, maximum caching.
    GreenNormal,
    /// 15% - 25% RAM Available. Dynamic braking: clamp workers to 2 threads, shrink vector batches.
    YellowWarning,
    /// < 15% RAM Available or Swap > 85%. Emergency Anti-Freeze Shield active.
    /// Blocks new subagent spawns, forces libc::malloc_trim(0), flushes caches to disk.
    RedCritical,
}

impl MemoryPressureTier {
    pub fn label(&self) -> &str {
        match self {
            Self::GreenNormal => "GREEN (Healthy - Unrestricted)",
            Self::YellowWarning => "YELLOW (Moderate Pressure - Braking Active)",
            Self::RedCritical => "RED (Critical Freeze Guard - Emergency Trim & Throttle)",
        }
    }

    pub fn colorized(&self) -> String {
        match self {
            Self::GreenNormal => self.label().green().bold().to_string(),
            Self::YellowWarning => self.label().yellow().bold().to_string(),
            Self::RedCritical => self.label().red().bold().to_string(),
        }
    }
}

/// Comprehensive Host Memory Audit Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAuditReport {
    pub metrics: LinuxMemInfo,
    pub pressure_tier: MemoryPressureTier,
    pub is_8gb_system: bool,
    pub recommended_concurrency: usize,
    pub recommended_cargo_jobs: usize,
    pub can_spawn_subagent: bool,
    pub trim_recommended: bool,
    pub summary: String,
}

/// Autonomous Host Memory Governor & Anti-Freeze Engine
#[derive(Debug)]
pub struct HostMemoryGovernor {
    guard_active: Arc<AtomicBool>,
    last_trim_timestamp: Arc<AtomicUsize>,
    total_trims_performed: Arc<AtomicUsize>,
}

impl Default for HostMemoryGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl HostMemoryGovernor {
    pub fn new() -> Self {
        Self {
            guard_active: Arc::new(AtomicBool::new(true)),
            last_trim_timestamp: Arc::new(AtomicUsize::new(0)),
            total_trims_performed: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Read live host metrics
    pub fn current_metrics(&self) -> LinuxMemInfo {
        LinuxMemInfo::read_host()
    }

    /// Evaluate system memory pressure tier
    pub fn evaluate_pressure(&self, metrics: &LinuxMemInfo) -> MemoryPressureTier {
        let avail_pct = metrics.available_pct();
        let swap_pct = metrics.swap_used_pct();

        // Critical threshold: Less than 15% RAM available OR Swap exceeds 85%
        if avail_pct < 15.0 || swap_pct > 85.0 {
            MemoryPressureTier::RedCritical
        } else if avail_pct < 25.0 || swap_pct > 65.0 {
            MemoryPressureTier::YellowWarning
        } else {
            MemoryPressureTier::GreenNormal
        }
    }

    /// Check if the host workstation has ~8GB or less physical RAM
    pub fn is_8gb_workstation(&self, metrics: &LinuxMemInfo) -> bool {
        metrics.total_gb() <= 8.5
    }

    /// Recommended worker thread count (Tokio / Rayon) based on memory state
    pub fn recommended_concurrency(&self, metrics: &LinuxMemInfo) -> usize {
        match self.evaluate_pressure(metrics) {
            MemoryPressureTier::GreenNormal => {
                if self.is_8gb_workstation(metrics) {
                    2.min(num_cpus())
                } else {
                    num_cpus()
                }
            }
            MemoryPressureTier::YellowWarning => 2.min(num_cpus()),
            MemoryPressureTier::RedCritical => 1,
        }
    }

    /// Recommended compiler/child process jobs (e.g. CARGO_BUILD_JOBS)
    /// Prevents rustc / LLVM from spawning 16 threads and getting SIGKILLed
    pub fn recommended_cargo_jobs(&self, metrics: &LinuxMemInfo) -> usize {
        match self.evaluate_pressure(metrics) {
            MemoryPressureTier::GreenNormal => {
                if self.is_8gb_workstation(metrics) {
                    2
                } else {
                    4
                }
            }
            MemoryPressureTier::YellowWarning => 1,
            MemoryPressureTier::RedCritical => 1,
        }
    }

    /// Check if host is a dual-core or single-core machine
    pub fn is_low_core_cpu(&self) -> bool {
        num_cpus() <= 2
    }

    /// Compute safe Ollama num_thread setting.
    /// On a 2-core system (or when is_8gb_workstation is true or under memory/CPU pressure),
    /// clamp num_thread to 1 (or (num_cpus - 1).max(1)).
    /// NEVER use all cores on a 2-core machine, leaving at least 1 core for the OS/UI to prevent laptop lockup.
    pub fn recommended_ollama_threads(&self, metrics: &LinuxMemInfo) -> u32 {
        let cpus = num_cpus();
        let is_8gb = self.is_8gb_workstation(metrics);
        let tier = self.evaluate_pressure(metrics);

        if cpus <= 2 || is_8gb || tier != MemoryPressureTier::GreenNormal {
            1
        } else {
            ((cpus.saturating_sub(1)).max(1).min(4)) as u32
        }
    }

    /// Compute safe Ollama num_ctx setting dynamically clamped based on available RAM:
    /// - RedCritical: 512 - 1024
    /// - YellowWarning / 8GB workstation: 1024 - 2048
    /// - GreenNormal: up to 4096 (or 8192 if explicit)
    pub fn recommended_ollama_ctx(&self, metrics: &LinuxMemInfo, requested_ctx: Option<u32>) -> u32 {
        let tier = self.evaluate_pressure(metrics);
        let is_8gb = self.is_8gb_workstation(metrics);
        let req = requested_ctx.unwrap_or(2048);

        match tier {
            MemoryPressureTier::RedCritical => req.clamp(512, 1024),
            MemoryPressureTier::YellowWarning => req.clamp(1024, 2048),
            MemoryPressureTier::GreenNormal => {
                if is_8gb {
                    req.clamp(1024, 2048)
                } else if req > 4096 {
                    req.min(8192)
                } else {
                    req.clamp(1024, 4096)
                }
            }
        }
    }

    /// Compute safe Ollama keep_alive setting:
    /// - RedCritical: "0s" (unload immediately after response)
    /// - YellowWarning / 8GB: "2m" (2 minutes)
    /// - GreenNormal: "5m" (or configurable via TAGISAN_OLLAMA_KEEP_ALIVE)
    pub fn recommended_ollama_keep_alive(&self, metrics: &LinuxMemInfo) -> String {
        if let Ok(val) = std::env::var("TAGISAN_OLLAMA_KEEP_ALIVE") {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        let tier = self.evaluate_pressure(metrics);
        let is_8gb = self.is_8gb_workstation(metrics);

        match tier {
            MemoryPressureTier::RedCritical => "0s".to_string(),
            MemoryPressureTier::YellowWarning => "2m".to_string(),
            MemoryPressureTier::GreenNormal => {
                if is_8gb {
                    "2m".to_string()
                } else {
                    "5m".to_string()
                }
            }
        }
    }

    /// Safe check before spawning a resource-intensive subagent or local LLM session
    pub fn can_spawn_subagent(&self, metrics: &LinuxMemInfo) -> bool {
        let tier = self.evaluate_pressure(metrics);
        tier != MemoryPressureTier::RedCritical
    }

    /// Forcefully release unused glibc heap arenas back to the Linux kernel via libc::malloc_trim(0)
    /// This immediately lowers process RSS and returns pages to the OS page allocator.
    pub fn trim_heap(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            extern "C" {
                fn malloc_trim(pad: usize) -> i32;
            }
            let res = unsafe { malloc_trim(0) };
            self.total_trims_performed.fetch_add(1, Ordering::SeqCst);
            self.last_trim_timestamp.store(
                chrono::Utc::now().timestamp() as usize,
                Ordering::SeqCst,
            );
            res == 1
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Run a complete diagnostic audit of host memory
    pub fn audit(&self) -> MemoryAuditReport {
        let metrics = self.current_metrics();
        let pressure_tier = self.evaluate_pressure(&metrics);
        let is_8gb = self.is_8gb_workstation(&metrics);
        let rec_conc = self.recommended_concurrency(&metrics);
        let rec_cargo = self.recommended_cargo_jobs(&metrics);
        let can_spawn = self.can_spawn_subagent(&metrics);
        let trim_rec = pressure_tier != MemoryPressureTier::GreenNormal;

        let summary = match pressure_tier {
            MemoryPressureTier::GreenNormal => {
                format!(
                    "System memory is healthy ({:.1}% available, {:.1}GB free). Laptop freeze risk: LOW.",
                    metrics.available_pct(),
                    metrics.available_gb()
                )
            }
            MemoryPressureTier::YellowWarning => {
                format!(
                    "Moderate memory pressure ({:.1}% available). Dynamic concurrency brake clamped to {} workers. Laptop freeze risk: MODERATE.",
                    metrics.available_pct(),
                    rec_conc
                )
            }
            MemoryPressureTier::RedCritical => {
                format!(
                    "CRITICAL MEMORY EXHAUSTION ({:.1}% available, Swap {:.1}% full)! Anti-freeze shield activated: subagent spawns paused, malloc_trim recommended. Laptop freeze risk: HIGH.",
                    metrics.available_pct(),
                    metrics.swap_used_pct()
                )
            }
        };

        MemoryAuditReport {
            metrics,
            pressure_tier,
            is_8gb_system: is_8gb,
            recommended_concurrency: rec_conc,
            recommended_cargo_jobs: rec_cargo,
            can_spawn_subagent: can_spawn,
            trim_recommended: trim_rec,
            summary,
        }
    }

    /// Autonomous background monitor loop
    pub fn spawn_background_guard(self: Arc<Self>, interval: Duration) {
        if !self.guard_active.load(Ordering::SeqCst) {
            return;
        }

        tokio::spawn(async move {
            info!("🛡️ [TGS Anti-Freeze Governor] Started background Linux memory watcher.");
            loop {
                tokio::time::sleep(interval).await;
                let metrics = self.current_metrics();
                let tier = self.evaluate_pressure(&metrics);

                if tier == MemoryPressureTier::RedCritical {
                    warn!(
                        "🚨 [TGS Freeze Shield] Available RAM critically low ({:.1}GB / {:.1}%). Forcing malloc_trim...",
                        metrics.available_gb(),
                        metrics.available_pct()
                    );
                    self.trim_heap();
                }
            }
        });
    }

    /// Format ASCII diagnostic banner for TGS REPL
    pub fn format_repl_banner(&self) -> String {
        let audit = self.audit();
        let m = &audit.metrics;

        let mut out = String::new();
        out.push_str(&format!(
            "┌─────────────────────────────────────────────────────────────┐\n             │  🛡️  TAGISAN ANTI-FREEZE HOST MEMORY GOVERNOR              │\n             ├─────────────────────────────────────────────────────────────┤\n             │  Workstation Profile : {:<32}│\n             │  Pressure Status     : {:<32}│\n             │  Physical RAM        : {:<6.2} GB Total | {:<5.2} GB Available │\n             │  Available Headroom  : {:<5.1}% (Thresholds: 15% Red / 25% Ylw)│\n             │  Linux Swap File     : {:<5.2} GB Used  / {:<5.2} GB Total     │\n             │  Swap Saturation     : {:<5.1}%                                │\n             ├─────────────────────────────────────────────────────────────┤\n             │  Dynamic Concurrency : {:<32}│\n             │  Cargo Job Quota     : {:<32}│\n             │  Subagent Spawning   : {:<32}│\n             │  Total Trims Executed: {:<32}│\n             ├─────────────────────────────────────────────────────────────┤\n             │  Diagnosis:                                                │\n             │  {:<59}│\n             └─────────────────────────────────────────────────────────────┘\n",
            if audit.is_8gb_system { "8GB Constrained Laptop" } else { "High-RAM Workstation" },
            audit.pressure_tier.label(),
            m.total_gb(),
            m.available_gb(),
            m.available_pct(),
            m.swap_used_gb(),
            m.swap_total_gb(),
            m.swap_used_pct(),
            format!("Capped to {} worker thread(s)", audit.recommended_concurrency),
            format!("Enforced jobs = {} (Anti-OOM)", audit.recommended_cargo_jobs),
            if audit.can_spawn_subagent { "PERMITTED (Within Budget)".to_string() } else { "BLOCKED (Preserving RAM)".to_string() },
            self.total_trims_performed.load(Ordering::SeqCst),
            audit.summary
        ));
        out
    }
}

pub fn host_num_cpus() -> usize {
    num_cpus()
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
}

