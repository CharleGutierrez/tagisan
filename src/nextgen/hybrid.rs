//! # Speculative Hybrid Orchestration Engine (`tgs nextgen hybrid`)
//!
//! Blends local high-speed NPU/GGUF drafting (100+ tokens/sec) with cloud frontier
//! reasoning (Gemini 2.5/3 Pro, DeepSeek Reasoner) using the Lakandiwa Epistemic Entropy Gate.
//!
//! Features:
//! - Shannon Entropy calculation: $H(X) = - \sum p_i \log_2 p_i$
//! - Top-1 vs Top-2 Logprob Margin ($\Delta = p_{(1)} - p_{(2)}$)
//! - Thermal and Battery Energy Governor
//! - Speculative draft validation with automatic fallback

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Candidate token prediction with associated probability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateToken {
    pub token: String,
    pub probability: f64, // Normalized in (0.0, 1.0]
    pub logprob: f64,
}

/// Token distribution emitted by local speculative draft model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftDistribution {
    pub candidates: Vec<CandidateToken>,
}

impl DraftDistribution {
    pub fn new(mut candidates: Vec<CandidateToken>) -> Self {
        // Sort descending by probability
        candidates.sort_by(|a, b| b.probability.partial_cmp(&a.probability).unwrap_or(std::cmp::Ordering::Equal));
        Self { candidates }
    }

    /// Calculate Shannon Entropy in bits
    pub fn shannon_entropy(&self) -> f64 {
        let mut entropy = 0.0;
        for c in &self.candidates {
            if c.probability > 1e-9 {
                entropy -= c.probability * c.probability.log2();
            }
        }
        entropy
    }

    /// Calculate Top-1 vs Top-2 probability margin
    pub fn top_margin(&self) -> f64 {
        if self.candidates.len() >= 2 {
            self.candidates[0].probability - self.candidates[1].probability
        } else if self.candidates.len() == 1 {
            self.candidates[0].probability
        } else {
            0.0
        }
    }

    /// Calculate Perplexity: 2^H(X)
    pub fn perplexity(&self) -> f64 {
        2.0_f64.powf(self.shannon_entropy())
    }
}

/// Lakandiwa Epistemic Entropy Gate Decision
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LakandiwaVerdict {
    /// Local draft accepted with high confidence
    AcceptLocalDraft {
        token: String,
        confidence: f64,
        entropy: f64,
    },
    /// High epistemic uncertainty; escalate to frontier reasoner
    EscalateToFrontier {
        entropy: f64,
        margin: f64,
        reason: String,
    },
    /// Throttled by thermal governor or battery constraints
    ThrottledToCloud {
        temp_celsius: f64,
        battery_pct: Option<u8>,
    },
}

/// Configuration parameters for the Lakandiwa Entropy Gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyGateConfig {
    /// Maximum allowed Shannon entropy before escalating (default: 1.75 bits)
    pub max_entropy_bits: f64,
    /// Minimum margin between top-1 and top-2 candidates (default: 0.18)
    pub min_margin: f64,
    /// Thermal throttle temperature threshold in Celsius (default: 75.0 C)
    pub thermal_throttle_celsius: f64,
    /// Minimum battery percentage before disabling local NPU/GPU drafting (default: 20%)
    pub min_battery_pct: u8,
}

impl Default for EntropyGateConfig {
    fn default() -> Self {
        Self {
            max_entropy_bits: 1.75,
            min_margin: 0.18,
            thermal_throttle_celsius: 75.0,
            min_battery_pct: 20,
        }
    }
}

/// System Thermal & Energy Sensor
pub struct ThermalGovernor;

impl ThermalGovernor {
    /// Read maximum system temperature from Linux thermal zones in Celsius
    pub fn read_max_temperature() -> f64 {
        let mut max_temp = 40.0; // Reasonable default
        let thermal_dir = Path::new("/sys/class/thermal");
        if let Ok(entries) = fs::read_dir(thermal_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("thermal_zone") {
                    let temp_path = entry.path().join("temp");
                    if let Ok(content) = fs::read_to_string(temp_path) {
                        if let Ok(millidegrees) = content.trim().parse::<f64>() {
                            let deg = millidegrees / 1000.0;
                            if deg > max_temp && deg < 120.0 {
                                max_temp = deg;
                            }
                        }
                    }
                }
            }
        }
        max_temp
    }

    /// Read primary battery percentage if running on laptop
    pub fn read_battery_percentage() -> Option<u8> {
        let power_dir = Path::new("/sys/class/power_supply");
        if let Ok(entries) = fs::read_dir(power_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("BAT") {
                    let cap_path = entry.path().join("capacity");
                    if let Ok(content) = fs::read_to_string(cap_path) {
                        if let Ok(pct) = content.trim().parse::<u8>() {
                            return Some(pct);
                        }
                    }
                }
            }
        }
        None
    }
}

/// Speculative Hybrid Execution Engine
pub struct SpeculativeHybridEngine {
    pub config: EntropyGateConfig,
}

impl SpeculativeHybridEngine {
    pub fn new(config: EntropyGateConfig) -> Self {
        Self { config }
    }

    /// Evaluate draft distribution through Lakandiwa Gate
    pub fn evaluate_draft(&self, dist: &DraftDistribution) -> LakandiwaVerdict {
        // 1. Check thermal and battery governor
        let current_temp = ThermalGovernor::read_max_temperature();
        let battery_pct = ThermalGovernor::read_battery_percentage();

        if current_temp >= self.config.thermal_throttle_celsius {
            return LakandiwaVerdict::ThrottledToCloud {
                temp_celsius: current_temp,
                battery_pct,
            };
        }

        if let Some(pct) = battery_pct {
            if pct < self.config.min_battery_pct {
                return LakandiwaVerdict::ThrottledToCloud {
                    temp_celsius: current_temp,
                    battery_pct: Some(pct),
                };
            }
        }

        if dist.candidates.is_empty() {
            return LakandiwaVerdict::EscalateToFrontier {
                entropy: 0.0,
                margin: 0.0,
                reason: "Empty draft distribution".to_string(),
            };
        }

        // 2. Compute information theoretic metrics
        let entropy = dist.shannon_entropy();
        let margin = dist.top_margin();
        let top_candidate = &dist.candidates[0];

        // 3. Evaluate Lakandiwa gating condition
        if entropy > self.config.max_entropy_bits {
            return LakandiwaVerdict::EscalateToFrontier {
                entropy,
                margin,
                reason: format!(
                    "High Shannon entropy ({:.2} bits > {:.2} limit) indicates branching uncertainty",
                    entropy, self.config.max_entropy_bits
                ),
            };
        }

        if margin < self.config.min_margin {
            return LakandiwaVerdict::EscalateToFrontier {
                entropy,
                margin,
                reason: format!(
                    "Top-1 vs Top-2 margin too narrow ({:.3} < {:.3} limit)",
                    margin, self.config.min_margin
                ),
            };
        }

        LakandiwaVerdict::AcceptLocalDraft {
            token: top_candidate.token.clone(),
            confidence: top_candidate.probability,
            entropy,
        }
    }

    /// Evaluate a speculative draft sequence (multi-token lookahead verification)
    pub fn evaluate_draft_sequence(&self, draft_sequence: &[DraftDistribution]) -> SpeculativeBatchResult {
        let mut accepted_tokens = Vec::new();
        let mut verdicts = Vec::new();
        let mut first_escalation = None;
        let mut total_entropy = 0.0;

        for (idx, dist) in draft_sequence.iter().enumerate() {
            let verdict = self.evaluate_draft(dist);
            total_entropy += dist.shannon_entropy();
            verdicts.push(verdict.clone());

            match verdict {
                LakandiwaVerdict::AcceptLocalDraft { token, .. } => {
                    accepted_tokens.push(token);
                }
                LakandiwaVerdict::EscalateToFrontier { .. } | LakandiwaVerdict::ThrottledToCloud { .. } => {
                    if first_escalation.is_none() {
                        first_escalation = Some(idx);
                    }
                    // In speculative decoding, once a draft token is rejected/escalated, subsequent drafts must be invalidated
                    break;
                }
            }
        }

        let total_eval = draft_sequence.len();
        let accepted_len = accepted_tokens.len();
        let mean_entropy = if total_eval > 0 {
            total_entropy / (total_eval as f64)
        } else {
            0.0
        };

        let acceptance_rate = if total_eval > 0 {
            (accepted_len as f64) / (total_eval as f64)
        } else {
            0.0
        };

        SpeculativeBatchResult {
            accepted_tokens,
            total_evaluated: total_eval,
            first_escalation_index: first_escalation,
            mean_entropy,
            acceptance_rate,
            verdicts,
        }
    }
}

/// Result of evaluating a multi-token speculative draft batch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeculativeBatchResult {
    pub accepted_tokens: Vec<String>,
    pub total_evaluated: usize,
    pub first_escalation_index: Option<usize>,
    pub mean_entropy: f64,
    pub acceptance_rate: f64,
    pub verdicts: Vec<LakandiwaVerdict>,
}

/// Real-time metrics tracking speculative swarm acceptance rates and speedup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SpeculativeSwarmAcceptanceMetrics {
    pub total_draft_tokens: u64,
    pub accepted_draft_tokens: u64,
    pub rejected_draft_tokens: u64,
    pub cloud_escalations: u64,
    pub thermal_throttles: u64,
}

impl SpeculativeSwarmAcceptanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the outcome of a Lakandiwa gating decision
    pub fn record_verdict(&mut self, verdict: &LakandiwaVerdict) {
        self.total_draft_tokens += 1;
        match verdict {
            LakandiwaVerdict::AcceptLocalDraft { .. } => {
                self.accepted_draft_tokens += 1;
            }
            LakandiwaVerdict::EscalateToFrontier { .. } => {
                self.rejected_draft_tokens += 1;
                self.cloud_escalations += 1;
            }
            LakandiwaVerdict::ThrottledToCloud { .. } => {
                self.rejected_draft_tokens += 1;
                self.thermal_throttles += 1;
            }
        }
    }

    /// Calculate acceptance rate: alpha = accepted / total
    pub fn acceptance_rate(&self) -> f64 {
        if self.total_draft_tokens == 0 {
            0.0
        } else {
            (self.accepted_draft_tokens as f64) / (self.total_draft_tokens as f64)
        }
    }

    /// Calculate theoretical speedup: S = 1 / ((1 - alpha) + alpha / gamma)
    /// where gamma is the local vs cloud speed ratio (e.g. 5.0 for 5x faster local drafting)
    pub fn effective_speedup(&self, gamma: f64) -> f64 {
        let alpha = self.acceptance_rate();
        if gamma <= 1.0 || alpha <= 0.0 {
            1.0
        } else {
            let denominator = (1.0 - alpha) + (alpha / gamma);
            if denominator > 0.0 {
                (1.0 / denominator).min(gamma)
            } else {
                1.0
            }
        }
    }
}

