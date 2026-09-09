use crate::error::{Result, TagisanError};
use crate::mcp::client::McpClient;
use crate::mcp::protocol::McpToolDefinition;
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

/// Wraps an external MCP tool into Tagisan's native `ToolHandler` trait
#[derive(Clone)]
pub struct McpToolWrapper {
    client: Arc<McpClient>,
    registered_name: String,
    raw_tool_name: String,
    description: String,
    input_schema: Value,
}

impl McpToolWrapper {
    /// Create a new tool wrapper from an MCP tool definition
    pub fn new(
        client: Arc<McpClient>,
        def: McpToolDefinition,
        prefix_server_name: bool,
    ) -> Self {
        let registered_name = if prefix_server_name {
            format!("{}__{}", client.server_name, def.name)
        } else {
            def.name.clone()
        };

        let description = def.description.unwrap_or_else(|| {
            format!(
                "Tool '{}' provided by MCP server '{}'",
                def.name, client.server_name
            )
        });

        Self {
            client,
            registered_name,
            raw_tool_name: def.name,
            description,
            input_schema: def.input_schema,
        }
    }

    /// Access the server name providing this tool
    pub fn server_name(&self) -> &str {
        &self.client.server_name
    }

    /// Access raw un-prefixed tool name
    pub fn raw_tool_name(&self) -> &str {
        &self.raw_tool_name
    }
}

#[async_trait]
impl ToolHandler for McpToolWrapper {
    fn name(&self) -> &str {
        &self.registered_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        self.input_schema.clone()
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let call_res = self
            .client
            .call_tool(&self.raw_tool_name, arguments)
            .await?;

        let output_text = call_res.extract_text();

        if call_res.is_error {
            Err(TagisanError::Execution(format!(
                "MCP tool '{}' returned an error: {}",
                self.registered_name, output_text
            )))
        } else {
            Ok(output_text)
        }
    }
}
