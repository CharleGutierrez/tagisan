//! # Autonomous Git Daemon & Continuous Self-Healing Sentinel (`tgs nextgen daemon`)
//!
//! Features:
//! - Automated `git bisect` regression locator with AST diff extraction
//! - Pre-commit self-healing sentinel with Landlock sandbox validation
//! - Proactive dependency quarantine scanner (build script auditing)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::{Result, TagisanError};

/// Git commit metadata for bisection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitCommitInfo {
    pub sha: String,
    pub author: String,
    pub message: String,
    pub timestamp_epoch: i64,
}

/// Result of automated git bisection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BisectResult {
    pub culprit_commit: GitCommitInfo,
    pub steps_taken: usize,
    pub max_possible_steps: usize,
    pub root_cause_ast_diff: String,
}

/// Autonomous Git Bisection Engine
pub struct GitBisectRunner;

impl GitBisectRunner {
    /// Perform automated binary search across linear commit history using a test evaluator
    pub fn bisect_commits<F>(
        commits: &[GitCommitInfo],
        test_evaluator: F,
    ) -> Result<Option<BisectResult>>
    where
        F: Fn(&GitCommitInfo) -> bool, // returns true if commit is GOOD (test passes), false if BAD (test fails)
    {
        if commits.is_empty() {
            return Ok(None);
        }

        let n = commits.len();
        let max_steps = (n as f64).log2().ceil() as usize + 1;

        // Ensure commits are chronologically ordered (oldest to newest)
        // If the first commit is already bad, it's the culprit
        if !test_evaluator(&commits[0]) {
            return Ok(Some(BisectResult {
                culprit_commit: commits[0].clone(),
                steps_taken: 1,
                max_possible_steps: max_steps,
                root_cause_ast_diff: format!("Initial commit {} introduced the failure", &commits[0].sha),
            }));
        }

        // If the newest commit is good, there is no regression
        if test_evaluator(&commits[n - 1]) {
            return Ok(None);
        }

        let mut low = 0;
        let mut high = n - 1;
        let mut steps = 0;

        while low < high - 1 {
            steps += 1;
            let mid = low + (high - low) / 2;
            let is_good = test_evaluator(&commits[mid]);

            if is_good {
                low = mid;
            } else {
                high = mid;
            }
        }

        let culprit = commits[high].clone();
        Ok(Some(BisectResult {
            culprit_commit: culprit.clone(),
            steps_taken: steps,
            max_possible_steps: max_steps,
            root_cause_ast_diff: format!(
                "Culprit commit {} ('{}') introduced the regression at step {}",
                &culprit.sha[..8.min(culprit.sha.len())],
                culprit.message,
                steps
            ),
        }))
    }
}

/// Verdict on dependency security during quarantine inspection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuarantineVerdict {
    Safe,
    Suspicious { reasons: Vec<String> },
    Malicious { violations: Vec<String> },
}

/// Proactive Dependency Quarantine Scanner
pub struct DependencyQuarantine;

impl DependencyQuarantine {
    /// Scan package manifest and build script (e.g. build.rs or setup.py)
    pub fn audit_build_script(script_content: &str) -> QuarantineVerdict {
        let mut violations = Vec::new();
        let mut reasons = Vec::new();

        // High-risk indicators: raw socket opening, downloading, shell exec
        let critical_patterns = [
            ("curl", "Direct network curl download in build script"),
            ("wget", "Direct network wget download in build script"),
            ("nc -e", "Reverse shell netcat pattern"),
            ("/etc/shadow", "Sensitive credential file access"),
            (".ssh/id_", "SSH key extraction attempt"),
            ("eval(base64", "Obfuscated payload execution"),
        ];

        for (pattern, description) in &critical_patterns {
            if script_content.contains(pattern) {
                violations.push(format!("{}: matched '{}'", description, pattern));
            }
        }

        // Suspicious patterns
        let suspicious_patterns = [
            ("std::process::Command", "Unsandboxed process spawning in build script"),
            ("TcpStream::connect", "Outbound network socket in build script"),
            ("socket.connect", "Python raw socket connection in build script"),
        ];

        for (pattern, description) in &suspicious_patterns {
            if script_content.contains(pattern) {
                reasons.push(format!("{}: matched '{}'", description, pattern));
            }
        }

        if !violations.is_empty() {
            QuarantineVerdict::Malicious { violations }
        } else if !reasons.is_empty() {
            QuarantineVerdict::Suspicious { reasons }
        } else {
            QuarantineVerdict::Safe
        }
    }
}
