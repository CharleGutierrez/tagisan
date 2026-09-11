//! Eval runner — multi-model async evaluator with Borda count and majority aggregation.

use super::{
    dataset::{EvalCase, EvalDataset},
    scorer::{evaluate_criteria, threshold_pass, EvalReport, EvalScore},
};
use crate::engine::EngineContext;
use crate::error::Result;
use crate::types::CompletionRequest;
use tracing::{debug, info, warn};

/// Runs evaluation cases across multiple LLM models and aggregates results.
#[derive(Debug, Clone)]
pub struct EvalRunner {
    /// Model identifiers to evaluate (e.g. "claude-3-5-sonnet-20241022").
    pub models: Vec<String>,
    /// Aggregation strategy: "borda" or "majority".
    pub voting_rule: String,
    /// Pass/fail threshold on the 1.0–5.0 scoring scale.
    pub threshold: f32,
}

impl EvalRunner {
    pub fn new(
        models: Vec<String>,
        voting_rule: impl Into<String>,
        threshold: f32,
    ) -> Self {
        Self {
            models,
            voting_rule: voting_rule.into(),
            threshold,
        }
    }

    /// Resolve a model name prefix to the registered provider id.
    fn provider_for_model(model: &str) -> &'static str {
        let m = model.to_lowercase();
        if m.starts_with("claude") {
            "anthropic"
        } else if m.starts_with("gpt") || m.starts_with("o1") || m.starts_with("o3") {
            "openai"
        } else if m.starts_with("gemini") {
            "gemini"
        } else if m.starts_with("deepseek") {
            "deepseek"
        } else if m.starts_with("grok") || m.starts_with("xai") {
            "xai"
        } else {
            "ollama"
        }
    }

    /// Run the full evaluation across all cases and models. Returns an `EvalReport`.
    pub async fn run(
        &self,
        dataset: &EvalDataset,
        ctx: &EngineContext,
    ) -> Result<EvalReport> {
        let mut all_scores: Vec<EvalScore> = Vec::new();
        let cases = &dataset.cases;

        info!(
            dataset = %dataset.name,
            cases = cases.len(),
            models = ?self.models,
            rule = %self.voting_rule,
            "Starting eval run"
        );

        for case in cases {
            let case_scores = self.evaluate_case(case, ctx).await?;

            let aggregated = if self.voting_rule == "borda" {
                self.borda_aggregate(&case_scores)
            } else {
                self.majority_aggregate(&case_scores)
            };

            all_scores.extend(aggregated);
        }

        let total_cases = cases.len();
        let passed = all_scores.iter().filter(|s| s.passed).count();
        let failed = total_cases.saturating_sub(passed);
        let avg_score = if all_scores.is_empty() {
            0.0
        } else {
            all_scores.iter().map(|s| s.score).sum::<f32>() / all_scores.len() as f32
        };

        Ok(EvalReport {
            dataset_name: dataset.name.clone(),
            total_cases,
            passed,
            failed,
            avg_score,
            scores: all_scores,
            models_used: self.models.clone(),
        })
    }

    /// Evaluate a single case across all configured models.
    async fn evaluate_case(
        &self,
        case: &EvalCase,
        ctx: &EngineContext,
    ) -> Result<Vec<EvalScore>> {
        let mut scores = Vec::new();

        for model in &self.models {
            let provider_id = Self::provider_for_model(model);

            let provider = match ctx.get_provider(provider_id) {
                Ok(p) => p,
                Err(_) => {
                    warn!(
                        case_id = %case.id,
                        model = %model,
                        provider = provider_id,
                        "Provider not found — skipping model"
                    );
                    continue;
                }
            };

            let request = CompletionRequest::new(model.clone(), &case.prompt)
                .with_system(
                    "You are a helpful assistant. Answer concisely and accurately.",
                )
                .with_max_tokens(1024)
                .with_temperature(0.0);

            let response = match provider.complete(request).await {
                Ok(r) => r,
                Err(e) => {
                    warn!(
                        case_id = %case.id,
                        model = %model,
                        error = %e,
                        "LLM call failed — skipping"
                    );
                    continue;
                }
            };

            let answer = response.message.extract_text();

            let (raw_score, reasoning) = evaluate_criteria(&answer, case);
            let passed = threshold_pass(raw_score, self.threshold);

            debug!(
                case_id = %case.id,
                model = %model,
                score = raw_score,
                passed = passed,
                "Eval case scored"
            );

            scores.push(EvalScore {
                case_id: case.id.clone(),
                model: model.clone(),
                score: raw_score,
                reasoning,
                passed,
            });
        }

        Ok(scores)
    }

    /// Borda count: rank models by score per case, assign rank points, re-normalize.
    pub fn borda_aggregate(&self, scores: &[EvalScore]) -> Vec<EvalScore> {
        if scores.is_empty() {
            return Vec::new();
        }
        let n = scores.len();
        let mut ranked = scores.to_vec();
        ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        ranked
            .into_iter()
            .enumerate()
            .map(|(rank, mut s)| {
                // Borda points: n - rank, normalized back to 1.0–5.0 range
                let borda_pts = (n - rank) as f32;
                let borda_score = 1.0 + (borda_pts / n as f32) * 4.0;
                s.score = borda_score;
                s.passed = threshold_pass(borda_score, self.threshold);
                s.reasoning = format!(
                    "Borda rank {}/{}: {} pts → score {borda_score:.2}",
                    rank + 1,
                    n,
                    borda_pts
                );
                s
            })
            .collect()
    }

    /// Majority: return each model's raw score as-is (mean-based pass/fail).
    pub fn majority_aggregate(&self, scores: &[EvalScore]) -> Vec<EvalScore> {
        let mean = if scores.is_empty() {
            0.0
        } else {
            scores.iter().map(|s| s.score).sum::<f32>() / scores.len() as f32
        };

        scores
            .iter()
            .cloned()
            .map(|mut s| {
                s.passed = threshold_pass(mean, self.threshold);
                s
            })
            .collect()
    }
}
