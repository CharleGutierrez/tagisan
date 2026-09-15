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

/// Microsoft Cloud Deployment Environment (Sovereign & National Clouds)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicrosoftCloud {
    Commercial,
    UsGovGccHigh,
    UsGovDoD,
    China21Vianet,
}

impl Default for MicrosoftCloud {
    fn default() -> Self {
        Self::Commercial
    }
}

impl MicrosoftCloud {
    pub fn login_host(&self) -> &'static str {
        match self {
            Self::Commercial => "login.microsoftonline.com",
            Self::UsGovGccHigh | Self::UsGovDoD => "login.microsoftonline.us",
            Self::China21Vianet => "login.chinacloudapi.cn",
        }
    }

    pub fn graph_host(&self) -> &'static str {
        match self {
            Self::Commercial => "graph.microsoft.com",
            Self::UsGovGccHigh => "graph.microsoft.us",
            Self::UsGovDoD => "dod-graph.microsoft.us",
            Self::China21Vianet => "microsoftgraph.chinacloudapi.cn",
        }
    }

    pub fn graph_base_url(&self) -> String {
        format!("https://{}/v1.0", self.graph_host())
    }

    pub fn authorize_endpoint(&self, tenant: &str) -> String {
        format!("https://{}/{}/oauth2/v2.0/authorize", self.login_host(), tenant)
    }

    pub fn token_endpoint(&self, tenant: &str) -> String {
        format!("https://{}/{}/oauth2/v2.0/token", self.login_host(), tenant)
    }

    pub fn devicecode_endpoint(&self, tenant: &str) -> String {
        format!("https://{}/{}/oauth2/v2.0/devicecode", self.login_host(), tenant)
    }
}

/// Configuration for Microsoft Entra ID Authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntraIdConfig {
    /// Azure / Entra Application (Client) ID
    pub client_id: String,
    /// Azure / Entra Directory (Tenant) ID (e.g. "common", "organizations", or GUID)
    pub tenant_id: String,
    /// Client Secret (for Client Credentials flow)
    pub client_secret: Option<String>,
    /// Cloud Environment (Commercial, US Gov GCC High, DoD, 21Vianet China)
    #[serde(default)]
    pub cloud: MicrosoftCloud,
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

        let cloud = match std::env::var("AZURE_CLOUD").or_else(|_| std::env::var("TGS_MICROSOFT_CLOUD")).as_deref() {
            Ok(val) if val.eq_ignore_ascii_case("usgov") || val.eq_ignore_ascii_case("gcchigh") => MicrosoftCloud::UsGovGccHigh,
            Ok(val) if val.eq_ignore_ascii_case("dod") => MicrosoftCloud::UsGovDoD,
            Ok(val) if val.eq_ignore_ascii_case("china") || val.eq_ignore_ascii_case("21vianet") => MicrosoftCloud::China21Vianet,
            _ => MicrosoftCloud::Commercial,
        };

        let mock = std::env::var("TGS_MOCK_ENTRA")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let token_cache_path = PathBuf::from(DEFAULT_TOKEN_CACHE_FILE);

        Self {
            client_id,
            tenant_id,
            client_secret,
            cloud,
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

    /// Save token to disk with atomic write and fallback directory creation
    pub fn save_token(&self, token: &EntraToken) -> Result<()> {
        let path = &self.config.token_cache_path;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let json = serde_json::to_string_pretty(token)?;
        let nonce = blake3::hash(format!("{}_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), std::process::id()).as_bytes()).to_hex();
        let temp_path = path.with_extension(format!("tmp.{}", &nonce[..12]));
        if std::fs::write(&temp_path, &json).is_ok() {
            let _ = std::fs::rename(&temp_path, path);
        } else {
            let _ = std::fs::write(path, json);
        }
        Ok(())
    }

    /// Retrieve a guaranteed valid access token (alias for get_valid_token)
    pub async fn get_access_token(&self) -> Result<String> {
        self.get_valid_token().await
    }

    /// Load token from disk cache safely handling concurrent reads
    pub fn load_token(&self) -> Result<Option<EntraToken>> {
        let path = &self.config.token_cache_path;
        if !path.exists() {
            return Ok(None);
        }

        // Try reading up to 3 times to mitigate transient concurrent rename windows
        for _ in 0..3 {
            if let Ok(content) = std::fs::read_to_string(path) {
                if !content.trim().is_empty() {
                    if let Ok(token) = serde_json::from_str::<EntraToken>(&content) {
                        return Ok(Some(token));
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        Ok(None)
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

    /// Acquire token via Azure Instance Metadata Service (IMDS) Managed Identity
    pub async fn acquire_token_managed_identity(&self, client_id: Option<&str>) -> Result<EntraToken> {
        if self.is_mock() {
            debug!("Acquiring token via Managed Identity in mock mode");
            let now = chrono::Utc::now().timestamp();
            let token = EntraToken {
                access_token: format!("mock_msi_entra_token_{}", now),
                token_type: "Bearer".to_string(),
                expires_in: 86400,
                expires_at: now + 86400,
                refresh_token: None,
                scope: Some(DEFAULT_GRAPH_SCOPE.to_string()),
            };
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token);
        }

        let mut url = format!(
            "http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01&resource={}",
            urlencoding::encode(DEFAULT_GRAPH_SCOPE)
        );
        if let Some(cid) = client_id {
            url.push_str(&format!("&client_id={}", cid));
        }

        let res = self
            .client
            .get(&url)
            .header("Metadata", "true")
            .send()
            .await
            .map_err(TagisanError::Network)?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(TagisanError::Authentication(
                "managed_identity".to_string(),
                format!("IMDS token request failed: {err_text}"),
            ));
        }

        let json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("managed_identity".to_string(), "Missing access_token".to_string()))?
            .to_string();
        let token_type = json["token_type"].as_str().unwrap_or("Bearer").to_string();
        let expires_in = json["expires_in"].as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(3600);
        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        let token = EntraToken {
            access_token,
            token_type,
            expires_in,
            expires_at,
            refresh_token: None,
            scope: Some(DEFAULT_GRAPH_SCOPE.to_string()),
        };

        let _ = self.save_token(&token);
        let mut lock = self.active_token.write().await;
        *lock = Some(token.clone());
        Ok(token)
    }

    /// Acquire token via RFC 7523 Workload Identity Federation (GitHub Actions / Azure DevOps OIDC)
    pub async fn acquire_token_federated_identity(&self, assertion: &str) -> Result<EntraToken> {
        if self.is_mock() {
            debug!("Acquiring token via Workload Identity Federation in mock mode");
            let now = chrono::Utc::now().timestamp();
            let token = EntraToken {
                access_token: format!("mock_federated_entra_token_{}", now),
                token_type: "Bearer".to_string(),
                expires_in: 3600,
                expires_at: now + 3600,
                refresh_token: None,
                scope: Some(DEFAULT_GRAPH_SCOPE.to_string()),
            };
            let _ = self.save_token(&token);
            let mut lock = self.active_token.write().await;
            *lock = Some(token.clone());
            return Ok(token);
        }

        let token_url = self.config.cloud.token_endpoint(&self.config.tenant_id);
        let params = [
            ("scope", DEFAULT_GRAPH_SCOPE),
            ("client_id", self.config.client_id.as_str()),
            ("client_assertion_type", "urn:ietf:params:oauth:client-assertion-type:jwt-bearer"),
            ("client_assertion", assertion),
            ("grant_type", "client_credentials"),
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
                "workload_identity".to_string(),
                format!("Federated credential exchange failed: {err_text}"),
            ));
        }

        let json: serde_json::Value = res.json().await.map_err(TagisanError::Network)?;
        let access_token = json["access_token"]
            .as_str()
            .ok_or_else(|| TagisanError::Authentication("workload_identity".to_string(), "Missing access_token".to_string()))?
            .to_string();
        let token_type = json["token_type"].as_str().unwrap_or("Bearer").to_string();
        let expires_in = json["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = chrono::Utc::now().timestamp() + expires_in;

        let token = EntraToken {
            access_token,
            token_type,
            expires_in,
            expires_at,
            refresh_token: None,
            scope: Some(DEFAULT_GRAPH_SCOPE.to_string()),
        };

        let _ = self.save_token(&token);
        let mut lock = self.active_token.write().await;
        *lock = Some(token.clone());
        Ok(token)
    }
}

// Simple URL encoding helper
mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut encoded = String::with_capacity(data.len() * 2);
        for byte in data.bytes() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}

// =========================================================================
// Tool 28: CopilotWorkloadIdentityTool (copilot_workload_identity)
// =========================================================================

/// Autonomous tool for Azure Workload Identity Federation & Managed Identity
#[derive(Clone)]
pub struct CopilotWorkloadIdentityTool {
    manager: std::sync::Arc<EntraAuthManager>,
}

impl Default for CopilotWorkloadIdentityTool {
    fn default() -> Self {
        Self {
            manager: std::sync::Arc::new(EntraAuthManager::mock()),
        }
    }
}

impl CopilotWorkloadIdentityTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_manager(manager: std::sync::Arc<EntraAuthManager>) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl crate::tools::ToolHandler for CopilotWorkloadIdentityTool {
    fn name(&self) -> &'static str {
        "copilot_workload_identity"
    }

    fn description(&self) -> &'static str {
        "Acquire Microsoft Entra ID tokens via passwordless Azure Managed Identity (IMDS) or RFC 7523 Workload Identity Federation (GitHub/DevOps OIDC)"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["managed_identity", "federated_credential", "status"],
                    "description": "Authentication mechanism to invoke"
                },
                "assertion": {
                    "type": "string",
                    "description": "OIDC JWT bearer assertion token for federated credential exchange"
                },
                "client_id": {
                    "type": "string",
                    "description": "Optional user-assigned Managed Identity client ID"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String> {
        let action = arguments
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("status");

        match action {
            "managed_identity" => {
                let cid = arguments.get("client_id").and_then(|v| v.as_str());
                let token = self.manager.acquire_token_managed_identity(cid).await?;
                Ok(format!(
                    "### 🔐 Azure Managed Identity Token Acquired\n\n\
                    - **Auth Mode:** IMDS Managed Identity (Zero-Secret)\n\
                    - **Token Type:** {}\n\
                    - **Expires In:** {}s\n\
                    - **Target Scope:** {}\n\
                    - **Token Fingerprint:** `sha256_{}`\n",
                    token.token_type,
                    token.expires_in,
                    token.scope.as_deref().unwrap_or("default"),
                    &blake3::hash(token.access_token.as_bytes()).to_hex()[..16]
                ))
            }
            "federated_credential" => {
                let assertion = arguments
                    .get("assertion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("mock_oidc_assertion_github_actions");
                let token = self.manager.acquire_token_federated_identity(assertion).await?;
                Ok(format!(
                    "### 🔐 Workload Identity Federation Token Exchanged\n\n\
                    - **Auth Mode:** RFC 7523 Federated OIDC Exchange\n\
                    - **Token Type:** {}\n\
                    - **Expires In:** {}s\n\
                    - **Target Scope:** {}\n\
                    - **Token Fingerprint:** `sha256_{}`\n",
                    token.token_type,
                    token.expires_in,
                    token.scope.as_deref().unwrap_or("default"),
                    &blake3::hash(token.access_token.as_bytes()).to_hex()[..16]
                ))
            }
            _ => {
                let status = self.manager.copilot_auth_status().await;
                Ok(format!(
                    "### 🛡️ Entra ID Workload Identity Status\n\n\
                    - **Authenticated:** {}\n\
                    - **Tenant ID:** `{}`\n\
                    - **Client ID:** `{}`\n\
                    - **Cloud Environment:** `{:?}`\n\
                    - **Auth Mode:** `{}`\n",
                    status.authenticated,
                    status.tenant_id,
                    status.client_id,
                    self.manager.config().cloud,
                    status.auth_mode
                ))
            }
        }
    }
}
