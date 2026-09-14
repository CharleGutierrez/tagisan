//! # Integration Test: Dynamic Skills Ingestion & MCP Resolution
//!
//! Validates:
//! 1. Dynamic skills in `.ecc/skills/` are resolvable by `find_built_in_skill`.
//! 2. Dynamic MCP configuration is discoverable by `McpConfig::discover_default`.
//! 3. Skill attributes, frontmatter, and invariants are preserved in the resolved skill.

use tagisan::ecc::skills::{find_built_in_skill, resolve_skill};
use tagisan::mcp::McpConfig;

#[test]
fn test_dynamic_skills_built_in_resolution() {
    // Test resolving one of the dynamic Tier 1 skills
    let brave_skill = find_built_in_skill("smithery-brave-search");
    assert!(brave_skill.is_some(), "smithery-brave-search must be resolvable");
    let skill = brave_skill.unwrap();
    assert_eq!(skill.name, "smithery-brave-search");
    assert!(skill.instructions.contains("ALWAYS"));
    assert!(skill.instructions.contains("NEVER"));
    assert!(skill.instructions.contains("MANDATORY"));
    assert!(skill.instructions.contains("STRICT_REJECT"));

    // Test resolving one of the dynamic Tier 3 low-level systems skills
    let io_uring_skill = find_built_in_skill("linux-io-uring-async-engine");
    assert!(io_uring_skill.is_some(), "linux-io-uring-async-engine must be resolvable");
    let s3 = io_uring_skill.unwrap();
    assert_eq!(s3.name, "linux-io-uring-async-engine");
    assert!(s3.instructions.contains("ZeroAmbientAuthority"));

    // Test resolving one of the dynamic Tier 4 formal verification skills
    let smt_skill = resolve_skill("smt-z3-qf-bv-bitvector", None);
    assert!(smt_skill.is_some(), "smt-z3-qf-bv-bitvector must be resolvable");
}

#[test]
fn test_dynamic_mcp_config_discovery() {
    let discovery = McpConfig::discover_default();
    assert!(discovery.is_some(), "Default MCP configuration must be discoverable");
    let (path, config) = discovery.unwrap();
    assert!(path.is_file(), "Discovered path must exist");
    assert!(
        config.mcp_servers.contains_key("smithery-brave-search")
            || config.mcp_servers.contains_key("smithery-exa-semantic-search"),
        "Dynamic MCP servers must be present in discovered config"
    );
}
