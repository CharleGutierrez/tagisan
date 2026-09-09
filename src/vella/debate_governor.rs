//! # Adversarial Debate & Borda Consensus Domain Governor
//!
//! Orchestrates high-stakes sovereign domain decisions: runs an adversarial
//! multi-agent debate (Proposer, Security Auditor, and Chief Adjudicator) followed
//! by rigorous Borda count consensus before authorizing critical Vella domain actions.

use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::swarm::consensus::{AgentReview, ConsensusVerdict, TeamConsensusEngine, VotingRule};
use crate::vella::VellaPolicyGovernor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

/// High-stakes domain action requiring adversarial peer debate and Borda consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainActionProposal {
    pub domain: String,       // e.g. "trading", "scada", "robotics", "medicine"
    pub action_type: String,  // e.g. "large_order", "coil_actuation", "estop_override", "experimental_drug"
    pub target: String,       // e.g. "VALVE_PRESSURE_MAIN", "BTC/USD_LEVERAGE_50X"
    pub parameters: Value,
    pub requested_by: String, // Agent ID or user identity
}

impl DomainActionProposal {
    pub fn new(
        domain: impl Into<String>,
        action_type: impl Into<String>,
        target: impl Into<String>,
        parameters: Value,
        requested_by: impl Into<String>,
    ) -> Self {
        Self {
            domain: domain.into(),
            action_type: action_type.into(),
            target: target.into(),
            parameters,
            requested_by: requested_by.into(),
        }
    }
}

/// Outcome of the adversarial debate and consensus vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VellaDebateVerdict {
    pub approved: bool,
    pub domain: String,
    pub action_type: String,
    pub target: String,
    pub winning_option: Option<String>,
    pub borda_points: HashMap<String, usize>,
    pub consensus: ConsensusVerdict,
    pub proposer_thesis: String,
    pub auditor_antithesis: String,
    pub judge_synthesis: String,
}

/// Coordinates adversarial multi-agent debate and voting before authorizing critical Vella actions
#[derive(Clone)]
pub struct VellaDebateGovernor {
    pub policy_governor: Arc<VellaPolicyGovernor>,
    pub consensus_engine: Arc<TeamConsensusEngine>,
}

impl Default for VellaDebateGovernor {
    fn default() -> Self {
        Self::new(Arc::new(VellaPolicyGovernor::default()))
    }
}

impl VellaDebateGovernor {
    pub fn new(policy_governor: Arc<VellaPolicyGovernor>) -> Self {
        Self {
            policy_governor,
            consensus_engine: Arc::new(TeamConsensusEngine::new()),
        }
    }

    /// Orchestrates an adversarial dialectical debate across 3 distinct perspectives:
    /// 1. Proponent (advocates for execution, utility, and mission success)
    /// 2. Security Auditor (rigorously probes safety margins, failure modes, and catastrophic risks)
    /// 3. Chief Adjudicator (synthesizes conflicting arguments, weighs risk/reward, and submits ranked options)
    pub async fn debate_and_govern(
        &self,
        proposal: &DomainActionProposal,
        _ctx: Option<&EngineContext>,
    ) -> Result<VellaDebateVerdict> {
        info!(
            "Initiating Vella Adversarial Debate Governance for domain '{}' action '{}' on target '{}'",
            proposal.domain, proposal.action_type, proposal.target
        );

        // First, check basic policy invariants
        if self.policy_governor.is_e_stop_active().await {
            warn!("DebateGovernor: Fast-rejecting proposal. System E-Stop is actively latched!");
            return Err(TagisanError::Execution(
                "System E-Stop is active. All domain action proposals are categorically blocked.".to_string(),
            ));
        }

        // 1. Proponent Review
        let (proposer_approved, proposer_score, proposer_thesis, prop_options) =
            self.evaluate_proponent(proposal);

        let mut prop_review = AgentReview::new("AgentProponent", proposer_approved, proposer_score)
            .with_score("correctness", 9)
            .with_score("performance", 9)
            .with_feedback(proposer_thesis.clone())
            .with_ranked_options(prop_options);

        if !proposer_approved {
            prop_review = prop_review.with_risk("Proponent detected unviable operational parameters");
        }

        // 2. Security Auditor Review (Red Team / Safety Audit)
        let (auditor_approved, auditor_score, auditor_antithesis, aud_risks, aud_options) =
            self.evaluate_auditor(proposal).await;

        let mut aud_review = AgentReview::new("AgentSecurityAuditor", auditor_approved, auditor_score)
            .with_score("security", if auditor_approved { 9 } else { 2 })
            .with_score("maintainability", 8)
            .with_feedback(auditor_antithesis.clone())
            .with_ranked_options(aud_options);

        for risk in aud_risks {
            aud_review = aud_review.with_risk(risk);
        }

        // 3. Chief Adjudicator (Lakandiwa / Synthesis)
        let (judge_approved, judge_score, judge_synthesis, judge_options) =
            self.evaluate_adjudicator(proposal, &prop_review, &aud_review);

        let judge_review = AgentReview::new("AgentChiefAdjudicator", judge_approved, judge_score)
            .with_score("correctness", judge_score)
            .with_score("security", judge_score)
            .with_score("maintainability", 9)
            .with_score("performance", 8)
            .with_feedback(judge_synthesis.clone())
            .with_ranked_options(judge_options);

        // Run Borda Consensus tally
        let reviews = vec![prop_review, aud_review, judge_review];
        let consensus = self.consensus_engine.evaluate(&VotingRule::WeightedBorda, &reviews);

        let approved = consensus.approved
            && consensus.winning_option.as_deref() == Some("EXECUTE_WITH_SAFETY_BOUNDS");

        let log_msg = format!(
            "[DEBATE-GOVERNOR] Domain: {} | Target: {} | Verdict: {} | Winning: {:?}",
            proposal.domain,
            proposal.target,
            if approved { "AUTHORIZED" } else { "REJECTED" },
            consensus.winning_option
        );

        {
            let mut log = self.policy_governor.audit_log.write().await;
            log.push(log_msg);
        }

        Ok(VellaDebateVerdict {
            approved,
            domain: proposal.domain.clone(),
            action_type: proposal.action_type.clone(),
            target: proposal.target.clone(),
            winning_option: consensus.winning_option.clone(),
            borda_points: consensus.borda_points.clone(),
            consensus,
            proposer_thesis,
            auditor_antithesis,
            judge_synthesis,
        })
    }

    fn evaluate_proponent(
        &self,
        proposal: &DomainActionProposal,
    ) -> (bool, u8, String, Vec<String>) {
        let thesis = format!(
            "Proponent Thesis: Action '{}' on target '{}' is strategically justified. Operational parameters {:?} fulfill system requirements with anticipated high utility.",
            proposal.action_type, proposal.target, proposal.parameters
        );
        let options = vec![
            "EXECUTE_WITH_SAFETY_BOUNDS".to_string(),
            "EXECUTE_UNCONDITIONAL".to_string(),
            "DEFER_ACTION".to_string(),
        ];
        (true, 9, thesis, options)
    }

    async fn evaluate_auditor(
        &self,
        proposal: &DomainActionProposal,
    ) -> (bool, u8, String, Vec<String>, Vec<String>) {
        let mut risks = Vec::new();
        let mut approved = true;

        // Domain-specific checks
        match proposal.domain.as_str() {
            "trading" => {
                let price = proposal.parameters.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let size = proposal.parameters.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                let leverage = proposal.parameters.get("leverage").and_then(|v| v.as_f64());

                if let Err(e) = self.policy_governor.validate_trade(&proposal.target, price, size, leverage) {
                    risks.push(format!("Trading policy violation: {}", e));
                    approved = false;
                }
            }
            "scada" => {
                let coil = proposal.parameters.get("coil_address").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                let state = proposal.parameters.get("state").and_then(|v| v.as_bool()).unwrap_or(false);

                if let Err(e) = self.policy_governor.validate_scada_actuation(coil, state).await {
                    risks.push(format!("SCADA actuator policy violation: {}", e));
                    approved = false;
                }
            }
            "robotics" => {
                let velocity = proposal.parameters.get("velocity_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
                if let Err(e) = self.policy_governor.validate_robotics_motion(velocity).await {
                    risks.push(format!("Kinematics safety breach: {}", e));
                    approved = false;
                }
            }
            "medicine" => {
                let dosage = proposal.parameters.get("dosage_mg").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let max_safe = proposal.parameters.get("max_safe_mg").and_then(|v| v.as_f64()).unwrap_or(1000.0);
                if dosage > max_safe {
                    risks.push(format!("Dosage {}mg exceeds max clinical limit of {}mg", dosage, max_safe));
                    approved = false;
                }
            }
            _ => {}
        }

        let score = if approved { 8 } else { 3 };
        let antithesis = if approved {
            format!(
                "Auditor Antithesis: Scrutiny complete. Parameters for '{}' satisfy all domain containment policies.",
                proposal.target
            )
        } else {
            format!(
                "Auditor Antithesis: CRITICAL HAZARD DETECTED in proposal for '{}'. Identified violations: {:?}",
                proposal.target, risks
            )
        };

        let options = if approved {
            vec![
                "EXECUTE_WITH_SAFETY_BOUNDS".to_string(),
                "DEFER_ACTION".to_string(),
                "ABORT_ACTION".to_string(),
            ]
        } else {
            vec![
                "ABORT_ACTION".to_string(),
                "DEFER_ACTION".to_string(),
                "EXECUTE_WITH_SAFETY_BOUNDS".to_string(),
            ]
        };

        (approved, score, antithesis, risks, options)
    }

    fn evaluate_adjudicator(
        &self,
        proposal: &DomainActionProposal,
        proponent: &AgentReview,
        auditor: &AgentReview,
    ) -> (bool, u8, String, Vec<String>) {
        if !auditor.approved {
            let synthesis = format!(
                "Adjudicator Synthesis: The Security Auditor identified fatal risks ({:?}). Overriding Proponent. Action REJECTED to preserve physical and financial safety.",
                auditor.identified_risks
            );
            let options = vec![
                "ABORT_ACTION".to_string(),
                "DEFER_ACTION".to_string(),
                "EXECUTE_WITH_SAFETY_BOUNDS".to_string(),
            ];
            (false, 3, synthesis, options)
        } else {
            let synthesis = format!(
                "Adjudicator Synthesis: Both Proponent ({}/10) and Auditor ({}/10) concur that action '{}' on '{}' is well-bounded. Action AUTHORIZED under continuous telemetry supervision.",
                proponent.overall_score, auditor.overall_score, proposal.action_type, proposal.target
            );
            let options = vec![
                "EXECUTE_WITH_SAFETY_BOUNDS".to_string(),
                "DEFER_ACTION".to_string(),
                "ABORT_ACTION".to_string(),
            ];
            (true, 9, synthesis, options)
        }
    }
}
