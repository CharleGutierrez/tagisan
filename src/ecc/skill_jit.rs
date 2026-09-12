use crate::ecc::skills::{resolve_skill, EccSkill};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::debug;

/// Memory tier in the 3-tier memory model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTier {
    /// L1: Active Context Tier (in active prompt context window)
    L1Active,
    /// L2: Warm RAM AST & Frontmatter Tier (in memory, quick promotion)
    L2Warm,
    /// L3: Cold NVMe/Disk Tier (persisted on disk or unparsed catalog)
    L3Cold,
}

/// A parsed warm representation of a skill kept in RAM (Tier 2)
#[derive(Debug, Clone)]
pub struct WarmSkillEntry {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub triggers: Vec<String>,
    pub invariants_summary: String,
    pub cached_skill: Option<EccSkill>,
    pub access_count: usize,
    pub heat_score: f64,
    pub last_accessed: Instant,
    pub pinned: bool,
}

/// Statistics and performance metrics for the Skill JIT Manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillJitStats {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_misses: u64,
    pub l1_evictions: u64,
    pub l2_evictions: u64,
    pub pilot_prefetches: u64,
    pub l1_active_count: usize,
    pub l2_warm_count: usize,
    pub pinned_count: usize,
    pub total_requests: u64,
}

impl SkillJitStats {
    pub fn hit_rate_percent(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        let total_hits = self.l1_hits + self.l2_hits;
        (total_hits as f64 / self.total_requests as f64) * 100.0
    }
}

/// Internal state guarded by RwLock
struct JitState {
    l1_active: HashMap<String, Arc<EccSkill>>,
    l2_warm: HashMap<String, WarmSkillEntry>,
    pinned_skills: HashSet<String>,
    access_history: HashMap<String, (usize, Instant)>,
    l1_access_order: Vec<String>,
    stats: SkillJitStats,
}

/// Skill JIT Paging & Lookahead Prefetch Engine (inspired by Colibrì memory hierarchy)
/// Manages L1 (Active Context), L2 (Warm RAM AST), and L3 (Cold NVMe/Disk) with PILOT lookahead prefetching.
pub struct SkillJitManager {
    l1_capacity: usize,
    l2_capacity: usize,
    custom_dir: Option<PathBuf>,
    state: RwLock<JitState>,
}

impl SkillJitManager {
    /// Create a new SkillJitManager with explicit tier capacities and optional disk directory
    pub fn new(l1_capacity: usize, l2_capacity: usize, custom_dir: Option<PathBuf>) -> Self {
        Self {
            l1_capacity: l1_capacity.max(1),
            l2_capacity: l2_capacity.max(l1_capacity),
            custom_dir,
            state: RwLock::new(JitState {
                l1_active: HashMap::new(),
                l2_warm: HashMap::new(),
                pinned_skills: HashSet::new(),
                access_history: HashMap::new(),
                l1_access_order: Vec::new(),
                stats: SkillJitStats::default(),
            }),
        }
    }

    /// Default configuration: 16 L1 active skills, 128 L2 warm skills
    pub fn default_manager() -> Self {
        Self::new(16, 128, None)
    }
}

impl Default for SkillJitManager {
    fn default() -> Self {
        Self::default_manager()
    }
}

impl SkillJitManager {
    /// Retrieve the current tier of a skill
    pub fn get_tier(&self, skill_name: &str) -> SkillTier {
        let state = self.state.read().unwrap();
        let key = skill_name.to_lowercase().replace('_', "-");
        if state.l1_active.contains_key(&key) {
            SkillTier::L1Active
        } else if state.l2_warm.contains_key(&key) {
            SkillTier::L2Warm
        } else {
            SkillTier::L3Cold
        }
    }

    /// Pin a skill into memory, preventing LRU eviction from L1/L2
    pub fn pin_skill(&self, skill_name: &str) -> bool {
        let mut state = self.state.write().unwrap();
        let key = skill_name.to_lowercase().replace('_', "-");
        state.pinned_skills.insert(key);
        true
    }

    /// Unpin a skill, making it eligible for LRU eviction
    pub fn unpin_skill(&self, skill_name: &str) -> bool {
        let mut state = self.state.write().unwrap();
        let key = skill_name.to_lowercase().replace('_', "-");
        state.pinned_skills.remove(&key)
    }

    /// Check if a skill is pinned
    pub fn is_pinned(&self, skill_name: &str) -> bool {
        let state = self.state.read().unwrap();
        let key = skill_name.to_lowercase().replace('_', "-");
        state.pinned_skills.contains(&key)
    }

    /// Page in a skill to L1 (Active Context Tier)
    /// If in L1: returns immediately (L1 Hit).
    /// If in L2: promotes from Warm RAM to L1 (L2 Hit).
    /// If in L3: loads from NVMe/Disk, parses frontmatter into L2, promotes to L1 (L3 Miss).
    pub fn page_in(&self, skill_name: &str) -> Result<Arc<EccSkill>> {
        let key = skill_name.to_lowercase().replace('_', "-");

        // 1. Check L1 Hit
        {
            let mut state = self.state.write().unwrap();
            state.stats.total_requests += 1;
            if let Some(skill) = state.l1_active.get(&key).cloned() {
                state.stats.l1_hits += 1;
                Self::update_access_record(&mut state, &key);
                return Ok(skill);
            }
        }

        // 2. Check L2 Hit (Promote L2 -> L1)
        {
            let mut state = self.state.write().unwrap();
            if let Some(mut warm_entry) = state.l2_warm.remove(&key) {
                state.stats.l2_hits += 1;
                warm_entry.access_count += 1;
                warm_entry.last_accessed = Instant::now();
                warm_entry.heat_score = Self::calculate_heat(warm_entry.access_count, 0.0);

                let full_skill = if let Some(skill) = warm_entry.cached_skill.take() {
                    Arc::new(skill)
                } else {
                    Arc::new(self.resolve_from_disk_or_builtins(&key)?)
                };

                // Enforce L1 capacity before inserting
                Self::evict_l1_if_needed(&mut state, self.l1_capacity);

                state.l1_active.insert(key.clone(), full_skill.clone());
                Self::update_access_record(&mut state, &key);
                Self::refresh_counts(&mut state);
                return Ok(full_skill);
            }
        }

        // 3. L3 Miss: Fetch from Cold NVMe/Disk
        let loaded_skill = self.resolve_from_disk_or_builtins(&key)?;
        let arc_skill = Arc::new(loaded_skill);

        {
            let mut state = self.state.write().unwrap();
            state.stats.l3_misses += 1;

            // Enforce L1 capacity
            Self::evict_l1_if_needed(&mut state, self.l1_capacity);

            state.l1_active.insert(key.clone(), arc_skill.clone());
            Self::update_access_record(&mut state, &key);
            Self::refresh_counts(&mut state);
        }

        Ok(arc_skill)
    }

    /// PILOT (Predictive Inference Lookahead Operation Transfer) Engine:
    /// Analyzes planned upcoming tasks, predicts required skills, and pages them into
    /// L2 (Warm RAM) and L1 (Active Context) in a 1-step lookahead fashion.
    pub fn pilot_prefetch(&self, task_names: &[&str]) -> usize {
        if task_names.is_empty() {
            return 0;
        }

        let mut candidate_skills = Vec::new();

        for task in task_names {
            let task_lower = task.to_lowercase();
            // Match task keywords to candidate engineering skills
            if task_lower.contains("concurrency") || task_lower.contains("async") || task_lower.contains("tokio") {
                candidate_skills.push("rust-tokio-concurrency");
            }
            if task_lower.contains("security") || task_lower.contains("vulnerability") || task_lower.contains("audit") {
                candidate_skills.push("security-hardened-development");
                candidate_skills.push("red-team-vulnerability-analysis");
            }
            if task_lower.contains("database") || task_lower.contains("sql") || task_lower.contains("migration") {
                candidate_skills.push("database-migration-lifecycle");
            }
            if task_lower.contains("api") || task_lower.contains("http") || task_lower.contains("rest") {
                candidate_skills.push("restful-api-design");
            }
            if task_lower.contains("error") || task_lower.contains("compiler") || task_lower.contains("diagnostics") {
                candidate_skills.push("compiler-error-resolution");
                candidate_skills.push("rust-advanced-typesystems");
            }
            if task_lower.contains("test") || task_lower.contains("tdd") || task_lower.contains("mock") {
                candidate_skills.push("tdd-first-design");
            }
            if task_lower.contains("cluster") || task_lower.contains("mesh") || task_lower.contains("p2p") {
                candidate_skills.push("distributed-systems-resilience");
            }
            if task_lower.contains("perf") || task_lower.contains("profil") || task_lower.contains("memory") {
                candidate_skills.push("performance-profiling-analysis");
            }
        }

        candidate_skills.dedup();
        let mut prefetched_count = 0;

        for skill_name in candidate_skills {
            let key = skill_name.to_lowercase().replace('_', "-");
            let already_in_l1 = {
                let state = self.state.read().unwrap();
                state.l1_active.contains_key(&key)
            };

            if already_in_l1 {
                continue;
            }

            // Prefetch into L2 (Warm RAM AST) or promote to L1 if capacity allows
            if let Ok(skill) = self.resolve_from_disk_or_builtins(&key) {
                let mut state = self.state.write().unwrap();
                state.stats.pilot_prefetches += 1;

                // Warm entry for L2
                let warm_entry = WarmSkillEntry {
                    name: skill.name.clone(),
                    description: skill.description.clone(),
                    domain: "systems".to_string(),
                    triggers: vec![key.clone()],
                    invariants_summary: skill.instructions.lines().take(5).collect::<Vec<_>>().join(" "),
                    cached_skill: Some(skill.clone()),
                    access_count: 1,
                    heat_score: 5.0,
                    last_accessed: Instant::now(),
                    pinned: state.pinned_skills.contains(&key),
                };

                // If L1 has room, prefetch directly to L1; otherwise place in L2
                if state.l1_active.len() < self.l1_capacity {
                    state.l1_active.insert(key.clone(), Arc::new(skill));
                    Self::update_access_record(&mut state, &key);
                } else {
                    Self::evict_l2_if_needed(&mut state, self.l2_capacity);
                    state.l2_warm.insert(key.clone(), warm_entry);
                }

                Self::refresh_counts(&mut state);
                prefetched_count += 1;
            }
        }

        debug!("PILOT 1-step lookahead prefetched {} skills", prefetched_count);
        prefetched_count
    }

    /// Evict LRU unpinned skill from L1 down to L2
    pub fn evict_lru(&self) -> Option<String> {
        let mut state = self.state.write().unwrap();
        Self::evict_single_l1_to_l2(&mut state, self.l2_capacity)
    }

    /// Read performance stats
    pub fn stats(&self) -> SkillJitStats {
        let state = self.state.read().unwrap();
        state.stats.clone()
    }

    /// List currently active skills in L1
    pub fn list_active_skills(&self) -> Vec<String> {
        let state = self.state.read().unwrap();
        let mut list: Vec<String> = state.l1_active.keys().cloned().collect();
        list.sort();
        list
    }

    /// Helper to resolve skill from disk directory or built-in presets
    fn resolve_from_disk_or_builtins(&self, name: &str) -> Result<EccSkill> {
        if let Some(skill) = resolve_skill(name, self.custom_dir.as_deref()) {
            return Ok(skill);
        }

        // Check if there is an exact or approximate built-in
        let lower = name.to_lowercase().replace('_', "-");
        if let Some(skill) = resolve_skill(&lower, self.custom_dir.as_deref()) {
            return Ok(skill);
        }

        // Synthetic fallback skill if not found
        Ok(EccSkill::new(
            name,
            format!("Dynamic JIT skill context for {}", name),
            format!("# Invariants for {}\n- Enforce strict type safety and error propagation\n- Verify all pre- and post-conditions\n", name),
        ))
    }

    fn calculate_heat(access_count: usize, elapsed_secs: f64) -> f64 {
        let recency_penalty = (elapsed_secs / 60.0).min(10.0);
        ((access_count as f64 * 3.5) - recency_penalty).max(1.0)
    }

    fn update_access_record(state: &mut JitState, key: &str) {
        let now = Instant::now();
        let entry = state.access_history.entry(key.to_string()).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;

        state.l1_access_order.retain(|k| k != key);
        state.l1_access_order.push(key.to_string());
    }

    fn evict_l1_if_needed(state: &mut JitState, capacity: usize) {
        while state.l1_active.len() >= capacity && !state.l1_access_order.is_empty() {
            if Self::evict_single_l1_to_l2(state, capacity).is_none() {
                break;
            }
        }
    }

    fn evict_single_l1_to_l2(state: &mut JitState, l2_capacity: usize) -> Option<String> {
        // Find the oldest unpinned key in l1_access_order
        let candidate_idx = state
            .l1_access_order
            .iter()
            .position(|k| !state.pinned_skills.contains(k))?;

        let candidate_key = state.l1_access_order.remove(candidate_idx);
        if let Some(skill_arc) = state.l1_active.remove(&candidate_key) {
            state.stats.l1_evictions += 1;

            // Demote into L2 Warm Tier
            let (count, last_time) = state
                .access_history
                .get(&candidate_key)
                .cloned()
                .unwrap_or((1, Instant::now()));

            let heat = Self::calculate_heat(count, last_time.elapsed().as_secs_f64());
            let warm = WarmSkillEntry {
                name: skill_arc.name.clone(),
                description: skill_arc.description.clone(),
                domain: "cached".to_string(),
                triggers: vec![candidate_key.clone()],
                invariants_summary: skill_arc.instructions.lines().take(3).collect::<Vec<_>>().join(" "),
                cached_skill: Some((*skill_arc).clone()),
                access_count: count,
                heat_score: heat,
                last_accessed: last_time,
                pinned: false,
            };

            Self::evict_l2_if_needed(state, l2_capacity);
            state.l2_warm.insert(candidate_key.clone(), warm);
            Self::refresh_counts(state);
            return Some(candidate_key);
        }

        None
    }

    fn evict_l2_if_needed(state: &mut JitState, l2_capacity: usize) {
        if state.l2_warm.len() >= l2_capacity {
            // Evict lowest heat unpinned entry from L2
            let lowest_key = state
                .l2_warm
                .iter()
                .filter(|(k, entry)| !entry.pinned && !state.pinned_skills.contains(*k))
                .min_by(|a, b| a.1.heat_score.partial_cmp(&b.1.heat_score).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| k.clone());

            if let Some(key) = lowest_key {
                state.l2_warm.remove(&key);
                state.stats.l2_evictions += 1;
            }
        }
    }

    fn refresh_counts(state: &mut JitState) {
        state.stats.l1_active_count = state.l1_active.len();
        state.stats.l2_warm_count = state.l2_warm.len();
        state.stats.pinned_count = state.pinned_skills.len();
    }
}
