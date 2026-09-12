use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::strategies::{CollaborationStrategy, IntermediateStep, StrategyInput, StrategyOutput};
use crate::types::{CompletionRequest, TokenUsage};
use async_trait::async_trait;
use std::time::Instant;

/// Dialectical Debate Strategy (Tagisan ng Talino / Balagtasan)
pub struct DialecticalDebateStrategy {
    pub proponent: (String, String),  // (provider_id, model_name) - Thesis
    pub adversary: (String, String),  // (provider_id, model_name) - Antithesis
    pub adjudicator: (String, String), // (provider_id, model_name) - Lakandiwa / Synthesis
}

impl DialecticalDebateStrategy {
    pub fn new(
        proponent: (String, String),
        adversary: (String, String),
        adjudicator: (String, String),
    ) -> Self {
        Self {
            proponent,
            adversary,
            adjudicator,
        }
    }
}

#[async_trait]
impl CollaborationStrategy for DialecticalDebateStrategy {
    fn name(&self) -> &'static str {
        "Dialectical Debate (Tagisan ng Talino)"
    }

    async fn execute(&self, input: StrategyInput, ctx: &EngineContext) -> Result<StrategyOutput> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut intermediate_steps = Vec::new();
        let mut total_usage = TokenUsage::default();

        // ----------------------------------------------------
        // ROUND 1: THESIS (Proponent drafts initial solution)
        // ----------------------------------------------------
        let (p_provider_id, p_model) = &self.proponent;
        let p_provider = ctx.get_provider(p_provider_id)?;

        let thesis_prompt = format!(
            "You are the Proponent in a high-rigor peer debate.\n\
            User Prompt:\n\"{}\"\n\n\
            TASK: Provide a comprehensive, thoroughly reasoned initial solution. Be precise, detailed, and structure your arguments logically.",
            input.prompt
        );

        let dispatcher = crate::ecc::skills::global_dispatcher();
        let (thesis_prompt, _) = dispatcher.equip_prompt_for_provider_with_bias(
            &thesis_prompt,
            &input.prompt,
            p_provider_id,
            None,
            Some("ba"),
        );

        let mut thesis_req = CompletionRequest::new(p_model.clone(), thesis_prompt)
            .with_temperature(0.7)
            .with_cancellation(ctx.cancellation_token.clone());

        if let Some(ref sys) = input.system_instruction {
            thesis_req = thesis_req.with_system(sys.clone());
        }

        let thesis_resp = p_provider.complete(thesis_req).await?;
        ctx.budget_tracker.record(p_model, thesis_resp.usage.prompt_tokens, thesis_resp.usage.completion_tokens)?;
        total_usage.prompt_tokens += thesis_resp.usage.prompt_tokens;
        total_usage.completion_tokens += thesis_resp.usage.completion_tokens;

        let thesis_text = thesis_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 1: Thesis (Proponent)".to_string(),
            provider: p_provider_id.clone(),
            model: p_model.clone(),
            message: thesis_resp.message,
            latency: thesis_resp.latency,
        });

        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        // ----------------------------------------------------
        // ROUND 2: ANTITHESIS (Adversary critiques & probes bugs)
        // ----------------------------------------------------
        let (a_provider_id, a_model) = &self.adversary;
        let a_provider = ctx.get_provider(a_provider_id)?;

        let antithesis_prompt = format!(
            "You are the Adversarial Critic / Red Team in a high-rigor peer debate.\n\n\
            Original User Prompt:\n\"{}\"\n\n\
            Proponent's Proposed Solution:\n\"\"\"\n{}\n\"\"\"\n\n\
            TASK:\n\
            1. Ruthlessly scrutinize the proposed solution for subtle bugs, logical flaws, security vulnerabilities, edge cases, and performance bottlenecks.\n\
            2. Point out unsupported assumptions and propose concrete corrections.",
            input.prompt, thesis_text
        );

        let adversary_query = format!("{} critique verification testing edge cases security", input.prompt);
        let (antithesis_prompt, _) = dispatcher.equip_prompt_for_provider_with_bias(
            &antithesis_prompt,
            &adversary_query,
            a_provider_id,
            None,
            Some("security"),
        );

        let mut antithesis_req = CompletionRequest::new(a_model.clone(), antithesis_prompt)
            .with_temperature(0.4)
            .with_cancellation(ctx.cancellation_token.clone());

        if let Some(ref sys) = input.system_instruction {
            antithesis_req = antithesis_req.with_system(sys.clone());
        }

        let antithesis_resp = a_provider.complete(antithesis_req).await?;
        ctx.budget_tracker.record(a_model, antithesis_resp.usage.prompt_tokens, antithesis_resp.usage.completion_tokens)?;
        total_usage.prompt_tokens += antithesis_resp.usage.prompt_tokens;
        total_usage.completion_tokens += antithesis_resp.usage.completion_tokens;

        let antithesis_text = antithesis_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 2: Antithesis (Adversarial Critique)".to_string(),
            provider: a_provider_id.clone(),
            model: a_model.clone(),
            message: antithesis_resp.message,
            latency: antithesis_resp.latency,
        });

        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        // ----------------------------------------------------
        // ROUND 3: SYNTHESIS / LAKANDIWA (Master Verdict)
        // ----------------------------------------------------
        let (adj_provider_id, adj_model) = &self.adjudicator;
        let adj_provider = ctx.get_provider(adj_provider_id)?;

        let synthesis_prompt = format!(
            "You are the Lakandiwa (Chief Adjudicator & Synthesizer) in this Tagisan debate.\n\n\
            Original User Prompt:\n\"{}\"\n\n\
            --- ROUND 1 (Thesis by {} / {}) ---\n{}\n\n\
            --- ROUND 2 (Antithesis Critique by {} / {}) ---\n{}\n\n\
            TASK:\n\
            1. Evaluate both arguments objectively.\n\
            2. Reconcile valid criticisms, discard invalid attacks, and produce the definitive, battle-tested, verified final solution.",
            input.prompt,
            p_provider_id, p_model, thesis_text,
            a_provider_id, a_model, antithesis_text
        );

        let synthesis_query = format!("{} software architecture design synthesis reconciliation", input.prompt);
        let (synthesis_prompt, _) = dispatcher.equip_prompt_for_provider_with_bias(
            &synthesis_prompt,
            &synthesis_query,
            adj_provider_id,
            None,
            Some("standards"),
        );

        let mut synthesis_req = CompletionRequest::new(adj_model.clone(), synthesis_prompt)
            .with_temperature(0.2)
            .with_cancellation(ctx.cancellation_token.clone());

        if let Some(ref sys) = input.system_instruction {
            synthesis_req = synthesis_req.with_system(sys.clone());
        }

        let synthesis_resp = adj_provider.complete(synthesis_req).await?;
        ctx.budget_tracker.record(adj_model, synthesis_resp.usage.prompt_tokens, synthesis_resp.usage.completion_tokens)?;
        total_usage.prompt_tokens += synthesis_resp.usage.prompt_tokens;
        total_usage.completion_tokens += synthesis_resp.usage.completion_tokens;

        let final_text = synthesis_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 3: Synthesis (Lakandiwa Verdict)".to_string(),
            provider: adj_provider_id.clone(),
            model: adj_model.clone(),
            message: synthesis_resp.message,
            latency: synthesis_resp.latency,
        });

        Ok(StrategyOutput {
            strategy_name: self.name(),
            final_answer: final_text,
            intermediate_steps,
            total_usage,
            total_cost_usd: ctx.budget_tracker.current_spent_usd(),
            total_latency: start_time.elapsed(),
        })
    }
}
