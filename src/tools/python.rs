use super::ToolHandler;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use super::PythonRuntime;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// =========================================================================
// 1. PythonEvalTool
// =========================================================================

/// Tool for evaluating Python 3 code snippet directly using the Python runtime
#[derive(Debug, Clone)]
pub struct PythonEvalTool {
    runtime: Option<Arc<PythonRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PythonEvalTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonEvalTool {
    pub fn new() -> Self {
        Self {
            runtime: PythonRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<PythonRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    fn get_runtime(&self) -> Result<Arc<PythonRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PythonRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PythonEvalTool {
    fn name(&self) -> &'static str {
        "python_eval"
    }

    fn description(&self) -> &'static str {
        "Evaluate Python 3 code snippet directly using the system Python runtime. Supports standard library, math, JSON, and installed packages."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The Python 3 code snippet to execute."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 30)."
                }
            },
            "required": ["code"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        // AgentShield Security Scanning
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block {
            reason,
            threat_level,
        } = verdict
        {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let code = arguments
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TagisanError::Execution("Missing required parameter: 'code'".to_string())
            })?;

        if code.trim().is_empty() {
            return Err(TagisanError::Execution("Parameter 'code' cannot be empty".to_string()));
        }

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let runtime = self.get_runtime()?;
        let res = runtime
            .eval(code, timeout_duration, None, self.working_dir.clone())
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 2. PythonRunTool
// =========================================================================

/// Tool for running a Python (.py) script file with arguments
#[derive(Debug, Clone)]
pub struct PythonRunTool {
    runtime: Option<Arc<PythonRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PythonRunTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonRunTool {
    pub fn new() -> Self {
        Self {
            runtime: PythonRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<PythonRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    fn get_runtime(&self) -> Result<Arc<PythonRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PythonRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PythonRunTool {
    fn name(&self) -> &'static str {
        "python_run"
    }

    fn description(&self) -> &'static str {
        "Execute a Python (.py) script file with command-line arguments using the Python runtime."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the Python script to execute."
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional command-line arguments to pass to the script."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 30)."
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        // AgentShield Security Scanning
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block {
            reason,
            threat_level,
        } = verdict
        {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let file_path_str = arguments
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TagisanError::Execution("Missing required parameter: 'file_path'".to_string())
            })?;

        if file_path_str.trim().is_empty() {
            return Err(TagisanError::Execution("Parameter 'file_path' cannot be empty".to_string()));
        }

        let args_vec: Vec<String> = arguments
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        // Resolve path relative to working_dir if configured and path is relative
        let resolved_path = match &self.working_dir {
            Some(base) => {
                let p = PathBuf::from(file_path_str);
                if p.is_relative() {
                    base.join(p)
                } else {
                    p
                }
            }
            None => PathBuf::from(file_path_str),
        };

        if !resolved_path.is_file() {
            return Err(TagisanError::Execution(format!(
                "Python script file not found: '{}'",
                resolved_path.display()
            )));
        }

        let runtime = self.get_runtime()?;
        let res = runtime
            .run_file(&resolved_path, &args_vec, Some(timeout_duration.as_secs()))
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 3. PythonInstallTool
// =========================================================================

/// Tool for installing Python packages using uv or pip
#[derive(Debug, Clone)]
pub struct PythonInstallTool {
    runtime: Option<Arc<PythonRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PythonInstallTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonInstallTool {
    pub fn new() -> Self {
        Self {
            runtime: PythonRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(300),
        }
    }

    pub fn with_runtime(runtime: Arc<PythonRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(300),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    fn get_runtime(&self) -> Result<Arc<PythonRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PythonRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PythonInstallTool {
    fn name(&self) -> &'static str {
        "python_install"
    }

    fn description(&self) -> &'static str {
        "Install Python packages into the environment using uv or pip. Enforces pre-built binary wheels only unless allow_native is explicitly true."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "packages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of PyPI package names to install (e.g. ['requests', 'pydantic'])."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly permit native C extension compilation from source. If false, enforces wheels only (--only-binary=:all:) (default: false)."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 300)."
                }
            },
            "required": ["packages"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block {
            reason,
            threat_level,
        } = verdict
        {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let packages: Vec<String> = arguments
            .get("packages")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|val| val.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let allow_native = arguments
            .get("allow_native")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let runtime = self.get_runtime()?;
        let res = runtime
            .install(
                &packages,
                allow_native,
                timeout_duration,
                self.working_dir.clone(),
            )
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 4. PythonAutoResolveTool & Error Parser
// =========================================================================

/// Extracts a missing Python package name from stderr (e.g. "ModuleNotFoundError: No module named 'requests'")
pub fn extract_missing_python_package(stderr: &str) -> Option<String> {
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

/// Tool for autonomously intercepting Python missing package errors and installing them
#[derive(Debug, Clone)]
pub struct PythonAutoResolveTool {
    installer: PythonInstallTool,
}

impl Default for PythonAutoResolveTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonAutoResolveTool {
    pub fn new() -> Self {
        Self {
            installer: PythonInstallTool::new(),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.installer = self.installer.with_working_dir(dir);
        self
    }
}

#[async_trait]
impl ToolHandler for PythonAutoResolveTool {
    fn name(&self) -> &'static str {
        "python_auto_resolve"
    }

    fn description(&self) -> &'static str {
        "Autonomously detect missing Python package from stderr and install it using uv or pip."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "stderr": {
                    "type": "string",
                    "description": "The stderr output from a failed Python execution containing 'ModuleNotFoundError: No module named ...'."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly permit native C extension compilation (default: false)."
                }
            },
            "required": ["stderr"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let stderr = arguments
            .get("stderr")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'stderr'".to_string()))?;

        let allow_native = arguments
            .get("allow_native")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(pkg) = extract_missing_python_package(stderr) {
            let install_args = json!({
                "packages": [pkg],
                "allow_native": allow_native
            });
            self.installer.execute(install_args).await
        } else {
            Err(TagisanError::Execution(
                "Could not detect any missing Python package in the provided stderr output.".to_string(),
            ))
        }
    }
}

