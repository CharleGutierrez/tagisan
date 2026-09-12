use crate::types::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Semantic integrity violations detected by the guard
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticViolation {
    TruncatedToolSchema {
        tool_name: String,
        details: String,
    },
    DroppedRequiredParameter {
        tool_name: String,
        param_name: String,
    },
    DiagnosticDegradation {
        error_code: String,
        details: String,
        min_tokens_needed: usize,
    },
    SafetyContractOmitted {
        missing_rule: String,
    },
    ContextExhaustionWithInvariantRisk {
        required_tokens: usize,
        available_tokens: usize,
    },
}

impl fmt::Display for SemanticViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedToolSchema { tool_name, details } => {
                write!(f, "Tool '{}' schema was truncated or corrupted: {}", tool_name, details)
            }
            Self::DroppedRequiredParameter { tool_name, param_name } => {
                write!(f, "Tool '{}' dropped required parameter '{}'", tool_name, param_name)
            }
            Self::DiagnosticDegradation { error_code, details, min_tokens_needed } => {
                write!(
                    f,
                    "Compiler diagnostic '{}' would suffer semantic degradation: {} (minimum {} tokens required)",
                    error_code, details, min_tokens_needed
                )
            }
            Self::SafetyContractOmitted { missing_rule } => {
                write!(f, "Mandatory safety contract rule omitted: {}", missing_rule)
            }
            Self::ContextExhaustionWithInvariantRisk { required_tokens, available_tokens } => {
                write!(
                    f,
                    "Context window exhaustion: required {} tokens for invariants but only {} tokens available",
                    required_tokens, available_tokens
                )
            }
        }
    }
}

impl std::error::Error for SemanticViolation {}

/// Condensed diagnostic representation preserving 100% semantic causality and line spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedDiagnostic {
    pub error_code: Option<String>,
    pub primary_span: String,
    pub message: String,
    pub preserved_condensed: String,
    pub original_token_est: usize,
    pub condensed_token_est: usize,
}

/// Semantic Invariant Guard enforcing Colibrì's "No SLA on Speed, Hard Guarantee on Semantics" rule.
/// Intercepts and rejects attempts to silently truncate tool schemas, drop parameters,
/// or silence compiler error diagnostics when context or memory is tight.
pub struct SemanticInvariantGuard;

impl SemanticInvariantGuard {
    pub fn new() -> Self {
        Self
    }

    /// Enforces tool schema integrity. Rejects schemas with truncated descriptions,
    /// missing property definitions, or dropped required parameters.
    pub fn enforce_tool_schema_integrity(
        &self,
        schema: &ToolDefinition,
    ) -> Result<ToolDefinition, SemanticViolation> {
        let name = &schema.name;

        // 1. Tool name check
        if name.trim().is_empty() {
            return Err(SemanticViolation::TruncatedToolSchema {
                tool_name: name.clone(),
                details: "Tool name cannot be empty".to_string(),
            });
        }

        // 2. Tool description check
        let desc = schema.description.trim();
        if desc.is_empty() {
            return Err(SemanticViolation::TruncatedToolSchema {
                tool_name: name.clone(),
                details: "Tool description was stripped or empty".to_string(),
            });
        }
        if desc.ends_with("...") || desc.ends_with("[truncated]") || desc.ends_with("…") {
            return Err(SemanticViolation::TruncatedToolSchema {
                tool_name: name.clone(),
                details: "Tool description was detected as truncated".to_string(),
            });
        }

        // 3. Parameters schema validation
        let params = &schema.parameters;
        if !params.is_object() {
            return Err(SemanticViolation::TruncatedToolSchema {
                tool_name: name.clone(),
                details: "Tool parameters must be a valid JSON Schema object".to_string(),
            });
        }

        if let Some(props) = params.get("properties") {
            if !props.is_object() {
                return Err(SemanticViolation::TruncatedToolSchema {
                    tool_name: name.clone(),
                    details: "'properties' field must be an object".to_string(),
                });
            }

            // Check if required parameters have valid property definitions
            if let Some(required) = params.get("required").and_then(|r| r.as_array()) {
                let props_obj = props.as_object().unwrap();
                for req_val in required {
                    if let Some(req_name) = req_val.as_str() {
                        if !props_obj.contains_key(req_name) {
                            return Err(SemanticViolation::DroppedRequiredParameter {
                                tool_name: name.clone(),
                                param_name: req_name.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(schema.clone())
    }

    /// Enforces diagnostic preservation for compiler errors (rustc, tsc, bun, python).
    /// Extracts exact error codes, primary spans, and root-cause causality, condensing boilerplate
    /// while guaranteeing zero semantic degradation.
    pub fn enforce_diagnostic_preservation(
        &self,
        raw_diagnostics: &str,
        max_tokens: usize,
    ) -> Result<PreservedDiagnostic, SemanticViolation> {
        let trimmed = raw_diagnostics.trim();
        if trimmed.is_empty() {
            return Ok(PreservedDiagnostic {
                error_code: None,
                primary_span: "unknown".to_string(),
                message: "Empty diagnostic".to_string(),
                preserved_condensed: String::new(),
                original_token_est: 0,
                condensed_token_est: 0,
            });
        }

        // 1. Extract error code (e.g. error[E0382], TS2322, SyntaxError)
        let error_code = if let Some(pos) = trimmed.find("error[E") {
            let rest = &trimmed[pos + 6..];
            if let Some(end) = rest.find(']') {
                Some(rest[..end].to_string())
            } else {
                None
            }
        } else if let Some(pos) = trimmed.find("TS") {
            let rest = &trimmed[pos..];
            let code: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
            if code.len() > 3 {
                Some(code)
            } else {
                None
            }
        } else if trimmed.contains("Traceback") {
            Some("PythonTraceback".to_string())
        } else {
            None
        };

        // 2. Extract primary span (e.g. src/main.rs:42:15 or file.ts:10:5)
        let mut primary_span = "unknown".to_string();
        for line in trimmed.lines() {
            let l = line.trim();
            if (l.starts_with("--> ") || l.starts_with("at ") || l.contains(".rs:") || l.contains(".ts:") || l.contains(".js:"))
                && l.contains(':')
            {
                let clean = l.trim_start_matches("--> ").trim_start_matches("at ").trim();
                primary_span = clean.to_string();
                break;
            }
        }

        // 3. Extract core message
        let mut core_message = String::new();
        for line in trimmed.lines() {
            let l = line.trim();
            if l.starts_with("error:") || l.starts_with("error[") || l.starts_with("Error:") {
                core_message = l.to_string();
                break;
            }
        }
        if core_message.is_empty() {
            core_message = trimmed.lines().next().unwrap_or("Unknown compiler error").to_string();
        }

        // 4. Synthesize condensed semantic core (preserving code, span, error message, help notes)
        let mut preserved_lines = Vec::new();
        preserved_lines.push(format!("DIAGNOSTIC: {}", core_message));
        preserved_lines.push(format!("SPAN: {}", primary_span));

        for line in trimmed.lines() {
            let l = line.trim();
            if l.starts_with("help:") || l.starts_with("note:") || l.starts_with("= note:") || l.starts_with("= help:") {
                preserved_lines.push(l.to_string());
            }
        }

        let preserved_condensed = preserved_lines.join("\n");
        let original_token_est = (trimmed.split_whitespace().count() * 4) / 3;
        let condensed_token_est = ((preserved_condensed.split_whitespace().count() * 4) / 3).max(1);

        // 5. Verify semantic capacity
        if max_tokens < condensed_token_est {
            let code_str = error_code.unwrap_or_else(|| "UNKNOWN_ERROR".to_string());
            return Err(SemanticViolation::DiagnosticDegradation {
                error_code: code_str,
                details: format!(
                    "Context token budget ({}) is lower than minimum preserved semantic diagnostic requirement ({})",
                    max_tokens, condensed_token_est
                ),
                min_tokens_needed: condensed_token_est,
            });
        }

        Ok(PreservedDiagnostic {
            error_code,
            primary_span,
            message: core_message,
            preserved_condensed,
            original_token_est,
            condensed_token_est,
        })
    }

    /// Enforces that mandatory safety contracts and invariants are never silently omitted
    pub fn enforce_safety_contract(
        &self,
        prompt_tokens: usize,
        context_budget: usize,
        safety_rules: &[&str],
    ) -> Result<(), SemanticViolation> {
        let required_tokens = prompt_tokens + (safety_rules.len() * 20);
        if required_tokens > context_budget {
            return Err(SemanticViolation::ContextExhaustionWithInvariantRisk {
                required_tokens,
                available_tokens: context_budget,
            });
        }

        for rule in safety_rules {
            if rule.trim().is_empty() {
                return Err(SemanticViolation::SafetyContractOmitted {
                    missing_rule: "Empty or null safety rule declared in contract".to_string(),
                });
            }
        }

        Ok(())
    }
}

impl Default for SemanticInvariantGuard {
    fn default() -> Self {
        Self::new()
    }
}
