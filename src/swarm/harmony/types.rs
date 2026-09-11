use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::engine::EngineContext;
use crate::error::Result;
use crate::providers::LlmProvider;
use crate::swarm::harmony::blackboard::SwarmBlackboard;

/// Cloud optimization tier profile for allocating models across assembly roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HarmonyTierProfile {
    /// Smart hybrid tier: Top reasoning/coding for Architect & QA, high-speed & cost-effective for Impl & Doc.
    #[default]
    Smart,
    /// Flagship tier: Uses top-tier frontier models (Claude 3.5 Sonnet / GPT-4o) across all roles.
    Flagship,
    /// Economy tier: Uses ultra-fast, cost-effective models (Gemini 2.0 Flash / DeepSeek V3) across all roles.
    Economy,
}

/// Configuration defining an assigned role in the harmony assembly line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HarmonyRoleConfig {
    pub role_id: String,
    pub role_title: String,
    pub provider: String,
    pub model: String,
    pub system_contract: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl HarmonyRoleConfig {
    pub fn new(
        role_id: impl Into<String>,
        role_title: impl Into<String>,
        provider: impl Into<String>,
        model: impl Into<String>,
        system_contract: impl Into<String>,
    ) -> Self {
        Self {
            role_id: role_id.into(),
            role_title: role_title.into(),
            provider: provider.into(),
            model: model.into(),
            system_contract: system_contract.into(),
            max_tokens: 4096,
            temperature: 0.3,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
}

/// A parsed code snippet extracted from a role's generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtractedCodeBlock {
    pub language: String,
    pub code: String,
}

/// Telemetry record capturing a dynamic stage evacuation from cloud to local LLM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FailoverEvent {
    pub original_provider: String,
    pub original_model: String,
    pub trigger_reason: String,
    pub evacuated_to_provider: String,
    pub evacuated_to_model: String,
    pub timestamp_epoch_ms: u64,
    pub cost_at_failover_usd: f64,
}

/// Complete output artifact produced by a single role stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleArtifact {
    pub role_id: String,
    pub role_title: String,
    pub provider: String,
    pub model: String,
    pub raw_output: String,
    pub code_blocks: Vec<ExtractedCodeBlock>,
    pub latency_secs: f64,
    pub tokens_used: u32,
    #[serde(default)]
    pub failover_event: Option<FailoverEvent>,
}

impl RoleArtifact {
    pub fn primary_code(&self) -> Option<&str> {
        self.code_blocks.first().map(|b| b.code.as_str())
    }

    pub fn code_by_language(&self, lang: &str) -> Option<&str> {
        self.code_blocks
            .iter()
            .find(|b| b.language.eq_ignore_ascii_case(lang))
            .map(|b| b.code.as_str())
    }
}

/// Contract for a discrete role in the structured harmony assembly line.
#[async_trait]
pub trait HarmonyRole: Send + Sync {
    fn config(&self) -> &HarmonyRoleConfig;

    fn set_auto_skills(&mut self, _enabled: bool) {}

    async fn execute_stage(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        ctx: &EngineContext,
        critique: Option<&str>,
    ) -> Result<RoleArtifact>;

    async fn execute_stage_with_override(
        &self,
        blackboard: &SwarmBlackboard,
        provider: Arc<dyn LlmProvider>,
        model_override: Option<&str>,
        ctx: &EngineContext,
        critique: Option<&str>,
    ) -> Result<RoleArtifact> {
        let _ = model_override;
        self.execute_stage(blackboard, provider, ctx, critique).await
    }
}
