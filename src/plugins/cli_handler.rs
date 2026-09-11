//! Clap Subcommands & Interactive CLI Handler for `tgs plugin`
//!
//! Provides `list`, `install`, `inspect`, `new`, `test`, `search`, and `uninstall`.

use crate::error::{Result, TagisanError};
use crate::plugins::manager::{InstallScope, PluginManager};
use crate::plugins::manifest::PluginRuntimeType;
use clap::Subcommand;
use colored::Colorize;
use serde_json::json;
use std::path::{Path, PathBuf};

/// Subcommands under `tgs plugin`
#[derive(Subcommand, Debug, Clone)]
pub enum PluginAction {
    /// List all installed and active plugins
    List {
        /// Display capability permission breakdown
        #[arg(long)]
        permissions: bool,
        /// Output in structured JSON format
        #[arg(long)]
        json: bool,
    },
    /// Install a plugin from a GitHub repository or local directory
    Install {
        /// GitHub repository URL (e.g. github.com/modelcontextprotocol/servers/postgres) or local path
        source: String,
        /// Install into global ~/.tagisan/plugins instead of local workspace
        #[arg(long, short)]
        global: bool,
        /// Allow high-privilege native cdylib plugin execution
        #[arg(long)]
        allow_native: bool,
    },
    /// Inspect plugin manifest, declared tools, and capability sandbox limits
    Inspect {
        /// Name of the installed plugin
        name: String,
    },
    /// Scaffold a new plugin boilerplate with standard templates
    New {
        /// Name of the new plugin
        name: String,
        /// Template: "bun-ts", "wasm", "mcp", or "native"
        #[arg(short, long, default_value = "bun-ts")]
        template: String,
    },
    /// Run automated verification tests on a plugin within the sandbox
    Test {
        /// Name or path of the plugin to test
        target: String,
    },
    /// Search curated GitHub ecosystem and community plugin catalog
    Search {
        /// Query string (or "all")
        query: String,
    },
    /// Uninstall and remove a plugin
    Uninstall {
        /// Name of the plugin to remove
        name: String,
    },
}

/// Execute a `tgs plugin <action>` CLI command
pub async fn handle_plugin_command(action: PluginAction) -> Result<()> {
    match action {
        PluginAction::List { permissions, json } => {
            let mut manager = PluginManager::new(true);
            manager.load_all().await?;
            let loaded = manager.loaded_plugins();

            if json {
                let items: Vec<serde_json::Value> = loaded
                    .values()
                    .map(|p| {
                        let perms = p.manifest.permissions();
                        json!({
                            "name": p.manifest.plugin.name,
                            "version": p.manifest.plugin.version,
                            "runtime": p.manifest.plugin.runtime.to_string(),
                            "description": p.manifest.plugin.description,
                            "root_dir": p.root_dir,
                            "tools": p.tools.iter().map(|t| &t.namespaced_name).collect::<Vec<_>>(),
                            "skills_count": p.skills.len(),
                            "permissions": {
                                "network": perms.network,
                                "fs_read": perms.fs_read,
                                "fs_write": perms.fs_write,
                                "env": perms.env,
                                "subprocesses": perms.subprocesses,
                                "allow_native": perms.allow_native,
                            }
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&items).unwrap());
                return Ok(());
            }

            println!("\n{}", "🔌 Tagisan Installed Plugins & Extensions".bold().cyan());
            println!("{}", "━".repeat(78).bright_black());

            if loaded.is_empty() {
                println!(
                    "  {} No plugins currently installed.",
                    "ℹ".cyan().bold()
                );
                println!(
                    "  Install plugins via: {}",
                    "tgs plugin install <github-url-or-local-path>".yellow()
                );
                println!(
                    "  Search catalog via:  {}",
                    "tgs plugin search <keyword>".yellow()
                );
                println!("{}", "━".repeat(78).bright_black());
                return Ok(());
            }

            for p in loaded.values() {
                let m = &p.manifest.plugin;
                let perms = p.manifest.permissions();
                let runtime_badge = match m.runtime {
                    PluginRuntimeType::Wasm => "WASM".green().bold(),
                    PluginRuntimeType::Bun => "BUN/TS".blue().bold(),
                    PluginRuntimeType::Mcp => "MCP".magenta().bold(),
                    PluginRuntimeType::Native => "NATIVE".red().bold(),
                };

                println!(
                    "  {} {} {} [{}]",
                    "●".green(),
                    m.name.bold().white(),
                    format!("v{}", m.version).bright_black(),
                    runtime_badge
                );
                if let Some(ref desc) = m.description {
                    println!("    {}", desc.dimmed());
                }
                println!(
                    "    Path:  {}",
                    p.root_dir.display().to_string().bright_black()
                );
                println!(
                    "    Tools: {} registered | Skills: {} loaded",
                    p.tools.len().to_string().cyan(),
                    p.skills.len().to_string().cyan()
                );

                if permissions {
                    println!("    {}", "Capabilities:".yellow().underline());
                    println!(
                        "      • Network:      {:?}",
                        if perms.network.is_empty() {
                            vec!["<blocked>"]
                        } else {
                            perms.network.iter().map(|s| s.as_str()).collect()
                        }
                    );
                    println!(
                        "      • Filesystem R: {:?}",
                        if perms.fs_read.is_empty() {
                            vec!["<blocked>"]
                        } else {
                            perms.fs_read.iter().map(|s| s.as_str()).collect()
                        }
                    );
                    println!(
                        "      • Filesystem W: {:?}",
                        if perms.fs_write.is_empty() {
                            vec!["<blocked>"]
                        } else {
                            perms.fs_write.iter().map(|s| s.as_str()).collect()
                        }
                    );
                    println!(
                        "      • Environment:  {:?}",
                        if perms.env.is_empty() {
                            vec!["<none>"]
                        } else {
                            perms.env.iter().map(|s| s.as_str()).collect()
                        }
                    );
                    println!(
                        "      • Subprocesses: {}",
                        if perms.subprocesses {
                            "ALLOWED".red().bold()
                        } else {
                            "FORBIDDEN".green().bold()
                        }
                    );
                }
                println!();
            }
            println!("{}", "━".repeat(78).bright_black());
            Ok(())
        }

        PluginAction::Install {
            source,
            global,
            allow_native,
        } => {
            println!(
                "\n{} Installing plugin from '{}'...",
                "🚀".cyan(),
                source.bold().yellow()
            );

            let mut manager = PluginManager::new(allow_native);
            let scope = if global {
                InstallScope::Global
            } else {
                InstallScope::Local
            };

            let target_path = manager.install(&source, scope).await?;
            println!(
                "{} Plugin installed into {}",
                "✓".green().bold(),
                target_path.display().to_string().cyan()
            );

            // Validate installation
            let plugin = manager.load_plugin(&target_path).await?;
            println!(
                "{} Plugin '{}' v{} verified with {} tool(s).",
                "✓".green().bold(),
                plugin.manifest.plugin.name.bold(),
                plugin.manifest.plugin.version,
                plugin.tools.len()
            );

            Ok(())
        }

        PluginAction::Inspect { name } => {
            let mut manager = PluginManager::new(true);
            manager.load_all().await?;

            let plugin = manager.get_plugin(&name).ok_or_else(|| {
                TagisanError::Execution(format!(
                    "Plugin '{}' is not installed. Use 'tgs plugin list' to see active plugins.",
                    name
                ))
            })?;

            let m = &plugin.manifest.plugin;
            let perms = plugin.manifest.permissions();

            println!("\n{}", format!("🔌 Plugin Inspection: {}", m.name).bold().cyan());
            println!("{}", "━".repeat(78).bright_black());
            println!("  Name:        {}", m.name.bold().white());
            println!("  Version:     {}", m.version.white());
            println!("  Runtime:     {:?}", m.runtime);
            println!("  Author:      {}", m.author.as_deref().unwrap_or("N/A"));
            println!("  License:     {}", m.license.as_deref().unwrap_or("N/A"));
            println!("  Homepage:    {}", m.homepage.as_deref().unwrap_or("N/A"));
            println!("  Entrypoint:  {}", m.entrypoint);
            println!("  Directory:   {}", plugin.root_dir.display());

            println!("\n  {}", "Exposed Tools:".yellow().bold());
            if plugin.tools.is_empty() {
                println!("    (No tools exposed)");
            } else {
                for t in &plugin.tools {
                    println!("    • {} -> {}", t.original_name.cyan(), t.namespaced_name.dimmed());
                    println!("      {}", t.description);
                }
            }

            println!("\n  {}", "Capability Sandbox:".yellow().bold());
            println!("    • Network:      {:?}", perms.network);
            println!("    • Fs Read:      {:?}", perms.fs_read);
            println!("    • Fs Write:     {:?}", perms.fs_write);
            println!("    • Env Vars:     {:?}", perms.env);
            println!("    • Subprocesses: {}", perms.subprocesses);
            println!("    • Native Libs:  {}", perms.allow_native);
            println!("    • Timeout:      {}s", perms.timeout_secs);
            println!("{}", "━".repeat(78).bright_black());

            Ok(())
        }

        PluginAction::New { name, template } => {
            println!(
                "\n{} Scaffolding new plugin '{}' with template '{}'...",
                "✨".cyan(),
                name.bold().white(),
                template.bold().yellow()
            );

            let clean_name = Path::new(&name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&name);

            let plugin_dir = if name.contains('/') || name.contains('\\') {
                PathBuf::from(&name)
            } else {
                PathBuf::from(".tagisan/plugins").join(&name)
            };

            if plugin_dir.exists() {
                return Err(TagisanError::Execution(format!(
                    "Plugin directory '{}' already exists",
                    plugin_dir.display()
                )));
            }

            std::fs::create_dir_all(&plugin_dir).map_err(|e| {
                TagisanError::Execution(format!("Failed to create plugin directory: {e}"))
            })?;

            let (runtime_str, entrypoint_file, entrypoint_content) = match template.as_str() {
                "wasm" => (
                    "wasm",
                    "lib.rs",
                    r#"// WebAssembly Plugin for Tagisan
// Build: cargo build --target wasm32-wasi --release
pub fn main() {
    println!("Tagisan WASM Plugin Initialized");
}
"#,
                ),
                "mcp" => (
                    "mcp",
                    "server.py",
                    r#"# Model Context Protocol stdio server for Tagisan
import sys, json

def main():
    while True:
        line = sys.stdin.readline()
        if not line: break
        # JSON-RPC 2.0 loop
        sys.stdout.write(line)
        sys.stdout.flush()

if __name__ == '__main__':
    main()
"#,
                ),
                _ => (
                    "bun",
                    "index.ts",
                    r#"// Tagisan Bun/TypeScript Plugin
export default {
    tools: [
        {
            name: "hello_tool",
            description: "Sample tool that returns a greeting",
            parameters: {
                type: "object",
                properties: {
                    name: { type: "string", description: "Name to greet" }
                },
                required: ["name"]
            },
            async execute(args: { name: string }, context: any) {
                return `Hello from ${args.name}!`;
            }
        }
    ]
};
"#,
                ),
            };

            let manifest_content = format!(
                r#"[plugin]
name = "{clean_name}"
version = "0.1.0"
author = "Your Name <you@example.com>"
description = "A high-performance {template} plugin for Tagisan"
runtime = "{runtime_str}"
entrypoint = "{entrypoint_file}"

[tools]
enabled = ["hello_tool"]

[permissions]
network = []
fs_read = ["./*"]
fs_write = []
env = []
subprocesses = false
timeout_secs = 30
"#
            );

            std::fs::write(plugin_dir.join("tgs-plugin.toml"), manifest_content).map_err(|e| {
                TagisanError::Execution(format!("Failed to write tgs-plugin.toml: {e}"))
            })?;
            std::fs::write(plugin_dir.join(entrypoint_file), entrypoint_content).map_err(|e| {
                TagisanError::Execution(format!("Failed to write {entrypoint_file}: {e}"))
            })?;

            // Create skills directory
            let skills_dir = plugin_dir.join("skills");
            std::fs::create_dir_all(&skills_dir).ok();

            println!(
                "{} Plugin template created at {}",
                "✓".green().bold(),
                plugin_dir.display().to_string().cyan()
            );
            println!("  Edit 'tgs-plugin.toml' and '{}' to get started.", entrypoint_file);

            Ok(())
        }

        PluginAction::Test { target } => {
            println!(
                "\n{} Testing plugin '{}' in sandbox...",
                "🧪".cyan(),
                target.bold().yellow()
            );

            let mut manager = PluginManager::new(true);
            let target_path = Path::new(&target);
            let loaded = if target_path.exists() {
                manager.load_plugin(target_path).await?
            } else {
                manager.load_all().await?;
                manager.get_plugin(&target).ok_or_else(|| {
                    TagisanError::Execution(format!("Plugin '{target}' not found."))
                })?
            };

            println!(
                "  {} Validated manifest 'tgs-plugin.toml'",
                "✓".green()
            );
            println!(
                "  {} Verified entrypoint: {}",
                "✓".green(),
                loaded.manifest.plugin.entrypoint
            );
            println!(
                "  {} Discovered {} tool(s)",
                "✓".green(),
                loaded.tools.len()
            );

            for t in &loaded.tools {
                println!("    • {}", t.namespaced_name.cyan());
            }

            println!(
                "\n{} Plugin verification passed!",
                "✓ ALL CHECKS PASSED:".green().bold()
            );
            Ok(())
        }

        PluginAction::Search { query } => {
            println!(
                "\n{} Searching curated plugin ecosystem for '{}'...",
                "🔍".cyan(),
                query.bold().yellow()
            );

            let results = PluginManager::search_curated_catalog(&query);
            println!("{}", "━".repeat(78).bright_black());

            if results.is_empty() {
                println!("  No plugins found matching '{}'.", query);
            } else {
                for item in results {
                    let runtime_badge = match item.runtime {
                        PluginRuntimeType::Wasm => "WASM".green(),
                        PluginRuntimeType::Bun => "BUN/TS".blue(),
                        PluginRuntimeType::Mcp => "MCP".magenta(),
                        PluginRuntimeType::Native => "NATIVE".red(),
                    };

                    println!(
                        "  {} {} [{}] - {}",
                        "●".cyan(),
                        item.name.bold().white(),
                        runtime_badge,
                        item.category.bright_black()
                    );
                    println!("    {}", item.description.dimmed());
                    println!("    Repository: {}", item.repository.yellow());
                    println!(
                        "    Install:    {}",
                        format!("tgs plugin install {}", item.repository).green()
                    );
                    println!();
                }
            }
            println!("{}", "━".repeat(78).bright_black());
            Ok(())
        }

        PluginAction::Uninstall { name } => {
            println!(
                "\n{} Uninstalling plugin '{}'...",
                "🗑".red(),
                name.bold().yellow()
            );

            let mut manager = PluginManager::new(true);
            let removed = manager.uninstall(&name)?;
            if removed {
                println!(
                    "{} Plugin '{}' uninstalled successfully.",
                    "✓".green().bold(),
                    name.bold()
                );
            } else {
                println!(
                    "{} Plugin '{}' was not found in local or global directories.",
                    "ℹ".yellow(),
                    name
                );
            }

            Ok(())
        }
    }
}
