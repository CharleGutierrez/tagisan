use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use crate::python::PythonRuntime;

/// Execution environment detected for sandboxed harness execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    /// Direct Python interpreter
    Direct(PathBuf),
    /// Isolated Virtual Environment (e.g. .venv, VIRTUAL_ENV)
    VirtualEnv {
        venv_root: PathBuf,
        python_bin: PathBuf,
    },
    /// Fast UV Ephemeral Runner (`uv run -- python`)
    Uv {
        project_root: PathBuf,
        manifest_file: PathBuf,
        uv_bin: PathBuf,
    },
}

impl ExecutionEnvironment {
    pub fn description(&self) -> String {
        match self {
            Self::Direct(p) => format!("Direct Python ({})", p.display()),
            Self::VirtualEnv { venv_root, python_bin } => {
                format!("VirtualEnv ({}, bin: {})", venv_root.display(), python_bin.display())
            }
            Self::Uv { project_root, manifest_file, .. } => {
                format!("UV Runner (project: {}, manifest: {})", project_root.display(), manifest_file.file_name().and_then(|n| n.to_str()).unwrap_or("manifest"))
            }
        }
    }
}

/// Execution outcome of a synthesized harness run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub json_data: Option<serde_json::Value>,
    pub duration_ms: u64,
    pub success: bool,
    pub environment: Option<ExecutionEnvironment>,
}

impl HarnessExecutionResult {
    /// Formats human-readable combined output
    pub fn display_summary(&self) -> String {
        let mut out = format!(
            "Exit Code: {} (Duration: {}ms)\n",
            self.exit_code, self.duration_ms
        );
        if let Some(ref env) = self.environment {
            out.push_str(&format!("Environment: {}\n", env.description()));
        }
        if let Some(ref j) = self.json_data {
            out.push_str("--- JSON Output ---\n");
            out.push_str(&serde_json::to_string_pretty(j).unwrap_or_default());
            out.push('\n');
        } else if !self.stdout.is_empty() {
            out.push_str("--- STDOUT ---\n");
            out.push_str(&self.stdout);
            if !self.stdout.ends_with('\n') {
                out.push('\n');
            }
        }
        if !self.stderr.is_empty() {
            out.push_str("--- STDERR ---\n");
            out.push_str(&self.stderr);
            if !self.stderr.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }
}

/// Secure harness runner that audits invocations against AgentShield before execution,
/// autodetects virtual environments (uv, venv, virtualenv), and applies Linux Landlock sandboxing.
pub struct HarnessRunner {
    python_bin: PathBuf,
    custom_python: bool,
    default_timeout: Duration,
    enforce_shield: bool,
    enforce_landlock: bool,
}

impl HarnessRunner {
    /// Maximum allowed output capture (1 MB) to prevent memory exhaustion
    pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

    pub fn new() -> Result<Self> {
        let python_bin = PythonRuntime::find_python().unwrap_or_else(|| PathBuf::from("python3"));
        let unrestricted = AgentShieldScanner::is_unrestricted();
        Ok(Self {
            python_bin,
            custom_python: false,
            default_timeout: Duration::from_secs(30),
            enforce_shield: !unrestricted,
            enforce_landlock: !unrestricted,
        })
    }

    pub fn with_python(mut self, bin: impl Into<PathBuf>) -> Self {
        self.python_bin = bin.into();
        self.custom_python = true;
        self
    }

    pub fn with_timeout(mut self, timeout_dur: Duration) -> Self {
        self.default_timeout = timeout_dur;
        self
    }

    pub fn with_shield(mut self, enabled: bool) -> Self {
        self.enforce_shield = enabled;
        self
    }

    pub fn with_landlock(mut self, enabled: bool) -> Self {
        self.enforce_landlock = enabled;
        self
    }

    /// Autodetects the appropriate execution environment (uv, virtualenv, or direct python)
    pub fn detect_environment(&self, script: &Path, cwd: Option<&Path>) -> ExecutionEnvironment {
        if self.custom_python {
            return ExecutionEnvironment::Direct(self.python_bin.clone());
        }

        let start_dir = cwd
            .map(|p| p.to_path_buf())
            .or_else(|| script.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // 1. Check if active VIRTUAL_ENV environment variable is set
        if let Ok(venv_str) = std::env::var("VIRTUAL_ENV") {
            let venv_path = PathBuf::from(venv_str);
            let py_candidates = [
                venv_path.join("bin").join("python3"),
                venv_path.join("bin").join("python"),
                venv_path.join("Scripts").join("python.exe"),
            ];
            for py in &py_candidates {
                if py.is_file() {
                    return ExecutionEnvironment::VirtualEnv {
                        venv_root: venv_path.clone(),
                        python_bin: py.clone(),
                    };
                }
            }
        }

        // 2. Search start_dir and parent directories for .venv, venv, or manifests
        let mut curr = Some(start_dir.as_path());
        let mut checked_depth = 0;
        while let Some(dir) = curr {
            if checked_depth > 6 {
                break;
            }

            // Check for .venv / venv
            for venv_name in &[".venv", "venv", ".env"] {
                let venv_dir = dir.join(venv_name);
                let py_candidates = [
                    venv_dir.join("bin").join("python3"),
                    venv_dir.join("bin").join("python"),
                    venv_dir.join("Scripts").join("python.exe"),
                ];
                for py in &py_candidates {
                    if py.is_file() {
                        return ExecutionEnvironment::VirtualEnv {
                            venv_root: venv_dir,
                            python_bin: py.clone(),
                        };
                    }
                }
            }

            // Check for uv + manifests (pyproject.toml, requirements.txt)
            for manifest_name in &["pyproject.toml", "requirements.txt", "Pipfile", "setup.py"] {
                let manifest = dir.join(manifest_name);
                if manifest.is_file() {
                    if let Some(uv_bin) = find_uv_binary() {
                        return ExecutionEnvironment::Uv {
                            project_root: dir.to_path_buf(),
                            manifest_file: manifest,
                            uv_bin,
                        };
                    }
                }
            }

            curr = dir.parent();
            checked_depth += 1;
        }

        // 3. Fallback to standard python binary
        ExecutionEnvironment::Direct(self.python_bin.clone())
    }

    /// Safely executes a CLI harness script with argument validation through AgentShield
    /// and ephemeral dependency sandboxing (uv / venv).
    pub async fn run(
        &self,
        script_path: impl AsRef<Path>,
        args: &[String],
        cwd: Option<&Path>,
    ) -> Result<HarnessExecutionResult> {
        let script = script_path.as_ref();
        let script_str = script.to_string_lossy();

        // 1. AgentShield File Path Security Audit
        if self.enforce_shield {
            let path_verdict = AgentShieldScanner::scan_file_path(&script_str);
            if let AgentShieldVerdict::Block { reason, threat_level } = path_verdict {
                return Err(TagisanError::Execution(format!(
                    "AgentShield blocked harness script path (Threat: {:?}): {}",
                    threat_level, reason
                )));
            }

            // 2. AgentShield Command Arguments Audit
            let full_command = format!("python3 {} {}", script_str, args.join(" "));
            let cmd_verdict = AgentShieldScanner::scan_command(&full_command);
            if let AgentShieldVerdict::Block { reason, threat_level } = cmd_verdict {
                return Err(TagisanError::Execution(format!(
                    "AgentShield blocked harness command (Threat: {:?}): {}",
                    threat_level, reason
                )));
            }
        }

        // 3. Detect Sandboxed Execution Environment (uv, venv, direct)
        let canonical_script = std::fs::canonicalize(script).unwrap_or_else(|_| script.to_path_buf());
        let env = self.detect_environment(script, cwd);

        let mut cmd = match &env {
            ExecutionEnvironment::Uv { uv_bin, project_root, .. } => {
                let mut c = Command::new(uv_bin);
                c.arg("run");
                c.arg("--project");
                c.arg(project_root);
                c.arg("python");
                c.arg(&canonical_script);
                for arg in args {
                    c.arg(arg);
                }
                c
            }
            ExecutionEnvironment::VirtualEnv { venv_root, python_bin } => {
                let mut c = Command::new(python_bin);
                c.env("VIRTUAL_ENV", venv_root);
                let bin_dir = venv_root.join("bin");
                if let Ok(path) = std::env::var("PATH") {
                    c.env("PATH", format!("{}:{}", bin_dir.display(), path));
                }
                c.arg(&canonical_script);
                for arg in args {
                    c.arg(arg);
                }
                c
            }
            ExecutionEnvironment::Direct(bin) => {
                let mut c = Command::new(bin);
                c.arg(&canonical_script);
                for arg in args {
                    c.arg(arg);
                }
                c
            }
        };

        let target_cwd = if let Some(dir) = cwd {
            dir.to_path_buf()
        } else if let Some(parent) = script.parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from(".")
        };
        cmd.current_dir(&target_cwd);

        // 4. Linux Landlock LSM Sandboxing
        #[cfg(target_os = "linux")]
        if self.enforce_landlock {
            let script_dir = canonical_script.parent().unwrap_or(Path::new(".")).to_path_buf();
            let working_dir = target_cwd.clone();
            let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            unsafe {
                cmd.pre_exec(move || {
                    let _ = apply_linux_landlock(&script_dir, &working_dir, &workspace_root);
                    Ok(())
                });
            }
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let start_time = Instant::now();

        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to spawn harness process '{}' (Env: {}): {}",
                script.display(),
                env.description(),
                e
            ))
        })?;

        let mut stdout_pipe = child.stdout.take().ok_or_else(|| {
            TagisanError::Execution("Failed to open stdout pipe for harness".to_string())
        })?;

        let mut stderr_pipe = child.stderr.take().ok_or_else(|| {
            TagisanError::Execution("Failed to open stderr pipe for harness".to_string())
        })?;

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stdout_pipe.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() >= Self::MAX_OUTPUT_BYTES {
                    break;
                }
            }
            buf
        });

        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stderr_pipe.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() >= Self::MAX_OUTPUT_BYTES {
                    break;
                }
            }
            buf
        });

        let status = match timeout(self.default_timeout, child.wait()).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                return Err(TagisanError::Execution(format!(
                    "Harness process execution failed: {}",
                    e
                )));
            }
            Err(_) => {
                let _ = child.kill().await;
                return Err(TagisanError::Execution(format!(
                    "Harness process execution timed out after {:.1}s",
                    self.default_timeout.as_secs_f32()
                )));
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let raw_stdout = stdout_task.await.unwrap_or_default();
        let raw_stderr = stderr_task.await.unwrap_or_default();

        let stdout = String::from_utf8_lossy(&raw_stdout).to_string();
        let stderr = String::from_utf8_lossy(&raw_stderr).to_string();
        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        // 5. JSON parse attempt
        let json_data = if args.iter().any(|a| a == "--json") || stdout.trim().starts_with('{') {
            serde_json::from_str::<serde_json::Value>(stdout.trim()).ok()
        } else {
            None
        };

        Ok(HarnessExecutionResult {
            exit_code,
            stdout,
            stderr,
            json_data,
            duration_ms,
            success,
            environment: Some(env),
        })
    }

    /// Safely executes an automated validation test script
    pub async fn run_tests(
        &self,
        test_script_path: impl AsRef<Path>,
        cwd: Option<&Path>,
    ) -> Result<HarnessExecutionResult> {
        let args = vec![];
        self.run(test_script_path, &args, cwd).await
    }
}

/// Locates uv executable in PATH or standard install locations
fn find_uv_binary() -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let cand = dir.join("uv");
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let cargo_uv = PathBuf::from(home).join(".cargo").join("bin").join("uv");
        if cargo_uv.is_file() {
            return Some(cargo_uv);
        }
    }
    for p in &["/root/.cargo/bin/uv", "/usr/local/bin/uv", "/usr/bin/uv"] {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Linux Landlock LSM micro-sandboxing helper. Restricts process filesystem access to
/// required system libraries, temporary directory, and current workspace.
#[cfg(target_os = "linux")]
unsafe fn apply_linux_landlock(
    script_dir: &Path,
    working_dir: &Path,
    workspace_root: &Path,
) -> std::result::Result<(), i32> {
    const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
    const SYS_LANDLOCK_ADD_RULE: libc::c_long = 445;
    const SYS_LANDLOCK_RESTRICT_SELF: libc::c_long = 446;
    const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;

    const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
    const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
    const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
    const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
    const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
    const LANDLOCK_ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
    const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
    const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
    const LANDLOCK_ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
    const LANDLOCK_ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
    const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
    const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;
    const LANDLOCK_ACCESS_FS_REFER: u64 = 1 << 13;
    const LANDLOCK_ACCESS_FS_TRUNCATE: u64 = 1 << 14;

    #[repr(C)]
    struct LandlockRulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C)]
    struct LandlockPathBeneathAttr {
        allowed_access: u64,
        parent_fd: i32,
    }

    let read_flags = LANDLOCK_ACCESS_FS_READ_FILE
        | LANDLOCK_ACCESS_FS_READ_DIR
        | LANDLOCK_ACCESS_FS_EXECUTE;
    let write_flags = read_flags
        | LANDLOCK_ACCESS_FS_WRITE_FILE
        | LANDLOCK_ACCESS_FS_REMOVE_DIR
        | LANDLOCK_ACCESS_FS_REMOVE_FILE
        | LANDLOCK_ACCESS_FS_MAKE_DIR
        | LANDLOCK_ACCESS_FS_MAKE_REG
        | LANDLOCK_ACCESS_FS_TRUNCATE;

    let attr = LandlockRulesetAttr {
        handled_access_fs: write_flags,
    };

    let fd = libc::syscall(
        SYS_LANDLOCK_CREATE_RULESET,
        &attr as *const _ as *const libc::c_void,
        std::mem::size_of::<LandlockRulesetAttr>(),
        0u32,
    );
    if fd < 0 {
        return Err(*libc::__errno_location());
    }
    let ruleset_fd = fd as i32;

    let read_only_paths = ["/usr", "/lib", "/lib64", "/etc", "/bin", "/opt", "/dev"];
    for p in &read_only_paths {
        if let Ok(c_path) = std::ffi::CString::new(*p) {
            let pfd = libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if pfd >= 0 {
                let pb = LandlockPathBeneathAttr {
                    allowed_access: read_flags,
                    parent_fd: pfd,
                };
                libc::syscall(
                    SYS_LANDLOCK_ADD_RULE,
                    ruleset_fd,
                    LANDLOCK_RULE_PATH_BENEATH,
                    &pb as *const _ as *const libc::c_void,
                    0u32,
                );
                libc::close(pfd);
            }
        }
    }

    let mut write_paths = vec![
        working_dir.as_os_str().to_string_lossy().to_string(),
        script_dir.as_os_str().to_string_lossy().to_string(),
        workspace_root.as_os_str().to_string_lossy().to_string(),
        "/tmp".to_string(),
    ];
    if let Ok(curr) = std::env::current_dir() {
        write_paths.push(curr.as_os_str().to_string_lossy().to_string());
    }

    for p in &write_paths {
        if let Ok(c_path) = std::ffi::CString::new(p.as_str()) {
            let pfd = libc::open(c_path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if pfd >= 0 {
                let pb = LandlockPathBeneathAttr {
                    allowed_access: write_flags,
                    parent_fd: pfd,
                };
                libc::syscall(
                    SYS_LANDLOCK_ADD_RULE,
                    ruleset_fd,
                    LANDLOCK_RULE_PATH_BENEATH,
                    &pb as *const _ as *const libc::c_void,
                    0u32,
                );
                libc::close(pfd);
            }
        }
    }

    libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
    let res = libc::syscall(SYS_LANDLOCK_RESTRICT_SELF, ruleset_fd, 0u32);
    libc::close(ruleset_fd);

    if res < 0 {
        return Err(*libc::__errno_location());
    }
    Ok(())
}
