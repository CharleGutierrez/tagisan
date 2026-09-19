//! Brutal Integration Tests for GitHub Planning with Files in Tagisan (TGS)

use std::path::Path;
use tagisan::gh_plan::{
    GhBlastRadiusAnalyzer, GhBlastRiskTier, GhFileGovernor, GhPlanFileOp, GhPlanIssue,
    GhPlanManager, GhPlanManifest, GhPrSynthesizer,
};
use tagisan::ecc::{find_preset, all_presets};
use tagisan::swarm::repl::{InteractiveRepl, ReplCommand};

#[test]
fn test_gh_plan_issue_and_manifest_creation() {
    let mut issue = GhPlanIssue::default();
    issue.number = 142;
    issue.title = "Implement High-Performance Scraper".to_string();

    let mut manifest = GhPlanManifest::new(issue);
    assert_eq!(manifest.issue.number, 142);
    assert_eq!(manifest.progress_pct(), 0.0);
    assert!(!manifest.is_all_completed());

    manifest.add_file("src/reach.rs", GhPlanFileOp::Create, "Scraper core", vec!["Memory bounded"]);
    manifest.add_file("src/lib.rs", GhPlanFileOp::Modify, "Expose reach module", vec!["Zero compile warnings"]);

    manifest.add_task("Create reach.rs", "src/reach.rs", Some("cargo check"));
    manifest.add_task("Expose in lib.rs", "src/lib.rs", Some("cargo test"));

    assert_eq!(manifest.affected_files.len(), 2);
    assert_eq!(manifest.tasks.len(), 2);

    // Mark task 1 complete
    let updated = manifest.mark_completed(1, Some("a1b2c3d"));
    assert!(updated);
    assert_eq!(manifest.progress_pct(), 50.0);
    assert!(!manifest.is_all_completed());

    // Mark task 2 complete
    manifest.mark_completed(2, Some("e4f5g6h"));
    assert_eq!(manifest.progress_pct(), 100.0);
    assert!(manifest.is_all_completed());
}

#[test]
fn test_gh_plan_markdown_serialization_and_deserialization() {
    let mut issue = GhPlanIssue::default();
    issue.number = 200;
    issue.title = "Refactor Auth Middleware".to_string();

    let mut manifest = GhPlanManifest::new(issue);
    manifest.add_file("src/auth/token.rs", GhPlanFileOp::Modify, "Token validation", vec!["Constant time comparison"]);
    manifest.add_task("Patch timing leak", "src/auth/token.rs", None);

    let md = manifest.to_markdown();
    assert!(md.contains("# Plan: Issue #200 - Refactor Auth Middleware"));
    assert!(md.contains("| `MODIFY` | `src/auth/token.rs` |"));
    assert!(md.contains("- [ ] **Task 1:** Patch timing leak"));

    // Parse back
    let parsed = GhPlanManifest::parse_markdown(&md, 200).expect("Failed to parse markdown plan");
    assert_eq!(parsed.issue.number, 200);
    assert_eq!(parsed.affected_files.len(), 1);
    assert_eq!(parsed.affected_files[0].path, "src/auth/token.rs");
    assert_eq!(parsed.affected_files[0].op, GhPlanFileOp::Modify);
    assert_eq!(parsed.tasks.len(), 1);
    assert!(!parsed.tasks[0].completed);
}

#[test]
fn test_gh_file_governor_whitelist_enforcement() {
    let mut manifest = GhPlanManifest::new(GhPlanIssue::default());
    manifest.add_file("src/reach.rs", GhPlanFileOp::Create, "New file", vec![]);
    manifest.add_file("src/lib.rs", GhPlanFileOp::Modify, "Exports", vec![]);

    let governor = GhFileGovernor::new(&manifest);

    // Permitted file access
    assert!(governor.validate_file_access(Path::new("src/reach.rs"), GhPlanFileOp::Create).is_ok());
    assert!(governor.validate_file_access(Path::new("./src/lib.rs"), GhPlanFileOp::Modify).is_ok());

    // Unauthorized file access should trigger PlanViolation error
    let rogue_res = governor.validate_file_access(Path::new("Cargo.lock"), GhPlanFileOp::Modify);
    assert!(rogue_res.is_err(), "Governor must block unauthorized writes");
    let err_msg = rogue_res.unwrap_err().to_string();
    assert!(err_msg.contains("Plan Violation"));
    assert!(err_msg.contains("Cargo.lock"));
}

#[test]
fn test_gh_blast_radius_analyzer() {
    // 1. Safe leaf module
    let mut safe_manifest = GhPlanManifest::new(GhPlanIssue::default());
    safe_manifest.add_file("tests/sample_test.rs", GhPlanFileOp::Create, "Unit test", vec![]);
    let (risk_low, _) = GhBlastRadiusAnalyzer::analyze(&safe_manifest);
    assert_eq!(risk_low, GhBlastRiskTier::Low);

    // 2. Critical infrastructural module
    let mut root_manifest = GhPlanManifest::new(GhPlanIssue::default());
    root_manifest.add_file("src/lib.rs", GhPlanFileOp::Modify, "Root exports", vec![]);
    let (risk_crit, warnings) = GhBlastRadiusAnalyzer::analyze(&root_manifest);
    assert_eq!(risk_crit, GhBlastRiskTier::Critical);
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("Root infrastructural file"));
}

#[test]
fn test_gh_pr_synthesizer() {
    let mut manifest = GhPlanManifest::new(GhPlanIssue::default());
    manifest.issue.number = 404;
    manifest.issue.title = "Fix Distributed Race Condition".to_string();
    manifest.add_file("src/engine/runner.rs", GhPlanFileOp::Modify, "Lock ordering", vec![]);
    manifest.add_task("Apply atomic CAS", "src/engine/runner.rs", None);

    // Conventional commit message
    let commit = GhPrSynthesizer::format_commit(&manifest, &manifest.tasks[0]);
    assert_eq!(commit, "feat(engine): Apply atomic CAS (refs #404)");

    // Pull request body
    let pr_body = GhPrSynthesizer::synthesize_pr(&manifest, "test result: ok. 12 passed.");
    assert!(pr_body.contains("## Summary (Fixes #404)"));
    assert!(pr_body.contains("### 📁 File Mutation Matrix"));
    assert!(pr_body.contains("### 💥 Blast Radius Assessment"));
    assert!(pr_body.contains("### 🛡️ Hermes Red-Team Audit"));
    assert!(pr_body.contains("test result: ok. 12 passed."));
}

#[test]
fn test_gh_plan_manager_save_and_load() {
    let mut manifest = GhPlanManifest::new(GhPlanIssue::default());
    manifest.issue.number = 999;
    manifest.issue.title = "Temporary Test Plan".to_string();
    manifest.add_file("src/test.rs", GhPlanFileOp::Create, "Test file", vec![]);

    let saved_path = GhPlanManager::save(&manifest).expect("Save should succeed");
    assert!(saved_path.exists());

    let loaded = GhPlanManager::load(999).expect("Load should succeed");
    assert_eq!(loaded.issue.number, 999);
    assert_eq!(loaded.affected_files.len(), 1);

    // Clean up
    let _ = std::fs::remove_file(saved_path);
}

#[test]
fn test_repl_plan_commands() {
    assert_eq!(
        InteractiveRepl::parse_command("/plan fetch 142"),
        ReplCommand::Plan("fetch 142".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/plan status"),
        ReplCommand::Plan("status".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/plan pr"),
        ReplCommand::Plan("pr".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/plan help"),
        ReplCommand::Plan("help".to_string())
    );
}

#[test]
fn test_ecc_github_planner_preset() {
    let all = all_presets();
    assert!(all.iter().any(|p| p.name == "github-planner"));

    let preset = find_preset("github-planner").expect("github-planner preset must be registered");
    assert_eq!(preset.name, "github-planner");
    assert!(preset.tools.contains(&"read_file".to_string()));
    assert!(preset.tools.contains(&"write_file".to_string()));
    assert!(preset.system_prompt.contains("GitHub Planning Architect"));
}
