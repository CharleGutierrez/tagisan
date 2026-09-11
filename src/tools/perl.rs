use super::ToolHandler;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use super::PerlRuntime;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// =========================================================================
// 1. PerlEvalTool
// =========================================================================

/// Tool for evaluating Perl 5 code snippet or one-liner directly using the Perl runtime
#[derive(Debug, Clone)]
pub struct PerlEvalTool {
    runtime: Option<Arc<PerlRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PerlEvalTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PerlEvalTool {
    pub fn new() -> Self {
        Self {
            runtime: PerlRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<PerlRuntime>) -> Self {
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

    fn get_runtime(&self) -> Result<Arc<PerlRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PerlRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PerlEvalTool {
    fn name(&self) -> &'static str {
        "perl_eval"
    }

    fn description(&self) -> &'static str {
        "Evaluate Perl 5 code snippet or one-liner directly using the Perl runtime. Ideal for text processing, regular expressions, and system scripting."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The Perl 5 code snippet or one-liner to execute."
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
// 2. PerlRunTool
// =========================================================================

/// Tool for running a Perl (.pl) script file with arguments
#[derive(Debug, Clone)]
pub struct PerlRunTool {
    runtime: Option<Arc<PerlRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PerlRunTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PerlRunTool {
    pub fn new() -> Self {
        Self {
            runtime: PerlRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<PerlRuntime>) -> Self {
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

    fn get_runtime(&self) -> Result<Arc<PerlRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PerlRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PerlRunTool {
    fn name(&self) -> &'static str {
        "perl_run"
    }

    fn description(&self) -> &'static str {
        "Execute a Perl (.pl) script file with command-line arguments using the Perl runtime."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the Perl script to execute."
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
                "Perl script file not found: '{}'",
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
// 3. PerlInstallTool
// =========================================================================

/// Tool for installing CPAN modules using cpanm / local::lib
#[derive(Debug, Clone)]
pub struct PerlInstallTool {
    runtime: Option<Arc<PerlRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for PerlInstallTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PerlInstallTool {
    pub fn new() -> Self {
        Self {
            runtime: PerlRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(300),
        }
    }

    pub fn with_runtime(runtime: Arc<PerlRuntime>) -> Self {
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

    fn get_runtime(&self) -> Result<Arc<PerlRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            PerlRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for PerlInstallTool {
    fn name(&self) -> &'static str {
        "perl_install"
    }

    fn description(&self) -> &'static str {
        "Install CPAN modules into local isolated sandbox (.tagisan/perl5/) using cpanm. Defaults to pure-Perl mode unless allow_native is explicitly true."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "modules": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of CPAN module names to install (e.g. ['JSON::MaybeXS', 'Path::Tiny'])."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly permit native C/XS compilation via make/gcc. If false, enforces pure-Perl mode (default: false)."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 300)."
                }
            },
            "required": ["modules"]
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

        let modules: Vec<String> = arguments
            .get("modules")
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
                &modules,
                allow_native,
                timeout_duration,
                self.working_dir.clone(),
            )
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 4. PerlAutoResolveTool & Error Parser
// =========================================================================

/// Extracts a missing Perl module name from stderr (e.g. "Can't locate Path/Tiny.pm in @INC")
pub fn extract_missing_perl_module(stderr: &str) -> Option<String> {
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

/// Tool for autonomously intercepting Perl missing module errors and installing them
#[derive(Debug, Clone)]
pub struct PerlAutoResolveTool {
    installer: PerlInstallTool,
}

impl Default for PerlAutoResolveTool {
    fn default() -> Self {
        Self::new()
    }
}

impl PerlAutoResolveTool {
    pub fn new() -> Self {
        Self {
            installer: PerlInstallTool::new(),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.installer = self.installer.with_working_dir(dir);
        self
    }
}

#[async_trait]
impl ToolHandler for PerlAutoResolveTool {
    fn name(&self) -> &'static str {
        "perl_auto_resolve"
    }

    fn description(&self) -> &'static str {
        "Autonomously detect missing CPAN module from Perl stderr and install it into local sandbox (.tagisan/perl5/)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "stderr": {
                    "type": "string",
                    "description": "The stderr output from a failed Perl execution containing 'Can't locate ... in @INC'."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly permit native C compilation for the missing module (default: false)."
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

        if let Some(module) = extract_missing_perl_module(stderr) {
            let install_args = json!({
                "modules": [module],
                "allow_native": allow_native
            });
            self.installer.execute(install_args).await
        } else {
            Err(TagisanError::Execution(
                "Could not detect any missing CPAN module in the provided stderr output.".to_string(),
            ))
        }
    }
}

