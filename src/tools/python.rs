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
