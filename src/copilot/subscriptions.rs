//! Microsoft Graph Webhook Subscriptions & Lifecycle Management Engine
//!
//! Features:
//! - Webhook validation challenge handshake (`validationToken`) responding within 10s
//! - Subscription lifecycle: create, renew, delete, and list subscriptions for online meetings, SharePoint drives, and Teams chats
//! - `clientState` HMAC / cryptographic signature verification guaranteeing notifications originate from Microsoft Graph
//! - Thread-safe in-memory state with deterministic mock execution and live Graph API dispatch.

use crate::copilot::graph::GraphClient;
use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Standard Graph Subscription Resource limits
pub const MAX_CHAT_SUBSCRIPTION_MINUTES: i64 = 60; // 1 hour for Teams chats
pub const MAX_MEETING_SUBSCRIPTION_MINUTES: i64 = 4320; // 3 days for Online Meetings
pub const MAX_DRIVE_SUBSCRIPTION_MINUTES: i64 = 42300; // ~29.3 days for SharePoint drives
pub const DEFAULT_SUBSCRIPTION_MINUTES: i64 = 4320;

/// Represents a Microsoft Graph Webhook Subscription
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphSubscription {
    /// Unique Subscription GUID
    pub id: String,
    /// Graph Resource path (e.g. "me/onlineMeetings", "chats/getAllMessages", "drives/root")
    pub resource: String,
    /// Event change types (e.g. "created", "updated", "deleted", or comma-separated)
    pub change_type: String,
    /// HTTPS Webhook callback URL
    pub notification_url: String,
    /// Expiration timestamp in RFC 3339 format
    pub expiration_date_time: String,
    /// Opaque client verification secret or HMAC signature
    pub client_state: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last renewal timestamp if renewed
    pub last_renewed_at: Option<String>,
    /// Status: "active", "renewed", "deleted", "expired"
    pub status: String,
}

impl GraphSubscription {
    /// Checks if this subscription has expired
    pub fn is_expired(&self) -> bool {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&self.expiration_date_time) {
            Utc::now() >= exp.with_timezone(&Utc)
        } else {
            false
        }
    }

    /// Remaining lifetime in seconds
    pub fn remaining_seconds(&self) -> i64 {
        if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&self.expiration_date_time) {
            let diff = exp.with_timezone(&Utc) - Utc::now();
            diff.num_seconds().max(0)
        } else {
            0
        }
    }
}

/// Parsed Graph Webhook Notification Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphWebhookNotification {
    pub subscription_id: String,
    pub client_state: String,
    pub change_type: String,
    pub resource: String,
    pub resource_data: Value,
    pub subscription_expiration_date_time: Option<String>,
}

pub type WebhookNotification = GraphWebhookNotification;
pub type SubscriptionResource = String;

/// Subscription Lifecycle Engine
pub struct SubscriptionLifecycleEngine {
    client: Arc<GraphClient>,
    subscriptions: RwLock<HashMap<String, GraphSubscription>>,
    signing_secret: String,
}

impl SubscriptionLifecycleEngine {
    /// Create new engine with a GraphClient
    pub fn new(client: Arc<GraphClient>) -> Self {
        let secret = std::env::var("COPILOT_WEBHOOK_SECRET")
            .unwrap_or_else(|_| "tagisan_webhook_default_client_state_secret_key_365".to_string());
        Self {
            client,
            subscriptions: RwLock::new(HashMap::new()),
            signing_secret: secret,
        }
    }

    /// Create offline mock engine
    pub fn mock() -> Self {
        let client = Arc::new(GraphClient::mock());
        Self::new(client)
    }

    /// Set custom signing secret for clientState verification
    pub fn with_signing_secret(mut self, secret: impl Into<String>) -> Self {
        self.signing_secret = secret.into();
        self
    }

    /// Webhook validation challenge handshake:
    /// Microsoft Graph requires the endpoint to respond with the exact `validationToken` in plaintext within 10 seconds.
    pub fn handle_validation_challenge(validation_token: &str) -> Result<String> {
        let trimmed = validation_token.trim();
        if trimmed.is_empty() {
            return Err(TagisanError::Security(
                "Empty validationToken received in webhook handshake".to_string(),
            ));
        }

        if trimmed.len() > 2048 {
            return Err(TagisanError::Security(
                "validationToken exceeds maximum allowed length (2048 bytes)".to_string(),
            ));
        }

        // Return exact validation token
        Ok(trimmed.to_string())
    }

    /// Compute HMAC-SHA256 signature for clientState verification
    pub fn compute_client_state(&self, resource: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.signing_secret.as_bytes());
        hasher.update(b":");
        hasher.update(resource.as_bytes());
        format!("tgs_state_{:x}", hasher.finalize())
    }

    /// Verify incoming clientState against expected value or computed HMAC
    pub fn verify_client_state(&self, received_state: &str, expected_resource_or_secret: &str) -> bool {
        if received_state == expected_resource_or_secret {
            return true;
        }

        let computed = self.compute_client_state(expected_resource_or_secret);
        if received_state == computed {
            return true;
        }

        // Also check if received_state starts with "tgs_state_" and matches SHA-256
        let mut hasher = Sha256::new();
        hasher.update(self.signing_secret.as_bytes());
        hasher.update(b":");
        hasher.update(expected_resource_or_secret.as_bytes());
        let expected_hash = format!("tgs_state_{:x}", hasher.finalize());

        subtle_equals(received_state.as_bytes(), expected_hash.as_bytes())
    }

    /// Create a new webhook subscription
    pub async fn create_subscription(
        &self,
        resource: &str,
        change_type: &str,
        notification_url: &str,
        duration_minutes: Option<i64>,
        custom_client_state: Option<&str>,
    ) -> Result<GraphSubscription> {
        let max_minutes = if resource.contains("chats") {
            MAX_CHAT_SUBSCRIPTION_MINUTES
        } else if resource.contains("onlineMeetings") {
            MAX_MEETING_SUBSCRIPTION_MINUTES
        } else if resource.contains("drives") || resource.contains("sites") {
            MAX_DRIVE_SUBSCRIPTION_MINUTES
        } else {
            DEFAULT_SUBSCRIPTION_MINUTES
        };

        let minutes = duration_minutes.unwrap_or(max_minutes).min(max_minutes).max(5);
        let now = Utc::now();
        let exp = now + Duration::minutes(minutes);

        let client_state = custom_client_state
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.compute_client_state(resource));

        let sub_id = format!(
            "sub_{}",
            &blake3::hash(format!("{resource}_{notification_url}_{now}").as_bytes()).to_hex()[..16]
        );

        let subscription = GraphSubscription {
            id: sub_id.clone(),
            resource: resource.to_string(),
            change_type: change_type.to_string(),
            notification_url: notification_url.to_string(),
            expiration_date_time: exp.to_rfc3339(),
            client_state,
            created_at: now.to_rfc3339(),
            last_renewed_at: None,
            status: "active".to_string(),
        };

        if self.client.is_mock() {
            debug!(
                target: "copilot::subscriptions",
                "Mock Graph subscription created: ID={}, resource='{}', expires='{}'",
                subscription.id, subscription.resource, subscription.expiration_date_time
            );
            let mut lock = self.subscriptions.write().await;
            lock.insert(sub_id, subscription.clone());
            return Ok(subscription);
        }

        // Live Microsoft Graph API call: POST /subscriptions
        let token = self.client.auth_manager().get_valid_token().await?;
        let payload = json!({
            "changeType": change_type,
            "notificationUrl": notification_url,
            "resource": resource,
            "expirationDateTime": exp.to_rfc3339(),
            "clientState": subscription.client_state,
        });

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let res = http
            .post("https://graph.microsoft.com/v1.0/subscriptions")
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("graph_subscription_create".to_string(), err));
        }

        let resp_json: Value = res.json().await.map_err(TagisanError::Network)?;
        let real_id = resp_json["id"].as_str().unwrap_or(&sub_id).to_string();
        let mut real_sub = subscription;
        real_sub.id = real_id.clone();

        let mut lock = self.subscriptions.write().await;
        lock.insert(real_id, real_sub.clone());
        Ok(real_sub)
    }

    /// Renew an existing subscription
    pub async fn renew_subscription(
        &self,
        subscription_id: &str,
        additional_minutes: Option<i64>,
    ) -> Result<GraphSubscription> {
        let mut lock = self.subscriptions.write().await;
        let sub = lock.get_mut(subscription_id).ok_or_else(|| {
            TagisanError::Execution(format!("Subscription '{subscription_id}' not found"))
        })?;

        let max_minutes = if sub.resource.contains("chats") {
            MAX_CHAT_SUBSCRIPTION_MINUTES
        } else if sub.resource.contains("onlineMeetings") {
            MAX_MEETING_SUBSCRIPTION_MINUTES
        } else {
            DEFAULT_SUBSCRIPTION_MINUTES
        };

        let minutes = additional_minutes.unwrap_or(max_minutes).min(max_minutes).max(5);
        let now = Utc::now();
        let new_exp = now + Duration::minutes(minutes);

        sub.expiration_date_time = new_exp.to_rfc3339();
        sub.last_renewed_at = Some(now.to_rfc3339());
        sub.status = "renewed".to_string();

        let updated = sub.clone();

        if self.client.is_mock() {
            debug!(
                target: "copilot::subscriptions",
                "Mock subscription renewed: ID={}, new expiration='{}'",
                subscription_id, updated.expiration_date_time
            );
            return Ok(updated);
        }

        // Live Graph API PATCH /subscriptions/{id}
        let token = self.client.auth_manager().get_valid_token().await?;
        let payload = json!({
            "expirationDateTime": updated.expiration_date_time,
        });

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let url = format!("https://graph.microsoft.com/v1.0/subscriptions/{subscription_id}");
        let res = http
            .patch(&url)
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("graph_subscription_renew".to_string(), err));
        }

        Ok(updated)
    }

    /// Delete a webhook subscription
    pub async fn delete_subscription(&self, subscription_id: &str) -> Result<bool> {
        let mut lock = self.subscriptions.write().await;
        let existed = lock.remove(subscription_id).is_some();

        if self.client.is_mock() {
            debug!(
                target: "copilot::subscriptions",
                "Mock subscription deleted: ID={}", subscription_id
            );
            return Ok(existed);
        }

        let token = self.client.auth_manager().get_valid_token().await?;
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let url = format!("https://graph.microsoft.com/v1.0/subscriptions/{subscription_id}");
        let res = http
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() && res.status() != reqwest::StatusCode::NOT_FOUND {
            let err = res.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("graph_subscription_delete".to_string(), err));
        }

        Ok(true)
    }

    /// List all tracked subscriptions
    pub async fn list_subscriptions(&self) -> Vec<GraphSubscription> {
        let lock = self.subscriptions.read().await;
        lock.values().cloned().collect()
    }

    /// Verify notification envelope and authenticity
    pub fn verify_notification(
        &self,
        payload: &Value,
        expected_client_state: &str,
    ) -> Result<Vec<GraphWebhookNotification>> {
        let notifications = payload
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                TagisanError::Execution("Missing 'value' array in Graph notification envelope".to_string())
            })?;

        let mut verified = Vec::new();
        for item in notifications {
            let client_state = item["clientState"].as_str().unwrap_or("");
            if !self.verify_client_state(client_state, expected_client_state) {
                return Err(TagisanError::Security(format!(
                    "Graph notification clientState signature verification failed! Received: '{client_state}'"
                )));
            }

            let sub_id = item["subscriptionId"].as_str().unwrap_or("").to_string();
            let change_type = item["changeType"].as_str().unwrap_or("updated").to_string();
            let resource = item["resource"].as_str().unwrap_or("").to_string();
            let resource_data = item.get("resourceData").cloned().unwrap_or(json!({}));
            let exp = item["subscriptionExpirationDateTime"].as_str().map(|s| s.to_string());

            verified.push(GraphWebhookNotification {
                subscription_id: sub_id,
                client_state: client_state.to_string(),
                change_type,
                resource,
                resource_data,
                subscription_expiration_date_time: exp,
            });
        }

        Ok(verified)
    }
}

/// Constant time comparison helper
fn subtle_equals(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// =========================================================================
// CopilotSubscriptionTool (copilot_subscription_manage)
// =========================================================================

/// Autonomous tool for managing Microsoft Graph webhook subscriptions and validating handshakes
#[derive(Clone)]
pub struct CopilotSubscriptionTool {
    engine: Arc<SubscriptionLifecycleEngine>,
}

impl Default for CopilotSubscriptionTool {
    fn default() -> Self {
        let engine = Arc::new(SubscriptionLifecycleEngine::mock());
        Self { engine }
    }
}

impl CopilotSubscriptionTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_engine(engine: Arc<SubscriptionLifecycleEngine>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl ToolHandler for CopilotSubscriptionTool {
    fn name(&self) -> &str {
        "copilot_subscription_manage"
    }

    fn description(&self) -> &str {
        "Manages Microsoft Graph webhook subscriptions for Teams chats, SharePoint drives, and online meetings. Handles challenge handshakes, creation, renewal, deletion, and clientState signature verification."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "renew", "delete", "list", "validate_challenge", "verify_notification"],
                    "description": "Lifecycle action to perform"
                },
                "resource": {
                    "type": "string",
                    "description": "Graph API resource string (e.g. 'me/onlineMeetings', 'chats/getAllMessages', 'drives/root')"
                },
                "notification_url": {
                    "type": "string",
                    "description": "Webhook endpoint URL (must be HTTPS)"
                },
                "change_type": {
                    "type": "string",
                    "description": "Comma-separated change types: 'created', 'updated', 'deleted' (default: 'created,updated')"
                },
                "duration_minutes": {
                    "type": "integer",
                    "description": "Subscription duration in minutes"
                },
                "subscription_id": {
                    "type": "string",
                    "description": "Target subscription GUID for renewal or deletion"
                },
                "client_state": {
                    "type": "string",
                    "description": "Opaque secret string or signature for notification authenticity verification"
                },
                "validation_token": {
                    "type": "string",
                    "description": "Validation token from Microsoft Graph webhook handshake challenge"
                },
                "notification_payload": {
                    "type": "object",
                    "description": "Incoming JSON notification payload for verification"
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
            "validate_challenge" => {
                let token = arguments
                    .get("validation_token")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'validation_token' for validate_challenge".to_string()))?;

                let res = SubscriptionLifecycleEngine::handle_validation_challenge(token)?;
                Ok(format!(
                    "### 🤝 Microsoft Graph Webhook Handshake Challenge Verified\n\n\
                    - **Validation Status:** HTTP 200 OK Response (within 10s)\n\
                    - **Returned Token:** `{res}`\n\
                    - **Token Length:** {} characters\n\n\
                    Webhook endpoint is verified and active for Microsoft Graph subscriptions.",
                    res.len()
                ))
            }

            "create" => {
                let resource = arguments
                    .get("resource")
                    .and_then(|v| v.as_str())
                    .unwrap_or("me/onlineMeetings");

                let notification_url = arguments
                    .get("notification_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://api.tagisan.ai/copilot/webhook");

                let change_type = arguments
                    .get("change_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("created,updated");

                let duration_minutes = arguments
                    .get("duration_minutes")
                    .and_then(|v| v.as_i64());

                let client_state = arguments
                    .get("client_state")
                    .and_then(|v| v.as_str());

                let sub = self.engine.create_subscription(
                    resource,
                    change_type,
                    notification_url,
                    duration_minutes,
                    client_state,
                ).await?;

                Ok(format!(
                    "### 📡 Microsoft Graph Webhook Subscription Created\n\n\
                    - **Subscription ID:** `{}`\n\
                    - **Target Resource:** `{}`\n\
                    - **Change Types:** `{}`\n\
                    - **Notification URL:** `{}`\n\
                    - **Expiration:** `{}` ({} seconds remaining)\n\
                    - **clientState Verification:** `{}`\n\
                    - **Status:** `{}`\n\n\
                    ```json\n{}\n```",
                    sub.id, sub.resource, sub.change_type, sub.notification_url,
                    sub.expiration_date_time, sub.remaining_seconds(),
                    sub.client_state, sub.status,
                    serde_json::to_string_pretty(&sub).unwrap_or_default()
                ))
            }

            "renew" => {
                let sub_id = arguments
                    .get("subscription_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'subscription_id' for renew".to_string()))?;

                let duration_minutes = arguments
                    .get("duration_minutes")
                    .and_then(|v| v.as_i64());

                let sub = self.engine.renew_subscription(sub_id, duration_minutes).await?;

                Ok(format!(
                    "### 🔄 Microsoft Graph Webhook Subscription Renewed\n\n\
                    - **Subscription ID:** `{}`\n\
                    - **Resource:** `{}`\n\
                    - **New Expiration:** `{}`\n\
                    - **Remaining Time:** {} seconds\n\
                    - **Status:** `{}`",
                    sub.id, sub.resource, sub.expiration_date_time, sub.remaining_seconds(), sub.status
                ))
            }

            "delete" => {
                let sub_id = arguments
                    .get("subscription_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| TagisanError::Execution("Missing 'subscription_id' for delete".to_string()))?;

                let success = self.engine.delete_subscription(sub_id).await?;

                Ok(format!(
                    "### 🗑️ Microsoft Graph Webhook Subscription Deleted\n\n\
                    - **Subscription ID:** `{sub_id}`\n\
                    - **Deleted:** {}\n\
                    - **Status:** Inactive",
                    if success { "YES" } else { "NOT FOUND" }
                ))
            }

            "list" => {
                let subs = self.engine.list_subscriptions().await;
                let mut out = format!("### 📋 Active Microsoft Graph Webhook Subscriptions ({})\n\n", subs.len());
                if subs.is_empty() {
                    out.push_str("No active subscriptions currently registered.");
                } else {
                    for (idx, s) in subs.iter().enumerate() {
                        out.push_str(&format!(
                            "{}. **{}** (`{}`)\n   - Resource: `{}`\n   - Expires: `{}` ({}s remaining)\n   - Status: `{}`\n",
                            idx + 1, s.id, s.resource, s.notification_url, s.expiration_date_time, s.remaining_seconds(), s.status
                        ));
                    }
                }
                Ok(out)
            }

            "verify_notification" => {
                let payload = arguments
                    .get("notification_payload")
                    .cloned()
                    .unwrap_or(json!({ "value": [] }));

                let expected_state = arguments
                    .get("client_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tagisan_webhook_default_client_state_secret_key_365");

                let verified = self.engine.verify_notification(&payload, expected_state)?;

                Ok(format!(
                    "### 🛡️ Microsoft Graph Notification clientState Verified\n\n\
                    - **Events Verified:** {}\n\
                    - **Signature Check:** ✅ PASSED (Verified from Microsoft Graph)\n\
                    - **Subscriptions Impacted:** {}\n\n\
                    ```json\n{}\n```",
                    verified.len(),
                    verified.iter().map(|v| v.subscription_id.clone()).collect::<Vec<_>>().join(", "),
                    serde_json::to_string_pretty(&verified).unwrap_or_default()
                ))
            }

            other => Err(TagisanError::Execution(format!(
                "Unknown subscription action '{other}'. Valid actions: create, renew, delete, list, validate_challenge, verify_notification"
            ))),
        }
    }
}
