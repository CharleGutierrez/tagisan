//! Microsoft Entra ID (Azure AD) OAuth2 Authentication Manager
//!
//! Supports Device Code Flow for interactive developer sign-in,
//! Client Credentials for daemon/service principal automation,
//! token persistence, expiration tracking, automatic refresh,
//! and offline deterministic mock testing.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Default Microsoft Graph OAuth2 scope
pub const DEFAULT_GRAPH_SCOPE: &str = "https://graph.microsoft.com/.default";

/// Default path for cached Copilot token
pub const DEFAULT_TOKEN_CACHE_FILE: &str = ".tagisan/copilot_token.json";

/// Configuration for Microsoft Entra ID Authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraIdConfig {
    /// Azure / Entra Application (Client) ID
    pub client_id: String,
    /// Azure / Entra Directory (Tenant) ID (e.g. "common", "organizations", or GUID)
    pub tenant_id: String,
    /// Client Secret (for Client Credentials flow)
    pub client_secret: Option<String>,
    /// OAuth2 Scopes requested
    pub scopes: Vec<String>,
    /// Filesystem location for caching tokens
    pub token_cache_path: PathBuf,
    /// Force offline mock mode for testing and air-gapped CI
    pub mock: bool,
}

impl Default for EntraIdConfig {
    fn default() -> Self {
        let client_id = std::env::var("ENTRA_CLIENT_ID")
            .or_else(|_| std::env::var("AZURE_CLIENT_ID"))
            .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string());

        let tenant_id = std::env::var("ENTRA_TENANT_ID")
            .or_else(|_| std::env::var("AZURE_TENANT_ID"))
            .unwrap_or_else(|_| "common".to_string());

        let client_secret = std::env::var("ENTRA_CLIENT_SECRET")
            .or_else(|_| std::env::var("AZURE_CLIENT_SECRET"))
            .ok();

        let mock = std::env::var("TGS_MOCK_ENTRA")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let token_cache_path = PathBuf::from(DEFAULT_TOKEN_CACHE_FILE);

        Self {
            client_id,
            tenant_id,
            client_secret,
            scopes: vec![
                "offline_access".to_string(),
                "User.Read".to_string(),
                "ChannelMessage.Send".to_string(),
                "Files.Read.All".to_string(),
                "OnlineMeetings.Read".to_string(),
                "Mail.Send".to_string(),
                "ExternalItem.ReadWrite.All".to_string(),
            ],
            token_cache_path,
            mock,
        }
    }
}

/// Serialized Entra ID OAuth2 Token
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntraToken {
    /// Bearer access token string
    pub access_token: String,
    /// Token type, typically "Bearer"
    pub token_type: String,
    /// Lifetime in seconds at issue time
    pub expires_in: i64,
    /// Unix timestamp in seconds when the token expires
    pub expires_at: i64,
    /// Refresh token for renewing access tokens without user re-auth
    pub refresh_token: Option<String>,
    /// Granted scopes
    pub scope: Option<String>,
}

impl EntraToken {
    /// Check if the token is expired or about to expire within 60 seconds
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= (self.expires_at - 60)
    }

    /// Remaining lifetime in seconds
    pub fn expires_in_secs(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        (self.expires_at - now).max(0)
    }

    /// Generate a deterministic mock token
    pub fn mock() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            access_token: format!("mock_entra_token_tagisan_{}", now),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            expires_at: now + 3600,
            refresh_token: Some("mock_refresh_token_tagisan_copilot".to_string()),
            scope: Some("User.Read ChannelMessage.Send Files.Read.All".to_string()),
        }
    }
}

/// Response returned from initiate_device_code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// Status report for Copilot Entra ID authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotAuthStatus {
    pub authenticated: bool,
    pub tenant_id: String,
    pub client_id: String,
    pub auth_mode: String,
    pub expires_at: Option<i64>,
    pub expires_in_secs: Option<i64>,
    pub is_expired: bool,
    pub scope: Option<String>,
}

/// Microsoft Entra ID Token & Authentication Manager
pub struct EntraAuthManager {
    config: EntraIdConfig,
    client: reqwest::Client,
    active_token: RwLock<Option<EntraToken>>,
}

impl EntraAuthManager {
    /// Create a new EntraAuthManager with custom configuration
    pub fn new(config: EntraIdConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            active_token: RwLock::new(None),
        }
    }

    /// Create manager with default environment-resolved settings
    pub fn with_defaults() -> Self {
        Self::new(EntraIdConfig::default())
    }

    /// Create an explicit mock manager for testing
    pub fn mock() -> Self {
        let mut config = EntraIdConfig::default();
        config.mock = true;
        Self::new(config)
    }

    /// Return reference to configuration
    pub fn config(&self) -> &EntraIdConfig {
        &self.config
    }

    /// Check if mock mode is enabled either via config or environment
    pub fn is_mock(&self) -> bool {
        self.config.mock
            || std::env::var("TGS_MOCK_ENTRA")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
    }

    /// Initiate OAuth2 Device Code Flow
    pub async fn initiate_device_code(&self) -> Result<DeviceCodeResponse> {
        if self.is_mock() {
            debug!("Entra ID device code initiated (Mock Mode)");
            return Ok(DeviceCodeResponse {
                device_code: "mock_device_code_tgs_123456789".to_string(),
                user_code: "TGS-COPILOT-777".to_string(),
                verification_uri: "https://microsoft.com/devicelogin".to_string(),
                expires_in: 900,
                interval: 5,
                message: "To sign in, use a web browser to open https://microsoft.com/devicelogin and enter code TGS-COPILOT-777".to_string(),
            });
        }

        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/devicecode",
            self.config.tenant_id
        );

        let scopes_joined = self.config.scopes.join(" ");
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("scope", scopes_joined.as_str()),
        ];

        let res = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(TagisanError::Authentication(
                "entra_id".to_string(),
                format!("Device code request failed: {err_text}"),
            ));
        }

        let resp_json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let device_code = resp_json["device_code"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("entra_id".to_string(), "Missing device_code".to_string()))?
            .to_string();
        let user_code = resp_json["user_code"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("entra_id".to_string(), "Missing user_code".to_string()))?
            .to_string();
        let verification_uri = resp_json["verification_uri"]
            .as_str()
            .unwrap_or("https://microsoft.com/devicelogin")
            .to_string();
        let expires_in = resp_json["expires_in"].as_u64().unwrap_or(900);
        let interval = resp_json["interval"].as_u64().unwrap_or(5);
        let message = resp_json["message"].as_str().unwrap_or("").to_string();

        Ok(DeviceCodeResponse {
            device_code,
            user_code,
            verification_uri,
            expires_in,
            interval,
            message,
        })
    }

    /// Poll for token during Device Code Flow
    pub async fn poll_for_token(&self, device_code: &str, interval: u64, timeout_secs: u64) -> Result<EntraToken> {
        if self.is_mock() {
            debug!("Polling for token in mock mode: immediate success");
            let token = EntraToken::mock();
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token);
        }

        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        let start = tokio::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);
        let mut poll_interval = std::time::Duration::from_secs(interval.max(2));

        while start.elapsed() < timeout {
            let params = [
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", self.config.client_id.as_str()),
                ("device_code", device_code),
            ];

            let res = self
                .client
                .post(&token_url)
                .form(&params)
                .send()
                .await
                .map_err(TagisanError::Network)?;

            let status = res.status();
            let json: serde_json::Value = res.json().await.unwrap_or(serde_json::Value::Null);

            if status.is_success() {
                let access_token = json["access_token"]
                    .as_str()
                    .ok_or_else(|| TagisanError::Authentication("entra_id".to_string(), "Missing access_token".to_string()))?
                    .to_string();
                let token_type = json["token_type"].as_str().unwrap_or("Bearer").to_string();
                let expires_in = json["expires_in"].as_i64().unwrap_or(3600);
                let expires_at = chrono::Utc::now().timestamp() + expires_in;
                let refresh_token = json["refresh_token"].as_str().map(|s| s.to_string());
                let scope = json["scope"].as_str().map(|s| s.to_string());

                let token = EntraToken {
                    access_token,
                    token_type,
                    expires_in,
                    expires_at,
                    refresh_token,
                    scope,
                };

                let _ = self.save_token(&token);
                let mut lock = self.active_token.write().await;
                *lock = Some(token.clone());
                return Ok(token);
            }

            if let Some(error_code) = json.get("error").and_then(|e| e.as_str()) {
                match error_code {
                    "authorization_pending" => {
                        // User hasn't logged in yet, continue waiting
                    }
                    "slow_down" => {
                        poll_interval += std::time::Duration::from_secs(5);
                    }
                    "expired_token" => {
                        return Err(TagisanError::Authentication(
                            "entra_id".to_string(),
                            "Device code has expired. Please initiate sign in again.".to_string(),
                        ));
                    }
                    other => {
                        let desc = json["error_description"].as_str().unwrap_or("");
                        return Err(TagisanError::Authentication(
                            "entra_id".to_string(),
                            format!("Entra ID device code auth error: {other} - {desc}"),
                        ));
                    }
                }
            }

            tokio::time::sleep(poll_interval).await;
        }

        Err(TagisanError::Authentication(
            "entra_id".to_string(),
            "Authentication timed out waiting for user confirmation.".to_string(),
        ))
    }

    /// Acquire token using Client Credentials flow (for daemon / service principals)
    pub async fn acquire_token_client_credentials(&self) -> Result<EntraToken> {
        if self.is_mock() {
            debug!("Acquiring client credentials token (Mock Mode)");
            let token = EntraToken::mock();
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token);
        }

        let secret = self.config.client_secret.as_ref().ok_or_else(|| {
            TagisanError::Authentication(
                "entra_id".to_string(),
                "Client secret not configured for client credentials flow".to_string(),
            )
        })?;

        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", secret.as_str()),
            ("scope", DEFAULT_GRAPH_SCOPE),
        ];

        let res = self
            .client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(TagisanError::Authentication(
                "entra_id".to_string(),
                format!("Client credentials token acquisition failed: {err_text}"),
            ));
        }

        let json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("entra_id".to_string(), "Missing access_token".to_string()))?
            .to_string();
        let token_type = json["token_type"].as_str().unwrap_or("Bearer").to_string();
        let expires_in = json["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = chrono::Utc::now().timestamp() + expires_in;
        let scope = json["scope"].as_str().map(|s| s.to_string());

        let token = EntraToken {
            access_token,
            token_type,
            expires_in,
            expires_at,
            refresh_token: None,
            scope,
        };

        let _ = self.save_token(&token);
        let mut lock = self.active_token.write().await;
        *lock = Some(token.clone());
        Ok(token)
    }

    /// Refresh an existing OAuth2 token using its refresh_token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<EntraToken> {
        if self.is_mock() {
            debug!("Refreshing token in mock mode");
            let now = chrono::Utc::now().timestamp();
            let token = EntraToken {
                access_token: format!("mock_refreshed_entra_token_{}", now),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                expires_at: now + 3600,
                refresh_token: Some(refresh_token.to_string()),
                scope: Some("User.Read ChannelMessage.Send Files.Read.All".to_string()),
            };
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token);
        }

        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        let scopes_joined = self.config.scopes.join(" ");
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", self.config.client_id.as_str()),
            ("refresh_token", refresh_token),
            ("scope", scopes_joined.as_str()),
        ];

        let res = self
            .client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(TagisanError::Authentication(
                "entra_id".to_string(),
                format!("Token refresh failed: {err_text}"),
            ));
        }

        let json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("entra_id".to_string(), "Missing access_token".to_string()))?
            .to_string();
        let token_type = json["token_type"].as_str().unwrap_or("Bearer").to_string();
        let expires_in = json["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = chrono::Utc::now().timestamp() + expires_in;
        let new_refresh_token = json["refresh_token"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| Some(refresh_token.to_string()));
        let scope = json["scope"].as_str().map(|s| s.to_string());

        let token = EntraToken {
            access_token,
            token_type,
            expires_in,
            expires_at,
            refresh_token: new_refresh_token,
            scope,
        };

        let _ = self.save_token(&token);
        let mut lock = self.active_token.write().await;
        *lock = Some(token.clone());
        Ok(token)
    }

    /// Save token to disk with fallback directory creation
    pub fn save_token(&self, token: &EntraToken) -> Result<()> {
        let path = &self.config.token_cache_path;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(token)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load token from disk cache
    pub fn load_token(&self) -> Result<Option<EntraToken>> {
        let path = &self.config.token_cache_path;
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let token: EntraToken = serde_json::from_str(&content)?;
        Ok(Some(token))
    }

    /// Clear persisted token
    pub fn clear_token(&self) -> Result<()> {
        if let Ok(mut lock) = self.active_token.try_write() {
            *lock = None;
        }
        let path = &self.config.token_cache_path;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }

    /// Retrieve a guaranteed valid access token, auto-refreshing if expired
    pub async fn get_valid_token(&self) -> Result<String> {
        // Check active in-memory token
        {
            let lock = self.active_token.read().await;
            if let Some(token) = lock.as_ref() {
                if !token.is_expired() {
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Check disk cache
        let cached = self.load_token()?;
        if let Some(token) = cached {
            if !token.is_expired() {
                let mut lock = self.active_token.write().await;
                *lock = Some(token.clone());
                return Ok(token.access_token);
            }

            // Attempt refresh if refresh_token is available
            if let Some(ref rf) = token.refresh_token {
                info!("Cached Entra ID token is expired, attempting refresh");
                if let Ok(refreshed) = self.refresh_token(rf).await {
                    return Ok(refreshed.access_token);
                }
            }
        }

        if self.is_mock() {
            let token = EntraToken::mock();
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token.access_token);
        }

        // If client secret configured, acquire via client credentials
        if self.config.client_secret.is_some() {
            info!("Acquiring new Entra ID token via Client Credentials");
            let token = self.acquire_token_client_credentials().await?;
            return Ok(token.access_token);
        }

        Err(TagisanError::Authentication(
            "entra_id".to_string(),
            "No valid Microsoft Entra ID token found. Please run 'tgs copilot auth' to login.".to_string(),
        ))
    }

    /// Inspect current authentication status
    pub async fn copilot_auth_status(&self) -> CopilotAuthStatus {
        // Check active token first
        let active = {
            let lock = self.active_token.read().await;
            lock.clone()
        };

        let token = match active {
            Some(t) => Some(t),
            None => self.load_token().ok().flatten(),
        };

        if let Some(t) = token {
            let expired = t.is_expired();
            let mode = if self.is_mock() {
                "mock"
            } else if self.config.client_secret.is_some() {
                "client_credentials"
            } else {
                "device_code"
            };

            CopilotAuthStatus {
                authenticated: !expired,
                tenant_id: self.config.tenant_id.clone(),
                client_id: self.config.client_id.clone(),
                auth_mode: mode.to_string(),
                expires_at: Some(t.expires_at),
                expires_in_secs: Some(t.expires_in_secs()),
                is_expired: expired,
                scope: t.scope,
            }
        } else if self.is_mock() {
            CopilotAuthStatus {
                authenticated: false,
                tenant_id: self.config.tenant_id.clone(),
                client_id: self.config.client_id.clone(),
                auth_mode: "mock".to_string(),
                expires_at: None,
                expires_in_secs: None,
                is_expired: true,
                scope: None,
            }
        } else {
            CopilotAuthStatus {
                authenticated: false,
                tenant_id: self.config.tenant_id.clone(),
                client_id: self.config.client_id.clone(),
                auth_mode: "unauthenticated".to_string(),
                expires_at: None,
                expires_in_secs: None,
                is_expired: true,
                scope: None,
            }
        }
    }
}
