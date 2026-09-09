pub mod builtin;

use crate::error::Result;
use crate::types::{ContentBlock, ToolDefinition};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub use builtin::{
    CalculatorTool, ReadFileTool, RunCommandTool, SaveMemoryTool, SearchMemoryTool, ViewImageTool,
    WriteFileTool,
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
        registry
    }

    /// Create a registry pre-populated with built-in tools bound to a specific working directory / sandbox
    pub fn with_builtins_in_dir(dir: impl Into<std::path::PathBuf>) -> Self {
        let dir = dir.into();
        let mut registry = Self::new();
        registry.register_tool(builtin::ReadFileTool::new().with_working_dir(dir.clone()));
        registry.register_tool(builtin::WriteFileTool::new().with_working_dir(dir.clone()));
        registry.register_tool(builtin::RunCommandTool::default().with_working_dir(dir));
        registry.register_tool(builtin::CalculatorTool::new());
        registry.register_tool(builtin::ViewImageTool::new());
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
        self.tools.keys().cloned().collect()
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
