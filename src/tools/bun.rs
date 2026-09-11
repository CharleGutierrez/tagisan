use super::ToolHandler;
use crate::bun::BunRuntime;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// =========================================================================
// 1. BunEvalTool
// =========================================================================

/// Tool for evaluating TypeScript or JavaScript code directly using Bun
#[derive(Debug, Clone)]
pub struct BunEvalTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunEvalTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunEvalTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<BunRuntime>) -> Self {
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

    fn get_runtime(&self) -> Result<Arc<BunRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            BunRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for BunEvalTool {
    fn name(&self) -> &'static str {
        "bun_eval"
    }

    fn description(&self) -> &'static str {
        "Evaluate TypeScript or JavaScript code directly using the ultra-fast Bun runtime. Supports top-level await, ESM imports, and full TypeScript syntax."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The TypeScript or JavaScript code snippet to execute."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 30)."
                },
                "auto_resolve": {
                    "type": "boolean",
                    "description": "Whether to autonomously install missing modules/packages if import errors occur (default: false)."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly allow native C compilation during auto-installation (default: false)."
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

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let auto_resolve = arguments
            .get("auto_resolve")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let allow_native = arguments
            .get("allow_native")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let runtime = self.get_runtime()?;
        let mut res = runtime
            .eval(code, timeout_duration, None, self.working_dir.clone())
            .await?;

        if auto_resolve && !res.is_success() {
            if let Some(pkg) = extract_missing_package(&res.stderr) {
                let _ = runtime
                    .install(&[pkg], false, allow_native, timeout_duration, self.working_dir.clone())
                    .await;
                res = runtime
                    .eval(code, timeout_duration, None, self.working_dir.clone())
                    .await?;
            }
        }

        Ok(res.combined_output())
    }
}

// =========================================================================
// 2. BunRunTool
// =========================================================================

/// Tool for executing TypeScript (.ts) or JavaScript (.js) script files with arguments
#[derive(Debug, Clone)]
pub struct BunRunTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunRunTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunRunTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_runtime(runtime: Arc<BunRuntime>) -> Self {
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

    fn get_runtime(&self) -> Result<Arc<BunRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            BunRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for BunRunTool {
    fn name(&self) -> &'static str {
        "bun_run"
    }

    fn description(&self) -> &'static str {
        "Execute a TypeScript (.ts) or JavaScript (.js) script file with arguments using Bun."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the TypeScript or JavaScript script to execute."
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

        let args: Vec<String> = arguments
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|val| val.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let script_path = Path::new(file_path_str);
        let resolved_path = if script_path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(script_path)
            } else {
                script_path.to_path_buf()
            }
        } else {
            script_path.to_path_buf()
        };

        if !resolved_path.exists() {
            return Err(TagisanError::Execution(format!(
                "Script file does not exist: {}",
                resolved_path.display()
            )));
        }

        let runtime = self.get_runtime()?;
        let res = runtime
            .run_script(
                &resolved_path,
                &args,
                timeout_duration,
                None,
                self.working_dir.clone(),
            )
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 3. BunTestTool
// =========================================================================

/// Tool for executing `bun test` on files, directories, or test suites
#[derive(Debug, Clone)]
pub struct BunTestTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunTestTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunTestTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(60),
        }
    }

    pub fn with_runtime(runtime: Arc<BunRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(60),
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

    fn get_runtime(&self) -> Result<Arc<BunRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            BunRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for BunTestTool {
    fn name(&self) -> &'static str {
        "bun_test"
    }

    fn description(&self) -> &'static str {
        "Run tests using Bun's built-in fast test runner on specified files or test suites."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target test file, directory, or pattern (default: '.')."
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional flags or arguments for bun test (e.g. ['--bail', '-t', 'pattern'])."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 60)."
                }
            }
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

        let target = arguments
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let args: Vec<String> = arguments
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|val| val.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let runtime = self.get_runtime()?;
        let res = runtime
            .test(target, &args, timeout_duration, self.working_dir.clone())
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 4. BunInstallTool
// =========================================================================

/// Tool for installing npm packages or dependencies using Bun's package manager
#[derive(Debug, Clone)]
pub struct BunInstallTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunInstallTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunInstallTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(120),
        }
    }

    pub fn with_runtime(runtime: Arc<BunRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(120),
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

    fn get_runtime(&self) -> Result<Arc<BunRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            BunRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for BunInstallTool {
    fn name(&self) -> &'static str {
        "bun_install"
    }

    fn description(&self) -> &'static str {
        "Install npm packages or project dependencies at native speed using Bun's package manager."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "packages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of package names to install. If omitted, installs dependencies from package.json."
                },
                "dev": {
                    "type": "boolean",
                    "description": "Install packages as development dependencies (--dev). Default is false."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly allow native C/C++ compilation (e.g. node-gyp lifecycle scripts). Default is false (enforcing --ignore-scripts)."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 120)."
                }
            }
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

        let is_dev = arguments
            .get("dev")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
                is_dev,
                allow_native,
                timeout_duration,
                self.working_dir.clone(),
            )
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// 5. BunBuildTool
// =========================================================================

/// Tool for bundling and optimizing TypeScript/JavaScript entry points
#[derive(Debug, Clone)]
pub struct BunBuildTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunBuildTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunBuildTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(60),
        }
    }

    pub fn with_runtime(runtime: Arc<BunRuntime>) -> Self {
        Self {
            runtime: Some(runtime),
            working_dir: None,
            default_timeout: Duration::from_secs(60),
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

    fn get_runtime(&self) -> Result<Arc<BunRuntime>> {
        if let Some(ref rt) = self.runtime {
            Ok(rt.clone())
        } else {
            BunRuntime::new().map(Arc::new)
        }
    }
}

#[async_trait]
impl ToolHandler for BunBuildTool {
    fn name(&self) -> &'static str {
        "bun_build"
    }

    fn description(&self) -> &'static str {
        "Bundle and optimize TypeScript/JavaScript entry points into production-ready bundles using Bun."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "entrypoint": {
                    "type": "string",
                    "description": "Path to the entrypoint TypeScript/JavaScript file."
                },
                "outdir": {
                    "type": "string",
                    "description": "Output directory for bundled files (default: './dist')."
                },
                "minify": {
                    "type": "boolean",
                    "description": "Whether to minify bundle output (default: false)."
                },
                "target": {
                    "type": "string",
                    "description": "Target environment: 'browser', 'bun', or 'node' (default: 'bun')."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional execution timeout in seconds (default: 60)."
                }
            },
            "required": ["entrypoint"]
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

        let entrypoint_str = arguments
            .get("entrypoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TagisanError::Execution("Missing required parameter: 'entrypoint'".to_string())
            })?;

        let outdir_str = arguments
            .get("outdir")
            .and_then(|v| v.as_str())
            .unwrap_or("./dist");

        let minify = arguments
            .get("minify")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let target = arguments
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("bun");

        let timeout_duration = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let ep_path = Path::new(entrypoint_str);
        let resolved_entrypoint = if ep_path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(ep_path)
            } else {
                ep_path.to_path_buf()
            }
        } else {
            ep_path.to_path_buf()
        };

        if !resolved_entrypoint.exists() {
            return Err(TagisanError::Execution(format!(
                "Entrypoint file does not exist: {}",
                resolved_entrypoint.display()
            )));
        }

        let outdir_path = Path::new(outdir_str);
        let resolved_outdir = if outdir_path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(outdir_path)
            } else {
                outdir_path.to_path_buf()
            }
        } else {
            outdir_path.to_path_buf()
        };

        let runtime = self.get_runtime()?;
        let res = runtime
            .build(
                &resolved_entrypoint,
                &resolved_outdir,
                minify,
                target,
                timeout_duration,
                self.working_dir.clone(),
            )
            .await?;

        Ok(res.combined_output())
    }
}

// =========================================================================
// Helper: extract_missing_package
// =========================================================================

/// Extracts missing module or package name from Bun error output
pub fn extract_missing_package(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        if let Some(pos) = line.find("Cannot find package '") {
            let rest = &line[pos + "Cannot find package '".len()..];
            if let Some(end) = rest.find('\'') {
                let pkg = &rest[..end];
                if !pkg.is_empty() && !pkg.starts_with('.') && !pkg.starts_with('/') {
                    return Some(pkg.to_string());
                }
            }
        }
        if let Some(pos) = line.find("Cannot find module '") {
            let rest = &line[pos + "Cannot find module '".len()..];
            if let Some(end) = rest.find('\'') {
                let pkg = &rest[..end];
                if !pkg.is_empty() && !pkg.starts_with('.') && !pkg.starts_with('/') {
                    return Some(pkg.to_string());
                }
            }
        }
        if let Some(pos) = line.find("Could not resolve: \"") {
            let rest = &line[pos + "Could not resolve: \"".len()..];
            if let Some(end) = rest.find('"') {
                let pkg = &rest[..end];
                if !pkg.is_empty() && !pkg.starts_with('.') && !pkg.starts_with('/') {
                    return Some(pkg.to_string());
                }
            }
        }
    }
    None
}

// =========================================================================
// 6. BunAutoResolveTool
// =========================================================================

/// Tool for autonomously executing TypeScript/JavaScript with JIT missing-package detection and installation
#[derive(Debug, Clone)]
pub struct BunAutoResolveTool {
    runtime: Option<Arc<BunRuntime>>,
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl Default for BunAutoResolveTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunAutoResolveTool {
    pub fn new() -> Self {
        Self {
            runtime: BunRuntime::new().ok().map(Arc::new),
            working_dir: None,
            default_timeout: Duration::from_secs(60),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for BunAutoResolveTool {
    fn name(&self) -> &'static str {
        "bun_auto_resolve"
    }

    fn description(&self) -> &'static str {
        "Execute TypeScript/JavaScript code or scripts with Autonomous JIT Dependency Resolution. Automatically detects missing modules, installs them via 'bun add', and resumes execution."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "TypeScript/JavaScript code snippet to execute with auto-resolution."
                },
                "script_path": {
                    "type": "string",
                    "description": "Optional script file path to execute with auto-resolution."
                },
                "allow_native": {
                    "type": "boolean",
                    "description": "Explicitly permit native C/C++ compilation during auto-installation (default: false)."
                },
                "max_retries": {
                    "type": "integer",
                    "description": "Maximum number of missing packages to auto-install and retry (default: 3)."
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let code_opt = arguments.get("code").and_then(|v| v.as_str());
        let script_opt = arguments.get("script_path").and_then(|v| v.as_str());
        let allow_native = arguments.get("allow_native").and_then(|v| v.as_bool()).unwrap_or(false);
        let max_retries = arguments.get("max_retries").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

        if code_opt.is_none() && script_opt.is_none() {
            return Err(TagisanError::Execution("Either 'code' or 'script_path' must be provided".to_string()));
        }

        let runtime = if let Some(ref rt) = self.runtime {
            rt.clone()
        } else {
            BunRuntime::new().map(Arc::new)?
        };

        let mut installed_packages = Vec::new();
        let mut retries = 0;

        loop {
            let res = if let Some(code) = code_opt {
                runtime.eval(code, self.default_timeout, None, self.working_dir.clone()).await?
            } else {
                let script_path = PathBuf::from(script_opt.unwrap());
                runtime.run_script(&script_path, &[], self.default_timeout, None, self.working_dir.clone()).await?
            };

            if res.is_success() {
                let resp = json!({
                    "status": "success",
                    "stdout": res.stdout,
                    "stderr": res.stderr,
                    "exit_code": 0,
                    "auto_installed_packages": installed_packages
                });
                return Ok(serde_json::to_string(&resp).unwrap());
            }

            if retries >= max_retries {
                let resp = json!({
                    "status": "error",
                    "stdout": res.stdout,
                    "stderr": res.stderr,
                    "exit_code": res.exit_code,
                    "auto_installed_packages": installed_packages,
                    "error": "Max retries exceeded while resolving missing dependencies"
                });
                return Ok(serde_json::to_string(&resp).unwrap());
            }

            if let Some(pkg) = extract_missing_package(&res.stderr) {
                if installed_packages.contains(&pkg) {
                    return Ok(res.combined_output());
                }

                let install_res = runtime
                    .install(
                        &[pkg.clone()],
                        false,
                        allow_native,
                        self.default_timeout,
                        self.working_dir.clone(),
                    )
                    .await?;
                if !install_res.is_success() {
                    return Ok(format!(
                        "Failed to auto-install missing package '{}':\n{}",
                        pkg, install_res.stderr
                    ));
                }
                installed_packages.push(pkg);
                retries += 1;
            } else {
                return Ok(res.combined_output());
            }
        }
    }
}

// =========================================================================
// 7. BunHmrTool
// =========================================================================

struct ActiveHmrProcess {
    child: tokio::process::Child,
    script_path: PathBuf,
    started_at: Instant,
    logs: Arc<Mutex<Vec<String>>>,
}

/// Tool for running long-lived TypeScript scratchpads with state-preserving Hot Module Reloading (bun --hot)
#[derive(Clone)]
pub struct BunHmrTool {
    processes: Arc<Mutex<HashMap<String, ActiveHmrProcess>>>,
    pub working_dir: Option<PathBuf>,
}

impl Default for BunHmrTool {
    fn default() -> Self {
        Self::new()
    }
}

impl BunHmrTool {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            working_dir: None,
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for BunHmrTool {
    fn name(&self) -> &'static str {
        "bun_hmr"
    }

    fn description(&self) -> &'static str {
        "Manage state-preserving Hot Module Reloading (HMR) development scratchpads using 'bun --hot'. Supports live script file updates, continuous log streaming, and process lifecycle."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["start", "update_script", "status", "logs", "stop"],
                    "description": "The HMR action to perform."
                },
                "id": {
                    "type": "string",
                    "description": "Unique session identifier for the HMR process (default: filename)."
                },
                "script_path": {
                    "type": "string",
                    "description": "Path to the TypeScript/JavaScript script file to watch with --hot."
                },
                "content": {
                    "type": "string",
                    "description": "New file content to write during 'update_script' to trigger HMR reload."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let verdict = AgentShieldScanner::scan_tool_call(self.name(), &arguments);
        if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
            return Err(TagisanError::Execution(format!(
                "AgentShield blocked tool '{}': {} (Threat Level: {:?})",
                self.name(),
                reason,
                threat_level
            )));
        }

        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'action'".to_string()))?;

        match action {
            "start" => {
                let script_path_str = arguments
                    .get("script_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'script_path' for HMR start".to_string()))?;

                let id = arguments
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        Path::new(script_path_str)
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("hmr_default")
                            .to_string()
                    });

                let script_path = PathBuf::from(script_path_str);
                let bun_path = BunRuntime::find_bun().ok_or_else(|| {
                    TagisanError::Execution("Bun binary not found for HMR execution".to_string())
                })?;

                let mut cmd = tokio::process::Command::new(bun_path);
                cmd.args(["--hot", "run"]);
                cmd.arg(&script_path);

                if let Some(ref dir) = self.working_dir {
                    cmd.current_dir(dir);
                }
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());
                cmd.kill_on_drop(true);

                let mut child = cmd.spawn().map_err(|e| {
                    TagisanError::Execution(format!("Failed to spawn bun --hot process: {e}"))
                })?;

                let logs = Arc::new(Mutex::new(Vec::new()));
                let logs_clone = logs.clone();

                if let Some(stdout) = child.stdout.take() {
                    tokio::spawn(async move {
                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let mut l = logs_clone.lock().await;
                            l.push(line);
                            if l.len() > 500 {
                                l.remove(0);
                            }
                        }
                    });
                }

                let logs_err = logs.clone();
                if let Some(stderr) = child.stderr.take() {
                    tokio::spawn(async move {
                        use tokio::io::{AsyncBufReadExt, BufReader};
                        let mut reader = BufReader::new(stderr).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let mut l = logs_err.lock().await;
                            l.push(format!("[stderr] {line}"));
                            if l.len() > 500 {
                                l.remove(0);
                            }
                        }
                    });
                }

                {
                    let mut map = self.processes.lock().await;
                    map.insert(
                        id.clone(),
                        ActiveHmrProcess {
                            child,
                            script_path: script_path.clone(),
                            started_at: Instant::now(),
                            logs,
                        },
                    );
                }

                let resp = json!({
                    "status": "started",
                    "id": id,
                    "script_path": script_path.display().to_string(),
                    "mode": "hot-module-reloading"
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "update_script" => {
                let script_path_str = arguments
                    .get("script_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'script_path' for HMR update".to_string()))?;
                let content = arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'content' for HMR update".to_string()))?;

                tokio::fs::write(script_path_str, content)
                    .await
                    .map_err(|e| TagisanError::Execution(format!("Failed to update HMR script file: {e}")))?;

                let resp = json!({
                    "status": "updated",
                    "script_path": script_path_str,
                    "bytes_written": content.len()
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "logs" => {
                let id = arguments.get("id").and_then(|v| v.as_str());
                let map = self.processes.lock().await;

                let logs_vec = if let Some(id_str) = id {
                    if let Some(proc) = map.get(id_str) {
                        proc.logs.lock().await.clone()
                    } else {
                        return Err(TagisanError::Execution(format!("No HMR process found with id '{id_str}'")));
                    }
                } else if let Some(proc) = map.values().next() {
                    proc.logs.lock().await.clone()
                } else {
                    return Err(TagisanError::Execution("No active HMR processes running".to_string()));
                };

                let resp = json!({
                    "status": "ok",
                    "logs": logs_vec
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "status" => {
                let map = self.processes.lock().await;
                let mut list = Vec::new();
                for (id, proc) in map.iter() {
                    list.push(json!({
                        "id": id,
                        "script_path": proc.script_path.display().to_string(),
                        "uptime_secs": proc.started_at.elapsed().as_secs()
                    }));
                }
                let resp = json!({
                    "status": "ok",
                    "processes": list
                });
                Ok(serde_json::to_string(&resp).unwrap())
            }

            "stop" => {
                let id_opt = arguments.get("id").and_then(|v| v.as_str());
                let mut map = self.processes.lock().await;

                if let Some(id_str) = id_opt {
                    if let Some(mut proc) = map.remove(id_str) {
                        let _ = proc.child.kill().await;
                        Ok(format!("HMR process '{id_str}' stopped successfully."))
                    } else {
                        Ok(format!("No HMR process found with id '{id_str}'."))
                    }
                } else {
                    let count = map.len();
                    for (_, mut proc) in map.drain() {
                        let _ = proc.child.kill().await;
                    }
                    Ok(format!("Stopped all {count} active HMR processes."))
                }
            }

            _ => Err(TagisanError::Execution(format!("Unknown action '{action}' for bun_hmr"))),
        }
    }
}

