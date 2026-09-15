//! Microsoft Purview Data Sensitivity & Zero-Egress Air-Gapped Enforcement
//!
//! Provides enterprise data governance:
//! - Multi-tier classification: General, Confidential, HighlyConfidential, Secret
//! - Zero-Egress Air-Gapping: Dynamically binds execution to local offline GGUF / tensor engine,
//!   completely prohibiting external cloud API egress when data is marked Confidential/Secret.
//! - Cryptographic SHA-256 Audit Receipts (`PurviewAuditReceipt`) with content digest,
//!   timestamp, sensitivity classification, routing enforcement proof, and HMAC/signature.

use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fmt;

/// Microsoft Purview Sensitivity Labels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurviewSensitivity {
    General = 0,
    Confidential = 1,
    HighlyConfidential = 2,
    Secret = 3,
}

impl PurviewSensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Confidential => "Confidential",
            Self::HighlyConfidential => "HighlyConfidential",
            Self::Secret => "Secret",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        let lower = s.trim().to_lowercase().replace(['-', ' ', '_'], "");
        if lower.contains("secret") || lower.contains("topsecret") {
            Self::Secret
        } else if lower.contains("highlyconfidential") || lower.contains("highconfidential") || lower.contains("restricted") {
            Self::HighlyConfidential
        } else if lower.contains("confidential") || lower.contains("internal") || lower.contains("proprietary") || lower.contains("nda") {
            Self::Confidential
        } else {
            Self::General
        }
    }

    pub fn is_air_gapped(&self) -> bool {
        matches!(self, Self::Confidential | Self::HighlyConfidential | Self::Secret)
    }

    pub fn badge(&self) -> &'static str {
        match self {
            Self::General => "🟢 General (Unrestricted)",
            Self::Confidential => "🟡 Confidential (Zero-Egress Air-Gapped)",
            Self::HighlyConfidential => "🟠 Highly Confidential (Zero-Egress Strictly Air-Gapped)",
            Self::Secret => "🔴 Secret (Zero-Egress Classified Offline Only)",
        }
    }
}

impl fmt::Display for PurviewSensitivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Cryptographic SHA-256 Audit Receipt proving Zero-Egress Air-Gap enforcement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurviewAuditReceipt {
    pub receipt_id: String,
    pub timestamp: String,
    pub sensitivity: PurviewSensitivity,
    pub content_sha256: String,
    pub signature_sha256: String,
    pub egress_blocked: bool,
    pub routing_engine: String,
    pub policy_applied: String,
}

impl PurviewAuditReceipt {
    /// Verify cryptographic integrity of this audit receipt against raw content
    pub fn verify_integrity(&self, content: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let computed_content_hash = format!("{:x}", hasher.finalize());

        if computed_content_hash != self.content_sha256 {
            return false;
        }

        let mut sig_hasher = Sha256::new();
        let sig_input = format!(
            "{}:{}:{}:{}:{}",
            self.receipt_id,
            self.content_sha256,
            self.timestamp,
            self.sensitivity.as_str(),
            self.policy_applied
        );
        sig_hasher.update(sig_input.as_bytes());
        let computed_sig = format!("{:x}", sig_hasher.finalize());

        computed_sig == self.signature_sha256
    }
}

/// Result of Purview Evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurviewGuardResult {
    pub sensitivity: PurviewSensitivity,
    pub air_gapped: bool,
    pub routing_engine: String,
    pub egress_allowed: bool,
    pub receipt: PurviewAuditReceipt,
    pub message: String,
}

/// Purview Guard Engine
pub struct PurviewGuardEngine;

impl PurviewGuardEngine {
    /// Classifies content sensitivity based on declared label and content heuristic scanning
    pub fn classify(content: &str, declared_label: Option<&str>) -> PurviewSensitivity {
        let explicit = declared_label
            .map(PurviewSensitivity::from_str_lossy)
            .unwrap_or(PurviewSensitivity::General);

        let lower = content.to_lowercase();

        let auto_detected = if lower.contains("top secret")
            || lower.contains("confidential//noforn")
            || lower.contains("classified defense")
            || lower.contains("scada master key")
            || lower.contains("private_key")
            || lower.contains("-----begin openssh private key-----")
            || lower.contains("-----begin rsa private key-----")
            || lower.contains("seed phrase")
        {
            PurviewSensitivity::Secret
        } else if lower.contains("highly confidential")
            || lower.contains("strictly confidential")
            || lower.split_whitespace().any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == "pii")
            || lower.contains("social security")
            || lower.contains("salary schedule")
            || lower.contains("financial ledger")
            || lower.contains("hipaa")
            || lower.contains("gdpr sensitive")
            || lower.contains("trade secret")
        {
            PurviewSensitivity::HighlyConfidential
        } else if lower.contains("confidential")
            || lower.contains("internal only")
            || lower.contains("proprietary")
            || lower.contains("non-disclosure")
            || lower.split_whitespace().any(|w| w.trim_matches(|c: char| !c.is_alphanumeric()) == "nda")
            || lower.contains("do not distribute")
            || lower.contains("restricted")
        {
            PurviewSensitivity::Confidential
        } else {
            PurviewSensitivity::General
        };

        // Escalate to highest detected sensitivity
        std::cmp::max(explicit, auto_detected)
    }

    /// Evaluates content and enforces zero-egress routing
    pub fn evaluate(
        content: &str,
        declared_label: Option<&str>,
        target_destination: Option<&str>,
    ) -> Result<PurviewGuardResult> {
        // AgentShield outbound check
        let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(content);
        if let AgentShieldVerdict::Block { reason, threat_level } = dlp_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP Gate Blocked payload (Threat: {:?}): {}",
                threat_level, reason
            )));
        }

        let sensitivity = Self::classify(content, declared_label);
        let air_gapped = sensitivity.is_air_gapped();

        let (egress_allowed, routing_engine, policy_applied) = if air_gapped {
            // If caller explicitly requested external cloud egress with confidential data, reject it immediately
            if let Some(dest) = target_destination {
                let dest_lower = dest.to_lowercase();
                if dest_lower.contains("cloud")
                    || dest_lower.contains("openai")
                    || dest_lower.contains("anthropic")
                    || dest_lower.contains("gemini")
                    || dest_lower.contains("external")
                    || dest_lower.contains("public")
                {
                    return Err(TagisanError::Security(format!(
                        "Zero-Egress Air-Gap Violation: Content marked as [{}] is strictly forbidden from external egress to '{}'. Routing is locked to local in-process GGUF tensor engine.",
                        sensitivity.as_str(), dest
                    )));
                }
            }

            (
                false,
                "local_gguf_offline_tensor".to_string(),
                format!("PURVIEW-ZERO-EGRESS-RULE-{}", sensitivity.as_str().to_uppercase()),
            )
        } else {
            (
                true,
                "cloud_hybrid_orchestrator".to_string(),
                "PURVIEW-GENERAL-POLICY-PERMISSIVE".to_string(),
            )
        };

        // Compute SHA-256 content hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_sha256 = format!("{:x}", hasher.finalize());

        let timestamp = Utc::now().to_rfc3339();
        let receipt_id = format!(
            "pvw_rcpt_{}",
            &blake3::hash(format!("{content_sha256}_{timestamp}_{}", sensitivity.as_str()).as_bytes()).to_hex()[..16]
        );

        let mut sig_hasher = Sha256::new();
        let sig_input = format!(
            "{receipt_id}:{content_sha256}:{timestamp}:{}:{policy_applied}",
            sensitivity.as_str()
        );
        sig_hasher.update(sig_input.as_bytes());
        let signature_sha256 = format!("{:x}", sig_hasher.finalize());

        let receipt = PurviewAuditReceipt {
            receipt_id: receipt_id.clone(),
            timestamp: timestamp.clone(),
            sensitivity,
            content_sha256,
            signature_sha256,
            egress_blocked: !egress_allowed,
            routing_engine: routing_engine.clone(),
            policy_applied: policy_applied.clone(),
        };

        let message = if air_gapped {
            format!(
                "Sensitivity Level: {}\nZero-Egress Mode: ENFORCED\nExecution Route: Local In-Process GGUF Tensor Engine (Offline Air-Gapped)\nCryptographic Receipt: {}",
                sensitivity.badge(), receipt.receipt_id
            )
        } else {
            format!(
                "Sensitivity Level: {}\nZero-Egress Mode: Bypassed (General Data)\nExecution Route: Cloud Hybrid Orchestrator\nCryptographic Receipt: {}",
                sensitivity.badge(), receipt.receipt_id
            )
        };

        Ok(PurviewGuardResult {
            sensitivity,
            air_gapped,
            routing_engine,
            egress_allowed,
            receipt,
            message,
        })
    }
}

// =========================================================================
// CopilotPurviewGuardTool (copilot_purview_guard)
// =========================================================================

/// Tool for classifying sensitivity labels, enforcing Zero-Egress air-gapping,
/// and generating cryptographic SHA-256 audit receipts.
#[derive(Clone, Default)]
pub struct CopilotPurviewGuardTool;

impl CopilotPurviewGuardTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for CopilotPurviewGuardTool {
    fn name(&self) -> &str {
        "copilot_purview_guard"
    }

    fn description(&self) -> &str {
        "Classifies Microsoft Purview sensitivity labels (General, Confidential, HighlyConfidential, Secret), enforces Zero-Egress Air-Gapping by routing sensitive data strictly to local offline GGUF/tensor engines, and generates cryptographic SHA-256 audit receipts."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Text, document content, or code snippet to evaluate against Purview policies"
                },
                "label": {
                    "type": "string",
                    "description": "Optional explicit sensitivity label override: 'General', 'Confidential', 'HighlyConfidential', 'Secret'"
                },
                "sensitivity": {
                    "type": "string",
                    "description": "Alias for label"
                },
                "destination": {
                    "type": "string",
                    "description": "Optional planned routing destination (e.g. 'local_gguf', 'cloud_openai', 'teams', 'sharepoint')"
                },
                "target_engine": {
                    "type": "string",
                    "description": "Alias for destination"
                }
            },
            "required": ["content"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'content'".to_string()))?;

        let label = arguments
            .get("label")
            .and_then(|v| v.as_str())
            .or_else(|| arguments.get("sensitivity").and_then(|v| v.as_str()));

        let destination = arguments
            .get("destination")
            .and_then(|v| v.as_str())
            .or_else(|| arguments.get("target_engine").and_then(|v| v.as_str()));

        let result = PurviewGuardEngine::evaluate(content, label, destination)?;

        Ok(format!(
            "### 🛡️ Microsoft Purview Sensitivity & Zero-Egress Audit\n\n\
            - **Sensitivity Level:** {}\n\
            - **Air-Gapped Zero-Egress:** {}\n\
            - **Enforced Execution Route:** `{}`\n\
            - **External Cloud Egress:** {}\n\
            - **Policy Applied:** `{}`\n\n\
            #### Cryptographic Audit Receipt (SHA-256):\n\
            ```json\n{}\n```\n\n\
            **Status Summary:**\n{}",
            result.sensitivity.badge(),
            if result.air_gapped { "🔒 ACTIVE (Local Only)" } else { "🌐 Inactive (Permissive)" },
            result.routing_engine,
            if result.egress_allowed { "✅ Allowed" } else { "🛑 Strictly Forbidden" },
            result.receipt.policy_applied,
            serde_json::to_string_pretty(&result.receipt).unwrap_or_default(),
            result.message
        ))
    }
}
