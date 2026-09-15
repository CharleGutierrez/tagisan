//! # Microsoft 365 Copilot & Microsoft Graph Communication System
//!
//! Production-grade integration connecting Tagisan's dialectical reasoning,
//! multi-agent consensus, and formal invariant verification to the Microsoft 365
//! Enterprise Ecosystem (Teams, SharePoint/OneDrive, Outlook, Microsoft Search, and Copilot Studio).

pub mod adr;
pub mod auth;
pub mod bot;
pub mod connector;
pub mod excel;
pub mod graph;
pub mod hardware;
pub mod incident;
pub mod obo;
pub mod planner;
pub mod plugin;
pub mod purview;
pub mod sentinel;
pub mod stream;
pub mod subscriptions;
pub mod throttling;
pub mod tools;

pub use adr::{AdrDocument, AdrEngine, AdrSyncReport, CopilotAdrSyncTool};
pub use auth::{
    CopilotAuthStatus, DeviceCodeResponse, EntraAuthManager, EntraIdConfig, EntraToken,
    DEFAULT_GRAPH_SCOPE, DEFAULT_TOKEN_CACHE_FILE,
};
pub use bot::{TeamsActionPayload, TeamsBotHandler, TeamsCardResponse};
pub use connector::{AclEntry, GraphConnectorEngine, IngestionItem};
pub use excel::{
    export_excel_addin_package, generate_excel_addin_manifest, generate_excel_functions_js,
    generate_excel_functions_json, CopilotExcelFunctionsTool, ExcelAddinPackage, ExcelEvalResult,
    ExcelFunctionsEngine,
};
pub use graph::{ActionItem, DocumentContent, GraphClient, TranscriptEntry};
pub use hardware::{
    AcceleratorType, CopilotHardwareTelemetryTool, HardwareTelemetryEngine, HardwareTelemetryReport,
};
pub use incident::{
    AutofixPatch, CopilotIncidentDebuggerTool, IncidentAnalysis, IncidentCategory,
    IncidentDebuggerEngine, IncidentReport,
};
pub use obo::{ClientCertificateConfig, CopilotOboExchangeTool, OboEngine, UserSecurityContext};
pub use planner::{
    CopilotPlannerSyncTool, PlannerAssignment, PlannerReference, PlannerSyncEngine,
    PlannerSyncReport, PlannerTask, PlannerTaskDetails, ToDoBody, ToDoLinkedResource, ToDoTask,
};
pub use plugin::{
    export_copilot_package, generate_ai_plugin_json, generate_compliance_attestation,
    generate_declarative_agent_manifest, generate_openapi_spec, generate_teams_app_manifest,
    generate_valid_png, CopilotCertifyTool, CopilotPackageInfo,
};
pub use purview::{
    CopilotPurviewGuardTool, CopilotPurviewSyncTool, PurviewAuditReceipt, PurviewGuardEngine,
    PurviewGuardResult, PurviewLabelPolicy, PurviewSensitivity,
};
pub use sentinel::{
    CopilotSentinelAuditTool, SentinelAuditEvent, SentinelAuditEngine, SentinelAuditResult,
    SentinelSeverity,
};
pub use stream::{
    CopilotStreamGateway, CopilotStreamGatewayTool, StreamEventType, StreamFrame, StreamMode,
};
pub use subscriptions::{
    CopilotSubscriptionTool, SubscriptionLifecycleEngine, SubscriptionResource,
    WebhookNotification,
};
pub use throttling::{AdaptiveThrottler, RateLimitPolicy, TokenBucket};
pub use tools::{
    CopilotBlastRadiusReportTool, CopilotCreatePrTool, CopilotDebateDispatchTool,
    CopilotExportDeckTool, CopilotExportReportTool, CopilotMeetingActionItemsTool,
    CopilotMeetingToCodeTool, CopilotSharepointGetTool, CopilotTeamsPostTool,
};

