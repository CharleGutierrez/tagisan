//! # Brutal Verification Test Suite: Microsoft SOAR & Frontier Enterprise Integration
//!
//! Exhaustive test suite verifying the 7 frontier Microsoft integration pillars:
//! 1. Microsoft Sentinel SOAR Playbooks: Logic Apps Standard workflow generator with multi-action remediation graph.
//! 2. Azure Event Grid & Event Hubs: CloudEvents v1.0 standard schema & HMAC-SHA256 SAS tokens.
//! 3. Microsoft Entra ID Privileged Identity Management (PIM): JIT role elevation & dual-agent cryptographic attestation.
//! 4. Microsoft Purview Information Protection (MIP): Binary .pfile compound envelope parser & Key Vault HSM integration.
//! 5. Fluent UI v9 & Offline PCF: Production React 18 PCF components & ControlManifest.Input.xml generator.
//! 6. Azure Resource Graph (ARG) & Hybrid Azure Arc: Multi-cloud connected machine fleet posture audits.
//! 7. T-SQL & Fabric SQL Endpoint Invariants: AST invariant analyzer for Columnstore, Temporal, RLS, and DDM.
//! 8. Unified Tool Handler: Dispatch through ToolHandler trait for all subsystem actions with edge-case fuzzing.
//! 9. 100-worker multi-threaded concurrency stress test and adversarial tampering resilience.

use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;
use tagisan::copilot::ms_soar_frontier::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. Microsoft Sentinel SOAR Playbooks Tests
// =========================================================================

#[test]
fn test_01_sentinel_soar_workflow_generation_and_topological_run_after() {
    let engine = LogicAppsWorkflowEngine::new();
    let actions = vec![
        RemediationAction::RevokeEntraUserSessions,
        RemediationAction::IsolateDefenderEndpoint,
        RemediationAction::PostTeamsWarRoomAdaptiveCard,
    ];

    let workflow = engine.generate_sentinel_remediation_workflow(
        "Tagisan-SecOps-AutoRemediate",
        "High",
        &actions,
    );

    // Validate Logic Apps Standard workflow definition envelope
    assert_eq!(
        workflow["$schema"],
        "https://schema.management.azure.com/providers/Microsoft.Logic/schemas/2016-06-01/workflowdefinition.json#"
    );
    assert_eq!(workflow["metadata"]["tagisanWorkflow"], "Tagisan-SecOps-AutoRemediate");
    assert_eq!(workflow["metadata"]["targetSeverity"], "High");

    let action_map = workflow["actions"].as_object().expect("Actions must be an object");
    assert!(action_map.contains_key("Revoke_Entra_Sign_In_Sessions"));
    assert!(action_map.contains_key("Isolate_Defender_Machine"));
    assert!(action_map.contains_key("Post_Teams_Incident_Card"));

    // Check sequential runAfter dependencies
    let revoke_act = &action_map["Revoke_Entra_Sign_In_Sessions"];
    assert!(revoke_act["runAfter"].as_object().unwrap().is_empty());

    let isolate_act = &action_map["Isolate_Defender_Machine"];
    assert!(isolate_act["runAfter"].as_object().unwrap().contains_key("Revoke_Entra_Sign_In_Sessions"));

    let teams_act = &action_map["Post_Teams_Incident_Card"];
    assert!(teams_act["runAfter"].as_object().unwrap().contains_key("Isolate_Defender_Machine"));
}

#[test]
fn test_02_sentinel_connections_metadata_generation() {
    let engine = LogicAppsWorkflowEngine::new();
    let connections = engine.generate_connections_metadata();

    assert!(connections.get("managedApiConnections").is_some());
    let managed = &connections["managedApiConnections"];
    assert!(managed.get("azuresentinel").is_some());
    assert!(managed.get("teams").is_some());
    assert_eq!(
        managed["azuresentinel"]["authentication"]["type"],
        "ManagedServiceIdentity"
    );
    assert_eq!(
        managed["teams"]["authentication"]["type"],
        "ManagedServiceIdentity"
    );
}

// =========================================================================
// 2. Azure Event Grid & Event Hubs Stream Router Tests
// =========================================================================

#[test]
fn test_03_eventgrid_cloudevent_spec_compliance() {
    let engine = AzureEventGridEngine::new();
    let payload = json!({
        "audit_id": "AUDIT-2026-X1",
        "actor": "TagisanAutonomousAgent_09",
        "action": "AST_Blast_Radius_Assessment",
        "risk_level": "LOW"
    });

    let event = engine.build_cloud_event(
        "tgs-evt-001",
        "/tagisan/agents/lakandiwa",
        "Microsoft.Tagisan.AgentActionCompleted",
        payload.clone(),
    );

    assert_eq!(event.specversion, "1.0");
    assert_eq!(event.event_type, "Microsoft.Tagisan.AgentActionCompleted");
    assert_eq!(event.source, "/tagisan/agents/lakandiwa");
    assert_eq!(event.datacontenttype, "application/json");
    assert_eq!(event.id, "tgs-evt-001");
    assert_eq!(event.data, payload);
    assert!(!event.time.is_empty());
}

#[test]
fn test_04_eventgrid_subscription_validation_handshake() {
    let engine = AzureEventGridEngine::new();

    // Standard Azure Event Grid validation request
    let handshake_request = r#"[
        {
            "id": "2d1781af-3a4c-4d7c-bd0c-e34b19da4e66",
            "topic": "/subscriptions/sub-123/resourceGroups/rg-sec/providers/Microsoft.EventGrid/topics/tgs-topic",
            "subject": "",
            "data": {
                "validationCode": "5148A7B8-1540-43DA-A75F-03F92B3AF100",
                "validationUrl": "https://rp-eastus.eventgrid.azure.net:553/eventsubscriptions/sub1/validate?data=test"
            },
            "eventType": "Microsoft.EventGrid.SubscriptionValidationEvent",
            "eventTime": "2026-09-17T08:30:00.0000000Z",
            "dataVersion": "1"
        }
    ]"#;

    let response_str = engine.handle_subscription_validation(handshake_request).expect("Handshake should succeed");
    let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");
    assert_eq!(response["validationResponse"], "5148A7B8-1540-43DA-A75F-03F92B3AF100");

    // Single object format variation
    let single_obj = r#"{
        "eventType": "Microsoft.EventGrid.SubscriptionValidationEvent",
        "data": {
            "validationCode": "ALPHA-BRAVO-99"
        }
    }"#;
    let response2_str = engine.handle_subscription_validation(single_obj).expect("Handshake single object should succeed");
    let response2: Value = serde_json::from_str(&response2_str).expect("Valid JSON");
    assert_eq!(response2["validationResponse"], "ALPHA-BRAVO-99");

    // Invalid format (missing validationCode)
    let invalid_event = r#"{"eventType": "Microsoft.Tagisan.OtherEvent", "data": { "foo": "bar" }}"#;
    assert!(engine.handle_subscription_validation(invalid_event).is_err());
}

#[test]
fn test_05_eventhubs_hmac_sha256_sas_token() {
    let engine = AzureEventGridEngine::new();
    let resource_uri = "https://tagisan-eh.servicebus.windows.net/ingest-hub";
    let key_name = "RootManageSharedAccessKey";
    let key_value = "c2VjcmV0LWtleS1mb3ItaG1hYy1zaGEyNTYtc2lnbmluZw==";
    let ttl_secs = 3600;

    let token = engine.generate_event_hubs_sas_token(resource_uri, key_name, key_value, ttl_secs);

    assert!(token.starts_with("SharedAccessSignature sr="));
    assert!(token.contains("&sig="));
    assert!(token.contains("&se="));
    assert!(token.contains("&skn=RootManageSharedAccessKey"));

    // Verify token expiration is approximately 1 hour in the future
    let se_part = token.split("&se=").nth(1).unwrap().split("&skn=").next().unwrap();
    let expiry: i64 = se_part.parse().expect("Expiry must be valid integer");
    let now = Utc::now().timestamp();
    assert!(expiry > now);
    assert!(expiry <= now + ttl_secs as i64 + 10);
}

// =========================================================================
// 3. Microsoft Entra ID Privileged Identity Management (PIM) Tests
// =========================================================================

#[test]
fn test_06_entra_pim_jit_role_elevation_payload() {
    let pim = EntraPimEngine::new("72f988bf-86f1-41af-91ab-2d7cd011db47");

    let req = PimRoleRequest {
        principal_id: "usr-agent-secops-01".to_string(),
        role_definition_id: "b24988ac-6180-42a0-ab88-20f7382dd24c".to_string(),
        directory_scope: "/subscriptions/sub-corp-prod-001".to_string(),
        justification: "Emergency AST remediation of cross-tenant privilege escalation".to_string(),
        duration_minutes: 240,
        ticket_number: "TGS-PIM-778899".to_string(),
    };

    let payload = pim.build_pim_elevation_request(&req);

    assert_eq!(payload["action"], "selfActivate");
    assert_eq!(payload["principalId"], "usr-agent-secops-01");
    assert_eq!(payload["roleDefinitionId"], "b24988ac-6180-42a0-ab88-20f7382dd24c");
    assert_eq!(payload["directoryScopeId"], "/subscriptions/sub-corp-prod-001");
    assert!(payload["justification"].as_str().unwrap().contains("Emergency AST remediation"));
    assert_eq!(payload["scheduleInfo"]["expiration"]["type"], "afterDuration");
    assert_eq!(payload["scheduleInfo"]["expiration"]["duration"], "PT240M");
    assert_eq!(payload["ticketInfo"]["ticketNumber"], "TGS-PIM-778899");
}

#[test]
fn test_07_entra_pim_dual_agent_attestation_ticket() {
    let pim = EntraPimEngine::new("72f988bf-86f1-41af-91ab-2d7cd011db47");

    // Success case: unanimous consent, high confidence (>90%), low blast radius (<= 3.0)
    let ticket = pim.validate_dual_agent_attestation("APPROVED", "APPROVED", 98.4, 1.2)
        .expect("Dual-agent attestation must pass");
    assert!(ticket.starts_with("TGS-PIM-"));

    // Failure case 1: Non-unanimous verdict
    let res_split = pim.validate_dual_agent_attestation("APPROVED", "REJECTED", 98.4, 1.2);
    assert!(res_split.is_err());
    assert!(res_split.unwrap_err().to_string().contains("not unanimous"));

    // Failure case 2: Insufficient confidence (< 90%)
    let res_low_conf = pim.validate_dual_agent_attestation("APPROVED", "APPROVED", 88.5, 1.2);
    assert!(res_low_conf.is_err());
    assert!(res_low_conf.unwrap_err().to_string().contains("Insufficient confidence"));

    // Failure case 3: High blast radius (> 3.0)
    let res_high_blast = pim.validate_dual_agent_attestation("APPROVED", "APPROVED", 99.0, 4.5);
    assert!(res_high_blast.is_err());
    assert!(res_high_blast.unwrap_err().to_string().contains("Blast Radius exceeds"));
}

// =========================================================================
// 4. Microsoft Purview Information Protection (MIP) Binary Parser Tests
// =========================================================================

#[test]
fn test_08_mip_rms_compound_pfile_envelope_roundtrip() {
    let parser = MipRmsCompoundParser::new();
    let key_id = "key-vault-rms-771";
    let pl_xml = "<PublishingLicense><Owner>corp\\admin</Owner><ContentId>DOC-992</ContentId></PublishingLicense>";
    let ciphertext = b"CONFIDENTIAL FINANCIAL FORECAST 2026: DO NOT EXPORT UNDER ANY CONDITIONS";

    let raw = parser.serialize_pfile_envelope(key_id, pl_xml, ciphertext);

    // Must start with standard PFILE magic header
    assert!(raw.starts_with(b"MSFT_RMS_PFILE\x00"));

    // Parse back
    let envelope = parser.parse_pfile_envelope(&raw).expect("Parsing failed");

    assert_eq!(envelope.magic, "MSFT_RMS_PFILE");
    assert_eq!(envelope.version, 2);
    assert_eq!(envelope.key_id, key_id);
    assert_eq!(envelope.publishing_license_xml, pl_xml);
    assert_eq!(envelope.ciphertext_length, ciphertext.len() as u64);
}

#[test]
fn test_09_mip_rms_corrupted_payload_rejection() {
    let parser = MipRmsCompoundParser::new();

    // 1. Invalid magic header
    let bad_magic = b"INVALID_HEADER\x00\x00\x00";
    assert!(parser.parse_pfile_envelope(bad_magic).is_err());

    // 2. Truncated envelope
    let truncated = b"MSFT_RMS_PFILE\x00\x02";
    assert!(parser.parse_pfile_envelope(truncated).is_err());

    // 3. Corrupted payload with invalid offsets
    let mut corrupt = Vec::from(&b"MSFT_RMS_PFILE\x00"[..]);
    corrupt.extend_from_slice(&2u16.to_le_bytes());
    corrupt.extend_from_slice(&5000u16.to_le_bytes()); // Claims 5000 byte key ID but buffer ends
    assert!(parser.parse_pfile_envelope(&corrupt).is_err());
}

// =========================================================================
// 5. Fluent UI v9 & Offline PowerApps Component Framework (PCF) Tests
// =========================================================================

#[test]
fn test_10_fluent_ui_v9_react_component_synthesis() {
    let engine = FluentPcfEngine::new();
    let code = engine.generate_fluent_v9_component("ConsensusTelemetryViewer", "Tagisan.Controls");

    assert!(code.contains("import * as React from \"react\";"));
    assert!(code.contains("@fluentui/react-components"));
    assert!(code.contains("FluentProvider"));
    assert!(code.contains("webLightTheme"));
    assert!(code.contains("export interface IConsensusTelemetryViewerProps"));
    assert!(code.contains("verdict: string;"));
    assert!(code.contains("confidencePct: number;"));
    assert!(code.contains("blastRadius: number;"));
    assert!(code.contains("export const ConsensusTelemetryViewer: React.FC<IConsensusTelemetryViewerProps>"));
}

#[test]
fn test_11_pcf_control_manifest_and_offline_cache() {
    let engine = FluentPcfEngine::new();

    let manifest = engine.generate_pcf_manifest_xml("ConsensusTelemetryViewer", "Tagisan.Controls");

    assert!(manifest.contains("<manifest>"));
    assert!(manifest.contains(r#"control namespace="Tagisan.Controls" constructor="ConsensusTelemetryViewer" version="2.0.0""#));
    assert!(manifest.contains(r#"<property name="verdict" display-name-key="Consensus Verdict" of-type="SingleLine.Text""#));
    assert!(manifest.contains(r#"<property name="confidencePct" display-name-key="Confidence Percentage" of-type="Decimal""#));
    assert!(manifest.contains(r#"<platform-library name="React" version="18.2.0" />"#));
    assert!(manifest.contains(r#"<platform-library name="Fluent" version="9.0.0" />"#));
    assert!(manifest.contains(r#"<uses-feature name="Offline" />"#));

    // Offline Dataverse cache helper
    let helper = engine.generate_offline_dataverse_helper();
    assert!(helper.contains("context.client.isOffline()"));
    assert!(helper.contains("(context.webAPI as any).offline.retrieveRecord"));
}

// =========================================================================
// 6. Azure Resource Graph (ARG) & Hybrid Azure Arc Tests
// =========================================================================

#[test]
fn test_12_arg_kql_query_and_fleet_posture_audit() {
    let engine = AzureResourceGraphEngine::new();
    let kql = AzureResourceGraphEngine::arc_machines_query();

    assert!(kql.contains("Resources"));
    assert!(kql.contains("where type =~ 'microsoft.hybridcompute/machines'"));
    assert!(kql.contains("project id, name, resourceGroup, status=properties.status"));

    // Payload generation
    let payload = engine.build_arg_query_payload(kql, &[
        "sub-corp-001".to_string(),
        "sub-corp-002".to_string(),
    ]);
    assert_eq!(payload["subscriptions"].as_array().unwrap().len(), 2);
    assert_eq!(payload["options"]["resultFormat"], "table");

    // Fleet audit parsing simulation
    let sample_arg_json = r#"{
        "data": {
            "rows": [
                ["/subscriptions/s1/rg1/m1", "onprem-sql-01", "rg1", "Connected", "Windows Server 2022", "1.34.02484.1481"],
                ["/subscriptions/s1/rg1/m2", "onprem-app-02", "rg1", "Disconnected", "Ubuntu 22.04", "1.32.02341.1210"]
            ]
        }
    }"#;

    let records = engine.parse_arc_query_results(sample_arg_json).expect("Parsing ARG results should succeed");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].name, "onprem-sql-01");
    assert_eq!(records[0].status, "Connected");
    assert!(records[0].is_connected);

    assert_eq!(records[1].name, "onprem-app-02");
    assert_eq!(records[1].status, "Disconnected");
    assert!(!records[1].is_connected);

    // Self-healing CLI command verification
    let heal_cmd = engine.generate_arc_remediation_script("onprem-app-02", "rg1");
    assert_eq!(heal_cmd, "az connectedmachine upgrade --name \"onprem-app-02\" --resource-group \"rg1\" --yes");
}

// =========================================================================
// 7. T-SQL & Fabric SQL Endpoint Invariant Engine Tests
// =========================================================================

#[test]
fn test_13_tsql_ast_invariant_checks_and_rls_ddl() {
    let engine = TsqlInvariantEngine::new();

    // Insecure DDL: no Columnstore, no Temporal, no RLS, unmasked PII
    let insecure_ddl = r#"
        CREATE TABLE Sales.CustomerData (
            CustomerID INT PRIMARY KEY,
            CustomerSSN VARCHAR(11),
            CreditCardNumber VARCHAR(16),
            TenantID INT,
            AccountBalance DECIMAL(18, 2)
        );
    "#;

    let analysis = engine.analyze_sql_script(insecure_ddl);
    assert!(!analysis.has_columnstore_index);
    assert!(!analysis.has_temporal_versioning);
    assert!(!analysis.has_rls_policy);
    assert!(!analysis.has_data_masking);
    assert!(analysis.risk_score >= 5.0);
    assert!(analysis.findings.iter().any(|f| f.contains("Dynamic Data Masking")));

    // Compliant DDL: Columnstore, Temporal, RLS, Masked
    let compliant_ddl = r#"
        CREATE TABLE Sales.CustomerData (
            CustomerID INT PRIMARY KEY,
            CustomerSSN VARCHAR(11) MASKED WITH (FUNCTION = 'partial(0, "XXX-XX-", 4)'),
            TenantID INT,
            SysStartTime DATETIME2 GENERATED ALWAYS AS ROW START,
            SysEndTime DATETIME2 GENERATED ALWAYS AS ROW END,
            PERIOD FOR SYSTEM_TIME (SysStartTime, SysEndTime)
        )
        WITH (SYSTEM_VERSIONING = ON (HISTORY_TABLE = Sales.CustomerDataHistory));

        CREATE CLUSTERED COLUMNSTORE INDEX CCI_CustomerData ON Sales.CustomerData;
        CREATE SECURITY POLICY Sales.CustomerSecurityPolicy ADD FILTER PREDICATE Security.fn_TenantAccess(TenantID) ON Sales.CustomerData;
    "#;

    let clean_analysis = engine.analyze_sql_script(compliant_ddl);
    assert!(clean_analysis.has_columnstore_index);
    assert!(clean_analysis.has_temporal_versioning);
    assert!(clean_analysis.has_rls_policy);
    assert!(clean_analysis.has_data_masking);
    assert_eq!(clean_analysis.risk_score, 0.0);
    assert!(clean_analysis.findings.is_empty());

    // Generate RLS Security Policy DDL
    let rls_ddl = engine.generate_rls_predicate_and_policy("CustomerData", "Sales", "fn_TenantSecurityPredicate");
    assert!(rls_ddl.contains("CREATE FUNCTION Sales.fn_TenantSecurityPredicate(@TenantId AS int)"));
    assert!(rls_ddl.contains("CREATE SECURITY POLICY Sales.CustomerDataSecurityPolicy"));
    assert!(rls_ddl.contains("ADD FILTER PREDICATE Sales.fn_TenantSecurityPredicate(TenantId) ON Sales.CustomerData"));
}

// =========================================================================
// 8. Unified ToolHandler Trait Dispatch Tests
// =========================================================================

#[tokio::test]
async fn test_14_tool_handler_dispatch_and_edge_case_fuzzing() {
    let tool = CopilotMsSoarTool::new();
    assert_eq!(tool.name(), "ms_soar_copilot");
    assert!(tool.description().contains("Microsoft Frontier SOAR"));

    // Action: generate_sentinel_workflow
    let call_sentinel_raw = tool.execute(json!({
        "action": "generate_sentinel_workflow",
        "workflow_name": "SOAR-Incident-Gate",
        "severity": "High"
    })).await.expect("Dispatch sentinel failed");
    let call_sentinel: Value = serde_json::from_str(&call_sentinel_raw).unwrap();
    assert!(call_sentinel.get("workflow").is_some());
    assert_eq!(call_sentinel["success"], true);

    // Action: emit_cloud_event
    let call_cloudevent_raw = tool.execute(json!({
        "action": "emit_cloud_event",
        "event_id": "evt-77",
        "source": "tgs/audit",
        "event_type": "Microsoft.Tagisan.AuditComplete",
        "data": { "passed": true }
    })).await.expect("Dispatch cloudevent failed");
    let call_cloudevent: Value = serde_json::from_str(&call_cloudevent_raw).unwrap();
    assert_eq!(call_cloudevent["cloud_event"]["specversion"], "1.0");

    // Action: validate_eventgrid_handshake
    let call_handshake_raw = tool.execute(json!({
        "action": "validate_eventgrid_handshake",
        "request_body": r#"[{"data":{"validationCode":"code-xyz"}}]"#
    })).await.expect("Dispatch handshake failed");
    let call_handshake: Value = serde_json::from_str(&call_handshake_raw).unwrap();
    assert_eq!(call_handshake["handshake_response"]["validationResponse"], "code-xyz");

    // Action: generate_eventhubs_sas
    let call_sas_raw = tool.execute(json!({
        "action": "generate_eventhubs_sas",
        "resource_uri": "https://tagisan-eh.servicebus.windows.net/test",
        "key_name": "SendOnlyKey",
        "key_secret": "mock-secret-base64",
        "ttl_seconds": 1800
    })).await.expect("Dispatch sas failed");
    let call_sas: Value = serde_json::from_str(&call_sas_raw).unwrap();
    assert!(call_sas["sas_token"].as_str().unwrap().contains("SharedAccessSignature"));

    // Action: build_pim_elevation
    let call_pim_raw = tool.execute(json!({
        "action": "build_pim_elevation",
        "principal_id": "usr-secops-99",
        "role_id": "b24988ac-6180-42a0-ab88-20f7382dd24c"
    })).await.expect("Dispatch pim failed");
    let call_pim: Value = serde_json::from_str(&call_pim_raw).unwrap();
    assert_eq!(call_pim["pim_request_payload"]["action"], "selfActivate");

    // Action: generate_fluent_v9_pcf
    let call_pcf_raw = tool.execute(json!({
        "action": "generate_fluent_v9_pcf",
        "control_name": "ASTAuditBadge",
        "namespace": "TagisanEnterprise"
    })).await.expect("Dispatch pcf failed");
    let call_pcf: Value = serde_json::from_str(&call_pcf_raw).unwrap();
    assert!(call_pcf["manifest_xml"].as_str().unwrap().contains("<manifest>"));

    // Action: audit_arc_fleet
    let call_arg_raw = tool.execute(json!({
        "action": "audit_arc_fleet"
    })).await.expect("Dispatch arg failed");
    let call_arg: Value = serde_json::from_str(&call_arg_raw).unwrap();
    assert_eq!(call_arg["arc_machines"].as_array().unwrap().len(), 2);

    // Action: analyze_tsql_invariants
    let call_tsql_raw = tool.execute(json!({
        "action": "analyze_tsql_invariants",
        "sql": "CREATE TABLE Inventory ( ItemId INT PRIMARY KEY, SecretCost INT );"
    })).await.expect("Dispatch tsql failed");
    let call_tsql: Value = serde_json::from_str(&call_tsql_raw).unwrap();
    assert!(call_tsql["analysis"]["risk_score"].as_f64().unwrap() > 0.0);

    // Negative / Fuzzing: Unknown action
    let call_invalid = tool.execute(json!({
        "action": "nonexistent_action_xyz"
    })).await;
    assert!(call_invalid.is_err());

    // Empty object handling
    let call_empty = tool.execute(json!({})).await;
    assert!(call_empty.is_err());
}

// =========================================================================
// 9. 100-Worker Concurrency Stress Test
// =========================================================================

#[test]
fn test_15_concurrent_100_workers_multi_threaded_stress() {
    let router = Arc::new(AzureEventGridEngine::new());
    let pim = Arc::new(EntraPimEngine::new("tenant-777"));
    let tsql = Arc::new(TsqlInvariantEngine::new());
    let soar = Arc::new(LogicAppsWorkflowEngine::new());
    let pcf = Arc::new(FluentPcfEngine::new());
    let rms = Arc::new(MipRmsCompoundParser::new());

    let mut handles = Vec::new();

    for worker_id in 0..100 {
        let router_c = Arc::clone(&router);
        let pim_c = Arc::clone(&pim);
        let tsql_c = Arc::clone(&tsql);
        let soar_c = Arc::clone(&soar);
        let pcf_c = Arc::clone(&pcf);
        let rms_c = Arc::clone(&rms);

        let handle = thread::spawn(move || {
            // Task 1: Generate CloudEvent
            let ev = router_c.build_cloud_event(
                &format!("evt-stress-{}", worker_id),
                &format!("workers/worker-{}", worker_id),
                "Microsoft.Tagisan.StressWorkerTick",
                json!({ "worker_id": worker_id, "iteration": 1 }),
            );
            assert_eq!(ev.specversion, "1.0");

            // Task 2: Generate SAS Token
            let sas = router_c.generate_event_hubs_sas_token(
                "https://eventhub.servicebus.windows.net/hub1",
                "WorkerKey",
                "dGVzdC1rZXk=",
                600,
            );
            assert!(sas.contains("SharedAccessSignature"));

            // Task 3: PIM Dual-Agent Attestation
            let ticket = pim_c.validate_dual_agent_attestation("APPROVED", "APPROVED", 95.0, 1.0)
                .expect("Attestation must succeed");
            assert!(ticket.starts_with("TGS-PIM-"));

            // Task 4: T-SQL Invariant Check
            let ddl = format!("CREATE TABLE TenantTable_{} ( TenantID INT, Data VARCHAR(50) );", worker_id);
            let analysis = tsql_c.analyze_sql_script(&ddl);
            assert!(analysis.risk_score > 0.0);

            // Task 5: Sentinel Workflow Synthesis
            let wf = soar_c.generate_sentinel_remediation_workflow(
                &format!("WF-Worker-{}", worker_id),
                "Medium",
                &[RemediationAction::RevokeEntraUserSessions],
            );
            assert_eq!(wf["actions"].as_object().unwrap().len(), 1);

            // Task 6: PCF Manifest & React generation
            let code = pcf_c.generate_fluent_v9_component(&format!("Widget{}", worker_id), "Tgs");
            assert!(code.contains("FluentProvider"));

            // Task 7: RMS Envelope serialization
            let envelope_bytes = rms_c.serialize_pfile_envelope(
                &format!("key-{}", worker_id),
                "<License/>",
                &(worker_id as u64).to_le_bytes(),
            );
            let parsed = rms_c.parse_pfile_envelope(&envelope_bytes).unwrap();
            assert_eq!(parsed.ciphertext_length, 8);
        });

        handles.push(handle);
    }

    for (i, h) in handles.into_iter().enumerate() {
        h.join().unwrap_or_else(|e| panic!("Worker {} panicked: {:?}", i, e));
    }
}
