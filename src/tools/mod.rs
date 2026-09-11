pub mod builtin;
pub mod bun;
pub mod bun_compile;
pub mod bun_serve;
pub mod wasm;

use crate::error::Result;
use crate::types::{ContentBlock, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub use builtin::{
    CalculatorTool, ReadFileTool, RunCommandTool, SaveMemoryTool, SearchMemoryTool,
    SearchSkillsTool, ViewImageTool, WriteFileTool,
};
pub use bun::{
    extract_missing_package, BunAutoResolveTool, BunBuildTool, BunEvalTool, BunHmrTool,
    BunInstallTool, BunRunTool, BunTestTool,
};
pub use bun_compile::BunCompileTool;
pub use bun_serve::{BunServeTool, BunStreamBusTool};
pub use wasm::{load_wasm_tools, WasmTool};
pub use crate::vella::{
    VellaDefenseDrillTool, VellaDigitalTwinTool, VellaEventBridgeTool, VellaFheShieldTool,
    VellaMedicineTool, VellaRoboticsTool, VellaScadaTool, VellaScaffolderTool,
    VellaSpaceCopilotTool, VellaTradingTool, VellaVectorSyncTool, VellaWeb3GuardianTool,
};

/// Trait implemented by all tools executable by autonomous agents
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Unique identifier / function name of the tool (e.g. "read_file", "calculator")
    fn name(&self) -> &str;

    /// Human-readable and LLM-targeted description of what the tool does
    fn description(&self) -> &str;

    /// JSON schema describing expected input parameters
    fn parameters_schema(&self) -> Value;

    /// Asynchronous execution of the tool with JSON arguments
    async fn execute(&self, arguments: Value) -> Result<String>;
}

/// Registry managing available tools for autonomous execution
#[derive(Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with standard built-in tools
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_tool(builtin::ReadFileTool::new());
        registry.register_tool(builtin::WriteFileTool::new());
        registry.register_tool(builtin::RunCommandTool::default());
        registry.register_tool(builtin::CalculatorTool::new());
        registry.register_tool(builtin::ViewImageTool::new());
        registry.register_tool(bun::BunEvalTool::new());
        registry.register_tool(bun::BunRunTool::new());
        registry.register_tool(bun::BunTestTool::new());
        registry.register_tool(bun::BunInstallTool::new());
        registry.register_tool(bun::BunBuildTool::new());
        registry.register_tool(bun::BunAutoResolveTool::new());
        registry.register_tool(bun::BunHmrTool::new());
        registry.register_tool(bun_serve::BunServeTool::new());
        registry.register_tool(bun_serve::BunStreamBusTool::new());
        registry.register_tool(bun_compile::BunCompileTool::new());
        registry.register_tool(builtin::SearchSkillsTool::with_default());
        // Vella Phase 1 Domain Tools
        registry.register_tool(crate::vella::VellaTradingTool::default());
        registry.register_tool(crate::vella::VellaScadaTool::default());
        registry.register_tool(crate::vella::VellaRoboticsTool::default());
        registry.register_tool(crate::vella::VellaMedicineTool::default());
        registry.register_tool(crate::vella::VellaEventBridgeTool::default());
        // Vella Phase 2 Deep-Systems Superpowers
        registry.register_tool(crate::vella::VellaVectorSyncTool::default());
        registry.register_tool(crate::vella::VellaDigitalTwinTool::default());
        registry.register_tool(crate::vella::VellaWeb3GuardianTool::default());
        registry.register_tool(crate::vella::VellaFheShieldTool::default());
        registry.register_tool(crate::vella::VellaSpaceCopilotTool::default());
        registry.register_tool(crate::vella::VellaScaffolderTool::default());
        registry.register_tool(crate::vella::VellaDefenseDrillTool::default());
        registry
    }

    /// Create a registry pre-populated with built-in tools bound to a specific working directory / sandbox
    pub fn with_builtins_in_dir(dir: impl Into<std::path::PathBuf>) -> Self {
        let dir = dir.into();
        let mut registry = Self::new();
        registry.register_tool(builtin::ReadFileTool::new().with_working_dir(dir.clone()));
        registry.register_tool(builtin::WriteFileTool::new().with_working_dir(dir.clone()));
        registry.register_tool(builtin::RunCommandTool::default().with_working_dir(dir.clone()));
        registry.register_tool(builtin::CalculatorTool::new());
        registry.register_tool(builtin::ViewImageTool::new());
        registry.register_tool(builtin::SearchSkillsTool::with_default());
        registry.register_tool(bun::BunEvalTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunRunTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunTestTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunInstallTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunBuildTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunAutoResolveTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun::BunHmrTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun_serve::BunServeTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun_serve::BunStreamBusTool::new().with_working_dir(dir.clone()));
        registry.register_tool(bun_compile::BunCompileTool::new().with_working_dir(dir));
        // Vella Phase 1 Domain Tools
        registry.register_tool(crate::vella::VellaTradingTool::default());
        registry.register_tool(crate::vella::VellaScadaTool::default());
        registry.register_tool(crate::vella::VellaRoboticsTool::default());
        registry.register_tool(crate::vella::VellaMedicineTool::default());
        registry.register_tool(crate::vella::VellaEventBridgeTool::default());
        // Vella Phase 2 Deep-Systems Superpowers
        registry.register_tool(crate::vella::VellaVectorSyncTool::default());
        registry.register_tool(crate::vella::VellaDigitalTwinTool::default());
        registry.register_tool(crate::vella::VellaWeb3GuardianTool::default());
        registry.register_tool(crate::vella::VellaFheShieldTool::default());
        registry.register_tool(crate::vella::VellaSpaceCopilotTool::default());
        registry.register_tool(crate::vella::VellaScaffolderTool::default());
        registry.register_tool(crate::vella::VellaDefenseDrillTool::default());
        registry
    }

    /// Register a tool wrapped in an Arc
    pub fn register(&mut self, tool: Arc<dyn ToolHandler>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Register an un-Arced tool instance
    pub fn register_tool(&mut self, tool: impl ToolHandler + 'static) {
        self.register(Arc::new(tool));
    }

    /// Retrieve a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists in the registry
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Check if a tool exists in the registry (alias for contains)
    pub fn has_tool(&self, name: &str) -> bool {
        self.contains(name)
    }

    /// Number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// List all registered tool names
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }

    /// Generate universal ToolDefinition schemas for LLM completion requests
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> = self
            .tools
            .values()
            .map(|t| ToolDefinition::new(t.name(), t.description(), t.parameters_schema()))
            .collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    /// Execute a tool call and produce a `ContentBlock::ToolResult`
    pub async fn execute_call(&self, tool_call_id: &str, name: &str, arguments: &Value) -> ContentBlock {
        match self.tools.get(name) {
            Some(tool) => match tool.execute(arguments.clone()).await {
                Ok(content) => ContentBlock::tool_result(tool_call_id, content, false),
                Err(err) => {
                    ContentBlock::tool_result(tool_call_id, format!("Tool execution error: {err}"), true)
                }
            },
            None => ContentBlock::tool_result(
                tool_call_id,
                format!("Error: Tool '{name}' is not registered in ToolRegistry"),
                true,
            ),
        }
    }
}
