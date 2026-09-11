//! Tagisan Universal Plugin Architecture & GitHub Ecosystem (RFC-002)
//!
//! Multi-tier execution engines (WASM, Bun/TS, MCP, Native), capability sandboxing,
//! cognitive lifecycle hooks, repository installation, and CLI tooling.

pub mod adapter;
pub mod cli_handler;
pub mod hooks;
pub mod manifest;
pub mod manager;
pub mod runtime;
pub mod security;

pub use adapter::PluginToolWrapper;
pub use cli_handler::{handle_plugin_command, PluginAction};
pub use hooks::{
    DebateJudgeVerdict, DebateRoundInfo, PipelineStageContext, PipelineStageOutput,
    PluginDebateJudge, PluginHookRegistry, PluginPipelineStage, PluginShieldInterceptor,
    PluginSkillPack, PluginTelemetryExporter, PluginToolProvider,
};
pub use manager::{CuratedCatalogItem, InstallScope, LoadedPlugin, PluginManager};
pub use manifest::{
    PluginCapabilities, PluginHooksConfig, PluginManifest, PluginMcpConfig, PluginMetadata,
    PluginNativeConfig, PluginPermissions, PluginRuntimeType, PluginSkillsConfig,
    PluginToolDefinition, PluginToolsConfig,
};
pub use runtime::{
    BunPluginEngine, McpPluginEngine, NativePluginEngine, PluginEngine, PluginExecutionContext,
    PluginToolDescriptor, WasmPluginEngine,
};
pub use security::PluginSecurityGovernor;
