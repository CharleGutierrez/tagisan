//! # Microsoft 365 Copilot & Microsoft Graph Communication System
//!
//! Production-grade integration connecting Tagisan's dialectical reasoning,
//! multi-agent consensus, and formal invariant verification to the Microsoft 365
//! Enterprise Ecosystem (Teams, SharePoint/OneDrive, Outlook, Microsoft Search, and Copilot Studio).

pub mod access;
pub mod adr;
pub mod ado;
pub mod airgap;
pub mod auth;
pub mod batch;
pub mod bot;
pub mod cae;
pub mod calendar;
pub mod connector;
pub mod dataverse;
pub mod delta;
pub mod excel;
pub mod fabric;
pub mod graph;
pub mod hardware;
pub mod icm;
pub mod incident;
pub mod jwe;
pub mod loop_pages;
pub mod obo;
pub mod ooxml;
pub mod perms_auditor;
pub mod planner;
pub mod plugin;
pub mod power_automate;
pub mod powerplatform;
pub mod purview;
pub mod sdl;
pub mod sentinel;
pub mod sharepoint_crawler;
pub mod stream;
pub mod studio;
pub mod subscriptions;
pub mod substrate;
pub mod throttling;
pub mod tools;
pub mod viva;
pub mod wam;

pub use adr::{AdrDocument, AdrEngine, AdrSyncReport, CopilotAdrSyncTool};
pub use auth::{
    CopilotAuthStatus, CopilotWorkloadIdentityTool, DeviceCodeResponse, EntraAuthManager,
    EntraIdConfig, EntraToken, MicrosoftCloud, DEFAULT_GRAPH_SCOPE, DEFAULT_TOKEN_CACHE_FILE,
};
pub use batch::{
    BatchEngine, BatchRequest, BatchResponse, BatchSubRequest, BatchSubResponse,
    CopilotGraphBatchTool, GRAPH_BATCH_MAX_LIMIT,
};
pub use bot::{
    TeamsActionPayload, TeamsBotHandler, TeamsCardResponse, UniversalActionPayload,
    CopilotUniversalActionTool,
};
pub use cae::{
    CaeClaimsChallenge, CaeRiskLevel, CopilotCaeHandlerTool,
};
pub use connector::{AclEntry, GraphConnectorEngine, IngestionItem};
pub use delta::{
    CopilotDeltaSyncTool, DeltaChangeItem, DeltaChangeType, DeltaSyncEngine, DeltaSyncReport,
};
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
pub use jwe::{
    CopilotJweDecryptTool, DecryptedGraphResource, GraphEncryptedContent, JweDecryptor,
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
    CopilotPurviewGuardTool, CopilotPurviewSyncTool, CopilotRmsGuardTool, PurviewAuditReceipt,
    PurviewGuardEngine, PurviewGuardResult, PurviewLabelPolicy, PurviewSensitivity,
    RmsProtectionHandler, RmsProtectionStatus,
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

pub use airgap::{AirgapAuditReceipt, AirgapDecision, AirgapRouter, CopilotAirgapRouterTool};
pub use calendar::{
    CalendarEngine, CalendarEvent, CopilotCalendarPreReadTool, CopilotOutlookDraftTool,
    DraftResult, LinkedPrInfo, PreReadBrief,
};
pub use fabric::{
    CopilotFabricQueryTool, DaxQueryRequest, DaxQueryResult, FabricEngine, FabricWorkspace,
    OneLakeTable,
};
pub use loop_pages::{
    CopilotLoopSyncTool, LoopChecklistItem, LoopComponent, LoopComponentType, LoopPagesEngine,
    LoopSyncAction, LoopSyncResult,
};
pub use ooxml::{
    calculate_crc32, CalloutSeverity, CopilotOoxmlGeneratorTool, DocxCallout, DocxCustomStyle,
    DocxSection, DocxTable, OoxmlEngine, OoxmlExportReport, PptxMetricCard, PptxSlide,
    ZipBuilder, ZipPackageVerification,
};
pub use perms_auditor::{
    CopilotPermsAuditorTool, GraphPermission, GraphPermissionType, ScopeAuditReport,
    ScopeAuditorEngine,
};
pub use sharepoint_crawler::{
    CopilotSharepointCrawlerTool, CrawlReport, CrawledDocument, DocumentChunk,
    SharePointCrawlerEngine, SharePointSiteCrawlerConfig,
};

pub use ado::{AdoConfig, AdoEngine, AdoPullRequest, AdoRelation, AdoWorkItem, CopilotAdoSyncTool};
pub use icm::{
    IcmEngine, IcmIncident, IcmSeverity, PirTimelineEntry, PostIncidentReview,
    CopilotIcmBridgeTool,
};
pub use sdl::{
    CredScanFinding, PoliCheckFinding, SbomPackage, SdlAuditReport, SdlEngine,
    CopilotSdlAuditTool,
};
pub use studio::{CopilotStudioEngine, CopilotStudioPackageReport, CopilotStudioPackagerTool};
pub use substrate::{
    SubstrateAcl, SubstrateConnection, SubstrateContent, SubstrateEngine, SubstrateIngestReport,
    SubstrateItem, SubstratePropertySchema, CopilotSubstrateIngestTool,
};
pub use viva::{Viva1on1Briefing, VivaEngine, VivaGoal, CopilotVivaSyncTool};
pub use wam::{WamAccount, WamBrokerEngine, WamTokenRequest, WamTokenResponse, CopilotWamAuthTool};
pub use dataverse::{
    CopilotDataverseSyncTool, DataverseAdrRecord, DataverseBatchReport, DataverseBatchSubRequest,
    DataverseBlastRadiusRecord, DataverseEngine, DataverseIncidentRecord, DEFAULT_DATAVERSE_API_VERSION,
    ENTITY_SET_ADR, ENTITY_SET_BLAST_RADIUS, ENTITY_SET_INCIDENT,
};
pub use power_automate::{
    CopilotPowerAutomateTool, FlowRunResult, FlowTriggerType, PowerAutomateEngine,
    DEFAULT_FLOW_HMAC_SECRET,
};
pub use powerplatform::{
    CopilotPowerPlatformPackagerTool, PowerPlatformPackagerEngine, PowerPlatformSolutionReport,
};
pub use access::{
    AccessBlastRadiusAnalyzer, AccessBlastRadiusReport, AccessDataType, AccessDataverseMigrator,
    AccessFormDefinition, AccessFormReportGenerator, AccessReportDefinition, AccessSchemaEngine,
    AccessVbaBridge, BlastImpactItem, ColumnDefinition, CopilotAccessTool, DataverseMigrationPlan,
    ForeignKeyDefinition, FormReportControl, FormSection, IndexDefinition, ReferentialRule,
    ReportSection, SchemaValidationReport, TableDefinition, TranspileResult, AceSqlTranspiler,
    TWIPS_PER_CM, TWIPS_PER_INCH, TWIPS_PER_POINT,
};




