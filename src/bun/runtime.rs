use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

/// Result of a Bun process execution containing exit code, stdout, stderr, and duration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BunExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub success: bool,
}

impl BunExecutionResult {
    /// Returns true if process exited with exit code 0
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Formats the stdout and stderr into a combined human-readable string
    pub fn combined_output(&self) -> String {
        let mut out = format!(
            "Exit Code: {}\nDuration: {}ms\n",
            self.exit_code, self.duration_ms
        );
        if !self.stdout.is_empty() {
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

/// Production-grade Bun runtime integration managing execution, bundling, testing, and package management
#[derive(Debug, Clone)]
pub struct BunRuntime {
    bun_path: PathBuf,
    default_timeout: Duration,
}

impl BunRuntime {
    /// Maximum allowed output capture (1 MB) to prevent memory exhaustion
    pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

    /// Discovers the Bun binary on the host system
    pub fn find_bun() -> Option<PathBuf> {
        // 1. Check TAGISAN_BUN_PATH
        if let Ok(p) = std::env::var("TAGISAN_BUN_PATH") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 2. Check BUN_BINARY
        if let Ok(p) = std::env::var("BUN_BINARY") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 3. Check known host location /home/dyna/.local/bin/bun
        let host_path = PathBuf::from("/home/dyna/.local/bin/bun");
        if host_path.is_file() {
            return Some(host_path);
        }

        // 4. Check user home ~/.bun/bin/bun and ~/.local/bin/bun
        if let Ok(home) = std::env::var("HOME") {
            let home_bun = PathBuf::from(&home).join(".bun").join("bin").join("bun");
            if home_bun.is_file() {
                return Some(home_bun);
            }
            let local_bun = PathBuf::from(&home).join(".local").join("bin").join("bun");
            if local_bun.is_file() {
                return Some(local_bun);
            }
        }

        // 5. Search in PATH directories
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                let candidate = dir.join("bun");
                if candidate.is_file() {
                    return Some(candidate);
                }
                #[cfg(target_os = "windows")]
                {
                    let candidate_exe = dir.join("bun.exe");
                    if candidate_exe.is_file() {
                        return Some(candidate_exe);
                    }
                }
            }
        }

        // 6. Standard Unix locations
        for standard in ["/usr/local/bin/bun", "/usr/bin/bun", "/opt/bun/bin/bun"] {
            let p = PathBuf::from(standard);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Discovers Bun or returns an error if not found
    pub fn new() -> Result<Self> {
        let bun_path = Self::find_bun().ok_or_else(|| {
            TagisanError::Execution(
                "Bun binary not found. Please install Bun from https://bun.sh or set BUN_BINARY or TAGISAN_BUN_PATH.".to_string(),
            )
        })?;

        Ok(Self {
            bun_path,
            default_timeout: Duration::from_secs(30),
        })
    }

    /// Instantiates BunRuntime with an explicit binary path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            bun_path: path.into(),
            default_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for BunRuntime {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_path("bun"))
    }
}

impl BunRuntime {

    /// Sets default execution timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Returns the resolved Bun binary path
    pub fn bun_path(&self) -> &Path {
        &self.bun_path
    }

    /// Checks if Bun binary exists and is executable
    pub fn is_available(&self) -> bool {
        self.bun_path.is_file()
    }

    /// Retrieves Bun version (e.g. "1.4.2")
    pub async fn version(&self) -> Result<String> {
        let mut cmd = Command::new(&self.bun_path);
        cmd.arg("--version");
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        if res.success {
            Ok(res.stdout.trim().to_string())
        } else {
            Err(TagisanError::Execution(format!(
                "Failed to get Bun version: {}",
                res.stderr
            )))
        }
    }

    /// Low-level process runner with strict timeout, SIGKILL cleanup, and bounded I/O buffers
    async fn execute_command(
        &self,
        mut cmd: Command,
        timeout_duration: Duration,
        stdin_input: Option<&[u8]>,
    ) -> Result<BunExecutionResult> {
        cmd.stdin(if stdin_input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        let start_time = Instant::now();
        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to spawn Bun process '{}': {e}",
                self.bun_path.display()
            ))
        })?;

        // Write stdin if provided
        if let Some(input) = stdin_input {
            if let Some(mut stdin) = child.stdin.take() {
                let input_vec = input.to_vec();
                tokio::spawn(async move {
                    let _ = stdin.write_all(&input_vec).await;
                    let _ = stdin.flush().await;
                });
            }
        }

        let stdout = child.stdout.take().ok_or_else(|| {
            TagisanError::Execution("Failed to capture Bun stdout pipe".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            TagisanError::Execution("Failed to capture Bun stderr pipe".to_string())
        })?;

        // Spawn bounded async reader tasks
        let stdout_task = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout);
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            while let Ok(n) = reader.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                if buf.len() < Self::MAX_OUTPUT_BYTES {
                    let to_take = n.min(Self::MAX_OUTPUT_BYTES - buf.len());
                    buf.extend_from_slice(&chunk[..to_take]);
                }
            }
            buf
        });

        let stderr_task = tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            while let Ok(n) = reader.read(&mut chunk).await {
                if n == 0 {
                    break;
                }
                if buf.len() < Self::MAX_OUTPUT_BYTES {
                    let to_take = n.min(Self::MAX_OUTPUT_BYTES - buf.len());
                    buf.extend_from_slice(&chunk[..to_take]);
                }
            }
            buf
        });

        // Await child completion within timeout
        let wait_res = timeout(timeout_duration, child.wait()).await;

        let status = match wait_res {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                let _ = child.kill().await;
                return Err(TagisanError::Execution(format!(
                    "Error waiting for Bun process: {e}"
                )));
            }
            Err(_) => {
                // SIGKILL child process on timeout
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(TagisanError::Execution(format!(
                    "Bun execution timed out after {:.1}s",
                    timeout_duration.as_secs_f32()
                )));
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let raw_stdout = stdout_task.await.unwrap_or_default();
        let raw_stderr = stderr_task.await.unwrap_or_default();

        let stdout_truncated = raw_stdout.len() >= Self::MAX_OUTPUT_BYTES;
        let stderr_truncated = raw_stderr.len() >= Self::MAX_OUTPUT_BYTES;

        let mut stdout = String::from_utf8_lossy(&raw_stdout).to_string();
        if stdout_truncated {
            stdout.push_str("\n... [Output truncated: exceeded 1MB limit]");
        }

        let mut stderr = String::from_utf8_lossy(&raw_stderr).to_string();
        if stderr_truncated {
            stderr.push_str("\n... [Stderr truncated: exceeded 1MB limit]");
        }

        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        Ok(BunExecutionResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
            success,
        })
    }

    /// Evaluates TypeScript or JavaScript code snippet directly with Bun
    /// Supports top-level await, ESM imports, and TypeScript syntax.
    pub async fn eval(
        &self,
        code: &str,
        timeout_duration: Duration,
        env: Option<HashMap<String, String>>,
        cwd: Option<PathBuf>,
    ) -> Result<BunExecutionResult> {
        let trimmed = code.trim();
        let is_pure_expression = !trimmed.contains('\n')
            && !trimmed.contains(';')
            && !trimmed.contains("console.log")
            && !trimmed.contains("process.stdout")
            && !trimmed.starts_with("import ")
            && !trimmed.starts_with("export ")
            && !trimmed.starts_with("const ")
            && !trimmed.starts_with("let ")
            && !trimmed.starts_with("var ")
            && !trimmed.starts_with("function ")
            && !trimmed.starts_with("class ")
            && !trimmed.starts_with("type ")
            && !trimmed.starts_with("interface ")
            && !trimmed.starts_with("enum ")
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ");

        let mut cmd = Command::new(&self.bun_path);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        if let Some(env_vars) = env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        if is_pure_expression {
            cmd.arg("-p").arg(trimmed);
            self.execute_command(cmd, timeout_duration, None).await
        } else if trimmed.len() <= 65536 {
            cmd.arg("-e").arg(trimmed);
            self.execute_command(cmd, timeout_duration, None).await
        } else {
            cmd.args(["run", "-"]);
            self.execute_command(cmd, timeout_duration, Some(trimmed.as_bytes()))
                .await
        }
    }

    /// Executes a TypeScript (.ts) or JavaScript (.js) script file with arguments
    pub async fn run_script(
        &self,
        script_path: impl AsRef<Path>,
        args: &[String],
        timeout_duration: Duration,
        env: Option<HashMap<String, String>>,
        cwd: Option<PathBuf>,
    ) -> Result<BunExecutionResult> {
        let script = script_path.as_ref();
        let mut cmd = Command::new(&self.bun_path);
        cmd.arg("run").arg(script);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        if let Some(env_vars) = env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        self.execute_command(cmd, timeout_duration, None).await
    }

    /// Executes `bun test` on files, directories, or test suites
    pub async fn test(
        &self,
        target_path: impl AsRef<Path>,
        args: &[String],
        timeout_duration: Duration,
        cwd: Option<PathBuf>,
    ) -> Result<BunExecutionResult> {
        let mut cmd = Command::new(&self.bun_path);
        cmd.arg("test");
        let target = target_path.as_ref().to_string_lossy();
        if !target.is_empty() && target != "." {
            cmd.arg(target.as_ref());
        }
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        self.execute_command(cmd, timeout_duration, None).await
    }

    /// Installs npm packages or project dependencies using Bun's package manager
    pub async fn install(
        &self,
        packages: &[String],
        is_dev: bool,
        allow_native: bool,
        timeout_duration: Duration,
        cwd: Option<PathBuf>,
    ) -> Result<BunExecutionResult> {
        let mut cmd = Command::new(&self.bun_path);
        if packages.is_empty() {
            cmd.arg("install");
        } else {
            cmd.arg("add");
            if is_dev {
                cmd.arg("-d");
            }
            cmd.args(packages);
        }

        if !allow_native {
            cmd.arg("--ignore-scripts");
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        self.execute_command(cmd, timeout_duration, None).await
    }

    /// Bundles and optimizes TypeScript/JavaScript code using Bun's fast bundler
    pub async fn build(
        &self,
        entrypoint: impl AsRef<Path>,
        outdir: impl AsRef<Path>,
        minify: bool,
        target: &str,
        timeout_duration: Duration,
        cwd: Option<PathBuf>,
    ) -> Result<BunExecutionResult> {
        let mut cmd = Command::new(&self.bun_path);
        cmd.arg("build");
        cmd.arg(entrypoint.as_ref());
        cmd.arg(format!("--outdir={}", outdir.as_ref().display()));
        if minify {
            cmd.arg("--minify");
        }
        if !target.is_empty() {
            cmd.arg(format!("--target={target}"));
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        self.execute_command(cmd, timeout_duration, None).await
    }
}
