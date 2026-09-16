//! Dedicated Brutal Verification Test Suite for Advanced Microsoft Subsystems
//!
//! Subsystems under test:
//! 1. `SarifEngine` & `AzurePipelinesGenerator` (`src/copilot/sarif.rs`, `copilot_sarif`)
//! 2. `JetBinaryEngine` (`src/copilot/jet_binary.rs`, `copilot_jet_binary`)
//! 3. `TeamsCallingEngine` & `WebRtcNegotiator` (`src/copilot/calling.rs`, `copilot_calling`)
//! 4. `DeltaLakeEngine` (`src/copilot/delta_lake.rs`, `copilot_delta_lake`)

use base64::Engine;
use serde_json::json;
use std::collections::HashMap;
use tagisan::copilot::calling::*;
use tagisan::copilot::delta_lake::*;
use tagisan::copilot::jet_binary::*;
use tagisan::copilot::sarif::*;
use tagisan::tools::ToolHandler;

// =========================================================================
// 1. SARIF 2.1.0 & Azure Pipelines Tests (Tests 1-8)
// =========================================================================

#[test]
fn test_sarif_schema_compliance_and_serialization() {
    let engine = SarifEngine::default();
    let doc = engine.to_sarif_document();

    assert_eq!(doc.schema, SARIF_SCHEMA_2_1_0);
    assert_eq!(doc.version, SARIF_VERSION);
    assert_eq!(doc.runs.len(), 1);
    assert_eq!(doc.runs[0].tool.driver.name, DEFAULT_TOOL_NAME);

    let json_str = engine.to_json_string(true).expect("SARIF serialization failed");
    assert!(json_str.contains("\"version\": \"2.1.0\""));
    assert!(json_str.contains("\"$schema\""));
    assert!(engine.validate_compliance().is_ok());
}

#[test]
fn test_sarif_transpile_invariant_failure() {
    let mut engine = SarifEngine::default();
    let result = engine.transpile_invariant_failure(
        "InvZeroKnowledgeProofValid",
        "verify_zkp(proof, public_inputs) == true",
        "src/fhe/prover.rs",
        88,
        0.94,
        "Zero-knowledge polynomial constraint unsatisfied",
    );

    assert_eq!(result.rule_id, "TAGISAN-INV-001");
    assert_eq!(result.level, SarifLevel::Error);
    assert_eq!(result.locations.len(), 1);
    assert_eq!(result.locations[0].physical_location.artifact_location.uri, "src/fhe/prover.rs");
    assert_eq!(result.locations[0].physical_location.region.as_ref().unwrap().start_line, 88);

    let doc = engine.to_sarif_document();
    assert_eq!(doc.runs[0].tool.driver.rules.len(), 1);
    assert_eq!(doc.runs[0].tool.driver.rules[0].id, "TAGISAN-INV-001");
}

#[test]
fn test_sarif_transpile_high_blast_radius() {
    let mut engine = SarifEngine::default();
    let downstream = vec![
        "execute_settlement".to_string(),
        "audit_ledger".to_string(),
        "dispatch_webhook".to_string(),
    ];

    let result = engine.transpile_blast_radius(
        "update_balance_atomic",
        "src/ledger/account.rs",
        210,
        "Critical",
        0.89,
        &downstream,
    );

    assert_eq!(result.rule_id, "TAGISAN-BLAST-001");
    assert_eq!(result.level, SarifLevel::Error);
    assert_eq!(result.related_locations.len(), 3);
    assert_eq!(
        result.properties.as_ref().unwrap().get("risk_tier").unwrap().as_str().unwrap(),
        "Critical"
    );
}

#[test]
fn test_sarif_transpile_cve_reachability_with_code_flows() {
    let mut engine = SarifEngine::default();
    let call_chain = [
        ("src/api/routes.rs", 42, "handle_http_request"),
        ("src/services/auth.rs", 108, "validate_jwt_token"),
        ("src/crypto/asn1.rs", 245, "parse_der_object"),
    ];

    let result = engine.transpile_cve_reachability(
        "CVE-2024-38077",
        "asn1-parser",
        9.8,
        &call_chain,
        "Buffer overflow in ASN.1 DER parser leads to RCE",
    );

    assert_eq!(result.rule_id, "DEFENDER-CVE_2024_38077");
    assert_eq!(result.level, SarifLevel::Error);
    assert_eq!(result.code_flows.len(), 1);

    let thread_flow = &result.code_flows[0].thread_flows[0];
    assert_eq!(thread_flow.locations.len(), 3);
    assert_eq!(thread_flow.locations[0].step, 1);
    assert_eq!(thread_flow.locations[0].kinds, vec!["source".to_string()]);
    assert_eq!(thread_flow.locations[2].step, 3);
    assert_eq!(thread_flow.locations[2].kinds, vec!["sink".to_string()]);
    assert_eq!(thread_flow.locations[2].importance.as_deref(), Some("essential"));
}

#[test]
fn test_sarif_engine_validation_and_integrity() {
    let mut engine = SarifEngine::default();
    assert!(engine.validate_compliance().is_ok());

    // Add multiple rules and results
    engine.transpile_invariant_failure("Inv1", "x > 0", "src/lib.rs", 10, 0.5, "Failed");
    engine.transpile_blast_radius("symbolA", "src/lib.rs", 20, "Low", 0.2, &[]);
    assert!(engine.validate_compliance().is_ok());

    let json_output = engine.to_json_string(false).unwrap();
    assert!(json_output.contains("TAGISAN-INV-001"));
    assert!(json_output.contains("TAGISAN-BLAST-001"));
}

#[test]
fn test_azure_pipelines_yaml_generation_and_validation() {
    let config = AzurePipelinesConfig::default();
    let yaml = AzurePipelinesGenerator::generate_yaml(&config);

    assert!(yaml.contains("Tagisan-CI-CD-Quality-Gate"));
    assert!(yaml.contains("TagisanVerification"));
    assert!(yaml.contains("QualityGates_And_Sarif"));
    assert!(yaml.contains("AST_Blast_Radius_Check"));
    assert!(yaml.contains("SMT_Formal_Verification"));
    assert!(yaml.contains("Defender_Reachable_CVE_Triage"));
    assert!(yaml.contains("AdvancedSecurity-Publish@1"));
    assert!(yaml.contains("PublishBuildArtifacts@1"));

    assert!(AzurePipelinesGenerator::validate_yaml(&yaml).is_ok());
}

#[test]
fn test_azure_pipelines_quality_gates_configuration() {
    let config = AzurePipelinesConfig {
        pipeline_name: "Enterprise-Payment-Gate".to_string(),
        trigger_branches: vec!["main".to_string(), "prod/*".to_string()],
        pr_branches: vec!["main".to_string(), "develop".to_string()],
        vm_image: "windows-latest".to_string(),
        blast_radius_threshold: 0.55,
        fail_on_sarif_errors: true,
        publish_to_advanced_security: false,
        smt_solver_timeout_secs: 120,
        enable_virtual_patch_pr: false,
        sarif_output_path: "C:\\build\\report.sarif".to_string(),
    };

    let yaml = AzurePipelinesGenerator::generate_yaml(&config);
    assert!(yaml.contains("Enterprise-Payment-Gate"));
    assert!(yaml.contains("vmImage: 'windows-latest'"));
    assert!(yaml.contains("BLAST_RADIUS_THRESHOLD: '0.55'"));
    assert!(yaml.contains("SMT_TIMEOUT_SECS: '120'"));
    // Since publish_to_advanced_security is false, task should not appear
    assert!(!yaml.contains("task: AdvancedSecurity-Publish@1"));
}

#[tokio::test]
async fn test_copilot_sarif_tool_actions() {
    let tool = CopilotSarifTool::new();
    assert_eq!(tool.name(), "copilot_sarif");

    // 1. transpile_invariants
    let inv_args = json!({
        "action": "transpile_invariants",
        "invariant_name": "InvNonNegativeBalance",
        "expression": "balance >= 0",
        "file": "src/vault.rs",
        "line": 45,
        "blast_score": 0.88,
        "details": "Vault underflow detected"
    });
    let inv_res = tool.execute(inv_args).await.expect("tool execution failed");
    assert!(inv_res.contains("TAGISAN-INV-001"));

    // 2. transpile_blast_radius
    let blast_args = json!({
        "action": "transpile_blast_radius",
        "symbol": "transfer_funds",
        "file": "src/vault.rs",
        "line": 102,
        "risk_tier": "Critical",
        "blast_score": 0.95,
        "downstream_symbols": ["auth_check", "notify_user"]
    });
    let blast_res = tool.execute(blast_args).await.expect("tool execution failed");
    assert!(blast_res.contains("TAGISAN-BLAST-001"));

    // 3. transpile_cve_reachability
    let cve_args = json!({
        "action": "transpile_cve_reachability",
        "cve_id": "CVE-2024-38077",
        "package": "win-rpc",
        "cvss_score": 9.8,
        "description": "Critical RCE in RPC subsystem"
    });
    let cve_res = tool.execute(cve_args).await.expect("tool execution failed");
    assert!(cve_res.contains("DEFENDER-CVE_2024_38077"));

    // 4. generate_sarif
    let gen_args = json!({ "action": "generate_sarif" });
    let gen_res = tool.execute(gen_args).await.expect("tool execution failed");
    assert!(gen_res.contains(SARIF_VERSION));

    // 5. validate_sarif
    let val_args = json!({ "action": "validate_sarif" });
    let val_res = tool.execute(val_args).await.expect("tool execution failed");
    assert!(val_res.contains("compliant"));

    // 6. generate_azure_pipeline
    let pipe_args = json!({ "action": "generate_azure_pipeline" });
    let pipe_res = tool.execute(pipe_args).await.expect("tool execution failed");
    assert!(pipe_res.contains("TagisanVerification"));
}

// =========================================================================
// 2. Jet/ACE Native Binary Database Tests (Tests 9-15)
// =========================================================================

#[test]
fn test_jet_binary_header_accdb_magic_validation() {
    let mut header_bytes = vec![0u8; PAGE_SIZE_ACE];
    header_bytes[0..4].copy_from_slice(&JET_MAGIC_PREFIX);
    header_bytes[4..4 + ACE_FORMAT_STRING.len()].copy_from_slice(ACE_FORMAT_STRING);
    header_bytes[0x14] = 0x03; // ACE 14

    let info = JetBinaryEngine::parse_header(&header_bytes).expect("Failed to parse ACCDB header");
    assert_eq!(info.format_signature, "Standard ACE DB");
    assert_eq!(info.engine_version, JetVersion::Ace14);
    assert_eq!(info.page_size, 4096);
    assert!(info.valid_magic);
    assert!(info.engine_version.is_accdb());
}

#[test]
fn test_jet_binary_header_mdb_magic_validation() {
    let mut header_bytes = vec![0u8; PAGE_SIZE_JET3];
    header_bytes[0..4].copy_from_slice(&JET_MAGIC_PREFIX);
    header_bytes[4..4 + JET_FORMAT_STRING.len()].copy_from_slice(JET_FORMAT_STRING);
    header_bytes[0x14] = 0x00; // Jet 3.x

    let info = JetBinaryEngine::parse_header(&header_bytes).expect("Failed to parse MDB header");
    assert_eq!(info.format_signature, "Standard Jet DB");
    assert_eq!(info.engine_version, JetVersion::Jet3);
    assert_eq!(info.page_size, 2048);
    assert!(!info.engine_version.is_accdb());
}

#[test]
fn test_jet_binary_tdef_column_parsing_and_types() {
    let mut tdef_page = vec![0u8; PAGE_SIZE_ACE];
    tdef_page[0] = PAGE_TYPE_TDEF;
    tdef_page[1] = 0x01;
    // Record count: 100
    tdef_page[4..8].copy_from_slice(&100u32.to_le_bytes());
    // Column count: 2
    tdef_page[8..10].copy_from_slice(&2u16.to_le_bytes());
    // Data page pointer: 3
    tdef_page[12..16].copy_from_slice(&3u32.to_le_bytes());
    // Table name
    let tname = b"Transactions";
    tdef_page[16..16 + tname.len()].copy_from_slice(tname);

    // Column 1: LongInteger
    let c1_offset = 48;
    tdef_page[c1_offset] = JetColumnType::LongInteger as u8;
    tdef_page[c1_offset + 1] = 0x02; // Autoincrement
    tdef_page[c1_offset + 2..c1_offset + 4].copy_from_slice(&4u16.to_le_bytes());
    tdef_page[c1_offset + 4..c1_offset + 6].copy_from_slice(&0u16.to_le_bytes());
    tdef_page[c1_offset + 8..c1_offset + 10].copy_from_slice(b"Id");

    // Column 2: Text
    let c2_offset = 48 + 24;
    tdef_page[c2_offset] = JetColumnType::Text as u8;
    tdef_page[c2_offset + 1] = 0x01; // Nullable
    tdef_page[c2_offset + 2..c2_offset + 4].copy_from_slice(&255u16.to_le_bytes());
    tdef_page[c2_offset + 4..c2_offset + 6].copy_from_slice(&4u16.to_le_bytes());
    tdef_page[c2_offset + 6..c2_offset + 8].copy_from_slice(&0u16.to_le_bytes()); // Var idx 0
    tdef_page[c2_offset + 8..c2_offset + 12].copy_from_slice(b"Desc");

    let table = JetBinaryEngine::parse_tdef_page(&tdef_page, 2).expect("TDEF parsing failed");
    assert_eq!(table.table_name, "Transactions");
    assert_eq!(table.record_count, 100);
    assert_eq!(table.columns.len(), 2);
    assert_eq!(table.columns[0].name, "Id");
    assert_eq!(table.columns[0].col_type, JetColumnType::LongInteger);
    assert!(table.columns[0].is_autoincrement);
    assert_eq!(table.columns[1].name, "Desc");
    assert_eq!(table.columns[1].col_type, JetColumnType::Text);
    assert!(table.columns[1].is_nullable);
}

#[test]
fn test_jet_binary_data_page_row_parsing_and_nulls() {
    let columns = vec![
        JetColumnDef {
            column_id: 0,
            name: "UserId".to_string(),
            col_type: JetColumnType::LongInteger,
            length: 4,
            offset: 0,
            variable_index: None,
            is_nullable: false,
            is_autoincrement: true,
        },
        JetColumnDef {
            column_id: 1,
            name: "Score".to_string(),
            col_type: JetColumnType::Double,
            length: 8,
            offset: 4,
            variable_index: None,
            is_nullable: true,
            is_autoincrement: false,
        },
        JetColumnDef {
            column_id: 2,
            name: "Comment".to_string(),
            col_type: JetColumnType::Text,
            length: 255,
            offset: 4,
            variable_index: Some(0),
            is_nullable: true,
            is_autoincrement: false,
        },
    ];

    let mut row1 = HashMap::new();
    row1.insert("UserId".to_string(), JetValue::Int32(42));
    row1.insert("Score".to_string(), JetValue::Float64(98.5));
    row1.insert("Comment".to_string(), JetValue::Text("Verified".to_string()));

    // Row 2 has NULL Score
    let mut row2 = HashMap::new();
    row2.insert("UserId".to_string(), JetValue::Int32(43));
    row2.insert("Score".to_string(), JetValue::Null);
    row2.insert("Comment".to_string(), JetValue::Text("Pending".to_string()));

    let accdb_bytes = JetBinaryEngine::synthesize_minimal_accdb("Users", &columns, &[row1, row2])
        .expect("Synthesis failed");

    let dumped = JetBinaryEngine::dump_table(&accdb_bytes, "Users").expect("Dump table failed");
    assert_eq!(dumped.len(), 2);

    assert_eq!(dumped[0].get("UserId"), Some(&JetValue::Int32(42)));
    assert_eq!(dumped[0].get("Score"), Some(&JetValue::Float64(98.5)));
    assert_eq!(dumped[0].get("Comment"), Some(&JetValue::Text("Verified".to_string())));

    assert_eq!(dumped[1].get("UserId"), Some(&JetValue::Int32(43)));
    assert_eq!(dumped[1].get("Score"), Some(&JetValue::Null));
    assert_eq!(dumped[1].get("Comment"), Some(&JetValue::Text("Pending".to_string())));
}

#[test]
fn test_jet_binary_accdb_synthesis_and_roundtrip() {
    let columns = vec![
        JetColumnDef {
            column_id: 0,
            name: "RuleId".to_string(),
            col_type: JetColumnType::Integer,
            length: 2,
            offset: 0,
            variable_index: None,
            is_nullable: false,
            is_autoincrement: false,
        },
        JetColumnDef {
            column_id: 1,
            name: "IsActive".to_string(),
            col_type: JetColumnType::Boolean,
            length: 1,
            offset: 2,
            variable_index: None,
            is_nullable: false,
            is_autoincrement: false,
        },
        JetColumnDef {
            column_id: 2,
            name: "RuleName".to_string(),
            col_type: JetColumnType::Text,
            length: 128,
            offset: 3,
            variable_index: Some(0),
            is_nullable: false,
            is_autoincrement: false,
        },
    ];

    let mut rows = Vec::new();
    for i in 1..=5 {
        let mut r = HashMap::new();
        r.insert("RuleId".to_string(), JetValue::Int16(i));
        r.insert("IsActive".to_string(), JetValue::Bool(i % 2 == 0));
        r.insert("RuleName".to_string(), JetValue::Text(format!("Security_Rule_{i}")));
        rows.push(r);
    }

    let bytes = JetBinaryEngine::synthesize_minimal_accdb("SecurityRules", &columns, &rows)
        .expect("Failed to synthesize ACCDB");

    assert_eq!(bytes.len(), PAGE_SIZE_ACE * 4);

    let report = JetBinaryEngine::verify_integrity(&bytes).expect("Integrity check failed");
    assert!(report.is_valid);
    assert_eq!(report.table_count, 1);
    assert_eq!(report.total_records, 5);

    let dumped = JetBinaryEngine::dump_table(&bytes, "SecurityRules").expect("Dump failed");
    assert_eq!(dumped.len(), 5);
    for (idx, r) in dumped.iter().enumerate() {
        let expected_id = (idx + 1) as i16;
        assert_eq!(r.get("RuleId"), Some(&JetValue::Int16(expected_id)));
        assert_eq!(r.get("IsActive"), Some(&JetValue::Bool(expected_id % 2 == 0)));
        assert_eq!(
            r.get("RuleName"),
            Some(&JetValue::Text(format!("Security_Rule_{expected_id}")))
        );
    }
}

#[test]
fn test_jet_binary_corrupted_magic_fuzzing() {
    let mut corrupted = vec![0xFFu8; 4096];
    assert!(JetBinaryEngine::parse_header(&corrupted).is_err());

    // Valid magic prefix but invalid format string
    corrupted[0..4].copy_from_slice(&JET_MAGIC_PREFIX);
    assert!(JetBinaryEngine::parse_header(&corrupted).is_err());

    // Truncated buffer
    let truncated = vec![0x00, 0x01, 0x00, 0x00];
    assert!(JetBinaryEngine::parse_header(&truncated).is_err());
}

#[tokio::test]
async fn test_copilot_jet_binary_tool_actions() {
    let tool = CopilotJetBinaryTool::new();
    assert_eq!(tool.name(), "copilot_jet_binary");

    // 1. synthesize_accdb
    let syn_args = json!({
        "action": "synthesize_accdb",
        "table_name": "AuditLogs"
    });
    let syn_res_str = tool.execute(syn_args).await.expect("tool execution failed");
    let syn_res: serde_json::Value = serde_json::from_str(&syn_res_str).unwrap();
    assert_eq!(syn_res["status"], "success");
    assert_eq!(syn_res["table_name"], "AuditLogs");

    // Synthesize real bytes to test parse_header, inspect_schema, verify_integrity
    let columns = vec![JetColumnDef {
        column_id: 0,
        name: "Id".to_string(),
        col_type: JetColumnType::LongInteger,
        length: 4,
        offset: 0,
        variable_index: None,
        is_nullable: false,
        is_autoincrement: true,
    }];
    let mut row = HashMap::new();
    row.insert("Id".to_string(), JetValue::Int32(101));
    let accdb_bytes = JetBinaryEngine::synthesize_minimal_accdb("AuditLogs", &columns, &[row]).unwrap();
    let b64_bytes = base64::prelude::BASE64_STANDARD.encode(&accdb_bytes);

    // 2. parse_header
    let header_args = json!({
        "action": "parse_header",
        "base64_bytes": b64_bytes
    });
    let header_res = tool.execute(header_args).await.expect("tool execution failed");
    assert!(header_res.contains("Standard ACE DB"));

    // 3. inspect_schema
    let schema_args = json!({
        "action": "inspect_schema",
        "base64_bytes": b64_bytes
    });
    let schema_res = tool.execute(schema_args).await.expect("tool execution failed");
    assert!(schema_res.contains("AuditLogs"));

    // 4. dump_table
    let dump_args = json!({
        "action": "dump_table",
        "table_name": "AuditLogs",
        "base64_bytes": b64_bytes
    });
    let dump_res = tool.execute(dump_args).await.expect("tool execution failed");
    assert!(dump_res.contains("101"));

    // 5. verify_integrity
    let verify_args = json!({
        "action": "verify_integrity",
        "base64_bytes": b64_bytes
    });
    let verify_res = tool.execute(verify_args).await.expect("tool execution failed");
    assert!(verify_res.contains("\"is_valid\": true"));
}

// =========================================================================
// 3. Teams Real-Time Calling & Live Media Bot Tests (Tests 16-21)
// =========================================================================

#[tokio::test]
async fn test_teams_calling_state_machine_lifecycle() {
    let engine = TeamsCallingEngine::new("call-meeting-lifecycle-101");
    let initial_snap = engine.snapshot().await;
    assert_eq!(initial_snap.state, CallState::Incoming);

    // Incoming -> Establishing
    assert!(engine.transition_state(CallState::Establishing).await.is_ok());
    // Establishing -> Established
    assert!(engine.transition_state(CallState::Established).await.is_ok());
    // Established -> Terminating
    assert!(engine.transition_state(CallState::Terminating).await.is_ok());
    // Terminating -> Terminated
    assert!(engine.transition_state(CallState::Terminated).await.is_ok());

    // Terminated cannot transition to anything
    assert!(engine.transition_state(CallState::Established).await.is_err());
}

#[tokio::test]
async fn test_webrtc_sdp_offer_answer_handshake() {
    let offer = WebRtcNegotiator::generate_offer("session-101", "192.168.1.50");
    assert!(offer.contains("v=0"));
    assert!(offer.contains("m=audio"));
    assert!(offer.contains("opus/48000/2"));
    assert!(offer.contains("ice-ufrag"));

    let answer = WebRtcNegotiator::generate_answer("session-101", "192.168.1.50", &offer)
        .expect("SDP answer negotiation failed");
    assert!(answer.contains("v=0"));
    assert!(answer.contains("m=audio"));
    assert!(answer.contains("a=sendrecv"));
}

#[tokio::test]
async fn test_pcm_audio_frame_buffering_and_vad_rms() {
    let engine = TeamsCallingEngine::new("call-audio-test-102");

    // 1. Silent PCM frame (all 0s, 640 bytes = 320 samples)
    let silence = vec![0u8; 640];
    let (rms_silence, is_active_silence) = engine.ingest_pcm_frame(&silence).await.unwrap();
    assert_eq!(rms_silence, 0.0);
    assert!(!is_active_silence);

    // 2. High amplitude sine / square wave simulating speech (amplitude 8000)
    let mut speech = Vec::with_capacity(640);
    for i in 0..320 {
        let sample: i16 = if i % 2 == 0 { 8000 } else { -8000 };
        speech.extend_from_slice(&sample.to_le_bytes());
    }
    let (rms_speech, is_active_speech) = engine.ingest_pcm_frame(&speech).await.unwrap();
    assert!(rms_speech > 500.0);
    assert!(is_active_speech);

    let snap = engine.snapshot().await;
    assert_eq!(snap.media_stats.total_frames, 2);
    assert_eq!(snap.media_stats.active_speech_frames, 1);
    assert_eq!(snap.media_stats.silence_frames, 1);
    assert!(snap.media_stats.peak_rms_energy >= 8000.0);
}

#[tokio::test]
async fn test_multi_speaker_transcript_diarization_and_triggers() {
    let engine = TeamsCallingEngine::new("call-transcript-103");

    // Speaker 1: Architect speaks without trigger
    let trig1 = engine
        .ingest_transcript_segment(
            "user-1",
            "Charle Gutierrez",
            "Good morning team, let's look at the pull request diff.",
            false,
        )
        .await;
    assert!(trig1.is_empty());

    // Speaker 2: Lead dev mentions blast radius
    let trig2 = engine
        .ingest_transcript_segment(
            "user-2",
            "Lead Developer",
            "Warning: modifying this symbol introduces high blast radius into production.",
            false,
        )
        .await;
    assert_eq!(trig2.len(), 1);
    assert_eq!(trig2[0].keyword, "blast radius");
    assert_eq!(trig2[0].risk_level, "Critical");
    assert_eq!(trig2[0].blast_score, Some(0.85));

    // Speaker 3: Security officer mentions CVE
    let trig3 = engine
        .ingest_transcript_segment(
            "user-3",
            "Security Officer",
            "@Tagisan please check if CVE-2024-38077 is reachable in this service.",
            false,
        )
        .await;
    assert_eq!(trig3.len(), 2); // '@tagisan' and 'cve'
}

#[test]
fn test_in_call_adaptive_card_intervention_payload() {
    let trigger = TriggerDetection {
        keyword: "blast radius".to_string(),
        speaker: "Lead Architect".to_string(),
        context_text: "The blast radius of this AST change affects 14 downstream systems.".to_string(),
        timestamp_ms: 1726000000000,
        risk_level: "Critical".to_string(),
        blast_score: Some(0.92),
    };

    let impacted = vec!["auth_service".to_string(), "billing_engine".to_string()];
    let intervention = TeamsCallingEngine::generate_adaptive_card_intervention(
        "meeting-card-001",
        &trigger,
        0.92,
        &impacted,
    );

    assert_eq!(intervention.target_meeting_id, "meeting-card-001");
    assert_eq!(intervention.risk_level, "Critical");
    let card_str = serde_json::to_string(&intervention.card_json).unwrap();
    assert!(card_str.contains("TAGISAN LIVE MEETING INTERVENTION"));
    assert!(card_str.contains("Enforce Invariant SMT Gate"));
    assert!(card_str.contains("auth_service"));
}

#[tokio::test]
async fn test_copilot_calling_tool_actions() {
    let tool = CopilotCallingTool::new();
    assert_eq!(tool.name(), "copilot_calling");

    // 1. create_call
    let create_args = json!({
        "action": "create_call",
        "call_id": "call-swarm-001"
    });
    let create_res = tool.execute(create_args).await.expect("tool execution failed");
    assert!(create_res.contains("sdp_offer"));

    // 2. answer_call
    let answer_args = json!({
        "action": "answer_call",
        "remote_sdp": "v=0\r\nm=audio 9 RTP/SAVPF 111\r\n"
    });
    let answer_res = tool.execute(answer_args).await.expect("tool execution failed");
    assert!(answer_res.contains("Established"));

    // 3. ingest_audio_frame (640 zero bytes)
    let pcm_b64 = base64::prelude::BASE64_STANDARD.encode(vec![0u8; 640]);
    let audio_args = json!({
        "action": "ingest_audio_frame",
        "pcm_base64": pcm_b64
    });
    let audio_res = tool.execute(audio_args).await.expect("tool execution failed");
    assert!(audio_res.contains("\"voice_activity\": false"));

    // 4. ingest_transcript
    let tx_args = json!({
        "action": "ingest_transcript",
        "speaker_id": "u-42",
        "speaker_name": "DevSecOps",
        "transcript_text": "We need to verify the invariant before merge."
    });
    let tx_res = tool.execute(tx_args).await.expect("tool execution failed");
    assert!(tx_res.contains("invariant"));

    // 5. trigger_intervention
    let int_args = json!({
        "action": "trigger_intervention",
        "speaker_name": "DevSecOps",
        "blast_score": 0.89,
        "impacted_symbols": ["order_processor"]
    });
    let int_res = tool.execute(int_args).await.expect("tool execution failed");
    assert!(int_res.contains("AdaptiveCard"));

    // 6. get_call_state
    let state_args = json!({ "action": "get_call_state" });
    let state_res = tool.execute(state_args).await.expect("tool execution failed");
    assert!(state_res.contains("call-default-meeting-001"));

    // 7. terminate_call
    let term_args = json!({ "action": "terminate_call" });
    let term_res = tool.execute(term_args).await.expect("tool execution failed");
    assert!(term_res.contains("Terminated"));
}

// =========================================================================
// 4. OneLake Delta Lake Parquet Engine Tests (Tests 22-27)
// =========================================================================

#[tokio::test]
async fn test_delta_lake_initial_commit_log_generation() {
    let engine = DeltaLakeEngine::new("ASTTelemetry");
    let schema_fields = [
        ("id", "long", false),
        ("symbol", "string", false),
        ("blast_score", "double", true),
        ("timestamp", "timestamp", false),
    ];
    let partition_cols = ["date"];

    let actions = engine
        .create_table(&schema_fields, &partition_cols, Some("AST Blast Telemetry"))
        .await
        .expect("Create Delta table failed");

    assert_eq!(actions.len(), 3);
    assert!(actions[0].commit_info.is_some());
    assert!(actions[1].protocol.is_some());
    assert!(actions[2].meta_data.is_some());

    let meta = actions[2].meta_data.as_ref().unwrap();
    assert_eq!(meta.name.as_deref(), Some("ASTTelemetry"));
    assert_eq!(meta.partition_columns, vec!["date".to_string()]);
    assert!(meta.schema_string.contains("\"name\":\"symbol\""));

    let snapshot = engine.read_version(0).await.expect("Failed to read version 0");
    assert_eq!(snapshot.version, 0);
    assert_eq!(snapshot.table_name, "ASTTelemetry");
    assert_eq!(snapshot.active_files.len(), 0);
}

#[tokio::test]
async fn test_delta_lake_multi_commit_and_stats_validation() {
    let engine = DeltaLakeEngine::new("SecurityAudit");
    let schema_fields = [("event_id", "string", false), ("score", "double", false)];
    engine.create_table(&schema_fields, &[], None).await.unwrap();

    // Commit 1: Add 2 Parquet files
    let file1_stats = serde_json::to_string(&DeltaFileStats {
        num_records: 1000,
        min_values: {
            let mut m = HashMap::new();
            m.insert("score".to_string(), json!(0.1));
            m
        },
        max_values: {
            let mut m = HashMap::new();
            m.insert("score".to_string(), json!(0.9));
            m
        },
        null_count: HashMap::new(),
    })
    .unwrap();

    let add1 = DeltaAddAction {
        path: "part-00001.parquet".to_string(),
        size: 51200,
        modification_time: 1726000000000,
        data_change: true,
        partition_values: HashMap::new(),
        stats: file1_stats,
    };

    let (ver1, actions1) = engine
        .commit_transaction("WRITE", vec![add1], Vec::new())
        .await
        .unwrap();
    assert_eq!(ver1, 1);
    assert_eq!(actions1.len(), 2); // 1 commitInfo + 1 add

    let snap1 = engine.read_version(1).await.unwrap();
    assert_eq!(snap1.active_files.len(), 1);
    assert_eq!(snap1.total_records, 1000);
    assert_eq!(snap1.total_bytes, 51200);
}

#[tokio::test]
async fn test_delta_lake_time_travel_version_reading() {
    let engine = DeltaLakeEngine::new("TimeTravelTest");
    let schema_fields = [("id", "long", false)];
    engine.create_table(&schema_fields, &[], None).await.unwrap();

    // Commit 1: Add file A (100 records)
    let add_a = DeltaAddAction {
        path: "file_a.parquet".to_string(),
        size: 1000,
        modification_time: 1000,
        data_change: true,
        partition_values: HashMap::new(),
        stats: serde_json::to_string(&DeltaFileStats {
            num_records: 100,
            min_values: HashMap::new(),
            max_values: HashMap::new(),
            null_count: HashMap::new(),
        })
        .unwrap(),
    };
    engine.commit_transaction("WRITE", vec![add_a], Vec::new()).await.unwrap();

    // Commit 2: Add file B (200 records) and remove file A
    let add_b = DeltaAddAction {
        path: "file_b.parquet".to_string(),
        size: 2000,
        modification_time: 2000,
        data_change: true,
        partition_values: HashMap::new(),
        stats: serde_json::to_string(&DeltaFileStats {
            num_records: 200,
            min_values: HashMap::new(),
            max_values: HashMap::new(),
            null_count: HashMap::new(),
        })
        .unwrap(),
    };
    let rem_a = DeltaRemoveAction {
        path: "file_a.parquet".to_string(),
        deletion_timestamp: 2000,
        data_change: true,
    };
    engine.commit_transaction("COMPACT", vec![add_b], vec![rem_a]).await.unwrap();

    // Time Travel to Version 0: 0 files, 0 records
    let v0 = engine.read_version(0).await.unwrap();
    assert_eq!(v0.active_files.len(), 0);
    assert_eq!(v0.total_records, 0);

    // Time Travel to Version 1: file_a active, 100 records
    let v1 = engine.read_version(1).await.unwrap();
    assert_eq!(v1.active_files.len(), 1);
    assert_eq!(v1.active_files[0].path, "file_a.parquet");
    assert_eq!(v1.total_records, 100);

    // Time Travel to Version 2: file_b active, file_a removed, 200 records
    let v2 = engine.read_version(2).await.unwrap();
    assert_eq!(v2.active_files.len(), 1);
    assert_eq!(v2.active_files[0].path, "file_b.parquet");
    assert_eq!(v2.total_records, 200);

    // Version beyond max should error
    assert!(engine.read_version(99).await.is_err());
}

#[tokio::test]
async fn test_delta_lake_checkpoint_compaction_and_restore() {
    let engine = DeltaLakeEngine::new("CheckpointTest");
    let schema_fields = [("metric", "double", false)];
    engine.create_table(&schema_fields, &[], None).await.unwrap();

    for i in 1..=5 {
        let add = DeltaAddAction {
            path: format!("metric_{i}.parquet"),
            size: 500,
            modification_time: i * 1000,
            data_change: true,
            partition_values: HashMap::new(),
            stats: serde_json::to_string(&DeltaFileStats {
                num_records: 10,
                min_values: HashMap::new(),
                max_values: HashMap::new(),
                null_count: HashMap::new(),
            })
            .unwrap(),
        };
        engine.commit_transaction("APPEND", vec![add], Vec::new()).await.unwrap();
    }

    let chk_name = engine.create_checkpoint(5).await.expect("Checkpoint creation failed");
    assert_eq!(chk_name, "00000000000000000005.checkpoint.json");

    // Reading version 5 now leverages the checkpoint
    let snap5 = engine.read_version(5).await.unwrap();
    assert_eq!(snap5.active_files.len(), 5);
    assert_eq!(snap5.total_records, 50);
}

#[test]
fn test_onelake_adls_path_mapping_and_uri_schemes() {
    let mapping = DeltaLakeEngine::map_onelake_path(
        "FinancialServices-Prod",
        "TradingLakehouse",
        "RiskLedger",
    );

    assert_eq!(mapping.workspace, "FinancialServices-Prod");
    assert_eq!(
        mapping.adls_https_url,
        "https://onelake.dfs.fabric.microsoft.com/FinancialServices-Prod/TradingLakehouse.Lakehouse/Tables/RiskLedger"
    );
    assert_eq!(
        mapping.blob_https_url,
        "https://onelake.blob.fabric.microsoft.com/FinancialServices-Prod/TradingLakehouse.Lakehouse/Tables/RiskLedger"
    );
    assert_eq!(
        mapping.abfss_url,
        "abfss://FinancialServices-Prod@onelake.dfs.fabric.microsoft.com/TradingLakehouse.Lakehouse/Tables/RiskLedger"
    );
    assert_eq!(mapping.local_relative_path, "TradingLakehouse/Tables/RiskLedger");
}

#[tokio::test]
async fn test_copilot_delta_lake_tool_actions() {
    let tool = CopilotDeltaLakeTool::new();
    assert_eq!(tool.name(), "copilot_delta_lake");

    // 1. create_table
    let create_args = json!({
        "action": "create_table",
        "table_name": "ASTTelemetry"
    });
    let create_res = tool.execute(create_args).await.expect("tool execution failed");
    assert!(create_res.contains("table_created"));

    // 2. commit_transaction
    let commit_args = json!({
        "action": "commit_transaction"
    });
    let commit_res = tool.execute(commit_args).await.expect("tool execution failed");
    assert!(commit_res.contains("committed"));

    // 3. read_version
    let read_args = json!({
        "action": "read_version",
        "version": 1
    });
    let read_res = tool.execute(read_args).await.expect("tool execution failed");
    assert!(read_res.contains("activeFiles"));

    // 4. create_checkpoint
    let chk_args = json!({
        "action": "create_checkpoint",
        "version": 1
    });
    let chk_res = tool.execute(chk_args).await.expect("tool execution failed");
    assert!(chk_res.contains("checkpoint_created"));

    // 5. map_onelake_path
    let map_args = json!({
        "action": "map_onelake_path",
        "workspace": "MyWorkspace",
        "lakehouse_name": "MyLakehouse",
        "table_name": "MyTable"
    });
    let map_res = tool.execute(map_args).await.expect("tool execution failed");
    assert!(map_res.contains("onelake.dfs.fabric.microsoft.com"));

    // 6. get_table_history
    let hist_args = json!({ "action": "get_table_history" });
    let hist_res = tool.execute(hist_args).await.expect("tool execution failed");
    assert!(hist_res.contains("commits"));
}

// =========================================================================
// 5. Concurrent Stress & Fuzzing Resilience Tests (Tests 28-29)
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_50_worker_concurrent_stress_across_subsystems() {
    let mut handles = Vec::with_capacity(50);

    for worker_id in 0..50 {
        let handle = tokio::spawn(async move {
            let task_type = worker_id % 4;
            match task_type {
                0 => {
                    // Subsystem 1: SARIF Engine
                    let mut sarif = SarifEngine::new(
                        format!("Worker-{worker_id}"),
                        "0.2.0",
                        "https://tagisan.ai",
                    );
                    sarif.transpile_invariant_failure(
                        &format!("Inv_{worker_id}"),
                        "a < b",
                        "src/lib.rs",
                        worker_id as u32,
                        0.75,
                        "Invariant failed",
                    );
                    assert!(sarif.validate_compliance().is_ok());
                }
                1 => {
                    // Subsystem 2: Jet/ACE Binary Engine
                    let columns = vec![JetColumnDef {
                        column_id: 0,
                        name: "Id".to_string(),
                        col_type: JetColumnType::LongInteger,
                        length: 4,
                        offset: 0,
                        variable_index: None,
                        is_nullable: false,
                        is_autoincrement: true,
                    }];
                    let mut row = HashMap::new();
                    row.insert("Id".to_string(), JetValue::Int32(worker_id as i32));
                    let accdb = JetBinaryEngine::synthesize_minimal_accdb(
                        &format!("T_{worker_id}"),
                        &columns,
                        &[row],
                    )
                    .unwrap();
                    let report = JetBinaryEngine::verify_integrity(&accdb).unwrap();
                    assert!(report.is_valid);
                }
                2 => {
                    // Subsystem 3: Teams Calling Engine
                    let calling = TeamsCallingEngine::new(format!("call-worker-{worker_id}"));
                    calling.transition_state(CallState::Establishing).await.unwrap();
                    calling.transition_state(CallState::Established).await.unwrap();
                    let pcm = vec![127u8; 640];
                    let (rms, _is_active) = calling.ingest_pcm_frame(&pcm).await.unwrap();
                    assert!(rms > 0.0);
                    let triggers = calling
                        .ingest_transcript_segment(
                            &format!("u-{worker_id}"),
                            "Architect",
                            "Let's review the blast radius",
                            false,
                        )
                        .await;
                    assert_eq!(triggers.len(), 1);
                    calling.terminate().await.unwrap();
                }
                3 => {
                    // Subsystem 4: Delta Lake Engine
                    let delta = DeltaLakeEngine::new(format!("Table_{worker_id}"));
                    delta.create_table(&[("id", "long", false)], &[], None).await.unwrap();
                    let (ver, _) = delta
                        .commit_transaction("WRITE", Vec::new(), Vec::new())
                        .await
                        .unwrap();
                    assert_eq!(ver, 1);
                    let snap = delta.read_version(1).await.unwrap();
                    assert_eq!(snap.version, 1);
                }
                _ => unreachable!(),
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("Concurrent worker failed");
    }
}

#[tokio::test]
async fn test_subsystems_fuzzing_malformed_inputs_resilience() {
    // 1. Fuzz SARIF validation with bad JSON
    let sarif_tool = CopilotSarifTool::new();
    let bad_sarif = json!({
        "action": "validate_sarif",
        "sarif_content": "{ not valid json"
    });
    assert!(sarif_tool.execute(bad_sarif).await.is_err());

    // 2. Fuzz Jet binary with arbitrary byte noise
    let noise = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33];
    let b64_noise = base64::prelude::BASE64_STANDARD.encode(&noise);
    let jet_tool = CopilotJetBinaryTool::new();
    let bad_jet = json!({
        "action": "parse_header",
        "base64_bytes": b64_noise
    });
    assert!(jet_tool.execute(bad_jet).await.is_err());

    // 3. Fuzz Calling PCM with odd byte count (non-16-bit aligned)
    let calling_tool = CopilotCallingTool::new();
    let bad_pcm = base64::prelude::BASE64_STANDARD.encode(vec![1, 2, 3]);
    let bad_audio = json!({
        "action": "ingest_audio_frame",
        "pcm_base64": bad_pcm
    });
    assert!(calling_tool.execute(bad_audio).await.is_err());

    // 4. Fuzz Delta Lake time travel with future version
    let delta_tool = CopilotDeltaLakeTool::new();
    let bad_read = json!({
        "action": "read_version",
        "version": 999999
    });
    assert!(delta_tool.execute(bad_read).await.is_err());
}
