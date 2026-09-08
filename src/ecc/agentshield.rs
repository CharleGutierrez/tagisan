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

    /// Scan shell commands for dangerous or irreversible patterns, with evasion de-obfuscation
    pub fn scan_command(command: &str) -> AgentShieldVerdict {
        let normalized = Self::normalize_command(command);

        // 1. Check for fork bombs
        if command.contains(":(){ :|:& };:")
            || command.contains(":(){:|:&};:")
            || normalized.contains(":(){:|:&};:")
            || normalized.contains("forkbomb")
        {
            return AgentShieldVerdict::Block {
                reason: "Fork bomb execution pattern detected".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 2. Check for disk/device destruction commands
        let raw_device_patterns = [
            ("mkfs", "Attempted disk formatting filesystem command", ThreatLevel::Critical),
            ("dd if=", "Low-level raw device write detected", ThreatLevel::Critical),
            ("> /dev/sd", "Direct write to raw storage block device", ThreatLevel::Critical),
            ("> /dev/nvme", "Direct write to NVMe block device", ThreatLevel::Critical),
            ("> /dev/hd", "Direct write to hard disk block device", ThreatLevel::Critical),
            ("tee /dev/sd", "Direct write to raw block device via tee", ThreatLevel::Critical),
        ];

        for (pat, reason, level) in raw_device_patterns {
            if normalized.contains(pat) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 3. Check for destructive recursive rm commands
        if Self::is_destructive_rm(&normalized) {
            return AgentShieldVerdict::Block {
                reason: "Attempted recursive deletion of root or critical filesystem hierarchy".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. Secret exfiltration and sensitive access patterns
        let sensitive_read_targets = [
            ("/etc/shadow", "Attempt to access system password hashes", ThreatLevel::Critical),
            ("etc/shadow", "Attempt to access system password hashes", ThreatLevel::Critical),
            ("/etc/passwd", "Attempt to access system user accounts", ThreatLevel::Medium),
            ("etc/passwd", "Attempt to access system user accounts", ThreatLevel::Medium),
            ("/etc/sudoers", "Attempt to access sudoers security configuration", ThreatLevel::Critical),
            ("id_rsa", "Attempt to access private SSH keys", ThreatLevel::High),
            ("id_ed25519", "Attempt to access private SSH keys", ThreatLevel::High),
            ("~/.ssh", "Attempt to access user SSH credentials", ThreatLevel::High),
            (".env", "Attempt to read raw environment secrets file directly via shell", ThreatLevel::Medium),
        ];

        let file_readers = [
            "cat", "head", "tail", "less", "more", "grep", "awk", "sed", "strings",
            "xxd", "od", "hexdump", "cp", "mv", "source", ".",
        ];

        // Check command tokens / chaining for file reader + sensitive target
        for subcmd in Self::split_chained_commands(&normalized) {
            let tokens: Vec<&str> = subcmd.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let prog = tokens[0];
            let is_reader = file_readers.iter().any(|&r| prog == r || prog.ends_with(&format!("/{r}")));

            for (target, reason, level) in sensitive_read_targets {
                if subcmd.contains(target) {
                    if is_reader || subcmd.contains(&format!("< {target}")) || subcmd.contains(&format!("<{target}")) {
                        return AgentShieldVerdict::Block {
                            reason: reason.to_string(),
                            threat_level: level,
                        };
                    }
                }
            }

            // Exfiltration tools (curl, wget, nc, etc.) targeting secrets
            let exfil_tools = ["curl", "wget", "nc", "ncat", "netcat", "socat"];
            let is_exfil = exfil_tools.iter().any(|&e| prog == e || prog.ends_with(&format!("/{e}")));
            if is_exfil && (subcmd.contains(".env") || subcmd.contains("/etc/shadow") || subcmd.contains("id_rsa")) {
                return AgentShieldVerdict::Block {
                    reason: "Attempted exfiltration of credentials or system secrets".to_string(),
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Detect destructive rm patterns like `rm -rf /`, `rm -r /`, `rm -fr /*`, `rm -rf ~`, `rm -rf .`
    fn is_destructive_rm(cmd: &str) -> bool {
        for subcmd in Self::split_chained_commands(cmd) {
            let tokens: Vec<&str> = subcmd.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let rm_idx = tokens.iter().position(|&t| t == "rm" || t.ends_with("/rm"));
            if let Some(idx) = rm_idx {
                let args = &tokens[idx + 1..];
                let mut has_recursive = false;
                let mut targets = Vec::new();

                for &arg in args {
                    let clean_arg = arg.trim_matches(|c| c == '"' || c == '\'');
                    if clean_arg.starts_with('-') && !clean_arg.starts_with("--") {
                        if clean_arg.contains('r') || clean_arg.contains('R') {
                            has_recursive = true;
                        }
                    } else if clean_arg == "--recursive" {
                        has_recursive = true;
                    } else if !clean_arg.starts_with('-') {
                        targets.push(clean_arg);
                    }
                }

                if has_recursive {
                    for target in targets {
                        let clean_target = target.trim_matches(|c| c == '"' || c == '\'');
                        if clean_target == "/"
                            || clean_target == "/*"
                            || clean_target == "*"
                            || clean_target == "."
                            || clean_target == "~"
                            || clean_target == "$home"
                            || clean_target == "$HOME"
                            || clean_target == "/."
                            || clean_target.starts_with("/etc")
                            || clean_target.starts_with("/bin")
                            || clean_target.starts_with("/usr")
                            || clean_target.starts_with("/boot")
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Split command line by operators (&&, ||, ;, |, subshells $(), backticks)
    fn split_chained_commands(cmd: &str) -> Vec<String> {
        let mut results = Vec::new();
        let mut current = String::new();
        let mut in_quote = None;
        let chars: Vec<char> = cmd.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if let Some(q) = in_quote {
                if c == q {
                    in_quote = None;
                }
                current.push(c);
            } else if c == '"' || c == '\'' {
                in_quote = Some(c);
                current.push(c);
            } else if c == ';' || c == '|' || c == '&' {
                if !current.trim().is_empty() {
                    results.push(current.trim().to_string());
                    current.clear();
                }
                if (c == '&' || c == '|') && i + 1 < chars.len() && chars[i + 1] == c {
                    i += 1;
                }
            } else {
                current.push(c);
            }
            i += 1;
        }

        if !current.trim().is_empty() {
            results.push(current.trim().to_string());
        }

        // Also unpack subshells: `$(cmd)`, `bash -c "cmd"`, `sh -c "cmd"`, `eval "cmd"`, and backticks
        let mut expanded = results.clone();
        for s in &results {
            if let Some(start) = s.find("$(") {
                if let Some(end) = s[start + 2..].find(')') {
                    expanded.push(s[start + 2..start + 2 + end].trim().to_string());
                }
            }
            if let Some(start) = s.find('`') {
                if let Some(end) = s[start + 1..].find('`') {
                    expanded.push(s[start + 1..start + 1 + end].trim().to_string());
                }
            }
            if s.contains("-c ") {
                if let Some(idx) = s.find("-c ") {
                    let sub = s[idx + 3..].trim().trim_matches(|c| c == '\'' || c == '"');
                    expanded.push(sub.to_string());
                }
            }
            if s.starts_with("eval ") || s.contains("eval ") {
                if let Some(idx) = s.find("eval ") {
                    let sub = s[idx + 5..].trim().trim_matches(|c| c == '\'' || c == '"');
                    expanded.push(sub.to_string());
                }
            }
        }

        expanded
    }

    /// Normalize command by lowercasing, stripping extra quotes, and collapsing whitespace
    fn normalize_command(cmd: &str) -> String {
        let lower = cmd.to_lowercase();
        let collapsed: Vec<&str> = lower.split_whitespace().collect();
        collapsed.join(" ")
    }

    /// Decode URL-encoded characters (e.g. `%2e%2e%2f` -> `../`)
    fn url_decode(s: &str) -> String {
        let mut res = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                    if let Ok(byte_val) = u8::from_str_radix(hex_str, 16) {
                        res.push(byte_val as char);
                        i += 3;
                        continue;
                    }
                }
            }
            res.push(bytes[i] as char);
            i += 1;
        }
        res
    }

    /// Scan file paths for path traversal or sensitive system file access
    pub fn scan_file_path(path_str: &str) -> AgentShieldVerdict {
        // 1. URL decode path in case of encoded traversal attacks like %2e%2e%2f
        let decoded = Self::url_decode(path_str);

        // 2. Normalize Windows backslashes and redundant slashes
        let mut normalized = decoded.replace('\\', "/").to_lowercase();

        // Strip leading Windows drive letter e.g. "c:/" -> "/"
        if normalized.len() >= 2
            && normalized.chars().next().unwrap().is_ascii_alphabetic()
            && normalized.chars().nth(1) == Some(':')
        {
            normalized = normalized[2..].to_string();
        }

        // 3. Path traversal check: any attempt to traverse upwards
        let segments: Vec<&str> = normalized
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        let has_traversal = segments.iter().any(|&s| s == "..")
            || normalized.contains("/../")
            || normalized.starts_with("../")
            || normalized == "..";

        if has_traversal {
            return AgentShieldVerdict::Block {
                reason: "Directory traversal pattern ('..') detected".to_string(),
                threat_level: ThreatLevel::High,
            };
        }

        // Canonical virtual path by resolving segments
        let mut clean_segments = Vec::new();
        for seg in segments {
            if seg == ".." {
                clean_segments.pop();
            } else {
                clean_segments.push(seg);
            }
        }
        let clean_path = format!("/{}", clean_segments.join("/"));

        // 4. Sensitive system targets
        let sensitive = [
            ("/etc/shadow", ThreatLevel::Critical),
            ("/etc/passwd", ThreatLevel::Medium),
            ("/etc/sudoers", ThreatLevel::Critical),
            ("/proc/kcore", ThreatLevel::Critical),
            ("/proc/mem", ThreatLevel::Critical),
            ("/dev/mem", ThreatLevel::Critical),
            (".ssh/id_rsa", ThreatLevel::High),
            (".ssh/id_ed25519", ThreatLevel::High),
            ("id_rsa", ThreatLevel::High),
            ("id_ed25519", ThreatLevel::High),
            ("config/sam", ThreatLevel::Critical),
            ("system32/config/sam", ThreatLevel::Critical),
        ];

        for (target, level) in sensitive {
            if clean_path == target || clean_path.ends_with(target) || normalized.ends_with(target) {
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
            ("ghp_", "[REDACTED_GITHUB_TOKEN]"),
        ];

        for (prefix, replacement) in patterns {
            let mut search_from = 0;
            while search_from < sanitized.len() {
                if let Some(rel_pos) = sanitized[search_from..].find(prefix) {
                    let pos = search_from + rel_pos;
                    // Token boundary check: whitespace, quotes, punctuation brackets, comma, semicolon
                    let end = sanitized[pos..]
                        .find(|c: char| {
                            c.is_whitespace()
                                || c == '"'
                                || c == '\''
                                || c == '`'
                                || c == '\n'
                                || c == '\r'
                                || c == ','
                                || c == ';'
                                || c == ')'
                                || c == ']'
                                || c == '}'
                                || c == '>'
                                || c == '\\'
                        })
                        .map(|i| pos + i)
                        .unwrap_or(sanitized.len());

                    sanitized.replace_range(pos..end, replacement);
                    search_from = pos + replacement.len();
                } else {
                    break;
                }
            }
        }

        sanitized
    }
}
