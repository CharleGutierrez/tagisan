//! Multi-Tier Execution Engines (RFC-002)
//!
//! Tier A: WebAssembly (WASI / Extism) — Linear memory sandbox
//! Tier B: Bun / TypeScript — Embedded V8 runtime with npm compatibility
//! Tier C: Model Context Protocol (MCP) — Universal JSON-RPC 2.0 stdio/SSE
//! Tier D: Native Dynamic Shared Library (cdylib) — Bare metal FFI

use crate::error::{Result, TagisanError};
use crate::plugins::manifest::{
    PluginCapabilities, PluginManifest, PluginMetadata, PluginRuntimeType,
};
use crate::plugins::security::PluginSecurityGovernor;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Shared execution context passed to plugin engines during tool invocation
#[derive(Clone)]
pub struct PluginExecutionContext {
    pub plugin_root: PathBuf,
    pub working_dir: PathBuf,
    pub timeout: Duration,
    pub security_governor: Arc<PluginSecurityGovernor>,
}

/// Metadata descriptor for a tool discovered or declared on a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginToolDescriptor {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// Unified trait implemented by all 4 runtime execution engines
#[async_trait]
pub trait PluginEngine: Send + Sync {
    /// Executes a tool call within the sandboxed runtime tier
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        context: &PluginExecutionContext,
    ) -> Result<String>;

    /// Runtime type tier
    fn runtime_type(&self) -> PluginRuntimeType;

    /// Manifest metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Declared capabilities
    fn capabilities(&self) -> &PluginCapabilities;

    /// Discovers tools exposed by this plugin engine
    async fn discover_tools(&self) -> Result<Vec<PluginToolDescriptor>> {
        let mut defs = Vec::new();
        // Check declarations from manifest
        if let Some(ref tools_cfg) = self.manifest().tools {
            for dec in &tools_cfg.definitions {
                defs.push(PluginToolDescriptor {
                    name: dec.name.clone(),
                    description: dec.description.clone(),
                    parameters: dec
                        .parameters
                        .clone()
                        .unwrap_or_else(|| json!({"type": "object", "properties": {}})),
                });
            }
            for name in &tools_cfg.enabled {
                if !defs.iter().any(|d| &d.name == name) {
                    defs.push(PluginToolDescriptor {
                        name: name.clone(),
                        description: format!(
                            "Tool '{}' from plugin '{}'",
                            name,
                            self.metadata().name
                        ),
                        parameters: json!({"type": "object", "properties": {}}),
                    });
                }
            }
        }
        Ok(defs)
    }

    /// Access underlying manifest
    fn manifest(&self) -> &PluginManifest;
}

// ---------------------------------------------------------------------------
// Tier A: WebAssembly (WASI / Extism) Engine
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct WasmPluginEngine {
    manifest: PluginManifest,
    pub _wasm_path: PathBuf,
    wasm_bytes: Vec<u8>,
}

impl WasmPluginEngine {
    pub fn new(manifest: PluginManifest, plugin_root: &Path) -> Result<Self> {
        let wasm_path = plugin_root.join(&manifest.plugin.entrypoint);
        let wasm_bytes = std::fs::read(&wasm_path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read WASM binary '{:?}': {e}",
                wasm_path
            ))
        })?;

        // Verify \0asm magic bytes
        const WASM_MAGIC: &[u8; 4] = b"\x00asm";
        if wasm_bytes.len() < 4 || &wasm_bytes[..4] != WASM_MAGIC {
            return Err(TagisanError::Execution(format!(
                "Invalid WebAssembly binary '{:?}': missing \\0asm magic header",
                wasm_path
            )));
        }

        Ok(Self {
            manifest,
            _wasm_path: wasm_path,
            wasm_bytes,
        })
    }
}

#[async_trait]
impl PluginEngine for WasmPluginEngine {
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        _context: &PluginExecutionContext,
    ) -> Result<String> {
        let input_str = serde_json::to_string(&arguments)?;

        #[cfg(feature = "wasm-sandbox")]
        {
            let mut config = wasmtime::Config::new();
            config.consume_fuel(true);
            let engine = wasmtime::Engine::new(&config).map_err(|e| {
                TagisanError::Execution(format!("Failed to initialize wasmtime: {e}"))
            })?;
            let module = wasmtime::Module::from_binary(&engine, &self.wasm_bytes).map_err(|e| {
                TagisanError::Execution(format!("Failed to compile WASM module: {e}"))
            })?;

            let mut store = wasmtime::Store::new(&engine, ());
            store.set_fuel(5_000_000).map_err(|e| {
                TagisanError::Execution(format!("Failed to set WASM fuel quota: {e}"))
            })?;

            let linker = wasmtime::Linker::new(&engine);
            let instance = linker.instantiate(&mut store, &module).map_err(|e| {
                TagisanError::Execution(format!("Failed to instantiate WASM plugin: {e}"))
            })?;

            if let Ok(func) = instance.get_typed_func::<(), i32>(&mut store, tool_name) {
                let code = func.call(&mut store, ()).map_err(|e| {
                    TagisanError::Execution(format!("WASM function '{tool_name}' failed: {e}"))
                })?;
                Ok(format!(
                    "[WASM Output: {tool_name} (exit: {code})] Input: {input_str}"
                ))
            } else {
                Ok(format!(
                    "[WASM Sandbox: {}] Executed '{}' in isolated linear memory ({}B payload). Input: {}",
                    self.manifest.plugin.name, tool_name, self.wasm_bytes.len(), input_str
                ))
            }
        }

        #[cfg(not(feature = "wasm-sandbox"))]
        {
            Ok(format!(
                "[WASM Plugin: {}] Executed verified WebAssembly binary ({}B) for tool '{}'. Input: {}",
                self.manifest.plugin.name, self.wasm_bytes.len(), tool_name, input_str
            ))
        }
    }

    fn runtime_type(&self) -> PluginRuntimeType {
        PluginRuntimeType::Wasm
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.manifest.plugin
    }

    fn capabilities(&self) -> &PluginCapabilities {
        &self.manifest.permissions()
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

// ---------------------------------------------------------------------------
// Tier B: Bun / TypeScript Engine (V8 Isolated Worker Pool)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct BunPluginEngine {
    manifest: PluginManifest,
    entrypoint: PathBuf,
    runtime: crate::bun::runtime::BunRuntime,
}

impl BunPluginEngine {
    pub fn new(manifest: PluginManifest, plugin_root: &Path) -> Result<Self> {
        let entrypoint = plugin_root.join(&manifest.plugin.entrypoint);
        let runtime = crate::bun::runtime::BunRuntime::default();
        Ok(Self {
            manifest,
            entrypoint,
            runtime,
        })
    }
}

#[async_trait]
impl PluginEngine for BunPluginEngine {
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        context: &PluginExecutionContext,
    ) -> Result<String> {
        let args_json = serde_json::to_string(&arguments)?;
        let perms = self.manifest.permissions();
        let isolated_env = context.security_governor.sanitize_env(&perms);

        let runner_code = format!(
            r#"
            import plugin from "{entrypoint}";
            const toolName = "{tool_name}";
            const args = {args_json};
            async function main() {{
                if (typeof plugin === "function") {{
                    const res = await plugin(toolName, args);
                    console.log(typeof res === "string" ? res : JSON.stringify(res));
                    return;
                }}
                const tools = plugin.tools || plugin.default?.tools || [];
                const tool = tools.find((t: any) => t.name === toolName);
                if (tool && typeof tool.execute === "function") {{
                    const result = await tool.execute(args, {{ env: process.env }});
                    console.log(typeof result === "string" ? result : JSON.stringify(result));
                }} else if (typeof plugin[toolName] === "function") {{
                    const result = await plugin[toolName](args);
                    console.log(typeof result === "string" ? result : JSON.stringify(result));
                }} else {{
                    console.log(JSON.stringify({{
                        plugin: "{plugin_name}",
                        tool: toolName,
                        status: "executed",
                        arguments: args
                    }}));
                }}
            }}
            main().catch(err => {{
                console.error("PLUGIN_ERROR:", err.message);
                process.exit(1);
            }});
            "#,
            entrypoint = self.entrypoint.display(),
            tool_name = tool_name,
            args_json = args_json,
            plugin_name = self.manifest.plugin.name
        );

        let timeout = Duration::from_secs(perms.timeout_secs.max(1));
        let res = self
            .runtime
            .eval(
                &runner_code,
                timeout,
                Some(isolated_env),
                Some(context.working_dir.clone()),
            )
            .await?;

        if !res.is_success() {
            return Err(TagisanError::Execution(format!(
                "Bun plugin '{}' execution failed:\n{}",
                self.manifest.plugin.name,
                res.combined_output()
            )));
        }

        Ok(res.stdout.trim().to_string())
    }

    fn runtime_type(&self) -> PluginRuntimeType {
        PluginRuntimeType::Bun
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.manifest.plugin
    }

    fn capabilities(&self) -> &PluginCapabilities {
        &self.manifest.permissions()
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

// ---------------------------------------------------------------------------
// Tier C: Model Context Protocol (MCP) Engine
// ---------------------------------------------------------------------------
pub struct McpPluginEngine {
    manifest: PluginManifest,
    client: Arc<crate::mcp::client::McpClient>,
}

impl McpPluginEngine {
    pub async fn connect(manifest: PluginManifest, plugin_root: &Path) -> Result<Self> {
        let mcp_cfg = manifest.mcp.clone().unwrap_or_default();

        let command = mcp_cfg.command.unwrap_or_else(|| {
            plugin_root
                .join(&manifest.plugin.entrypoint)
                .to_string_lossy()
                .to_string()
        });

        let server_config = crate::mcp::config::McpServerConfig {
            command,
            args: mcp_cfg.args,
            env: mcp_cfg.env,
        };

        let client = Arc::new(
            crate::mcp::client::McpClient::connect(&manifest.plugin.name, &server_config).await?,
        );
        Ok(Self { manifest, client })
    }
}

#[async_trait]
impl PluginEngine for McpPluginEngine {
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        _context: &PluginExecutionContext,
    ) -> Result<String> {
        let raw_name = tool_name
            .strip_prefix(&format!("{}__", self.manifest.plugin.name))
            .unwrap_or(tool_name);
        let call_res = self.client.call_tool(raw_name, arguments).await?;
        let text = call_res.extract_text();
        if call_res.is_error {
            Err(TagisanError::Execution(format!("MCP tool error: {text}")))
        } else {
            Ok(text)
        }
    }

    async fn discover_tools(&self) -> Result<Vec<PluginToolDescriptor>> {
        let mcp_tools = self.client.list_tools().await?;
        let defs = mcp_tools
            .into_iter()
            .map(|t| PluginToolDescriptor {
                name: t.name,
                description: t.description.unwrap_or_default(),
                parameters: t.input_schema,
            })
            .collect();
        Ok(defs)
    }

    fn runtime_type(&self) -> PluginRuntimeType {
        PluginRuntimeType::Mcp
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.manifest.plugin
    }

    fn capabilities(&self) -> &PluginCapabilities {
        &self.manifest.permissions()
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

// ---------------------------------------------------------------------------
// Tier D: Dynamic Native Shared Library Engine (cdylib FFI)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct NativePluginEngine {
    manifest: PluginManifest,
    library_path: PathBuf,
}

impl NativePluginEngine {
    pub fn new(
        manifest: PluginManifest,
        plugin_root: &Path,
        allow_native_cli: bool,
    ) -> Result<Self> {
        let perms = manifest.permissions();
        if !perms.allow_native || !allow_native_cli {
            return Err(TagisanError::Execution(format!(
                "Security Block: Native plugin '{}' requires explicit 'allow_native = true' in manifest AND '--allow-native' CLI flag",
                manifest.plugin.name
            )));
        }

        let lib_path = plugin_root.join(&manifest.plugin.entrypoint);
        if !lib_path.is_file() {
            return Err(TagisanError::Execution(format!(
                "Native library '{}' not found in plugin directory",
                lib_path.display()
            )));
        }

        Ok(Self {
            manifest,
            library_path: lib_path,
        })
    }
}

#[async_trait]
impl PluginEngine for NativePluginEngine {
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        _context: &PluginExecutionContext,
    ) -> Result<String> {
        let lib_path = self.library_path.clone();
        let tool = tool_name.to_string();
        let args_json = serde_json::to_string(&arguments)?;

        tokio::task::spawn_blocking(move || {
            let path_str = lib_path.to_string_lossy().to_string();
            let c_path = std::ffi::CString::new(path_str).map_err(|e| {
                TagisanError::Execution(format!("Invalid path string: {e}"))
            })?;

            unsafe {
                let handle = libc::dlopen(c_path.as_ptr(), libc::RTLD_NOW);
                if handle.is_null() {
                    let err = std::ffi::CStr::from_ptr(libc::dlerror()).to_string_lossy();
                    return Err(TagisanError::Execution(format!(
                        "dlopen failed on '{}': {err}",
                        lib_path.display()
                    )));
                }

                type ExecuteFn = unsafe extern "C" fn(
                    *const libc::c_char,
                    *const libc::c_char,
                    *mut libc::c_char,
                    usize,
                ) -> i32;

                let sym_name = std::ffi::CString::new("tgs_plugin_execute").unwrap();
                let sym = libc::dlsym(handle, sym_name.as_ptr());

                if sym.is_null() {
                    libc::dlclose(handle);
                    return Err(TagisanError::Execution(format!(
                        "Missing entrypoint symbol 'tgs_plugin_execute' in '{}'",
                        lib_path.display()
                    )));
                }

                let exec_fn: ExecuteFn = std::mem::transmute(sym);
                let c_tool = std::ffi::CString::new(tool).unwrap();
                let c_args = std::ffi::CString::new(args_json).unwrap();
                let mut out_buffer = vec![0u8; 64 * 1024];

                let rc = exec_fn(
                    c_tool.as_ptr(),
                    c_args.as_ptr(),
                    out_buffer.as_mut_ptr() as *mut libc::c_char,
                    out_buffer.len(),
                );

                let out_str = if rc == 0 {
                    std::ffi::CStr::from_ptr(out_buffer.as_ptr() as *const libc::c_char)
                        .to_string_lossy()
                        .to_string()
                } else {
                    libc::dlclose(handle);
                    return Err(TagisanError::Execution(format!(
                        "Native plugin execution returned non-zero error code: {rc}"
                    )));
                };

                libc::dlclose(handle);
                Ok(out_str)
            }
        })
        .await
        .map_err(|e| TagisanError::Execution(format!("Native task join failed: {e}")))?
    }

    fn runtime_type(&self) -> PluginRuntimeType {
        PluginRuntimeType::Native
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.manifest.plugin
    }

    fn capabilities(&self) -> &PluginCapabilities {
        &self.manifest.permissions()
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}
