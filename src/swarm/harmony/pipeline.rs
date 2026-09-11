use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::swarm::harmony::blackboard::SwarmBlackboard;
use crate::swarm::harmony::gates::{GateResult, ValidationGate};
use crate::swarm::harmony::types::{HarmonyRole, RoleArtifact};
use crate::types::{CompletionRequest, TokenUsage};

/// Pipeline stage configuration containing the executing role and attached validation gates.
pub struct HarmonyStage {
    pub role: Box<dyn HarmonyRole>,
    pub gates: Vec<Box<dyn ValidationGate>>,
}

impl HarmonyStage {
    pub fn new(role: Box<dyn HarmonyRole>) -> Self {
        Self {
            role,
            gates: Vec::new(),
        }
    }

    pub fn with_gate(mut self, gate: Box<dyn ValidationGate>) -> Self {
        self.gates.push(gate);
        self
    }
}

/// The complete execution outcome of a Structured Role-Based Harmony Swarm pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmonyExecutionResult {
    pub objective: String,
    pub artifacts: Vec<RoleArtifact>,
    pub complete_project: String,
    pub audit_verdict: Option<String>,
    pub total_latency: Duration,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
}

/// The orchestrator executing the multi-stage assembly line across local and cloud LLMs.
pub struct StructuredHarmonyPipeline {
    pub blackboard: Arc<SwarmBlackboard>,
    stages: Vec<HarmonyStage>,
    audit_adversary: Option<(String, String)>,
    max_stage_retries: usize,
}

impl StructuredHarmonyPipeline {
    /// Create a new pipeline for a specific user objective.
    pub fn new(objective: impl Into<String>) -> Self {
        let obj = objective.into();
        Self {
            blackboard: Arc::new(SwarmBlackboard::new(obj)),
            stages: Vec::new(),
            audit_adversary: None,
            max_stage_retries: 2,
        }
    }

    /// Add a stage with its role and validation gates.
    pub fn add_stage(mut self, stage: HarmonyStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Attach an optional adversarial audit model to critique the final assembly.
    pub fn with_audit(mut self, adversary: (String, String)) -> Self {
        self.audit_adversary = Some(adversary);
        self
    }

    /// Set maximum retries allowed if a validation gate triggers a critique.
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_stage_retries = retries;
        self
    }

    /// Execute the complete assembly line pipeline.
    pub async fn execute(&self, ctx: &EngineContext) -> Result<HarmonyExecutionResult> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut total_tokens = TokenUsage::default();

        for (idx, stage) in self.stages.iter().enumerate() {
            if ctx.cancellation_token.is_cancelled() {
                return Err(TagisanError::Cancelled);
            }

            let role_cfg = stage.role.config();
            let provider = ctx.get_provider(&role_cfg.provider)?;

            let mut retries = 0usize;
            let mut last_critique: Option<String> = None;
            let mut committed_artifact: Option<RoleArtifact> = None;

            while retries <= self.max_stage_retries {
                if ctx.cancellation_token.is_cancelled() {
                    return Err(TagisanError::Cancelled);
                }

                let artifact = stage
                    .role
                    .execute_stage(
                        &self.blackboard,
                        provider.clone(),
                        ctx,
                        last_critique.as_deref(),
                    )
                    .await?;

                // Run all validation gates
                let mut all_passed = true;
                for gate in &stage.gates {
                    match gate.validate(&artifact) {
                        GateResult::Pass => {}
                        GateResult::RetryWithCritique { critique } => {
                            last_critique = Some(critique);
                            all_passed = false;
                            break;
                        }
                        GateResult::HardFailure { reason } => {
                            return Err(TagisanError::Execution(format!(
                                "Stage {} ('{}') hard gate failure: {}",
                                idx + 1,
                                role_cfg.role_title,
                                reason
                            )));
                        }
                    }
                }

                if all_passed {
                    committed_artifact = Some(artifact);
                    break;
                } else {
                    retries += 1;
                }
            }

            let final_artifact = committed_artifact.ok_or_else(|| {
                TagisanError::Execution(format!(
                    "Stage {} ('{}') failed to satisfy validation gates after {} retries. Last critique: {:?}",
                    idx + 1,
                    role_cfg.role_title,
                    self.max_stage_retries,
                    last_critique
                ))
            })?;

            total_tokens.prompt_tokens += final_artifact.tokens_used / 2;
            total_tokens.completion_tokens += final_artifact.tokens_used / 2;

            self.blackboard.append_artifact(final_artifact);
        }

        let complete_project = self.blackboard.assemble_complete_project();

        // Optional Adversarial Audit Phase
        let mut audit_verdict = None;
        if let Some((ref adv_prov_id, ref adv_model)) = self.audit_adversary {
            if !ctx.cancellation_token.is_cancelled() {
                let adv_prov = ctx.get_provider(adv_prov_id)?;
                let audit_prompt = format!(
                    "You are a Principal Security and Architecture Auditor inspecting the assembled project.\n\n\
                    PROJECT OBJECTIVE:\n\"{}\"\n\n\
                    ASSEMBLED CODEBASE:\n\"\"\"\n{}\n\"\"\"\n\n\
                    TASK:\n\
                    1. Rigorously inspect this implementation for edge-case vulnerabilities, performance regressions, or type violations.\n\
                    2. Provide an executive summary and final verdict (APPROVED, CONDITIONAL, or REJECTED) with concrete reasoning.",
                    self.blackboard.user_objective, complete_project
                );

                let req = CompletionRequest::new(adv_model.clone(), audit_prompt)
                    .with_temperature(0.3)
                    .with_cancellation(ctx.cancellation_token.clone());

                let resp = adv_prov.complete(req).await?;
                total_tokens.prompt_tokens += resp.usage.prompt_tokens;
                total_tokens.completion_tokens += resp.usage.completion_tokens;
                audit_verdict = Some(resp.message.extract_text());
            }
        }

        let total_latency = start_time.elapsed();
        let total_cost_usd = ctx.budget_tracker.current_spent_usd();

        Ok(HarmonyExecutionResult {
            objective: self.blackboard.user_objective.clone(),
            artifacts: self.blackboard.get_artifacts(),
            complete_project,
            audit_verdict,
            total_latency,
            total_usage: total_tokens,
            total_cost_usd,
        })
    }
}
