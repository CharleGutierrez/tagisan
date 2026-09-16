//! Microsoft Enterprise Hardening Brutal Test Suite
//!
//! Exhaustive, zero-mock, high-concurrency verification covering:
//! 1. Microsoft Defender XDR & Graph Security API Webhook Ingestion, RFC 2104 HMAC-SHA256 verification, and AST call-graph triage.
//! 2. Automated Remediation PR Synthesizer, git remediation branches, virtual patches, and Microsoft Teams Adaptive Card 1.5 schema.
//! 3. Microsoft Sentinel KQL Analytic Rule Generation across endpoints, processes, and network events.
//! 4. Azure Bicep Function Evaluator (`uniqueString`, `guid`, `resourceId`, `concat`, string interpolation).
//! 5. Bicep-to-ARM JSON Transpiler producing 100% compliant ARM templates.
//! 6. Azure Verified Modules (AVM) Compliance Engine (`AVM-TAG-001`, `AVM-DIAG-001`, `AVM-SEC-001`).
//! 7. Power BI TMSL XMLA Deployment Script Generator (`createOrReplace`, `refresh`, `alter`, XMLA SOAP envelope).
//! 8. Power Apps Component Framework (PCF) Control Package Synthesizer (`ControlManifest.Input.xml`, `index.ts`, Fluent UI React widget).
//! 9. Power Automate Cloud Flow Transpiler (`workflowDefinition.json`).
//! 10. 50-Worker Concurrency Stress Test & Malformed / Corrupted Payload / Injection Resilience.

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tagisan::copilot::bicep::*;
use tagisan::copilot::defender::*;
use tagisan::copilot::powerbi::*;
use tagisan::copilot::powerplatform::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. Defender XDR & Graph Security Webhook Gateway Tests
// =========================================================================

#[test]
fn test_defender_webhook_hmac_sha256_rfc2104_standard_vectors() {
    // RFC 4231 Test Case 2:
    // Key: "Jefe" (4 bytes)
    // Data: "what do ya want for nothing?" (28 bytes)
    // Digest: 5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
    let key = b"Jefe";
    let data = b"what do ya want for nothing?";
    let hmac = compute_hmac_sha256(key, data);
    let expected_hex = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";

    let mut actual_hex = String::new();
    for b in &hmac {
        use std::fmt::Write;
        let _ = write!(actual_hex, "{:02x}", b);
    }
    assert_eq!(actual_hex, expected_hex);

    // Verify constant-time comparison helper
    assert!(constant_time_eq(&hmac, &hmac));
    assert!(!constant_time_eq(&hmac, &[0u8; 32]));

    // Verify clientState HMAC verification
    assert!(verify_client_state_hmac(key, data, expected_hex));
    assert!(verify_client_state_hmac(key, data, &format!("hmac-sha256={expected_hex}")));
    assert!(verify_client_state_hmac(key, data, &expected_hex.to_uppercase()));
}

#[test]
fn test_defender_webhook_hmac_invalid_secret_rejected() {
    let secret = b"my-enterprise-secret-key-12345";
    let payload = b"{\"event\": \"security_alert_created\"}";

    // Valid state
    let valid_hmac = compute_hmac_sha256(secret, payload);
    let mut valid_hex = String::new();
    for b in &valid_hmac {
        use std::fmt::Write;
        let _ = write!(valid_hex, "{:02x}", b);
    }
    assert!(verify_client_state_hmac(secret, payload, &valid_hex));

    // Tampered payload
    let tampered_payload = b"{\"event\": \"security_alert_tampered\"}";
    assert!(!verify_client_state_hmac(secret, tampered_payload, &valid_hex));

    // Tampered secret
    let wrong_secret = b"wrong-enterprise-secret-key";
    assert!(!verify_client_state_hmac(wrong_secret, payload, &valid_hex));

    // Tampered client state token
    let bad_token = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec0000";
    assert!(!verify_client_state_hmac(secret, payload, bad_token));
}

#[test]
fn test_defender_webhook_gateway_ingest_graph_v2_notification() {
    let engine = Arc::new(DefenderEngine::new());
    let secret = b"super-secret-graph-key".to_vec();
    let gateway = DefenderWebhookGateway::new(Arc::clone(&engine), secret.clone());

    let subscription_id = "sub-azure-defender-001";
    let hmac = compute_hmac_sha256(&secret, subscription_id.as_bytes());
    let mut client_state = String::new();
    for b in &hmac {
        use std::fmt::Write;
        let _ = write!(client_state, "{:02x}", b);
    }

    let graph_webhook_payload = json!({
        "value": [
            {
                "subscriptionId": subscription_id,
                "clientState": client_state,
                "changeType": "created",
                "resource": "security/alerts_v2/alert_99812",
                "resourceData": {
                    "@odata.type": "#microsoft.graph.security.alert",
                    "id": "alert_99812",
                    "title": "Vulnerability CVE-2024-38077 detected in rpc_dispatch",
                    "severity": "high",
                    "package_name": "tagisan_rpc",
                    "package_version": "1.0.4",
                    "vulnerable_symbol": "rpc_dispatch",
                    "cvss_score": 9.8
                }
            }
        ]
    })
    .to_string();

    let source_code = r#"
        pub fn main() {
            api_gateway();
        }
        pub fn api_gateway() {
            rpc_dispatch("payload");
        }
    "#;

    let results = gateway
        .ingest_webhook(&graph_webhook_payload, Some(source_code), Some("src/rpc.rs"))
        .expect("Webhook ingestion failed");

    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert_eq!(res.alert_id, "alert_99812");
    assert_eq!(res.cve_id, "CVE-2024-38077");
    assert!(res.client_state_verified);
    assert_eq!(res.triage_finding.reachability, ReachabilityStatus::DirectlyReachable);
    assert_eq!(res.triage_finding.defended_risk_score, 9.8);
    assert!(res.auto_remediation_recommended);
    assert!(res.sentinel_rule_generated);
}

#[test]
fn test_defender_webhook_gateway_dormant_unreachable_alert() {
    let engine = Arc::new(DefenderEngine::new());
    let secret = b"secret-key".to_vec();
    let gateway = DefenderWebhookGateway::new(Arc::clone(&engine), secret);

    let raw_payload = json!({
        "alert_id": "DEF-DORMANT-001",
        "cve_id": "CVE-2023-44487",
        "package_name": "hyper",
        "package_version": "0.14.20",
        "vulnerable_symbol": "ancient_dead_function",
        "cvss_score": 7.5,
        "clientState": "secret-key"
    })
    .to_string();

    let source_code = "pub fn main() { println!(\"active application code\"); }";

    let results = gateway
        .ingest_webhook(&raw_payload, Some(source_code), None)
        .expect("Ingestion failed");

    assert_eq!(results.len(), 1);
    let finding = &results[0].triage_finding;
    assert_eq!(finding.reachability, ReachabilityStatus::DormantUnreachable);
    assert!(finding.defended_risk_score <= 1.0);
    assert!(finding.defender_action.cvss_reduction_percent >= 85.0);
    assert_eq!(finding.defender_action.recommended_status, "Suppressed");
    assert_eq!(finding.defender_action.graph_security_tag, "TAGISAN_AST_VERIFIED_UNREACHABLE");
}

#[test]
fn test_defender_webhook_gateway_sanitized_invariant_alert() {
    let engine = Arc::new(DefenderEngine::new());
    let secret = b"secret-key".to_vec();
    let gateway = DefenderWebhookGateway::new(Arc::clone(&engine), secret);

    let raw_payload = json!({
        "alert_id": "DEF-SANITIZED-001",
        "cve_id": "CVE-2024-21626",
        "vulnerable_symbol": "runc_exec",
        "cvss_score": 8.6,
        "clientState": "secret-key"
    })
    .to_string();

    let source_code = r#"
        pub fn handler() {
            let verdict = AgentShieldScanner::scan_prompt_text(payload);
            runc_exec(payload);
        }
    "#;

    let results = gateway
        .ingest_webhook(&raw_payload, Some(source_code), None)
        .expect("Ingestion failed");

    let finding = &results[0].triage_finding;
    assert_eq!(finding.reachability, ReachabilityStatus::Sanitized);
    assert!(finding.defended_risk_score <= 3.0);
    assert!(finding.defender_action.cvss_reduction_percent >= 70.0);
    assert_eq!(finding.defender_action.graph_security_tag, "TAGISAN_SANITIZER_ACTIVE");
}

// =========================================================================
// 2. Automated Remediation PR Generator & Teams Adaptive Card Tests
// =========================================================================

#[test]
fn test_automated_remediation_pr_generator_git_branch_and_commit() {
    let alert = DefenderCveAlert {
        alert_id: "ALT-2024-001".to_string(),
        cve_id: "CVE-2024-38077".to_string(),
        package_name: "win32_rpc".to_string(),
        package_version: "1.2.0".to_string(),
        vulnerable_symbol: "rpc_dispatch".to_string(),
        cvss_score: 9.8,
        severity: "Critical".to_string(),
        description: "Windows Remote Procedure Call RCE vulnerability".to_string(),
        published_date: Some("2024-08-13".to_string()),
    };

    let engine = DefenderEngine::new();
    let finding = engine.triage_alert(
        &alert,
        Some("pub fn handler() { rpc_dispatch(input); }"),
        Some("src/rpc_service.rs"),
    );

    let pr = AutomatedRemediationPrGenerator::generate_remediation_pr(
        &finding,
        Some("src/rpc_service.rs"),
        Some("main"),
    );

    // Branch naming convention: security/patch-<cve_lower>
    assert_eq!(pr.branch_name, "security/patch-cve-2024-38077");
    assert_eq!(pr.target_branch, "main");
    assert!(pr.title.contains("CVE-2024-38077"));
    assert!(pr.commit_message.starts_with("fix(security): [AgentShield]"));
    assert_eq!(pr.changed_files, vec!["src/rpc_service.rs"]);

    // Virtual Patch diff verification
    assert!(pr.virtual_patch.patch_diff.contains("--- a/src/rpc_service.rs"));
    assert!(pr.virtual_patch.patch_diff.contains("+++ b/src/rpc_service.rs"));
    assert!(pr.virtual_patch.patch_diff.contains("AgentShieldScanner"));

    // PR description verification
    assert!(pr.description.contains("Tagisan Autonomous Security Remediation PR"));
    assert!(pr.description.contains("CVE-2024-38077"));
    assert!(pr.description.contains("Risk Reduction Telemetry"));

    // PR JSON payload verification
    assert_eq!(pr.pr_payload_json["head"], "security/patch-cve-2024-38077");
    assert_eq!(pr.pr_payload_json["base"], "main");
    let labels = pr.pr_payload_json["labels"].as_array().unwrap();
    assert!(labels.iter().any(|l| l.as_str() == Some("agentshield")));
}

#[test]
fn test_automated_remediation_pr_teams_adaptive_card_1_5_schema() {
    let alert = DefenderCveAlert {
        alert_id: "ALT-TEAMS-01".to_string(),
        cve_id: "CVE-2024-43451".to_string(),
        package_name: "ms_auth".to_string(),
        package_version: "2.1.0".to_string(),
        vulnerable_symbol: "ntlm_hash_exchange".to_string(),
        cvss_score: 8.8,
        severity: "High".to_string(),
        description: "NTLM hash disclosure spoofing vulnerability".to_string(),
        published_date: None,
    };

    let engine = DefenderEngine::new();
    let finding = engine.triage_alert(&alert, None, None);

    let pr = AutomatedRemediationPrGenerator::generate_remediation_pr(&finding, None, None);
    let card = &pr.teams_adaptive_card;

    assert_eq!(card["$schema"], "http://adaptivecards.io/schemas/adaptive-card.json");
    assert_eq!(card["type"], "AdaptiveCard");
    assert_eq!(card["version"], "1.5");

    let body = card["body"].as_array().expect("Body should be an array");
    assert!(body.len() >= 3);

    // Verify FactSet exists with CVE and scores
    let fact_set = body
        .iter()
        .find(|item| item["type"] == "FactSet")
        .expect("Card missing FactSet element");

    let facts = fact_set["facts"].as_array().expect("Facts should be an array");
    let cve_fact = facts.iter().find(|f| f["title"] == "CVE Identifier").unwrap();
    assert_eq!(cve_fact["value"], "CVE-2024-43451");

    // Verify OpenUrl Actions
    let actions = card["actions"].as_array().expect("Actions should be an array");
    assert!(actions.len() >= 2);
    assert_eq!(actions[0]["type"], "Action.OpenUrl");
    assert!(actions[0]["url"].as_str().unwrap().contains("github.com"));
}

// =========================================================================
// 3. Microsoft Sentinel KQL Analytic Rule Generator Tests
// =========================================================================

#[test]
fn test_sentinel_kql_analytic_rule_multi_table_query() {
    let symbol = "unsafe_buffer_unpack";
    let cve_id = "CVE-2024-38077";
    let rule = SentinelKqlRuleGenerator::generate_rule(symbol, cve_id, "High");

    assert_eq!(rule.rule_id, "TAGISAN-SENTINEL-CVE_2024_38077");
    assert_eq!(rule.severity, "High");
    assert!(rule.display_name.contains(symbol));
    assert!(rule.display_name.contains(cve_id));

    // KQL query verification
    assert!(rule.query.contains("DeviceProcessEvents"));
    assert!(rule.query.contains("DeviceNetworkEvents"));
    assert!(rule.query.contains("let monitored_symbol = \"unsafe_buffer_unpack\";"));
    assert!(rule.query.contains("let monitored_cve = \"CVE-2024-38077\";"));
    assert!(rule.query.contains("union isfuzzy=true process_events, network_events"));
    assert!(rule.query.contains("where TriggerCount > 0"));

    // ARM deployment template verification
    assert_eq!(
        rule.arm_template["$schema"],
        "https://schema.management.azure.com/schemas/2019-04-01/deploymentTemplate.json#"
    );
    let resources = rule.arm_template["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0]["properties"]["severity"], "High");
    assert_eq!(resources[0]["properties"]["triggerOperator"], "GreaterThan");
}

// =========================================================================
// 4. Azure Bicep Function Evaluator Tests
// =========================================================================

#[test]
fn test_bicep_function_evaluator_resource_id_signatures() {
    let ctx = BicepEvaluationContext {
        subscription_id: "11111111-2222-3333-4444-555555555555".to_string(),
        resource_group: "rg-prod-eastus".to_string(),
        parameters: HashMap::new(),
        variables: HashMap::new(),
    };

    // 1. resourceId(type, name)
    let res1 = BicepFunctionEvaluator::resource_id(
        &["Microsoft.Storage/storageAccounts", "stgprod2026"],
        &ctx,
    )
    .unwrap();
    assert_eq!(
        res1,
        "/subscriptions/11111111-2222-3333-4444-555555555555/resourceGroups/rg-prod-eastus/providers/Microsoft.Storage/storageAccounts/stgprod2026"
    );

    // 2. resourceId(rg, type, name)
    let res2 = BicepFunctionEvaluator::resource_id(
        &["rg-shared-core", "Microsoft.KeyVault/vaults", "kv-shared-core"],
        &ctx,
    )
    .unwrap();
    assert_eq!(
        res2,
        "/subscriptions/11111111-2222-3333-4444-555555555555/resourceGroups/rg-shared-core/providers/Microsoft.KeyVault/vaults/kv-shared-core"
    );

    // 3. resourceId(sub, rg, type, name)
    let res3 = BicepFunctionEvaluator::resource_id(
        &[
            "99999999-8888-7777-6666-555555555555",
            "rg-hub-network",
            "Microsoft.Network/virtualNetworks",
            "vnet-hub",
        ],
        &ctx,
    )
    .unwrap();
    assert_eq!(
        res3,
        "/subscriptions/99999999-8888-7777-6666-555555555555/resourceGroups/rg-hub-network/providers/Microsoft.Network/virtualNetworks/vnet-hub"
    );

    // 4. Sub-resource: resourceId(type, parent, child)
    let res4 = BicepFunctionEvaluator::resource_id(
        &["Microsoft.Network/virtualNetworks/subnets", "vnet-hub", "snet-private-endpoints"],
        &ctx,
    )
    .unwrap();
    assert_eq!(
        res4,
        "/subscriptions/11111111-2222-3333-4444-555555555555/resourceGroups/rg-prod-eastus/providers/Microsoft.Network/virtualNetworks/vnet-hub/subnets/snet-private-endpoints"
    );
}

#[test]
fn test_bicep_function_evaluator_unique_string_determinism_and_length() {
    let args1 = ["sub-12345", "rg-dev", "deploy-01"];
    let u1 = BicepFunctionEvaluator::unique_string(&args1);
    let u2 = BicepFunctionEvaluator::unique_string(&args1);

    // Deterministic
    assert_eq!(u1, u2);
    // Exactly 13 lowercase alphanumeric characters
    assert_eq!(u1.len(), 13);
    assert!(u1.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));

    // Different inputs yield different outputs
    let args2 = ["sub-12345", "rg-prod", "deploy-01"];
    let u3 = BicepFunctionEvaluator::unique_string(&args2);
    assert_ne!(u1, u3);
    assert_eq!(u3.len(), 13);
}

#[test]
fn test_bicep_function_evaluator_guid_determinism_and_uuid_format() {
    let args = ["tenant-id-01", "app-registration-tagisan", "role-contributor"];
    let g1 = BicepFunctionEvaluator::guid(&args);
    let g2 = BicepFunctionEvaluator::guid(&args);

    assert_eq!(g1, g2);
    assert_eq!(g1.len(), 36);

    // Format: 8-4-4-4-12
    let parts: Vec<&str> = g1.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 4);
    assert_eq!(parts[2].len(), 4);
    assert_eq!(parts[3].len(), 4);
    assert_eq!(parts[4].len(), 12);
}

#[test]
fn test_bicep_function_evaluator_concat_and_string_interpolation() {
    let evaluator = BicepFunctionEvaluator::new();
    let ctx = BicepEvaluationContext::default();

    // 1. concat
    let c = BicepFunctionEvaluator::concat(&["stg", "tagisan", "prod"]);
    assert_eq!(c, "stgtagisanprod");

    // 2. String interpolation with uniqueString
    let expr = "'stg${uniqueString('rg-tagisan')}'";
    let evaluated = evaluator.evaluate_expression(expr, &ctx).unwrap();
    let s = evaluated.as_str().unwrap();
    assert!(s.starts_with("stg"));
    assert_eq!(s.len(), 3 + 13);
}

// =========================================================================
// 5. Bicep to ARM JSON Transpiler Tests
// =========================================================================

#[test]
fn test_bicep_to_arm_transpiler_complete_template() {
    let transpiler = BicepToArmTranspiler::new();
    let ctx = BicepEvaluationContext::default();

    let bicep_code = r#"
        @description('The primary location for resources')
        param location string = 'eastus2'

        param environment string = 'Production'

        var storageAccountName = 'stg${uniqueString('rg-tagisan')}'

        resource storage 'Microsoft.Storage/storageAccounts@2023-01-01' = {
          name: storageAccountName
          location: location
          properties: {
            supportsHttpsTrafficOnly: true
            publicNetworkAccess: 'Disabled'
          }
        }

        output storageAccountId string = storage.id
    "#;

    let arm_json = transpiler.transpile(bicep_code, &ctx).expect("Transpilation failed");

    assert_eq!(
        arm_json["$schema"],
        "https://schema.management.azure.com/schemas/2019-04-01/deploymentTemplate.json#"
    );
    assert_eq!(arm_json["contentVersion"], "1.0.0.0");

    // Parameters
    let params = arm_json["parameters"].as_object().unwrap();
    assert!(params.contains_key("location"));
    assert_eq!(params["location"]["defaultValue"], "eastus2");
    assert_eq!(
        params["location"]["metadata"]["description"],
        "The primary location for resources"
    );

    // Variables
    let vars = arm_json["variables"].as_object().unwrap();
    assert!(vars.contains_key("storageAccountName"));
    let stg_var = vars["storageAccountName"].as_str().unwrap();
    assert!(stg_var.starts_with("stg"));

    // Resources
    let resources = arm_json["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 1);
    let r = &resources[0];
    assert_eq!(r["type"], "Microsoft.Storage/storageAccounts");
    assert_eq!(r["apiVersion"], "2023-01-01");
    assert_eq!(r["properties"]["supportsHttpsTrafficOnly"], "true");
    assert_eq!(r["properties"]["publicNetworkAccess"], "Disabled");

    // Outputs
    let outputs = arm_json["outputs"].as_object().unwrap();
    assert!(outputs.contains_key("storageAccountId"));
}

// =========================================================================
// 6. Azure Verified Modules (AVM) Compliance Tests
// =========================================================================

#[test]
fn test_avm_compliance_checker_clean_module_certified() {
    let checker = AvmComplianceChecker::new();
    let engine = BicepEngine::new();

    let clean_bicep = r#"
        resource kv 'Microsoft.KeyVault/vaults@2023-02-01' = {
          name: 'kv-avm-compliant'
          location: 'eastus'
          tags: {
            Environment: 'Production'
            Owner: 'TagisanArchitect'
            WorkloadName: 'TagisanCore'
          }
          identity: {
            type: 'SystemAssigned'
          }
          properties: {
            publicNetworkAccess: 'Disabled'
            diagnosticSettings: true
          }
        }
    "#;

    let resources = engine.parse_bicep(clean_bicep).unwrap();
    let report = checker.check_avm_compliance(&resources);

    assert_eq!(report.failed_rules, 0);
    assert_eq!(report.compliance_percentage, 100.0);
    assert!(report.is_avm_certified);
    assert!(report.summary.contains("CERTIFIED AVM COMPLIANT"));
}

#[test]
fn test_avm_compliance_checker_violations_and_virtual_patches() {
    let checker = AvmComplianceChecker::new();
    let engine = BicepEngine::new();

    // Missing tags, public network access enabled, missing managed identity, missing diagnostic settings
    let non_compliant_bicep = r#"
        resource stg 'Microsoft.Storage/storageAccounts@2023-01-01' = {
          name: 'stgnoncompliant'
          location: 'eastus'
          properties: {
            publicNetworkAccess: 'Enabled'
          }
        }
    "#;

    let resources = engine.parse_bicep(non_compliant_bicep).unwrap();
    let report = checker.check_avm_compliance(&resources);

    assert!(!report.is_avm_certified);
    assert!(report.failed_rules >= 3);
    assert!(report.compliance_percentage < 50.0);

    // Check individual rule violations
    let tag_violation = report.violations.iter().find(|v| v.rule_id == "AVM-TAG-001");
    assert!(tag_violation.is_some());
    assert!(tag_violation.unwrap().virtual_patch.is_some());

    let diag_violation = report.violations.iter().find(|v| v.rule_id == "AVM-DIAG-001");
    assert!(diag_violation.is_some());

    let sec_violation = report.violations.iter().find(|v| v.rule_id == "AVM-SEC-001");
    assert!(sec_violation.is_some());
    assert_eq!(sec_violation.unwrap().severity, "Critical");
}

// =========================================================================
// 7. Power BI TMSL XMLA Deployment Payload Tests
// =========================================================================

#[test]
fn test_powerbi_tmsl_create_or_replace_full_payload() {
    let tmsl_engine = TmslEngine::new();
    let db = PowerBiEngine::create_default_telemetry_model();

    let tmsl = tmsl_engine.generate_create_or_replace(&db);

    let create_or_replace = &tmsl["createOrReplace"];
    assert_eq!(create_or_replace["object"]["database"], "TagisanTelemetryDb");

    let database = &create_or_replace["database"];
    assert_eq!(database["name"], "TagisanTelemetryDb");
    assert_eq!(database["compatibilityLevel"], 1600);

    let model = &database["model"];
    assert_eq!(model["culture"], "en-US");

    let tables = model["tables"].as_array().expect("Tables should be an array");
    assert_eq!(tables.len(), 5);

    let commits_table = tables.iter().find(|t| t["name"] == "GitCommits").unwrap();
    let cols = commits_table["columns"].as_array().unwrap();
    assert!(cols.iter().any(|c| c["name"] == "CommitSha"));

    let measures_table = tables.iter().find(|t| t["name"] == "TelemetryMeasures").unwrap();
    let measures = measures_table["measures"].as_array().unwrap();
    assert!(measures.iter().any(|m| m["name"] == "Blast Radius Index"));
}

#[test]
fn test_powerbi_tmsl_refresh_and_alter_operations() {
    let tmsl_engine = TmslEngine::new();

    // 1. Full database refresh
    let refresh_db = tmsl_engine.generate_refresh("TagisanTelemetryDb", TmslRefreshType::Full, &[]);
    assert_eq!(refresh_db["refresh"]["type"], "full");
    assert_eq!(refresh_db["refresh"]["objects"][0]["database"], "TagisanTelemetryDb");

    // 2. Targeted table refresh
    let targets = vec![
        TmslTargetObject {
            database: "TagisanTelemetryDb".to_string(),
            table: Some("GitCommits".to_string()),
            partition: None,
        },
        TmslTargetObject {
            database: "TagisanTelemetryDb".to_string(),
            table: Some("SymbolChanges".to_string()),
            partition: Some("Partition2026".to_string()),
        },
    ];
    let refresh_tables = tmsl_engine.generate_refresh("TagisanTelemetryDb", TmslRefreshType::DataOnly, &targets);
    assert_eq!(refresh_tables["refresh"]["type"], "dataOnly");
    assert_eq!(refresh_tables["refresh"]["objects"].as_array().unwrap().len(), 2);

    // 3. Alter Table
    let table = TmdlTable {
        name: "SecurityAlerts".to_string(),
        lineage_tag: "alert-tag-001".to_string(),
        description: Some("Defender alerts table".to_string()),
        columns: vec![TmdlColumn {
            name: "AlertId".to_string(),
            data_type: "string".to_string(),
            format_string: None,
            summarize_by: None,
            source_column: "AlertId".to_string(),
            description: None,
        }],
        measures: vec![],
        partitions: vec![],
    };
    let alter = tmsl_engine.generate_alter_table("TagisanTelemetryDb", &table);
    assert_eq!(alter["alter"]["object"]["table"], "SecurityAlerts");
    assert_eq!(alter["alter"]["table"]["name"], "SecurityAlerts");
}

#[test]
fn test_powerbi_xmla_soap_envelope_and_connection_string() {
    let tmsl_engine = TmslEngine::new();
    let db = PowerBiEngine::create_default_telemetry_model();
    let tmsl_cmd = tmsl_engine.generate_create_or_replace(&db);

    let pkg = tmsl_engine.generate_xmla_envelope(
        "TagisanEnterpriseWorkspace",
        "TagisanTelemetryDb",
        &tmsl_cmd,
        "createOrReplace",
    );

    assert_eq!(pkg.endpoint_url, "powerbi://api.powerbi.com/v1.0/myorg/TagisanEnterpriseWorkspace");
    assert_eq!(pkg.database_name, "TagisanTelemetryDb");
    assert_eq!(pkg.command_type, "createOrReplace");
    assert!(pkg.soap_envelope_xml.contains("<Envelope xmlns=\"http://schemas.xmlsoap.org/soap/envelope/\">"));
    assert!(pkg.soap_envelope_xml.contains("<Execute xmlns=\"urn:schemas-microsoft-com:xml-analysis\">"));
    assert!(pkg.soap_envelope_xml.contains("<Catalog>TagisanTelemetryDb</Catalog>"));
    assert!(pkg.soap_envelope_xml.contains("createOrReplace"));
}

// =========================================================================
// 8. Power Platform PCF Control & Cloud Flow Tests
// =========================================================================

#[test]
fn test_powerplatform_pcf_control_package_complete_generation() {
    let generator = PcfControlGenerator::new();
    let config = PcfControlConfig {
        namespace: "Tagisan.Enterprise".to_string(),
        constructor_name: "AstBlastRadiusVisualizer".to_string(),
        version: "2.0.0".to_string(),
        display_name: "Tagisan AST Blast Radius & Dialectical Consensus".to_string(),
        description: "Fluent UI React control visualizing blast radius and multi-agent consensus.".to_string(),
        external_domain: "api.tagisan.ai".to_string(),
    };

    let pcf = generator.generate_pcf_package(&config);

    assert_eq!(pcf.control_name, "AstBlastRadiusVisualizer");
    assert_eq!(pcf.namespace, "Tagisan.Enterprise");
    assert_eq!(pcf.version, "2.0.0");
    assert_eq!(pcf.file_count, 7);

    // Manifest verification
    assert!(pcf.manifest_xml.contains("<control namespace=\"Tagisan.Enterprise\" constructor=\"AstBlastRadiusVisualizer\""));
    assert!(pcf.manifest_xml.contains("<domain>api.tagisan.ai</domain>"));
    assert!(pcf.manifest_xml.contains("<property name=\"targetSymbol\""));
    assert!(pcf.manifest_xml.contains("<property name=\"blastRadiusScore\""));
    assert!(pcf.manifest_xml.contains("<property name=\"consensusVerdict\""));
    assert!(pcf.manifest_xml.contains("<uses-feature name=\"WebAPI\" required=\"true\" />"));

    // TypeScript lifecycle verification
    assert!(pcf.index_ts.contains("export class AstBlastRadiusVisualizerControl implements ComponentFramework.StandardControl<IInputs, IOutputs>"));
    assert!(pcf.index_ts.contains("public init("));
    assert!(pcf.index_ts.contains("public updateView("));
    assert!(pcf.index_ts.contains("public getOutputs(): IOutputs"));
    assert!(pcf.index_ts.contains("public destroy(): void"));

    // Fluent UI React widget verification
    assert!(pcf.widget_tsx.contains("export const AstBlastRadiusVisualizer: React.FC<ITagisanWidgetProps>"));
    assert!(pcf.widget_tsx.contains("Tagisan AST Exploitability & Consensus"));
    assert!(pcf.widget_tsx.contains("ProgressIndicator"));
    assert!(pcf.widget_tsx.contains("Badge"));

    // Project files verification
    assert!(pcf.package_json.contains("\"name\": \"astblastradiusvisualizer\""));
    assert!(pcf.tsconfig_json.contains("\"compilerOptions\""));
    assert!(pcf.resx_xml.contains("AstBlastRadiusVisualizer_Display_Key"));
}

#[test]
fn test_power_automate_flow_transpiler_workflow_definition() {
    let transpiler = PowerAutomateTranspiler::new();
    let config = PowerAutomateFlowConfig {
        flow_name: "Auto_Approve_Tagisan_ADR_Flow".to_string(),
        dataverse_entity_name: "tgs_architecturaldecisions".to_string(),
        tagisan_api_url: "https://api.tagisan.ai/api/v1/copilot/debate".to_string(),
        consensus_threshold: 0.90,
        teams_channel_name: "Production Architecture Review".to_string(),
    };

    let flow = transpiler.transpile_flow(&config);

    assert_eq!(flow.flow_name, "Auto_Approve_Tagisan_ADR_Flow");
    assert_eq!(flow.triggers_count, 1);
    assert_eq!(flow.actions_count, 4);

    let def = &flow.workflow_definition;
    assert_eq!(def["$schema"], flow.schema_url);
    assert_eq!(def["contentVersion"], "1.0.0.0");

    // Trigger verification
    let trigger = &def["triggers"]["When_a_Dataverse_row_is_added_or_modified"];
    assert_eq!(trigger["inputs"]["parameters"]["subscriptionRequest/entityname"], "tgs_architecturaldecisions");

    // Actions verification
    let debate_call = &def["actions"]["Call_Tagisan_Debate_API"];
    assert_eq!(debate_call["inputs"]["uri"], "https://api.tagisan.ai/api/v1/copilot/debate");

    let condition = &def["actions"]["Condition_Check_Consensus_Score"];
    assert_eq!(condition["type"], "If");
    assert!(condition["actions"]["Update_Dataverse_Record_Approved"].is_object());
    assert!(condition["else"]["actions"]["Post_Adaptive_Card_To_Teams_Channel"].is_object());
}

// =========================================================================
// 9. ToolHandler Integration & Execution Tests
// =========================================================================

#[tokio::test]
async fn test_copilot_defender_tool_actions() {
    let tool = CopilotDefenderTool::new();

    // 1. Action: calculate_risk_score
    let res = tool
        .execute(json!({
            "action": "calculate_risk_score",
            "cvss_score": 9.8,
            "reachability": "Sanitized"
        }))
        .await
        .expect("calculate_risk_score failed");
    let score_json: serde_json::Value = serde_json::from_str(&res).unwrap();
    assert!(score_json["defended_risk_score"].as_f64().unwrap() <= 3.0);

    // 2. Action: generate_sentinel_kql
    let kql_res = tool
        .execute(json!({
            "action": "generate_sentinel_kql",
            "vulnerable_symbol": "rpc_buffer_read",
            "cve_id": "CVE-2024-38077"
        }))
        .await
        .expect("generate_sentinel_kql failed");
    assert!(kql_res.contains("DeviceProcessEvents"));

    // 3. Action: generate_remediation_pr
    let pr_res = tool
        .execute(json!({
            "action": "generate_remediation_pr",
            "alert": {
                "alert_id": "ALT-001",
                "cve_id": "CVE-2024-38077",
                "package_name": "tagisan_rpc",
                "package_version": "1.0.0",
                "vulnerable_symbol": "rpc_dispatch",
                "cvss_score": 9.8,
                "severity": "Critical",
                "description": "Critical RPC RCE"
            }
        }))
        .await
        .expect("generate_remediation_pr failed");
    assert!(pr_res.contains("security/patch-cve-2024-38077"));
}

#[tokio::test]
async fn test_copilot_bicep_tool_actions() {
    let tool = CopilotBicepTool::new();

    // 1. Action: evaluate_function uniqueString
    let res = tool
        .execute(json!({
            "action": "evaluate_function",
            "function_name": "uniqueString",
            "function_args": ["sub-1", "rg-1"]
        }))
        .await
        .expect("evaluate_function failed");
    let val: serde_json::Value = serde_json::from_str(&res).unwrap();
    assert_eq!(val["result"].as_str().unwrap().len(), 13);

    // 2. Action: transpile_to_arm
    let bicep_code = "param loc string = 'eastus'\nresource stg 'Microsoft.Storage/storageAccounts@2023-01-01' = { name: 'stg1' location: loc }";
    let trans_res = tool
        .execute(json!({
            "action": "transpile_to_arm",
            "bicep_code": bicep_code
        }))
        .await
        .expect("transpile_to_arm failed");
    assert!(trans_res.contains("deploymentTemplate.json"));
}

#[tokio::test]
async fn test_copilot_powerbi_tool_actions() {
    let tool = CopilotPowerBiTool::new();

    let res = tool
        .execute(json!({
            "action": "generate_tmsl",
            "tmsl_type": "createOrReplace"
        }))
        .await
        .expect("generate_tmsl failed");
    assert!(res.contains("createOrReplace"));
    assert!(res.contains("TagisanTelemetryDb"));
}

#[tokio::test]
async fn test_copilot_powerplatform_tool_actions() {
    let tool = CopilotPowerPlatformPackagerTool::new();

    // 1. generate_pcf_control
    let pcf_res = tool
        .execute(json!({
            "action": "generate_pcf_control"
        }))
        .await
        .expect("generate_pcf_control failed");
    assert!(pcf_res.contains("manifest_xml"));
    assert!(pcf_res.contains("<manifest"));

    // 2. generate_power_automate_flow
    let flow_res = tool
        .execute(json!({
            "action": "generate_power_automate_flow"
        }))
        .await
        .expect("generate_power_automate_flow failed");
    assert!(flow_res.to_lowercase().contains("workflowdefinition.json"));
}

// =========================================================================
// 10. Concurrency Stress (50 Workers) & Malformed Payload Resilience Tests
// =========================================================================

#[tokio::test]
async fn test_50_worker_concurrency_stress_test() {
    let defender_engine = Arc::new(DefenderEngine::new());
    let bicep_engine = Arc::new(BicepEngine::new());
    let tmsl_engine = Arc::new(TmslEngine::new());
    let pcf_generator = Arc::new(PcfControlGenerator::new());

    let mut handles = Vec::new();

    for worker_id in 0..50 {
        let d_engine = Arc::clone(&defender_engine);
        let b_engine = Arc::clone(&bicep_engine);
        let t_engine = Arc::clone(&tmsl_engine);
        let p_gen = Arc::clone(&pcf_generator);

        handles.push(tokio::spawn(async move {
            // 1. Defender triage
            let alert = DefenderCveAlert {
                alert_id: format!("STRESS-ALT-{worker_id}"),
                cve_id: format!("CVE-2024-{:04}", 3000 + worker_id),
                package_name: "stress_pkg".to_string(),
                package_version: "1.0.0".to_string(),
                vulnerable_symbol: format!("worker_symbol_{worker_id}"),
                cvss_score: 9.0,
                severity: "Critical".to_string(),
                description: "Concurrency stress test alert".to_string(),
                published_date: None,
            };
            let finding = d_engine.triage_alert(&alert, None, None);
            assert_eq!(finding.reachability, ReachabilityStatus::DormantUnreachable);

            // 2. Bicep evaluation & transpilation
            let u_str = BicepFunctionEvaluator::unique_string(&[&format!("worker_{worker_id}")]);
            assert_eq!(u_str.len(), 13);
            let bicep_code = format!(
                "param loc string = 'eastus'\nresource res{worker_id} 'Microsoft.Storage/storageAccounts@2023-01-01' = {{ name: 'stg{u_str}' location: loc }}"
            );
            let arm = b_engine.transpiler().transpile(&bicep_code, &BicepEvaluationContext::default()).unwrap();
            assert!(arm.is_object());

            // 3. Power BI TMSL generation
            let db = PowerBiEngine::create_default_telemetry_model();
            let tmsl = t_engine.generate_create_or_replace(&db);
            assert!(tmsl["createOrReplace"].is_object());

            // 4. Power Platform PCF package synthesis
            let pcf_config = PcfControlConfig {
                constructor_name: format!("WidgetWorker{worker_id}"),
                ..Default::default()
            };
            let pcf = p_gen.generate_pcf_package(&pcf_config);
            assert_eq!(pcf.file_count, 7);
        }));
    }

    for handle in handles {
        handle.await.expect("Concurrency worker task panicked");
    }
}

#[tokio::test]
async fn test_malformed_json_corrupted_payload_and_injection_resilience() {
    let defender_tool = CopilotDefenderTool::new();
    let bicep_tool = CopilotBicepTool::new();
    let powerbi_tool = CopilotPowerBiTool::new();
    let powerplatform_tool = CopilotPowerPlatformPackagerTool::new();

    // 1. Missing action
    assert!(defender_tool.execute(json!({})).await.is_err());
    assert!(bicep_tool.execute(json!({})).await.is_err());
    assert!(powerbi_tool.execute(json!({})).await.is_err());
    assert!(powerplatform_tool.execute(json!({})).await.is_err());

    // 2. Malformed JSON payload string in webhook ingestion
    let malformed_webhook = "{ \"value\": [ { unclosed json payload... ";
    let res = defender_tool
        .execute(json!({
            "action": "ingest_webhook",
            "webhook_payload": malformed_webhook
        }))
        .await;
    assert!(res.is_err());

    // 3. SQL injection strings in Bicep code and KQL rule generation
    let sql_injection_symbol = "'; DROP TABLE Users; --";
    let kql_rule = SentinelKqlRuleGenerator::generate_rule(sql_injection_symbol, "CVE-2024-SQLI", "High");
    assert!(kql_rule.query.contains(sql_injection_symbol)); // Properly quoted without panic

    // 4. Extremely large / edge case numbers in risk score calculation
    let res = defender_tool
        .execute(json!({
            "action": "calculate_risk_score",
            "cvss_score": 999999.99,
            "reachability": "DormantUnreachable"
        }))
        .await
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&res).unwrap();
    assert!(parsed["defended_risk_score"].as_f64().unwrap() <= 1.0);

    // 5. Corrupted Bicep template syntax
    let corrupted_bicep = "resource {{{{{ [[[[(((( incomplete syntax";
    let parse_res = bicep_tool
        .execute(json!({
            "action": "parse_bicep",
            "bicep_code": corrupted_bicep
        }))
        .await
        .unwrap();
    let parsed_res: serde_json::Value = serde_json::from_str(&parse_res).unwrap();
    assert_eq!(parsed_res.as_array().unwrap().len(), 0); // Gracefully handles empty / corrupted tokens without crash
}
