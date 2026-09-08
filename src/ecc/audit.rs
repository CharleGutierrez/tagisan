use crate::ecc::presets;
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::strategies::{CollaborationStrategy, IntermediateStep, StrategyInput, StrategyOutput};
use crate::types::{CompletionRequest, TokenUsage};
use async_trait::async_trait;
use std::time::Instant;

/// Adversarial Multi-Model Audit Debate using specialized ECC personas
pub struct EccAuditDebate {
    pub architect_spec: (String, String),  // (provider_id, model)
    pub security_spec: (String, String),   // (provider_id, model)
    pub adjudicator_spec: (String, String), // (provider_id, model)
}

impl EccAuditDebate {
    pub fn new(
        architect: (String, String),
        security_auditor: (String, String),
        chief_adjudicator: (String, String),
    ) -> Self {
        Self {
            architect_spec: architect,
            security_spec: security_auditor,
            adjudicator_spec: chief_adjudicator,
        }
    }
}

#[async_trait]
impl CollaborationStrategy for EccAuditDebate {
    fn name(&self) -> &'static str {
        "ECC Adversarial Engineering Audit"
    }

    async fn execute(&self, input: StrategyInput, ctx: &EngineContext) -> Result<StrategyOutput> {
        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        let start_time = Instant::now();
        let mut intermediate_steps = Vec::new();
        let mut total_usage = TokenUsage::default();

        // ----------------------------------------------------
        // ROUND 1: ARCHITECT PROPOSAL (ECC Architect Persona)
        // ----------------------------------------------------
        let (arch_prov_id, arch_model) = &self.architect_spec;
        let arch_prov = ctx.get_provider(arch_prov_id)?;
        let arch_persona = presets::architect();

        let arch_prompt = format!(
            "You are the ECC Lead Systems Architect.\n\
            User Prompt / Problem Statement:\n\"{}\"\n\n\
            TASK:\n\
            1. Formulate a comprehensive, production-grade architectural design or solution.\n\
            2. Be explicit about concurrency model, data flow, error handling, and component boundaries.\n\
            3. Defend your design decisions against potential scaling bottlenecks.",
            input.prompt
        );

        let mut arch_req = CompletionRequest::new(arch_model.clone(), arch_prompt)
            .with_system(arch_persona.system_prompt)
            .with_temperature(0.5)
            .with_cancellation(ctx.cancellation_token.clone());

        let arch_resp = arch_prov.complete(arch_req).await?;
        ctx.budget_tracker.record(
            arch_model,
            arch_resp.usage.prompt_tokens,
            arch_resp.usage.completion_tokens,
        )?;
        total_usage.prompt_tokens += arch_resp.usage.prompt_tokens;
        total_usage.completion_tokens += arch_resp.usage.completion_tokens;

        let proposal_text = arch_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 1: Architectural Proposal (ECC Architect)".to_string(),
            provider: arch_prov_id.clone(),
            model: arch_model.clone(),
            message: arch_resp.message,
            latency: arch_resp.latency,
        });

        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        // ----------------------------------------------------
        // ROUND 2: SECURITY & RESILIENCE RED-TEAM (ECC Security Auditor)
        // ----------------------------------------------------
        let (sec_prov_id, sec_model) = &self.security_spec;
        let sec_prov = ctx.get_provider(sec_prov_id)?;
        let sec_persona = presets::security_auditor();

        let sec_prompt = format!(
            "You are the ECC Principal Security Auditor and Red-Team Specialist.\n\n\
            Original Problem:\n\"{}\"\n\n\
            Architect's Proposed Design:\n\"\"\"\n{}\n\"\"\"\n\n\
            TASK:\n\
            1. Ruthlessly probe this proposed architecture for vulnerabilities, injection vectors, TOCTOU/race conditions, and denial-of-service risks.\n\
            2. Highlight any unvalidated assumptions, memory safety concerns, or credential/secret leakage potentials.\n\
            3. Propose concrete, non-negotiable security requirements to harden the system.",
            input.prompt, proposal_text
        );

        let mut sec_req = CompletionRequest::new(sec_model.clone(), sec_prompt)
            .with_system(sec_persona.system_prompt)
            .with_temperature(0.3)
            .with_cancellation(ctx.cancellation_token.clone());

        let sec_resp = sec_prov.complete(sec_req).await?;
        ctx.budget_tracker.record(
            sec_model,
            sec_resp.usage.prompt_tokens,
            sec_resp.usage.completion_tokens,
        )?;
        total_usage.prompt_tokens += sec_resp.usage.prompt_tokens;
        total_usage.completion_tokens += sec_resp.usage.completion_tokens;

        let critique_text = sec_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 2: Threat & Vulnerability Audit (ECC Security Auditor)".to_string(),
            provider: sec_prov_id.clone(),
            model: sec_model.clone(),
            message: sec_resp.message,
            latency: sec_resp.latency,
        });

        if ctx.cancellation_token.is_cancelled() {
            return Err(TagisanError::Cancelled);
        }

        // ----------------------------------------------------
        // ROUND 3: LAKANDIWA SYNTHESIS & AUDIT VERDICT (ECC Chief Adjudicator)
        // ----------------------------------------------------
        let (adj_prov_id, adj_model) = &self.adjudicator_spec;
        let adj_prov = ctx.get_provider(adj_prov_id)?;

        let adj_prompt = format!(
            "You are the Chief Adjudicator and Senior Engineering Director (Lakandiwa) under the ECC framework.\n\n\
            Context:\n\
            Problem: \"{}\"\n\n\
            1. Architect's Proposal:\n\"\"\"\n{}\n\"\"\"\n\n\
            2. Security Auditor's Critique:\n\"\"\"\n{}\n\"\"\"\n\n\
            TASK:\n\
            1. Impartially adjudicate both rounds: validate legitimate security findings while dismissing pedantic nitpicks.\n\
            2. Synthesize the final, battle-hardened production specification incorporating the necessary security and resilience mitigations.\n\
            3. Deliver the definitive master verdict.",
            input.prompt, proposal_text, critique_text
        );

        let mut adj_req = CompletionRequest::new(adj_model.clone(), adj_prompt)
            .with_system("You are the definitive ECC Adjudicator and Chief Engineering Director. Deliver balanced, authoritative, and verified synthesis.")
            .with_temperature(0.2)
            .with_cancellation(ctx.cancellation_token.clone());

        let adj_resp = adj_prov.complete(adj_req).await?;
        ctx.budget_tracker.record(
            adj_model,
            adj_resp.usage.prompt_tokens,
            adj_resp.usage.completion_tokens,
        )?;
        total_usage.prompt_tokens += adj_resp.usage.prompt_tokens;
        total_usage.completion_tokens += adj_resp.usage.completion_tokens;

        let final_verdict = adj_resp.message.extract_text();
        intermediate_steps.push(IntermediateStep {
            step_name: "Round 3: Lakandiwa Synthesis & Hardened Verdict".to_string(),
            provider: adj_prov_id.clone(),
            model: adj_model.clone(),
            message: adj_resp.message,
            latency: adj_resp.latency,
        });

        Ok(StrategyOutput {
            strategy_name: self.name(),
            final_answer: final_verdict,
            intermediate_steps,
            total_usage,
            total_cost_usd: ctx.budget_tracker.current_spent_usd(),
            total_latency: start_time.elapsed(),
        })
    }
}
