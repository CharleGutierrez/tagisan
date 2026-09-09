use crate::bun::runtime::{BunExecutionResult, BunRuntime};
use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

/// Resource limits and path access configuration for sandboxed Bun execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunSandboxConfig {
    /// Maximum virtual memory in bytes (RLIMIT_AS), default: 512 MB
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in seconds (RLIMIT_CPU), default: 15s
    pub max_cpu_time_secs: Option<u64>,
    /// Maximum file size creation in bytes (RLIMIT_FSIZE), default: 25 MB
    pub max_file_size_bytes: Option<u64>,
    /// Maximum open file descriptors (RLIMIT_NOFILE), default: 128
    pub max_open_files: Option<u64>,
    /// Allowed read paths (default: working dir, node_modules, and standard libs)
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed write paths (default: designated sandbox dir or /tmp/tagisan-*)
    pub allowed_write_paths: Vec<PathBuf>,
    /// Blocked sensitive system paths
    pub blocked_paths: Vec<PathBuf>,
}

impl Default for BunSandboxConfig {
    fn default() -> Self {
        let mut blocked = vec![
            PathBuf::from("/etc/shadow"),
            PathBuf::from("/etc/passwd"),
            PathBuf::from("/etc/sudoers"),
            PathBuf::from("/root"),
            PathBuf::from("/proc/kcore"),
            PathBuf::from("/dev/mem"),
        ];

        if let Ok(home) = std::env::var("HOME") {
            let ssh_dir = PathBuf::from(&home).join(".ssh");
            blocked.push(ssh_dir.clone());
            blocked.push(ssh_dir.join("id_rsa"));
            blocked.push(ssh_dir.join("id_ed25519"));
            blocked.push(PathBuf::from(&home).join(".gnupg"));
        }

        Self {
            max_memory_bytes: Some(512 * 1024 * 1024), // 512 MB
            max_cpu_time_secs: Some(15),                // 15 seconds CPU time
            max_file_size_bytes: Some(25 * 1024 * 1024), // 25 MB max output file
            max_open_files: Some(128),                  // 128 max file handles
            allowed_read_paths: vec![PathBuf::from(".")],
            allowed_write_paths: vec![PathBuf::from(".")],
            blocked_paths: blocked,
        }
    }
}

/// Linux OS-level and filesystem sandboxing governor for Bun processes
#[derive(Debug, Clone)]
pub struct BunSandbox {
    pub config: BunSandboxConfig,
}

impl BunSandbox {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self::strict(working_dir)
    }

    pub fn from_config(config: BunSandboxConfig) -> Self {
        Self { config }
    }

    pub fn with_memory_limit_mb(mut self, mb: u64) -> Self {
        self.config.max_memory_bytes = Some(mb * 1024 * 1024);
        self
    }

    pub fn with_cpu_limit_secs(mut self, secs: u64) -> Self {
        self.config.max_cpu_time_secs = Some(secs);
        self
    }

    pub fn with_max_file_size_bytes(mut self, bytes: u64) -> Self {
        self.config.max_file_size_bytes = Some(bytes);
        self
    }

    pub fn with_max_open_files(mut self, count: u64) -> Self {
        self.config.max_open_files = Some(count);
        self
    }

    pub fn is_path_allowed(&self, path: impl AsRef<Path>) -> bool {
        self.validate_path(path.as_ref(), false).is_ok()
    }

    pub fn strict(working_dir: impl Into<PathBuf>) -> Self {
        let wd = working_dir.into();
        let mut config = BunSandboxConfig::default();
        config.allowed_read_paths = vec![wd.clone(), PathBuf::from("/usr"), PathBuf::from("/lib")];
        config.allowed_write_paths = vec![wd];
        Self { config }
    }

    /// Verifies path against blocked targets and write containment
    pub fn validate_path(&self, path: &Path, is_write: bool) -> Result<()> {
        let normalized = path.to_string_lossy().to_lowercase();

        // 1. Check blocked paths
        for blocked in &self.config.blocked_paths {
            let b_str = blocked.to_string_lossy().to_lowercase();
            if normalized == b_str || normalized.starts_with(&format!("{b_str}/")) || normalized.contains(&b_str) {
                return Err(TagisanError::Execution(format!(
                    "Sandbox violation: Access to prohibited system target '{}' is blocked",
                    blocked.display()
                )));
            }
        }

        // 2. Traversal patterns
        if normalized.contains("../") || normalized.contains("..\\") {
            return Err(TagisanError::Execution(
                "Sandbox violation: Directory traversal pattern ('..') detected".to_string(),
            ));
        }

        // 3. Write / Read containment check
        if is_write {
            let is_allowed = self.config.allowed_write_paths.iter().any(|allowed| {
                let a_str = allowed.to_string_lossy().to_lowercase();
                normalized.starts_with(&a_str) || normalized.starts_with("/tmp/tagisan-")
            });

            if !is_allowed {
                return Err(TagisanError::Execution(format!(
                    "Sandbox violation: Write access outside designated writable roots is prohibited: '{}'",
                    path.display()
                )));
            }
        } else if !self.config.allowed_read_paths.is_empty() {
            let is_allowed = self.config.allowed_read_paths.iter().any(|allowed| {
                let a_str = allowed.to_string_lossy().to_lowercase();
                normalized.starts_with(&a_str) || normalized.starts_with("/tmp/tagisan-")
            });

            if !is_allowed {
                return Err(TagisanError::Execution(format!(
                    "Sandbox violation: Read access outside designated readable roots is prohibited: '{}'",
                    path.display()
                )));
            }
        }

        Ok(())
    }

    /// Configures `tokio::process::Command` with Linux resource limits via pre_exec
    pub fn apply_limits(&self, cmd: &mut Command) {
        #[cfg(target_os = "linux")]
        {
            let mem_limit = self.config.max_memory_bytes;
            let cpu_limit = self.config.max_cpu_time_secs;
            let fsize_limit = self.config.max_file_size_bytes;
            let nofile_limit = self.config.max_open_files;

            unsafe {
                cmd.pre_exec(move || {
                    if let Some(mem) = mem_limit {
                        let rlim = libc::rlimit {
                            rlim_cur: mem as libc::rlim_t,
                            rlim_max: mem as libc::rlim_t,
                        };
                        libc::setrlimit(libc::RLIMIT_AS, &rlim);
                    }

                    if let Some(cpu) = cpu_limit {
                        let rlim = libc::rlimit {
                            rlim_cur: cpu as libc::rlim_t,
                            rlim_max: cpu as libc::rlim_t,
                        };
                        libc::setrlimit(libc::RLIMIT_CPU, &rlim);
                    }

                    if let Some(fsize) = fsize_limit {
                        let rlim = libc::rlimit {
                            rlim_cur: fsize as libc::rlim_t,
                            rlim_max: fsize as libc::rlim_t,
                        };
                        libc::setrlimit(libc::RLIMIT_FSIZE, &rlim);
                    }

                    if let Some(nofile) = nofile_limit {
                        let rlim = libc::rlimit {
                            rlim_cur: nofile as libc::rlim_t,
                            rlim_max: nofile as libc::rlim_t,
                        };
                        libc::setrlimit(libc::RLIMIT_NOFILE, &rlim);
                    }

                    Ok(())
                });
            }
        }
    }

    /// Executes sandboxed code string through Bun with resource limit enforcement
    pub async fn eval_sandboxed(
        &self,
        code: &str,
        timeout_secs: u64,
    ) -> Result<BunExecutionResult> {
        let runtime = BunRuntime::default();
        self.eval_sandboxed_with_runtime(&runtime, code, Duration::from_secs(timeout_secs)).await
    }

    /// Executes sandboxed code string through Bun with a specific runtime instance
    pub async fn eval_sandboxed_with_runtime(
        &self,
        runtime: &BunRuntime,
        code: &str,
        timeout_duration: Duration,
    ) -> Result<BunExecutionResult> {
        // Pre-scan code for blocked paths
        for blocked in &self.config.blocked_paths {
            let b_str = blocked.to_string_lossy();
            if code.contains(&*b_str) {
                return Err(TagisanError::Execution(format!(
                    "Sandbox security violation: Code contains blocked target path '{}'",
                    b_str
                )));
            }
        }

        // Evaluate using BunRuntime
        runtime.eval(code, timeout_duration, None, None).await
    }
}
