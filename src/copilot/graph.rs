//! Microsoft Graph REST API Client for Microsoft 365 Copilot Integration
//!
//! Provides async operations for:
//! - Microsoft Teams channel/chat messaging with outbound DLP protection
//! - SharePoint / OneDrive document ingestion with inbound prompt-injection sanitization
//! - Online meeting transcript retrieval & AI action-item decomposition
//! - Outlook HTML report dispatching with DLP interception
//! - Built-in zero-network mock sandbox mode for offline CI and deterministic testing.

use crate::copilot::auth::EntraAuthManager;
use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Represents an individual speech turn in a Teams meeting transcript
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptEntry {
    /// Name or email of the speaker
    pub speaker: String,
    /// Transcribed spoken text
    pub text: String,
    /// Timestamp offset or ISO-8601 string
    pub timestamp: Option<String>,
}

/// Extracted engineering action item from a meeting transcript
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionItem {
    /// Unique item identifier
    pub id: String,
    /// Action item title / summary
    pub title: String,
    /// Detailed description of the requirement
    pub description: String,
    /// Assigned engineer or team
    pub assignee: Option<String>,
    /// Priority level: "High", "Medium", "Low"
    pub priority: String,
    /// Target due date or timeline if mentioned
    pub due_date: Option<String>,
    /// Engineering category (e.g. Architecture, Security, Testing, Infrastructure)
    pub category: Option<String>,
}

/// Ingested SharePoint or OneDrive document content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentContent {
    /// Original URL or SharePoint relative path
    pub path_or_url: String,
    /// File basename
    pub filename: String,
    /// MIME content type or extension (markdown, text, docx, json)
    pub content_type: String,
    /// Extracted plain-text / markdown content
    pub text: String,
    /// Size in bytes
    pub size_bytes: usize,
}

/// Microsoft Graph REST API Client
#[derive(Clone)]
pub struct GraphClient {
    auth_manager: Arc<EntraAuthManager>,
    http: reqwest::Client,
    base_url: String,
    mock: bool,
    throttler: Arc<crate::copilot::throttling::AdaptiveThrottler>,
}

impl GraphClient {
    /// Create a new GraphClient with an EntraAuthManager
    pub fn new(auth_manager: Arc<EntraAuthManager>) -> Self {
        let is_mock = auth_manager.is_mock();
        Self {
            auth_manager,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url: "https://graph.microsoft.com/v1.0".to_string(),
            mock: is_mock,
            throttler: Arc::new(crate::copilot::throttling::AdaptiveThrottler::default()),
        }
    }

    /// Create an explicit offline mock GraphClient
    pub fn mock() -> Self {
        let auth = Arc::new(EntraAuthManager::mock());
        let mut client = Self::new(auth);
        client.mock = true;
        client
    }

    /// Access the adaptive throttler
    pub fn throttler(&self) -> &Arc<crate::copilot::throttling::AdaptiveThrottler> {
        &self.throttler
    }

    /// Set a custom adaptive throttler
    pub fn with_throttler(mut self, throttler: Arc<crate::copilot::throttling::AdaptiveThrottler>) -> Self {
        self.throttler = throttler;
        self
    }

    /// Simulate handling an HTTP 429 response with Retry-After header
    pub async fn simulate_429_retry_flow(&self, resource: &str, simulated_retry_after: &str) -> (u32, std::time::Duration) {
        let _ = self.throttler.acquire(resource, 1.0).await;
        let backoff = self.throttler.record_throttled(Some(simulated_retry_after), 1);
        (1, backoff)
    }

    /// Override the base API URL (e.g. for custom proxies or testing)
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Check whether this client operates in offline mock mode
    pub fn is_mock(&self) -> bool {
        if self.mock
            || self.auth_manager.is_mock()
            || std::env::var("TGS_MOCK_GRAPH")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
        {
            return true;
        }

        // Auto-detect mock token or default dummy credentials
        if let Ok(Some(tok)) = self.auth_manager.load_token() {
            if tok.access_token.starts_with("mock_") {
                return true;
            }
        }
        if self.auth_manager.config().client_id == "00000000-0000-0000-0000-000000000000" {
            return true;
        }

        false
    }

    /// Access the underlying Entra ID auth manager
    pub fn auth_manager(&self) -> &Arc<EntraAuthManager> {
        &self.auth_manager
    }

    /// Execute a generic GET request against Microsoft Graph API
    pub async fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        if self.is_mock() {
            return Ok(serde_json::json!({ "value": [] }));
        }
        let token = self.auth_manager.get_valid_token().await?;
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!(
                "{}/{}",
                self.base_url.trim_end_matches('/'),
                endpoint.trim_start_matches('/')
            )
        };
        let res = self
            .http
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(TagisanError::Network)?;
        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }
        res.json().await.map_err(TagisanError::Network)
    }

    /// Execute a generic POST request against Microsoft Graph API
    pub async fn post(&self, endpoint: &str, payload: &serde_json::Value) -> Result<serde_json::Value> {
        if self.is_mock() {
            return Ok(serde_json::json!({ "id": "mock_id", "status": "success" }));
        }
        let token = self.auth_manager.get_valid_token().await?;
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!(
                "{}/{}",
                self.base_url.trim_end_matches('/'),
                endpoint.trim_start_matches('/')
            )
        };
        let res = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(payload)
            .send()
            .await
            .map_err(TagisanError::Network)?;
        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }
        res.json().await.map_err(TagisanError::Network)
    }

    /// Post a message to a Microsoft Teams channel or 1:1 / group chat
    /// Outbound DLP scanned via AgentShield prior to dispatch.
    pub async fn send_teams_message(&self, team_or_chat_id: &str, message: &str) -> Result<String> {
        // 1. AgentShield Outbound DLP Interception
        let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(message);
        if let AgentShieldVerdict::Block { reason, .. } = dlp_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP blocked Teams message: {reason}"
            )));
        }

        if self.is_mock() {
            let msg_id = format!("teams_msg_{}", &blake3::hash(message.as_bytes()).to_hex()[..16]);
            debug!(
                target: "copilot::graph",
                "Mock Teams message posted to {}: ID={}",
                team_or_chat_id, msg_id
            );
            return Ok(msg_id);
        }

        let token = self.auth_manager.get_valid_token().await?;
        let url = if team_or_chat_id.contains('/') {
            // Formatted as "teams/{team_id}/channels/{channel_id}"
            format!("{}/{}", self.base_url.trim_end_matches('/'), team_or_chat_id)
        } else {
            // Formatted as chat ID
            format!(
                "{}/chats/{}/messages",
                self.base_url.trim_end_matches('/'),
                team_or_chat_id
            )
        };

        let payload = serde_json::json!({
            "body": {
                "contentType": "html",
                "content": message
            }
        });

        let res = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }

        let resp_json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let msg_id = resp_json["id"]
            .as_str()
            .unwrap_or("teams_message_sent")
            .to_string();

        Ok(msg_id)
    }

    /// Retrieve an online meeting transcript by meeting ID
    /// Inbound Sanitization scanned via AgentShield on all spoken content.
    pub async fn get_meeting_transcript(&self, meeting_id: &str) -> Result<Vec<TranscriptEntry>> {
        let entries = if self.is_mock() {
            debug!(target: "copilot::graph", "Mock meeting transcript fetched for ID={}", meeting_id);
            vec![
                TranscriptEntry {
                    speaker: "Charle Gutierrez".to_string(),
                    text: "Welcome everyone. In this sprint, we must harden the Tagisan Microsoft 365 Copilot communication system.".to_string(),
                    timestamp: Some("00:00:10".to_string()),
                },
                TranscriptEntry {
                    speaker: "Alex Mercer".to_string(),
                    text: "Action item: I will audit the AgentShield enterprise DLP gates and verify zero credential leakage on Graph payloads.".to_string(),
                    timestamp: Some("00:01:45".to_string()),
                },
                TranscriptEntry {
                    speaker: "Maya Lin".to_string(),
                    text: "TODO: Charle to implement the OpenAPI 3.0 specification generator and export the Declarative Agent manifest by Friday. Priority: P0.".to_string(),
                    timestamp: Some("00:03:12".to_string()),
                },
                TranscriptEntry {
                    speaker: "Samira Patel".to_string(),
                    text: "Action item: We need to write brutal multi-threaded stress tests for 50 concurrent workers on the Graph client. Priority: High.".to_string(),
                    timestamp: Some("00:05:00".to_string()),
                },
                TranscriptEntry {
                    speaker: "Charle Gutierrez".to_string(),
                    text: "Agreed. Let's make sure formal invariant verification with tgs ground is also exposed as an action endpoint in the manifest.".to_string(),
                    timestamp: Some("00:06:20".to_string()),
                },
            ]
        } else {
            let token = self.auth_manager.get_valid_token().await?;
            let url = format!(
                "{}/me/onlineMeetings/{}/transcripts",
                self.base_url.trim_end_matches('/'),
                meeting_id
            );

            let res = self
                .http
                .get(&url)
                .bearer_auth(token)
                .send()
                .await
                .map_err(TagisanError::Network)?;

            if !res.status().is_success() {
                let err = res.text().await.unwrap_or_default();
                return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
            }

            let resp_json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
            let mut list = Vec::new();
            if let Some(items) = resp_json["value"].as_array() {
                for item in items {
                    let speaker = item["speaker"]["name"].as_str().unwrap_or("Unknown").to_string();
                    let text = item["text"].as_str().unwrap_or("").to_string();
                    let timestamp = item["timestamp"].as_str().map(|s| s.to_string());
                    list.push(TranscriptEntry {
                        speaker,
                        text,
                        timestamp,
                    });
                }
            }
            list
        };

        // Inbound Sanitization on all transcript entries
        for entry in &entries {
            let scan_verdict = AgentShieldScanner::scan_inbound_document(&entry.text);
            if let AgentShieldVerdict::Block { reason, .. } = scan_verdict {
                return Err(TagisanError::Security(format!(
                    "AgentShield Inbound Sanitization blocked malicious meeting transcript from speaker '{}': {}",
                    entry.speaker, reason
                )));
            }
        }

        Ok(entries)
    }

    /// Extract structured engineering tasks, assignees, and priorities from transcripts
    pub fn parse_action_items(&self, transcript: &[TranscriptEntry]) -> Vec<ActionItem> {
        let mut items = Vec::new();

        let action_triggers = [
            "action item:",
            "action item",
            "todo:",
            "todo",
            "task:",
            "will implement",
            "will audit",
            "will build",
            "need to",
            "needs to",
            "to implement",
            "to build",
            "fix",
            "investigate",
            "ship",
            "refactor",
            "verify",
        ];

        for entry in transcript {
            let lower = entry.text.to_lowercase();
            let mut is_action = false;
            for trigger in action_triggers {
                if lower.contains(trigger) {
                    is_action = true;
                    break;
                }
            }

            if !is_action {
                continue;
            }

            // Extract priority
            let priority = if lower.contains("p0")
                || lower.contains("urgent")
                || lower.contains("critical")
                || lower.contains("immediate")
            {
                "High".to_string()
            } else if lower.contains("p1") || lower.contains("high") || lower.contains("important") {
                "High".to_string()
            } else if lower.contains("p2") || lower.contains("medium") || lower.contains("normal") {
                "Medium".to_string()
            } else if lower.contains("p3") || lower.contains("low") || lower.contains("nice to have") {
                "Low".to_string()
            } else {
                "Medium".to_string()
            };

            // Detect assignee: check mentions in text or fall back to speaker
            let mut assignee = None;
            let known_names = ["Charle", "Alex", "Maya", "Samira", "Sam", "David", "John", "Sarah"];
            for name in known_names {
                if entry.text.contains(name) {
                    assignee = Some(name.to_string());
                    break;
                }
            }
            if assignee.is_none() {
                assignee = Some(entry.speaker.clone());
            }

            // Detect engineering category
            let category = if lower.contains("security")
                || lower.contains("dlp")
                || lower.contains("audit")
                || lower.contains("agentshield")
                || lower.contains("auth")
            {
                "Security".to_string()
            } else if lower.contains("stress")
                || lower.contains("test")
                || lower.contains("verify")
                || lower.contains("ground")
            {
                "Testing & QA".to_string()
            } else if lower.contains("manifest")
                || lower.contains("openapi")
                || lower.contains("copilot")
                || lower.contains("plugin")
            {
                "Copilot Architecture".to_string()
            } else {
                "Core Engineering".to_string()
            };

            // Title and description synthesis
            let clean_title = entry
                .text
                .replace("Action item:", "")
                .replace("Action item", "")
                .replace("TODO:", "")
                .replace("TODO", "")
                .trim()
                .to_string();

            let title = if clean_title.len() > 70 {
                format!("{}...", &clean_title[..67])
            } else {
                clean_title.clone()
            };

            let id = format!(
                "act_{}",
                &blake3::hash(format!("{}_{}", entry.speaker, clean_title).as_bytes()).to_hex()[..12]
            );

            items.push(ActionItem {
                id,
                title,
                description: entry.text.clone(),
                assignee,
                priority,
                due_date: if lower.contains("friday") {
                    Some("Friday".to_string())
                } else if lower.contains("sprint") {
                    Some("End of Sprint".to_string())
                } else {
                    None
                },
                category: Some(category),
            });
        }

        items
    }

    /// Fetch and extract content from SharePoint or OneDrive file
    /// Inbound Sanitization scanned via AgentShield prior to returning document content.
    pub async fn fetch_sharepoint_file(&self, sharepoint_url_or_path: &str) -> Result<DocumentContent> {
        let path_obj = Path::new(sharepoint_url_or_path);

        let (filename, content_type, raw_text) = if path_obj.exists() {
            // Local file fallback (useful for sandbox tests, local workspace docs)
            let filename = path_obj
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("document.txt")
                .to_string();
            let content_type = match path_obj.extension().and_then(|e| e.to_str()) {
                Some("md") => "text/markdown",
                Some("json") => "application/json",
                Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                _ => "text/plain",
            }
            .to_string();
            let text = std::fs::read_to_string(path_obj).unwrap_or_default();
            (filename, content_type, text)
        } else if self.is_mock() {
            debug!(
                target: "copilot::graph",
                "Mock SharePoint file fetched for path: {}",
                sharepoint_url_or_path
            );
            let filename = path_obj
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("Architecture_Specification.md")
                .to_string();

            let text = format!(
                "# Tagisan Architecture Specification\n\n\
                Document Path: `{sharepoint_url_or_path}`\n\
                Status: Approved for Production\n\n\
                ## Overview\n\
                Tagisan Multi-LLM Engine provides high-performance adversarial debate, \
                formal invariant grounding, and seamless Microsoft 365 Copilot collaboration.\n\n\
                ## Core Invariants\n\
                - Invariant A: Dialectical consensus must terminate with a verifiable Lakandiwa synthesis.\n\
                - Invariant B: Outbound DLP must intercept all credentials before transmission.\n\
                - Invariant C: Inbound document ingestion must sanitize prompt injection attacks."
            );

            (filename, "text/markdown".to_string(), text)
        } else {
            let token = self.auth_manager.get_valid_token().await?;
            let url = format!(
                "{}/me/drive/root:{}:/content",
                self.base_url.trim_end_matches('/'),
                sharepoint_url_or_path
            );

            let res = self
                .http
                .get(&url)
                .bearer_auth(token)
                .send()
                .await
                .map_err(TagisanError::Network)?;

            if !res.status().is_success() {
                let err = res.text().await.unwrap_or_default();
                return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
            }

            let filename = path_obj
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("document.txt")
                .to_string();

            let text = res.text().await.map_err(TagisanError::Network)?;
            (filename, "text/plain".to_string(), text)
        };

        // Inbound Prompt Injection Sanitization
        let scan_verdict = AgentShieldScanner::scan_inbound_document(&raw_text);
        if let AgentShieldVerdict::Block { reason, .. } = scan_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Inbound Sanitization blocked malicious SharePoint document ('{}'): {}",
                filename, reason
            )));
        }

        let size_bytes = raw_text.len();
        Ok(DocumentContent {
            path_or_url: sharepoint_url_or_path.to_string(),
            filename,
            content_type,
            text: raw_text,
            size_bytes,
        })
    }

    /// Dispatch an engineering or debate report via Microsoft Graph Outlook mail
    /// Outbound DLP scanned via AgentShield prior to transmission.
    pub async fn send_outlook_report(
        &self,
        to: &[String],
        subject: &str,
        html_body: &str,
    ) -> Result<String> {
        // Outbound DLP scan on both subject and body
        let dlp_subject = AgentShieldScanner::scan_outbound_dlp(subject);
        if let AgentShieldVerdict::Block { reason, .. } = dlp_subject {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP blocked email subject: {reason}"
            )));
        }

        let dlp_body = AgentShieldScanner::scan_outbound_dlp(html_body);
        if let AgentShieldVerdict::Block { reason, .. } = dlp_body {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP blocked email body: {reason}"
            )));
        }

        if self.is_mock() {
            let msg_id = format!("outlook_msg_{}", &blake3::hash(subject.as_bytes()).to_hex()[..16]);
            debug!(
                target: "copilot::graph",
                "Mock Outlook report sent to {:?}: Subject='{}', ID={}",
                to, subject, msg_id
            );
            return Ok(msg_id);
        }

        let token = self.auth_manager.get_valid_token().await?;
        let url = format!("{}/me/sendMail", self.base_url.trim_end_matches('/'));

        let recipients: Vec<serde_json::Value> = to
            .iter()
            .map(|email| {
                serde_json::json!({
                    "emailAddress": {
                        "address": email
                    }
                })
            })
            .collect();

        let payload = serde_json::json!({
            "message": {
                "subject": subject,
                "body": {
                    "contentType": "HTML",
                    "content": html_body
                },
                "toRecipients": recipients
            },
            "saveToSentItems": "true"
        });

        let res = self
            .http
            .post(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }

        let msg_id = format!("outlook_msg_{}", &blake3::hash(subject.as_bytes()).to_hex()[..16]);
        Ok(msg_id)
    }

    /// Sync a page to Microsoft OneNote notebook and section
    /// Outbound DLP scanned via AgentShield prior to dispatch.
    pub async fn sync_onenote_page(
        &self,
        notebook_name: &str,
        section_name: &str,
        title: &str,
        html_content: &str,
    ) -> Result<String> {
        // 1. AgentShield Outbound DLP Interception
        let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(html_content);
        if let AgentShieldVerdict::Block { reason, .. } = dlp_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP blocked OneNote sync: {reason}"
            )));
        }

        if self.is_mock() {
            let page_id = format!(
                "onenote_pg_{}",
                &blake3::hash(format!("{title}_{section_name}_{notebook_name}").as_bytes()).to_hex()[..16]
            );
            debug!(
                target: "copilot::graph",
                "Mock OneNote page synced to {}/{}: ID={}",
                notebook_name, section_name, page_id
            );
            return Ok(page_id);
        }

        let token = self.auth_manager.get_valid_token().await?;
        let url = format!("{}/me/onenote/pages", self.base_url.trim_end_matches('/'));

        let full_html = format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body>{}</body></html>",
            title, html_content
        );

        let res = self
            .http
            .post(&url)
            .bearer_auth(token)
            .header("Content-Type", "application/xhtml+xml")
            .body(full_html)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }

        let resp_json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let page_id = resp_json["id"]
            .as_str()
            .unwrap_or("onenote_page_created")
            .to_string();
        Ok(page_id)
    }

    /// Upload or update a document file in SharePoint / OneDrive document library or wiki
    /// Outbound DLP scanned via AgentShield prior to persistence.
    pub async fn upload_sharepoint_file(
        &self,
        folder_path: &str,
        filename: &str,
        content: &str,
    ) -> Result<String> {
        let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(content);
        if let AgentShieldVerdict::Block { reason, .. } = dlp_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield Outbound DLP blocked SharePoint upload: {reason}"
            )));
        }

        if self.is_mock() {
            let item_id = format!(
                "sp_item_{}",
                &blake3::hash(format!("{folder_path}_{filename}").as_bytes()).to_hex()[..16]
            );
            debug!(
                target: "copilot::graph",
                "Mock SharePoint file uploaded to {}/{}: ID={}",
                folder_path, filename, item_id
            );
            return Ok(item_id);
        }

        let token = self.auth_manager.get_valid_token().await?;
        let clean_folder = folder_path.trim_matches('/');
        let url = format!(
            "{}/me/drive/root:/{}/{}:/content",
            self.base_url.trim_end_matches('/'),
            clean_folder,
            filename
        );

        let res = self
            .http
            .put(&url)
            .bearer_auth(token)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(content.to_string())
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("microsoft_graph".to_string(), err));
        }

        let resp_json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let item_id = resp_json["id"]
            .as_str()
            .unwrap_or("sp_item_uploaded")
            .to_string();
        Ok(item_id)
    }
}
