//! # Brutal Verification Test Suite: Top Skills Wishlist & Systems Tools
//!
//! Validates:
//! 1. All 4 wishlist skills (`episodic-reflexion-vault`, `git-worktree-merge-arbiter`,
//!    `formal-invariant-prover`, `database-dba-query-optimizer`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections.
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `ReflexionVaultTool` recording, query relevance scoring, and listing past case-law.
//! 5. `GitWorktreeTool` isolated branch provisioning, command execution, diffing, atomic commit, and cleanup.
//! 6. Tool registration in `ToolRegistry::with_builtins()`.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{GitWorktreeTool, ReflexionVaultTool, ToolHandler, ToolRegistry};

// =========================================================================
// Pillar 1: Wishlist Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_episodic_reflexion_vault_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/episodic-reflexion-vault/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/episodic-reflexion-vault");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read episodic-reflexion-vault SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: episodic-reflexion-vault"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("episodic-memory"));
    assert!(raw_content.contains("reflexion-vault"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("compiler-panic"));
    assert!(raw_content.contains("preventative-invariant"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on episodic-reflexion-vault");
    assert_eq!(skill.name, "episodic-reflexion-vault");
    assert!(
        skill.description.contains("case law") || skill.description.contains("episodic memory"),
        "Description must reference case law or episodic memory: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Mandatory Pre-Debug Vault Query"),
        "Must contain Pre-Debug Vault Query invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Atomic Incident Recording"),
        "Must contain Atomic Incident Recording invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Generalization Over Situational Band-Aids"),
        "Must contain Generalization invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER"),
        "Must contain strict ALWAYS and NEVER directives"
    );
}

#[test]
fn test_git_worktree_merge_arbiter_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/git-worktree-merge-arbiter/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/git-worktree-merge-arbiter");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read git-worktree-merge-arbiter SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: git-worktree-merge-arbiter"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("git-worktree"));
    assert!(raw_content.contains("merge-arbiter"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("clean-tree-invariant"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on git-worktree-merge-arbiter");
    assert_eq!(skill.name, "git-worktree-merge-arbiter");
    assert!(
        skill.description.contains("worktree") || skill.description.contains("isolation"),
        "Description must reference worktree isolation"
    );
    assert!(
        skill.instructions.contains("Invariant 1: Clean Tree Preservation Invariant"),
        "Must contain Clean Tree Preservation Invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Pre-Commit Verification Gate"),
        "Must contain Pre-Commit Verification Gate invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Semantic AST Conflict Resolution"),
        "Must contain Semantic AST Conflict Resolution invariant"
    );
}

#[test]
fn test_formal_invariant_prover_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/formal-invariant-prover/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/formal-invariant-prover");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read formal-invariant-prover SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: formal-invariant-prover"));
    assert!(raw_content.contains("proptest"));
    assert!(raw_content.contains("kani"));
    assert!(raw_content.contains("inductive-invariants"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on formal-invariant-prover");
    assert_eq!(skill.name, "formal-invariant-prover");
    assert!(
        skill.instructions.contains("Invariant 1: Beyond Happy-Path Unit Testing"),
        "Must contain Beyond Happy-Path invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Exhaustive Boundary Singularities"),
        "Must contain Boundary Singularities invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Inductive Invariant & Property Axioms"),
        "Must contain Inductive Invariant & Property Axioms invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Bounded Model Checking & Panic-Freedom (Kani)"),
        "Must contain Kani Bounded Model Checking invariant"
    );
}

#[test]
fn test_database_dba_query_optimizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/database-dba-query-optimizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/database-dba-query-optimizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read database-dba-query-optimizer SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: database-dba-query-optimizer"));
    assert!(raw_content.contains("explain-analyze"));
    assert!(raw_content.contains("create-index-concurrently"));
    assert!(raw_content.contains("zero-downtime-ddl"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on database-dba-query-optimizer");
    assert_eq!(skill.name, "database-dba-query-optimizer");
    assert!(
        skill.instructions.contains("Invariant 1: Mandatory EXPLAIN ANALYZE Verification"),
        "Must contain Mandatory EXPLAIN ANALYZE invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Optimal Index Architecture"),
        "Must contain Optimal Index Architecture invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Zero-Downtime Non-Blocking DDL"),
        "Must contain Zero-Downtime Non-Blocking DDL invariant"
    );
    assert!(
        skill.instructions.contains("CREATE INDEX CONCURRENTLY"),
        "Must reference CREATE INDEX CONCURRENTLY"
    );
}

#[test]
fn test_built_in_catalog_registration_and_aliases() {
    let all_skills = all_built_in_skills();

    for name in &[
        "episodic-reflexion-vault",
        "git-worktree-merge-arbiter",
        "formal-invariant-prover",
        "database-dba-query-optimizer",
    ] {
        let found = all_skills.iter().find(|s| s.name == *name);
        assert!(found.is_some(), "Skill '{}' must be present in all_built_in_skills()", name);

        let retrieved = find_built_in_skill(name);
        assert!(retrieved.is_some(), "find_built_in_skill must resolve '{}'", name);
    }

    // Verify aliases
    assert!(find_built_in_skill("reflexion-vault").is_some(), "Alias 'reflexion-vault' must resolve");
    assert!(find_built_in_skill("git-worktree").is_some(), "Alias 'git-worktree' must resolve");
    assert!(find_built_in_skill("proptest").is_some(), "Alias 'proptest' must resolve");
    assert!(find_built_in_skill("query-optimizer").is_some(), "Alias 'query-optimizer' must resolve");
}

// =========================================================================
// Pillar 2: ReflexionVaultTool Real Systems Tests
// =========================================================================

#[tokio::test]
async fn test_reflexion_vault_tool_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_refl_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let vault_file = temp_dir.join("test_reflexions.json");
    if vault_file.exists() {
        let _ = fs::remove_file(&vault_file);
    }

    let tool = ReflexionVaultTool::new().with_path(&vault_file);
    assert_eq!(tool.name(), "reflexion_vault");

    // 1. Initial list should be empty
    let list_res = tool.execute(json!({"action": "list"})).await.expect("List failed");
    let list_val: Value = serde_json::from_str(&list_res).expect("Valid JSON");
    assert_eq!(list_val["count"], 0);

    // 2. Record first reflexion: borrow-checker incident
    let record1_res = tool
        .execute(json!({
            "action": "record",
            "error_signature": "cannot borrow `*self` as mutable, as it is also borrowed as immutable (E0502)",
            "root_cause": "Holding immutable reference from self.items.get() while calling mutating helper method on self.",
            "fix_applied": "Scoped the immutable lookup into a narrow block before calling the mutating method.",
            "preventative_invariant": "ALWAYS drop read references or scope borrows before calling mutating receiver methods.",
            "tags": ["rust", "borrowck", "lifetimes"]
        }))
        .await
        .expect("Record 1 failed");

    let rec1_val: Value = serde_json::from_str(&record1_res).expect("Valid JSON");
    assert_eq!(rec1_val["status"], "recorded");
    let id1 = rec1_val["id"].as_str().expect("id must be string");
    assert!(id1.starts_with("refl-"));

    // 3. Record second reflexion: database lock incident
    let record2_res = tool
        .execute(json!({
            "action": "record",
            "error_signature": "PostgreSQL deadlock / relation lock timeout (55P03)",
            "root_cause": "Executed blocking CREATE INDEX on active transactions table without CONCURRENTLY.",
            "fix_applied": "Re-executed using CREATE INDEX CONCURRENTLY with SET lock_timeout = '2s'.",
            "preventative_invariant": "ALWAYS use CREATE INDEX CONCURRENTLY and strict lock_timeout on production databases.",
            "tags": ["database", "postgres", "indexing", "ddl"]
        }))
        .await
        .expect("Record 2 failed");

    let rec2_val: Value = serde_json::from_str(&record2_res).expect("Valid JSON");
    assert_eq!(rec2_val["status"], "recorded");

    // 4. List must now contain 2 entries
    let list2_res = tool.execute(json!({"action": "list"})).await.expect("List failed");
    let list2_val: Value = serde_json::from_str(&list2_res).expect("Valid JSON");
    assert_eq!(list2_val["count"], 2);

    // 5. Query for borrow-checker error
    let query_rust_res = tool
        .execute(json!({
            "action": "query",
            "query": "borrow mutable E0502"
        }))
        .await
        .expect("Query failed");

    let q_rust_val: Value = serde_json::from_str(&query_rust_res).expect("Valid JSON");
    assert!(q_rust_val["count"].as_u64().unwrap_or(0) >= 1);
    let top_rust = &q_rust_val["results"][0];
    assert!(top_rust["error_signature"].as_str().unwrap().contains("E0502"));
    assert!(top_rust["preventative_invariant"].as_str().unwrap().contains("ALWAYS"));

    // 6. Query for database index
    let query_db_res = tool
        .execute(json!({
            "action": "query",
            "query": "CREATE INDEX CONCURRENTLY postgres"
        }))
        .await
        .expect("Query failed");

    let q_db_val: Value = serde_json::from_str(&query_db_res).expect("Valid JSON");
    assert!(q_db_val["count"].as_u64().unwrap_or(0) >= 1);
    let top_db = &q_db_val["results"][0];
    assert!(top_db["root_cause"].as_str().unwrap().contains("CREATE INDEX"));

    // 7. Query with zero matches
    let query_empty_res = tool
        .execute(json!({
            "action": "query",
            "query": "unrelated_quantum_physics_keyword_xyz"
        }))
        .await
        .expect("Query failed");
    let q_empty_val: Value = serde_json::from_str(&query_empty_res).expect("Valid JSON");
    assert_eq!(q_empty_val["count"], 0);

    // Cleanup test vault
    let _ = fs::remove_file(&vault_file);
    let _ = fs::remove_dir(&temp_dir);
}

// =========================================================================
// Pillar 3: GitWorktreeTool Real Systems Tests
// =========================================================================

#[tokio::test]
async fn test_git_worktree_tool_lifecycle() {
    // Provision a clean temporary git repository to test worktree isolation safely
    let temp_repo = std::env::temp_dir().join(format!("tgs_test_repo_{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
    let _ = fs::create_dir_all(&temp_repo);

    // git init
    let init_out = Command::new("git")
        .args(["init"])
        .current_dir(&temp_repo)
        .output()
        .expect("git init must succeed");
    assert!(init_out.status.success());

    // git config user.name / email
    let _ = Command::new("git").args(["config", "user.name", "Tagisan Agent"]).current_dir(&temp_repo).output();
    let _ = Command::new("git").args(["config", "user.email", "agent@tagisan.ai"]).current_dir(&temp_repo).output();

    // Initial commit so HEAD exists
    let readme_path = temp_repo.join("README.md");
    fs::write(&readme_path, "# Initial Repo Root\n").expect("write README");
    let _ = Command::new("git").args(["add", "README.md"]).current_dir(&temp_repo).output();
    let commit_init = Command::new("git").args(["commit", "-m", "Initial commit"]).current_dir(&temp_repo).output().expect("commit");
    assert!(commit_init.status.success(), "Initial commit must succeed");

    let tool = GitWorktreeTool::new().with_working_dir(&temp_repo);
    assert_eq!(tool.name(), "git_worktree");

    let branch_name = format!("sandbox/test-{}", std::process::id());
    let worktree_id = format!("wt-{}", std::process::id());

    // 1. Create isolated worktree
    let create_res = tool
        .execute(json!({
            "action": "create",
            "repo_path": temp_repo.to_str().unwrap(),
            "branch": branch_name,
            "worktree_id": worktree_id
        }))
        .await
        .expect("Create worktree failed");

    let create_val: Value = serde_json::from_str(&create_res).expect("Valid JSON");
    assert_eq!(create_val["status"], "created");
    let wt_path = PathBuf::from(create_val["worktree_path"].as_str().expect("worktree_path"));
    assert!(wt_path.is_dir(), "Worktree directory must exist on disk: {}", wt_path.display());

    // 2. Run command inside worktree
    let run_res = tool
        .execute(json!({
            "action": "run",
            "worktree_id": worktree_id,
            "command": "echo 'autonomous experiment' > experiment.txt"
        }))
        .await
        .expect("Run command failed");

    let run_val: Value = serde_json::from_str(&run_res).expect("Valid JSON");
    assert_eq!(run_val["exit_code"], 0);

    // Verify file exists inside worktree and NOT in root repo
    assert!(wt_path.join("experiment.txt").is_file(), "File must exist in worktree");
    assert!(!temp_repo.join("experiment.txt").exists(), "File must NOT pollute root repo");

    // 3. Diff inspection
    let diff_res = tool
        .execute(json!({
            "action": "diff",
            "worktree_id": worktree_id
        }))
        .await
        .expect("Diff failed");

    let diff_val: Value = serde_json::from_str(&diff_res).expect("Valid JSON");
    assert_eq!(diff_val["status"], "success");

    // 4. Atomic commit inside worktree
    let commit_res = tool
        .execute(json!({
            "action": "commit",
            "worktree_id": worktree_id,
            "message": "feat: add isolated experiment"
        }))
        .await
        .expect("Commit failed");

    let commit_val: Value = serde_json::from_str(&commit_res).expect("Valid JSON");
    assert_eq!(commit_val["status"], "committed");
    assert!(!commit_val["commit"].as_str().unwrap().is_empty());

    // 5. List active worktrees
    let list_res = tool.execute(json!({"action": "list"})).await.expect("List failed");
    let list_val: Value = serde_json::from_str(&list_res).expect("Valid JSON");
    assert_eq!(list_val["active_worktrees"].as_array().unwrap().len(), 1);

    // 6. Cleanup worktree
    let cleanup_res = tool
        .execute(json!({
            "action": "cleanup",
            "worktree_id": worktree_id
        }))
        .await
        .expect("Cleanup failed");

    let cleanup_val: Value = serde_json::from_str(&cleanup_res).expect("Valid JSON");
    assert_eq!(cleanup_val["status"], "cleaned_up");
    assert!(!wt_path.exists(), "Worktree path must be removed after cleanup");

    // Remove temporary test repository
    let _ = fs::remove_dir_all(&temp_repo);
}

// =========================================================================
// Pillar 4: ToolRegistry with_builtins Integration
// =========================================================================

#[test]
fn test_tool_registry_contains_wishlist_builtins() {
    let registry = ToolRegistry::with_builtins();
    assert!(registry.has_tool("git_worktree"), "ToolRegistry must contain 'git_worktree'");
    assert!(registry.has_tool("reflexion_vault"), "ToolRegistry must contain 'reflexion_vault'");

    let defs = registry.definitions();
    assert!(defs.iter().any(|d| d.name == "git_worktree"));
    assert!(defs.iter().any(|d| d.name == "reflexion_vault"));
}
