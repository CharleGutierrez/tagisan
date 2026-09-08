use crate::error::{Result, TagisanError};
use std::fs;
use std::path::Path;
use tracing::warn;

/// A reusable engineering skill defined according to the ECC specification
#[derive(Debug, Clone, PartialEq)]
pub struct EccSkill {
    pub name: String,
    pub description: String,
    pub instructions: String,
}

impl EccSkill {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            instructions: instructions.into(),
        }
    }

    /// Parse a SKILL.md file with YAML frontmatter
    pub fn parse(content: &str) -> Result<Self> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return Err(TagisanError::Execution(
                "Invalid ECC skill format: missing leading '---' frontmatter delimiter".to_string(),
            ));
        }

        let rest = &trimmed[3..];
        let end_idx = rest.find("\n---").or_else(|| rest.find("\r\n---")).ok_or_else(|| {
            TagisanError::Execution(
                "Invalid ECC skill format: missing closing '---' frontmatter delimiter".to_string(),
            )
        })?;

        let frontmatter_str = &rest[..end_idx];
        let after_close = &rest[end_idx..];
        let delim_pos = after_close.find("---").unwrap_or(0);
        let mut body_start_offset = delim_pos + 3;
        if let Some(nl_pos) = after_close[body_start_offset..].find('\n') {
            body_start_offset += nl_pos + 1;
        } else {
            body_start_offset = after_close.len();
        }
        let body = after_close.get(body_start_offset..).unwrap_or("").trim().to_string();

        let mut name = String::new();
        let mut description = String::new();

        for line in frontmatter_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim().trim_matches('"').trim_matches('\'').trim();

                match key.as_str() {
                    "name" => name = val.to_string(),
                    "description" => description = val.to_string(),
                    _ => {}
                }
            }
        }

        if name.trim().is_empty() {
            return Err(TagisanError::Execution(
                "Invalid ECC skill format: missing 'name' field in frontmatter".to_string(),
            ));
        }

        Ok(Self {
            name,
            description,
            instructions: body,
        })
    }

    /// Load an ECC skill from a file on disk
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read ECC skill file '{}': {}",
                path_ref.display(),
                e
            ))
        })?;
        Self::parse(&content)
    }
}

/// Return all built-in ECC engineering skills
pub fn all_built_in_skills() -> Vec<EccSkill> {
    vec![
        tdd_workflow(),
        security_review(),
        api_design(),
        verification_loop(),
    ]
}

/// Retrieve a built-in skill by name
pub fn find_built_in_skill(name: &str) -> Option<EccSkill> {
    let lower = name.to_lowercase().replace('_', "-");
    all_built_in_skills().into_iter().find(|s| s.name == lower)
}

/// 1. TDD Workflow Skill
pub fn tdd_workflow() -> EccSkill {
    EccSkill::new(
        "tdd-workflow",
        "Test-Driven Development discipline: write failing assertions and reproduction cases before code",
        r#"# ECC TDD Workflow

## Phase 1: Test Formulation
- Always formulate concrete, failing unit tests before authoring implementation logic.
- Identify the public interface, inputs, expected outputs, and error variants.
- Write tests that capture boundary limits (0, 1, MAX, empty, null/None, unicode).

## Phase 2: Minimal Implementation
- Write the minimum amount of code required to make the tests pass.
- Resist premature optimization during greening phase.

## Phase 3: Refactor
- Eliminate duplicate code (DRY).
- Improve naming and readability without altering observable external behavior.
- Ensure all tests continue to pass.
"#,
    )
}

/// 2. Security Review Skill
pub fn security_review() -> EccSkill {
    EccSkill::new(
        "security-review",
        "Rigorous offensive and defensive threat modeling and vulnerability scanning",
        r#"# ECC Security Review

## Threat Checklist
1. Input Sanitization: Validate and sanitize all external inputs (CLI flags, file inputs, network bytes).
2. Injection Prevention: Avoid shell concatenation or string interpolation in command execution.
3. Concurrency Safety: Check for race conditions, deadlock cycles, and time-of-check to time-of-use (TOCTOU).
4. Secret Protection: Ensure API keys, tokens, and credentials are never logged or echoed to stdout.
5. Memory & Resource Safety: Validate array bounds, recursion depths, and allocation limits to prevent denial-of-service.
"#,
    )
}

/// 3. API Design Skill
pub fn api_design() -> EccSkill {
    EccSkill::new(
        "api-design",
        "Robust API design patterns emphasizing backward compatibility and intuitive ergonomic interfaces",
        r#"# ECC API Design

## Principles
1. Explicit over Implicit: Design function signatures where failure modes are represented in types (`Result`, `Option`).
2. Least Astonishment: Follow canonical idioms of the host programming language.
3. Extensibility: Use the builder pattern or options structs for functions with many parameters.
4. Documentation: Document every public struct, enum, and function with doc comments and usage examples.
"#,
    )
}

/// 4. Verification Loop Skill
pub fn verification_loop() -> EccSkill {
    EccSkill::new(
        "verification-loop",
        "Continuous verification, automated test runs, and regression monitoring",
        r#"# ECC Verification Loop

## Protocol
1. Baseline: Run the existing test suite to ensure an unpolluted baseline.
2. Reproduction: If fixing a bug, write a test that fails reliably without the fix.
3. Application: Apply the targeted, minimal fix.
4. Confirmation: Rerun tests to confirm the reproduction test passes AND zero regressions occur in existing tests.
"#,
    )
}

/// Discover and load all ECC skills from a directory (scanning both `*.md` and `<dir>/SKILL.md`)
pub fn load_skills_from_dir(dir: impl AsRef<Path>) -> Vec<EccSkill> {
    let mut skills = Vec::new();
    let dir_ref = dir.as_ref();

    if !dir_ref.is_dir() {
        return skills;
    }

    if let Ok(entries) = fs::read_dir(dir_ref) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match EccSkill::from_file(&path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => warn!("Failed to load skill from '{}': {}", path.display(), e),
                }
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.is_file() {
                    match EccSkill::from_file(&skill_md) {
                        Ok(skill) => skills.push(skill),
                        Err(e) => warn!("Failed to load skill from '{}': {}", skill_md.display(), e),
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Retrieve an ECC skill by name, checking built-in skills first, then an optional disk directory
pub fn resolve_skill(name: &str, custom_dir: Option<&Path>) -> Option<EccSkill> {
    if let Some(skill) = find_built_in_skill(name) {
        return Some(skill);
    }

    if let Some(dir) = custom_dir {
        let loaded = load_skills_from_dir(dir);
        let lower = name.to_lowercase().replace('_', "-");
        return loaded.into_iter().find(|s| s.name == lower);
    }

    None
}
