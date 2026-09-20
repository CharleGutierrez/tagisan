//! Dataset formats for fine-tuning: Alpaca, ShareGPT, and DPO.

use serde::{Deserialize, Serialize};

/// Alpaca instruction fine-tuning record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlpacaRecord {
    pub instruction: String,
    pub input: String,
    pub output: String,
}

/// Message in a ShareGPT conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShareGptMessage {
    pub from: String, // "human" or "gpt"
    pub value: String,
}

/// ShareGPT multi-turn conversation format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShareGptRecord {
    pub conversations: Vec<ShareGptMessage>,
}

/// Direct Preference Optimization (DPO) pairwise record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DpoRecord {
    pub prompt: String,
    pub chosen: String,
    pub rejected: String,
}

/// Export format selector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistillationFormat {
    Alpaca,
    ShareGpt,
    Dpo,
}

impl DistillationFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "alpaca" => Some(Self::Alpaca),
            "sharegpt" => Some(Self::ShareGpt),
            "dpo" => Some(Self::Dpo),
            _ => None,
        }
    }
}
