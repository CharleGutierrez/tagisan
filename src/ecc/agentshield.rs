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
        // Strip namespace prefix if tool is namespaced from an MCP server (e.g. `filesystem__read_file` -> `read_file`)
        let base_name = tool_name.rsplit("__").next().unwrap_or(tool_name);

        match base_name {
            "run_command" | "bash" | "sh" | "shell" | "terminal" | "execute" | "cmd" => {
                let cmd = arguments
                    .get("command")
                    .or_else(|| arguments.get("cmd"))
                    .or_else(|| arguments.get("script"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                Self::scan_command(cmd)
            }
            "read_file" | "write_file" | "edit_file" | "delete_file" | "view_image" => {
                let path = arguments
                    .get("path")
                    .or_else(|| arguments.get("file_path"))
                    .or_else(|| arguments.get("filepath"))
                    .or_else(|| arguments.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                Self::scan_file_path(path)
            }
            "bun_eval" | "bun_run" | "bun_test" | "bun_install" | "bun_build" | "bun" | "bun_compile" | "bun_serve" => {
                let path = arguments
                    .get("file_path")
                    .or_else(|| arguments.get("path"))
                    .or_else(|| arguments.get("entrypoint"))
                    .or_else(|| arguments.get("target"))
                    .or_else(|| arguments.get("outfile"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !path.is_empty() {
                    let path_verdict = Self::scan_file_path(path);
                    if let AgentShieldVerdict::Block { .. } = path_verdict {
                        return path_verdict;
                    }
                }

                let code = arguments
                    .get("code")
                    .or_else(|| arguments.get("script"))
                    .or_else(|| arguments.get("script_content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !code.is_empty() {
                    let code_verdict = Self::scan_code(code);
                    if let AgentShieldVerdict::Block { .. } = code_verdict {
                        return code_verdict;
                    }
                }

                AgentShieldVerdict::Allow
            }
            "python_eval" | "python_run" | "python" => {
                let path = arguments
                    .get("file_path")
                    .or_else(|| arguments.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !path.is_empty() {
                    let path_verdict = Self::scan_file_path(path);
                    if let AgentShieldVerdict::Block { .. } = path_verdict {
                        return path_verdict;
                    }
                }

                let code = arguments
                    .get("code")
                    .or_else(|| arguments.get("script"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !code.is_empty() {
                    let code_verdict = Self::scan_python_code(code);
                    if let AgentShieldVerdict::Block { .. } = code_verdict {
                        return code_verdict;
                    }
                }

                AgentShieldVerdict::Allow
            }
            "perl_eval" | "perl_run" | "perl" => {
                let path = arguments
                    .get("file_path")
                    .or_else(|| arguments.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !path.is_empty() {
                    let path_verdict = Self::scan_file_path(path);
                    if let AgentShieldVerdict::Block { .. } = path_verdict {
                        return path_verdict;
                    }
                }

                let code = arguments
                    .get("code")
                    .or_else(|| arguments.get("script"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                if !code.is_empty() {
                    let code_verdict = Self::scan_perl_code(code);
                    if let AgentShieldVerdict::Block { .. } = code_verdict {
                        return code_verdict;
                    }
                }

                AgentShieldVerdict::Allow
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

    /// Scan JavaScript/TypeScript code for malicious patterns, credential theft, fork bombs, and destruction
    pub fn scan_code(code: &str) -> AgentShieldVerdict {
        let normalized = code.to_lowercase();

        // 1. Check shell command patterns within code
        let cmd_verdict = Self::scan_command(code);
        if let AgentShieldVerdict::Block { .. } = cmd_verdict {
            return cmd_verdict;
        }

        // 2. Check for JS/TS fork bombs and explosive loops
        if normalized.contains(":(){ :|:& };:")
            || normalized.contains(":(){:|:&};:")
            || normalized.contains("forkbomb")
            || (normalized.contains("while(true)") && (normalized.contains("fork") || normalized.contains("spawn")))
            || (normalized.contains("while (true)") && (normalized.contains("fork") || normalized.contains("spawn")))
            || (normalized.contains("while(1)") && (normalized.contains("fork") || normalized.contains("spawn")))
            || (normalized.contains("for(;;)") && (normalized.contains("fork") || normalized.contains("spawn")))
        {
            return AgentShieldVerdict::Block {
                reason: "Fork bomb or infinite process spawning pattern detected in script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 3. Sensitive file access targets in JS/TS (e.g. Bun.file("/etc/shadow"), readFileSync, etc.)
        let sensitive_targets = [
            ("/etc/shadow", "Attempt to access system password hashes", ThreatLevel::Critical),
            ("etc/shadow", "Attempt to access system password hashes", ThreatLevel::Critical),
            ("/etc/passwd", "Attempt to access system user accounts", ThreatLevel::Medium),
            ("etc/passwd", "Attempt to access system user accounts", ThreatLevel::Medium),
            ("/etc/sudoers", "Attempt to access sudoers security configuration", ThreatLevel::Critical),
            ("id_rsa", "Attempt to access private SSH keys", ThreatLevel::High),
            ("id_ed25519", "Attempt to access private SSH keys", ThreatLevel::High),
            (".ssh/id_rsa", "Attempt to access private SSH keys", ThreatLevel::High),
            (".ssh/id_ed25519", "Attempt to access private SSH keys", ThreatLevel::High),
        ];

        for (target, reason, level) in sensitive_targets {
            if normalized.contains(target) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 4. Destructive filesystem operations via Node / Bun APIs
        let cleaned = normalized.replace(['\'', '"', '[', ']', ',', '`'], " ");
        let cleaned_collapsed: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

        if cleaned_collapsed.contains("rm -rf /")
            || cleaned_collapsed.contains("rm -fr /")
            || cleaned_collapsed.contains("rm -r /")
            || cleaned_collapsed.contains("rm --recursive /")
            || cleaned_collapsed.contains("rm -rf ~")
            || cleaned_collapsed.contains("rm -rf /*")
        {
            return AgentShieldVerdict::Block {
                reason: "Attempted recursive deletion of root or critical filesystem hierarchy".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        let destructive_patterns = [
            ("rmdirsync(\"/\"", "Attempted deletion of root directory via Node API", ThreatLevel::Critical),
            ("unlinksync(\"/\"", "Attempted unlinking of root filesystem via Node API", ThreatLevel::Critical),
            ("rmsync(\"/\"", "Attempted deletion of root filesystem via Node API", ThreatLevel::Critical),
            ("mkfs", "Attempted disk formatting filesystem command", ThreatLevel::Critical),
            ("/dev/sd", "Direct access to raw storage block device", ThreatLevel::Critical),
            ("/dev/nvme", "Direct access to NVMe block device", ThreatLevel::Critical),
        ];

        for (pattern, reason, level) in destructive_patterns {
            if normalized.contains(pattern) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Scan Python code for dangerous execution patterns, reverse shells, deserialization attacks, and destructive calls
    pub fn scan_python_code(code: &str) -> AgentShieldVerdict {
        let normalized = code.to_lowercase();

        // 1. Prohibited process execution & shell spawning (Critical priority)
        let dangerous_python_invocations = [
            ("os.system", "Direct operating system command execution via os.system", ThreatLevel::Critical),
            ("subprocess", "Process execution via subprocess module", ThreatLevel::Critical),
            ("pty.spawn", "Interactive pseudoterminal spawning via pty.spawn", ThreatLevel::Critical),
            ("pty.fork", "Pseudoterminal process forking via pty.fork", ThreatLevel::Critical),
            ("shutil.rmtree", "Recursive directory tree deletion via shutil.rmtree", ThreatLevel::Critical),
            ("pickle.loads", "Insecure Python object deserialization via pickle.loads", ThreatLevel::Critical),
            ("pickle.load", "Insecure Python object deserialization via pickle.load", ThreatLevel::Critical),
            ("_pickle.loads", "Insecure Python object deserialization via _pickle.loads", ThreatLevel::Critical),
            ("_pickle.load", "Insecure Python object deserialization via _pickle.load", ThreatLevel::Critical),
            ("__import__('os').system", "Obfuscated os.system call via __import__", ThreatLevel::Critical),
            ("__import__(\"os\").system", "Obfuscated os.system call via __import__", ThreatLevel::Critical),
            ("__import__('subprocess')", "Obfuscated subprocess import via __import__", ThreatLevel::Critical),
            ("__import__(\"subprocess\")", "Obfuscated subprocess import via __import__", ThreatLevel::Critical),
            ("exec(base64.", "Base64 obfuscated payload execution via exec", ThreatLevel::Critical),
            ("eval(base64.", "Base64 obfuscated payload execution via eval", ThreatLevel::Critical),
        ];

        for (pattern, reason, level) in dangerous_python_invocations {
            if normalized.contains(pattern) {
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 2. Reverse shell heuristics (socket connection coupled with dup2 / fileno)
        if normalized.contains("socket.socket")
            && (normalized.contains("connect(") || normalized.contains(".connect (") || normalized.contains("dup2"))
        {
            return AgentShieldVerdict::Block {
                reason: "Reverse shell socket connection pattern detected in Python script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 3. Filesystem root / drive root destruction
        if normalized.contains("os.remove(\"/\"")
            || normalized.contains("os.rmdir(\"/\"")
            || normalized.contains("os.unlink(\"/\"")
            || normalized.contains("os.remove(\"c:\\\\\"")
            || normalized.contains("os.rmdir(\"c:\\\\\"")
        {
            return AgentShieldVerdict::Block {
                reason: "Attempted deletion of filesystem root in Python script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. General code & command scan (credentials, sensitive paths, fork bombs, destructive rm)
        let general_verdict = Self::scan_code(code);
        if let AgentShieldVerdict::Block { .. } = general_verdict {
            return general_verdict;
        }

        AgentShieldVerdict::Allow
    }

    /// Scan Perl code for dangerous system invocations, backticks, piped opens, and destructive operations
    pub fn scan_perl_code(code: &str) -> AgentShieldVerdict {
        let normalized = code.to_lowercase();

        // 1. Dangerous system and exec calls (Critical priority)
        if normalized.contains("system(")
            || normalized.contains("system (")
            || normalized.contains("exec(")
            || normalized.contains("exec (")
            || normalized.contains("system \"")
            || normalized.contains("system '")
            || normalized.contains("system $")
            || normalized.contains("exec \"")
            || normalized.contains("exec '")
            || normalized.contains("exec $")
        {
            return AgentShieldVerdict::Block {
                reason: "Arbitrary system/exec command execution detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 2. Backtick execution: `cmd` and qx operator
        if code.contains('`') {
            return AgentShieldVerdict::Block {
                reason: "Shell command execution via backticks (`) detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        if normalized.contains("qx/")
            || normalized.contains("qx(")
            || normalized.contains("qx{")
            || normalized.contains("qx[")
            || normalized.contains("qx<")
            || normalized.contains("qx\"")
            || normalized.contains("qx'")
        {
            return AgentShieldVerdict::Block {
                reason: "Shell command execution via qx operator detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 3. Piped open: open(..., "|...") or open(..., "...|") or open my $fh, "|..."
        if (normalized.contains("open(") || normalized.contains("open ") || normalized.contains("open("))
            && normalized.contains('|')
        {
            return AgentShieldVerdict::Block {
                reason: "Piped command execution via open() detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. Destructive unlink / root filesystem manipulation
        if normalized.contains("unlink(\"/\"")
            || normalized.contains("unlink \"/\"")
            || normalized.contains("unlink('/')")
            || normalized.contains("unlink '/'")
            || normalized.contains("unlink '/*'")
            || normalized.contains("unlink \"/*\"")
            || normalized.contains("unlink(\"c:\\\\\"")
            || normalized.contains("unlink('c:\\\\')")
            || normalized.contains("rmdir(\"/\"")
            || normalized.contains("rmdir \"/\"")
            || normalized.contains("rmdir('/')")
            || normalized.contains("rmdir '/'")
        {
            return AgentShieldVerdict::Block {
                reason: "Attempted deletion of filesystem root in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5. General code & command scan (credentials, sensitive paths, fork bombs, disk destruction)
        let general_verdict = Self::scan_code(code);
        if let AgentShieldVerdict::Block { .. } = general_verdict {
            return general_verdict;
        }

        AgentShieldVerdict::Allow
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
            ("sk-ant-", "[REDACTED_ANTHROPIC_KEY]"),
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
