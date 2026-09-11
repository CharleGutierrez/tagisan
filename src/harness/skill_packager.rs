use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ecc::skills::EccSkill;
use crate::error::{Result, TagisanError};
use crate::harness::generator::GeneratedHarness;
use crate::harness::spec::HarnessSpec;

/// Packaged engineering skill ready for installation and discovery by SkillDispatcher
#[derive(Debug, Clone)]
pub struct PackagedSkill {
    pub name: String,
    pub skill_md_content: String,
    pub harness_files: GeneratedHarness,
}

impl PackagedSkill {
    /// Installs the skill into the target skills directory (typically .ecc/skills/<name>)
    pub fn install_to(&self, skills_root: &Path) -> Result<PathBuf> {
        let skill_dir = skills_root.join(&self.name);
        fs::create_dir_all(&skill_dir).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create skill directory '{}': {}",
                skill_dir.display(),
                e
            ))
        })?;

        // 1. Write SKILL.md
        let skill_md_path = skill_dir.join("SKILL.md");
        let mut skill_file = File::create(&skill_md_path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create SKILL.md at '{}': {}",
                skill_md_path.display(),
                e
            ))
        })?;
        skill_file
            .write_all(self.skill_md_content.as_bytes())
            .map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to write SKILL.md at '{}': {}",
                    skill_md_path.display(),
                    e
                ))
            })?;

        // 2. Write the executable CLI and test scripts inside the skill directory
        self.harness_files.write_to_dir(&skill_dir)?;

        // 3. Verify that the written SKILL.md parses cleanly via EccSkill
        EccSkill::from_file(&skill_md_path)?;

        Ok(skill_dir)
    }
}

/// Synthesizes standardized RFC-004 compliant SKILL.md packages
pub struct SkillPackager;

impl SkillPackager {
    /// Packages a synthesized CLI harness into an RFC-004 SKILL.md document
    pub fn package(spec: &HarnessSpec, generated: &GeneratedHarness) -> Result<PackagedSkill> {
        let skill_name = spec.name.to_lowercase().replace('_', "-");
        let skill_md_content = Self::generate_rfc004_skill_md(spec, generated)?;

        // Verify valid format
        EccSkill::parse(&skill_md_content)?;

        Ok(PackagedSkill {
            name: skill_name,
            skill_md_content,
            harness_files: generated.clone(),
        })
    }

    /// Generates RFC-004 compliant markdown with YAML frontmatter, Local Cheat Sheet, and Cloud Specification
    fn generate_rfc004_skill_md(spec: &HarnessSpec, generated: &GeneratedHarness) -> Result<String> {
        let mut md = String::new();

        // 1. Compute Triggers
        let mut triggers = Vec::new();
        triggers.push(spec.name.clone());
        triggers.push(format!("{} cli", spec.name));
        triggers.push(format!("{} harness", spec.name));
        for cmd in &spec.commands {
            triggers.push(format!("{} {}", spec.name, cmd.name));
            triggers.push(cmd.name.clone());
        }

        let description = format!(
            "Agent-native CLI harness for {}. Provides structured machine commands for {}. Triggers: {}.",
            spec.name,
            spec.commands
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            triggers.join(", ")
        );

        // 2. YAML Frontmatter
        md.push_str("---\n");
        md.push_str(&format!("name: {}\n", spec.name.to_lowercase().replace('_', "-")));
        md.push_str(&format!("description: \"{}\"\n", description.replace('"', "\\\"")));
        md.push_str("triggers:\n");
        for trig in &triggers {
            md.push_str(&format!("  - \"{}\"\n", trig.replace('"', "\\\"")));
        }
        md.push_str("---\n\n");

        // 3. Document Title
        md.push_str(&format!("# {}\n\n", spec.name));
        if !spec.description.is_empty() {
            md.push_str(&format!("{}\n\n", spec.description));
        }

        // 4. Section 1: Local LLM Cheat Sheet (<1,000 tokens)
        md.push_str("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]\n\n");
        md.push_str(&format!("#### CLI Binary: `python3 {}`\n\n", generated.cli_filename));
        md.push_str("- Invariant: Always pass `--json` to ensure stdout returns machine-readable JSON.\n");
        md.push_str("- Invariant: Exit code 0 indicates success. Non-zero indicates command error.\n");
        md.push_str("- Invariant: Never attempt to parse stderr for data output; stderr is strictly for diagnostics.\n\n");

        md.push_str("#### Command Overview:\n");
        for cmd in &spec.commands {
            let req_args: Vec<String> = cmd
                .arguments
                .iter()
                .filter(|a| a.required)
                .map(|a| a.cli_flag())
                .collect();
            let req_str = if req_args.is_empty() {
                "None".to_string()
            } else {
                req_args.join(", ")
            };
            md.push_str(&format!(
                "- `{}`: {} (Required: {})\n",
                cmd.name,
                if cmd.description.is_empty() { "Execute operation" } else { &cmd.description },
                req_str
            ));
        }
        md.push('\n');

        // 5. Section 2: Cloud Comprehensive Specification
        md.push_str("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]\n\n");
        md.push_str("#### Architectural Overview\n");
        md.push_str(&format!(
            "The `{}` CLI is a standalone, deterministic harness synthesized by Tagisan. \
            It interfaces directly with the underlying source code at `{}`.\n\n",
            spec.name,
            spec.source_path.display()
        ));

        md.push_str("#### Subcommands & Parameter Specifications\n\n");
        for cmd in &spec.commands {
            md.push_str(&format!("##### `{}`\n\n", cmd.name));
            if !cmd.description.is_empty() {
                md.push_str(&format!("{}\n\n", cmd.description));
            }

            md.push_str("**Parameters:**\n\n");
            if cmd.arguments.is_empty() {
                md.push_str("*None*\n\n");
            } else {
                md.push_str("| Flag / Arg | Type | Required | Default | Description |\n");
                md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
                for arg in &cmd.arguments {
                    let flag_col = if arg.positional {
                        format!("`{}` (positional)", arg.name)
                    } else {
                        format!("`{}`", arg.cli_flag())
                    };
                    let type_col = format!("`{}`", arg.arg_type.python_type_hint());
                    let req_col = if arg.required { "Yes" } else { "No" };
                    let def_col = arg.default_value.as_deref().unwrap_or("-");
                    let desc_col = if arg.description.is_empty() { "-" } else { &arg.description };
                    md.push_str(&format!(
                        "| {} | {} | {} | {} | {} |\n",
                        flag_col, type_col, req_col, def_col, desc_col
                    ));
                }
                md.push('\n');
            }

            md.push_str("**Sample Invocation:**\n");
            md.push_str("```bash\n");
            md.push_str(&cmd.sample_invocation(&format!("python3 {}", generated.cli_filename)));
            md.push_str("\n```\n\n");

            md.push_str("**Expected Output Schema (`--json`):**\n");
            md.push_str("```json\n");
            md.push_str("{\n");
            md.push_str("  \"status\": \"success\",\n");
            md.push_str(&format!("  \"command\": \"{}\",\n", cmd.name));
            md.push_str("  \"result\": {\n");
            md.push_str("    \"action\": \"completed\"\n");
            md.push_str("  },\n");
            md.push_str("  \"timestamp\": \"2026-09-11T12:00:00Z\"\n");
            md.push_str("}\n");
            md.push_str("```\n\n");
        }

        md.push_str("#### Error Handling & Diagnostics\n");
        md.push_str("On failure, the CLI exits with return code `1` and emits an error payload:\n");
        md.push_str("```json\n");
        md.push_str("{\n");
        md.push_str("  \"status\": \"error\",\n");
        md.push_str("  \"command\": \"<subcommand>\",\n");
        md.push_str("  \"error\": \"<human_readable_error>\",\n");
        md.push_str("  \"type\": \"<exception_class>\",\n");
        md.push_str("  \"timestamp\": \"2026-09-11T12:00:00Z\"\n");
        md.push_str("}\n");
        md.push_str("```\n");

        Ok(md)
    }
}
