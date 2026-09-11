//! Plugin Lifecycle Manager & Repository Orchestrator
//!
//! Handles plugin discovery across local (`.tagisan/plugins`) and global (`~/.tagisan/plugins`)
//! directories, Git cloning from GitHub, tool registration into `ToolRegistry`,
//! and skill pack mounting into `SkillDispatcher`.

use crate::ecc::skills::{load_skills_from_dir, EccSkill, SkillDispatcher};
use crate::error::{Result, TagisanError};
use crate::plugins::adapter::PluginToolWrapper;
use crate::plugins::hooks::PluginHookRegistry;
use crate::plugins::manifest::{PluginManifest, PluginRuntimeType};
use crate::plugins::runtime::{
    BunPluginEngine, McpPluginEngine, NativePluginEngine, PluginEngine, WasmPluginEngine,
};
use crate::plugins::security::PluginSecurityGovernor;
use crate::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

/// Target directory scope for plugin installation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    /// Workspace-local: `./.tagisan/plugins/`
    Local,
    /// User-global: `~/.tagisan/plugins/`
    Global,
}

/// Representation of an active, verified plugin loaded into Tagisan
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub root_dir: PathBuf,
    pub engine: Arc<dyn PluginEngine>,
    pub tools: Vec<PluginToolWrapper>,
    pub skills: Vec<EccSkill>,
}

/// Curated entry in the Tagisan Plugin Catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedCatalogItem {
    pub name: String,
    pub repository: String,
    pub description: String,
    pub runtime: PluginRuntimeType,
    pub category: String,
}

/// Primary orchestrator managing plugin discovery, sandbox governance, and registration
pub struct PluginManager {
    pub local_dir: PathBuf,
    pub global_dir: PathBuf,
    loaded_plugins: HashMap<String, Arc<LoadedPlugin>>,
    pub hooks: PluginHookRegistry,
    pub allow_native_plugins: bool,
    pub security_governor: Arc<PluginSecurityGovernor>,
}

impl PluginManager {
    /// Creates a new PluginManager with default search paths
    pub fn new(allow_native_plugins: bool) -> Self {
        let local_dir = PathBuf::from(".tagisan/plugins");
        let global_dir = dirs_fallback_global_dir();
        let security_governor = Arc::new(PluginSecurityGovernor::new());

        Self {
            local_dir,
            global_dir,
            loaded_plugins: HashMap::new(),
            hooks: PluginHookRegistry::new(),
            allow_native_plugins,
            security_governor,
        }
    }

    /// Creates a new PluginManager with custom directories (ideal for tests)
    pub fn with_dirs(local_dir: PathBuf, global_dir: PathBuf, allow_native: bool) -> Self {
        Self {
            local_dir,
            global_dir,
            loaded_plugins: HashMap::new(),
            hooks: PluginHookRegistry::new(),
            allow_native_plugins: allow_native,
            security_governor: Arc::new(PluginSecurityGovernor::new()),
        }
    }

    /// Discover all plugin directories containing a valid `tgs-plugin.toml`
    pub fn discover_paths(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // 1. Scan local workspace: .tagisan/plugins/
        if self.local_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&self.local_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("tgs-plugin.toml").is_file() {
                        dirs.push(path);
                    }
                }
            }
        }

        // 2. Scan global home: ~/.tagisan/plugins/
        if self.global_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&self.global_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && path.join("tgs-plugin.toml").is_file() {
                        if !dirs.iter().any(|d| d.file_name() == path.file_name()) {
                            dirs.push(path);
                        }
                    }
                }
            }
        }

        dirs
    }

    /// Load a specific plugin from a directory into memory
    pub async fn load_plugin(&mut self, plugin_dir: &Path) -> Result<Arc<LoadedPlugin>> {
        let manifest_path = plugin_dir.join("tgs-plugin.toml");
        let manifest = PluginManifest::load_from(&manifest_path)?;
        manifest.validate_filesystem(plugin_dir)?;

        let engine: Arc<dyn PluginEngine> = match manifest.plugin.runtime {
            PluginRuntimeType::Wasm => {
                Arc::new(WasmPluginEngine::new(manifest.clone(), plugin_dir)?)
            }
            PluginRuntimeType::Bun => {
                Arc::new(BunPluginEngine::new(manifest.clone(), plugin_dir)?)
            }
            PluginRuntimeType::Mcp => {
                Arc::new(McpPluginEngine::connect(manifest.clone(), plugin_dir).await?)
            }
            PluginRuntimeType::Native => Arc::new(NativePluginEngine::new(
                manifest.clone(),
                plugin_dir,
                self.allow_native_plugins,
            )?),
        };

        // Discover and build tool wrappers
        let tool_descriptors = engine.discover_tools().await.unwrap_or_default();
        let mut tools = Vec::new();

        for desc in tool_descriptors {
            let wrapper = PluginToolWrapper::new(
                &manifest,
                desc.name,
                desc.description,
                desc.parameters,
                engine.clone(),
                plugin_dir.to_path_buf(),
                self.security_governor.clone(),
            );
            tools.push(wrapper);
        }

        // Load SKILL.md packages if declared
        let mut skills = Vec::new();
        if let Some(ref skill_cfg) = manifest.skills {
            if let Some(ref rel_dir) = skill_cfg.catalog_dir {
                let skill_dir = plugin_dir.join(rel_dir);
                if skill_dir.is_dir() {
                    let loaded_skills = load_skills_from_dir(&skill_dir);
                    skills.extend(loaded_skills);
                }
            }
        }

        let loaded = Arc::new(LoadedPlugin {
            manifest: manifest.clone(),
            root_dir: plugin_dir.to_path_buf(),
            engine,
            tools,
            skills,
        });

        self.loaded_plugins
            .insert(manifest.plugin.name.clone(), loaded.clone());
        info!(
            "Loaded plugin '{}' v{} ({:?})",
            manifest.plugin.name, manifest.plugin.version, manifest.plugin.runtime
        );

        Ok(loaded)
    }

    /// Load all discovered plugins into memory
    pub async fn load_all(&mut self) -> Result<usize> {
        let paths = self.discover_paths();
        let mut count = 0;
        for path in paths {
            match self.load_plugin(&path).await {
                Ok(_) => count += 1,
                Err(e) => warn!("Failed to load plugin at '{}': {e}", path.display()),
            }
        }
        Ok(count)
    }

    /// Populate Tagisan's ToolRegistry with all enabled plugin tools
    pub fn populate_tool_registry(&self, registry: &mut ToolRegistry) -> usize {
        let mut count = 0;
        for plugin in self.loaded_plugins.values() {
            for tool in &plugin.tools {
                registry.register(Arc::new(tool.clone()));
                count += 1;
            }
        }
        count
    }

    /// Populate Tagisan's hybrid SkillDispatcher with all plugin skills
    pub fn populate_skill_dispatcher(&self, dispatcher: &mut SkillDispatcher) -> usize {
        let mut count = 0;
        for plugin in self.loaded_plugins.values() {
            for skill in &plugin.skills {
                dispatcher.register_skill(skill.clone());
                count += 1;
            }
        }
        count
    }

    /// Install a plugin directly from a GitHub repository, local directory, or archive
    pub async fn install(&mut self, source: &str, scope: InstallScope) -> Result<PathBuf> {
        let target_base = match scope {
            InstallScope::Local => &self.local_dir,
            InstallScope::Global => &self.global_dir,
        };
        std::fs::create_dir_all(target_base).map_err(|e| {
            TagisanError::Execution(format!("Failed to create plugin directory: {e}"))
        })?;

        // 1. Local path installation
        let src_path = Path::new(source);
        if src_path.exists() {
            let manifest_path = if src_path.is_file()
                && src_path.file_name().map_or(false, |f| f == "tgs-plugin.toml")
            {
                src_path.to_path_buf()
            } else {
                src_path.join("tgs-plugin.toml")
            };

            let manifest = PluginManifest::load_from(&manifest_path)?;
            let dest_dir = target_base.join(&manifest.plugin.name);
            let copy_source = if src_path.is_file() {
                src_path.parent().unwrap_or(src_path)
            } else {
                src_path
            };
            copy_dir_recursive(copy_source, &dest_dir)?;
            return Ok(dest_dir);
        }

        // 2. GitHub URL installation via git clone
        let repo_url = if source.starts_with("github.com/") {
            format!("https://{source}.git")
        } else if source.starts_with("https://github.com/") {
            source.to_string()
        } else {
            return Err(TagisanError::Execution(format!(
                "Unrecognized plugin source: '{source}'. Must be an existing local path or GitHub URL."
            )));
        };

        let plugin_slug = repo_url
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .unwrap_or("unnamed-plugin");
        let dest_dir = target_base.join(plugin_slug);

        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir).ok();
        }

        info!("Cloning plugin from '{}' into '{}'...", repo_url, dest_dir.display());
        let status = std::process::Command::new("git")
            .args(["clone", "--depth", "1", &repo_url, &dest_dir.to_string_lossy()])
            .status()
            .map_err(|e| TagisanError::Execution(format!("Failed to run git clone: {e}")))?;

        if !status.success() {
            return Err(TagisanError::Execution(format!(
                "git clone failed with status code: {:?}",
                status.code()
            )));
        }

        let manifest_path = dest_dir.join("tgs-plugin.toml");
        if !manifest_path.exists() {
            std::fs::remove_dir_all(&dest_dir).ok();
            return Err(TagisanError::Execution(format!(
                "Repository '{}' is missing required 'tgs-plugin.toml' manifest",
                repo_url
            )));
        }

        let manifest = PluginManifest::load_from(&manifest_path)?;
        info!(
            "Installed plugin '{}' v{} successfully",
            manifest.plugin.name, manifest.plugin.version
        );

        Ok(dest_dir)
    }

    /// Uninstall a plugin by removing its directory
    pub fn uninstall(&mut self, name: &str) -> Result<bool> {
        let mut removed = false;
        let local_plugin = self.local_dir.join(name);
        if local_plugin.exists() {
            std::fs::remove_dir_all(&local_plugin).map_err(|e| {
                TagisanError::Execution(format!("Failed to delete local plugin directory: {e}"))
            })?;
            removed = true;
        }

        let global_plugin = self.global_dir.join(name);
        if global_plugin.exists() {
            std::fs::remove_dir_all(&global_plugin).map_err(|e| {
                TagisanError::Execution(format!("Failed to delete global plugin directory: {e}"))
            })?;
            removed = true;
        }

        self.loaded_plugins.remove(name);
        Ok(removed)
    }

    /// Returns map of currently loaded plugins
    pub fn loaded_plugins(&self) -> &HashMap<String, Arc<LoadedPlugin>> {
        &self.loaded_plugins
    }

    /// Retrieve a specific loaded plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<LoadedPlugin>> {
        self.loaded_plugins.get(name).cloned()
    }

    /// Curated catalog search from RFC-002 ecosystem blueprints
    pub fn search_curated_catalog(query: &str) -> Vec<CuratedCatalogItem> {
        let catalog = vec![
            CuratedCatalogItem {
                name: "mcp-postgres".to_string(),
                repository: "https://github.com/modelcontextprotocol/servers/tree/main/src/postgres".to_string(),
                description: "PostgreSQL schema inspector, execution plan analyzer, and index auditor".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Databases".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-sqlite".to_string(),
                repository: "https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite".to_string(),
                description: "Local SQLite database inspector, schema migration runner, and table querying".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Databases".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-docker".to_string(),
                repository: "https://github.com/docker/mcp-server-docker".to_string(),
                description: "Spins up isolated container sandboxes during ecc_verify to test builds".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "DevOps".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-sentry".to_string(),
                repository: "https://github.com/getsentry/mcp-server-sentry".to_string(),
                description: "Fetches live production crash stack traces for automatic swarm diagnosis".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Observability".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-cloudflare".to_string(),
                repository: "https://github.com/cloudflare/mcp-server-cloudflare".to_string(),
                description: "Deploys and debugs Cloudflare Workers, KV namespaces, and D1 serverless SQL".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Cloud".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-qdrant".to_string(),
                repository: "https://github.com/qdrant/mcp-server-qdrant".to_string(),
                description: "High-performance vector memory connector for cross-session agent recall".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Vector DB".to_string(),
            },
            CuratedCatalogItem {
                name: "ast-grep".to_string(),
                repository: "https://github.com/ast-grep/ast-grep".to_string(),
                description: "Fast syntax-tree-aware pattern replacement across 20+ programming languages".to_string(),
                runtime: PluginRuntimeType::Native,
                category: "Code Intelligence".to_string(),
            },
            CuratedCatalogItem {
                name: "tree-sitter".to_string(),
                repository: "https://github.com/tree-sitter/tree-sitter".to_string(),
                description: "Incremental code parsing and symbol definition lookups with microsecond latency".to_string(),
                runtime: PluginRuntimeType::Native,
                category: "Code Intelligence".to_string(),
            },
            CuratedCatalogItem {
                name: "mcp-puppeteer".to_string(),
                repository: "https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer".to_string(),
                description: "Headless browser automation, visual diff testing, and screenshot verification".to_string(),
                runtime: PluginRuntimeType::Mcp,
                category: "Browser Automation".to_string(),
            },
            CuratedCatalogItem {
                name: "gitleaks-scanner".to_string(),
                repository: "https://github.com/gitleaks/gitleaks".to_string(),
                description: "Automated secret, API key, and credential leak scanning before git commits".to_string(),
                runtime: PluginRuntimeType::Wasm,
                category: "Security".to_string(),
            },
        ];

        let q = query.to_lowercase();
        if q.is_empty() || q == "all" {
            return catalog;
        }

        catalog
            .into_iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&q)
                    || item.description.to_lowercase().contains(&q)
                    || item.category.to_lowercase().contains(&q)
            })
            .collect()
    }
}

fn dirs_fallback_global_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".tagisan").join("plugins")
    } else {
        PathBuf::from(".tagisan/plugins")
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst).map_err(|e| {
        TagisanError::Execution(format!("Failed to create directory '{}': {e}", dst.display()))
    })?;
    for entry in std::fs::read_dir(src).map_err(|e| {
        TagisanError::Execution(format!("Failed to read directory '{}': {e}", src.display()))
    })? {
        let entry = entry.map_err(|e| TagisanError::Execution(e.to_string()))?;
        let ty = entry
            .file_type()
            .map_err(|e| TagisanError::Execution(e.to_string()))?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path).map_err(|e| {
                TagisanError::Execution(format!("Failed to copy file: {e}"))
            })?;
        }
    }
    Ok(())
}
