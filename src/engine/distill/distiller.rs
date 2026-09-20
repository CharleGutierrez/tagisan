//! Distills episodic Reflexion entries and agent case-law into fine-tuning datasets.

use super::format::{AlpacaRecord, DpoRecord, ShareGptMessage, ShareGptRecord};
use crate::error::{Result, TagisanError};
use crate::tools::builtin::ReflexionEntry;
use std::path::Path;

/// Engine for distilling episodic case law into supervised fine-tuning and DPO datasets
pub struct ReflexionDistiller {
    pub entries: Vec<ReflexionEntry>,
}

impl ReflexionDistiller {
    pub fn new(entries: Vec<ReflexionEntry>) -> Self {
        Self { entries }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(TagisanError::Io)?;
        let entries: Vec<ReflexionEntry> = serde_json::from_str(&content)
            .map_err(|e| TagisanError::Execution(format!("Failed to parse reflexions JSON: {e}")))?;
        Ok(Self::new(entries))
    }

    /// Distill to Alpaca instruction format
    pub fn to_alpaca(&self) -> Vec<AlpacaRecord> {
        self.entries
            .iter()
            .map(|e| AlpacaRecord {
                instruction: "Analyze the following engineering error signature, determine its root cause, and synthesize a preventative invariant to ensure the failure is never repeated.".to_string(),
                input: format!(
                    "Error Signature:\n{}\nContext Tags: [{}]",
                    e.error_signature,
                    e.tags.join(", ")
                ),
                output: format!(
                    "Root Cause:\n{}\n\nPreventative Invariant:\n{}",
                    e.root_cause,
                    e.preventative_invariant
                ),
            })
            .collect()
    }

    /// Distill to ShareGPT multi-turn conversation format
    pub fn to_sharegpt(&self) -> Vec<ShareGptRecord> {
        self.entries
            .iter()
            .map(|e| ShareGptRecord {
                conversations: vec![
                    ShareGptMessage {
                        from: "human".to_string(),
                        value: format!(
                            "We encountered an issue during execution:\n{}\nHow should we prevent this?",
                            e.error_signature
                        ),
                    },
                    ShareGptMessage {
                        from: "gpt".to_string(),
                        value: format!(
                            "The root cause is:\n{}\n\nTo prevent this, enforce the following invariant:\n{}",
                            e.root_cause, e.preventative_invariant
                        ),
                    },
                ],
            })
            .collect()
    }

    /// Distill to Direct Preference Optimization (DPO) pairwise format
    pub fn to_dpo(&self) -> Vec<DpoRecord> {
        self.entries
            .iter()
            .map(|e| DpoRecord {
                prompt: format!(
                    "Diagnose and resolve the following system error:\n{}\nTags: {}",
                    e.error_signature,
                    e.tags.join(", ")
                ),
                chosen: format!(
                    "Correct Analysis: {}\nAction: Enforce preventative invariant: {}",
                    e.root_cause, e.preventative_invariant
                ),
                rejected: format!(
                    "Uninformed retry: Retrying the exact same command or code edit without adjusting for root cause: {}",
                    e.error_signature
                ),
            })
            .collect()
    }
}
