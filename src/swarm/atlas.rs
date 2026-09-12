use crate::error::{Result, TagisanError};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Topic cluster classification for empirical semantic affinity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopicCluster {
    Systems,
    Forensics,
    Strategy,
    Web,
    Security,
    DevOps,
    Data,
    General,
}

impl TopicCluster {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Systems => "Systems [SYS]",
            Self::Forensics => "Forensics [FOR]",
            Self::Strategy => "Strategy [STR]",
            Self::Web => "Web [WEB]",
            Self::Security => "Security [SEC]",
            Self::DevOps => "DevOps [OPS]",
            Self::Data => "Data [DAT]",
            Self::General => "General [GEN]",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Systems => "SYS",
            Self::Forensics => "FOR",
            Self::Strategy => "STR",
            Self::Web => "WEB",
            Self::Security => "SEC",
            Self::DevOps => "OPS",
            Self::Data => "DAT",
            Self::General => "GEN",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("sys") || lower.contains("kernel") || lower.contains("rust") || lower.contains("io") || lower.contains("tokio") {
            Self::Systems
        } else if lower.contains("forensic") || lower.contains("audit") || lower.contains("debug") || lower.contains("error") || lower.contains("diagnostic") {
            Self::Forensics
        } else if lower.contains("strat") || lower.contains("plan") || lower.contains("architect") || lower.contains("moa") || lower.contains("debate") {
            Self::Strategy
        } else if lower.contains("web") || lower.contains("http") || lower.contains("bun") || lower.contains("ts") || lower.contains("rest") {
            Self::Web
        } else if lower.contains("sec") || lower.contains("shield") || lower.contains("vuln") || lower.contains("crypto") || lower.contains("fhe") {
            Self::Security
        } else if lower.contains("ops") || lower.contains("deploy") || lower.contains("cluster") || lower.contains("mesh") || lower.contains("p2p") {
            Self::DevOps
        } else if lower.contains("dat") || lower.contains("vector") || lower.contains("sql") || lower.contains("embed") || lower.contains("memory") {
            Self::Data
        } else {
            Self::General
        }
    }

    /// Empirical topic affinity clustering given an entity's name, description, and triggers
    pub fn infer(name: &str, description: &str, triggers: &[String]) -> Self {
        let combined = format!(
            "{} {} {}",
            name.to_lowercase(),
            description.to_lowercase(),
            triggers.join(" ").to_lowercase()
        );

        if combined.contains("security") || combined.contains("shield") || combined.contains("vulnerability") || combined.contains("auth") || combined.contains("crypto") || combined.contains("tfhe") {
            Self::Security
        } else if combined.contains("forensic") || combined.contains("debug") || combined.contains("trace") || combined.contains("diagnostic") || combined.contains("compiler") || combined.contains("error") {
            Self::Forensics
        } else if combined.contains("low-level") || combined.contains("concurrency") || combined.contains("thread") || combined.contains("kernel") || combined.contains("io") || combined.contains("tokio") || combined.contains("memory") {
            Self::Systems
        } else if combined.contains("architect") || combined.contains("planner") || combined.contains("strategy") || combined.contains("lead") || combined.contains("consensus") || combined.contains("debate") {
            Self::Strategy
        } else if combined.contains("http") || combined.contains("bun") || combined.contains("api") || combined.contains("frontend") || combined.contains("typescript") || combined.contains("server") {
            Self::Web
        } else if combined.contains("docker") || combined.contains("cluster") || combined.contains("mesh") || combined.contains("p2p") || combined.contains("deploy") || combined.contains("ci") {
            Self::DevOps
        } else if combined.contains("vector") || combined.contains("database") || combined.contains("sqlite") || combined.contains("pgvector") || combined.contains("embedding") {
            Self::Data
        } else {
            Self::General
        }
    }
}

/// Routing statistics and heat profile for an agent or skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasEntry {
    pub name: String,
    pub is_agent: bool,
    pub cluster: String,
    pub invocations: u64,
    pub successes: u64,
    pub failures: u64,
    pub total_latency_ms: u64,
    pub total_tokens: u64,
    pub heat_score: f64,
    pub last_invoked_epoch_secs: u64,
    pub tier: String,
}

impl AtlasEntry {
    pub fn new(name: impl Into<String>, is_agent: bool, cluster: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            name: name.into(),
            is_agent,
            cluster: cluster.to_string(),
            invocations: 0,
            successes: 0,
            failures: 0,
            total_latency_ms: 0,
            total_tokens: 0,
            heat_score: 0.0,
            last_invoked_epoch_secs: now,
            tier: "L3 Cold".to_string(),
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            100.0
        } else {
            (self.successes as f64 / self.invocations as f64) * 100.0
        }
    }

    pub fn avg_latency_ms(&self) -> u64 {
        if self.invocations == 0 {
            0
        } else {
            self.total_latency_ms / self.invocations
        }
    }

    pub fn update_heat(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let elapsed_secs = now.saturating_sub(self.last_invoked_epoch_secs);

        // Exponential time decay: half-life of 2 hours (7200 seconds)
        let decay = (-1.0 * (elapsed_secs as f64) / 7200.0).exp();
        let activity = (self.invocations as f64 * 4.0) + (self.success_rate() * 0.2);
        self.heat_score = (activity * decay).clamp(0.0, 100.0);

        if self.heat_score >= 60.0 {
            self.tier = "L1 Active".to_string();
        } else if self.heat_score >= 20.0 {
            self.tier = "L2 Warm".to_string();
        } else {
            self.tier = "L3 Cold".to_string();
        }
    }
}

/// Swarm Cortex & Heat Atlas tracking routing frequency, topic clustering, and tier status
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmAtlas {
    pub entries: HashMap<String, AtlasEntry>,
    pub total_routes: u64,
    pub last_updated_epoch_secs: u64,
}

impl SwarmAtlas {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            total_routes: 0,
            last_updated_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Default file path: .tagisan/swarm_heat.json
    pub fn default_path() -> PathBuf {
        PathBuf::from(".tagisan/swarm_heat.json")
    }

    /// Load from disk or initialize default
    pub fn load_or_default(path: Option<&Path>) -> Self {
        let p = path.map(PathBuf::from).unwrap_or_else(Self::default_path);
        if p.is_file() {
            if let Ok(data) = fs::read_to_string(&p) {
                if let Ok(mut atlas) = serde_json::from_str::<Self>(&data) {
                    for entry in atlas.entries.values_mut() {
                        entry.update_heat();
                    }
                    return atlas;
                }
            }
        }
        Self::new()
    }

    /// Persist to disk atomically
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let p = path.map(PathBuf::from).unwrap_or_else(Self::default_path);
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| TagisanError::Execution(format!("Failed to serialize swarm heat atlas: {}", e)))?;

        let temp_file = p.with_extension(format!("tmp.{}", std::process::id()));
        fs::write(&temp_file, json)
            .map_err(|e| TagisanError::Execution(format!("Failed to write swarm atlas temp file: {}", e)))?;
        fs::rename(&temp_file, &p)
            .map_err(|e| TagisanError::Execution(format!("Failed to commit swarm atlas file: {}", e)))?;

        Ok(())
    }

    /// Record an invocation / routing event
    pub fn record_route(
        &mut self,
        name: &str,
        is_agent: bool,
        topic: Option<&str>,
        success: bool,
        latency_ms: u64,
        tokens: usize,
    ) {
        let key = name.to_lowercase().replace('_', "-");
        let cluster = if let Some(t) = topic {
            TopicCluster::from_str(t).as_str().to_string()
        } else {
            TopicCluster::infer(name, "", &[]).as_str().to_string()
        };

        let entry = self
            .entries
            .entry(key)
            .or_insert_with(|| AtlasEntry::new(name, is_agent, &cluster));

        entry.invocations += 1;
        if success {
            entry.successes += 1;
        } else {
            entry.failures += 1;
        }
        entry.total_latency_ms += latency_ms;
        entry.total_tokens += tokens as u64;
        entry.last_invoked_epoch_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        entry.update_heat();
        self.total_routes += 1;
        self.last_updated_epoch_secs = entry.last_invoked_epoch_secs;
    }

    /// Retrieve an entry by name
    pub fn get_entry(&self, name: &str) -> Option<&AtlasEntry> {
        let key = name.to_lowercase().replace('_', "-");
        self.entries.get(&key)
    }

    /// Find the highest heat entry
    pub fn hottest_entry(&self) -> Option<(&String, &AtlasEntry)> {
        self.entries
            .iter()
            .max_by(|a, b| a.1.heat_score.partial_cmp(&b.1.heat_score).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Get cluster distribution counts
    pub fn cluster_breakdown(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for entry in self.entries.values() {
            *counts.entry(entry.cluster.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Render live ANSI/ASCII cortex dashboard table
    pub fn render_atlas_table(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "\n{}\n",
            "╔═══════════════════════════════════════════════════════════════════════════════════════════════╗".bright_cyan()
        ));
        out.push_str(&format!(
            "║  {}  ║\n",
            "🧠 TAGISAN SWARM CORTEX: AGENT & SKILL ROUTING ATLAS                                        ".bold().bright_white()
        ));
        out.push_str(&format!(
            "{}\n",
            "╠═══════════════════════════════════════════════════════════════════════════════════════════════╣".bright_cyan()
        ));

        let total_entities = self.entries.len();
        let total_invocations: u64 = self.entries.values().map(|e| e.invocations).sum();
        let total_successes: u64 = self.entries.values().map(|e| e.successes).sum();
        let overall_success_rate = if total_invocations == 0 {
            100.0
        } else {
            (total_successes as f64 / total_invocations as f64) * 100.0
        };

        let hottest = self
            .hottest_entry()
            .map(|(name, e)| format!("{} ({:.1}°C)", name, e.heat_score))
            .unwrap_or_else(|| "None".to_string());

        out.push_str(&format!(
            "║  Entities: {:<4} │ Total Invocations: {:<6} │ Swarm Success: {:>5.1}% │ Peak Heat: {:<16} ║\n",
            total_entities, total_invocations, overall_success_rate, hottest
        ));
        out.push_str(&format!(
            "{}\n",
            "╠═══════════════════════════════════════════════════════════════════════════════════════════════╣".bright_cyan()
        ));

        // Table Header
        out.push_str(&format!(
            "║ {:<22} │ {:<6} │ {:<15} │ {:>5} │ {:>6} │ {:>7} │ {:<12} │ {:<9} ║\n",
            "NAME", "KIND", "TOPIC CLUSTER", "CALLS", "SUCC %", "LAT(ms)", "HEAT GAUGE", "TIER"
        ));
        out.push_str(&format!(
            "{}\n",
            "╟────────────────────────┼────────┼─────────────────┼───────┼────────┼─────────┼──────────────┼───────────╢".bright_cyan()
        ));

        if self.entries.is_empty() {
            out.push_str(&format!(
                "║  {:<89}║\n",
                "No routing heat recorded yet. Run agent or swarm workflows to populate atlas.".italic().bright_black()
            ));
        } else {
            let mut sorted: Vec<&AtlasEntry> = self.entries.values().collect();
            sorted.sort_by(|a, b| b.heat_score.partial_cmp(&a.heat_score).unwrap_or(std::cmp::Ordering::Equal));

            for entry in sorted {
                let kind_str = if entry.is_agent { "Agent" } else { "Skill" };
                let heat_gauge = Self::format_heat_gauge(entry.heat_score);
                let tier_colored = match entry.tier.as_str() {
                    "L1 Active" => "L1 Active".bright_green().bold(),
                    "L2 Warm" => "L2 Warm".bright_yellow(),
                    _ => "L3 Cold".bright_black(),
                };

                let name_truncated = if entry.name.len() > 22 {
                    format!("{}...", &entry.name[..19])
                } else {
                    entry.name.clone()
                };

                let cluster_short = if entry.cluster.len() > 15 {
                    &entry.cluster[..15]
                } else {
                    &entry.cluster
                };

                out.push_str(&format!(
                    "║ {:<22} │ {:<6} │ {:<15} │ {:>5} │ {:>5.1}% │ {:>7} │ {} │ {:<9} ║\n",
                    name_truncated.bright_white(),
                    kind_str.cyan(),
                    cluster_short,
                    entry.invocations,
                    entry.success_rate(),
                    entry.avg_latency_ms(),
                    heat_gauge,
                    tier_colored,
                ));
            }
        }

        out.push_str(&format!(
            "{}\n",
            "╚═══════════════════════════════════════════════════════════════════════════════════════════════╝".bright_cyan()
        ));

        out
    }

    fn format_heat_gauge(score: f64) -> String {
        let bars = (score / 10.0).round().clamp(0.0, 10.0) as usize;
        let mut bar_str = String::new();
        for _ in 0..bars {
            bar_str.push('█');
        }
        for _ in bars..10 {
            bar_str.push('░');
        }

        if score >= 60.0 {
            format!("[{}] {:>4.1}°", bar_str.bright_red(), score)
        } else if score >= 20.0 {
            format!("[{}] {:>4.1}°", bar_str.bright_yellow(), score)
        } else {
            format!("[{}] {:>4.1}°", bar_str.bright_blue(), score)
        }
    }
}
