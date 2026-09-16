//! # Brutal Verification Test Suite: Microsoft Platform Strengthening
//!
//! Exhaustive test suite verifying the 7 strengthened Microsoft tech stack subsystems:
//! 1. Azure DevOps (ADO): PR review threads, line/col positions, status checks, OIDC federation, state transitions.
//! 2. Microsoft IcM: dynamic commit correlation, live-site rollback PR generator, schema v2 parsing, 5-Whys PIR.
//! 3. Microsoft Sentinel: Kusto direct execution, tabular parser with all typed columns, hunting catalog.
//! 4. Microsoft Purview: Apache Atlas models, lineage graph relations, Access 365 -> Dataverse -> OneLake -> Power BI 4-tier lineage.
//! 5. Microsoft Viva: dynamic 1:1 briefing generation, Graph Viva Goals OKR payloads.
//! 6. Teams Bot: search-based Message Extension router (`composeExtension/query`), Universal Actions with user-specific views.
//! 7. Power BI: Direct Lake TMDL generator, Fabric Lakehouse maintenance SQL (`VACUUM`, `OPTIMIZE`).
//! 8. 50-worker concurrency stress test and malformed payload injection fuzzing.

use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tagisan::copilot::ado::*;
use tagisan::copilot::bot::*;
use tagisan::copilot::icm::*;
use tagisan::copilot::powerbi::*;
use tagisan::copilot::purview::*;
use tagisan::copilot::sentinel::*;
use tagisan::copilot::viva::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. Azure DevOps (ADO) Tests
// =========================================================================

#[tokio::test]
async fn test_01_ado_pr_review_threads_creation_and_listing() {
    let engine = AdoEngine::default();
    let thread = engine
        .create_pull_request_thread(
            "Tagisan",
            101,
            "src/copilot/auth.rs",
            42,
            15,
            "Consider caching the token response to prevent rate limiting.",
            "reviewer@microsoft.com",
        )
        .await
        .expect("create thread failed");

    assert_eq!(thread.id, 1);
    assert_eq!(thread.status, AdoThreadStatus::Active);
    assert_eq!(thread.comments.len(), 1);
    assert_eq!(thread.comments[0].content, "Consider caching the token response to prevent rate limiting.");
    assert_eq!(thread.comments[0].author, "reviewer@microsoft.com");

    let pos = thread.thread_context.expect("missing thread position");
    assert_eq!(pos.file_path, "src/copilot/auth.rs");
    assert_eq!(pos.line, 42);
    assert_eq!(pos.offset, 15);

    let threads = engine.list_pull_request_threads("Tagisan", 101).await.expect("list threads failed");
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, 1);
}

#[tokio::test]
async fn test_02_ado_pr_review_threads_reply_and_status_transitions() {
    let engine = AdoEngine::default();
    let thread = engine
        .create_pull_request_thread(
            "Tagisan",
            102,
            "src/copilot/throttling.rs",
            88,
            4,
            "Initial review comment",
            "lead@microsoft.com",
        )
        .await
        .expect("create thread failed");

    let reply = engine
        .add_comment_to_thread(
            "Tagisan",
            102,
            thread.id,
            "Fixed in commit c4f891a with adaptive token bucket.",
            "developer@microsoft.com",
        )
        .await
        .expect("reply failed");

    assert_eq!(reply.id, 2);
    assert_eq!(reply.parent_comment_id, 1);

    // Transition Active -> Fixed
    let updated = engine
        .update_thread_status("Tagisan", 102, thread.id, AdoThreadStatus::Fixed)
        .await
        .expect("update status to Fixed failed");
    assert_eq!(updated.status, AdoThreadStatus::Fixed);
    assert_eq!(updated.status.as_str(), "fixed");
    assert_eq!(updated.comments.len(), 2);

    // Transition Fixed -> Closed
    let closed = engine
        .update_thread_status("Tagisan", 102, thread.id, AdoThreadStatus::Closed)
        .await
        .expect("update status to Closed failed");
    assert_eq!(closed.status, AdoThreadStatus::Closed);
    assert_eq!(closed.status.as_str(), "closed");
}

#[tokio::test]
async fn test_03_ado_git_status_check_policy_gates() {
    let engine = AdoEngine::default();

    // Post Pending
    let s_pending = engine
        .post_pull_request_status(
            "Tagisan",
            103,
            AdoGitStatusState::Pending,
            "Tagisan/BlastRadius",
            "continuous-integration",
            "Calculating AST blast radius...",
            None,
        )
        .await
        .expect("post status failed");
    assert_eq!(s_pending.state, AdoGitStatusState::Pending);

    // Post Succeeded
    let s_succeeded = engine
        .post_pull_request_status(
            "Tagisan",
            103,
            AdoGitStatusState::Succeeded,
            "Tagisan/BlastRadius",
            "continuous-integration",
            "AST Blast Radius check passed: 0 breaking changes",
            Some("https://dev.azure.com/mseng/1ES-Tagisan/build/12345"),
        )
        .await
        .expect("post status failed");
    assert_eq!(s_succeeded.state, AdoGitStatusState::Succeeded);
    assert_eq!(s_succeeded.context.name, "Tagisan/BlastRadius");
    assert_eq!(s_succeeded.context.genre, "continuous-integration");

    let statuses = engine.list_pull_request_statuses("Tagisan", 103).await.expect("list statuses failed");
    assert_eq!(statuses.len(), 2);
}

#[tokio::test]
async fn test_04_ado_workload_identity_federation_oidc_exchange() {
    let engine = AdoEngine::default();
    let req = AdoFederatedTokenRequest {
        oidc_token: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxRVNfVGFnaXNhbiJ9.signature".to_string(),
        client_id: "e6f8c491-1111-2222-3333-444455556666".to_string(),
        tenant_id: "72f988bf-86f1-41af-91ab-2d7cd011db47".to_string(),
        scope: "499b84ac-1321-427f-aa17-267ca6975798/.default".to_string(),
    };

    let resp = engine.exchange_federated_token(&req).await.expect("token exchange failed");
    assert_eq!(resp.token_type, "Bearer");
    assert!(resp.access_token.contains("eyJhbGciOiJSUzI1NiI"));
    assert_eq!(resp.expires_in, 3600);
    assert_eq!(resp.scope, req.scope);

    // Empty token failure test
    let bad_req = AdoFederatedTokenRequest {
        oidc_token: "".to_string(),
        client_id: "e6f8c491".to_string(),
        tenant_id: "72f988bf".to_string(),
        scope: "scope".to_string(),
    };
    assert!(engine.exchange_federated_token(&bad_req).await.is_err());
}

#[tokio::test]
async fn test_05_ado_work_item_state_transitions_with_git_commit_links() {
    let engine = AdoEngine::default();
    let wi = engine
        .create_work_item("Bug", "Token bucket overflow on batch calls", "Repro steps...", 1, None, None, &[])
        .await
        .expect("create work item failed");

    assert_eq!(wi.state, "New");

    // Transition to Resolved with Git commit link
    let transitioned = engine
        .transition_work_item_state(
            wi.id,
            "Resolved",
            Some("d3a8b29c1e"),
            Some("Fixed burst rate capacity in commit d3a8b29c1e"),
        )
        .await
        .expect("transition failed");

    assert_eq!(transitioned.state, "Resolved");
    assert_eq!(transitioned.relations.len(), 1);
    assert_eq!(transitioned.relations[0].rel, "ArtifactLink");
    assert_eq!(transitioned.relations[0].url, "vstfs:///Git/Commit/d3a8b29c1e");
}

#[tokio::test]
async fn test_06_copilot_ado_tool_actions_execution() {
    let tool = CopilotAdoTool::default();

    // 1. create_work_item
    let r1 = tool
        .execute(json!({
            "action": "create_work_item",
            "work_item_type": "Task",
            "title": "Enforce PR thread review checks",
            "priority": 1
        }))
        .await
        .expect("create_work_item failed");
    assert!(r1.contains("Azure DevOps Work Item Created"));

    // 2. create_pr_thread
    let r2 = tool
        .execute(json!({
            "action": "create_pr_thread",
            "pull_request_id": 501,
            "file_path": "src/copilot/ado.rs",
            "line": 10,
            "offset": 5,
            "comment": "Verify thread policy enforcement."
        }))
        .await
        .expect("create_pr_thread failed");
    assert!(r2.contains("Azure Repos PR Review Thread Created"));

    // 3. post_pr_status
    let r3 = tool
        .execute(json!({
            "action": "post_pr_status",
            "pull_request_id": 501,
            "state": "succeeded",
            "name": "Tagisan/QualityGate",
            "description": "All 25 brutal tests passing"
        }))
        .await
        .expect("post_pr_status failed");
    assert!(r3.contains("Azure Repos Git Status Policy Posted"));

    // 4. exchange_federated_token
    let r4 = tool
        .execute(json!({
            "action": "exchange_federated_token",
            "oidc_token": "valid_token_sample",
            "client_id": "client-123",
            "tenant_id": "tenant-456"
        }))
        .await
        .expect("exchange_federated_token failed");
    assert!(r4.contains("Workload Identity Federation Token Exchanged"));

    // 5. transition_work_item
    let r5 = tool
        .execute(json!({
            "action": "transition_work_item",
            "work_item_id": 1948201,
            "state": "Closed",
            "commit_sha": "abc12345"
        }))
        .await
        .expect("transition_work_item failed");
    assert!(r5.contains("Azure DevOps Work Item State Transitioned"));
}

// =========================================================================
// 2. Microsoft IcM Tests
// =========================================================================

#[tokio::test]
async fn test_07_icm_v2_schema_ingestion_and_severity_triage() {
    let engine = IcmEngine::new();

    // Microsoft IcM schema v2 payload
    let raw_v2 = json!({
        "IncidentId": 984729112,
        "Severity": 1,
        "Title": "Core Authentication Service Major Outage",
        "Summary": "Token endpoint throwing 500 Internal Server Error globally.",
        "OwningService": "Entra-Core-Auth",
        "OwningTeam": "Identity-Foundations",
        "Status": "Investigating",
        "OccurredDate": "2026-09-16T08:00:00Z",
        "ImpactedRegions": ["East US 2", "West US 3", "West Europe", "Southeast Asia"],
        "CorrelatedCommit": "9f8e7d6c"
    }).to_string();

    let inc = engine.ingest_incident(&raw_v2).expect("ingest v2 failed");
    assert_eq!(inc.incident_id, 984729112);
    assert_eq!(inc.severity, 1);
    assert_eq!(inc.owning_service, "Entra-Core-Auth");
    assert_eq!(inc.impacted_regions.len(), 4);
    assert_eq!(inc.correlated_commit.as_deref(), Some("9f8e7d6c"));

    // Severity mapping test
    assert_eq!(IcmSeverity::from_str_lossy("sev1"), IcmSeverity::Sev1);
    assert_eq!(IcmSeverity::from_str_lossy("critical"), IcmSeverity::Sev1);
    assert_eq!(IcmSeverity::from_str_lossy("sev2"), IcmSeverity::Sev2);
    assert_eq!(IcmSeverity::from_str_lossy("sev3"), IcmSeverity::Sev3);
    assert_eq!(IcmSeverity::from_str_lossy("sev4"), IcmSeverity::Sev4);
}

#[tokio::test]
async fn test_08_icm_dynamic_git_commit_correlation() {
    let engine = IcmEngine::new();
    let incident = IcmIncident {
        incident_id: 4829103,
        severity: 2,
        title: "Auth Gateway Throttling Burst".to_string(),
        summary: "Spike in 429s after deployment".to_string(),
        owning_service: "Entra-Core-Auth".to_string(),
        owning_team: "Identity-Foundations".to_string(),
        status: "Active".to_string(),
        occurred_at: "2026-09-16T10:30:00Z".to_string(),
        impacted_regions: vec!["East US".to_string()],
        correlated_commit: None,
    };

    let candidate_commits = vec![
        GitCommitInfo {
            commit_hash: "11111111".to_string(),
            author: "frontend-dev@microsoft.com".to_string(),
            timestamp: "2026-09-15T00:00:00Z".to_string(),
            message: "docs: update css styles".to_string(),
            modified_files: vec!["ui/styles.css".to_string()],
        },
        GitCommitInfo {
            commit_hash: "88888888".to_string(),
            author: "identity-eng@microsoft.com".to_string(),
            timestamp: "2026-09-16T10:15:00Z".to_string(),
            message: "fix(auth): update token bucket capacity for entra-core-auth".to_string(),
            modified_files: vec!["src/copilot/auth.rs".to_string(), "src/copilot/throttling.rs".to_string()],
        },
    ];

    let result = engine.correlate_commits(&incident, &candidate_commits).expect("correlation should match");
    assert_eq!(result.commit_hash, "88888888");
    assert_eq!(result.confidence, CorrelationConfidence::High);
    assert!(result.matched_reasons.iter().any(|r| r.contains("owning service")));
}

#[tokio::test]
async fn test_09_icm_automated_live_site_rollback_pr_generator() {
    let incident = IcmIncident {
        incident_id: 384729104,
        severity: 1,
        title: "Global Outage in Token Exchange".to_string(),
        summary: "500 errors".to_string(),
        owning_service: "Entra-Core-Auth".to_string(),
        owning_team: "Identity-Foundations".to_string(),
        status: "Active".to_string(),
        occurred_at: Utc::now().to_rfc3339(),
        impacted_regions: vec!["East US 2".to_string()],
        correlated_commit: Some("c4e82a1d".to_string()),
    };

    let rollback_pr = IcmRollbackPrGenerator::generate_rollback_pr(
        &incident,
        "c4e82a1d",
        &["src/copilot/auth.rs".to_string(), "src/copilot/throttling.rs".to_string()],
        None,
    )
    .expect("rollback PR generation failed");

    assert_eq!(rollback_pr.branch_name, "hotfix/rollback-384729104-c4e82a1d");
    assert!(rollback_pr.commit_message.contains("Revert commit c4e82a1d"));
    assert!(rollback_pr.unified_patch_diff.contains("diff --git a/src/copilot/auth.rs"));
    assert!(rollback_pr.blast_radius_safety_check.contains("PASSED"));
    assert_eq!(rollback_pr.teams_card["type"], "AdaptiveCard");
}

#[tokio::test]
async fn test_10_icm_pir_generator_and_tool_execution() {
    let tool = CopilotIcmTool::default();

    // 1. ingest_incident
    let r1 = tool
        .execute(json!({
            "action": "ingest_incident",
            "incident_id": 9991234,
            "severity": 1,
            "title": "Sev-1 Live-Site Outage",
            "owning_service": "Entra-Core-Auth"
        }))
        .await
        .expect("ingest failed");
    assert!(r1.contains("Microsoft IcM Incident Ingested"));

    // 2. correlate_commit
    let r2 = tool
        .execute(json!({
            "action": "correlate_commit",
            "incident_id": 9991234,
            "owning_service": "Entra-Core-Auth"
        }))
        .await
        .expect("correlate_commit failed");
    assert!(r2.contains("Git Commit Correlated"));

    // 3. generate_pir
    let r3 = tool
        .execute(json!({
            "action": "generate_pir",
            "incident_id": 9991234,
            "owning_service": "Entra-Core-Auth"
        }))
        .await
        .expect("generate_pir failed");
    assert!(r3.contains("Post-Incident Review (PIR)"));
    assert!(r3.contains("Root Cause (5 Whys Analysis)"));

    // 4. generate_rollback_pr
    let r4 = tool
        .execute(json!({
            "action": "generate_rollback_pr",
            "incident_id": 9991234,
            "commit_sha": "be3e9b32",
            "files_to_revert": ["src/copilot/auth.rs"]
        }))
        .await
        .expect("generate_rollback_pr failed");
    assert!(r4.contains("Live-Site Rollback PR Generated"));
    assert!(r4.contains("hotfix/rollback-9991234-be3e9b32"));
}

// =========================================================================
// 3. Microsoft Sentinel & Kusto Tests
// =========================================================================

#[tokio::test]
async fn test_11_sentinel_kusto_tabular_response_parser_all_types() {
    let raw_kusto_json = json!({
        "Tables": [
            {
                "TableName": "PrimaryResult",
                "Columns": [
                    {"ColumnName": "TimeGenerated", "DataType": "DateTime"},
                    {"ColumnName": "DeviceName", "DataType": "String"},
                    {"ColumnName": "ProcessId", "DataType": "Int64"},
                    {"ColumnName": "CpuUsage", "DataType": "Double"},
                    {"ColumnName": "IsCompromised", "DataType": "Boolean"},
                    {"ColumnName": "EventData", "DataType": "Dynamic"}
                ],
                "Rows": [
                    [
                        "2026-09-16T10:00:00Z",
                        "SAW-CORP-01",
                        4096,
                        99.8,
                        true,
                        {"action": "escalation", "source": "npu"}
                    ]
                ]
            }
        ]
    }).to_string();

    let result = KustoQueryResult::parse(&raw_kusto_json).expect("parse kusto json failed");
    let table = result.primary_table().expect("missing primary table");
    assert_eq!(table.name, "PrimaryResult");
    assert_eq!(table.columns.len(), 6);
    assert_eq!(table.columns[0].data_type, "datetime");
    assert_eq!(table.columns[1].data_type, "string");
    assert_eq!(table.columns[2].data_type, "long");
    assert_eq!(table.columns[3].data_type, "real");
    assert_eq!(table.columns[4].data_type, "bool");
    assert_eq!(table.columns[5].data_type, "dynamic");

    let row = &table.rows[0];
    assert_eq!(row.get_string(1), Some("SAW-CORP-01"));
    assert_eq!(row.get_i64(2), Some(4096));
    assert_eq!(row.get_f64(3), Some(99.8));
    assert_eq!(row.get_bool(4), Some(true));
    assert!(row.get_json(5).is_some());
}

#[tokio::test]
async fn test_12_sentinel_kusto_execution_engine_endpoints() {
    let engine = KustoExecutionEngine::new();

    // Log Analytics Workspace endpoint format
    let ws_url = KustoExecutionEngine::format_endpoint_url("00000000-1111-2222-3333-444455556666");
    assert_eq!(ws_url, "https://api.loganalytics.io/v1/workspaces/00000000-1111-2222-3333-444455556666/query");

    // Kusto cluster endpoint format
    let kusto_url = KustoExecutionEngine::format_endpoint_url("mycluster.kusto.windows.net");
    assert_eq!(kusto_url, "https://mycluster.kusto.windows.net/v1/rest/query");

    // Execute AST Churn query
    let res = engine
        .execute_kql(
            "00000000-1111-2222-3333-444455556666",
            None,
            "Tagisan_AstChurn_CL | where AstChurnZScore > 3.0",
            Some("P1D"),
        )
        .await
        .expect("query failed");

    let table = res.primary_table().expect("missing table");
    assert!(table.columns.iter().any(|c| c.name == "AstChurnZScore"));
    assert!(!table.rows.is_empty());
}

#[tokio::test]
async fn test_13_sentinel_kql_hunting_query_catalog() {
    let q1 = KqlHuntingCatalog::ast_churn_anomaly();
    assert_eq!(q1.id, "Tagisan_AstChurn_Anomaly");
    assert_eq!(q1.severity, SentinelSeverity::High);
    assert!(q1.query.contains("Tagisan_AstChurn_CL"));

    let q2 = KqlHuntingCatalog::reachable_cve_exploitation();
    assert_eq!(q2.id, "Tagisan_Reachable_CVE_Exploit");
    assert_eq!(q2.severity, SentinelSeverity::Critical);
    assert!(q2.table_targets.contains(&"DeviceProcessEvents".to_string()));

    let q3 = KqlHuntingCatalog::prompt_injection_attempts();
    assert_eq!(q3.id, "Tagisan_AgentShield_Prompt_Injection");
    assert_eq!(q3.severity, SentinelSeverity::Critical);
    assert!(q3.table_targets.contains(&"AgentShield_Audit_CL".to_string()));

    assert_eq!(KqlHuntingCatalog::all_queries().len(), 3);
}

#[tokio::test]
async fn test_14_copilot_sentinel_tool_actions_execution() {
    let tool = CopilotSentinelTool::default();

    // 1. ingest_event
    let r1 = tool
        .execute(json!({
            "action": "ingest_event",
            "event_type": "ast_blast_radius",
            "severity": "High",
            "summary": "AST Churn Z-score spike on auth router"
        }))
        .await
        .expect("ingest failed");
    assert!(r1.contains("Microsoft Sentinel Security Event Emitted"));
    assert!(r1.contains("CEF:0"));

    // 2. generate_kql_rule
    let r2 = tool
        .execute(json!({
            "action": "generate_kql_rule",
            "summary": "High AST Churn Threshold Alert",
            "severity": "High"
        }))
        .await
        .expect("generate rule failed");
    assert!(r2.contains("Microsoft Sentinel Analytics Rule Generated"));

    // 3. execute_kql_query
    let r3 = tool
        .execute(json!({
            "action": "execute_kql_query",
            "query": "Tagisan_AstChurn_CL | take 2"
        }))
        .await
        .expect("execute kql failed");
    assert!(r3.contains("Kusto KQL Query Execution Result"));

    // 4. get_hunting_queries
    let r4 = tool
        .execute(json!({
            "action": "get_hunting_queries"
        }))
        .await
        .expect("get hunting queries failed");
    assert!(r4.contains("Microsoft Sentinel Threat Hunting Queries"));
    assert!(r4.contains("Tagisan_AstChurn_Anomaly"));
}

// =========================================================================
// 4. Microsoft Purview & Apache Atlas Tests
// =========================================================================

#[tokio::test]
async fn test_15_purview_atlas_models_and_entity_registration() {
    let engine = PurviewDataMapEngine::new();
    let entity = AtlasEntity::new(
        "delta_table",
        "guid_delta_customers",
        "onelake://workspace/lakehouse/Tables/Customers",
        "Customers_Delta",
    );

    let registered = engine.register_entity(entity).await.expect("register entity failed");
    assert_eq!(registered.entity.type_name, "delta_table");
    assert_eq!(registered.entity.guid, "guid_delta_customers");
    assert_eq!(registered.entity.status, "ACTIVE");
}

#[tokio::test]
async fn test_16_purview_lineage_registration_and_relation_edges() {
    let engine = PurviewDataMapEngine::new();

    let e1 = AtlasEntity::new("access_table", "guid_in", "msaccess://db.accdb/In", "InTable");
    let e2 = AtlasEntity::new("dataverse_entity", "guid_out", "dataverse://crm/Out", "OutEntity");
    engine.register_entity(e1).await.expect("register e1 failed");
    engine.register_entity(e2).await.expect("register e2 failed");

    let lineage = engine
        .register_lineage(
            "AccessToDataverseSync",
            &["guid_in".to_string()],
            &["guid_out".to_string()],
        )
        .await
        .expect("register lineage failed");

    assert_eq!(lineage.relations.len(), 1);
    assert_eq!(lineage.relations[0].from_entity_id, "guid_in");
    assert_eq!(lineage.relations[0].to_entity_id, "guid_out");
    assert_eq!(lineage.lineage_depth, 2);
}

#[tokio::test]
async fn test_17_purview_end_to_end_4_tier_lineage_synthesis() {
    let engine = PurviewDataMapEngine::new();
    let lineage = engine
        .generate_tagisan_end_to_end_lineage("TelemetryStream")
        .await
        .expect("e2e lineage failed");

    assert_eq!(lineage.lineage_depth, 4);
    assert!(lineage.relations.len() >= 3);
    assert!(lineage.guid_entity_map.values().any(|e| e.type_name == "access_table"));
    assert!(lineage.guid_entity_map.values().any(|e| e.type_name == "dataverse_entity"));
    assert!(lineage.guid_entity_map.values().any(|e| e.type_name == "delta_table"));
    assert!(lineage.guid_entity_map.values().any(|e| e.type_name == "powerbi_direct_lake_model"));
}

#[tokio::test]
async fn test_18_copilot_purview_tool_actions_execution() {
    let tool = CopilotPurviewTool::default();

    // 1. classify
    let r1 = tool
        .execute(json!({
            "action": "classify",
            "content": "Secret encryption key and classified telemetry",
            "declared_label": "Secret"
        }))
        .await
        .expect("classify failed");
    assert!(r1.contains("Zero-Egress Air-Gap Enforced") && r1.contains("🔒 YES"));

    // 2. register_entity
    let r2 = tool
        .execute(json!({
            "action": "register_entity",
            "type_name": "delta_table",
            "guid": "guid_telemetry_delta",
            "name": "Telemetry_Delta",
            "qualified_name": "onelake://workspace/lakehouse/Tables/Telemetry"
        }))
        .await
        .expect("register entity failed");
    assert!(r2.contains("Purview Data Map Entity Registered"));

    // 3. generate_e2e_lineage
    let r3 = tool
        .execute(json!({
            "action": "generate_e2e_lineage",
            "dataset_name": "EngineeringMetrics"
        }))
        .await
        .expect("generate_e2e_lineage failed");
    assert!(r3.contains("Tagisan End-to-End Microsoft Data Lineage Synthesized"));
    assert!(r3.contains("Access 365 ➡️ Dataverse ➡️ OneLake ➡️ Power BI"));
}

// =========================================================================
// 5. Microsoft Viva Suite Tests
// =========================================================================

#[tokio::test]
async fn test_19_viva_dynamic_1on1_prep_synthesis() {
    let engine = VivaEngine::new();

    let commits = vec![
        "7a9b1c2: Implemented pure Rust TMDL generator".to_string(),
        "4d8f2e1: Added Direct Lake partition bindings for OneLake".to_string(),
    ];
    let prs = vec!["PR #42: Microsoft Tech Stack Strengthening".to_string()];
    let blast_radius = vec!["Containment score improved by 34% across 8 modules".to_string()];
    let verdicts = vec!["Accepted: Pure Rust architecture without external COM/ODBC drivers".to_string()];

    let prep = engine
        .generate_1on1_prep_dynamic(
            "Principal Architect",
            "Partner Director",
            &commits,
            &prs,
            &blast_radius,
            &verdicts,
        )
        .expect("dynamic 1on1 failed");

    assert_eq!(prep.lead_name, "Partner Director");
    assert_eq!(prep.engineer_name, "Principal Architect");
    assert!(prep.shared_initiatives.iter().any(|i| i.contains("PR Track")));
    assert!(prep.recent_deliverables.iter().any(|d| d.contains("Direct Lake")));
    assert!(prep.suggested_discussion_topics.iter().any(|t| t.contains("architectural decisions")));
}

#[tokio::test]
async fn test_20_viva_goals_client_graph_payloads() {
    let goal_payload = VivaGoalsClient::create_goal_payload(
        "Zero-Defect Formal Invariant Verification",
        "principal-dev@microsoft.com",
        100.0,
        99.5,
        "%",
    );
    assert_eq!(goal_payload["@odata.type"], "#microsoft.graph.goal");
    assert_eq!(goal_payload["status"], "OnTrack");

    let okr_sync = VivaGoalsClient::generate_okr_sync_payload(100.0, 4.2);
    let key_results = okr_sync["keyResults"].as_array().expect("missing keyResults");
    assert_eq!(key_results.len(), 2);
    assert_eq!(key_results[0]["target"], 100.0);
    assert_eq!(key_results[1]["target"], 10.0);
}

#[tokio::test]
async fn test_21_copilot_viva_tool_actions_execution() {
    let tool = CopilotVivaTool::default();

    // 1. sync_okr
    let r1 = tool
        .execute(json!({
            "action": "sync_okr",
            "goal_id": "Invariant-Pass-Rate",
            "current_value": 100.0,
            "target_value": 100.0,
            "metric_unit": "%"
        }))
        .await
        .expect("sync_okr failed");
    assert!(r1.contains("Microsoft Viva Goals Key Result Synced"));

    // 2. generate_1on1_prep
    let r2 = tool
        .execute(json!({
            "action": "generate_1on1_prep",
            "engineer_name": "Chief Scientist",
            "lead_name": "VP of Engineering",
            "commits": ["fix: direct lake partition"],
            "pull_requests": ["PR #10: msft strengthening"]
        }))
        .await
        .expect("generate_1on1 failed");
    assert!(r2.contains("Microsoft Viva Insights 1:1 Briefing Prepared"));
    assert!(r2.contains("Chief Scientist"));

    // 3. generate_viva_goals_payload
    let r3 = tool
        .execute(json!({
            "action": "generate_viva_goals_payload",
            "invariant_pass_rate": 100.0,
            "blast_radius_index": 3.8
        }))
        .await
        .expect("generate_viva_goals_payload failed");
    assert!(r3.contains("Microsoft Graph Viva Goals OKR Payload Generated"));
}

// =========================================================================
// 6. Microsoft Teams Bot & Message Extensions Tests
// =========================================================================

#[tokio::test]
async fn test_22_teams_bot_compose_extension_query_router() {
    let handler = TeamsMessageExtensionHandler::new();

    let query_payload = json!({
        "commandId": "searchCodebase",
        "parameters": [
            {
                "name": "queryText",
                "value": "PurviewAirgap"
            }
        ]
    });

    let resp = handler.handle_query(&query_payload).expect("handle query failed");
    let ext = resp.compose_extension;
    assert_eq!(ext.attachment_layout, "list");
    assert_eq!(ext.result_type, "result");
    assert_eq!(ext.attachments.len(), 3);

    // Verify preview and content structure
    let att0 = &ext.attachments[0];
    assert_eq!(att0.content_type, "application/vnd.microsoft.card.adaptive");
    assert_eq!(att0.preview["contentType"], "application/vnd.microsoft.card.thumbnail");
    assert!(att0.content.to_string().contains("PurviewAirgap"));
}

#[tokio::test]
async fn test_23_teams_bot_universal_action_user_specific_views() {
    let ua = AdaptiveCardUniversalAction::new(
        "approve_hotfix",
        json!({"patch_id": "patch_101"}),
        Some("eng-lead@microsoft.com"),
        Some("Lead"),
    );

    assert_eq!(ua.action_type, "Action.Execute");
    assert_eq!(ua.verb, "approve_hotfix");
    assert!(ua.refresh_token.is_some());

    let card = ua.create_user_specific_view("Hotfix Deployment Approval", "Patch patch_101 passed formal verification.");
    assert_eq!(card["version"], "1.6");
    assert!(card["refresh"]["userIds"].as_array().unwrap().contains(&json!("eng-lead@microsoft.com")));
    assert!(card.to_string().contains("Engineering Lead View"));
}

// =========================================================================
// 7. Power BI Direct Lake & Lakehouse Maintenance Tests
// =========================================================================

#[tokio::test]
async fn test_24_powerbi_direct_lake_tmdl_generation() {
    let cols = vec![
        TmdlColumn {
            name: "EventId".to_string(),
            data_type: "string".to_string(),
            format_string: None,
            summarize_by: Some("none".to_string()),
            source_column: "EventId".to_string(),
            description: Some("Unique event identifier".to_string()),
        },
        TmdlColumn {
            name: "DurationMs".to_string(),
            data_type: "int64".to_string(),
            format_string: Some("#,##0".to_string()),
            summarize_by: Some("average".to_string()),
            source_column: "DurationMs".to_string(),
            description: Some("Execution duration in ms".to_string()),
        },
    ];

    let measures = vec![
        TmdlMeasure {
            name: "Avg Duration".to_string(),
            expression: "AVERAGE(TelemetryEvents[DurationMs])".to_string(),
            format_string: Some("#,##0.0".to_string()),
            display_folder: Some("Performance".to_string()),
            description: Some("Average execution duration".to_string()),
        }
    ];

    let db = PowerBiEngine::generate_direct_lake_model(
        "ProdWorkspace",
        "ProdLakehouse",
        "TelemetryEvents",
        &cols,
        &measures,
    )
    .expect("generate direct lake model failed");

    assert_eq!(db.compatibility_level, 1604);
    assert_eq!(db.tables.len(), 1);
    let table = &db.tables[0];
    assert_eq!(table.name, "TelemetryEvents");
    assert_eq!(table.partitions[0].mode, "directLake");
    assert_eq!(table.partitions[0].source_expression, "onelake://ProdWorkspace/ProdLakehouse/Tables/TelemetryEvents");

    // TMDL serialization test
    let engine = PowerBiEngine::new();
    let tmdl = engine.generate_tmdl(&db);
    assert!(tmdl.contains("mode: directLake"));
    assert!(tmdl.contains("onelake://ProdWorkspace/ProdLakehouse/Tables/TelemetryEvents"));
}

#[tokio::test]
async fn test_25_powerbi_fabric_lakehouse_maintenance_sql() {
    let maint = PowerBiEngine::generate_lakehouse_maintenance_sql(
        "TelemetryEvents",
        168,
        &["CommitSha", "Timestamp"],
    );

    assert_eq!(maint.table_name, "TelemetryEvents");
    assert_eq!(maint.vacuum_sql, "VACUUM TelemetryEvents RETAIN 168 HOURS;");
    assert_eq!(maint.optimize_sql, "OPTIMIZE TelemetryEvents ZORDER BY (CommitSha, Timestamp);");
    assert!(maint.full_maintenance_script.contains("Step 1: Remove uncommitted"));
    assert!(maint.full_maintenance_script.contains("Step 2: Compact small parquet files"));
}

#[tokio::test]
async fn test_26_copilot_powerbi_tool_direct_lake_and_maintenance() {
    let tool = CopilotPowerBiTool::default();

    // 1. generate_direct_lake_model
    let r1 = tool
        .execute(json!({
            "action": "generate_direct_lake_model",
            "workspace_name": "FinOpsWorkspace",
            "lakehouse_name": "FinLakehouse",
            "table_name": "LedgerDelta"
        }))
        .await
        .expect("generate_direct_lake_model failed");
    assert!(r1.contains("Power BI Direct Lake TMDL Model Generated"));
    assert!(r1.contains("Partition Mode") && r1.contains("directLake"));
    assert!(r1.contains("onelake://FinOpsWorkspace/FinLakehouse/Tables/LedgerDelta"));

    // 2. lakehouse_maintenance
    let r2 = tool
        .execute(json!({
            "action": "lakehouse_maintenance",
            "table_name": "LedgerDelta",
            "vacuum_retain_hours": 72,
            "zorder_columns": ["AccountId", "TransactionDate"]
        }))
        .await
        .expect("lakehouse_maintenance failed");
    assert!(r2.contains("Microsoft Fabric Lakehouse Table Maintenance SQL"));
    assert!(r2.contains("VACUUM LedgerDelta RETAIN 72 HOURS;"));
    assert!(r2.contains("OPTIMIZE LedgerDelta ZORDER BY (AccountId, TransactionDate);"));
}

// =========================================================================
// 8. 50-Worker Concurrency Stress Test & Resilience Fuzzing
// =========================================================================

#[tokio::test]
async fn test_27_50_worker_concurrent_stress_across_strengthened_ms_platform() {
    let ado_engine = Arc::new(AdoEngine::default());
    let icm_engine = Arc::new(IcmEngine::new());
    let sentinel_engine = Arc::new(SentinelBridgeEngine::new());
    let purview_engine = Arc::new(PurviewDataMapEngine::new());
    let viva_engine = Arc::new(VivaEngine::new());
    let bot_handler = Arc::new(TeamsMessageExtensionHandler::new());
    let pbi_engine = Arc::new(PowerBiEngine::new());

    let mut handles = Vec::new();

    for worker_id in 0..50 {
        let ado = ado_engine.clone();
        let icm = icm_engine.clone();
        let sentinel = sentinel_engine.clone();
        let purview = purview_engine.clone();
        let viva = viva_engine.clone();
        let bot = bot_handler.clone();
        let pbi = pbi_engine.clone();

        handles.push(tokio::spawn(async move {
            // 1. ADO
            let thread = ado
                .create_pull_request_thread(
                    "Tagisan",
                    worker_id,
                    &format!("src/worker_{}.rs", worker_id),
                    worker_id as u32 + 1,
                    1,
                    "Concurrent stress comment",
                    "worker@tagisan.ai",
                )
                .await
                .expect("ado worker thread failed");
            assert_eq!(thread.status, AdoThreadStatus::Active);

            let status = ado
                .post_pull_request_status(
                    "Tagisan",
                    worker_id,
                    AdoGitStatusState::Succeeded,
                    "Tagisan/StressGate",
                    "ci",
                    "Passed",
                    None,
                )
                .await
                .expect("ado status failed");
            assert_eq!(status.state, AdoGitStatusState::Succeeded);

            // 2. IcM
            let incident = IcmIncident {
                incident_id: 1000 + worker_id,
                severity: (worker_id % 4) as u32 + 1,
                title: format!("Worker {} degradation", worker_id),
                summary: "Concurrent stress incident".to_string(),
                owning_service: "Entra-Core-Auth".to_string(),
                owning_team: "Identity".to_string(),
                status: "Active".to_string(),
                occurred_at: Utc::now().to_rfc3339(),
                impacted_regions: vec!["West US".to_string()],
                correlated_commit: None,
            };
            let pir = icm.generate_pir(&incident, None).expect("icm pir failed");
            assert_eq!(pir.root_cause_5_whys.len(), 5);

            // 3. Sentinel
            let query_res = sentinel
                .kusto_engine()
                .execute_kql(
                    "workspace-concurrent",
                    None,
                    "Tagisan_AstChurn_CL | take 1",
                    None,
                )
                .await
                .expect("sentinel query failed");
            assert!(!query_res.tables.is_empty());

            // 4. Purview
            let ent = AtlasEntity::new(
                "delta_table",
                &format!("guid_worker_{}", worker_id),
                &format!("onelake://ws/lh/Tables/t_{}", worker_id),
                &format!("Table_{}", worker_id),
            );
            purview.register_entity(ent).await.expect("purview entity failed");

            // 5. Viva
            let briefing = viva
                .generate_1on1_prep(&format!("Engineer_{}", worker_id), "Lead_Mgr")
                .expect("viva briefing failed");
            assert_eq!(briefing.engineer_name, format!("Engineer_{}", worker_id));

            // 6. Bot
            let bot_resp = bot
                .handle_query(&json!({
                    "parameters": [{"name": "queryText", "value": format!("symbol_{}", worker_id)}]
                }))
                .expect("bot query failed");
            assert_eq!(bot_resp.compose_extension.attachments.len(), 3);

            // 7. Power BI
            let maint = PowerBiEngine::generate_lakehouse_maintenance_sql(
                &format!("Table_{}", worker_id),
                24,
                &["Id"],
            );
            assert!(maint.vacuum_sql.contains("VACUUM"));
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        handle.await.unwrap_or_else(|e| panic!("Worker {} panicked: {}", i, e));
    }
}

#[tokio::test]
async fn test_28_malformed_and_injection_payload_resilience() {
    let icm_engine = IcmEngine::new();
    let purview_tool = CopilotPurviewTool::default();
    let sentinel_tool = CopilotSentinelTool::default();
    let ado_engine = AdoEngine::default();

    // 1. Corrupted IcM schema
    let corrupted_json = "{ \"IncidentId\": not_a_number, corrupted ...";
    let fallback = icm_engine.ingest_incident(corrupted_json).expect("fallback parser should gracefully handle corrupted JSON");
    assert!(fallback.incident_id > 0);

    // 2. Empty Purview receipt verification
    let bad_verify = purview_tool.execute(json!({
        "action": "verify_receipt",
        "content": "some text",
        "receipt": {
            "receipt_id": "invalid",
            "timestamp": "now",
            "sensitivity": "general",
            "content_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
            "signature_sha256": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "egress_blocked": false,
            "routing_engine": "local",
            "policy_applied": "default"
        }
    })).await.expect("verify should return result string without panic");
    assert!(bad_verify.contains("FORGED / MISMATCHED"));

    // 3. KQL injection in Sentinel rule generation
    let kql_injection = "Tagisan_AstChurn_CL'; DROP TABLE SecurityAlert; --";
    let rule_res = sentinel_tool.execute(json!({
        "action": "generate_kql_rule",
        "query": kql_injection
    })).await.expect("kql rule should format without panicking");
    assert!(rule_res.contains(kql_injection));

    // 4. ADO federated token with whitespace
    let bad_ado_token = AdoFederatedTokenRequest {
        oidc_token: "   \t\n".to_string(),
        client_id: "c1".to_string(),
        tenant_id: "t1".to_string(),
        scope: "s1".to_string(),
    };
    assert!(ado_engine.exchange_federated_token(&bad_ado_token).await.is_err());
}

#[tokio::test]
async fn test_29_purview_cryptographic_audit_receipt_verification() {
    let content = "Highly confidential defense telemetry and cryptographic private key.";
    let eval = PurviewGuardEngine::evaluate(content, Some("Secret"), None).expect("evaluation failed");
    assert!(eval.air_gapped);

    // Verify valid receipt
    assert!(eval.receipt.verify_integrity(content));

    // Tampered content verification failure
    assert!(!eval.receipt.verify_integrity("Tampered content"));
}

#[tokio::test]
async fn test_30_ado_pr_thread_offset_and_multi_reply_chain() {
    let engine = AdoEngine::default();
    let thread = engine
        .create_pull_request_thread(
            "Tagisan",
            777,
            "src/copilot/powerbi.rs",
            214,
            12,
            "Verify directLake partition mode expression syntax.",
            "reviewer@microsoft.com",
        )
        .await
        .expect("create thread failed");

    let r1 = engine
        .add_comment_to_thread("Tagisan", 777, thread.id, "Acknowledged, checking OneLake prefix.", "dev1@microsoft.com")
        .await
        .expect("r1 failed");
    assert_eq!(r1.id, 2);

    let r2 = engine
        .add_comment_to_thread("Tagisan", 777, thread.id, "Confirmed onelake:// URI format conforms to Fabric spec.", "dev2@microsoft.com")
        .await
        .expect("r2 failed");
    assert_eq!(r2.id, 3);
    assert_eq!(r2.parent_comment_id, 2);

    let list = engine.list_pull_request_threads("Tagisan", 777).await.expect("list failed");
    assert_eq!(list[0].comments.len(), 3);
}
