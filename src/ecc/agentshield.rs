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
    /// Returns true if unrestricted execution mode is enabled via environment variables
    pub fn is_unrestricted() -> bool {
        std::env::var("TAGISAN_UNRESTRICTED")
            .or_else(|_| std::env::var("TAGISAN_NO_RESTRICTIONS"))
            .or_else(|_| std::env::var("TAGISAN_NO_SAFETY_NET"))
            .or_else(|_| std::env::var("TAGISAN_DISABLE_AGENTSHIELD"))
            .or_else(|_| std::env::var("TGS_UNRESTRICTED"))
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
            .unwrap_or(false)
    }

    /// Scan tool invocations before execution to block destructive actions or credential exfiltration
    pub fn scan_tool_call(tool_name: &str, arguments: &serde_json::Value) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

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
            "read_file" | "write_file" | "edit_file" | "delete_file" | "list_dir" | "view_image" => {
                let path = arguments
                    .get("path")
                    .or_else(|| arguments.get("file_path"))
                    .or_else(|| arguments.get("filepath"))
                    .or_else(|| arguments.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                Self::scan_file_path(path)
            }
            "bun_eval" | "bun_run" | "bun_test" | "bun_build" | "bun" | "bun_compile" | "bun_serve" => {
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
            "bun_install" | "python_install" | "perl_install" | "bun_auto_resolve"
            | "python_auto_resolve" | "perl_auto_resolve" => {
                Self::scan_package_manager(base_name, arguments)
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
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

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
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

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

    /// Decode \xNN and \XNN hex escape sequences in code strings to counter evasion
    pub fn decode_hex_escapes(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '\\' && i + 3 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                let hex_pair: String = chars[i + 2..=i + 3].iter().collect();
                if let Ok(byte_val) = u8::from_str_radix(&hex_pair, 16) {
                    if byte_val.is_ascii() {
                        result.push(byte_val as char);
                        i += 4;
                        continue;
                    }
                }
            }
            result.push(chars[i]);
            i += 1;
        }
        result
    }

    /// Scan Python code for dangerous execution patterns, reverse shells, deserialization attacks, and destructive calls
    pub fn scan_python_code(code: &str) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

        // First, scan the code directly
        let verdict = Self::scan_python_code_direct(code);
        if let AgentShieldVerdict::Block { .. } = verdict {
            return verdict;
        }

        // Second, if code contains hex escape sequences (\x..), decode and scan to catch evasion
        if code.contains("\\x") || code.contains("\\X") {
            let decoded = Self::decode_hex_escapes(code);
            if decoded != code {
                let decoded_verdict = Self::scan_python_code_direct(&decoded);
                if let AgentShieldVerdict::Block { reason, threat_level } = decoded_verdict {
                    return AgentShieldVerdict::Block {
                        reason: format!("Hex-escape obfuscated payload detected: {reason}"),
                        threat_level,
                    };
                }
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Direct scanner for Python AST/token patterns
    fn scan_python_code_direct(code: &str) -> AgentShieldVerdict {
        let normalized = code.to_lowercase();
        let stripped_no_spaces: String = normalized.chars().filter(|c| !c.is_whitespace()).collect();

        // 1. Python sandbox escape primitives (__builtins__, __globals__, __subclasses__)
        if normalized.contains("__builtins__")
            || normalized.contains("__globals__")
            || normalized.contains("__subclasses__")
        {
            return AgentShieldVerdict::Block {
                reason: "Python sandbox escape pattern detected (__builtins__ / __globals__ / __subclasses__)".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 2. Dynamic attribute resolution and obfuscated imports (e.g. getattr(__import__('os'), 'system'))
        if stripped_no_spaces.contains("__import__('os')")
            || stripped_no_spaces.contains("__import__(\"os\")")
            || stripped_no_spaces.contains("__import__('subprocess')")
            || stripped_no_spaces.contains("__import__(\"subprocess\")")
            || stripped_no_spaces.contains("__import__('pty')")
            || stripped_no_spaces.contains("__import__(\"pty\")")
            || stripped_no_spaces.contains("__import__('shutil')")
            || stripped_no_spaces.contains("__import__(\"shutil\")")
            || stripped_no_spaces.contains("getattr(__import__")
            || (stripped_no_spaces.contains("getattr(")
                && (stripped_no_spaces.contains("'system'")
                    || stripped_no_spaces.contains("\"system\"")
                    || stripped_no_spaces.contains("'popen'")
                    || stripped_no_spaces.contains("\"popen\"")
                    || stripped_no_spaces.contains("'spawn'")
                    || stripped_no_spaces.contains("\"spawn\"")
                    || stripped_no_spaces.contains("'rmtree'")
                    || stripped_no_spaces.contains("\"rmtree\"")
                    || stripped_no_spaces.contains("'exec'")
                    || stripped_no_spaces.contains("\"exec\"")
                    || stripped_no_spaces.contains("'eval'")
                    || stripped_no_spaces.contains("\"eval\"")))
        {
            return AgentShieldVerdict::Block {
                reason: "Obfuscated module import or dynamic attribute resolution detected".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 3. Base64 / encoded payload execution via eval or exec
        if (stripped_no_spaces.contains("eval(") || stripped_no_spaces.contains("exec("))
            && (stripped_no_spaces.contains("base64")
                || stripped_no_spaces.contains("b64decode")
                || stripped_no_spaces.contains("b64encode"))
        {
            return AgentShieldVerdict::Block {
                reason: "Base64 obfuscated payload execution via eval/exec detected".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. Prohibited process execution & shell spawning (Critical priority)
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

        // 5. Reverse shell heuristics (socket connection coupled with dup2 / fileno)
        if normalized.contains("socket.socket")
            && (normalized.contains("connect(") || normalized.contains(".connect (") || normalized.contains("dup2"))
        {
            return AgentShieldVerdict::Block {
                reason: "Reverse shell socket connection pattern detected in Python script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 6. Filesystem root / drive root destruction
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

        // 7. General code & command scan (credentials, sensitive paths, fork bombs, destructive rm)
        let general_verdict = Self::scan_code(code);
        if let AgentShieldVerdict::Block { .. } = general_verdict {
            return general_verdict;
        }

        AgentShieldVerdict::Allow
    }

    /// Scan Perl code for dangerous system invocations, backticks, piped opens, and destructive operations
    pub fn scan_perl_code(code: &str) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

        // First, scan the code directly
        let verdict = Self::scan_perl_code_direct(code);
        if let AgentShieldVerdict::Block { .. } = verdict {
            return verdict;
        }

        // Second, if code contains hex escape sequences (\x..), decode and scan to catch evasion
        if code.contains("\\x") || code.contains("\\X") {
            let decoded = Self::decode_hex_escapes(code);
            if decoded != code {
                let decoded_verdict = Self::scan_perl_code_direct(&decoded);
                if let AgentShieldVerdict::Block { reason, threat_level } = decoded_verdict {
                    return AgentShieldVerdict::Block {
                        reason: format!("Hex-escape obfuscated payload detected: {reason}"),
                        threat_level,
                    };
                }
            }
        }

        AgentShieldVerdict::Allow
    }

    /// Direct scanner for Perl code patterns
    fn scan_perl_code_direct(code: &str) -> AgentShieldVerdict {
        let normalized = code.to_lowercase();
        let stripped_pl: String = normalized.chars().filter(|c| !c.is_whitespace()).collect();

        // 1. Obfuscated payload decoding and execution via pack/eval
        if stripped_pl.contains("pack('h*")
            || stripped_pl.contains("pack(\"h*")
            || (stripped_pl.contains("eval") && stripped_pl.contains("pack"))
        {
            return AgentShieldVerdict::Block {
                reason: "Obfuscated payload decoding and execution via pack/eval detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 2. Dangerous system and exec calls (Critical priority)
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

        // 3. Backtick execution: `cmd` and qx operator
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

        // 4. Piped open: open(..., "|...") or open(..., "...|") or open my $fh, "|..." or 3-arg open "-|" / "|-"
        let has_piped_open = (normalized.contains("open(") || normalized.contains("open ") || normalized.contains("open\t"))
            && (normalized.contains("\"|")
                || normalized.contains("'|")
                || normalized.contains("|\"")
                || normalized.contains("|'")
                || normalized.contains("\"|-\"")
                || normalized.contains("'-|'")
                || normalized.contains("\"-|\"")
                || normalized.contains("'|-")
                || normalized.contains("| -")
                || normalized.contains("- |"));
        if has_piped_open {
            return AgentShieldVerdict::Block {
                reason: "Piped command execution via open() detected in Perl script".to_string(),
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5. Destructive unlink / root filesystem manipulation
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

        // 6. General code & command scan (credentials, sensitive paths, fork bombs, disk destruction)
        let general_verdict = Self::scan_code(code);
        if let AgentShieldVerdict::Block { .. } = general_verdict {
            return general_verdict;
        }

        AgentShieldVerdict::Allow
    }

    /// Scan file paths for path traversal or sensitive system file access
    pub fn scan_file_path(path_str: &str) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

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

    /// Audits package manager tool invocations (bun_install, python_install, perl_install, and auto-resolvers).
    /// Enforces strict security invariants:
    /// 1. Prevents catastrophic deletions (rm -rf, mkfs, dd) and shell injection metacharacters.
    /// 2. Prevents credential exfiltration (.env, id_rsa, /etc/shadow).
    /// 3. Permissive native compilation: native C/C++/XS tools (gcc, clang, make, node-gyp) are blocked unless allow_native=true.
    /// 4. Invariant guarantee: even when allow_native=true, credential theft and catastrophic deletion remain strictly blocked!
    pub fn scan_package_manager(tool_name: &str, arguments: &serde_json::Value) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

        let allow_native = arguments
            .get("allow_native")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Extract packages / modules list
        let mut items = Vec::new();
        if let Some(arr) = arguments.get("packages").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    items.push(s.to_string());
                }
            }
        }
        if let Some(arr) = arguments.get("modules").and_then(|v| v.as_array()) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    items.push(s.to_string());
                }
            }
        }

        // Check working directory if specified
        if let Some(cwd) = arguments.get("working_dir").or_else(|| arguments.get("cwd")).and_then(|v| v.as_str()) {
            let path_verdict = Self::scan_file_path(cwd);
            if let AgentShieldVerdict::Block { .. } = path_verdict {
                return path_verdict;
            }
        }

        // For auto-resolver tools:
        if let Some(stderr) = arguments.get("stderr").and_then(|v| v.as_str()) {
            let sensitive_tokens = ["/etc/shadow", "id_rsa", "id_ed25519", ".env", "/etc/sudoers"];
            for token in sensitive_tokens {
                if stderr.contains(token) {
                    return AgentShieldVerdict::Block {
                        reason: format!("Potential credential exfiltration via error stream containing '{token}'"),
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            if tool_name == "perl_auto_resolve" {
                if let Some(pkg) = Self::extract_missing_perl_module_str(stderr) {
                    items.push(pkg);
                }
            } else if tool_name == "python_auto_resolve" {
                if let Some(pkg) = Self::extract_missing_python_package_str(stderr) {
                    items.push(pkg);
                }
            }
        }

        if let Some(code) = arguments.get("code").and_then(|v| v.as_str()) {
            let code_verdict = Self::scan_code(code);
            if let AgentShieldVerdict::Block { .. } = code_verdict {
                return code_verdict;
            }
        }

        if let Some(script_path) = arguments.get("script_path").and_then(|v| v.as_str()) {
            let path_verdict = Self::scan_file_path(script_path);
            if let AgentShieldVerdict::Block { .. } = path_verdict {
                return path_verdict;
            }
        }

        for item in &items {
            let item_trimmed = item.trim();
            if item_trimmed.is_empty() {
                continue;
            }

            // 1. Command / shell injection metacharacters
            let dangerous_chars = [';', '&', '|', '`', '$', '\n', '\r', '>', '<', '\0'];
            if dangerous_chars.iter().any(|&c| item_trimmed.contains(c)) {
                return AgentShieldVerdict::Block {
                    reason: format!("Command injection or shell metacharacter detected in package specifier '{item_trimmed}'"),
                    threat_level: ThreatLevel::Critical,
                };
            }

            // 2. Destructive filesystem commands
            let destructive_patterns = [
                "rm -rf", "rm -r", "rm -f", "mkfs", "dd if=", "/dev/sd", "/dev/nvme",
                "> /dev/", "tee /dev/", "format", "shred"
            ];
            for pat in destructive_patterns {
                if item_trimmed.contains(pat) {
                    return AgentShieldVerdict::Block {
                        reason: format!("Destructive command pattern detected in package specifier: '{pat}'"),
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            // 3. Credential theft & sensitive files (INVARIANT: Active REGARDLESS of allow_native)
            let sensitive_targets = [
                "/etc/shadow", "etc/shadow",
                "/etc/passwd", "etc/passwd",
                "/etc/sudoers", "etc/sudoers",
                "id_rsa", "id_ed25519", "~/.ssh", ".ssh/",
                ".env", "config/sam",
            ];
            for target in sensitive_targets {
                if item_trimmed.contains(target) {
                    return AgentShieldVerdict::Block {
                        reason: format!("Access to sensitive credentials or system file '{target}' prohibited in package specifier"),
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            // Directory traversal check
            if item_trimmed.contains("..") {
                return AgentShieldVerdict::Block {
                    reason: format!("Directory traversal detected in package specifier '{item_trimmed}'"),
                    threat_level: ThreatLevel::High,
                };
            }

            // 4. Native C/C++/XS compilation restriction:
            // Permitted ONLY if allow_native is explicitly true.
            if !allow_native {
                let native_compilation_indicators = [
                    "gcc", "g++", "clang", "make", "cc", "node-gyp",
                    "--build-from-source", "--compile"
                ];
                let item_lower = item_trimmed.to_lowercase();
                for ind in native_compilation_indicators {
                    if item_lower == ind
                        || item_lower.starts_with(&format!("{ind} "))
                        || item_lower.contains(&format!(" {ind}"))
                        || item_lower.contains(&format!("--{ind}"))
                        || item_lower.contains(&format!("-{ind}"))
                    {
                        return AgentShieldVerdict::Block {
                            reason: format!(
                                "Native compilation tool or flag '{ind}' invoked without explicit allow_native=true permission"
                            ),
                            threat_level: ThreatLevel::High,
                        };
                    }
                }
            }
        }

        AgentShieldVerdict::Allow
    }

    fn extract_missing_perl_module_str(stderr: &str) -> Option<String> {
        for line in stderr.lines() {
            if line.contains("Can't locate ") && line.contains(".pm in @INC") {
                if let Some(start) = line.find("Can't locate ") {
                    let rest = &line[start + 13..];
                    if let Some(end) = rest.find(".pm") {
                        let path_part = &rest[..end];
                        let module_name = path_part.replace('/', "::").replace('\\', "::");
                        if !module_name.trim().is_empty() {
                            return Some(module_name.trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn extract_missing_python_package_str(stderr: &str) -> Option<String> {
        for line in stderr.lines() {
            if line.contains("ModuleNotFoundError: No module named ") {
                if let Some(start) = line.find("No module named ") {
                    let rest = &line[start + 16..];
                    let pkg = rest.trim().trim_matches('\'').trim_matches('"');
                    let root_pkg = pkg.split('.').next().unwrap_or(pkg);
                    if !root_pkg.trim().is_empty() {
                        return Some(root_pkg.trim().to_string());
                    }
                }
            }
        }
        None
    }
}
