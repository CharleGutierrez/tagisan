use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

/// Result of a Python process execution containing exit code, stdout, stderr, and duration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub success: bool,
}

impl PythonExecutionResult {
    /// Returns true if process exited with exit code 0
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Formats stdout and stderr into a combined human-readable string
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

/// Production-grade Python runtime integration managing execution, package inspection, and syntax validation
#[derive(Debug, Clone)]
pub struct PythonRuntime {
    python_path: PathBuf,
    default_timeout: Duration,
}

impl PythonRuntime {
    /// Maximum allowed output capture (1 MB) to prevent memory exhaustion
    pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

    /// Discovers the Python binary on the host system
    pub fn find_python() -> Option<PathBuf> {
        // 1. Check TAGISAN_PYTHON_PATH
        if let Ok(p) = std::env::var("TAGISAN_PYTHON_PATH") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 2. Check PYTHON_BINARY
        if let Ok(p) = std::env::var("PYTHON_BINARY") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 3. Check virtualenv in current directory or ancestors
        let venv_rel_candidates = [
            ".venv/Scripts/python.exe",
            ".venv/bin/python",
            "venv/Scripts/python.exe",
            "venv/bin/python",
        ];
        for rel in venv_rel_candidates {
            let p = PathBuf::from(rel);
            if p.is_file() {
                return Some(p);
            }
        }

        // 4. Search in PATH directories
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                #[cfg(target_os = "windows")]
                {
                    for exe_name in ["python.exe", "python3.exe"] {
                        let candidate = dir.join(exe_name);
                        // Filter out Windows Store 0-byte execution aliases
                        if candidate.is_file() {
                            if let Ok(meta) = candidate.metadata() {
                                if meta.len() > 0 {
                                    return Some(candidate);
                                }
                            }
                        }
                    }
                }

                for bin_name in ["python3", "python"] {
                    let candidate = dir.join(bin_name);
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }

        // 5. Standard Windows installation paths
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let base = PathBuf::from(local_app_data).join("Programs").join("Python");
                for ver in [
                    "Python314",
                    "Python313",
                    "Python312",
                    "Python311",
                    "Python310",
                ] {
                    let candidate = base.join(ver).join("python.exe");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }

            for root in ["C:\\Program Files\\Python", "C:\\Python"] {
                for ver in ["314", "313", "312", "311", "310"] {
                    let candidate = PathBuf::from(format!("{root}{ver}\\python.exe"));
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
        }

        // 6. Standard Unix locations
        for standard in [
            "/usr/bin/python3",
            "/usr/local/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/bin/python",
        ] {
            let p = PathBuf::from(standard);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Discovers Python or returns an error if not found
    pub fn new() -> Result<Self> {
        let python_path = Self::find_python().ok_or_else(|| {
            TagisanError::Execution(
                "Python binary not found. Please install Python 3 or set PYTHON_BINARY or TAGISAN_PYTHON_PATH.".to_string(),
            )
        })?;

        Ok(Self {
            python_path,
            default_timeout: Duration::from_secs(30),
        })
    }

    /// Instantiates PythonRuntime with an explicit binary path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            python_path: path.into(),
            default_timeout: Duration::from_secs(30),
        }
    }

    /// Sets default execution timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Returns the resolved Python binary path
    pub fn python_path(&self) -> &Path {
        &self.python_path
    }

    /// Checks if Python binary exists and is executable
    pub fn is_available(&self) -> bool {
        self.python_path.is_file()
    }

    /// Low-level process runner with strict timeout, SIGKILL cleanup, and bounded I/O buffers
    async fn execute_command(
        &self,
        mut cmd: Command,
        timeout_duration: Duration,
        stdin_input: Option<&[u8]>,
    ) -> Result<PythonExecutionResult> {
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
                "Failed to spawn Python process '{}': {e}",
                self.python_path.display()
            ))
        })?;

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
            TagisanError::Execution("Failed to capture Python stdout pipe".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            TagisanError::Execution("Failed to capture Python stderr pipe".to_string())
        })?;

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

        let wait_res = timeout(timeout_duration, child.wait()).await;

        let status = match wait_res {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                let _ = child.kill().await;
                return Err(TagisanError::Execution(format!(
                    "Error waiting for Python process: {e}"
                )));
            }
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(TagisanError::Execution(format!(
                    "Python execution timed out after {:.1}s",
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

        Ok(PythonExecutionResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
            success,
        })
    }

    /// Retrieves Python version string (e.g. "Python 3.14.6")
    pub async fn version(&self) -> Result<String> {
        let mut cmd = Command::new(&self.python_path);
        cmd.arg("--version");
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        if res.success {
            let ver = if !res.stdout.trim().is_empty() {
                res.stdout.trim()
            } else {
                res.stderr.trim()
            };
            Ok(ver.to_string())
        } else {
            Err(TagisanError::Execution(format!(
                "Failed to get Python version: {}",
                res.stderr
            )))
        }
    }

    /// Evaluates Python code snippet directly with configurable timeout
    pub async fn eval_code(&self, code: &str, timeout_secs: Option<u64>) -> Result<PythonExecutionResult> {
        let dur = timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);
        self.eval(code, dur, None, None).await
    }

    /// Detailed evaluation with environment variables and working directory support
    pub async fn eval(
        &self,
        code: &str,
        timeout_duration: Duration,
        env: Option<HashMap<String, String>>,
        cwd: Option<PathBuf>,
    ) -> Result<PythonExecutionResult> {
        let trimmed = code.trim();
        let mut cmd = Command::new(&self.python_path);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        if let Some(env_vars) = env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        // Prevent buffering in python output
        cmd.env("PYTHONUNBUFFERED", "1");

        if trimmed.len() <= 32768 {
            cmd.arg("-c").arg(trimmed);
            self.execute_command(cmd, timeout_duration, None).await
        } else {
            cmd.arg("-");
            self.execute_command(cmd, timeout_duration, Some(trimmed.as_bytes()))
                .await
        }
    }

    /// Executes a Python script file (.py) with command-line arguments
    pub async fn run_file(
        &self,
        path: &Path,
        args: &[String],
        timeout_secs: Option<u64>,
    ) -> Result<PythonExecutionResult> {
        let timeout_duration = timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let mut cmd = Command::new(&self.python_path);
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.arg(path);
        cmd.args(args);

        self.execute_command(cmd, timeout_duration, None).await
    }

    /// Validates syntax of Python code without executing it
    pub async fn check_syntax(&self, code: &str) -> Result<bool> {
        let check_script = r#"
import sys
try:
    compile(sys.stdin.read(), '<string>', 'exec')
    sys.exit(0)
except SyntaxError:
    sys.exit(1)
"#;
        let mut cmd = Command::new(&self.python_path);
        cmd.arg("-c").arg(check_script);
        let res = self
            .execute_command(cmd, Duration::from_secs(5), Some(code.as_bytes()))
            .await?;
        Ok(res.success)
    }

    /// Checks if a Python package or module is installed and importable
    pub async fn check_package(&self, package_name: &str) -> Result<bool> {
        let check_script = format!(
            "import importlib.util, sys; sys.exit(0 if importlib.util.find_spec({:?}) is not None else 1)",
            package_name
        );
        let mut cmd = Command::new(&self.python_path);
        cmd.arg("-c").arg(&check_script);
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        Ok(res.success)
    }
}

impl Default for PythonRuntime {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_path("python"))
    }
}
