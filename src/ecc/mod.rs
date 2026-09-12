pub mod agent;
pub mod agentshield;
pub mod audit;
pub mod pipeline;
pub mod presets;
pub mod skills;

pub use agent::EccAgent;
pub use agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
pub use audit::EccAuditDebate;
pub use pipeline::build_ecc_pipeline;
pub use presets::{all_presets, find_preset};
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

/// Retrieve an ECC agent by name, checking built-in presets first, then an optional disk directory
pub fn resolve_agent(name: &str, custom_dir: Option<&Path>) -> Option<EccAgent> {
    if let Some(preset) = find_preset(name) {
        return Some(preset);
    }

    if let Some(dir) = custom_dir {
        let loaded = load_agents_from_dir(dir);
        let lower = name.to_lowercase().replace('_', "-");
        return loaded.into_iter().find(|a| a.name == lower);
    }

    None
}
