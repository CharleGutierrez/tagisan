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

/// Execution outcome of a synthesized harness run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub json_data: Option<serde_json::Value>,
    pub duration_ms: u64,
    pub success: bool,
}

impl HarnessExecutionResult {
    /// Formats human-readable combined output
    pub fn display_summary(&self) -> String {
        let mut out = format!(
            "Exit Code: {} (Duration: {}ms)\n",
            self.exit_code, self.duration_ms
        );
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

/// Secure harness runner that audits invocations against AgentShield before execution
pub struct HarnessRunner {
    python_bin: PathBuf,
    default_timeout: Duration,
    enforce_shield: bool,
}

impl HarnessRunner {
    /// Maximum allowed output capture (1 MB) to prevent memory exhaustion
    pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

    pub fn new() -> Result<Self> {
        let python_bin = PythonRuntime::find_python().unwrap_or_else(|| PathBuf::from("python3"));
        Ok(Self {
            python_bin,
            default_timeout: Duration::from_secs(30),
            enforce_shield: true,
        })
    }

    pub fn with_python(mut self, bin: impl Into<PathBuf>) -> Self {
        self.python_bin = bin.into();
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

    /// Safely executes a CLI harness script with argument validation through AgentShield
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

        // 3. Process Execution
        let canonical_script = std::fs::canonicalize(script).unwrap_or_else(|_| script.to_path_buf());
        let mut cmd = Command::new(&self.python_bin);
        cmd.arg(&canonical_script);
        for arg in args {
            cmd.arg(arg);
        }

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        } else if let Some(parent) = script.parent() {
            cmd.current_dir(parent);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let start_time = Instant::now();

        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to spawn harness process '{}': {}",
                script.display(),
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

        // 4. JSON parse attempt
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
