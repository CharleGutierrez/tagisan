use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::swarm::coordinator::SwarmCoordinator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Voting rules supported by the Team Consensus Engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VotingRule {
    /// Strict majority: > 50% approval
    Majority,
    /// Unanimous agreement: 100% approval
    Unanimous,
    /// Super-majority with configurable ratio (e.g. 0.66 or 0.75)
    SuperMajority(f32),
    /// Positional Borda count for ranked alternative selection
    WeightedBorda,
}

/// Standard criteria used to evaluate code, architecture, or proposals
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewCriterion {
    Correctness,
    Security,
    Maintainability,
    Performance,
    Custom(String),
}

impl ReviewCriterion {
    pub fn as_str(&self) -> &str {
        match self {
            ReviewCriterion::Correctness => "correctness",
            ReviewCriterion::Security => "security",
            ReviewCriterion::Maintainability => "maintainability",
            ReviewCriterion::Performance => "performance",
            ReviewCriterion::Custom(s) => s.as_str(),
        }
    }
}

/// An individual review submitted by a swarm agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReview {
    pub reviewer: String,
    pub approved: bool,
    pub scores: HashMap<String, u8>, // 1 to 10
    pub overall_score: u8,           // 1 to 10
    pub feedback: String,
    pub identified_risks: Vec<String>,
    pub confidence: f32, // 0.0 to 1.0
    pub ranked_options: Vec<String>,
}

impl AgentReview {
    pub fn new(reviewer: impl Into<String>, approved: bool, overall_score: u8) -> Self {
        Self {
            reviewer: reviewer.into(),
            approved,
            scores: HashMap::new(),
            overall_score: overall_score.clamp(1, 10),
            feedback: String::new(),
            identified_risks: Vec::new(),
            confidence: 1.0,
            ranked_options: Vec::new(),
        }
    }

    pub fn with_score(mut self, criterion: impl Into<String>, score: u8) -> Self {
        self.scores.insert(criterion.into(), score.clamp(1, 10));
        self
    }

    pub fn with_feedback(mut self, feedback: impl Into<String>) -> Self {
        self.feedback = feedback.into();
        self
    }

    pub fn with_risk(mut self, risk: impl Into<String>) -> Self {
        self.identified_risks.push(risk.into());
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_ranked_options(mut self, options: Vec<String>) -> Self {
        self.ranked_options = options;
        self
    }
}

/// Consolidated consensus verdict produced by the Team Consensus Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVerdict {
    pub rule: VotingRule,
    pub approved: bool,
    pub voters_count: usize,
    pub approval_ratio: f32,
    pub average_score: f32,
    pub criterion_averages: HashMap<String, f32>,
    pub borda_points: HashMap<String, usize>,
    pub winning_option: Option<String>,
    pub reviews: Vec<AgentReview>,
    pub synthesis: String,
    pub action_items: Vec<String>,
}

/// Team Consensus Engine coordinating multi-agent evaluations and tallying votes
#[derive(Clone, Default)]
pub struct TeamConsensusEngine;

impl TeamConsensusEngine {
    pub fn new() -> Self {
        Self
    }

    /// Pure mathematical evaluation of a set of agent reviews against a voting rule
    pub fn evaluate(&self, rule: &VotingRule, reviews: &[AgentReview]) -> ConsensusVerdict {
        let voters_count = reviews.len();
        if voters_count == 0 {
            return ConsensusVerdict {
                rule: rule.clone(),
                approved: false,
                voters_count: 0,
                approval_ratio: 0.0,
                average_score: 0.0,
                criterion_averages: HashMap::new(),
                borda_points: HashMap::new(),
                winning_option: None,
                reviews: Vec::new(),
                synthesis: "No reviews submitted for consensus evaluation.".to_string(),
                action_items: Vec::new(),
            };
        }

        let approved_count = reviews.iter().filter(|r| r.approved).count();
        let approval_ratio = approved_count as f32 / voters_count as f32;

        let total_score: u32 = reviews.iter().map(|r| r.overall_score as u32).sum();
        let average_score = total_score as f32 / voters_count as f32;

        // Criterion averages
        let mut criterion_sums: HashMap<String, (u32, usize)> = HashMap::new();
        for r in reviews {
            for (crit, &score) in &r.scores {
                let entry = criterion_sums.entry(crit.clone()).or_insert((0, 0));
                entry.0 += score as u32;
                entry.1 += 1;
            }
        }
        let mut criterion_averages: HashMap<String, f32> = HashMap::new();
        for (crit, (sum, count)) in criterion_sums {
            if count > 0 {
                criterion_averages.insert(crit, sum as f32 / count as f32);
            }
        }

        // Borda Count computation
        let mut borda_points: HashMap<String, usize> = HashMap::new();
        for r in reviews {
            let n = r.ranked_options.len();
            for (rank, option) in r.ranked_options.iter().enumerate() {
                // First choice gets n - 1 points, last gets 0
                let points = n.saturating_sub(rank + 1);
                *borda_points.entry(option.clone()).or_insert(0) += points;
            }
        }

        let winning_option = borda_points
            .iter()
            .max_by_key(|(_, &pts)| pts)
            .map(|(opt, _)| opt.clone());

        // Determine approval status based on rule
        let approved = match rule {
            VotingRule::Majority => approval_ratio > 0.5,
            VotingRule::Unanimous => approved_count == voters_count,
            VotingRule::SuperMajority(thresh) => approval_ratio >= *thresh,
            VotingRule::WeightedBorda => {
                // For Borda count, approval is met if average score >= 6.0 and majority approved
                approval_ratio > 0.5 && average_score >= 6.0
            }
        };

        // Extract action items & risks
        let mut action_items = Vec::new();
        for r in reviews {
            for risk in &r.identified_risks {
                let item = format!("[{}] Risk: {}", r.reviewer, risk);
                if !action_items.contains(&item) {
                    action_items.push(item);
                }
            }
            if !r.approved && !r.feedback.trim().is_empty() {
                let feedback_item = format!("[{}] Dissent feedback: {}", r.reviewer, r.feedback.trim());
                if !action_items.contains(&feedback_item) {
                    action_items.push(feedback_item);
                }
            }
        }

        // Construct synthesis narrative
        let synthesis = format!(
            "Consensus Verdict: {status} ({approved_count}/{voters_count} approvals, {pct:.1}%). \
            Average quality score: {avg:.2}/10. {details}",
            status = if approved { "APPROVED" } else { "REJECTED / REVISION REQUIRED" },
            pct = approval_ratio * 100.0,
            avg = average_score,
            details = if let Some(ref win) = winning_option {
                format!("Preferred option via Borda count: '{}'.", win)
            } else if approved {
                "All consensus thresholds met successfully.".to_string()
            } else {
                format!("Failed to satisfy {:?} threshold.", rule)
            }
        );

        ConsensusVerdict {
            rule: rule.clone(),
            approved,
            voters_count,
            approval_ratio,
            average_score,
            criterion_averages,
            borda_points,
            winning_option,
            reviews: reviews.to_vec(),
            synthesis,
            action_items,
        }
    }

    /// Orchestrates a live multi-agent consensus review of an artifact across all swarm members
    pub async fn run_consensus_review(
        &self,
        coordinator: &SwarmCoordinator,
        artifact: &str,
        rule: VotingRule,
        ctx: &EngineContext,
    ) -> Result<ConsensusVerdict> {
        let members = coordinator.members();
        if members.is_empty() {
            return Err(TagisanError::Execution(
                "Cannot execute consensus review with empty swarm".to_string(),
            ));
        }

        info!(
            "Initiating live consensus review across {} swarm members with rule {:?}",
            members.len(),
            rule
        );

        let review_prompt = format!(
            "You are conducting a strict peer review and consensus vote on the following artifact.\n\n\
            ```\n{}\n```\n\n\
            Carefully evaluate the artifact on correctness, security, maintainability, and performance.\n\
            Respond in valid JSON format matching this schema:\n\
            {{\n  \
              \"approved\": true|false,\n  \
              \"overall_score\": 1-10,\n  \
              \"correctness\": 1-10,\n  \
              \"security\": 1-10,\n  \
              \"maintainability\": 1-10,\n  \
              \"performance\": 1-10,\n  \
              \"feedback\": \"detailed review summary\",\n  \
              \"identified_risks\": [\"risk1\", \"risk2\"],\n  \
              \"ranked_options\": [\"opt1\", \"opt2\"]\n\
            }}",
            artifact.trim()
        );

        let broadcast_results = coordinator.execute_broadcast(&review_prompt, ctx).await?;
        let mut parsed_reviews = Vec::new();

        for (member_name, agent_result) in broadcast_results {
            let answer = agent_result.final_answer;
            let review = Self::parse_review_json(&member_name, &answer);
            parsed_reviews.push(review);
        }

        Ok(self.evaluate(&rule, &parsed_reviews))
    }

    /// Helper to parse structured review JSON from model responses with resilient fallback
    pub fn parse_review_json(reviewer: &str, text: &str) -> AgentReview {
        let json_str = if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                &text[start..=end]
            } else {
                text
            }
        } else {
            text
        };

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            let approved = val.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);
            let overall_score = val
                .get("overall_score")
                .and_then(|v| v.as_u64())
                .unwrap_or(if approved { 8 } else { 4 }) as u8;

            let mut review = AgentReview::new(reviewer, approved, overall_score);

            if let Some(c) = val.get("correctness").and_then(|v| v.as_u64()) {
                review = review.with_score("correctness", c as u8);
            }
            if let Some(s) = val.get("security").and_then(|v| v.as_u64()) {
                review = review.with_score("security", s as u8);
            }
            if let Some(m) = val.get("maintainability").and_then(|v| v.as_u64()) {
                review = review.with_score("maintainability", m as u8);
            }
            if let Some(p) = val.get("performance").and_then(|v| v.as_u64()) {
                review = review.with_score("performance", p as u8);
            }

            if let Some(fb) = val.get("feedback").and_then(|v| v.as_str()) {
                review = review.with_feedback(fb);
            } else {
                review = review.with_feedback(text);
            }

            if let Some(risks) = val.get("identified_risks").and_then(|v| v.as_array()) {
                for r in risks {
                    if let Some(r_str) = r.as_str() {
                        review = review.with_risk(r_str);
                    }
                }
            }

            if let Some(opts) = val.get("ranked_options").and_then(|v| v.as_array()) {
                let options: Vec<String> = opts
                    .iter()
                    .filter_map(|o| o.as_str().map(|s| s.to_string()))
                    .collect();
                review = review.with_ranked_options(options);
            }

            review
        } else {
            // Resilient text fallback: check for positive sentiment keywords
            let lower = text.to_lowercase();
            let approved = (lower.contains("approve") || lower.contains("pass") || lower.contains("lgtm"))
                && !lower.contains("reject")
                && !lower.contains("do not approve");
            let score = if approved { 7 } else { 4 };
            AgentReview::new(reviewer, approved, score).with_feedback(text)
        }
    }
}
