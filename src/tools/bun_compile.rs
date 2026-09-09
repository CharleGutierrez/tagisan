use super::ToolHandler;
use crate::bun::runtime::BunRuntime;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Tool for compiling TypeScript/JavaScript scripts into standalone zero-dependency native executables
#[derive(Debug, Default, Clone)]
pub struct BunCompileTool {
    pub working_dir: Option<PathBuf>,
    pub default_timeout: Duration,
}

impl BunCompileTool {
    pub fn new() -> Self {
        Self {
            working_dir: None,
            default_timeout: Duration::from_secs(120),
        }
    }

    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

#[async_trait]
impl ToolHandler for BunCompileTool {
    fn name(&self) -> &'static str {
        "bun_compile"
    }

    fn description(&self) -> &'static str {
        "Compile a TypeScript or JavaScript file into a standalone, zero-dependency native binary executable using 'bun build --compile'."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "entrypoint": {
                    "type": "string",
                    "description": "Path to the TypeScript (.ts) or JavaScript (.js) entrypoint file to compile."
                },
                "outfile": {
                    "type": "string",
                    "description": "Output path where the standalone binary will be created."
                },
                "minify": {
                    "type": "boolean",
                    "description": "Whether to minify code and optimize binary footprint (default: true)."
                },
                "bytecode": {
                    "type": "boolean",
                    "description": "Whether to use ahead-of-time bytecode compilation (default: false)."
                },
                "target": {
                    "type": "string",
                    "description": "Optional single target platform (e.g. 'bun-linux-x64', 'bun-darwin-arm64', 'bun-windows-x64')."
                },
                "targets": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional matrix of target platforms for cross-compilation."
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Optional compilation timeout in seconds (default: 120)."
                }
            },
            "required": ["entrypoint", "outfile"]
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

        let entrypoint_str = arguments
            .get("entrypoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'entrypoint'".to_string()))?;

        let outfile_str = arguments
            .get("outfile")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter: 'outfile'".to_string()))?;

        let minify = arguments.get("minify").and_then(|v| v.as_bool()).unwrap_or(true);
        let bytecode = arguments.get("bytecode").and_then(|v| v.as_bool()).unwrap_or(false);
        let timeout_secs = arguments
            .get("timeout_secs")
            .and_then(|v| v.as_u64())
            .map(Duration::from_secs)
            .unwrap_or(self.default_timeout);

        let single_target = arguments.get("target").and_then(|v| v.as_str());
        let target_list: Vec<String> = if let Some(targets_val) = arguments.get("targets").and_then(|v| v.as_array()) {
            targets_val.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
        } else if let Some(t) = single_target {
            vec![t.to_string()]
        } else {
            vec![]
        };

        let ep_path = Path::new(entrypoint_str);
        let resolved_ep = if ep_path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(ep_path)
            } else {
                ep_path.to_path_buf()
            }
        } else {
            ep_path.to_path_buf()
        };

        if !resolved_ep.exists() {
            return Err(TagisanError::Execution(format!(
                "Entrypoint file does not exist: {}",
                resolved_ep.display()
            )));
        }

        let out_path = Path::new(outfile_str);
        let resolved_out = if out_path.is_relative() {
            if let Some(ref base) = self.working_dir {
                base.join(out_path)
            } else {
                out_path.to_path_buf()
            }
        } else {
            out_path.to_path_buf()
        };

        if let Some(parent) = resolved_out.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
        }

        let bun_path = BunRuntime::find_bun().ok_or_else(|| {
            TagisanError::Execution("Bun binary not found for standalone compilation".to_string())
        })?;

        let mut artifacts = Vec::new();
        let is_multi_target = target_list.len() > 1;

        let targets_to_process = if target_list.is_empty() {
            vec![None]
        } else {
            target_list.iter().map(|t| Some(t.as_str())).collect()
        };

        let mut primary_duration = 0;
        let mut primary_size = 0;
        let mut primary_path = resolved_out.clone();

        for target_opt in targets_to_process {
            let target_outfile = if is_multi_target {
                let target_name = target_opt.unwrap_or("default");
                let extension = if target_name.contains("windows") { ".exe" } else { "" };
                let mut path_str = resolved_out.display().to_string();
                path_str.push('-');
                path_str.push_str(target_name);
                path_str.push_str(extension);
                PathBuf::from(path_str)
            } else {
                resolved_out.clone()
            };

            let mut cmd = Command::new(&bun_path);
            cmd.arg("build");
            cmd.arg("--compile");
            if minify {
                cmd.arg("--minify");
            }
            if bytecode {
                cmd.arg("--bytecode");
            }
            if let Some(tgt) = target_opt {
                cmd.arg(format!("--target={tgt}"));
            }
            cmd.arg(&resolved_ep);
            cmd.arg(format!("--outfile={}", target_outfile.display()));

            if let Some(ref dir) = self.working_dir {
                cmd.current_dir(dir);
            }
            cmd.kill_on_drop(true);

            let start_time = std::time::Instant::now();
            let output_fut = cmd.output();
            let output = timeout(timeout_secs, output_fut)
                .await
                .map_err(|_| {
                    TagisanError::Execution(format!(
                        "Compilation timed out after {:.1}s",
                        timeout_secs.as_secs_f32()
                    ))
                })?
                .map_err(|e| TagisanError::Execution(format!("Failed to execute 'bun build --compile': {e}")))?;

            let duration_ms = start_time.elapsed().as_millis();

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(TagisanError::Execution(format!(
                    "Compilation failed (exit code {}):\n{stderr}",
                    output.status.code().unwrap_or(-1)
                )));
            }

            if !target_outfile.exists() {
                return Err(TagisanError::Execution(format!(
                    "Compilation completed but output binary not found at {}",
                    target_outfile.display()
                )));
            }

            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&target_outfile) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&target_outfile, perms);
                }
            }

            let file_size = std::fs::metadata(&target_outfile)
                .map(|m| m.len())
                .unwrap_or(0);
            let size_mb = file_size as f64 / (1024.0 * 1024.0);

            if artifacts.is_empty() {
                primary_duration = duration_ms;
                primary_size = file_size;
                primary_path = target_outfile.clone();
            }

            artifacts.push(json!({
                "target": target_opt.unwrap_or("host-default"),
                "binary_path": target_outfile.display().to_string(),
                "file_size": file_size,
                "file_size_mb": size_mb,
                "duration_ms": duration_ms
            }));
        }

        let primary_mb = primary_size as f64 / (1024.0 * 1024.0);

        let compile_resp = json!({
            "status": "success",
            "binary_path": primary_path.display().to_string(),
            "duration_ms": primary_duration,
            "file_size": primary_size,
            "file_size_mb": primary_mb,
            "artifacts": artifacts,
            "message": format!("Standalone native binary compilation succeeded ({} artifacts generated)", artifacts.len())
        });
        Ok(serde_json::to_string(&compile_resp).unwrap())
    }
}
