pub mod agent;
pub mod agentshield;
pub mod audit;
pub mod pipeline;
pub mod presets;
pub mod semantic_guard;
pub mod skill_jit;
pub mod skills;

pub use agent::EccAgent;
pub use agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
pub use audit::EccAuditDebate;
pub use pipeline::build_ecc_pipeline;
pub use presets::{all_presets, find_preset};
pub use semantic_guard::{PreservedDiagnostic, SemanticInvariantGuard, SemanticViolation};
pub use skill_jit::{SkillJitManager, SkillJitStats, SkillTier, WarmSkillEntry};
pub use skills::{
    all_built_in_skills, compute_dir_fingerprint, estimate_tokens, extract_triggers_from_text,
    find_built_in_skill, format_cheat_sheet, format_cloud_guidelines, format_dense_invariants,
    format_hierarchical, global_dispatcher, is_local_provider, load_skills_from_dir, resolve_skill,
    CachedCatalog, DispatchedSkill, DiversifiedDispatchResult, EccSkill, InjectionMode,
    SkillDispatcher, SkillMetadata, TokenBudget, SKILLS_CACHE_MAGIC, SKILLS_CACHE_VERSION,
};

use std::path::Path;
use tracing::warn;

/// Discover and load all ECC agent definitions (*.md) from a given directory
pub fn load_agents_from_dir(dir: impl AsRef<Path>) -> Vec<EccAgent> {
    let mut agents = Vec::new();
    let dir_ref = dir.as_ref();

    if !dir_ref.is_dir() {
        return agents;
    }

    if let Ok(entries) = std::fs::read_dir(dir_ref) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match EccAgent::from_file(&path) {
                    Ok(agent) => agents.push(agent),
                    Err(e) => {
                        warn!("Failed to load ECC agent from '{}': {}", path.display(), e);
                    }
                }
            }
        }
    }

    // Sort alphabetically by name for deterministic order
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    agents
}

/// Retrieve an ECC agent by name, checking built-in presets first, then an optional disk directory,
/// followed by fallback discovery locations (cwd, parent directories, ~/.ecc/agents, TAGISAN_ECC_DIR).
pub fn resolve_agent(name: &str, custom_dir: Option<&Path>) -> Option<EccAgent> {
    if let Some(preset) = find_preset(name) {
        return Some(preset);
    }

    let lower = name.to_lowercase().replace('_', "-");

    // Helper to probe a directory for an agent:
    // 1. Direct file lookup: `<dir>/<lower>.md` or `<dir>/<name>.md`
    // 2. Scan directory entries
    let probe_dir = |dir: &Path| -> Option<EccAgent> {
        if !dir.is_dir() {
            return None;
        }

        // Fast O(1) direct file checks
        let direct_lower = dir.join(format!("{}.md", lower));
        if direct_lower.is_file() {
            if let Ok(agent) = EccAgent::from_file(&direct_lower) {
                return Some(agent);
            }
        }
        let direct_name = dir.join(format!("{}.md", name));
        if direct_name.is_file() {
            if let Ok(agent) = EccAgent::from_file(&direct_name) {
                return Some(agent);
            }
        }

        // Full directory scan fallback
        let loaded = load_agents_from_dir(dir);
        loaded
            .into_iter()
            .find(|a| a.name.to_lowercase().replace('_', "-") == lower)
    };

    // 1. Check custom_dir if supplied
    if let Some(dir) = custom_dir {
        if let Some(agent) = probe_dir(dir) {
            return Some(agent);
        }
    }

    // 2. Check current working directory `.ecc/agents`
    let cwd_agents = Path::new(".ecc/agents");
    if let Some(agent) = probe_dir(cwd_agents) {
        return Some(agent);
    }

    // 3. Ascend parent directories to locate `.ecc/agents` (workspace / project root)
    if let Ok(cwd) = std::env::current_dir() {
        let mut curr = cwd.as_path();
        while let Some(parent) = curr.parent() {
            let candidate = parent.join(".ecc/agents");
            if candidate.is_dir() {
                if let Some(agent) = probe_dir(&candidate) {
                    return Some(agent);
                }
            }
            curr = parent;
        }
    }

    // 4. Check user home directory `~/.ecc/agents`
    if let Ok(home) = std::env::var("HOME") {
        let home_agents = std::path::PathBuf::from(home).join(".ecc/agents");
        if let Some(agent) = probe_dir(&home_agents) {
            return Some(agent);
        }
    }

    // 5. Check environment variables
    for env_var in &["TAGISAN_ECC_DIR", "ECC_DIR"] {
        if let Ok(val) = std::env::var(env_var) {
            let p = std::path::PathBuf::from(&val);
            let target = if p.ends_with("agents") {
                p
            } else {
                p.join("agents")
            };
            if let Some(agent) = probe_dir(&target) {
                return Some(agent);
            }
        }
    }

    None
}

