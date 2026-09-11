//! Cognitive Lifecycle Extension Hooks for Tagisan Swarm
//!
//! Provides traits and registry for extending tool provisioning, skill cataloging,
//! ECC pipeline stages, debate arbitration judges, and AgentShield domain guardrails.

use crate::ecc::agentshield::AgentShieldVerdict;
use crate::ecc::skills::EccSkill;
use crate::error::Result;
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// 1. ToolProvider Hook
// ---------------------------------------------------------------------------
#[async_trait]
pub trait PluginToolProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    fn provide_tools(&self) -> Vec<Arc<dyn ToolHandler>>;
}

// ---------------------------------------------------------------------------
// 2. SkillPack Hook
// ---------------------------------------------------------------------------
pub trait PluginSkillPack: Send + Sync {
    fn pack_id(&self) -> &str;
    fn provide_skills(&self) -> Vec<EccSkill>;
}

// ---------------------------------------------------------------------------
// 3. PipelineStage Hook (ECC 5-Stage DAG Injection)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageContext {
    pub objective: String,
    pub previous_stage_outputs: HashMap<String, String>,
    pub working_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageOutput {
    pub stage_name: String,
    pub content: String,
    pub success: bool,
}

#[async_trait]
pub trait PluginPipelineStage: Send + Sync {
    fn stage_id(&self) -> &str;
    fn stage_name(&self) -> &str;
    fn dependencies(&self) -> Vec<String>;
    async fn execute_stage(&self, context: &PipelineStageContext) -> Result<PipelineStageOutput>;
}

// ---------------------------------------------------------------------------
// 4. DebateJudge Hook (Consensus & Arbitration)
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRoundInfo {
    pub round: usize,
    pub speaker: String,
    pub argument: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateJudgeVerdict {
    pub winner: String,
    pub confidence_score: f32,
    pub rationale: String,
}

#[async_trait]
pub trait PluginDebateJudge: Send + Sync {
    fn judge_id(&self) -> &str;
    async fn arbitrate(&self, rounds: &[DebateRoundInfo]) -> Result<DebateJudgeVerdict>;
}

// ---------------------------------------------------------------------------
// 5. ShieldInterceptor Hook (Domain Guardrails: HIPAA, PII, SQLi)
// ---------------------------------------------------------------------------
#[async_trait]
pub trait PluginShieldInterceptor: Send + Sync {
    fn interceptor_id(&self) -> &str;
    async fn audit_tool_call(&self, tool_name: &str, arguments: &Value) -> Result<AgentShieldVerdict>;
    async fn audit_prompt(&self, prompt: &str) -> Result<AgentShieldVerdict>;
}

// ---------------------------------------------------------------------------
// 6. TelemetryExporter Hook
// ---------------------------------------------------------------------------
#[async_trait]
pub trait PluginTelemetryExporter: Send + Sync {
    fn exporter_id(&self) -> &str;
    async fn export_metric(
        &self,
        metric_name: &str,
        value: f64,
        labels: &[(String, String)],
    ) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Hook Registry
// ---------------------------------------------------------------------------
#[derive(Default)]
pub struct PluginHookRegistry {
    pub tool_providers: Vec<Arc<dyn PluginToolProvider>>,
    pub skill_packs: Vec<Arc<dyn PluginSkillPack>>,
    pub pipeline_stages: Vec<Arc<dyn PluginPipelineStage>>,
    pub debate_judges: Vec<Arc<dyn PluginDebateJudge>>,
    pub shield_interceptors: Vec<Arc<dyn PluginShieldInterceptor>>,
    pub telemetry_exporters: Vec<Arc<dyn PluginTelemetryExporter>>,
}

impl PluginHookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tool_provider(&mut self, provider: Arc<dyn PluginToolProvider>) {
        self.tool_providers.push(provider);
    }

    pub fn register_skill_pack(&mut self, pack: Arc<dyn PluginSkillPack>) {
        self.skill_packs.push(pack);
    }

    pub fn register_stage(&mut self, stage: Arc<dyn PluginPipelineStage>) {
        self.pipeline_stages.push(stage);
    }

    pub fn register_judge(&mut self, judge: Arc<dyn PluginDebateJudge>) {
        self.debate_judges.push(judge);
    }

    pub fn register_interceptor(&mut self, interceptor: Arc<dyn PluginShieldInterceptor>) {
        self.shield_interceptors.push(interceptor);
    }

    pub fn register_telemetry(&mut self, exporter: Arc<dyn PluginTelemetryExporter>) {
        self.telemetry_exporters.push(exporter);
    }
}
