//! Capability-Based Sandboxing & AgentShield Security Governor
//!
//! Enforces zero ambient authority, least-privilege capability boundaries,
//! path traversal prevention, network whitelisting, and AgentShield integration.

use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use crate::plugins::manifest::PluginCapabilities;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tracing::warn;

/// Security Governor enforcing capability bounds per tool invocation
#[derive(Debug, Clone, Default)]
pub struct PluginSecurityGovernor;

impl PluginSecurityGovernor {
    pub fn new() -> Self {
        Self
    }

    /// Authorizes a tool invocation against declared capabilities and AgentShield guardrails
    pub fn verify_tool_invocation(
        &self,
        plugin_name: &str,
        tool_name: &str,
        arguments: &Value,
        caps: &PluginCapabilities,
        working_dir: &Path,
    ) -> Result<()> {
        // 1. AgentShield Core Security Scan
        let shield_verdict = AgentShieldScanner::scan_tool_call(tool_name, arguments);
        if let AgentShieldVerdict::Block { reason, threat_level } = shield_verdict {
            warn!(
                "AgentShield blocked plugin '{}' tool '{}' (Threat: {:?}): {}",
                plugin_name, tool_name, threat_level, reason
            );
            return Err(TagisanError::Execution(format!(
                "[AgentShield Security Block {:?}] {}",
                threat_level, reason
            )));
        }

        // 2. Subprocess Restriction
        let base_name = tool_name.rsplit("__").next().unwrap_or(tool_name);
        if !caps.subprocesses && is_subprocess_tool(base_name) {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Plugin '{}' attempted to spawn a subprocess via '{}' but lacks 'subprocesses = true' permission",
                plugin_name, tool_name
            )));
        }

        // 3. Filesystem Capability Verification
        if let Some(target_path) = extract_path_argument(arguments) {
            self.verify_path_access(
                plugin_name,
                &target_path,
                caps,
                working_dir,
                is_write_tool(base_name),
            )?;
        }

        // 4. Network Destination Capability Verification
        if let Some(destination) = extract_network_argument(arguments) {
            self.verify_network_access(plugin_name, &destination, caps)?;
        }

        Ok(())
    }

    /// Verifies that target path falls strictly within declared read or write glob capabilities
    pub fn verify_path_access(
        &self,
        plugin_name: &str,
        raw_path: &str,
        caps: &PluginCapabilities,
        _working_dir: &Path,
        is_write: bool,
    ) -> Result<()> {
        let normalized = raw_path.replace('\\', "/");

        // 1. Prevent directory traversal attacks
        if normalized.contains("../") || normalized.contains("/..") || normalized == ".." {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Directory traversal detected in path '{}' for plugin '{}'",
                raw_path, plugin_name
            )));
        }

        // 2. Prevent absolute root breakouts unless explicitly whitelisted
        let patterns = if is_write {
            &caps.fs_write
        } else {
            &caps.fs_read
        };

        if patterns.is_empty() {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Plugin '{}' attempted {} access to '{}' with zero {} permissions declared in tgs-plugin.toml",
                plugin_name,
                if is_write { "write" } else { "read" },
                raw_path,
                if is_write { "fs_write" } else { "fs_read" }
            )));
        }

        // Check against patterns
        let is_allowed = patterns.iter().any(|pat| {
            path_matches_pattern(&normalized, pat)
        });

        if !is_allowed {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Path '{}' is outside declared {} capability boundaries for plugin '{}'. Allowed: {:?}",
                raw_path,
                if is_write { "write" } else { "read" },
                plugin_name,
                patterns
            )));
        }

        Ok(())
    }

    /// Verifies network host and port against declared whitelist
    pub fn verify_network_access(
        &self,
        plugin_name: &str,
        host_or_url: &str,
        caps: &PluginCapabilities,
    ) -> Result<()> {
        if caps.network.is_empty() {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Plugin '{}' attempted network connection to '{}' with zero network capabilities declared in tgs-plugin.toml",
                plugin_name, host_or_url
            )));
        }

        // Extract host from url if needed
        let target_host = if let Some(stripped) = host_or_url.strip_prefix("http://") {
            stripped.split('/').next().unwrap_or(stripped)
        } else if let Some(stripped) = host_or_url.strip_prefix("https://") {
            stripped.split('/').next().unwrap_or(stripped)
        } else {
            host_or_url.split('/').next().unwrap_or(host_or_url)
        };

        let is_allowed = caps.network.iter().any(|rule| {
            if rule == "*" || rule == "*:*" {
                return true;
            }
            if rule == target_host {
                return true;
            }
            // Port-insensitive check
            let rule_host_only = rule.split(':').next().unwrap_or(rule);
            let target_host_only = target_host.split(':').next().unwrap_or(target_host);
            if rule_host_only == target_host_only {
                return true;
            }
            // Subdomain wildcard: *.domain.com
            if let Some(domain) = rule_host_only.strip_prefix("*.") {
                if target_host_only.ends_with(domain) {
                    return true;
                }
            }
            false
        });

        if !is_allowed {
            return Err(TagisanError::Execution(format!(
                "Sandbox Violation: Network destination '{}' is not permitted for plugin '{}'. Permitted: {:?}",
                host_or_url, plugin_name, caps.network
            )));
        }

        Ok(())
    }

    /// Filters and scrubs environment variables, exposing ONLY variables declared in `env`
    pub fn sanitize_env(&self, caps: &PluginCapabilities) -> HashMap<String, String> {
        let mut clean = HashMap::new();
        for key in &caps.env {
            if let Ok(val) = std::env::var(key) {
                clean.insert(key.clone(), val);
            }
        }
        clean
    }
}

fn extract_path_argument(arguments: &Value) -> Option<String> {
    for key in ["path", "file_path", "filepath", "target", "file", "outfile", "uri"] {
        if let Some(v) = arguments.get(key).and_then(|s| s.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

fn extract_network_argument(arguments: &Value) -> Option<String> {
    for key in ["url", "host", "endpoint", "address", "connection_string"] {
        if let Some(v) = arguments.get(key).and_then(|s| s.as_str()) {
            return Some(v.to_string());
        }
    }
    None
}

fn is_write_tool(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("write")
        || lower.contains("save")
        || lower.contains("delete")
        || lower.contains("edit")
        || lower.contains("remove")
        || lower.contains("modify")
        || lower.contains("create")
}

fn is_subprocess_tool(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "run_command" | "bash" | "sh" | "exec" | "spawn" | "shell" | "terminal" | "cmd"
    )
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    let p = path.strip_prefix("./").unwrap_or(path);
    let pat = pattern.strip_prefix("./").unwrap_or(pattern);

    if pat == "*" || pat == "**/*" || pat == "." || pat.is_empty() {
        return true;
    }

    if p == pat {
        return true;
    }

    // Directory prefix match: e.g. "data/" or "output/"
    if pat.ends_with('/') && p.starts_with(pat) {
        return true;
    }

    // Directory match without trailing slash: e.g. "data" matching "data/file.txt"
    if !pat.contains('*') && p.starts_with(&format!("{}/", pat)) {
        return true;
    }

    // Wildcard / Glob pattern matching
    if pat.contains('*') || pat.contains('?') {
        return glob_match(pat, p);
    }

    false
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pat_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    fn match_helper(p: &[char], t: &[char]) -> bool {
        match (p.first(), t.first()) {
            (None, None) => true,
            (Some(&'*'), _) => {
                // Double star '**' matches across path boundaries
                if p.get(1) == Some(&'*') {
                    let rest_p = if p.get(2) == Some(&'/') { &p[3..] } else { &p[2..] };
                    (0..=t.len()).any(|i| match_helper(rest_p, &t[i..]))
                } else {
                    // Single '*' matches characters within the same directory level (not '/')
                    if match_helper(&p[1..], t) {
                        return true;
                    }
                    if let Some(&c) = t.first() {
                        if c != '/' {
                            return match_helper(p, &t[1..]);
                        }
                    }
                    false
                }
            }
            (Some(&'?'), Some(_)) => match_helper(&p[1..], &t[1..]),
            (Some(c1), Some(c2)) if c1 == c2 => match_helper(&p[1..], &t[1..]),
            _ => false,
        }
    }

    match_helper(&pat_chars, &text_chars)
}
