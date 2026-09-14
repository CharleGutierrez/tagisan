//// Type-Safe Model Context Protocol (MCP) Decoders & Tool Schemas.

import gleam/result

pub type McpRequest {
  CallTool(tool: String, arguments: String, id: Int)
  ListTools(id: Int)
  Ping(id: Int)
}

pub type ToolCall {
  ToolCall(name: String, parameters: String)
}

pub type ToolResult {
  ToolSuccess(output: String)
  ToolError(code: Int, message: String)
}

pub type ToolDefinition {
  ToolDefinition(
    name: String,
    description: String,
    schema_json: String,
  )
}

/// Decode incoming MCP JSON-RPC payload string into typed ToolCall
pub fn decode_tool_call(tool_name: String, args_json: String) -> Result(ToolCall, String) {
  case tool_name {
    "" -> Error("Tool name cannot be empty")
    name -> Ok(ToolCall(name: name, parameters: args_json))
  }
}

/// Encode tool execution result to standardized string representation
pub fn encode_tool_result(res: ToolResult) -> String {
  case res {
    ToolSuccess(output) -> "{\"status\": \"success\", \"output\": \"" <> output <> "\"}"
    ToolError(code, msg) -> "{\"status\": \"error\", \"code\": " <> "500" <> ", \"message\": \"" <> msg <> "\"}"
  }
}

/// Standard Tagisan system tools exposed via MCP
pub fn default_tool_catalog() -> List(ToolDefinition) {
  [
    ToolDefinition(
      name: "run_command",
      description: "Execute audited shell commands under AgentShield",
      schema_json: "{\"type\": \"object\", \"properties\": {\"command\": {\"type\": \"string\"}}}",
    ),
    ToolDefinition(
      name: "read_file",
      description: "Read text contents of a file securely",
      schema_json: "{\"type\": \"object\", \"properties\": {\"path\": {\"type\": \"string\"}}}",
    ),
    ToolDefinition(
      name: "write_file",
      description: "Write atomic file contents",
      schema_json: "{\"type\": \"object\", \"properties\": {\"path\": {\"type\": \"string\"}, \"content\": {\"type\": \"string\"}}}",
    ),
  ]
}
