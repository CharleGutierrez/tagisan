//! # Brutal Verification Test Suite: Microsoft Frontier Cloud & Enterprise Ecosystem Integration
//!
//! Exhaustive test suite verifying the 7 frontier enterprise Microsoft integration pillars:
//! 1. Azure AI Foundry & TypeSpec Gateway: TypeSpec (.tsp), Semantic Kernel 1.x plugins, Prompty, and Rate Limit throttling.
//! 2. Azure API Management (APIM): Enterprise XML policies (<validate-jwt>, <rate-limit>, <ip-filter>) and Bicep bindings.
//! 3. Microsoft Defender XDR: Advanced Hunting KQL queries, Graph Security API parser, and Live Response forensics.
//! 4. Microsoft Fabric Lakehouse: Bronze/Silver/Gold PySpark notebooks, Delta V-Order DDL, and Fabric Git .platform metadata.
//! 5. Azure Kubernetes & KEDA: ScaledObject CRDs for queue depth triggers and hardened Pod manifests with Workload Identity.
//! 6. Pure-Rust Dataverse SolutionPackager: In-memory PKZIP unpacker/packer and publisher prefix schema integrity audit.
//! 7. Windows Named Pipes & WinUI 3 IPC: \\.\pipe\tagisan_ipc binary framing with CRC-32 integrity and tagisan:// URL parser.
//! 8. Unified Tool Handler: Async dispatch through ToolHandler trait for all 7 subsystem actions with edge-case fuzzing.
//! 9. 100-worker multi-threaded concurrency stress test and adversarial tampering audit.

use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tagisan::copilot::ms_ecosystem_frontier::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. Azure AI Foundry, TypeSpec & Semantic Kernel Tests
// =========================================================================

#[test]
fn test_01_typespec_synthesis_and_openapi_structure() {
    let engine = AzureAiFoundryEngine::new();
    let endpoints = vec![
        TypeSpecEndpoint {
            name: "evaluateConsensus".to_string(),
            method: "POST".to_string(),
            path: "/consensus/evaluate".to_string(),
            doc: "Run dialectical debate and evaluate consensus verdict".to_string(),
            request_body_type: Some("DebateProposalRequest".to_string()),
            response_type: "ConsensusVerdictResponse".to_string(),
        },
        TypeSpecEndpoint {
            name: "getTelemetry".to_string(),
            method: "GET".to_string(),
            path: "/telemetry/{symbol}".to_string(),
            doc: "Retrieve AST blast radius telemetry for a symbol".to_string(),
            request_body_type: None,
            response_type: "TelemetryRecord".to_string(),
        },
    ];

    let tsp = engine.generate_typespec_definition("TagisanEnterpriseEngine", "Tagisan.Enterprise", &endpoints);

    assert!(tsp.contains("import \"@typespec/http\";"));
    assert!(tsp.contains("import \"@typespec/rest\";"));
    assert!(tsp.contains("import \"@typespec/openapi3\";"));
    assert!(tsp.contains("namespace Tagisan.Enterprise;"));
    assert!(tsp.contains("@route(\"/consensus/evaluate\")"));
    assert!(tsp.contains("@post"));
    assert!(tsp.contains("op evaluateConsensus(@body body: DebateProposalRequest, ): ConsensusVerdictResponse;"));
    assert!(tsp.contains("@route(\"/telemetry/{symbol}\")"));
    assert!(tsp.contains("@get"));
    assert!(tsp.contains("op getTelemetry(): TelemetryRecord;"));
}

#[test]
fn test_02_semantic_kernel_plugin_and_prompty_assets() {
    let engine = AzureAiFoundryEngine::new();

    // Test Semantic Kernel 1.x plugin manifest
    let sk_plugin = engine.generate_semantic_kernel_plugin(
        "TagisanGovernancePlugin",
        "Multi-agent architectural governance and dialectical consensus",
        &[
            SkFunctionDef {
                name: "AuditBlastRadius".to_string(),
                description: "Compute transitive blast radius of a symbol refactor".to_string(),
                parameters: vec![
                    ("symbol".to_string(), "string".to_string(), "Name of the symbol to evaluate".to_string()),
                    ("max_depth".to_string(), "number".to_string(), "Maximum graph traversal depth".to_string()),
                ],
            },
            SkFunctionDef {
                name: "ExecuteDialecticalDebate".to_string(),
                description: "Trigger 3-round Proposer vs Challenger debate".to_string(),
                parameters: vec![
                    ("proposal".to_string(), "string".to_string(), "Architectural proposal text".to_string()),
                ],
            },
        ],
    );

    assert_eq!(sk_plugin["schema"], "1.0");
    assert_eq!(sk_plugin["name"], "TagisanGovernancePlugin");
    let funcs = sk_plugin["functions"].as_array().expect("Functions must be array");
    assert_eq!(funcs.len(), 2);
    assert_eq!(funcs[0]["name"], "AuditBlastRadius");
    assert_eq!(funcs[0]["parameters"].as_array().unwrap().len(), 2);
    assert_eq!(funcs[1]["name"], "ExecuteDialecticalDebate");

    // Test Prompty asset generation
    let prompty = engine.generate_prompty_asset(
        "SovereignAuditor",
        "gpt-4o-msft-internal",
        "You are a principal enterprise architect reviewing T-SQL invariants.",
        "Evaluate the following schema:\n{{task}}",
    );
    assert!(prompty.starts_with("---"));
    assert!(prompty.contains("name: SovereignAuditor"));
    assert!(prompty.contains("azure_deployment: gpt-4o-msft-internal"));
    assert!(prompty.contains("system:\nYou are a principal enterprise architect"));
    assert!(prompty.contains("user:\nEvaluate the following schema:"));
}

#[test]
fn test_03_azure_rate_limit_header_parsing_and_backoff() {
    let engine = AzureAiFoundryEngine::new();

    // Case 1: Healthy rate limit state
    let healthy_headers = vec![
        ("x-ratelimit-remaining-requests".to_string(), "120".to_string()),
        ("x-ratelimit-remaining-tokens".to_string(), "45000".to_string()),
    ];
    let state_healthy = engine.parse_azure_rate_limit_headers(&healthy_headers);
    assert_eq!(state_healthy.remaining_requests, Some(120));
    assert_eq!(state_healthy.remaining_tokens, Some(45000));
    assert_eq!(state_healthy.retry_after_ms, None);
    assert!(!state_healthy.should_throttle);
    assert_eq!(state_healthy.recommended_delay_ms, 0);

    // Case 2: Throttled with explicit Retry-After-Ms header
    let throttled_ms = vec![
        ("x-ratelimit-remaining-requests".to_string(), "0".to_string()),
        ("retry-after-ms".to_string(), "1850".to_string()),
    ];
    let state_ms = engine.parse_azure_rate_limit_headers(&throttled_ms);
    assert!(state_ms.should_throttle);
    assert_eq!(state_ms.retry_after_ms, Some(1850));
    assert_eq!(state_ms.recommended_delay_ms, 1850);

    // Case 3: Throttled with standard Retry-After header in seconds
    let throttled_secs = vec![
        ("retry-after".to_string(), "3".to_string()),
    ];
    let state_secs = engine.parse_azure_rate_limit_headers(&throttled_secs);
    assert!(state_secs.should_throttle);
    assert_eq!(state_secs.retry_after_ms, Some(3000));
    assert_eq!(state_secs.recommended_delay_ms, 3000);

    // Case 4: Low token threshold backoff trigger (< 1000 tokens remaining)
    let low_tokens = vec![
        ("x-ratelimit-remaining-tokens".to_string(), "450".to_string()),
    ];
    let state_low = engine.parse_azure_rate_limit_headers(&low_tokens);
    assert!(state_low.should_throttle);
    assert_eq!(state_low.recommended_delay_ms, 500);
}

// =========================================================================
// 2. Azure API Management (APIM) Enterprise Policy Tests
// =========================================================================

#[test]
fn test_04_apim_policies_xml_synthesis_and_bicep() {
    let engine = AzureApimPolicyEngine::new();
    let tenant_id = "72f988bf-86f1-41af-91ab-2d7cd011db47";
    let client_id = "11112222-3333-4444-5555-666677778888";
    let allowed_ips = vec!["10.240.0.0/16".to_string(), "192.168.1.50".to_string()];

    let policy_xml = engine.generate_apim_policy_xml(tenant_id, client_id, 1200, &allowed_ips);

    assert!(policy_xml.contains("<policies>"));
    assert!(policy_xml.contains("<validate-jwt header-name=\"Authorization\""));
    assert!(policy_xml.contains(&format!("<openid-config url=\"https://login.microsoftonline.com/{}/v2.0/.well-known/openid-configuration\" />", tenant_id)));
    assert!(policy_xml.contains(&format!("<audience>{}</audience>", client_id)));
    assert!(policy_xml.contains("<rate-limit-by-key calls=\"1200\" renewal-period=\"60\""));
    assert!(policy_xml.contains("<ip-filter action=\"allow\">"));
    assert!(policy_xml.contains("<address>10.240.0.0/16</address>"));
    assert!(policy_xml.contains("<address>192.168.1.50</address>"));
    assert!(policy_xml.contains("<authentication-managed-identity resource=\"https://cognitiveservices.azure.com\" />"));
    assert!(policy_xml.contains("<set-header name=\"x-tagisan-gateway-version\" exists-action=\"override\">"));

    // Verify Bicep template
    let bicep = engine.generate_apim_bicep_template("tagisan-core-api", "https://tgs-internal.azurewebsites.net", "corp-apim-prod");
    assert!(bicep.contains("resource apimService 'Microsoft.ApiManagement/service@2023-05-01-preview' existing"));
    assert!(bicep.contains("resource tagisanApi 'Microsoft.ApiManagement/service/apis@2023-05-01-preview'"));
    assert!(bicep.contains("displayName: 'tagisan-core-api Gateway'"));
    assert!(bicep.contains("path: 'tagisan-core-api'"));
}

// =========================================================================
// 3. Microsoft Defender XDR Advanced Hunting & Live Response Tests
// =========================================================================

#[test]
fn test_05_defender_xdr_hunting_kql_and_parser() {
    let engine = DefenderXdrHuntingEngine::new();

    // Query builder test
    let kql = engine.build_advanced_hunting_kql(
        "DeviceProcessEvents",
        "FileName =~ 'cmd.exe' and ProcessCommandLine has 'whoami'",
        &["Timestamp", "DeviceName", "AccountName", "ProcessCommandLine"],
        25,
    );
    assert_eq!(
        kql,
        "DeviceProcessEvents | where FileName =~ 'cmd.exe' and ProcessCommandLine has 'whoami' | project Timestamp, DeviceName, AccountName, ProcessCommandLine | take 25"
    );

    // Response parsing test
    let mock_json = r#"{
        "Results": [
            {
                "Timestamp": "2026-09-17T08:15:30Z",
                "DeviceName": "CORP-FIN-WS-09",
                "ActionType": "ProcessCreated",
                "AccountName": "admin_jdoe",
                "RemoteIP": "10.0.5.21",
                "ProcessCommandLine": "cmd.exe /c whoami /all",
                "SHA256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            },
            {
                "Timestamp": "2026-09-17T08:16:10Z",
                "DeviceName": "CORP-SQL-SRV-01",
                "ActionType": "ConnectionSuccess",
                "AccountName": "SYSTEM",
                "RemoteIP": "192.168.10.4",
                "LocalPort": "1433"
            }
        ]
    }"#;

    let records = engine.parse_hunting_query_results(mock_json).expect("Hunting parse failed");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].device_name, "CORP-FIN-WS-09");
    assert_eq!(records[0].action_type, "ProcessCreated");
    assert_eq!(records[0].remote_ip, Some("10.0.5.21".to_string()));
    assert_eq!(records[0].additional_fields.get("ProcessCommandLine").unwrap(), "\"cmd.exe /c whoami /all\"");

    assert_eq!(records[1].device_name, "CORP-SQL-SRV-01");
    assert_eq!(records[1].action_type, "ConnectionSuccess");
    assert_eq!(records[1].remote_ip, Some("192.168.10.4".to_string()));
}

#[test]
fn test_06_defender_live_response_script_synthesis() {
    let engine = DefenderXdrHuntingEngine::new();
    let actions = vec![
        LiveResponseAction::IsolateProcess { pid: 8842 },
        LiveResponseAction::DumpProcessMemory { pid: 8842 },
        LiveResponseAction::QuarantineFile { path: "C:\\Windows\\Temp\\malware.exe".to_string() },
        LiveResponseAction::CollectForensicFile { path: "C:\\Windows\\System32\\winevt\\Logs\\Security.evtx".to_string() },
    ];

    let script = engine.generate_live_response_script("mach-guid-corp-99", &actions);

    assert_eq!(script["machineId"], "mach-guid-corp-99");
    assert_eq!(script["initiatedBy"], "Tagisan Autonomous Threat Responder");
    let commands = script["commands"].as_array().expect("Commands must be array");
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[0]["arguments"], "process terminate 8842");
    assert_eq!(commands[1]["arguments"], "procdump 8842");
    assert_eq!(commands[2]["arguments"], "quarantine C:\\Windows\\Temp\\malware.exe");
    assert_eq!(commands[3]["arguments"], "C:\\Windows\\System32\\winevt\\Logs\\Security.evtx");
}

// =========================================================================
// 4. Microsoft Fabric Lakehouse Medallion Pipeline Tests
// =========================================================================

#[test]
fn test_07_fabric_medallion_notebook_generation() {
    let engine = FabricLakehouseMedallionEngine::new();
    let nb = engine.generate_medallion_pyspark_notebook(
        "workspace-financial-analytics",
        "LakehouseTreasury",
        "wire_transfers",
    );

    assert_eq!(nb["nbformat"], 4);
    assert_eq!(nb["metadata"]["kernel_info"]["name"], "synapse_pyspark");
    let cells = nb["cells"].as_array().expect("Cells must be array");
    assert_eq!(cells.len(), 3);

    // Verify Bronze Cell
    let bronze_code = cells[0]["source"][0].as_str().unwrap();
    assert!(bronze_code.contains("abfss://workspace-financial-analytics@onelake.dfs.fabric.microsoft.com/LakehouseTreasury.Lakehouse/Files/raw/wire_transfers"));
    assert!(bronze_code.contains("saveAsTable(\"wire_transfers_bronze\")"));

    // Verify Silver Cell
    let silver_code = cells[1]["source"][0].as_str().unwrap();
    assert!(silver_code.contains(".dropDuplicates([\"id\"])"));
    assert!(silver_code.contains("saveAsTable(\"wire_transfers_silver\")"));

    // Verify Gold Cell
    let gold_code = cells[2]["source"][0].as_str().unwrap();
    assert!(gold_code.contains("spark.sql(\"OPTIMIZE wire_transfers_gold ZORDER BY (status)\")"));

    // Verify V-Order DDL
    let vorder_ddl = engine.generate_vorder_optimization_ddl("wire_transfers_gold", &["account_id", "booking_date"]);
    assert!(vorder_ddl.contains("'delta.enableDeletionVectors' = 'true'"));
    assert!(vorder_ddl.contains("OPTIMIZE wire_transfers_gold ZORDER BY (account_id, booking_date);"));

    // Verify Fabric Git .platform metadata
    let platform = engine.generate_fabric_git_platform_metadata("Notebook", "MedallionIngestNotebook");
    assert_eq!(platform["metadata"]["type"], "Notebook");
    assert_eq!(platform["metadata"]["displayName"], "MedallionIngestNotebook");
    assert!(platform["config"]["logicalId"].as_str().unwrap().starts_with("tgs-fabric-"));
}

// =========================================================================
// 5. Azure Kubernetes & KEDA Scaler Tests
// =========================================================================

#[test]
fn test_08_keda_scaler_and_hardened_pod_manifests() {
    let engine = KedaKubernetesScalerEngine::new();

    let keda_yaml = engine.generate_keda_scaledobject_yaml(
        "tgs-worker-swarm",
        "tagisan-system",
        "ingest-telemetry-queue",
        "servicebus-credentials",
        20,
    );

    assert!(keda_yaml.contains("apiVersion: keda.sh/v1alpha1"));
    assert!(keda_yaml.contains("kind: ScaledObject"));
    assert!(keda_yaml.contains("name: tgs-worker-swarm-keda-scaler"));
    assert!(keda_yaml.contains("namespace: tagisan-system"));
    assert!(keda_yaml.contains("type: azure-servicebus"));
    assert!(keda_yaml.contains("queueName: ingest-telemetry-queue"));
    assert!(keda_yaml.contains("messageCount: \"20\""));

    let pod_yaml = engine.generate_hardened_pod_manifest(
        "tgs-worker-swarm",
        "mcr.microsoft.com/tagisan/runtime:2026.2",
        "b04859a1-4433-41e1-91ef-001122334455",
    );

    assert!(pod_yaml.contains("azure.workload.identity/use: \"true\""));
    assert!(pod_yaml.contains("runAsNonRoot: true"));
    assert!(pod_yaml.contains("runAsUser: 10001"));
    assert!(pod_yaml.contains("readOnlyRootFilesystem: true"));
    assert!(pod_yaml.contains("allowPrivilegeEscalation: false"));
    assert!(pod_yaml.contains("drop:\n            - ALL"));
    assert!(pod_yaml.contains("AZURE_CLIENT_ID"));
    assert!(pod_yaml.contains("b04859a1-4433-41e1-91ef-001122334455"));
}

// =========================================================================
// 6. Pure-Rust Dataverse SolutionPackager Tests
// =========================================================================

#[test]
fn test_09_dataverse_solution_packager_lifecycle_and_audit() {
    let engine = DataverseSolutionPackagerEngine::new();

    let solution_xml = r#"<ImportExportXml version="9.2.0.0">
  <SolutionPackage>
    <UniqueName>TagisanAutonomousDefense</UniqueName>
    <LocalizedNames><LocalizedName description="Tagisan Defense Solution" languagecode="1033"/></LocalizedNames>
    <Version>1.0.0.0</Version>
  </SolutionPackage>
</ImportExportXml>"#;

    let customizations_xml = r#"<ImportExportXml>
  <Entities>
    <Entity><Name>tgs_consensus_verdict</Name></Entity>
    <Entity><Name>tgs_blast_radius_audit</Name></Entity>
    <Entity><Name>tgs_agent_telemetry</Name></Entity>
  </Entities>
</ImportExportXml>"#;

    let content_types_xml = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="xml" ContentType="application/xml"/>
</Types>"#;

    let solution = UnpackedSolution {
        solution_xml: solution_xml.to_string(),
        customizations_xml: customizations_xml.to_string(),
        content_types_xml: content_types_xml.to_string(),
        extra_files: HashMap::new(),
    };

    // Pack into in-memory PKZIP archive
    let zip_bytes = engine.pack_solution_zip_in_memory(&solution).expect("Packing solution failed");
    assert!(zip_bytes.starts_with(b"PK\x03\x04"));
    assert!(zip_bytes.len() > 300);

    // Unpack back from bytes
    let unpacked = engine.unpack_solution_zip_in_memory(&zip_bytes).expect("Unpacking solution failed");
    assert!(unpacked.solution_xml.contains("TagisanAutonomousDefense"));
    assert!(unpacked.customizations_xml.contains("tgs_consensus_verdict"));
    assert!(unpacked.content_types_xml.contains("application/xml"));

    // Run audit: all entities start with "tgs_"
    let audit = engine.audit_solution_customizations(&unpacked.customizations_xml, "tgs_").expect("Audit failed");
    assert!(audit.is_valid);
    assert_eq!(audit.total_entities, 3);
    assert_eq!(audit.compliant_entities, 3);
    assert!(audit.breached_entities.is_empty());

    // Adversarial audit: entity with non-compliant prefix
    let corrupt_xml = r#"<ImportExportXml>
  <Entities>
    <Entity><Name>tgs_consensus_verdict</Name></Entity>
    <Entity><Name>bad_rogue_entity</Name></Entity>
  </Entities>
</ImportExportXml>"#;
    let audit_breach = engine.audit_solution_customizations(corrupt_xml, "tgs_").expect("Audit failed");
    assert!(!audit_breach.is_valid);
    assert_eq!(audit_breach.total_entities, 2);
    assert_eq!(audit_breach.compliant_entities, 1);
    assert_eq!(audit_breach.breached_entities, vec!["bad_rogue_entity"]);
}

// =========================================================================
// 7. Windows Desktop Named Pipes & WinUI 3 Deep-Link IPC Tests
// =========================================================================

#[test]
fn test_10_named_pipe_framing_crc32_and_tampering() {
    let engine = WindowsNamedPipeIpcEngine::new();
    assert_eq!(WindowsNamedPipeIpcEngine::pipe_name(), r"\\.\pipe\tagisan_ipc");

    let payload = json!({
        "agent_id": "Agent_Lakandiwa_01",
        "verdict": "APPROVED",
        "confidence_pct": 99.4,
        "blast_radius": 0.32
    });

    let frame = engine.format_ipc_frame("broadcast_consensus", &payload);

    // Verify magic header
    assert!(frame.starts_with(b"TGS_IPC\x00"));

    // Parse back successfully
    let (action, parsed_payload) = engine.parse_ipc_frame(&frame).expect("Parsing IPC frame failed");
    assert_eq!(action, "broadcast_consensus");
    assert_eq!(parsed_payload["verdict"], "APPROVED");
    assert_eq!(parsed_payload["confidence_pct"], 99.4);

    // Tampering test 1: Flip a byte in the payload
    let mut tampered_payload = frame.clone();
    let idx = tampered_payload.len() - 10;
    tampered_payload[idx] ^= 0xFF; // Invert byte
    let tampered_res = engine.parse_ipc_frame(&tampered_payload);
    assert!(tampered_res.is_err());
    assert!(tampered_res.unwrap_err().to_string().contains("CRC32 mismatch"));

    // Tampering test 2: Truncate frame
    let truncated = &frame[..15];
    assert!(engine.parse_ipc_frame(truncated).is_err());

    // Tampering test 3: Corrupt magic bytes
    let mut corrupt_magic = frame.clone();
    corrupt_magic[0] = b'X';
    assert!(engine.parse_ipc_frame(&corrupt_magic).is_err());
}

#[test]
fn test_11_winui_deeplink_uri_parser() {
    let engine = WindowsNamedPipeIpcEngine::new();

    // Valid audit route with parameters
    let target = engine.parse_winui_deeplink("tagisan://audit?symbol=PaymentGateway&path=src/core&depth=4")
        .expect("Deep link parsing failed");
    assert_eq!(target.route, "audit");
    assert_eq!(target.params.get("symbol").unwrap(), "PaymentGateway");
    assert_eq!(target.params.get("path").unwrap(), "src/core");
    assert_eq!(target.params.get("depth").unwrap(), "4");

    // Valid debate route
    let debate_target = engine.parse_winui_deeplink("tagisan://debate?id=DEBATE-2026-X1").unwrap();
    assert_eq!(debate_target.route, "debate");
    assert_eq!(debate_target.params.get("id").unwrap(), "DEBATE-2026-X1");

    // Invalid scheme
    assert!(engine.parse_winui_deeplink("https://tagisan.ai/audit").is_err());
    assert!(engine.parse_winui_deeplink("ftp://server/file").is_err());
}

// =========================================================================
// 8. Unified ToolHandler Trait Dispatch Tests
// =========================================================================

#[tokio::test]
async fn test_12_tool_handler_dispatch_and_edge_case_fuzzing() {
    let tool = CopilotMsEcosystemTool::new();
    assert_eq!(tool.name(), "ms_ecosystem_copilot");
    assert!(tool.description().contains("Microsoft Frontier Cloud"));

    // Action: typespec_synthesize
    let call_tsp_raw = tool.execute(json!({
        "action": "typespec_synthesize",
        "service_name": "SovereignAgentApi",
        "namespace": "Tagisan.Agents"
    })).await.expect("Tool call failed");
    let call_tsp: Value = serde_json::from_str(&call_tsp_raw).unwrap();
    assert!(call_tsp["typespec_code"].as_str().unwrap().contains("namespace Tagisan.Agents"));

    // Action: apim_policy_generate
    let call_apim_raw = tool.execute(json!({
        "action": "apim_policy_generate",
        "tenant_id": "tenant-abc",
        "client_id": "client-xyz",
        "rate_limit": 500
    })).await.expect("Tool call failed");
    let call_apim: Value = serde_json::from_str(&call_apim_raw).unwrap();
    assert!(call_apim["policy_xml"].as_str().unwrap().contains("<validate-jwt"));

    // Action: defender_hunting_query
    let call_def_raw = tool.execute(json!({
        "action": "defender_hunting_query",
        "table": "DeviceNetworkEvents",
        "filter": "RemotePort == 443"
    })).await.expect("Tool call failed");
    let call_def: Value = serde_json::from_str(&call_def_raw).unwrap();
    assert!(call_def["kql_query"].as_str().unwrap().contains("DeviceNetworkEvents"));

    // Action: fabric_medallion_notebook
    let call_fab_raw = tool.execute(json!({
        "action": "fabric_medallion_notebook",
        "workspace_id": "ws-123",
        "lakehouse_name": "SalesLake",
        "table_name": "customers"
    })).await.expect("Tool call failed");
    let call_fab: Value = serde_json::from_str(&call_fab_raw).unwrap();
    assert!(call_fab["optimization_ddl"].as_str().unwrap().contains("OPTIMIZE customers"));

    // Action: keda_manifest_generate
    let call_keda_raw = tool.execute(json!({
        "action": "keda_manifest_generate",
        "app_name": "tgs-ingest-worker",
        "namespace": "production",
        "queue_name": "inbound-events"
    })).await.expect("Tool call failed");
    let call_keda: Value = serde_json::from_str(&call_keda_raw).unwrap();
    assert!(call_keda["scaled_object_yaml"].as_str().unwrap().contains("kind: ScaledObject"));

    // Action: dataverse_solution_audit
    let call_dv_raw = tool.execute(json!({
        "action": "dataverse_solution_audit",
        "publisher_prefix": "tgs_"
    })).await.expect("Tool call failed");
    let call_dv: Value = serde_json::from_str(&call_dv_raw).unwrap();
    assert_eq!(call_dv["audit_report"]["is_valid"], true);

    // Action: ipc_frame_encode_decode
    let call_ipc_raw = tool.execute(json!({
        "action": "ipc_frame_encode_decode",
        "ipc_action": "heartbeat_tick"
    })).await.expect("Tool call failed");
    let call_ipc: Value = serde_json::from_str(&call_ipc_raw).unwrap();
    assert_eq!(call_ipc["decoded_action"], "heartbeat_tick");
    assert!(call_ipc["frame_bytes_length"].as_u64().unwrap() > 20);

    // Negative / Fuzzing: Unknown action
    let call_invalid = tool.execute(json!({
        "action": "nonexistent_action_xyz"
    })).await;
    assert!(call_invalid.is_err());

    // Empty parameters
    let call_empty = tool.execute(json!({})).await;
    assert!(call_empty.is_err());
}

// =========================================================================
// 9. 100-Worker Concurrency Stress Test
// =========================================================================

#[test]
fn test_13_concurrent_100_workers_multi_threaded_stress() {
    let foundry = Arc::new(AzureAiFoundryEngine::new());
    let apim = Arc::new(AzureApimPolicyEngine::new());
    let defender = Arc::new(DefenderXdrHuntingEngine::new());
    let fabric = Arc::new(FabricLakehouseMedallionEngine::new());
    let keda = Arc::new(KedaKubernetesScalerEngine::new());
    let solution_packager = Arc::new(DataverseSolutionPackagerEngine::new());
    let ipc = Arc::new(WindowsNamedPipeIpcEngine::new());

    let mut handles = Vec::new();

    for worker_id in 0..100 {
        let foundry_c = Arc::clone(&foundry);
        let apim_c = Arc::clone(&apim);
        let defender_c = Arc::clone(&defender);
        let fabric_c = Arc::clone(&fabric);
        let keda_c = Arc::clone(&keda);
        let solution_packager_c = Arc::clone(&solution_packager);
        let ipc_c = Arc::clone(&ipc);

        let handle = thread::spawn(move || {
            // Task 1: TypeSpec generation
            let tsp = foundry_c.generate_typespec_definition(
                &format!("Service{}", worker_id),
                &format!("Namespace{}", worker_id),
                &[TypeSpecEndpoint {
                    name: "runTest".to_string(),
                    method: "POST".to_string(),
                    path: "/test".to_string(),
                    doc: "Test endpoint".to_string(),
                    request_body_type: None,
                    response_type: "TestResponse".to_string(),
                }],
            );
            assert!(tsp.contains(&format!("Service{}", worker_id)));

            // Task 2: APIM Policy synthesis
            let policy = apim_c.generate_apim_policy_xml(
                &format!("tenant-{}", worker_id),
                &format!("client-{}", worker_id),
                300,
                &[],
            );
            assert!(policy.contains(&format!("tenant-{}", worker_id)));

            // Task 3: Defender KQL query
            let kql = defender_c.build_advanced_hunting_kql(
                "DeviceProcessEvents",
                &format!("ProcessId == {}", worker_id),
                &["Timestamp", "DeviceName"],
                10,
            );
            assert!(kql.contains(&format!("ProcessId == {}", worker_id)));

            // Task 4: Fabric Medallion Notebook
            let nb = fabric_c.generate_medallion_pyspark_notebook(
                &format!("ws-{}", worker_id),
                "Lakehouse",
                &format!("table_{}", worker_id),
            );
            assert_eq!(nb["cells"].as_array().unwrap().len(), 3);

            // Task 5: KEDA Scaler YAML
            let yaml = keda_c.generate_keda_scaledobject_yaml(
                &format!("worker-{}", worker_id),
                "default",
                "queue-1",
                "secret-1",
                15,
            );
            assert!(yaml.contains(&format!("worker-{}", worker_id)));

            // Task 6: Dataverse Solution Pack & Audit
            let xml = format!("<ImportExportXml><Entities><Entity><Name>tgs_entity_{}</Name></Entity></Entities></ImportExportXml>", worker_id);
            let audit = solution_packager_c.audit_solution_customizations(&xml, "tgs_").unwrap();
            assert!(audit.is_valid);

            // Task 7: Named Pipe IPC binary framing
            let payload = json!({ "worker_id": worker_id, "timestamp": Utc::now().timestamp() });
            let frame = ipc_c.format_ipc_frame(&format!("worker_tick_{}", worker_id), &payload);
            let (parsed_action, parsed_payload) = ipc_c.parse_ipc_frame(&frame).unwrap();
            assert_eq!(parsed_action, format!("worker_tick_{}", worker_id));
            assert_eq!(parsed_payload["worker_id"], worker_id);
        });

        handles.push(handle);
    }

    for (i, h) in handles.into_iter().enumerate() {
        h.join().unwrap_or_else(|e| panic!("Worker {} panicked: {:?}", i, e));
    }
}
