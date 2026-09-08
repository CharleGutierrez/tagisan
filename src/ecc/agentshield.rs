use serde::{Deserialize, Serialize};

/// Outcome of an AgentShield security audit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentShieldVerdict {
    Allow,
    Block {
        reason: String,
        threat_level: ThreatLevel,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// AgentShield security scanner inspired by the ECC AgentShield specification
pub struct AgentShieldScanner;

impl AgentShieldScanner {
    /// Scan tool invocations before execution to block destructive actions or credential exfiltration
    pub fn scan_tool_call(tool_name: &str, arguments: &serde_json::Value) -> AgentShieldVerdict {
        match tool_name {
            "run_command" => {
                let cmd = arguments
                    .get("command")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                Self::scan_command(cmd)
            }
            "read_file" | "write_file" => {
                let path = arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                Self::scan_file_path(path)
            }
            _ => AgentShieldVerdict::Allow,
        }
    }

    /// Scan shell commands for dangerous or irreversible patterns
    pub fn scan_command(command: &str) -> AgentShieldVerdict {
        let lower = command.to_lowercase();

        // 1. Destructive system commands
        let destructive_patterns = [
            ("rm -rf /", "Attempted recursive deletion of root filesystem", ThreatLevel::Critical),
            ("rm -rf /*", "Attempted recursive deletion of root filesystem", ThreatLevel::Critical),
            ("mkfs", "Attempted disk formatting filesystem command", ThreatLevel::Critical),
            ("dd if=", "Low-level raw device write detected", ThreatLevel::Critical),
            (":(){ :|:& };:", "Fork bomb execution pattern detected", ThreatLevel::Critical),
            ("> /dev/sda", "Direct write to block device", ThreatLevel::Critical),
        ];

        for (pattern, reason, level) in destructive_patterns {
            if lower.contains(pattern) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 2. Secret exfiltration patterns
        let secret_targets = [
            ("cat /etc/shadow", "Attempt to access system password hashes", ThreatLevel::Critical),
            ("cat ~/.ssh/id_", "Attempt to access private SSH keys", ThreatLevel::High),
            ("cat .env", "Attempt to read raw environment secrets file directly via shell", ThreatLevel::Medium),
            ("curl -d @.env", "Attempted exfiltration of environment credentials", ThreatLevel::Critical),
        ];

        for (pattern, reason, level) in secret_targets {
            if lower.contains(pattern) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Scan file paths for path traversal or sensitive system file access
    pub fn scan_file_path(path_str: &str) -> AgentShieldVerdict {
        let normalized = path_str.replace('\\', "/");

        // Path traversal check
        if normalized.starts_with("../../../") || normalized.contains("/../../") {
            return AgentShieldVerdict::Block {
                reason: "Suspicious deep directory traversal detected".to_string(),
                threat_level: ThreatLevel::High,
            };
        }

        // Sensitive system targets
        let sensitive = [
            ("/etc/shadow", ThreatLevel::Critical),
            ("/etc/passwd", ThreatLevel::Medium),
            ("/proc/kcore", ThreatLevel::Critical),
            (".ssh/id_rsa", ThreatLevel::High),
            (".ssh/id_ed25519", ThreatLevel::High),
        ];

        for (target, level) in sensitive {
            if normalized.ends_with(target) || normalized == target {
                return AgentShieldVerdict::Block {
                    reason: format!("Access to sensitive system file '{}' is prohibited", target),
                    threat_level: level,
                };
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Redact recognized secrets (API keys, bearer tokens) from output strings
    pub fn redact_secrets(content: &str) -> String {
        let mut sanitized = content.to_string();

        let patterns = [
            ("sk-ant-api03-", "[REDACTED_ANTHROPIC_KEY]"),
            ("sk-proj-", "[REDACTED_OPENAI_KEY]"),
            ("AIzaSy", "[REDACTED_GEMINI_KEY]"),
            ("xai-", "[REDACTED_XAI_KEY]"),
        ];

        for (prefix, replacement) in patterns {
            if let Some(pos) = sanitized.find(prefix) {
                // Find token boundary (whitespace or quote)
                let end = sanitized[pos..]
                    .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '\n')
                    .map(|i| pos + i)
                    .unwrap_or(sanitized.len());

                sanitized.replace_range(pos..end, replacement);
            }
        }

        sanitized
    }
}
