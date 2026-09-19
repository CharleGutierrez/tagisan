use tagisan::ecc::{all_presets, find_preset, EccSkill};
use tagisan::tools::ToolRegistry;

#[test]
fn test_all_10_ms_skills_exist_and_parse() {
    let expected_skills = vec![
        ("power-platform-alm-pro-max", "Dataverse", "PCF"),
        ("dotnet-modern-enterprise-pro-max", "Span", "Native AOT"),
        ("copilot-studio-agent-architect", "declarativeAgent.json", "OpenAPI"),
        ("fabric-onelake-architect-pro-max", "OneLake", "Direct Lake"),
        ("entra-zero-trust-guardian", "Workload Identity", "CAE"),
        ("sentinel-defender-soar-pro-max", "KQL", "ASIM"),
        ("azure-bicep-landing-zone-pro-max", "Bicep", "Landing Zone"),
        ("purview-data-governor-pro-max", "Sensitivity Label", "DLP"),
        ("m365-teams-platform-pro-max", "Graph", "Retry-After"),
        ("semantic-kernel-orchestrator-pro-max", "Semantic Kernel", "KernelFunction"),
    ];

    for (skill_name, keyword1, keyword2) in expected_skills {
        let skill_path = format!("assets/skills/{}/SKILL.md", skill_name);
        let skill = EccSkill::from_file(&skill_path)
            .unwrap_or_else(|e| panic!("Failed to load skill from {}: {:?}", skill_path, e));

        assert_eq!(skill.name, skill_name, "Skill name mismatch in {}", skill_path);
        assert!(!skill.description.trim().is_empty(), "Skill description empty in {}", skill_path);
        assert!(!skill.instructions.trim().is_empty(), "Skill instructions empty in {}", skill_path);

        assert!(
            skill.instructions.contains(keyword1),
            "Skill {} missing expected keyword '{}'",
            skill_name,
            keyword1
        );
        assert!(
            skill.instructions.contains(keyword2),
            "Skill {} missing expected keyword '{}'",
            skill_name,
            keyword2
        );
        assert!(
            skill.instructions.contains("Strict Invariants"),
            "Skill {} missing Strict Invariants section",
            skill_name
        );
    }
}

#[test]
fn test_ms_ecc_presets_registered_and_configured() {
    let presets = all_presets();
    assert!(presets.len() >= 11, "Must have at least 11 presets, found {}", presets.len());

    // 1. ms-cloud-architect
    let ms_arch = find_preset("ms-cloud-architect").expect("ms-cloud-architect preset missing");
    assert!(ms_arch.tools.contains(&"copilot_bicep".to_string()));
    assert!(ms_arch.tools.contains(&"copilot_access".to_string()));
    assert!(ms_arch.system_prompt.contains("Enterprise Architecture"));

    // 2. power-platform-architect
    let pp_arch = find_preset("power-platform-architect").expect("power-platform-architect preset missing");
    assert!(pp_arch.tools.contains(&"copilot_dataverse_sync".to_string()));
    assert!(pp_arch.tools.contains(&"copilot_power_automate".to_string()));
    assert!(pp_arch.system_prompt.contains("Dataverse Solution ALM"));

    // 3. dotnet-enterprise-architect
    let net_arch = find_preset("dotnet-enterprise-architect").expect("dotnet-enterprise-architect preset missing");
    assert!(net_arch.tools.contains(&"copilot_vscode".to_string()));
    assert!(net_arch.system_prompt.contains("Zero-Allocation Hot Paths"));

    // 4. entra-identity-guardian
    let entra_guard = find_preset("entra-identity-guardian").expect("entra-identity-guardian preset missing");
    assert!(entra_guard.tools.contains(&"copilot_workload_identity".to_string()));
    assert!(entra_guard.tools.contains(&"copilot_cae_handler".to_string()));
    assert!(entra_guard.system_prompt.contains("Secretless Workload Identity"));

    // 5. sentinel-defender-hunter
    let sec_hunter = find_preset("sentinel-defender-hunter").expect("sentinel-defender-hunter preset missing");
    assert!(sec_hunter.tools.contains(&"copilot_sentinel_audit".to_string()));
    assert!(sec_hunter.tools.contains(&"copilot_defender".to_string()));
    assert!(sec_hunter.system_prompt.contains("High-Performance KQL"));
}

#[tokio::test]
async fn test_copilot_enterprise_tools_in_registry() {
    let registry = ToolRegistry::with_builtins();

    let expected_tools = [
        "copilot_teams_post",
        "copilot_dataverse_sync",
        "copilot_power_automate",
        "copilot_bicep",
        "copilot_defender",
        "copilot_purview_guard",
        "copilot_sentinel_audit",
        "copilot_fabric_query",
        "copilot_workload_identity",
        "copilot_cae_handler",
        "copilot_obo_exchange",
        "copilot_jwe_decrypt",
        "copilot_sharepoint_crawler",
    ];

    for tool_name in expected_tools {
        assert!(
            registry.contains(tool_name),
            "Tool '{}' must be registered in ToolRegistry::with_builtins()",
            tool_name
        );
    }
}

#[test]
fn test_notification_suppression_and_anti_screen_clutter() {
    use tagisan::notify::{
        handle_notify_command, is_desktop_suppressed, set_desktop_enabled, NotifyAction,
    };

    // 1. When desktop is disabled, is_desktop_suppressed must be true
    set_desktop_enabled(false);
    assert!(is_desktop_suppressed(), "Desktop notifications must be recognized as suppressed");

    // 2. Test status handler
    let res = handle_notify_command(NotifyAction::Status);
    assert!(res.is_ok(), "Status command must succeed");

    // 3. Test clear history handler
    let res = handle_notify_command(NotifyAction::Clear);
    assert!(res.is_ok(), "Clear command must succeed");

    // 4. Test mute handler
    let res = handle_notify_command(NotifyAction::Mute);
    assert!(res.is_ok(), "Mute command must succeed");
    assert!(is_desktop_suppressed(), "Desktop notifications must remain muted after Mute");
}

