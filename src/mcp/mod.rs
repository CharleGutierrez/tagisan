pub mod adapter;
pub mod catalog;
pub mod client;
pub mod config;
pub mod manager;
pub mod protocol;
pub mod server;
pub mod transport;

pub use adapter::McpToolWrapper;
pub use catalog::{McpCatalog, McpCatalogEntry};
pub use client::{McpClient, DEFAULT_MCP_PROTOCOL_VERSION};
pub use config::{expand_env_vars, McpConfig, McpServerConfig};
pub use manager::McpManager;
pub use protocol::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, McpContentBlock,
    McpInitializeResult, McpServerInfo, McpToolCallResult, McpToolDefinition,
};
pub use server::McpServer;
pub use transport::StdioTransport;
