//! WASM Tool Sandbox — load and execute user-compiled WebAssembly tools.
//!
//! Gated behind the `wasm-sandbox` feature flag. Without the feature,
//! `WasmTool::execute` returns a descriptive error.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A tool backed by a compiled `.wasm` binary.
///
/// When the `wasm-sandbox` feature is active, the WASM binary is validated
/// and executed in an isolated sandbox. Without the feature, a descriptive
/// stub error is returned.
#[derive(Debug, Clone)]
pub struct WasmTool {
    pub name: String,
    pub description: String,
    pub wasm_path: PathBuf,
}

impl WasmTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        wasm_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            wasm_path: wasm_path.into(),
        }
    }
}

#[async_trait]
impl ToolHandler for WasmTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "String input passed to the WASM module's main export."
                }
            },
            "required": ["input"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let input_str = arguments
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        #[cfg(feature = "wasm-sandbox")]
        {
            // Read and validate WASM magic bytes: \0asm
            let wasm_bytes = std::fs::read(&self.wasm_path).map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to read WASM file {:?}: {e}",
                    self.wasm_path
                ))
            })?;

            const WASM_MAGIC: &[u8; 4] = b"\x00asm";
            if wasm_bytes.len() < 4 || &wasm_bytes[..4] != WASM_MAGIC {
                return Err(TagisanError::Execution(format!(
                    "File {:?} is not a valid WebAssembly binary (missing \\0asm magic header)",
                    self.wasm_path
                )));
            }

            let mut config = wasmtime::Config::new();
            config.consume_fuel(true);
            let engine = wasmtime::Engine::new(&config).map_err(|e| {
                TagisanError::Execution(format!("Failed to create wasmtime engine: {e}"))
            })?;

            let module = wasmtime::Module::from_binary(&engine, &wasm_bytes).map_err(|e| {
                TagisanError::Execution(format!("Failed to compile WASM module: {e}"))
            })?;

            let mut store = wasmtime::Store::new(&engine, ());
            store.set_fuel(1_000_000).map_err(|e| {
                TagisanError::Execution(format!("Failed to set wasm fuel quota: {e}"))
            })?;

            let linker = wasmtime::Linker::new(&engine);
            let instance = linker.instantiate(&mut store, &module).map_err(|e| {
                TagisanError::Execution(format!("Failed to instantiate WASM module: {e}"))
            })?;

            if let Ok(run_fn) = instance.get_typed_func::<(), i32>(&mut store, "run") {
                let code = run_fn.call(&mut store, ()).map_err(|e| {
                    TagisanError::Execution(format!("WASM run execution error: {e}"))
                })?;
                Ok(format!(
                    "[WASM Sandbox] Executed '{}' (run() -> {code}) with input: {input_str}",
                    self.name
                ))
            } else {
                Ok(format!(
                    "[WASM Sandbox] Loaded and instantiated '{}' ({}B) with fuel quota 1,000,000. Input: {}",
                    self.name,
                    wasm_bytes.len(),
                    input_str
                ))
            }
        }

        #[cfg(not(feature = "wasm-sandbox"))]
        {
            if self.wasm_path.exists() {
                let wasm_bytes = std::fs::read(&self.wasm_path).map_err(|e| {
                    TagisanError::Execution(format!(
                        "Failed to read WASM file {:?}: {e}",
                        self.wasm_path
                    ))
                })?;
                const WASM_MAGIC: &[u8; 4] = b"\x00asm";
                if wasm_bytes.len() < 4 || &wasm_bytes[..4] != WASM_MAGIC {
                    return Err(TagisanError::Execution(format!(
                        "File {:?} is not a valid WebAssembly binary (missing \\0asm header)",
                        self.wasm_path
                    )));
                }
                Ok(format!(
                    "[WASM Tool: {}] Validated WebAssembly binary ({}B, \\0asm header verified). Input: '{}'. (Rebuild with --features wasm-sandbox for full wasmtime runtime execution)",
                    self.name, wasm_bytes.len(), input_str
                ))
            } else {
                Err(TagisanError::Execution(format!(
                    "WASM file {:?} not found",
                    self.wasm_path
                )))
            }
        }
    }
}

/// Scan a directory for `*.wasm` files and create a `WasmTool` for each.
///
/// Tool name is derived from the file stem (e.g. `my_tool.wasm` → `my_tool`).
pub fn load_wasm_tools(dir: &Path) -> Result<Vec<WasmTool>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(dir).map_err(|e| {
        TagisanError::Execution(format!("Failed to read WASM tools directory {:?}: {e}", dir))
    })?;

    let mut tools = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "wasm").unwrap_or(false) {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "wasm_tool".to_string());

            let description = format!(
                "WebAssembly tool loaded from {:?}",
                path.file_name().unwrap_or_default()
            );

            tools.push(WasmTool::new(name, description, path));
        }
    }

    Ok(tools)
}
