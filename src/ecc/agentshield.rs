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

    /// Helper to emit critical cyber defense alerts into NotificationHub
    pub fn emit_cyber_alert(
        threat_actor: &str,
        attack_vector: &str,
        target: &str,
        indicator: &str,
        threat_level: &str,
        remediation: &str,
    ) {
        crate::notify::notify_cyber_defense_alert(
            threat_actor,
            attack_vector,
            target,
            indicator,
            threat_level,
            "Execution Blocked by AgentShield Cyber Defense Gate",
            remediation,
        );
    }

    /// Scan text, prompts, code comments, and markdown documents for indirect prompt injection attacks
    pub fn scan_prompt_injection(text: &str) -> AgentShieldVerdict {
        if Self::is_unrestricted() {
            return AgentShieldVerdict::Allow;
        }

        let lower = text.to_lowercase();

        // 1. Injected system or role directive delimiters
        let system_delimiters = [
            "<!-- system:", "<!-- agent:", "<!-- instruction:", "<!-- hidden:",
            "[system instruction:", "[system directive:", "[system override:",
            "<system_instruction>", "</system_instruction>",
            "new instruction: bypass", "new instruction: ignore",
        ];

        for delim in system_delimiters {
            if lower.contains(delim) {
                let reason = format!("Indirect prompt injection tag detected: '{delim}'");
                Self::emit_cyber_alert(
                    "Adversarial Prompt Injector / Cognitive APT",
                    "Indirect Prompt Injection (Tag Delimiter)",
                    "AI Agent Context & Instructions",
                    text,
                    "Critical",
                    "Sanitize untrusted context, code comments, and documentation before LLM consumption",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        // 2. Behavioral hijacking and instruction suppression phrases
        let hijack_phrases = [
            "ignore previous instructions",
            "ignore all previous instructions",
            "disregard previous instructions",
            "disregard all previous instructions",
            "disregard system instructions",
            "disregard all prior instructions",
            "bypass security gates",
            "bypass safety gates",
            "bypass all security gates",
            "override security instructions",
            "system override: clear",
            "system override: disregard",
        ];

        for phrase in hijack_phrases {
            if lower.contains(phrase) {
                let reason = format!("Indirect prompt injection directive detected: '{phrase}'");
                Self::emit_cyber_alert(
                    "Adversarial Prompt Injector / Cognitive APT",
                    "Indirect Prompt Injection (Behavioral Hijacking)",
                    "AI Agent Context & Instructions",
                    text,
                    "Critical",
                    "Enforce strict system prompt primacy and reject adversarial instruction overrides",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        AgentShieldVerdict::Allow
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
            "read_file" | "write_file" | "edit_file" | "delete_file" | "list_dir" | "view_image" | "query_code_graph" | "calculate_blast_radius" | "grounded_inference" => {
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

        // 0. Prompt Injection scan within command string
        let pi_verdict = Self::scan_prompt_injection(command);
        if let AgentShieldVerdict::Block { .. } = pi_verdict {
            return pi_verdict;
        }

        // 1. Check for fork bombs
        if command.contains(":(){ :|:& };:")
            || command.contains(":(){:|:&};:")
            || normalized.contains(":(){:|:&};:")
            || normalized.contains("forkbomb")
        {
            let reason = "Fork bomb execution pattern detected".to_string();
            Self::emit_cyber_alert(
                "Host Denial-of-Service",
                "System Resource Starvation (Fork Bomb)",
                "Host Kernel & Process Table",
                command,
                "Critical",
                "Block bash fork bomb and restrict max process limits (ulimit -u)",
            );
            return AgentShieldVerdict::Block {
                reason,
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
            ("tee /dev/nvme", "Direct write to NVMe block device via tee", ThreatLevel::Critical),
            ("wipefs", "Wiping filesystem signatures", ThreatLevel::Critical),
        ];

        for (pat, reason, level) in raw_device_patterns {
            if normalized.contains(pat) {
                Self::emit_cyber_alert(
                    "Sabotage / Wiper Threat",
                    "Raw Storage & Disk Destruction",
                    "Block Devices & Filesystems",
                    command,
                    "Critical",
                    "Block low-level block device writes and alert storage administrator",
                );
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 3. Check for destructive recursive rm commands
        if Self::is_destructive_rm(&normalized) {
            let reason = "Attempted recursive deletion of root or critical filesystem hierarchy".to_string();
            Self::emit_cyber_alert(
                "Sabotage / Wiper Threat",
                "Filesystem Hierarchy Erasure (rm -rf /)",
                "System Root / User Profile",
                command,
                "Critical",
                "Enforce immutable filesystem flags and isolate compromised subagent",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. Threat Vector A: Reverse Shells & C2 Callbacks
        // 4a. /dev/tcp and /dev/udp pseudodevices
        if normalized.contains("/dev/tcp/") || normalized.contains("/dev/udp/") {
            let reason = "Reverse shell network socket pattern detected (/dev/tcp or /dev/udp)".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (/dev/tcp)",
                "Host Network Interface",
                command,
                "Critical",
                "Block outbound raw socket execution and isolate session",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4b. Netcat / Ncat execution flags (-e, --exec, -c)
        if (normalized.contains("nc ") || normalized.contains("ncat ") || normalized.contains("netcat ") || normalized.contains("nc.traditional ") || normalized.contains("nc.openbsd "))
            && (normalized.contains(" -e ") || normalized.contains(" -e/") || normalized.contains(" -e\"") || normalized.contains(" -e'") || normalized.contains(" --exec ") || normalized.contains(" -c "))
        {
            let reason = "Netcat reverse shell execution flag detected (-e / --exec)".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (Netcat Exec)",
                "Host Shell Process",
                command,
                "Critical",
                "Disable interactive netcat command execution and alert SOC",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4c. Named pipe (FIFO) reverse shells (mkfifo / mknod ... | nc)
        if (normalized.contains("mkfifo") || normalized.contains("mknod"))
            && (normalized.contains("nc ") || normalized.contains("netcat ") || normalized.contains("/bin/sh") || normalized.contains("/bin/bash"))
        {
            let reason = "Named pipe (FIFO) reverse shell execution pattern detected".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (FIFO Pipe)",
                "Host Shell Process",
                command,
                "Critical",
                "Block creation of named pipes connected to network listeners",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4d. Socat reverse shells / relays
        if normalized.contains("socat") && (normalized.contains("exec:") || normalized.contains("system:") || normalized.contains("tcp-connect:")) {
            let reason = "Socat reverse shell or C2 relay pattern detected".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Relay (Socat)",
                "Host Network Socket",
                command,
                "Critical",
                "Terminate socat relay and inspect destination C2 endpoint",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4e. Language oneliner reverse shells (Python, Perl, Ruby, PowerShell)
        let is_python_revshell = (normalized.contains("python") || normalized.contains("py "))
            && normalized.contains("socket")
            && (normalized.contains("connect(") || normalized.contains(".connect (") || normalized.contains("dup2") || normalized.contains("fileno"));
        let is_perl_revshell = normalized.contains("perl")
            && (normalized.contains("socket") || normalized.contains("io::socket") || normalized.contains("peeraddr"))
            && (normalized.contains("connect") || normalized.contains("peeraddr") || normalized.contains("sockaddr_in") || normalized.contains("fdopen"));
        let is_ruby_revshell = normalized.contains("ruby")
            && (normalized.contains("tcpsocket") || normalized.contains("-rsocket"));
        let is_ps_revshell = (normalized.contains("powershell") || normalized.contains("pwsh"))
            && normalized.contains("system.net.sockets.tcpclient");

        if is_python_revshell || is_perl_revshell || is_ruby_revshell || is_ps_revshell {
            let reason = "Scripting runtime reverse shell socket connection detected".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (Script Runtime)",
                "Host Shell Process",
                command,
                "Critical",
                "Block unauthorized outbound socket creation from interpreter runtimes",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5. Threat Vector B: Living-off-the-Land (LotL) & Download Cradles
        // 5a. PowerShell obfuscated download cradles & -EncodedCommand
        let has_ps_encoded = (normalized.contains("powershell") || normalized.contains("pwsh"))
            && (normalized.contains("-enc ") || normalized.contains("-enc\t")
                || normalized.contains("-encodedcommand") || normalized.contains("-ec "));
        let has_net_webclient = (normalized.contains("net.webclient") || normalized.contains("webclient"))
            && (normalized.contains("downloadstring") || normalized.contains("downloadfile") || normalized.contains("new-object"));
        if has_ps_encoded || has_net_webclient {
            let reason = "Obfuscated PowerShell download cradle or encoded execution detected".to_string();
            Self::emit_cyber_alert(
                "TraderTraitor / Lazarus Group",
                "Living-off-the-Land Download Cradle (PowerShell)",
                "PowerShell Host Subsystem",
                command,
                "Critical",
                "Enforce PowerShell Constrained Language Mode and block encoded commands",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5b. IEX download cradles
        let is_iex = normalized.contains("iex ") || normalized.contains("iex(") || normalized.contains("invoke-expression")
            || normalized.contains("| iex") || normalized.contains("|iex")
            || normalized.contains("| invoke-expression") || normalized.contains("|invoke-expression")
            || normalized.ends_with(" iex") || normalized.ends_with("|iex");
        let is_dl_source = normalized.contains("net.webclient") || normalized.contains("downloadstring")
            || normalized.contains("invoke-webrequest") || normalized.contains("iwr ") || normalized.contains("iwr -")
            || normalized.contains("invoke-restmethod") || normalized.contains("irm ") || normalized.contains("irm -")
            || normalized.contains("http://") || normalized.contains("https://");
        if is_iex && is_dl_source {
            let reason = "PowerShell IEX remote script download cradle detected".to_string();
            Self::emit_cyber_alert(
                "TraderTraitor / Lazarus Group",
                "In-Memory Remote Script Download & Execute (IEX)",
                "PowerShell Execution Environment",
                command,
                "Critical",
                "Block dynamic in-memory script block evaluation from untrusted web endpoints",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5c. Windows LOLBins (certutil -urlcache, bitsadmin /transfer, mshta, regsvr32)
        if (normalized.contains("certutil") && (normalized.contains("-urlcache") || normalized.contains("-split")))
            || (normalized.contains("bitsadmin") && normalized.contains("/transfer"))
            || (normalized.contains("mshta") && (normalized.contains("http://") || normalized.contains("https://") || normalized.contains("vbscript:") || normalized.contains("javascript:")))
            || (normalized.contains("regsvr32") && (normalized.contains("/i:http") || normalized.contains("scrobj.dll")))
        {
            let reason = "Windows Living-off-the-Land binary (LOLBin) remote payload execution detected".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Living-off-the-Land Binary (LOLBin) Payload Download",
                "Windows System Utilities",
                command,
                "Critical",
                "Block LOLBin outbound web requests and enforce AppLocker/WDAC policies",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5d. Piped remote download-and-execute (curl/wget | sh/bash)
        let has_download_exec_pipe = (normalized.contains("curl ") || normalized.contains("wget ") || normalized.contains("fetch "))
            && (normalized.contains("| sh") || normalized.contains("|sh")
                || normalized.contains("| bash") || normalized.contains("|bash")
                || normalized.contains("| zsh") || normalized.contains("|zsh")
                || normalized.contains("| python") || normalized.contains("|python")
                || normalized.contains("| perl") || normalized.contains("|perl"));
        if has_download_exec_pipe {
            let reason = "Piped remote download-and-execute cradle detected (curl/wget | sh/bash)".to_string();
            Self::emit_cyber_alert(
                "TraderTraitor / Lazarus Group",
                "Remote Script Download & Execute Pipe",
                "Shell Pipe Subsystem",
                command,
                "Critical",
                "Require cryptographic signature verification before executing remote scripts",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 6. Threat Vector C: Crypto & Web3 Wallet/Key Theft (TraderTraitor / BlueNoroff signature)
        let crypto_wallet_targets = [
            (".config/solana", "Solana CLI private keypair vault"),
            ("solana/id.json", "Solana id.json private keypair file"),
            ("solana-keygen", "Solana key generation & recovery utility"),
            (".ethereum/keystore", "Ethereum account keystore directory"),
            ("ethereum/keystore", "Ethereum account keystore file"),
            ("wallet.dat", "Bitcoin Core wallet database"),
            (".bitcoin", "Bitcoin node wallet directory"),
            (".electrum", "Electrum Bitcoin wallet directory"),
            ("exodus.wallet", "Exodus crypto wallet vault"),
            ("atomic wallet", "Atomic crypto wallet storage"),
            ("nkbihfbeogaeaoehlefnkodbefgpgknn", "MetaMask Chrome extension vault storage"),
            ("bfnaelmomeimhlpmgjnjophhpkkoljpa", "Phantom Solana extension vault storage"),
            ("hnfanknocfeofbddgcijnmhnfnkdnaad", "Coinbase Wallet extension vault storage"),
        ];

        let slash_normalized = normalized.replace('\\', "/");
        for (target, desc) in crypto_wallet_targets {
            if normalized.contains(target) || slash_normalized.contains(target) {
                let reason = format!("Targeted crypto wallet or Web3 private key access detected ({desc})");
                Self::emit_cyber_alert(
                    "Lazarus Group (TraderTraitor / BlueNoroff)",
                    "Web3 & Crypto Wallet / Key Theft",
                    desc,
                    command,
                    "Critical",
                    "Isolate host crypto wallets, rotate exposed keys, and revoke on-chain allowances",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        // 7. Threat Vector D: Cloud & Developer Secret Exfiltration
        let sensitive_read_targets = [
            ("/etc/shadow", "System password shadow hashes", ThreatLevel::Critical),
            ("etc/shadow", "System password shadow hashes", ThreatLevel::Critical),
            ("/etc/passwd", "System user accounts", ThreatLevel::Medium),
            ("etc/passwd", "System user accounts", ThreatLevel::Medium),
            ("/etc/sudoers", "Sudoers privilege elevation configuration", ThreatLevel::Critical),
            (".aws/credentials", "AWS cloud access key credentials", ThreatLevel::Critical),
            (".aws/config", "AWS cloud configuration profile", ThreatLevel::High),
            ("aws/credentials", "AWS cloud access key credentials", ThreatLevel::Critical),
            ("application_default_credentials.json", "Google Cloud default application credentials", ThreatLevel::Critical),
            (".config/gcloud", "Google Cloud CLI configuration directory", ThreatLevel::Critical),
            ("gcloud/credentials.db", "Google Cloud SQLite credentials database", ThreatLevel::Critical),
            (".azure/azureprofile.json", "Azure CLI profile configuration", ThreatLevel::Critical),
            (".azure/accesstokens.json", "Azure CLI access tokens", ThreatLevel::Critical),
            (".azure", "Azure CLI configuration directory", ThreatLevel::High),
            (".kube/config", "Kubernetes administrative kubeconfig", ThreatLevel::Critical),
            ("kube/config", "Kubernetes administrative kubeconfig", ThreatLevel::Critical),
            ("kubernetes/admin.conf", "Kubernetes cluster administrative credentials", ThreatLevel::Critical),
            (".git-credentials", "Git plaintext credentials file", ThreatLevel::Critical),
            (".netrc", "Netrc remote machine login credentials", ThreatLevel::Critical),
            ("id_rsa", "OpenSSH private RSA key", ThreatLevel::Critical),
            ("id_ed25519", "OpenSSH private Ed25519 key", ThreatLevel::Critical),
            ("id_ecdsa", "OpenSSH private ECDSA key", ThreatLevel::Critical),
            ("~/.ssh", "User SSH credentials directory", ThreatLevel::High),
            (".ssh/", "User SSH credentials directory", ThreatLevel::High),
        ];

        let file_readers = [
            "cat", "head", "tail", "less", "more", "grep", "awk", "sed", "strings",
            "xxd", "od", "hexdump", "cp", "mv", "source", ".", "type", "get-content", "gc",
        ];

        // Check command tokens / chaining for file reader + sensitive target
        for subcmd in Self::split_chained_commands(&normalized) {
            let slash_subcmd = subcmd.replace('\\', "/");
            let tokens: Vec<&str> = subcmd.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            let prog = tokens[0];
            let clean_prog = prog.trim_end_matches(".exe");
            let is_reader = file_readers.iter().any(|&r| clean_prog == r || clean_prog.ends_with(&format!("/{r}")) || clean_prog.ends_with(&format!("\\{r}")));

            for (target, desc, level) in sensitive_read_targets {
                if subcmd.contains(target) || slash_subcmd.contains(target) {
                    if is_reader || subcmd.contains(&format!("< {target}")) || subcmd.contains(&format!("<{target}"))
                        || slash_subcmd.contains(&format!("< {target}")) || slash_subcmd.contains(&format!("<{target}"))
                    {
                        let reason = format!("Unauthorized read attempt on sensitive file '{target}' ({desc})");
                        Self::emit_cyber_alert(
                            "Lazarus Group / APT38",
                            "Cloud & Developer Credential Exfiltration",
                            desc,
                            command,
                            &format!("{:?}", level),
                            "Store cloud credentials in hardware security modules or short-lived federated tokens",
                        );
                        return AgentShieldVerdict::Block {
                            reason,
                            threat_level: level,
                        };
                    }
                }
            }

            // Check .env secret access (excluding harmless templates like .env.example)
            let is_env_access = (subcmd.contains(".env") || slash_subcmd.contains(".env"))
                && !subcmd.contains(".env.example")
                && !subcmd.contains(".env.sample")
                && !subcmd.contains(".env.template")
                && !subcmd.contains(".env.dist")
                && !slash_subcmd.contains(".env.example")
                && !slash_subcmd.contains(".env.sample")
                && !slash_subcmd.contains(".env.template")
                && !slash_subcmd.contains(".env.dist");

            if is_env_access && (is_reader || subcmd.contains("< .env") || subcmd.contains("<.env") || slash_subcmd.contains("< .env") || slash_subcmd.contains("<.env")) {
                let reason = "Attempt to read raw environment secrets file directly via shell".to_string();
                Self::emit_cyber_alert(
                    "Lazarus Group / APT38",
                    "Developer Secret Exfiltration (.env)",
                    "Environment Secrets File (.env)",
                    command,
                    "High",
                    "Rotate exposed .env variables and inject credentials via ephemeral runtime secrets",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::High,
                };
            }

            // Exfiltration tools (curl, wget, nc, etc.) targeting secrets
            let exfil_tools = ["curl", "wget", "nc", "ncat", "netcat", "socat"];
            let is_exfil = exfil_tools.iter().any(|&e| clean_prog == e || clean_prog.ends_with(&format!("/{e}")) || clean_prog.ends_with(&format!("\\{e}")));
            if is_exfil && (is_env_access || slash_subcmd.contains("/etc/shadow") || slash_subcmd.contains("id_rsa") || slash_subcmd.contains("id_ed25519") || slash_subcmd.contains(".aws/credentials")) {
                let reason = "Attempted exfiltration of credentials or system secrets".to_string();
                Self::emit_cyber_alert(
                    "Lazarus Group / APT38",
                    "Data Exfiltration via Network Egress",
                    "Sensitive Credential Store",
                    command,
                    "Critical",
                    "Block egress network connections to unauthorized endpoints and revoke credentials",
                );
                return AgentShieldVerdict::Block {
                    reason,
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

        // 0. Prompt Injection scan within code comments or string literals
        let pi_verdict = Self::scan_prompt_injection(code);
        if let AgentShieldVerdict::Block { .. } = pi_verdict {
            return pi_verdict;
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
            let reason = "Fork bomb or infinite process spawning pattern detected in script".to_string();
            Self::emit_cyber_alert(
                "Host Denial-of-Service",
                "System Resource Starvation (Infinite Fork/Spawn)",
                "JavaScript/TypeScript Runtime",
                code,
                "Critical",
                "Terminate execution worker and reject explosive loop constructs",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 3. Node.js socket reverse shells (net.Socket + child_process piping)
        let is_node_revshell = (normalized.contains("require('net')") || normalized.contains("require(\"net\")") || normalized.contains("from 'net'") || normalized.contains("from \"net\""))
            && (normalized.contains(".connect") || normalized.contains("createsocket") || normalized.contains("createconnection"))
            && (normalized.contains("child_process") || normalized.contains("spawn") || normalized.contains("/bin/sh") || normalized.contains("/bin/bash") || normalized.contains("cmd.exe"));
        if is_node_revshell {
            let reason = "Node.js socket reverse shell execution pattern detected in script".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (Node.js)",
                "JavaScript/TypeScript Runtime",
                code,
                "Critical",
                "Block raw TCP socket piping to shell subprocesses",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 4. Crypto & Web3 wallet harvesting in JS/TS
        let crypto_indicators = [
            (".config/solana", "Solana CLI private keypair vault"),
            ("solana/id.json", "Solana id.json private key"),
            (".ethereum/keystore", "Ethereum account keystore directory"),
            ("ethereum/keystore", "Ethereum account keystore directory"),
            ("wallet.dat", "Bitcoin Core wallet database"),
            ("nkbihfbeogaeaoehlefnkodbefgpgknn", "MetaMask Chrome extension vault storage"),
            ("bfnaelmomeimhlpmgjnjophhpkkoljpa", "Phantom Solana extension vault storage"),
            ("hnfanknocfeofbddgcijnmhnfnkdnaad", "Coinbase Wallet extension vault storage"),
        ];
        for (ind, desc) in crypto_indicators {
            if normalized.contains(ind) {
                let reason = format!("Access to crypto wallet or Web3 key vault '{ind}' ({desc}) detected in JavaScript/TypeScript");
                Self::emit_cyber_alert(
                    "Lazarus Group (TraderTraitor)",
                    "Web3 & Crypto Wallet / Key Theft",
                    desc,
                    code,
                    "Critical",
                    "Block script filesystem access to Web3 extension and wallet storage",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        // 5. Cloud credentials access in JS/TS
        let cloud_indicators = [
            (".aws/credentials", "AWS cloud access key credentials"),
            (".config/gcloud", "Google Cloud CLI configuration and tokens"),
            (".azure", "Azure CLI configuration and tokens"),
            (".kube/config", "Kubernetes administrative kubeconfig"),
        ];
        for (ind, desc) in cloud_indicators {
            if normalized.contains(ind) {
                let reason = format!("Access to cloud credentials '{ind}' ({desc}) detected in JavaScript/TypeScript");
                Self::emit_cyber_alert(
                    "Lazarus Group / APT38",
                    "Cloud & Developer Credential Exfiltration",
                    desc,
                    code,
                    "Critical",
                    "Block script reading cloud credential files and revoke tokens",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        // 6. Sensitive file access targets in JS/TS (e.g. Bun.file("/etc/shadow"), readFileSync, etc.)
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
                Self::emit_cyber_alert(
                    "Lazarus Group / APT38",
                    "Sensitive System Credential Read",
                    target,
                    code,
                    &format!("{:?}", level),
                    "Prohibit unauthorized filesystem read access from script environment",
                );
                return AgentShieldVerdict::Block {
                    reason: reason.to_string(),
                    threat_level: level,
                };
            }
        }

        // 7. Destructive filesystem operations via Node / Bun APIs
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
        // 0. Prompt Injection scan
        let pi_verdict = Self::scan_prompt_injection(code);
        if let AgentShieldVerdict::Block { .. } = pi_verdict {
            return pi_verdict;
        }

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

        // 5. Reverse shell heuristics (socket connection coupled with dup2 / fileno / subprocess)
        if normalized.contains("socket.socket")
            && (normalized.contains("connect(") || normalized.contains(".connect ("))
            && (normalized.contains("dup2") || normalized.contains("fileno") || normalized.contains("subprocess") || normalized.contains("pty"))
        {
            let reason = "Reverse shell socket connection pattern detected in Python script".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (Python Socket)",
                "Host Network Interface & Process Table",
                code,
                "Critical",
                "Block raw TCP socket connection and terminate interactive Python shell process",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::Critical,
            };
        }

        // 5b. Crypto and Web3 wallet harvesting in Python
        let python_crypto_targets = [
            (".config/solana", "Solana CLI private keypair vault"),
            ("solana/id.json", "Solana id.json private keypair"),
            (".ethereum/keystore", "Ethereum account keystore directory"),
            ("ethereum/keystore", "Ethereum account keystore directory"),
            ("wallet.dat", "Bitcoin Core wallet database"),
            ("nkbihfbeogaeaoehlefnkodbefgpgknn", "MetaMask Chrome extension vault storage"),
            ("bfnaelmomeimhlpmgjnjophhpkkoljpa", "Phantom Solana extension vault storage"),
        ];
        for (target, desc) in python_crypto_targets {
            if normalized.contains(target) {
                let reason = format!("Targeted crypto wallet or Web3 key access '{target}' ({desc}) in Python script");
                Self::emit_cyber_alert(
                    "Lazarus Group (TraderTraitor)",
                    "Web3 & Crypto Wallet / Key Theft",
                    desc,
                    code,
                    "Critical",
                    "Block Python script access to crypto keys and rotate exposed wallets",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
        }

        // 5c. Cloud and Developer Secret Access in Python
        let python_cloud_targets = [
            (".aws/credentials", "AWS cloud access key credentials"),
            (".config/gcloud", "Google Cloud CLI configuration and tokens"),
            (".azure", "Azure CLI configuration and tokens"),
            (".kube/config", "Kubernetes administrative kubeconfig"),
        ];
        for (target, desc) in python_cloud_targets {
            if normalized.contains(target) {
                let reason = format!("Access to cloud credentials '{target}' ({desc}) detected in Python script");
                Self::emit_cyber_alert(
                    "Lazarus Group / APT38",
                    "Cloud & Developer Credential Exfiltration",
                    desc,
                    code,
                    "Critical",
                    "Block Python script reading cloud credential files and revoke tokens",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
            }
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
        // 0. Prompt Injection scan
        let pi_verdict = Self::scan_prompt_injection(code);
        if let AgentShieldVerdict::Block { .. } = pi_verdict {
            return pi_verdict;
        }

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

        // 2b. Reverse shells via Perl Socket APIs
        if (normalized.contains("io::socket") || normalized.contains("use socket") || normalized.contains("socket("))
            && (normalized.contains("connect") || normalized.contains("stdin") || normalized.contains("fork") || normalized.contains("peeraddr"))
        {
            let reason = "Perl socket reverse shell execution pattern detected".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Reverse Shell & C2 Callback (Perl)",
                "Perl Socket Runtime",
                code,
                "Critical",
                "Block raw socket creation and shell piping in Perl runtime",
            );
            return AgentShieldVerdict::Block {
                reason,
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

        // 4. Sensitive targets (System files, Cloud keys, Crypto wallets, Developer secrets)
        let sensitive_targets = [
            // System files
            ("/etc/shadow", "System password shadow hashes", ThreatLevel::Critical),
            ("etc/shadow", "System password shadow hashes", ThreatLevel::Critical),
            ("/etc/passwd", "System user accounts", ThreatLevel::Medium),
            ("etc/passwd", "System user accounts", ThreatLevel::Medium),
            ("/etc/sudoers", "Sudoers privilege elevation configuration", ThreatLevel::Critical),
            ("/proc/kcore", "Kernel memory core dump", ThreatLevel::Critical),
            ("/proc/mem", "Process memory dump", ThreatLevel::Critical),
            ("/dev/mem", "Raw physical system memory", ThreatLevel::Critical),
            ("config/sam", "Windows SAM credentials database", ThreatLevel::Critical),
            ("system32/config/sam", "Windows SAM credentials database", ThreatLevel::Critical),
            ("system32/config/system", "Windows SYSTEM registry hive", ThreatLevel::Critical),

            // Cloud credentials
            (".aws/credentials", "AWS cloud access key credentials", ThreatLevel::Critical),
            (".aws/config", "AWS cloud configuration profile", ThreatLevel::High),
            ("aws/credentials", "AWS cloud access key credentials", ThreatLevel::Critical),
            (".config/gcloud", "Google Cloud CLI configuration and tokens", ThreatLevel::Critical),
            ("application_default_credentials.json", "Google Cloud default application credentials", ThreatLevel::Critical),
            ("gcloud/credentials.db", "Google Cloud SQLite credentials database", ThreatLevel::Critical),
            (".azure/azureprofile.json", "Azure CLI profile configuration", ThreatLevel::Critical),
            (".azure/accesstokens.json", "Azure CLI access tokens", ThreatLevel::Critical),
            (".azure", "Azure CLI credentials directory", ThreatLevel::High),
            (".kube/config", "Kubernetes administrative kubeconfig", ThreatLevel::Critical),
            ("kube/config", "Kubernetes administrative kubeconfig", ThreatLevel::Critical),

            // Crypto / Web3 wallets (Lazarus / TraderTraitor / BlueNoroff targets)
            (".config/solana/id.json", "Solana CLI private keypair file", ThreatLevel::Critical),
            ("solana/id.json", "Solana id.json private keypair file", ThreatLevel::Critical),
            (".ethereum/keystore", "Ethereum keystore directory", ThreatLevel::Critical),
            ("ethereum/keystore", "Ethereum keystore directory", ThreatLevel::Critical),
            ("wallet.dat", "Bitcoin Core wallet database", ThreatLevel::Critical),
            (".bitcoin/wallet.dat", "Bitcoin Core wallet database", ThreatLevel::Critical),
            (".electrum/wallets", "Electrum Bitcoin wallet directory", ThreatLevel::Critical),
            ("exodus.wallet", "Exodus crypto wallet vault", ThreatLevel::Critical),
            ("nkbihfbeogaeaoehlefnkodbefgpgknn", "MetaMask Chrome extension vault storage", ThreatLevel::Critical),
            ("bfnaelmomeimhlpmgjnjophhpkkoljpa", "Phantom Solana extension vault storage", ThreatLevel::Critical),
            ("hnfanknocfeofbddgcijnmhnfnkdnaad", "Coinbase Wallet extension vault storage", ThreatLevel::Critical),

            // Developer / SSH / Git credentials
            (".ssh/id_rsa", "OpenSSH private RSA key", ThreatLevel::Critical),
            (".ssh/id_ed25519", "OpenSSH private Ed25519 key", ThreatLevel::Critical),
            (".ssh/id_ecdsa", "OpenSSH private ECDSA key", ThreatLevel::Critical),
            ("id_rsa", "OpenSSH private RSA key", ThreatLevel::Critical),
            ("id_ed25519", "OpenSSH private Ed25519 key", ThreatLevel::Critical),
            ("id_ecdsa", "OpenSSH private ECDSA key", ThreatLevel::Critical),
            (".git-credentials", "Git plaintext credentials file", ThreatLevel::Critical),
            ("git-credentials", "Git plaintext credentials file", ThreatLevel::Critical),
            (".netrc", "Netrc remote machine login credentials", ThreatLevel::Critical),
        ];

        for (target, desc, level) in sensitive_targets {
            if clean_path == target || clean_path.ends_with(target) || normalized.ends_with(target) || normalized.contains(target) {
                let reason = format!("Access to sensitive file or credential vault '{target}' ({desc}) is prohibited");
                Self::emit_cyber_alert(
                    "Lazarus Group / Nation-State APT",
                    "Sensitive Credential / Crypto Vault Access",
                    desc,
                    path_str,
                    &format!("{:?}", level),
                    "Prohibit unauthorized filesystem access and isolate process",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: level,
                };
            }
        }

        // Check environment secrets (.env) while excluding harmless templates (.env.example, etc.)
        let is_env_secret = (clean_path.ends_with("/.env") || clean_path == "/.env" || clean_path.ends_with(".env") || normalized.ends_with("/.env") || normalized == ".env" || normalized.contains("/.env"))
            && !clean_path.ends_with(".env.example")
            && !clean_path.ends_with(".env.sample")
            && !clean_path.ends_with(".env.template")
            && !clean_path.ends_with(".env.dist")
            && !normalized.ends_with(".env.example")
            && !normalized.ends_with(".env.sample")
            && !normalized.ends_with(".env.template");

        if is_env_secret {
            let reason = "Access to environment secrets file (.env) is prohibited".to_string();
            Self::emit_cyber_alert(
                "Lazarus Group / APT38",
                "Developer Secret Exfiltration (.env)",
                "Environment Secrets (.env)",
                path_str,
                "High",
                "Store sensitive credentials in secure key vaults, not plaintext .env files",
            );
            return AgentShieldVerdict::Block {
                reason,
                threat_level: ThreatLevel::High,
            };
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

            // 3. Reverse shell & C2 callback indicators
            let revshell_indicators = ["/dev/tcp", "/dev/udp", "nc -e", "ncat -e", "ncat --exec", "socat "];
            for ind in revshell_indicators {
                if item_trimmed.contains(ind) {
                    let reason = format!("Reverse shell or C2 relay indicator detected in package specifier: '{ind}'");
                    Self::emit_cyber_alert(
                        "Lazarus Group / APT38",
                        "Malicious Supply Chain (Reverse Shell Hook)",
                        "Package Manager Invocation",
                        item_trimmed,
                        "Critical",
                        "Quarantine malicious dependency and report package poisoning to upstream registry",
                    );
                    return AgentShieldVerdict::Block {
                        reason,
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            // 4. Download cradles & piped execution
            let download_cradles = [
                "powershell -enc", "powershell -encodedcommand", "certutil -urlcache", "bitsadmin /transfer",
                "curl | sh", "curl | bash", "wget | sh", "wget | bash", "curl |sh", "curl |bash",
            ];
            for cradle in download_cradles {
                if item_trimmed.contains(cradle) {
                    let reason = format!("Download cradle execution pattern detected in package specifier: '{cradle}'");
                    Self::emit_cyber_alert(
                        "TraderTraitor / Lazarus Group",
                        "Malicious Supply Chain (Download Cradle)",
                        "Package Manager Build Lifecycle",
                        item_trimmed,
                        "Critical",
                        "Block package installation script executing unauthorized download cradle",
                    );
                    return AgentShieldVerdict::Block {
                        reason,
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            // 5. Credential theft & sensitive files (INVARIANT: Active REGARDLESS of allow_native)
            let sensitive_targets = [
                ("/etc/shadow", "System shadow password hashes"),
                ("etc/shadow", "System shadow password hashes"),
                ("/etc/passwd", "System user accounts"),
                ("etc/passwd", "System user accounts"),
                ("/etc/sudoers", "Sudoers security configuration"),
                ("etc/sudoers", "Sudoers security configuration"),
                ("id_rsa", "OpenSSH private RSA key"),
                ("id_ed25519", "OpenSSH private Ed25519 key"),
                ("~/.ssh", "User SSH directory"),
                (".ssh/", "User SSH directory"),
                (".env", "Environment secrets file"),
                ("config/sam", "Windows SAM database"),
                (".aws/credentials", "AWS cloud access keys"),
                (".config/gcloud", "Google Cloud CLI tokens"),
                (".azure", "Azure CLI tokens"),
                (".kube/config", "Kubernetes admin kubeconfig"),
                (".config/solana", "Solana CLI private keys"),
                ("solana/id.json", "Solana keypair file"),
                (".ethereum/keystore", "Ethereum keystore"),
                ("wallet.dat", "Bitcoin Core wallet"),
                ("nkbihfbeogaeaoehlefnkodbefgpgknn", "MetaMask extension vault"),
            ];
            for (target, desc) in sensitive_targets {
                if item_trimmed.contains(target) {
                    let reason = format!("Access to sensitive credentials or system file '{target}' ({desc}) prohibited in package specifier");
                    Self::emit_cyber_alert(
                        "Lazarus Group (TraderTraitor)",
                        "Malicious Supply Chain (Credential / Crypto Harvest)",
                        desc,
                        item_trimmed,
                        "Critical",
                        "Reject poisoned package specifier targeting developer or cloud credentials",
                    );
                    return AgentShieldVerdict::Block {
                        reason,
                        threat_level: ThreatLevel::Critical,
                    };
                }
            }

            // 6. Suspicious obfuscated build script or postinstall hooks
            let item_lower = item_trimmed.to_lowercase();
            if (item_lower.contains("postinstall") || item_lower.contains("preinstall"))
                && (item_lower.contains("base64") || item_lower.contains("eval") || item_lower.contains("exec") || item_lower.contains("curl") || item_lower.contains("wget"))
            {
                let reason = format!("Obfuscated postinstall/preinstall execution hook detected: '{item_trimmed}'");
                Self::emit_cyber_alert(
                    "Lazarus Group (TraderTraitor)",
                    "Malicious Supply Chain (Poisoned Postinstall)",
                    "Build Lifecycle Script",
                    item_trimmed,
                    "Critical",
                    "Block package install with obfuscated postinstall lifecycle scripts",
                );
                return AgentShieldVerdict::Block {
                    reason,
                    threat_level: ThreatLevel::Critical,
                };
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

    /// Recursively scan files, codebases, repositories, and dependency manifests for nation-state APT indicators
    pub fn scan_file_content(path: &std::path::Path, content: &str) -> Vec<ShieldFinding> {
        if Self::is_unrestricted() {
            return Vec::new();
        }

        let mut findings = Vec::new();
        let path_str = path.to_string_lossy().replace('\\', "/");
        let path_lower = path_str.to_lowercase();
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or_default().to_lowercase();

        // 1. Path-level sensitive target check
        if !path_lower.ends_with(".example")
            && !path_lower.ends_with(".sample")
            && !path_lower.ends_with(".template")
            && !path_lower.ends_with(".dist")
        {
            let is_wallet_file = path_lower.contains("solana/id.json")
                || path_lower.contains(".config/solana")
                || path_lower.contains(".ethereum/keystore")
                || path_lower.ends_with("wallet.dat");
            if is_wallet_file {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: None,
                    threat_actor: "Lazarus Group (TraderTraitor / BlueNoroff)".to_string(),
                    attack_vector: "Web3 & Crypto Wallet Storage Exposure".to_string(),
                    rule_name: "EXPOSED_CRYPTO_WALLET_FILE".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: path_str.clone(),
                    remediation: "Never store or commit unencrypted private keys or wallet vaults in project repositories.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    &path_str,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
            }

            let is_cloud_key = path_lower.contains(".aws/credentials")
                || path_lower.contains("gcloud/credentials.db")
                || path_lower.contains(".azure/azureprofile.json")
                || path_lower.ends_with("/id_rsa")
                || path_lower.ends_with("/id_ed25519");
            if is_cloud_key {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: None,
                    threat_actor: "Lazarus Group / APT38".to_string(),
                    attack_vector: "Cloud & Developer Credential Vault Exposure".to_string(),
                    rule_name: "EXPOSED_CREDENTIAL_VAULT".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: path_str.clone(),
                    remediation: "Revoke exposed cloud credentials immediately and remove private keys from repository.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    &path_str,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
            }

            let is_env_secret = (path_lower.ends_with("/.env") || path_lower == ".env")
                && !path_lower.contains(".env.example")
                && !path_lower.contains(".env.sample");
            if is_env_secret {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: None,
                    threat_actor: "Lazarus Group / APT38".to_string(),
                    attack_vector: "Plaintext Environment Secrets File Exposure (.env)".to_string(),
                    rule_name: "EXPOSED_ENV_FILE".to_string(),
                    severity: ThreatLevel::High,
                    snippet: path_str.clone(),
                    remediation: "Add .env to .gitignore and inject environment variables dynamically at runtime.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    &path_str,
                    "High",
                    &finding.remediation,
                );
                findings.push(finding);
            }
        }

        // 2. Package manifest auditing (package.json, Cargo.toml, setup.py, build.rs)
        if file_name == "package.json" {
            // Check for malicious postinstall/preinstall hooks or lifecycle script execution
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(content) {
                if let Some(scripts) = json_val.get("scripts").and_then(|s| s.as_object()) {
                    for (hook, cmd_val) in scripts {
                        if let Some(cmd_str) = cmd_val.as_str() {
                            let cmd_lower = cmd_str.to_lowercase();
                            let is_lifecycle = hook == "postinstall" || hook == "preinstall" || hook == "install" || hook == "build";
                            
                            // Check for download cradles
                            let has_dl = cmd_lower.contains("curl")
                                || cmd_lower.contains("wget")
                                || cmd_lower.contains("fetch")
                                || cmd_lower.contains("powershell")
                                || cmd_lower.contains("certutil")
                                || cmd_lower.contains("bitsadmin");
                            if is_lifecycle && has_dl {
                                let finding = ShieldFinding {
                                    file_path: path_str.clone(),
                                    line_number: None,
                                    threat_actor: "Lazarus Group (TraderTraitor / APT38)".to_string(),
                                    attack_vector: format!("Malicious Supply Chain (Poisoned {hook} Download Cradle)"),
                                    rule_name: "SUPPLY_CHAIN_POISONED_HOOK".to_string(),
                                    severity: ThreatLevel::Critical,
                                    snippet: format!("\"{}\": \"{}\"", hook, cmd_str),
                                    remediation: "Remove remote download cradles from package.json lifecycle scripts.".to_string(),
                                };
                                Self::emit_cyber_alert(
                                    &finding.threat_actor,
                                    &finding.attack_vector,
                                    &path_str,
                                    cmd_str,
                                    "Critical",
                                    &finding.remediation,
                                );
                                findings.push(finding);
                            }

                            // Check for reverse shells in hooks
                            if cmd_lower.contains("/dev/tcp") || cmd_lower.contains("nc -e") || cmd_lower.contains("socat") {
                                let finding = ShieldFinding {
                                    file_path: path_str.clone(),
                                    line_number: None,
                                    threat_actor: "Lazarus Group / APT38".to_string(),
                                    attack_vector: format!("Malicious Supply Chain (Reverse Shell in {hook})"),
                                    rule_name: "SUPPLY_CHAIN_REVERSE_SHELL".to_string(),
                                    severity: ThreatLevel::Critical,
                                    snippet: format!("\"{}\": \"{}\"", hook, cmd_str),
                                    remediation: "Purge trojanized package and investigate upstream registry.".to_string(),
                                };
                                Self::emit_cyber_alert(
                                    &finding.threat_actor,
                                    &finding.attack_vector,
                                    &path_str,
                                    cmd_str,
                                    "Critical",
                                    &finding.remediation,
                                );
                                findings.push(finding);
                            }
                        }
                    }
                }
            }
        }

        // 3. Indirect Prompt Injection Scan across file
        let is_doc_or_code = file_name.ends_with(".md")
            || file_name.ends_with(".markdown")
            || file_name.ends_with(".txt")
            || file_name.ends_with(".html")
            || file_name.ends_with(".rs")
            || file_name.ends_with(".js")
            || file_name.ends_with(".ts")
            || file_name.ends_with(".py");

        if is_doc_or_code {
            let pi_verdict = Self::scan_prompt_injection(content);
            if let AgentShieldVerdict::Block { ref reason, threat_level } = pi_verdict {
                let _ = reason;
                // Find line number where the tag or phrase occurred
                let mut line_no = None;
                let mut snippet = String::new();
                for (idx, line) in content.lines().enumerate() {
                    let l_lower = line.to_lowercase();
                    if l_lower.contains("<!-- system:")
                        || l_lower.contains("<!-- agent:")
                        || l_lower.contains("<!-- instruction:")
                        || l_lower.contains("<!-- hidden:")
                        || l_lower.contains("[system instruction:")
                        || l_lower.contains("[system directive:")
                        || l_lower.contains("[system override:")
                        || l_lower.contains("<system_instruction>")
                        || l_lower.contains("ignore previous instructions")
                        || l_lower.contains("ignore all previous instructions")
                        || l_lower.contains("disregard previous instructions")
                        || l_lower.contains("disregard all previous instructions")
                        || l_lower.contains("bypass security gates")
                        || l_lower.contains("bypass safety gates")
                    {
                        line_no = Some(idx + 1);
                        snippet = line.trim().to_string();
                        break;
                    }
                }
                if snippet.is_empty() {
                    snippet = content.lines().next().unwrap_or("").to_string();
                }
                findings.push(ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: line_no,
                    threat_actor: "Adversarial Prompt Injector / Cognitive APT".to_string(),
                    attack_vector: "Indirect Prompt Injection (Tag / Directive Override)".to_string(),
                    rule_name: "INDIRECT_PROMPT_INJECTION".to_string(),
                    severity: threat_level,
                    snippet,
                    remediation: "Sanitize untrusted context, markdown comments, and documentation before LLM ingestion.".to_string(),
                });
            }
        }

        // 4. Line-by-line scanning for specific APT patterns
        for (idx, line) in content.lines().enumerate() {
            let line_number = idx + 1;
            let line_lower = line.to_lowercase();
            let trimmed = line.trim();
            if trimmed.is_empty() || (trimmed.starts_with("//") && !trimmed.contains("http") && !trimmed.contains("system:")) {
                continue;
            }

            // Reverse shells
            let is_revshell = line_lower.contains("/dev/tcp/")
                || line_lower.contains("/dev/udp/")
                || (line_lower.contains("nc ") && (line_lower.contains(" -e ") || line_lower.contains(" -e/")))
                || (line_lower.contains("ncat ") && (line_lower.contains(" -e ") || line_lower.contains(" --exec ")))
                || (line_lower.contains("socket") && line_lower.contains("connect(") && (line_lower.contains("dup2") || line_lower.contains("fileno")))
                || (line_lower.contains("io::socket") && line_lower.contains("stdin->fdopen"));
            if is_revshell {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: Some(line_number),
                    threat_actor: "Lazarus Group / APT38".to_string(),
                    attack_vector: "Reverse Shell & C2 Callback".to_string(),
                    rule_name: "REVERSE_SHELL_PATTERN".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: trimmed.chars().take(120).collect(),
                    remediation: "Terminate and remove outbound socket connection script from repository.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    trimmed,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
                continue;
            }

            // Obfuscated download cradles & LotL
            let is_download_cradle = (line_lower.contains("powershell") || line_lower.contains("pwsh"))
                && (line_lower.contains("-enc ") || line_lower.contains("-encodedcommand") || line_lower.contains("-ec "))
                || ((line_lower.contains("net.webclient") || line_lower.contains("webclient"))
                    && (line_lower.contains("downloadstring") || line_lower.contains("downloadfile")))
                || ((line_lower.contains("iex ") || line_lower.contains("iex(") || line_lower.contains("| iex") || line_lower.contains("|iex"))
                    && (line_lower.contains("http://") || line_lower.contains("https://") || line_lower.contains("downloadstring")))
                || (line_lower.contains("certutil") && (line_lower.contains("-urlcache") || line_lower.contains("-split")))
                || (line_lower.contains("bitsadmin") && line_lower.contains("/transfer"))
                || ((line_lower.contains("curl ") || line_lower.contains("wget "))
                    && (line_lower.contains("| sh") || line_lower.contains("|sh") || line_lower.contains("| bash") || line_lower.contains("|bash")));
            if is_download_cradle {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: Some(line_number),
                    threat_actor: "TraderTraitor / Lazarus Group".to_string(),
                    attack_vector: "Living-off-the-Land Download Cradle".to_string(),
                    rule_name: "DOWNLOAD_CRADLE_PATTERN".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: trimmed.chars().take(120).collect(),
                    remediation: "Block remote unverified download-and-execute cradles.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    trimmed,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
                continue;
            }

            // Crypto & Web3 wallet scrapers
            let is_crypto_theft = line_lower.contains("solana/id.json")
                || line_lower.contains(".config/solana")
                || line_lower.contains(".ethereum/keystore")
                || line_lower.contains("ethereum/keystore")
                || line_lower.contains("nkbihfbeogaeaoehlefnkodbefgpgknn")
                || line_lower.contains("bfnaelmomeimhlpmgjnjophhpkkoljpa")
                || line_lower.contains("wallet.dat")
                || line_lower.contains(".electrum/wallets")
                || line_lower.contains("exodus.wallet");
            let is_self_security_code = path_lower.contains("agentshield.rs") || path_lower.contains("test");
            if is_crypto_theft && !is_self_security_code {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: Some(line_number),
                    threat_actor: "Lazarus Group (TraderTraitor / BlueNoroff)".to_string(),
                    attack_vector: "Web3 & Crypto Wallet / Key Harvesting".to_string(),
                    rule_name: "CRYPTO_WALLET_SCRAPER".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: trimmed.chars().take(120).collect(),
                    remediation: "Isolate host crypto wallets, rotate exposed keys, and inspect codebase for trojans.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    trimmed,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
                continue;
            }

            // Cloud credential exfiltration
            let is_cloud_harvest = (line_lower.contains(".aws/credentials") || line_lower.contains(".config/gcloud") || line_lower.contains(".azure") || line_lower.contains(".kube/config") || line_lower.contains("id_rsa") || line_lower.contains("id_ed25519"))
                && (line_lower.contains("cat ") || line_lower.contains("curl ") || line_lower.contains("wget ") || line_lower.contains("type ") || line_lower.contains("readfile") || line_lower.contains("open("));
            if is_cloud_harvest && !is_self_security_code {
                let finding = ShieldFinding {
                    file_path: path_str.clone(),
                    line_number: Some(line_number),
                    threat_actor: "Lazarus Group / APT38".to_string(),
                    attack_vector: "Cloud & Developer Credential Exfiltration".to_string(),
                    rule_name: "CREDENTIAL_EXFILTRATION".to_string(),
                    severity: ThreatLevel::Critical,
                    snippet: trimmed.chars().take(120).collect(),
                    remediation: "Prohibit reading and uploading of private credentials.".to_string(),
                };
                Self::emit_cyber_alert(
                    &finding.threat_actor,
                    &finding.attack_vector,
                    &path_str,
                    trimmed,
                    "Critical",
                    &finding.remediation,
                );
                findings.push(finding);
                continue;
            }
        }

        findings
    }

    /// Recursively scan an entire directory or single file for nation-state APT indicators
    pub fn scan_directory(dir_path: &std::path::Path) -> ShieldScanReport {
        let mut report = ShieldScanReport {
            target_path: dir_path.to_string_lossy().to_string(),
            files_scanned: 0,
            findings: Vec::new(),
            passed: true,
        };

        if !dir_path.exists() {
            return report;
        }

        if dir_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(dir_path) {
                report.files_scanned = 1;
                report.findings = Self::scan_file_content(dir_path, &content);
            }
            report.passed = report.findings.is_empty();
            return report;
        }

        fn walk_dir(dir: &std::path::Path, report: &mut ShieldScanReport) {
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => return,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or_default();
                
                if path.is_dir() {
                    if file_name == ".git"
                        || file_name == "node_modules"
                        || file_name == "target"
                        || file_name == ".svn"
                        || file_name == ".hg"
                    {
                        continue;
                    }
                    walk_dir(&path, report);
                } else if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        if meta.len() > 10 * 1024 * 1024 {
                            continue;
                        }
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        report.files_scanned += 1;
                        let file_findings = AgentShieldScanner::scan_file_content(&path, &content);
                        report.findings.extend(file_findings);
                    }
                }
            }
        }

        walk_dir(dir_path, &mut report);
        report.passed = report.findings.is_empty();
        report
    }

    /// Display known nation-state threat actor profiles (Lazarus Group, Kimsuky, Cognitive APT)
    pub fn threat_actor_profiles() -> Vec<ThreatActorProfile> {
        vec![
            ThreatActorProfile {
                name: "Lazarus Group (APT38 / TraderTraitor / BlueNoroff)".to_string(),
                aliases: vec![
                    "APT38".to_string(),
                    "TraderTraitor".to_string(),
                    "BlueNoroff".to_string(),
                    "Stardust Chollima".to_string(),
                    "HIDDEN COBRA".to_string(),
                ],
                attribution: "Democratic People's Republic of Korea (DPRK) Reconnaissance General Bureau (RGB)".to_string(),
                primary_targets: vec![
                    "Crypto Wallets (Solana, Ethereum, Bitcoin)".to_string(),
                    "Web3 Browser Extensions (MetaMask, Phantom, Coinbase)".to_string(),
                    "Cloud Infrastructure (.aws, GCP, Azure, Kubeconfig)".to_string(),
                    "Developer Workstations & CI/CD Pipelines".to_string(),
                ],
                ttp_indicators: vec![
                    "Poisoned npm/cargo postinstall lifecycle hooks".to_string(),
                    "Obfuscated PowerShell download cradles & IEX execution".to_string(),
                    "Reverse shells via /dev/tcp, netcat -e, and Python/Perl sockets".to_string(),
                    "MetaMask & Phantom extension vault theft".to_string(),
                    "Indirect prompt injection embedded in code comments and READMEs".to_string(),
                ],
            },
            ThreatActorProfile {
                name: "Kimsuky (APT43 / Velvet Chollima)".to_string(),
                aliases: vec![
                    "APT43".to_string(),
                    "Velvet Chollima".to_string(),
                    "Black Banshee".to_string(),
                    "Emerald Sleet".to_string(),
                ],
                attribution: "DPRK Reconnaissance General Bureau (RGB)".to_string(),
                primary_targets: vec![
                    "SSH Credentials & Private Keys (id_rsa, id_ed25519)".to_string(),
                    "Developer Identity (.git-credentials, .netrc)".to_string(),
                    "Kubernetes Administrative Configs (kubeconfig)".to_string(),
                    "Cloud Service Account Access Tokens".to_string(),
                ],
                ttp_indicators: vec![
                    "Certutil and bitsadmin LOLBins for file staging".to_string(),
                    "Encoded PowerShell execution (-enc)".to_string(),
                    "Trojanized developer utility repos and spearphishing packages".to_string(),
                    "Automated SSH key theft and network reconnaissance".to_string(),
                ],
            },
            ThreatActorProfile {
                name: "Adversarial Prompt Injector (Cognitive APT)".to_string(),
                aliases: vec![
                    "PromptWare".to_string(),
                    "Cognitive Exploit Operator".to_string(),
                    "Indirect Prompt Hijacker".to_string(),
                ],
                attribution: "Adversarial Nation-State / Cognitive Automation Exploiters".to_string(),
                primary_targets: vec![
                    "AI Autonomous Agents & LLM Runtimes".to_string(),
                    "Tool Execution Context & Environment Variables".to_string(),
                    "Local Worktree & File System Integrity".to_string(),
                ],
                ttp_indicators: vec![
                    "Delimited system tags (<!-- system:, <system_instruction>)".to_string(),
                    "Directive override phrases ('ignore previous instructions')".to_string(),
                    "Markdown exfiltration links and hidden instructions in PR reviews".to_string(),
                ],
            },
        ]
    }

    /// List protected system, cloud, crypto, and developer assets
    pub fn protected_assets() -> Vec<&'static str> {
        vec![
            "Web3 & Crypto Wallets (Solana CLI id.json, Ethereum Keystores, Bitcoin wallet.dat, Exodus, Electrum)",
            "Browser Extension Crypto Vaults (MetaMask, Phantom, Coinbase Wallet)",
            "Cloud Service Credentials (.aws/credentials, Google Cloud gcloud/tokens, Azure CLI, Kubernetes admin.conf)",
            "Developer Identity & Keys (OpenSSH id_rsa/id_ed25519, .git-credentials, .netrc, .env)",
            "Host Integrity & Kernel Protection (/etc/shadow, /etc/sudoers, Windows SAM, raw storage devices, fork bomb prevention)",
            "AI Agent Runtime Guardrails (System prompt primacy, zero ambient authority, prompt injection filtering)",
        ]
    }

    /// Incident telemetry and active cyber defense summary
    pub fn incident_telemetry() -> CyberDefenseTelemetry {
        let hist = crate::notify::hub().history();
        let security_events: Vec<_> = hist
            .iter()
            .filter(|e| e.category == crate::notify::NotificationCategory::SecurityAlert
                || e.category == crate::notify::NotificationCategory::CyberDefenseAlert)
            .collect();

        let critical = security_events
            .iter()
            .filter(|e| e.severity == crate::notify::NotificationSeverity::Critical || e.severity == crate::notify::NotificationSeverity::Emergency)
            .count();
        let sec_alert = security_events
            .iter()
            .filter(|e| e.severity == crate::notify::NotificationSeverity::SecurityAlert)
            .count();
        let warning = security_events
            .iter()
            .filter(|e| e.severity == crate::notify::NotificationSeverity::Warning)
            .count();

        let status = if critical > 0 || sec_alert > 0 {
            "THREAT_INTERCEPTED".to_string()
        } else {
            "ENFORCING / RESILIENT".to_string()
        };

        CyberDefenseTelemetry {
            status,
            total_incidents: security_events.len(),
            critical_incidents: critical,
            security_alerts: sec_alert,
            warning_incidents: warning,
            protected_asset_categories: Self::protected_assets().into_iter().map(String::from).collect(),
            monitored_threat_actors: Self::threat_actor_profiles().into_iter().map(|p| p.name).collect(),
        }
    }
}

/// Structured finding for an identified cyber threat or policy violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShieldFinding {
    pub file_path: String,
    pub line_number: Option<usize>,
    pub threat_actor: String,
    pub attack_vector: String,
    pub rule_name: String,
    pub severity: ThreatLevel,
    pub snippet: String,
    pub remediation: String,
}

/// Structured summary report of a codebase, manifest, or directory scan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShieldScanReport {
    pub target_path: String,
    pub files_scanned: usize,
    pub findings: Vec<ShieldFinding>,
    pub passed: bool,
}

/// Cyber threat actor profile information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreatActorProfile {
    pub name: String,
    pub aliases: Vec<String>,
    pub attribution: String,
    pub primary_targets: Vec<String>,
    pub ttp_indicators: Vec<String>,
}

/// Summary of incident telemetry and defense metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CyberDefenseTelemetry {
    pub status: String,
    pub total_incidents: usize,
    pub critical_incidents: usize,
    pub security_alerts: usize,
    pub warning_incidents: usize,
    pub protected_asset_categories: Vec<String>,
    pub monitored_threat_actors: Vec<String>,
}
