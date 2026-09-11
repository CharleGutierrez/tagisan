use async_trait::async_trait;
use crate::engine::EngineContext;
use crate::error::Result;
use crate::strategies::{CollaborationStrategy, IntermediateStep, StrategyInput, StrategyOutput};
use crate::swarm::harmony::{build_standard_harmony_pipeline, RoleModelOverrides};
use crate::types::Message;

/// Strategy implementation of the Structured Role-Based Harmony Swarm (*Bayanihan*).
pub struct StructuredHarmonyStrategy {
    pub overrides: RoleModelOverrides,
    pub audit: bool,
}

impl StructuredHarmonyStrategy {
    pub fn new(overrides: RoleModelOverrides, audit: bool) -> Self {
        Self { overrides, audit }
    }

    pub fn default_local() -> Self {
        Self {
            overrides: RoleModelOverrides::default(),
            audit: false,
        }
    }

    pub fn with_audit(mut self, audit: bool) -> Self {
        self.audit = audit;
        self
    }
}

impl Default for StructuredHarmonyStrategy {
    fn default() -> Self {
        Self::default_local()
    }
}

#[async_trait]
impl CollaborationStrategy for StructuredHarmonyStrategy {
    fn name(&self) -> &'static str {
        "Structured Role-Based Harmony Swarm (Bayanihan)"
    }

    async fn execute(&self, input: StrategyInput, ctx: &EngineContext) -> Result<StrategyOutput> {
        let pipeline = build_standard_harmony_pipeline(&input.prompt, ctx, &self.overrides, self.audit);
        let result = pipeline.execute(ctx).await?;

        let mut steps = Vec::new();
        for artifact in &result.artifacts {
            steps.push(IntermediateStep {
                step_name: format!("{}: {}", artifact.role_id, artifact.role_title),
                provider: artifact.provider.clone(),
                model: artifact.model.clone(),
                message: Message::assistant(artifact.raw_output.clone()),
                latency: std::time::Duration::from_secs_f64(artifact.latency_secs),
            });
        }

        if let Some(ref audit) = result.audit_verdict {
            steps.push(IntermediateStep {
                step_name: "Audit: Adversarial Security & Architecture Review".to_string(),
                provider: "auditor".to_string(),
                model: "auditor".to_string(),
                message: Message::assistant(audit.clone()),
                latency: std::time::Duration::from_secs(0),
            });
        }

        let mut final_answer = result.complete_project;
        if let Some(ref audit) = result.audit_verdict {
            final_answer.push_str("\n\n// =========================================================================\n");
            final_answer.push_str("// ADVERSARIAL AUDIT VERDICT:\n");
            final_answer.push_str("// =========================================================================\n");
            final_answer.push_str(audit);
        }

        Ok(StrategyOutput {
            strategy_name: self.name(),
            final_answer,
            intermediate_steps: steps,
            total_usage: result.total_usage,
            total_cost_usd: result.total_cost_usd,
            total_latency: result.total_latency,
        })
    }
}
