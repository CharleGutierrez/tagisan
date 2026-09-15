//! # Microsoft 365 Copilot & Microsoft Graph Communication System
//!
//! Production-grade integration connecting Tagisan's dialectical reasoning,
//! multi-agent consensus, and formal invariant verification to the Microsoft 365
//! Enterprise Ecosystem (Teams, SharePoint/OneDrive, Outlook, Microsoft Search, and Copilot Studio).

pub mod adr;
pub mod auth;
pub mod bot;
pub mod connector;
pub mod graph;
pub mod plugin;
pub mod purview;
pub mod tools;

pub use adr::{AdrDocument, AdrEngine, AdrSyncReport, CopilotAdrSyncTool};
pub use auth::{
    CopilotAuthStatus, DeviceCodeResponse, EntraAuthManager, EntraIdConfig, EntraToken,
    DEFAULT_GRAPH_SCOPE, DEFAULT_TOKEN_CACHE_FILE,
};
pub use bot::{TeamsActionPayload, TeamsBotHandler, TeamsCardResponse};
pub use connector::{AclEntry, GraphConnectorEngine, IngestionItem};
pub use graph::{ActionItem, DocumentContent, GraphClient, TranscriptEntry};
pub use plugin::{
    export_copilot_package, generate_ai_plugin_json, generate_declarative_agent_manifest,
    generate_openapi_spec, generate_teams_app_manifest, generate_valid_png, CopilotPackageInfo,
};
pub use purview::{
    CopilotPurviewGuardTool, PurviewAuditReceipt, PurviewGuardEngine, PurviewGuardResult,
    PurviewSensitivity,
};
pub use tools::{
    CopilotBlastRadiusReportTool, CopilotCreatePrTool, CopilotDebateDispatchTool,
    CopilotExportDeckTool, CopilotExportReportTool, CopilotMeetingActionItemsTool,
    CopilotMeetingToCodeTool, CopilotSharepointGetTool, CopilotTeamsPostTool,
};
