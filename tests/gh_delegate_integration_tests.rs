//! Brutal Integration Tests for GitHub Delegate-Skills in Tagisan (TGS)

use std::path::Path;
use tagisan::gh_delegate::{
    GhActionsCiWatcher, GhDelegateTask, GhSkillJitLoader, GhSkillManifest, GhSkillScope,
};
use tagisan::ecc::{find_preset, all_presets};
use tagisan::swarm::repl::{InteractiveRepl, ReplCommand};

#[test]
fn test_gh_skill_manifest_frontmatter_parsing() {
    let skill_md = r#"---
name: rust-borrow-fixer
description: Ephemeral specialist for resolving borrow checker and lifetime errors
max_tokens: 8192
verification_cmd: cargo check
tools:
  - read_file
  - write_file
  - cargo_check
target_files:
  - src/reach.rs
  - src/lib.rs
---

# Rust Borrow Fixer Instructions
When delegated a task, parse rustc errors and apply minimal borrow lifetime annotations.
"#;

    let manifest = GhSkillJitLoader::parse_skill_markdown(skill_md).expect("Failed to parse skill markdown");
    assert_eq!(manifest.name, "rust-borrow-fixer");
    assert_eq!(manifest.description, "Ephemeral specialist for resolving borrow checker and lifetime errors");
    assert_eq!(manifest.max_tokens, 8192);
    assert_eq!(manifest.verification_cmd.as_deref(), Some("cargo check"));
    assert_eq!(manifest.tools, vec!["read_file", "write_file", "cargo_check"]);
    assert_eq!(manifest.target_files, vec!["src/reach.rs", "src/lib.rs"]);
    assert_eq!(manifest.scope, GhSkillScope::ReadOnlyWorkspaceTargetWrite);
}

#[test]
fn test_gh_skill_frontmatter_missing_name_fails() {
    let bad_md = r#"---
description: Missing name attribute
tools:
  - read_file
---
Some body text
"#;
    let res = GhSkillJitLoader::parse_skill_markdown(bad_md);
    assert!(res.is_err(), "Manifest with missing name must fail validation");
}

#[test]
fn test_gh_skill_jit_loader_discovery() {
    let discovered = GhSkillJitLoader::discover_skills(Path::new("assets/skills")).expect("Failed to discover skills");
    assert!(!discovered.is_empty(), "Must discover skills in assets/skills");
    
    // Check for our integrated skills
    let skill_names: Vec<&str> = discovered.keys().map(|k| k.as_str()).collect();
    println!("Discovered skills: {:?}", skill_names);
    assert!(
        skill_names.contains(&"hermes-agent-integration")
            || skill_names.contains(&"agent-reach-integration")
            || skill_names.contains(&"github-planning-with-files")
            || skill_names.contains(&"memory-governor-anti-freeze"),
        "Expected integrated skills to be present in assets/skills"
    );
}

#[test]
fn test_gh_delegate_path_traversal_sandbox_violation() {
    let loader = GhSkillJitLoader::new();
    let mut skill = GhSkillManifest::default();
    skill.target_files = vec!["../../etc/shadow".to_string()];

    let task = GhDelegateTask {
        id: "del-malicious-001".to_string(),
        parent_goal: "Break out of sandbox".to_string(),
        specialist_agent: "rogue-agent".to_string(),
        skill,
        task_instruction: "Read sensitive host file".to_string(),
        created_at: "2026-09-19T00:00:00Z".to_string(),
    };

    let res = loader.execute_delegation(&task);
    assert!(res.is_err(), "Sandbox must strictly reject path traversal");
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Path traversal attempt") || err_str.contains("Sandbox Violation"));
}

#[test]
fn test_gh_delegate_absolute_path_sandbox_violation() {
    let loader = GhSkillJitLoader::new();
    let mut skill = GhSkillManifest::default();
    skill.target_files = vec!["/var/log/syslog".to_string()];

    let task = GhDelegateTask {
        id: "del-malicious-002".to_string(),
        parent_goal: "Write to root system directory".to_string(),
        specialist_agent: "rogue-agent".to_string(),
        skill,
        task_instruction: "Corrupt system log".to_string(),
        created_at: "2026-09-19T00:00:00Z".to_string(),
    };

    let res = loader.execute_delegation(&task);
    assert!(res.is_err(), "Sandbox must strictly reject absolute paths");
    let err_str = res.unwrap_err().to_string();
    assert!(err_str.contains("Path traversal attempt") || err_str.contains("Sandbox Violation"));
}

#[test]
fn test_gh_delegate_task_execution_and_heap_trim() {
    let loader = GhSkillJitLoader::new();
    let mut skill = GhSkillManifest::default();
    skill.name = "test-refactor".to_string();
    skill.target_files = vec!["src/governor.rs".to_string()];
    skill.max_tokens = 2048;

    let task = GhDelegateTask {
        id: "del-clean-001".to_string(),
        parent_goal: "Optimize memory governor".to_string(),
        specialist_agent: "perf-engineer".to_string(),
        skill,
        task_instruction: "Audit libc::malloc_trim usage in HostMemoryGovernor".to_string(),
        created_at: "2026-09-19T00:00:00Z".to_string(),
    };

    let res = loader.execute_delegation(&task).expect("Delegation execution failed");
    assert_eq!(res.task_id, "del-clean-001");
    assert!(res.success);
    assert!(res.tokens_used > 0);
    assert!(res.tokens_used <= 2048);
    assert_eq!(res.modified_files, vec!["src/governor.rs"]);
    assert!(res.output.contains("perf-engineer"));
    // On Linux glibc, malloc_trim(0) returns true when memory is reclaimed/trimmed
    println!("Delegation result memory_trimmed: {}", res.memory_trimmed);
}

#[test]
fn test_gh_actions_ci_watcher_compiler_error() {
    let raw_ci_log = r#"
   Compiling tagisan v0.2.0 (/home/dyna/TGS Projects/tagisan)
error[E0425]: cannot find value `undefined_variable` in this scope
  --> src/reach.rs:120:5
   |
120 |     undefined_variable.crawl();
   |     ^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `tagisan` (lib) due to 1 previous error
"#;

    let report = GhActionsCiWatcher::parse_ci_log(raw_ci_log);
    assert!(report.is_some(), "CI watcher must parse rustc compiler diagnostic");
    let r = report.unwrap();
    assert_eq!(r.failing_file.as_deref(), Some("src/reach.rs"));
    assert_eq!(r.line_number, Some(120));
    assert!(r.error_diagnostic.contains("error[E0425]"));
    assert!(r.suggested_remediation.contains("src/reach.rs"));

    let pr_comment = GhActionsCiWatcher::format_pr_comment(&r);
    assert!(pr_comment.contains("Automated CI/CD Self-Healing Audit"));
    assert!(pr_comment.contains("`src/reach.rs`"));
}

#[test]
fn test_gh_actions_ci_watcher_clean_log() {
    let clean_log = r#"
   Compiling tagisan v0.2.0 (/home/dyna/TGS Projects/tagisan)
    Finished `test` profile [unoptimized] target(s) in 2.14s
     Running tests/gh_plan_integration_tests.rs (target/debug/deps/gh_plan_integration_tests-123)

running 8 tests
test test_gh_plan_issue_and_manifest_creation ... ok
test test_gh_blast_radius_analyzer ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
"#;

    let report = GhActionsCiWatcher::parse_ci_log(clean_log);
    assert!(report.is_none(), "Clean test log must not produce a failure report");
}

#[test]
fn test_github_delegator_preset_registered() {
    let presets = all_presets();
    let delegator = presets.iter().find(|p| p.name == "github-delegator");
    assert!(delegator.is_some(), "Preset 'github-delegator' must be registered");
    let d = delegator.unwrap();
    assert!(d.system_prompt.contains("GitHub delegate-skills") || d.description.contains("delegat"));

    let found = find_preset("github-delegator");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "github-delegator");
}

#[test]
fn test_repl_delegate_commands() {
    // 1. /delegate help
    assert_eq!(
        InteractiveRepl::parse_command("/delegate help"),
        ReplCommand::Delegate("help".to_string())
    );

    // 2. /delegate list
    assert_eq!(
        InteractiveRepl::parse_command("/delegate list"),
        ReplCommand::Delegate("list".to_string())
    );

    // 3. /delegate run
    assert_eq!(
        InteractiveRepl::parse_command("/delegate run rust-analyst src/lib.rs audit exports"),
        ReplCommand::Delegate("run rust-analyst src/lib.rs audit exports".to_string())
    );

    // 4. /delegate ci
    assert_eq!(
        InteractiveRepl::parse_command("/delegate ci error[E0308]: mismatched types --> src/lib.rs:42:10"),
        ReplCommand::Delegate("ci error[E0308]: mismatched types --> src/lib.rs:42:10".to_string())
    );
}
