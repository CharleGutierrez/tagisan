//! Eval dataset types — JSON-serializable evaluation cases and datasets.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single evaluation test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    pub id: String,
    pub prompt: String,
    pub reference_answer: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub expected_tools: Vec<String>,
}

/// A named collection of evaluation cases loaded from a JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalDataset {
    pub name: String,
    pub cases: Vec<EvalCase>,
}

impl EvalDataset {
    /// Load an `EvalDataset` from a JSON file on disk.
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "name": "my_benchmark",
    ///   "cases": [
    ///     { "id": "q1", "prompt": "...", "reference_answer": "...", "tags": ["math"] }
    ///   ]
    /// }
    /// ```
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            TagisanError::Execution(format!("Failed to read eval dataset {:?}: {e}", path))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            TagisanError::Execution(format!("Failed to parse eval dataset {:?}: {e}", path))
        })
    }

    /// Filter cases to only those whose tag list contains at least one of the given tags.
    /// Returns all cases if `tags` is empty.
    pub fn filter_by_tags<'a>(&'a self, tags: &[String]) -> Vec<&'a EvalCase> {
        if tags.is_empty() {
            return self.cases.iter().collect();
        }
        self.cases
            .iter()
            .filter(|c| c.tags.iter().any(|t| tags.contains(t)))
            .collect()
    }
}
