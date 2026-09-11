//! ToolHandler Adapter for Plugin Tools
//!
//! Wraps any plugin engine tool into Tagisan's universal `ToolHandler` interface,
//! automatically applying capability checks and AgentShield verification.

use crate::error::Result;
use crate::plugins::manifest::PluginManifest;
use crate::plugins::runtime::{PluginEngine, PluginExecutionContext};
use crate::plugins::security::PluginSecurityGovernor;
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Universal adapter wrapping a plugin tool into `ToolHandler`
#[derive(Clone)]
pub struct PluginToolWrapper {
    pub namespaced_name: String,
    pub original_name: String,
    pub description: String,
    pub parameters: Value,
    pub engine: Arc<dyn PluginEngine>,
    pub plugin_root: PathBuf,
    pub security_governor: Arc<PluginSecurityGovernor>,
    pub timeout_secs: u64,
}

impl PluginToolWrapper {
    pub fn new(
        manifest: &PluginManifest,
        tool_name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
        engine: Arc<dyn PluginEngine>,
        plugin_root: PathBuf,
        security_governor: Arc<PluginSecurityGovernor>,
    ) -> Self {
        let original_name = tool_name.into();
        let namespaced_name = format!("{}__lib_{}", manifest.plugin.name, original_name);
        let perms = manifest.permissions();
        let timeout_secs = perms.timeout_secs.max(1);

        Self {
            namespaced_name,
            original_name,
            description: description.into(),
            parameters,
            engine,
            plugin_root,
            security_governor,
            timeout_secs,
        }
    }
}

#[async_trait]
impl ToolHandler for PluginToolWrapper {
    fn name(&self) -> &str {
        &self.namespaced_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        self.parameters.clone()
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Enforce capability bounds and AgentShield interceptor
        self.security_governor.verify_tool_invocation(
            &self.engine.metadata().name,
            &self.original_name,
            &arguments,
            self.engine.capabilities(),
            &current_dir,
        )?;

        let context = PluginExecutionContext {
            plugin_root: self.plugin_root.clone(),
            working_dir: current_dir,
            timeout: Duration::from_secs(self.timeout_secs),
            security_governor: self.security_governor.clone(),
        };

        self.engine
            .execute_tool(&self.original_name, arguments, &context)
            .await
    }
}
