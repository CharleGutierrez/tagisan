use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::timeout;

/// Result of a Perl process execution containing exit code, stdout, stderr, and duration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerlExecutionResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub success: bool,
}

impl PerlExecutionResult {
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

/// Production-grade Perl 5 runtime integration managing execution, regex processing, and syntax validation
#[derive(Debug, Clone)]
pub struct PerlRuntime {
    perl_path: PathBuf,
    default_timeout: Duration,
}

impl PerlRuntime {
    /// Maximum allowed output capture (1 MB) to prevent memory exhaustion
    pub const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

    /// Discovers the Perl binary on the host system
    pub fn find_perl() -> Option<PathBuf> {
        // 1. Check TAGISAN_PERL_PATH
        if let Ok(p) = std::env::var("TAGISAN_PERL_PATH") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 2. Check PERL_BINARY
        if let Ok(p) = std::env::var("PERL_BINARY") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 3. Check Git for Windows Perl (known host location)
        let git_perl_known = PathBuf::from(r"C:\Users\CharleOGutierrez\AppData\Local\Programs\Git\usr\bin\perl.exe");
        if git_perl_known.is_file() {
            return Some(git_perl_known);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let candidate = PathBuf::from(local_app_data)
                    .join("Programs")
                    .join("Git")
                    .join("usr")
                    .join("bin")
                    .join("perl.exe");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }

            for p in [
                r"C:\Program Files\Git\usr\bin\perl.exe",
                r"C:\Git\usr\bin\perl.exe",
                r"C:\Strawberry\perl\bin\perl.exe",
                r"C:\strawberry\perl\bin\perl.exe",
            ] {
                let candidate = PathBuf::from(p);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 4. Search in PATH directories
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                #[cfg(target_os = "windows")]
                {
                    let candidate = dir.join("perl.exe");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }

                let candidate = dir.join("perl");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 5. Standard Unix locations
        for standard in [
            "/usr/bin/perl",
            "/usr/local/bin/perl",
            "/opt/homebrew/bin/perl",
        ] {
            let p = PathBuf::from(standard);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Discovers Perl or returns an error if not found
    pub fn new() -> Result<Self> {
        let perl_path = Self::find_perl().ok_or_else(|| {
            TagisanError::Execution(
                "Perl binary not found. Please install Perl 5 or set PERL_BINARY or TAGISAN_PERL_PATH.".to_string(),
            )
        })?;

        Ok(Self {
            perl_path,
            default_timeout: Duration::from_secs(30),
        })
    }

    /// Instantiates PerlRuntime with an explicit binary path
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            perl_path: path.into(),
            default_timeout: Duration::from_secs(30),
        }
    }

    /// Sets default execution timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Returns the resolved Perl binary path
    pub fn perl_path(&self) -> &Path {
        &self.perl_path
    }

    /// Checks if Perl binary exists and is executable
    pub fn is_available(&self) -> bool {
        self.perl_path.is_file()
    }

    /// Low-level process runner with strict timeout, SIGKILL cleanup, and bounded I/O buffers
    async fn execute_command(
        &self,
        mut cmd: Command,
        timeout_duration: Duration,
        stdin_input: Option<&[u8]>,
    ) -> Result<PerlExecutionResult> {
        cmd.stdin(if stdin_input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);

        // Auto-inject .tagisan/perl5/lib/perl5 into PERL5LIB if it exists
        let tagisan_p5_lib = PathBuf::from(".tagisan").join("perl5").join("lib").join("perl5");
        if tagisan_p5_lib.is_dir() {
            let existing_lib = std::env::var("PERL5LIB").unwrap_or_default();
            let p5_str = tagisan_p5_lib.to_string_lossy();
            let new_lib = if existing_lib.is_empty() {
                p5_str.to_string()
            } else if !existing_lib.contains(&*p5_str) {
                format!("{p5_str}:{existing_lib}")
            } else {
                existing_lib
            };
            cmd.env("PERL5LIB", new_lib);
        }

        let start_time = Instant::now();
        let mut child = cmd.spawn().map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to spawn Perl process '{}': {e}",
                self.perl_path.display()
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
            TagisanError::Execution("Failed to capture Perl stdout pipe".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            TagisanError::Execution("Failed to capture Perl stderr pipe".to_string())
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
                    "Error waiting for Perl process: {e}"
                )));
            }
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(TagisanError::Execution(format!(
                    "Perl execution timed out after {:.1}s",
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

        Ok(PerlExecutionResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
            success,
        })
    }

    /// Retrieves Perl version string (e.g. "5.42.3" or parsed version line)
    pub async fn version(&self) -> Result<String> {
        let mut cmd = Command::new(&self.perl_path);
        cmd.arg("-v");
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        if res.success {
            // Find line with "This is perl" or "(v5.x.y)"
            for line in res.stdout.lines() {
                let trimmed = line.trim();
                if trimmed.contains("This is perl") {
                    // Extract version like (v5.42.3) if present
                    if let Some(start) = trimmed.find("(v") {
                        if let Some(end) = trimmed[start..].find(')') {
                            let ver = &trimmed[start + 1..start + end];
                            return Ok(ver.to_string());
                        }
                    }
                    return Ok(trimmed.to_string());
                }
            }
            // Fallback to first non-empty line
            let first_line = res.stdout.lines().find(|l| !l.trim().is_empty()).unwrap_or("Perl 5");
            Ok(first_line.trim().to_string())
        } else {
            Err(TagisanError::Execution(format!(
                "Failed to get Perl version: {}",
                res.stderr
            )))
        }
    }

    /// Evaluates Perl code snippet or one-liner with configurable timeout
    pub async fn eval_code(&self, code: &str, timeout_secs: Option<u64>) -> Result<PerlExecutionResult> {
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
    ) -> Result<PerlExecutionResult> {
        let trimmed = code.trim();
        let mut cmd = Command::new(&self.perl_path);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        if let Some(env_vars) = env {
            for (k, v) in env_vars {
                cmd.env(k, v);
            }
        }

        if trimmed.len() <= 32768 {
            cmd.arg("-e").arg(trimmed);
            self.execute_command(cmd, timeout_duration, None).await
        } else {
            cmd.arg("-");
            self.execute_command(cmd, timeout_duration, Some(trimmed.as_bytes()))
                .await
        }
    }

    /// Executes a Perl script file (.pl) with command-line arguments
    pub async fn run_file(
        &self,
        path: &Path,
        args: &[String],
        timeout_secs: Option<u64>,
    ) -> Result<PerlExecutionResult> {
        let timeout_duration = timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let mut cmd = Command::new(&self.perl_path);
        cmd.arg(path);
        cmd.args(args);

        self.execute_command(cmd, timeout_duration, None).await
    }

    /// Validates syntax of Perl code without executing it (`perl -c -e <code>`)
    pub async fn check_syntax(&self, code: &str) -> Result<bool> {
        let mut cmd = Command::new(&self.perl_path);
        cmd.arg("-c").arg("-e").arg(code);
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        Ok(res.success)
    }

    /// Performs regular expression text transformation on `input` using Perl regex `expr` (e.g. `s/foo/bar/g`)
    pub async fn regex_transform(
        &self,
        input: &str,
        expr: &str,
        timeout_secs: Option<u64>,
    ) -> Result<String> {
        let timeout_duration = timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let mut cmd = Command::new(&self.perl_path);
        cmd.arg("-pe").arg(expr);

        let res = self
            .execute_command(cmd, timeout_duration, Some(input.as_bytes()))
            .await?;

        if res.success {
            Ok(res.stdout)
        } else {
            Err(TagisanError::Execution(format!(
                "Perl regex transformation failed (exit code {}): {}",
                res.exit_code, res.stderr
            )))
        }
    }

    /// Discovers cpanm on the system, in PATH, or in .tagisan/bin/cpanm
    pub fn find_cpanm() -> Option<PathBuf> {
        // 1. Check TAGISAN_CPANM_PATH
        if let Ok(p) = std::env::var("TAGISAN_CPANM_PATH") {
            let path = PathBuf::from(p);
            if path.is_file() {
                return Some(path);
            }
        }

        // 2. Check .tagisan/bin/cpanm
        let local_bin = PathBuf::from(".tagisan").join("bin").join("cpanm");
        if local_bin.is_file() {
            return Some(local_bin);
        }

        // 3. Search in PATH
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                #[cfg(target_os = "windows")]
                {
                    let candidate = dir.join("cpanm.bat");
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
                let candidate = dir.join("cpanm");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 4. Standard locations
        for standard in ["/usr/bin/cpanm", "/usr/local/bin/cpanm", "/opt/homebrew/bin/cpanm"] {
            let p = PathBuf::from(standard);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Verifies whether a CPAN module is installed and loadable (`perl -M$module -e 1`)
    pub async fn check_module(&self, module_name: &str) -> Result<bool> {
        let mut cmd = Command::new(&self.perl_path);
        cmd.arg(format!("-M{module_name}")).arg("-e").arg("1");
        let res = self
            .execute_command(cmd, Duration::from_secs(5), None)
            .await?;
        Ok(res.success)
    }

    /// Installs CPAN modules into a local directory (`.tagisan/perl5/`) using cpanm or cpan
    ///
    /// When `allow_native` is `false`, pure-Perl mode is enforced and C compiler execution is blocked.
    /// When `allow_native` is `true`, C/XS compilation via make/gcc is permitted.
    pub async fn install(
        &self,
        modules: &[String],
        allow_native: bool,
        timeout_duration: Duration,
        cwd: Option<PathBuf>,
    ) -> Result<PerlExecutionResult> {
        if modules.is_empty() {
            return Err(TagisanError::Execution(
                "No Perl modules specified for installation.".to_string(),
            ));
        }

        let base_dir = cwd.clone().unwrap_or_else(|| PathBuf::from("."));
        let local_lib_dir = base_dir.join(".tagisan").join("perl5");
        let _ = std::fs::create_dir_all(&local_lib_dir);

        let cpanm_opt = Self::find_cpanm();
        let mut cmd = if let Some(cpanm_bin) = cpanm_opt {
            let mut c = Command::new(cpanm_bin);
            c.arg("-l").arg(&local_lib_dir);
            c.arg("--notest");
            if !allow_native {
                c.arg("--pureperl-only");
            }
            c.args(modules);
            c
        } else {
            // Check if local standalone cpanm exists or bootstrap it
            let bin_dir = PathBuf::from(".tagisan").join("bin");
            let _ = std::fs::create_dir_all(&bin_dir);
            let local_cpanm = bin_dir.join("cpanm");

            if !local_cpanm.is_file() {
                // Fetch standalone cpanmin.us bootstrap script via curl or lwp
                let curl_cmd = std::process::Command::new("curl")
                    .args(["-sL", "https://cpanmin.us", "-o", local_cpanm.to_str().unwrap_or("")])
                    .output();
                if let Ok(out) = curl_cmd {
                    if out.status.success() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = std::fs::set_permissions(&local_cpanm, std::fs::Permissions::from_mode(0o755));
                        }
                    }
                }
            }

            if local_cpanm.is_file() {
                let mut c = Command::new(&self.perl_path);
                c.arg(&local_cpanm);
                c.arg("-l").arg(&local_lib_dir);
                c.arg("--notest");
                if !allow_native {
                    c.arg("--pureperl-only");
                }
                c.args(modules);
                c
            } else {
                // Fallback to core CPAN shell
                let mut c = Command::new(&self.perl_path);
                c.arg("-MCPAN").arg("-e");
                let cpan_cmd = format!(
                    "CPAN::Shell->install('{}')",
                    modules.join("', '")
                );
                c.arg(cpan_cmd);
                c
            }
        };

        if let Some(ref dir) = cwd {
            cmd.current_dir(dir);
        }

        // Set local::lib environment
        let lib_path = local_lib_dir.join("lib").join("perl5");
        let existing_lib = std::env::var("PERL5LIB").unwrap_or_default();
        let new_lib = if existing_lib.is_empty() {
            lib_path.to_string_lossy().to_string()
        } else {
            format!("{}:{}", lib_path.to_string_lossy(), existing_lib)
        };
        cmd.env("PERL5LIB", new_lib);
        cmd.env("PERL_LOCAL_LIB_ROOT", &local_lib_dir);

        self.execute_command(cmd, timeout_duration, None).await
    }
}

impl Default for PerlRuntime {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_path("perl"))
    }
}
