//! Comprehensive Integration Tests for Native Hermes Agent Integration in Tagisan (TGS)

use serde_json::json;
use tagisan::ecc::{all_presets, find_preset};
use tagisan::hermes::{
    HermesHybridTier, HermesRedTeamAuditor, HermesSavingsReport, HermesTaskType,
    HermesToolResponse, HermesXmlProtocol, RedTeamProbeCategory, RedTeamSeverity,
};
use tagisan::swarm::repl::ReplCommand;
use tagisan::types::ToolDefinition;

// ============================================================================
// 1. XML Protocol: Tool Calling & Scratchpad Extraction Tests
// ============================================================================

#[test]
fn test_hermes_xml_tool_call_extraction_single() {
    let raw_response = r#"
I will check the files in the directory.
<tool_call>
{"name": "list_dir", "arguments": {"path": "/workspace/src"}}
</tool_call>
"#;

    let calls = HermesXmlProtocol::extract_tool_calls(raw_response).expect("Failed to extract tool call");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "list_dir");
    assert_eq!(calls[0].arguments, json!({ "path": "/workspace/src" }));
}

#[test]
fn test_hermes_xml_tool_call_extraction_multiple() {
    let raw_response = r#"
We need to read both the library file and the cargo config:
<tool_call>
{"name": "read_file", "arguments": {"path": "src/lib.rs"}}
</tool_call>
Now reading Cargo.toml:
<tool_call>
{"name": "read_file", "arguments": {"path": "Cargo.toml"}}
</tool_call>
"#;

    let calls = HermesXmlProtocol::extract_tool_calls(raw_response).expect("Failed to extract multiple tool calls");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "read_file");
    assert_eq!(calls[0].arguments, json!({ "path": "src/lib.rs" }));
    assert_eq!(calls[1].name, "read_file");
    assert_eq!(calls[1].arguments, json!({ "path": "Cargo.toml" }));
}

#[test]
fn test_hermes_xml_with_thought() {
    let raw_response = r#"
<thought>
The user asked for code metrics. I should invoke the code graph query tool first to measure cyclomatic complexity and symbol depth.
</thought>
<tool_call>
{"name": "query_code_graph", "arguments": {"symbol": "HermesXmlProtocol"}}
</tool_call>
"#;

    let thoughts = HermesXmlProtocol::extract_thoughts(raw_response);
    assert_eq!(thoughts.len(), 1);
    assert!(thoughts[0].contains("cyclomatic complexity"));

    let calls = HermesXmlProtocol::extract_tool_calls(raw_response).expect("Failed to extract tool call");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "query_code_graph");
}

#[test]
fn test_hermes_xml_without_thought() {
    let raw_response = r#"
<tool_call>
{"name": "calculator", "arguments": {"expression": "2 * 3.14159 * 10"}}
</tool_call>
"#;

    let thoughts = HermesXmlProtocol::extract_thoughts(raw_response);
    assert!(thoughts.is_empty());

    let calls = HermesXmlProtocol::extract_tool_calls(raw_response).expect("Failed to extract tool call");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "calculator");
}

#[test]
fn test_hermes_xml_with_markdown_fences() {
    let raw_response = r#"
<tool_call>
```json
{
  "name": "run_command",
  "arguments": {
    "cmd": "cargo test --release"
  }
}
```
</tool_call>
"#;

    let calls = HermesXmlProtocol::extract_tool_calls(raw_response).expect("Failed to extract tool call with fences");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "run_command");
    assert_eq!(calls[0].arguments, json!({ "cmd": "cargo test --release" }));
}

#[test]
fn test_hermes_xml_conversational_text_stripping() {
    let raw_response = r#"
Hello! I am ready to inspect your architecture.
<thought>
Need to read the configuration file.
</thought>
<tool_call>
{"name": "read_file", "arguments": {"path": "config.json"}}
</tool_call>
Please wait while I read the configuration.
"#;

    let turn = HermesXmlProtocol::parse_turn(raw_response).expect("Failed to parse full turn");
    assert_eq!(turn.thoughts.len(), 1);
    assert_eq!(turn.tool_calls.len(), 1);
    assert!(turn.conversational_content.contains("Hello! I am ready to inspect your architecture."));
    assert!(turn.conversational_content.contains("Please wait while I read the configuration."));
    assert!(!turn.conversational_content.contains("<thought>"));
    assert!(!turn.conversational_content.contains("<tool_call>"));
}

#[test]
fn test_hermes_tools_system_prompt_generation() {
    let tools = vec![
        ToolDefinition::new(
            "calculator",
            "Perform arithmetic evaluation",
            json!({
                "type": "object",
                "properties": {
                    "expression": { "type": "string" }
                },
                "required": ["expression"]
            }),
        ),
        ToolDefinition::new(
            "read_file",
            "Read file contents",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        ),
    ];

    let prompt = HermesXmlProtocol::format_tools_system_prompt(&tools, Some("Custom System Rule"));
    assert!(prompt.contains("Custom System Rule"));
    assert!(prompt.contains("<tools>"));
    assert!(prompt.contains("</tools>"));
    assert!(prompt.contains("calculator"));
    assert!(prompt.contains("read_file"));
    assert!(prompt.contains("<tool_call>"));
    assert!(prompt.contains("<thought>"));
}

#[test]
fn test_hermes_tool_response_formatting() {
    let resp1 = HermesToolResponse::new("read_file", json!({ "bytes": 1024, "lines": 42 }));
    let formatted1 = HermesXmlProtocol::format_tool_response(&resp1);
    assert!(formatted1.starts_with("<tool_response>"));
    assert!(formatted1.ends_with("</tool_response>"));
    assert!(formatted1.contains("\"name\":\"read_file\"") || formatted1.contains("\"name\": \"read_file\""));

    let resp2 = HermesToolResponse::error("execute", "Execution timed out after 30s");
    let formatted2 = HermesXmlProtocol::format_tool_response(&resp2);
    assert!(formatted2.contains("Execution timed out after 30s"));

    let combined = HermesXmlProtocol::format_tool_responses(&[resp1, resp2]);
    assert!(combined.contains("read_file"));
    assert!(combined.contains("Execution timed out"));
}

// ============================================================================
// 2. Hybrid Tier Routing & Cost Savings Tests
// ============================================================================

#[test]
fn test_hermes_hybrid_tier_routing() {
    let tier = HermesHybridTier::new("hermes3:8b", "claude-3-5-sonnet-20241022");

    // Routine edits -> local Hermes workhorse
    let dec_edit = tier.route_task(HermesTaskType::RoutineEdit, 500);
    assert!(dec_edit.is_local);
    assert_eq!(dec_edit.target_model, "hermes3:8b");

    // Red team probe -> local Hermes workhorse (zero refusal friction)
    let dec_rt = tier.route_task(HermesTaskType::RedTeamProbe, 1200);
    assert!(dec_rt.is_local);
    assert_eq!(dec_rt.target_model, "hermes3:8b");

    // Compiler syntax fix -> local Hermes workhorse
    let dec_syntax = tier.route_task(HermesTaskType::SyntaxFix, 300);
    assert!(dec_syntax.is_local);
    assert_eq!(dec_syntax.target_model, "hermes3:8b");

    // Architectural decisions -> cloud orchestrator
    let dec_arch = tier.route_task(HermesTaskType::ArchitecturalDecision, 2000);
    assert!(!dec_arch.is_local);
    assert_eq!(dec_arch.target_model, "claude-3-5-sonnet-20241022");

    // Judicial debate arbitration -> cloud orchestrator
    let dec_judge = tier.route_task(HermesTaskType::JudicialDebateArbiter, 1500);
    assert!(!dec_judge.is_local);
    assert_eq!(dec_judge.target_model, "claude-3-5-sonnet-20241022");

    // High stakes synthesis exceeding token threshold -> cloud
    let dec_high_exceed = tier.route_task(HermesTaskType::HighStakesSynthesis, 8000);
    assert!(!dec_high_exceed.is_local);
    assert_eq!(dec_high_exceed.target_model, "claude-3-5-sonnet-20241022");
}

#[test]
fn test_hermes_hybrid_tier_cost_savings() {
    let mut tier = HermesHybridTier::new("hermes3:8b", "claude-3-5-sonnet-20241022")
        .with_cloud_pricing(3.00, 15.00); // $3/M in, $15/M out

    // 10 local workhorse turns (e.g. 50,000 input tokens, 25,000 output tokens)
    for _ in 0..10 {
        tier.record_usage(true, 5_000, 2_500);
    }

    // 2 cloud strategic turns (e.g. 10,000 input tokens, 4,000 output tokens)
    for _ in 0..2 {
        tier.record_usage(false, 5_000, 2_000);
    }

    let report: HermesSavingsReport = tier.calculate_savings();

    assert_eq!(report.local_calls_count, 10);
    assert_eq!(report.cloud_calls_count, 2);
    assert_eq!(report.total_local_tokens, 75_000);
    assert_eq!(report.total_cloud_tokens, 14_000);
    assert_eq!(report.total_tokens, 89_000);

    // Local cost is 0
    assert_eq!(report.local_cost, 0.0);

    // Cloud cost incurred:
    // 10,000 * 3 / 1M = 0.03
    // 4,000 * 15 / 1M = 0.06
    // Total = $0.09
    assert!((report.cloud_cost_incurred - 0.09).abs() < 1e-4);

    // Counterfactual if all 60,000 input & 29,000 output were run on cloud:
    // 60,000 * 3 / 1M = 0.18
    // 29,000 * 15 / 1M = 0.435
    // Total counterfactual = $0.615
    assert!((report.counterfactual_cloud_cost - 0.615).abs() < 1e-4);

    // Dollars saved = 0.615 - 0.09 = $0.525
    assert!((report.dollars_saved - 0.525).abs() < 1e-4);

    // Savings percentage: (0.525 / 0.615) * 100 = ~85.36%
    assert!(report.savings_percentage > 80.0);
    assert!(report.local_percentage > 80.0);

    let summary = tier.format_summary_table();
    assert!(summary.contains("TAGISAN HYBRID TIER TELEMETRY"));
    assert!(summary.contains("Cost Reduction"));
}

// ============================================================================
// 3. Red Team Auditor Tests
// ============================================================================

#[test]
fn test_hermes_redteam_auditor_probes_and_findings() {
    let auditor = HermesRedTeamAuditor::new();
    let sys_prompt = auditor.format_adversarial_system_prompt();
    assert!(sys_prompt.contains("Principal Adversarial Red-Team Auditor"));
    assert!(sys_prompt.contains("zero corporate refusal friction"));

    let code_sample = r#"
pub async fn transfer_funds(from: &str, to: &str, amount: u64) -> Result<()> {
    let bal = get_balance(from).await?;
    if bal >= amount {
        // TOCTOU hazard between check and deduct
        deduct_balance(from, amount).await?;
        add_balance(to, amount).await?;
    }
    Ok(())
}
"#;

    let probes = auditor.generate_probes(code_sample, &[RedTeamProbeCategory::All]);
    assert!(!probes.is_empty());
    assert!(probes.iter().any(|p| p.category == RedTeamProbeCategory::ConcurrencyAndRaces));
    assert!(probes.iter().any(|p| p.category == RedTeamProbeCategory::CryptographicAndTiming));
    assert!(probes.iter().any(|p| p.category == RedTeamProbeCategory::MemorySafetyAndFfi));

    let audit_mock_response = r#"
[
  {
    "id": "VULN-TOCTOU-001",
    "title": "Time-Of-Check Time-Of-Use Race in transfer_funds",
    "severity": "CRITICAL",
    "line_start": 3,
    "line_end": 7,
    "summary": "The check `bal >= amount` is evaluated without holding an exclusive lock on the account record, permitting concurrent double-spend transfers.",
    "exploit_scenario": "Spawn 10 concurrent async tasks attempting to transfer `bal` simultaneously before the first deduction completes.",
    "remediation": "Wrap balance query and deduction inside an atomic compare-and-swap or transactional mutex."
  }
]
"#;

    let findings = auditor.parse_findings(audit_mock_response);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].id, "VULN-TOCTOU-001");
    assert_eq!(findings[0].severity, RedTeamSeverity::Critical);
    assert!(findings[0].vulnerability_summary.contains("double-spend"));
    assert!(findings[0].exploit_scenario.contains("concurrent async tasks"));

    let poc = auditor.generate_exploit_poc(&findings[0]);
    assert!(poc.contains("ADVERSARIAL REPRODUCTION PROOF-OF-CONCEPT"));
    assert!(poc.contains("test_reproduce_vuln_toctou_001"));
}

// ============================================================================
// 4. Preset Registration & Retrieval Tests
// ============================================================================

#[test]
fn test_hermes_presets_registration() {
    let presets = all_presets();

    let hermes_agent = presets.iter().find(|p| p.name == "hermes-agent");
    assert!(hermes_agent.is_some(), "hermes-agent preset not found in all_presets()");
    let ha = hermes_agent.unwrap();
    assert!(ha.tools.contains(&"read_file".to_string()));
    assert!(ha.tools.contains(&"write_file".to_string()));
    assert!(ha.tools.contains(&"run_command".to_string()));

    let hermes_redteam = presets.iter().find(|p| p.name == "hermes-redteam");
    assert!(hermes_redteam.is_some(), "hermes-redteam preset not found in all_presets()");
    let hr = hermes_redteam.unwrap();
    assert!(hr.tools.contains(&"read_file".to_string()));
    assert!(hr.tools.contains(&"run_command".to_string()));

    let hermes_workhorse = presets.iter().find(|p| p.name == "hermes-workhorse");
    assert!(hermes_workhorse.is_some(), "hermes-workhorse preset not found in all_presets()");

    // Test find_preset lookup with variations
    assert!(find_preset("hermes-agent").is_some());
    assert!(find_preset("hermes_agent").is_some());
    assert!(find_preset("HERMES-REDTEAM").is_some());
    assert!(find_preset("hermes-workhorse").is_some());
}

// ============================================================================
// 5. REPL /hermes Command Tests
// ============================================================================

#[test]
fn test_hermes_repl_command_parsing() {
    assert_eq!(
        tagisan::swarm::repl::InteractiveRepl::parse_command("/hermes status"),
        ReplCommand::Hermes("status".to_string())
    );
    assert_eq!(
        tagisan::swarm::repl::InteractiveRepl::parse_command("/hermes redteam src/lib.rs"),
        ReplCommand::Hermes("redteam src/lib.rs".to_string())
    );
    assert_eq!(
        tagisan::swarm::repl::InteractiveRepl::parse_command("/hermes hybrid"),
        ReplCommand::Hermes("hybrid".to_string())
    );
    assert_eq!(
        tagisan::swarm::repl::InteractiveRepl::parse_command("/hermes parse <tool_call>...</tool_call>"),
        ReplCommand::Hermes("parse <tool_call>...</tool_call>".to_string())
    );
    assert_eq!(
        tagisan::swarm::repl::InteractiveRepl::parse_command("/hermes"),
        ReplCommand::Hermes(String::new())
    );
}
