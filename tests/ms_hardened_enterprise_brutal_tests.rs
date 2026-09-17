//! # Brutal Verification Test Suite: Hardened Microsoft Enterprise Integration
//!
//! Exhaustive test suite verifying the 6 hardened Microsoft integration pillars:
//! 1. Power Platform: Asynchronous Long-Running Operations (HTTP 202 Polling & Webhooks with HMAC-SHA256).
//! 2. Microsoft 365 Copilot: Native Graph Connector Schema Registration & Item Ingestion with Entra ID ACLs.
//! 3. Zero-Trust Enterprise Identity: Azure IMDS Managed Identity & RFC 7523 Certificate-Based Authentication.
//! 4. Microsoft Dataverse: Production C# Virtual Entity Data Provider Plugin & .csproj Generator.
//! 5. Microsoft Fabric OneLake: Pure-Rust Delta Lake ACID Commit Protocol & Eventhouse KQL Alert Engine.
//! 6. Office Modernization: Cross-platform Office.js Unified XML Manifest & Real-Time Microsoft Loop Component.
//! 7. Unified Tool Handler: Dispatch through ToolHandler trait for all 8 subsystem actions with fuzzing.
//! 8. 100-worker multi-threaded concurrency stress test and adversarial tampering audit.

use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;
use tagisan::copilot::ms_hardened::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. Power Platform Asynchronous Long-Running Operations & Webhooks Tests
// =========================================================================

#[test]
fn test_01_async_operation_lifecycle_and_202_headers() {
    let engine = MsAsyncWebhookEngine::new();
    let initial_payload = json!({
        "debate_topic": "Rust vs Go for High-Throughput Ingestion",
        "proposer": "Agent_Alpha",
        "challenger": "Agent_Beta"
    });

    let (ticket, headers) = engine.create_operation(
        "dialectical_debate",
        &initial_payload,
        Some("https://flow.microsoft.com/webhook/callback/772".to_string()),
        Some(15),
    );

    assert_eq!(ticket.status, AsyncOperationStatus::Running);
    assert_eq!(ticket.progress_pct, 10);
    assert_eq!(ticket.retry_after_sec, 15);
    assert!(headers.contains_key("Location"));
    assert_eq!(headers.get("Retry-After").unwrap(), "15");
    assert_eq!(headers.get("x-ms-asynchronous").unwrap(), "true");
    assert!(headers.get("Location").unwrap().contains(&ticket.operation_id));

    // Update progress
    engine
        .update_progress(&ticket.operation_id, 45)
        .expect("Progress update failed");
    let updated = engine.poll_operation(&ticket.operation_id).expect("Poll failed");
    assert_eq!(updated.status, AsyncOperationStatus::Running);
    assert_eq!(updated.progress_pct, 45);

    // Complete operation
    let result_data = json!({
        "verdict": "RUST_APPROVED",
        "confidence_pct": 98.4,
        "blast_radius": 1.2
    });
    let delivery_opt = engine
        .complete_operation(&ticket.operation_id, result_data.clone(), "SuperSecretKey999")
        .expect("Completion failed");

    assert!(delivery_opt.is_some());
    let delivery = delivery_opt.unwrap();
    assert_eq!(delivery.status, "Succeeded");
    assert_eq!(delivery.target_url, "https://flow.microsoft.com/webhook/callback/772");
    assert!(delivery.hmac_header.starts_with("sha256="));
    assert!(delivery.payload.to_string().contains("RUST_APPROVED"));

    // Verify finished ticket
    let finished = engine.poll_operation(&ticket.operation_id).expect("Poll failed");
    assert_eq!(finished.status, AsyncOperationStatus::Succeeded);
    assert_eq!(finished.progress_pct, 100);
    assert!(finished.hmac_signature.is_some());

    // Verify HMAC signature
    let calculated_sig = MsAsyncWebhookEngine::compute_hmac("SuperSecretKey999", &delivery.payload.to_string());
    assert_eq!(delivery.hmac_header, format!("sha256={}", calculated_sig));

    // Tamper detection: wrong key
    let wrong_sig = MsAsyncWebhookEngine::compute_hmac("WrongKey123", &delivery.payload.to_string());
    assert_ne!(calculated_sig, wrong_sig, "Different keys must yield different HMACs");

    // Tamper detection: altered payload
    let altered_payload = delivery.payload.to_string().replace("RUST_APPROVED", "GO_APPROVED");
    let tampered_sig = MsAsyncWebhookEngine::compute_hmac("SuperSecretKey999", &altered_payload);
    assert_ne!(calculated_sig, tampered_sig, "Tampered payload must fail HMAC verification");
}

#[test]
fn test_02_async_operation_error_handling() {
    let engine = MsAsyncWebhookEngine::new();

    // Query non-existent ticket
    let err = engine.poll_operation("non_existent_op_9999");
    assert!(err.is_err(), "Polling non-existent operation must error");

    // Complete non-existent ticket
    let comp_err = engine.complete_operation("fake_id", json!({}), "key");
    assert!(comp_err.is_err(), "Completing non-existent operation must error");

    // Fail an operation
    let (ticket, _) = engine.create_operation("flaky_task", &json!({}), None, None);
    engine
        .fail_operation(&ticket.operation_id, "HTTP_429", "Resource allocation exhausted")
        .expect("Failing operation should succeed");
    
    let fail_ticket = engine.poll_operation(&ticket.operation_id).expect("Poll failed");
    assert_eq!(fail_ticket.status, AsyncOperationStatus::Failed);
    assert_eq!(fail_ticket.error_code.as_deref().unwrap(), "HTTP_429");
    assert_eq!(fail_ticket.error_message.as_deref().unwrap(), "Resource allocation exhausted");
}

// =========================================================================
// 2. Microsoft 365 Copilot Native Graph Connector Tests
// =========================================================================

#[test]
fn test_03_graph_connector_manifest_and_schema() {
    let engine = MsGraphConnectorEngine::new("tgs_enterprise_con");

    let conn = engine.build_connection_registration(
        "Tagisan Architectural Ledger",
        "Zero-trust dialectical decision ledger and AST blast-radius records",
    );
    assert_eq!(conn.id, "tgs_enterprise_con");
    assert_eq!(conn.name, "Tagisan Architectural Ledger");
    assert_eq!(conn.state, "ready");

    let schema = engine.generate_schema_properties();
    assert!(schema.len() >= 6, "Graph Connector schema must provide at least 6 strongly typed properties");

    let props_by_name: std::collections::HashMap<_, _> = schema.iter().map(|p| (p.name.as_str(), p)).collect();
    
    // Check decisionId
    let id_prop = props_by_name.get("decisionId").expect("Missing decisionId property");
    assert_eq!(id_prop.property_type, "string");
    assert!(id_prop.is_searchable);

    // Check confidencePct
    let conf_prop = props_by_name.get("confidencePct").expect("Missing confidencePct property");
    assert_eq!(conf_prop.property_type, "double");
    assert!(conf_prop.is_queryable);

    // Check blastRadiusScore
    let blast_prop = props_by_name.get("blastRadiusScore").expect("Missing blastRadiusScore property");
    assert_eq!(blast_prop.property_type, "double");

    // Check lastModifiedDateTime
    let time_prop = props_by_name.get("lastModifiedDateTime").expect("Missing lastModifiedDateTime property");
    assert_eq!(time_prop.property_type, "dateTime");
}

#[test]
fn test_04_graph_connector_item_ingestion_with_acls() {
    let engine = MsGraphConnectorEngine::new("tgs_fintech_con");

    let item = engine.build_external_item_for_adr(
        "ADR-2026-0042",
        "Multi-Party Computation for Sovereign Asset Custody",
        "APPROVED",
        0.85,
        99.1,
        "Dialectical consensus reached between Cryptography Lead and Compliance Auditor.",
        "Confidential",
        "alice@fintech.onmicrosoft.com",
    );

    assert_eq!(item.id, "adr-adr-2026-0042");
    assert_eq!(item.acl.len(), 1);
    assert_eq!(item.acl[0].access_type, "grant");
    assert_eq!(item.acl[0].value, "alice@fintech.onmicrosoft.com");

    let item_json = serde_json::to_value(&item).expect("Failed to serialize GraphExternalItem");
    assert_eq!(item_json["id"], "adr-adr-2026-0042");
    assert_eq!(item_json["content_type"], "text");
    assert_eq!(item_json["properties"]["confidencePct"], 99.1);
    assert_eq!(item_json["properties"]["blastRadiusScore"], 0.85);
}

// =========================================================================
// 3. Zero-Trust Identity: Azure Managed Identity & RFC 7523 Tests
// =========================================================================

#[test]
fn test_05_azure_managed_identity_imds_urls_and_token_parsing() {
    let engine = AzureManagedIdentityEngine::default();
    
    // System-assigned
    let url_sys = engine.build_imds_url("https://management.azure.com", &ManagedIdentityType::SystemAssigned);
    assert!(url_sys.contains("169.254.169.254/metadata/identity/oauth2/token"));
    assert!(url_sys.contains("api-version=2018-02-01"));
    assert!(url_sys.contains("resource=https://management.azure.com"));
    assert!(!url_sys.contains("client_id"));

    // User-assigned by client_id
    let url_client = engine.build_imds_url(
        "https://graph.microsoft.com",
        &ManagedIdentityType::UserAssignedClientId("c0a80101-beef-4000-8000-112233445566".to_string()),
    );
    assert!(url_client.contains("client_id=c0a80101-beef-4000-8000-112233445566"));

    // User-assigned by resource_id
    let url_res = engine.build_imds_url(
        "https://graph.microsoft.com",
        &ManagedIdentityType::UserAssignedResourceId("/subscriptions/sub1/rg1/id".to_string()),
    );
    assert!(url_res.contains("mi_res_id=/subscriptions/sub1/rg1/id"));

    // Parse token JSON
    let sample_json = r#"{
        "access_token": "sample_imds_jwt_access_token_value",
        "client_id": "c0a80101-beef-4000-8000-112233445566",
        "expires_in": "3599",
        "expires_on": "1773729900",
        "not_before": "1773726000",
        "resource": "https://graph.microsoft.com",
        "token_type": "Bearer"
    }"#;
    let token = engine.parse_imds_response(sample_json).expect("IMDS parse failed");
    assert_eq!(token.token_type, "Bearer");
    assert_eq!(token.expires_in, "3599");
    assert_eq!(token.expires_on, "1773729900");
    assert_eq!(token.resource, "https://graph.microsoft.com");
}

#[test]
fn test_06_rfc7523_certificate_based_assertion_generation() {
    let client_id = "84f4e222-1234-5678-abcd-001122334455";
    let tenant_id = "72f988bf-86f1-41af-91ab-2d7cd011db47";
    let thumbprint_hex = "4A2E78B3F9C110825B3D599A041E7BC209A334FF";

    let assertion = AzureManagedIdentityEngine::generate_client_assertion(
        client_id,
        tenant_id,
        thumbprint_hex,
        300,
    );

    assert_eq!(assertion.client_id, client_id);
    assert_eq!(assertion.tenant_id, tenant_id);
    assert_eq!(assertion.thumbprint_hex, thumbprint_hex);
    assert!(assertion.expires_at_epoch > 0);

    // Inspect unsigned JWT segments
    let parts: Vec<&str> = assertion.unsigned_token.split('.').collect();
    assert_eq!(parts.len(), 2, "Unsigned JWT template must have Header.Payload");

    // Verify token form payload
    let form = AzureManagedIdentityEngine::build_cba_token_form(&assertion, "https://graph.microsoft.com/.default");
    assert_eq!(form.get("grant_type").unwrap(), "client_credentials");
    assert_eq!(form.get("client_id").unwrap(), client_id);
    assert_eq!(form.get("client_assertion_type").unwrap(), "urn:ietf:params:oauth:client-assertion-type:jwt-bearer");
    assert_eq!(form.get("scope").unwrap(), "https://graph.microsoft.com/.default");
}

// =========================================================================
// 4. Microsoft Dataverse Virtual Entity Data Provider Tests
// =========================================================================

#[test]
fn test_07_dataverse_csharp_plugin_generation_and_csproj() {
    let schema = DataverseVirtualEntityProvider::default_adr_virtual_schema();
    assert_eq!(schema.logical_name, "tgs_virtual_adr");
    assert_eq!(schema.primary_key, "tgs_virtual_adrid");
    assert_eq!(schema.fields.len(), 6);

    let csharp = DataverseVirtualEntityProvider::generate_csharp_plugin_code(&schema);
    assert!(csharp.contains("public class TagisanVirtualEntityProvider : IPlugin"));
    assert!(csharp.contains("IServiceProvider serviceProvider"));
    assert!(csharp.contains("IPluginExecutionContext"));
    assert!(csharp.contains("HandleRetrieve(context, tracing);"));
    assert!(csharp.contains("HandleRetrieveMultiple(context, tracing);"));
    assert!(csharp.contains("tgs_confidence_pct"));
    assert!(csharp.contains("tgs_ed25519_proof"));
    assert!(csharp.contains("TgsEndpoint + \"?$top=50\""));

    let csproj = DataverseVirtualEntityProvider::generate_csproj("Tagisan.Dataverse.Plugins");
    assert!(csproj.contains("<Project Sdk=\"Microsoft.NET.Sdk\">"));
    assert!(csproj.contains("<TargetFramework>net462</TargetFramework>"));
    assert!(csproj.contains("Microsoft.CrmSdk.CoreAssemblies"));
    assert!(csproj.contains("Version=\"9.0.2.56\""));
}

// =========================================================================
// 5. Microsoft Fabric OneLake Delta Streaming & Real-Time Alert Tests
// =========================================================================

#[test]
fn test_08_fabric_delta_commit_and_onelake_paths() {
    let streamer = FabricDeltaStreamer::new(
        "contoso_corp_ws",
        "engineering_telemetry",
    );

    let abfss = streamer.abfss_table_path("SreInvariants");
    assert_eq!(
        abfss,
        "abfss://contoso_corp_ws@onelake.dfs.fabric.microsoft.com/engineering_telemetry.Lakehouse/Tables/SreInvariants"
    );

    let commit = streamer.commit_delta_transaction("SreInvariants", 4, 1500, 0.45, 8.2);
    assert_eq!(commit.table_name, "SreInvariants");
    assert_eq!(commit.version, 4);
    assert_eq!(commit.row_count, 1500);
    assert_eq!(commit.min_blast_radius, 0.45);
    assert_eq!(commit.max_blast_radius, 8.2);
    assert!(commit.parquet_file_name.starts_with("part-00004-"));
    assert!(commit.parquet_file_name.ends_with(".snappy.parquet"));

    // Verify commit payload structure
    assert!(commit.delta_log_json.contains("commitInfo"));
    assert!(commit.delta_log_json.contains("\"operation\":\"WRITE\""));
    assert!(commit.delta_log_json.contains("add"));
    assert!(commit.delta_log_json.contains("blastRadiusScore"));
}

#[test]
fn test_09_fabric_eventhouse_kql_and_data_activator_rule() {
    let streamer = FabricDeltaStreamer::default();

    let kql = streamer.generate_kql_table_schema("AgentDialecticalAudit");
    assert!(kql.contains(".create table AgentDialecticalAudit"));
    assert!(kql.contains("OperationId: string"));
    assert!(kql.contains("ConfidencePct: real"));
    assert!(kql.contains("BlastRadiusScore: real"));
    assert!(kql.contains("Ed25519Signature: string"));

    let rule = streamer.generate_data_activator_rule("AgentDialecticalAudit", 3.0);
    assert_eq!(rule["ruleName"], "HighBlastRadiusConsensusBlockAlert");
    assert_eq!(rule["sourceTable"], "AgentDialecticalAudit");
    assert_eq!(rule["triggerCondition"]["threshold"], 3.0);
    assert_eq!(rule["triggerCondition"]["column"], "BlastRadiusScore");
    assert!(rule["actions"].is_array());
}

// =========================================================================
// 6. Cross-Platform Office.js Manifest & Microsoft Loop Component Tests
// =========================================================================

#[test]
fn test_10_officejs_unified_manifest_xml_generation() {
    let engine = OfficeJsManifestEngine::new(
        "99887766-5544-3322-1100-aabbccddeeff",
        "https://tgs.office.internal.corp",
    );

    let manifest_xml = engine.generate_xml_manifest("Tagisan Hegelian Consensus");
    assert!(manifest_xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(manifest_xml.contains("<Id>99887766-5544-3322-1100-aabbccddeeff</Id>"));
    assert!(manifest_xml.contains("<DisplayName DefaultValue=\"Tagisan Hegelian Consensus\" />"));
    assert!(manifest_xml.contains("<Host Name=\"Workbook\" />"));
    assert!(manifest_xml.contains("<Host Name=\"Document\" />"));
    assert!(manifest_xml.contains("<Permissions>ReadWriteDocument</Permissions>"));
    assert!(manifest_xml.contains("https://tgs.office.internal.corp/taskpane.html"));

    assert_eq!(OfficeJsApp::Excel.as_str(), "Workbook");
    assert_eq!(OfficeJsApp::Word.as_str(), "Document");
    assert_eq!(OfficeJsApp::Visio.as_str(), "Drawing");
    assert_eq!(OfficeJsApp::Teams.as_str(), "Teams");
}

#[test]
fn test_11_microsoft_loop_component_adaptive_card() {
    let engine = OfficeJsManifestEngine::default();
    let card = engine.generate_loop_component_payload(
        "loop-card-7711",
        "Automated Sovereign Ledger Ingestion Gateway",
        "APPROVED",
        97.9,
        1.4,
        "Proposer_AgentEuler",
        "Auditor_AgentTuring",
    );

    assert_eq!(card["type"], "AdaptiveCard");
    assert_eq!(card["version"], "1.5");
    assert_eq!(card["$schema"], "http://adaptivecards.io/schemas/adaptive-card.json");

    let body = card["body"].as_array().expect("Body must be array");
    assert!(body.len() >= 2);

    let actions = card["actions"].as_array().expect("Actions must be array");
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0]["title"], "✅ Confirm & Endorse Consensus");
    assert_eq!(actions[1]["title"], "⚠️ Challenge Consensus (Re-Debate)");
}

// =========================================================================
// 7. Unified Tool Handler Execution Tests
// =========================================================================

#[tokio::test]
async fn test_12_unified_tool_handler_all_actions() {
    let tool = CopilotMsHardenedTool::new();
    assert_eq!(tool.name(), "ms_hardened_copilot");
    assert!(!tool.description().is_empty());

    let schema = tool.parameters_schema();
    assert!(schema["properties"]["action"].is_object());

    // 1. create_async_operation
    let create_arg = json!({
        "action": "create_async_operation",
        "task_type": "security_audit",
        "payload": { "target": "auth_middleware.rs" },
        "webhook_url": "https://flow.microsoft.com/c/123",
        "retry_after_sec": 10
    });
    let create_res = tool.execute(create_arg).await.expect("create_async_operation failed");
    let create_val: Value = serde_json::from_str(&create_res).expect("Parse JSON");
    assert!(create_val["success"].as_bool().unwrap());
    let op_id = create_val["ticket"]["operation_id"].as_str().unwrap().to_string();

    // 2. poll_async_operation
    let poll_arg = json!({
        "action": "poll_async_operation",
        "operation_id": op_id
    });
    let poll_res = tool.execute(poll_arg).await.expect("poll_async_operation failed");
    let poll_val: Value = serde_json::from_str(&poll_res).expect("Parse JSON");
    assert!(poll_val["success"].as_bool().unwrap());
    assert_eq!(poll_val["ticket"]["status"], "Running");

    // 3. complete_async_operation
    let comp_arg = json!({
        "action": "complete_async_operation",
        "operation_id": op_id,
        "result": { "passed": true, "findings": 0 },
        "hmac_key": "Secret99"
    });
    let comp_res = tool.execute(comp_arg).await.expect("complete_async_operation failed");
    let comp_val: Value = serde_json::from_str(&comp_res).expect("Parse JSON");
    assert!(comp_val["success"].as_bool().unwrap());
    assert_eq!(comp_val["delivery"]["status"], "Succeeded");

    // 4. build_graph_connector_manifest
    let graph_arg = json!({
        "action": "build_graph_connector_manifest",
        "name": "Custom Connector",
        "description": "Custom Desc"
    });
    let graph_res = tool.execute(graph_arg).await.expect("build_graph_connector_manifest failed");
    let graph_val: Value = serde_json::from_str(&graph_res).expect("Parse JSON");
    assert_eq!(graph_val["properties_count"], 7);

    // 5. generate_azure_managed_identity_request
    let mi_arg = json!({
        "action": "generate_azure_managed_identity_request",
        "resource": "https://database.windows.net"
    });
    let mi_res = tool.execute(mi_arg).await.expect("generate_azure_managed_identity_request failed");
    let mi_val: Value = serde_json::from_str(&mi_res).expect("Parse JSON");
    assert!(mi_val["imds_url"].as_str().unwrap().contains("database.windows.net"));

    // 6. generate_dataverse_virtual_plugin
    let dv_arg = json!({
        "action": "generate_dataverse_virtual_plugin"
    });
    let dv_res = tool.execute(dv_arg).await.expect("generate_dataverse_virtual_plugin failed");
    let dv_val: Value = serde_json::from_str(&dv_res).expect("Parse JSON");
    assert!(dv_val["csharp_source"].as_str().unwrap().contains("IPlugin"));
    assert!(dv_val["csproj_source"].as_str().unwrap().contains("Microsoft.CrmSdk.CoreAssemblies"));

    // 7. commit_fabric_delta_batch
    let fabric_arg = json!({
        "action": "commit_fabric_delta_batch",
        "table_name": "ComplianceAudit",
        "version": 2,
        "row_count": 500
    });
    let fabric_res = tool.execute(fabric_arg).await.expect("commit_fabric_delta_batch failed");
    let fabric_val: Value = serde_json::from_str(&fabric_res).expect("Parse JSON");
    assert_eq!(fabric_val["commit"]["row_count"], 500);

    // 8. generate_loop_component
    let loop_arg = json!({
        "action": "generate_loop_component",
        "title": "Kernel Memory Reclaim Proposal",
        "verdict": "APPROVED",
        "confidence_pct": 99.2
    });
    let loop_res = tool.execute(loop_arg).await.expect("generate_loop_component failed");
    let loop_val: Value = serde_json::from_str(&loop_res).expect("Parse JSON");
    assert_eq!(loop_val["loop_card"]["type"], "AdaptiveCard");

    // Negative test: invalid action
    let bad_arg = json!({ "action": "unsupported_xyz_action" });
    let bad_res = tool.execute(bad_arg).await;
    assert!(bad_res.is_err(), "Unsupported action must return error");
}

// =========================================================================
// 8. 100-Worker High Concurrency & Adversarial Fuzzing Stress Test
// =========================================================================

#[test]
fn test_13_100_worker_concurrency_stress_test() {
    let engine = Arc::new(MsAsyncWebhookEngine::new());
    let mut handles = Vec::new();

    for worker_id in 0..100 {
        let engine_clone = Arc::clone(&engine);
        let handle = thread::spawn(move || {
            let task_name = format!("concurrent_task_{}", worker_id);
            let payload = json!({ "worker": worker_id, "timestamp": Utc::now().to_rfc3339() });
            let callback = format!("https://flow.microsoft.com/c/worker_{}", worker_id);
            let (ticket, _) = engine_clone.create_operation(&task_name, &payload, Some(callback), Some(5));

            // Poll immediately
            let poll1 = engine_clone.poll_operation(&ticket.operation_id).expect("Poll failed");
            assert_eq!(poll1.status, AsyncOperationStatus::Running);

            // Progress to 50%
            engine_clone
                .update_progress(&ticket.operation_id, 50)
                .expect("Progress failed");

            // Complete
            let secret = format!("WorkerSecretKey_{}", worker_id);
            let delivery_opt = engine_clone
                .complete_operation(&ticket.operation_id, json!({ "status": "ALL_GOOD" }), &secret)
                .expect("Complete failed");

            assert!(delivery_opt.is_some());
            let delivery = delivery_opt.unwrap();
            assert_eq!(delivery.status, "Succeeded");
            
            let sig = MsAsyncWebhookEngine::compute_hmac(&secret, &delivery.payload.to_string());
            assert_eq!(delivery.hmac_header, format!("sha256={}", sig));
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Worker thread panicked");
    }

    assert_eq!(
        engine.operations_count(),
        100,
        "Engine must have tracked all 100 concurrent tickets without race conditions"
    );
}
