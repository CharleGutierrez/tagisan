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
pub mod bicep;
pub mod bot;
pub mod cae;
pub mod calendar;
pub mod calling;
pub mod connector;
pub mod dataverse;
pub mod defender;
pub mod delta;
pub mod delta_lake;
pub mod excel;
pub mod fabric;
pub mod graph;
pub mod hardware;
pub mod icm;
pub mod incident;
pub mod jet_binary;
pub mod jwe;
pub mod loop_pages;
pub mod obo;
pub mod ooxml;
pub mod perms_auditor;
pub mod planner;
pub mod plugin;
pub mod power_automate;
pub mod powerplatform;
pub mod powerbi;
pub mod purview;
pub mod sarif;
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
pub mod vscode;
pub mod visio;
pub mod wam;
pub mod ms_enterprise;
pub mod ms_frontier;
pub mod ms_hardened;
pub mod ms_soar_frontier;
pub mod ms_ecosystem_frontier;
pub mod ms_deeptech_frontier;

pub use ms_deeptech_frontier::{
    AutopilotProfile, AzureDigitalTwinsEngine, AzureIotEdgeIndustrialBridge,
    AzureQuantumEngine, AzureSovereignCloudEngine, AzureStackHciBridge,
    BusinessCentralAlEngine, BusinessCentralObject, CiemIdentity, CiemPciReport,
    CopilotMsDeepTechTool, CredentialGuardAuditor, CredentialGuardStatus,
    DeepTechVerifiedCredential, DeepTechVerifiedIdEngine, DisconnectedSyncBundle,
    DmfBatchPackage, DmfEntityDefinition, DtdlContent, DtdlInterface, DualWriteMapping,
    DynamicsDmfBatchEngine, DynamicsDualWriteCoordinator, EntraCiemEngine,
    IntuneAppInfo, IntuneCompliancePolicy, IntuneDetectionRule, IntuneWinPackageResult,
    IntuneWinPackager, OpcUaTelemetryRecord, QuantumArchitecture,
    QuantumResourceEstimationRequest, QuantumResourceReport, SovereignCloudType,
    SovereignEndpoints, Tpm2SecurityEngine, TpmPcrState, TpmSealedEnvelope,
    WdacCodeIntegrityEngine, WdacPolicy,
};

pub use ms_ecosystem_frontier::{
    AzureAiFoundryEngine, AzureApimPolicyEngine, AzureRateLimitState, CopilotMsEcosystemTool,
    DataverseSolutionPackagerEngine, DefenderHuntingRecord, DefenderXdrHuntingEngine,
    FabricLakehouseMedallionEngine, KedaKubernetesScalerEngine, LiveResponseAction,
    SkFunctionDef, SolutionAuditReport, TypeSpecEndpoint, UnpackedSolution,
    WinUiDeepLinkTarget, WindowsNamedPipeIpcEngine,
};

pub use ms_soar_frontier::{
    ArcMachineRecord, AzureEventGridEngine, AzureResourceGraphEngine, CloudEventEnvelope,
    CopilotMsSoarTool, EntraPimEngine, FluentPcfEngine, LogicAppsWorkflowEngine,
    MipRmsCompoundParser, PimRoleRequest, RemediationAction, RmsPfileEnvelope, TsqlAnalysis,
    TsqlInvariantEngine,
};

pub use ms_hardened::{
    AsyncOperationStatus, AsyncOperationTicket, AzureManagedIdentityEngine,
    ClientCertificateAssertion, CopilotMsHardenedTool, DataverseVirtualEntityProvider,
    DeltaLogCommit, FabricDeltaStreamer, GraphExternalConnection, GraphExternalItem,
    GraphItemAcl, GraphPropertyDefinition, ImdsTokenResponse, ManagedIdentityType,
    MsAsyncWebhookEngine, MsGraphConnectorEngine, OfficeJsApp, OfficeJsManifestEngine,
    VirtualEntitySchema, VirtualTableField, WebhookDeliveryPackage,
};

pub use ms_enterprise::{
    A2AAgentCard, A2AAgentRole, A2AConsensusVerdict, A2ADebateRound, A2ADelegationTask,
    AceQueryRequest, AceQueryResult, CaeChallenge, CaeStepUpRequest, CaeZeroTrustGuard,
    CopilotA2ASwarmEngine, DataverseAlmEngine, DataverseCdcEvent, DeltaCommitSummary,
    DesktopAppType, DirectLakeSemanticModel, DirectLineActivity, DriftSeverity, FabricOneLakeEngine,
    FabricRestDeploymentPlan, MsDesktopRuntimeBridge, OneLakeAbfsPath, RotEntry, SchemaDriftReport,
    ShapeSheetEvalRequest, ShapeSheetEvalResult, SolutionComponentType, SolutionManifest,
    VbaExecutionRequest, VbaExecutionResult, VirtualTableConfig, ZeroTrustToken,
};

pub use ms_frontier::{
    AmqpFrameType, AmqpMessage, AudioPacket, TeamsCallParticipant, CallSessionState,
    ComplianceEvaluationResult, CompliancePolicyRule, CredentialValidationReceipt,
    CryptographicProof, DispositionStatus, EntraVerifiedIdEngine, IntuneDevicePosture,
    MsSecurityCopilotIntuneEngine, OleBinaryForensicsEngine, OleDirectoryEntry,
    OleForensicReport, OleObjectType, SecurityCopilotManifest, SecurityCopilotSkill,
    SpeechInvariantAlert, TeamsRealTimeMediaEngine, VerifiableCredential,
    VerifiablePresentation, AzureServiceBusAmqpEngine, OLE_MAGIC,
};

pub use bicep::{
    AvmComplianceChecker, AvmComplianceReport, AvmRuleViolation, BicepAuditReport,
    BicepEngine, BicepEvaluationContext, BicepFunctionEvaluator, BicepGraph,
    BicepResource, BicepToArmTranspiler, CopilotBicepTool, IacBlastRadiusReport, InvariantViolation,
};
pub use defender::{
    AutomatedRemediationPrGenerator, CopilotDefenderTool, DefenderCveAlert, DefenderEngine,
    DefenderRemediationAction, DefenderWebhookGateway, ReachabilityStatus, RemediationPullRequest,
    RiskTelemetry, SentinelKqlRule, SentinelKqlRuleGenerator, TriageFinding, VirtualPatch,
    WebhookTriageResult, compute_hmac_sha256, verify_client_state_hmac,
};
pub use powerbi::{
    CopilotPowerBiTool, LakehouseMaintenanceCommands, PowerBiEngine, TmdlColumn, TmdlDatabase,
    TmdlMeasure, TmdlPartition, TmdlRelationship, TmdlTable, TmdlValidationResult, TmslEngine,
    TmslRefreshType, TmslTargetObject, XmlaDeploymentPackage,
};
pub use vscode::{CopilotChatRequest, CopilotChatResponse, CopilotVsCodeTool, GutterDecorationPayload, LspCodeAction, LspCodeLens, LspDiagnostic, LspDocumentHighlight, LspPosition, LspRange, VsCodeEngine, VsCodeExtensionManifestGenerator};

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
    AdaptiveCardUniversalAction, CopilotUniversalActionTool, MessagingExtensionAttachment,
    MessagingExtensionResponse, MessagingExtensionResult, TeamsActionPayload, TeamsBotHandler,
    TeamsCardResponse, TeamsMessageExtensionHandler, UniversalActionPayload,
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
    AtlasEntity, AtlasEntityWithExtInfo, AtlasLineageInfo, AtlasLineageRelation, AtlasObjectId,
    CopilotPurviewGuardTool, CopilotPurviewSyncTool, CopilotPurviewTool, CopilotRmsGuardTool,
    PurviewAuditReceipt, PurviewDataMapEngine, PurviewGuardEngine, PurviewGuardResult,
    PurviewLabelPolicy, PurviewSensitivity, RmsProtectionHandler, RmsProtectionStatus,
};
pub use sentinel::{
    CopilotSentinelAuditTool, CopilotSentinelTool, KqlHuntingCatalog, KqlHuntingQuery, KustoClient,
    KustoColumn, KustoExecutionEngine, KustoQueryResult, KustoRow, KustoTable, SentinelAuditEvent,
    SentinelAuditEngine, SentinelAuditResult, SentinelBridgeEngine, SentinelEventType,
    SentinelSecurityEvent, SentinelSeverity,
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

pub use ado::{
    AdoComment, AdoCommentPosition, AdoCommentThread, AdoConfig, AdoEngine, AdoFederatedTokenRequest,
    AdoFederatedTokenResponse, AdoGitStatus, AdoGitStatusContext, AdoGitStatusState, AdoPullRequest,
    AdoRelation, AdoThreadStatus, AdoWorkItem, CopilotAdoSyncTool, CopilotAdoTool,
};
pub use icm::{
    CopilotIcmBridgeTool, CopilotIcmTool, CorrelationConfidence, GitCommitInfo, IcmEngine,
    IcmIncident, IcmRollbackPr, IcmRollbackPrGenerator, IcmSeverity, IncidentCorrelationResult,
    PirTimelineEntry, PostIncidentReview,
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
pub use viva::{CopilotVivaSyncTool, CopilotVivaTool, Viva1on1Briefing, VivaEngine, VivaGoal, VivaGoalsClient};
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
    CopilotPowerPlatformPackagerTool, PcfControlConfig, PcfControlGenerator, PcfPackage,
    PowerAutomateFlow, PowerAutomateFlowConfig, PowerAutomateTranspiler,
    PowerPlatformPackagerEngine, PowerPlatformSolutionReport,
};
pub use access::{
    AccessBlastRadiusAnalyzer, AccessBlastRadiusReport, AccessDataType, AccessDataverseMigrator,
    AccessFormDefinition, AccessFormReportGenerator, AccessReportDefinition, AccessSchemaEngine,
    AccessVbaBridge, BlastImpactItem, ColumnDefinition, CopilotAccessTool, DataverseMigrationPlan,
    ForeignKeyDefinition, FormReportControl, FormSection, IndexDefinition, ReferentialRule,
    ReportSection, SchemaValidationReport, TableDefinition, TranspileResult, AceSqlTranspiler,
    TWIPS_PER_CM, TWIPS_PER_INCH, TWIPS_PER_POINT,
};
pub use visio::{
    ArrowType, AzureResource, AzureResourceType, AzureSecurityCallout, AzureTopologyEngine,
    BlastDependency, BlastNode, BlastRadiusEngine, BlastRiskTier, BpmnConsensusEngine,
    C4DiagramType, C4Element, C4ElementType, C4Model, C4ModelEngine, C4Relationship,
    ConnectorType, ConsensusRound, CopilotVisioTool, DataVisualizerEngine, DataVisualizerRow,
    ErdCardinality, ErdColumn, ErdEngine, ErdRelationship, ErdTable, ShapeType, VisioConnector,
    VisioDocument, VisioPackageVerification, VisioPackager, VisioPage, VisioProperty,
    VisioPropertyType, VisioShape, VisioTranspiler,
};

pub use sarif::{
    AzurePipelinesConfig, AzurePipelinesGenerator, CopilotSarifTool, SarifArtifactLocation,
    SarifCodeFlow, SarifDocument, SarifDriver, SarifEngine, SarifHelp, SarifInvocation,
    SarifLevel, SarifLocation, SarifMessage, SarifPhysicalLocation, SarifRegion, SarifResult,
    SarifRule, SarifRuleConfig, SarifRun, SarifSnippet, SarifThreadFlow, SarifThreadFlowLocation,
    SarifTool, DEFAULT_TOOL_INFO_URI, DEFAULT_TOOL_NAME, DEFAULT_TOOL_VERSION, SARIF_SCHEMA_2_1_0,
    SARIF_VERSION,
};
pub use jet_binary::{
    CopilotJetBinaryTool, JetBinaryEngine, JetColumnDef, JetColumnType, JetDatabase,
    JetHeaderInfo, JetIntegrityReport, JetTableDef, JetValue, JetVersion, ACE_FORMAT_STRING,
    JET_FORMAT_STRING, JET_MAGIC_PREFIX, PAGE_SIZE_ACE, PAGE_SIZE_JET3, PAGE_TYPE_DATA,
    PAGE_TYPE_INDEX, PAGE_TYPE_LEAF, PAGE_TYPE_TDEF, PAGE_TYPE_USAGE_MAP,
};
pub use calling::{
    AdaptiveCardIntervention, AudioStreamStats, CallParticipant, CallSession, CallState,
    CopilotCallingTool, TeamsCallingEngine, TranscriptSegment, TriggerDetection, WebRtcNegotiator,
};
pub use delta_lake::{
    CopilotDeltaLakeTool, DeltaActionEnvelope, DeltaAddAction, DeltaCommitInfo, DeltaFileStats,
    DeltaFormatSpec, DeltaLakeEngine, DeltaMetaData, DeltaProtocol, DeltaRemoveAction,
    DeltaTableSnapshot, OneLakePathMapping, DEFAULT_ENGINE_INFO, ONELAKE_BLOB_BASE_URL,
    ONELAKE_DFS_BASE_URL,
};

