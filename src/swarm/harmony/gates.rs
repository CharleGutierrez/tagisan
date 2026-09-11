use serde::{Deserialize, Serialize};
use crate::ecc::AgentShieldScanner;
use crate::swarm::harmony::types::RoleArtifact;

/// Outcome of a validation gate check between stages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateResult {
    /// Validation passed cleanly. Proceed to next stage.
    Pass,
    /// Validation failed but can be self-corrected by re-prompting the role with a critique.
    RetryWithCritique { critique: String },
    /// Critical unrecoverable validation failure.
    HardFailure { reason: String },
}

impl GateResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, GateResult::Pass)
    }
}

/// A validation gate positioned between stages in the harmony assembly line.
pub trait ValidationGate: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, artifact: &RoleArtifact) -> GateResult;
}

/// Validates that an artifact contains valid code fences and structural syntax.
pub struct SyntaxValidationGate {
    pub expected_language: Option<String>,
    pub require_code_block: bool,
}

impl SyntaxValidationGate {
    pub fn new(expected_language: Option<String>, require_code_block: bool) -> Self {
        Self {
            expected_language,
            require_code_block,
        }
    }

    pub fn for_rust() -> Self {
        Self::new(Some("rust".to_string()), true)
    }

    pub fn permissive() -> Self {
        Self::new(None, false)
    }
}

impl ValidationGate for SyntaxValidationGate {
    fn name(&self) -> &'static str {
        "Syntax & AST Validation Gate"
    }

    fn validate(&self, artifact: &RoleArtifact) -> GateResult {
        if self.require_code_block && artifact.code_blocks.is_empty() {
            let lang_hint = self.expected_language.as_deref().unwrap_or("code");
            return GateResult::RetryWithCritique {
                critique: format!(
                    "No code block was detected in your generation. You MUST enclose your code in markdown code fences, e.g.:\n```{lang_hint}\n// your code here\n```"
                ),
            };
        }

        // Structural balance check for C-style languages (Rust, TypeScript, C++)
        for block in &artifact.code_blocks {
            let lang = block.language.to_lowercase();
            if lang == "rust" || lang == "rs" || lang == "typescript" || lang == "ts" || lang == "c" || lang == "cpp" {
                let mut open_braces = 0i32;
                let mut open_parens = 0i32;
                let mut in_string = false;
                let mut escape_next = false;

                for ch in block.code.chars() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }
                    if ch == '\\' {
                        escape_next = true;
                        continue;
                    }
                    if ch == '"' {
                        in_string = !in_string;
                        continue;
                    }
                    if !in_string {
                        match ch {
                            '{' => open_braces += 1,
                            '}' => open_braces -= 1,
                            '(' => open_parens += 1,
                            ')' => open_parens -= 1,
                            _ => {}
                        }
                    }
                }

                if open_braces != 0 {
                    return GateResult::RetryWithCritique {
                        critique: format!(
                            "Unbalanced curly braces in your {lang} code (difference: {open_braces}). Please ensure every opened '{{' has a matching '}}'."
                        ),
                    };
                }
                if open_parens != 0 {
                    return GateResult::RetryWithCritique {
                        critique: format!(
                            "Unbalanced parentheses in your {lang} code (difference: {open_parens}). Please ensure every opened '(' has a matching ')'."
                        ),
                    };
                }
            }
        }

        GateResult::Pass
    }
}

/// Validates that generated code does not contain malicious or destructive payloads.
#[derive(Debug, Default, Clone)]
pub struct AgentShieldSecurityGate;

impl AgentShieldSecurityGate {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationGate for AgentShieldSecurityGate {
    fn name(&self) -> &'static str {
        "AgentShield Security Gate"
    }

    fn validate(&self, artifact: &RoleArtifact) -> GateResult {
        // 1. Language-specific code block audits
        for block in &artifact.code_blocks {
            let lang = block.language.to_lowercase();
            if lang == "python" || lang == "py" {
                let v = AgentShieldScanner::scan_python_code(&block.code);
                if let crate::ecc::AgentShieldVerdict::Block { ref reason, threat_level } = v {
                    return GateResult::RetryWithCritique {
                        critique: format!(
                            "AgentShield Security Rule Triggered in Python [{:?}]: {reason}. Rewrite without this dangerous operation.",
                            threat_level
                        ),
                    };
                }
            } else if lang == "perl" || lang == "pl" {
                let v = AgentShieldScanner::scan_perl_code(&block.code);
                if let crate::ecc::AgentShieldVerdict::Block { ref reason, threat_level } = v {
                    return GateResult::RetryWithCritique {
                        critique: format!(
                            "AgentShield Security Rule Triggered in Perl [{:?}]: {reason}. Rewrite without this dangerous operation.",
                            threat_level
                        ),
                    };
                }
            } else if lang == "javascript" || lang == "js" || lang == "typescript" || lang == "ts" {
                let v = AgentShieldScanner::scan_code(&block.code);
                if let crate::ecc::AgentShieldVerdict::Block { ref reason, threat_level } = v {
                    return GateResult::RetryWithCritique {
                        critique: format!(
                            "AgentShield Security Rule Triggered in JS/TS [{:?}]: {reason}. Rewrite without this dangerous operation.",
                            threat_level
                        ),
                    };
                }
            }
        }

        // 2. Universal scan for destructive operations, raw storage access, and fork bombs
        let texts = std::iter::once(&artifact.raw_output)
            .chain(artifact.code_blocks.iter().map(|b| &b.code));

        for text in texts {
            let lower = text.to_lowercase();
            if (lower.contains("rm") && (lower.contains("-rf") || lower.contains("-fr")) && (lower.contains('/') || lower.contains("\"/\"") || lower.contains("'/'")))
                || lower.contains("rm -rf")
                || lower.contains("mkfs")
                || lower.contains(":(){:|:&};:")
                || lower.contains(":(){ :|:& };:")
                || lower.contains("forkbomb")
                || lower.contains("dd if=/dev")
            {
                return GateResult::RetryWithCritique {
                    critique: "AgentShield Security Rule Triggered [Critical]: Destructive filesystem deletion, raw device overwrite, or fork bomb pattern detected. Rewrite your code to eliminate this dangerous operation immediately.".to_string(),
                };
            }
        }

        GateResult::Pass
    }
}
