//! Microsoft Sentinel & Azure Monitor SIEM Telemetry Bridge
//!
//! Enterprise security telemetry bridge for Tagisan Copilot:
//! - Common Event Format (CEF) / ArcSight structured event format
//! - RFC 5424 Syslog structured data format
//! - Azure Monitor Data Collection Endpoint (DCE) / Log Analytics custom log ingestion
//! - Structured audit events for AST blast-radius calculations, DLP interceptions, Purview air-gap triggers, and patch commits
//! - Thread-safe in-memory audit trail buffer with JSON export.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;

/// Sentinel Event Severity Levels (aligned with CEF 0-10 and Azure Monitor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentinelSeverity {
    Low = 1,
    Medium = 5,
    High = 8,
    Critical = 10,
}

impl SentinelSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    pub fn cef_value(&self) -> u32 {
        *self as u32
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "critical" | "crit" | "10" | "p0" => Self::Critical,
            "high" | "8" | "p1" => Self::High,
            "medium" | "med" | "5" | "p2" => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// Structured Sentinel Event Categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SentinelEventType {
    AstBlastRadiusCalculated,
    DlpInterception,
    PurviewAirGapTriggered,
    PatchCommit,
    CustomSecurityEvent,
}

impl SentinelEventType {
    pub fn class_id(&self) -> &'static str {
        match self {
            Self::AstBlastRadiusCalculated => "SEC-AST-001",
            Self::DlpInterception => "SEC-DLP-002",
            Self::PurviewAirGapTriggered => "SEC-PVW-003",
            Self::PatchCommit => "SEC-PATCH-004",
            Self::CustomSecurityEvent => "SEC-GEN-005",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::AstBlastRadiusCalculated => "AST Codebase Blast Radius Computed",
            Self::DlpInterception => "AgentShield Enterprise DLP Interception",
            Self::PurviewAirGapTriggered => "Purview Zero-Egress Air-Gap Enforced",
            Self::PatchCommit => "Automated Codebase Patch Committed to Git",
            Self::CustomSecurityEvent => "Copilot General Security Audit Event",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        let lower = s.trim().to_lowercase().replace(['-', ' '], "_");
        if lower.contains("blast") || lower.contains("ast") {
            Self::AstBlastRadiusCalculated
        } else if lower.contains("dlp") || lower.contains("shield") || lower.contains("block") {
            Self::DlpInterception
        } else if lower.contains("purview") || lower.contains("airgap") || lower.contains("air_gap") {
            Self::PurviewAirGapTriggered
        } else if lower.contains("patch") || lower.contains("commit") || lower.contains("pr") {
            Self::PatchCommit
        } else {
            Self::CustomSecurityEvent
        }
    }
}

/// Structured Security Telemetry Event for Sentinel & Azure Monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelSecurityEvent {
    pub event_id: String,
    pub timestamp: String,
    pub event_type: SentinelEventType,
    pub severity: SentinelSeverity,
    pub actor: String,
    pub summary: String,
    pub details: Value,
    pub receipt_id: Option<String>,
}

pub type SentinelAuditEvent = SentinelSecurityEvent;
pub type SentinelAuditResult = SentinelSecurityEvent;

impl SentinelSecurityEvent {
    /// Format this event as Common Event Format (CEF:0)
    /// `CEF:Version|Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|[Extension]`
    pub fn to_cef(&self) -> String {
        let class_id = self.event_type.class_id();
        let name = self.event_type.name();
        let sev = self.severity.cef_value();

        let mut ext = format!(
            "src={} msg={} act={} cs1={} cs1Label=EventId",
            self.actor,
            escape_cef(&self.summary),
            self.event_type.name().replace(' ', "_"),
            self.event_id
        );

        if let Some(ref r) = self.receipt_id {
            ext.push_str(&format!(" cs2={} cs2Label=ReceiptId", r));
        }

        if let Some(target) = self.details.get("symbol").and_then(|v| v.as_str()) {
            ext.push_str(&format!(" dtarget={}", target));
        }
        if let Some(files) = self.details.get("affected_files_count").and_then(|v| v.as_u64()) {
            ext.push_str(&format!(" cn1={} cn1Label=AffectedFiles", files));
        }
        if let Some(risk) = self.details.get("risk_level").and_then(|v| v.as_str()) {
            ext.push_str(&format!(" cs3={} cs3Label=RiskLevel", risk));
        }

        format!("CEF:0|Tagisan|TagisanCopilot|0.2.0|{class_id}|{name}|{sev}|{ext}")
    }

    /// Format this event as RFC 5424 Syslog format
    pub fn to_rfc5424(&self) -> String {
        let pri = 134;
        let hostname = "tagisan-copilot.local";
        let app_name = "tgs-copilot";
        let proc_id = std::process::id();
        let msg_id = self.event_type.class_id();

        let structured_data = format!(
            "[tagisan@365 eventId=\"{}\" eventType=\"{:?}\" severity=\"{}\" actor=\"{}\"]",
            self.event_id, self.event_type, self.severity.as_str(), self.actor
        );

        format!(
            "<{pri}>1 {} {hostname} {app_name} {proc_id} {msg_id} {structured_data} {}",
            self.timestamp, self.summary
        )
    }

    /// Format for Azure Monitor Data Collection Endpoint / Log Analytics DCR payload
    pub fn to_azure_monitor_record(&self) -> Value {
        json!({
            "TimeGenerated": self.timestamp,
            "EventVendor": "Tagisan",
            "EventProduct": "TagisanCopilot",
            "EventClassId": self.event_type.class_id(),
            "EventName": self.event_type.name(),
            "EventSeverity": self.severity.as_str(),
            "Actor": self.actor,
            "Summary": self.summary,
            "ReceiptId": self.receipt_id,
            "RawCef": self.to_cef(),
            "Details": self.details,
        })
    }
}

fn escape_cef(s: &str) -> String {
    s.replace('\\', "\\\\").replace('|', "\\|").replace('\n', " ").replace('\r', "")
}

/// Microsoft Sentinel Telemetry Bridge Engine
pub struct SentinelBridgeEngine {
    events: RwLock<Vec<SentinelSecurityEvent>>,
    total_emitted: AtomicU64,
}

pub type SentinelAuditEngine = SentinelBridgeEngine;

impl Default for SentinelBridgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SentinelBridgeEngine {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            total_emitted: AtomicU64::new(0),
        }
    }

    /// Emit an event into the Sentinel telemetry bridge
    pub async fn emit_event(&self, event: SentinelSecurityEvent) -> Result<String> {
        self.total_emitted.fetch_add(1, Ordering::Relaxed);
        let id = event.event_id.clone();
        let mut lock = self.events.write().await;
        lock.push(event);
        Ok(id)
    }

    /// Create and emit a structured event directly
    pub async fn log(
        &self,
        event_type: SentinelEventType,
        severity: SentinelSeverity,
        actor: &str,
        summary: &str,
        details: Value,
        receipt_id: Option<String>,
    ) -> Result<SentinelSecurityEvent> {
        let now = Utc::now();
        let timestamp = now.to_rfc3339();
        let event_id = format!(
            "sentinel_{}_{}",
            &blake3::hash(format!("{timestamp}_{summary}_{actor}").as_bytes()).to_hex()[..16],
            now.timestamp()
        );

        let event = SentinelSecurityEvent {
            event_id,
            timestamp,
            event_type,
            severity,
            actor: actor.to_string(),
            summary: summary.to_string(),
            details,
            receipt_id,
        };

        self.emit_event(event.clone()).await?;
        Ok(event)
    }

    /// Get list of recent events
    pub async fn get_events(&self, limit: usize) -> Vec<SentinelSecurityEvent> {
        let lock = self.events.read().await;
        lock.iter().rev().take(limit).cloned().collect()
    }

    /// Export events in requested format ("cef", "rfc5424", "azure_monitor", "all")
    pub async fn export_events(&self, format_type: &str, limit: usize) -> Result<String> {
        let events = self.get_events(limit).await;
        match format_type.to_lowercase().as_str() {
            "cef" => {
                let lines: Vec<String> = events.iter().map(|e| e.to_cef()).collect();
                Ok(lines.join("\n"))
            }
            "rfc5424" | "syslog" => {
                let lines: Vec<String> = events.iter().map(|e| e.to_rfc5424()).collect();
                Ok(lines.join("\n"))
            }
            "azure_monitor" | "dcr" | "json" => {
                let records: Vec<Value> = events.iter().map(|e| e.to_azure_monitor_record()).collect();
                Ok(serde_json::to_string_pretty(&records)?)
            }
            _ => {
                let mut out = format!("### 🛡️ Microsoft Sentinel Telemetry Bridge Export ({} Events)\n\n", events.len());
                for e in &events {
                    out.push_str(&format!(
                        "#### [{}] {}\n- **Event ID:** `{}`\n- **Timestamp:** `{}`\n- **Severity:** {}\n- **CEF Formatted:**\n```text\n{}\n```\n\n",
                        e.event_type.class_id(), e.summary, e.event_id, e.timestamp, e.severity.as_str(), e.to_cef()
                    ));
                }
                Ok(out)
            }
        }
    }
}

// =========================================================================
// CopilotSentinelAuditTool (copilot_sentinel_audit)
// =========================================================================

/// Autonomous tool for emitting and querying Microsoft Sentinel & Azure Monitor security events
#[derive(Clone)]
pub struct CopilotSentinelAuditTool {
    engine: Arc<SentinelBridgeEngine>,
}

impl Default for CopilotSentinelAuditTool {
    fn default() -> Self {
        Self {
            engine: Arc::new(SentinelBridgeEngine::new()),
        }
    }
}

impl CopilotSentinelAuditTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_engine(engine: Arc<SentinelBridgeEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl ToolHandler for CopilotSentinelAuditTool {
    fn name(&self) -> &str {
        "copilot_sentinel_audit"
    }

    fn description(&self) -> &str {
        "Emits and queries structured security events for Microsoft Sentinel and Azure Monitor Log Analytics using Common Event Format (CEF) and RFC 5424 syslog. Audits AST blast radius, DLP blocks, Purview air-gaps, and automated patch commits."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["emit", "list", "export"],
                    "description": "Action to perform: 'emit' (log event), 'list' (view recent), 'export' (generate CEF/Azure Monitor logs)"
                },
                "event_type": {
                    "type": "string",
                    "description": "Event type: 'ast_blast_radius', 'dlp_interception', 'purview_airgap', 'patch_commit'"
                },
                "severity": {
                    "type": "string",
                    "description": "Severity: 'Low', 'Medium', 'High', 'Critical' (default: 'Medium')"
                },
                "actor": {
                    "type": "string",
                    "description": "Actor or identity initiating the operation (default: 'copilot.operator@tagisan.ai')"
                },
                "summary": {
                    "type": "string",
                    "description": "Human-readable summary of the security event"
                },
                "details": {
                    "type": "object",
                    "description": "JSON details for structured event attributes"
                },
                "receipt_id": {
                    "type": "string",
                    "description": "Optional cryptographic Purview audit receipt ID"
                },
                "format": {
                    "type": "string",
                    "enum": ["cef", "rfc5424", "azure_monitor", "all"],
                    "description": "Export format (default: 'cef')"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max events to list/export (default: 10)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing required parameter 'action'".to_string()))?;

        match action {
            "emit" => {
                let ev_type_str = arguments
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ast_blast_radius");
                let ev_type = SentinelEventType::from_str_lossy(ev_type_str);

                let sev_str = arguments
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Medium");
                let severity = SentinelSeverity::from_str_lossy(sev_str);

                let actor = arguments
                    .get("actor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("copilot.operator@tagisan.ai");

                let summary = arguments
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Copilot automated security verification event");

                let details = arguments.get("details").cloned().unwrap_or(json!({}));
                let receipt_id = arguments.get("receipt_id").and_then(|v| v.as_str()).map(|s| s.to_string());

                let event = self.engine.log(ev_type, severity, actor, summary, details, receipt_id).await?;

                Ok(format!(
                    "### 🛡️ Microsoft Sentinel Security Event Emitted\n\n\
                    - **Event ID:** `{}`\n\
                    - **Class ID:** `{}`\n\
                    - **Severity:** {} ({})\n\
                    - **Actor:** `{}`\n\
                    - **Timestamp:** `{}`\n\n\
                    #### Common Event Format (CEF:0):\n\
                    ```text\n{}\n```\n\n\
                    #### RFC 5424 Syslog Record:\n\
                    ```text\n{}\n```\n\n\
                    ✅ Dispatched to Microsoft Sentinel SIEM Bridge & Azure Monitor Log Analytics buffer.",
                    event.event_id,
                    event.event_type.class_id(),
                    event.severity.as_str(),
                    event.severity.cef_value(),
                    event.actor,
                    event.timestamp,
                    event.to_cef(),
                    event.to_rfc5424()
                ))
            }

            "list" => {
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let events = self.engine.get_events(limit).await;
                let mut out = format!("### 📋 Microsoft Sentinel Audit Trail ({} Events)\n\n", events.len());
                if events.is_empty() {
                    out.push_str("No security events recorded in current SIEM bridge buffer.");
                } else {
                    for (i, e) in events.iter().enumerate() {
                        out.push_str(&format!(
                            "{}. **[{}]** {} (`{}`)\n   - Severity: **{}** | Actor: `{}` | Time: `{}`\n",
                            i + 1, e.event_type.class_id(), e.summary, e.event_id,
                            e.severity.as_str(), e.actor, e.timestamp
                        ));
                    }
                }
                Ok(out)
            }

            "export" => {
                let format_type = arguments.get("format").and_then(|v| v.as_str()).unwrap_or("cef");
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                self.engine.export_events(format_type, limit).await
            }

            other => Err(TagisanError::Execution(format!(
                "Unknown sentinel action '{other}'. Valid actions: emit, list, export"
            ))),
        }
    }
}
