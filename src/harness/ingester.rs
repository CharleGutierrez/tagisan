use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use crate::error::{Result, TagisanError};
use crate::harness::generator::GeneratedHarness;
use crate::harness::skill_packager::SkillPackager;
use crate::harness::spec::{ArgumentSpec, ArgumentType, CommandSpec, HarnessSpec, SourceType};

/// Ingestion result containing generated CLI, test suite, and skill documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub binary_name: String,
    pub binary_path: PathBuf,
    pub subcommands_count: usize,
    pub options_count: usize,
    pub output_dir: PathBuf,
    pub cli_path: PathBuf,
    pub test_path: PathBuf,
    pub skill_path: Option<PathBuf>,
}

impl IngestResult {
    pub fn display_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} Successfully ingested black-box binary '{}'\n",
            "✔ [INGESTED]".green().bold(),
            self.binary_name.cyan().bold()
        ));
        out.push_str(&format!("  • Binary Location: {}\n", self.binary_path.display().to_string().dimmed()));
        out.push_str(&format!("  • Ingested Commands: {} subcommands, {} options\n", self.subcommands_count, self.options_count));
        out.push_str(&format!("  • Standalone CLI: {}\n", self.cli_path.display().to_string().green()));
        out.push_str(&format!("  • Test Harness:   {}\n", self.test_path.display().to_string().green()));
        if let Some(ref sk) = self.skill_path {
            out.push_str(&format!("  • Installed Skill: {}\n", sk.display().to_string().cyan().bold()));
        }
        out
    }
}

/// Black-Box Binary Ingester that converts arbitrary system binaries into agent-native CLIs & SKILL.md
pub struct BinaryIngester;

impl BinaryIngester {
    /// Ingests a system binary by probing `--help`, parsing flags and subcommands, and synthesizing harness
    pub async fn ingest(
        binary_target: &str,
        custom_name: Option<String>,
        output_dir: Option<PathBuf>,
        install: bool,
    ) -> Result<IngestResult> {
        let binary_path = Self::resolve_binary(binary_target)?;
        let name = custom_name.unwrap_or_else(|| {
            binary_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("binary_tool")
                .to_string()
        });

        // 1. Probe --help / -h
        let help_output = Self::probe_help(&binary_path).await?;

        // 2. Parse Help Output into HarnessSpec
        let spec = Self::parse_help_to_spec(&name, &binary_path, &help_output).await?;

        // 3. Generate Standalone Binary Wrapper CLI & Tests
        let generated = Self::generate_binary_wrapper(&spec, &binary_path)?;

        // 4. Determine Output Directory
        let target_dir = output_dir.unwrap_or_else(|| {
            Path::new(".tagisan")
                .join("harness")
                .join(&name.to_lowercase().replace('_', "-"))
        });
        fs::create_dir_all(&target_dir).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to create output directory '{}': {}",
                target_dir.display(),
                e
            ))
        })?;

        let (cli_path, test_path) = generated.write_to_dir(&target_dir)?;

        // 5. Package RFC-004 SKILL.md
        let packaged = SkillPackager::package(&spec, &generated)?;
        let skill_md_path = target_dir.join("SKILL.md");
        fs::write(&skill_md_path, &packaged.skill_md_content).map_err(|e| {
            TagisanError::Execution(format!("Failed to write SKILL.md: {}", e))
        })?;

        let installed_path = if install {
            let skills_root = Path::new(".ecc").join("skills");
            Some(packaged.install_to(&skills_root)?)
        } else {
            None
        };

        let subcommands_count = spec.commands.len();
        let options_count = spec.global_options.len()
            + spec.commands.iter().map(|c| c.arguments.len()).sum::<usize>();

        Ok(IngestResult {
            binary_name: name,
            binary_path,
            subcommands_count,
            options_count,
            output_dir: target_dir,
            cli_path,
            test_path,
            skill_path: installed_path,
        })
    }

    /// Resolves executable binary path from PATH or direct path
    pub fn resolve_binary(target: &str) -> Result<PathBuf> {
        let direct = PathBuf::from(target);
        if direct.is_file() {
            return Ok(direct);
        }

        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let cand = dir.join(target);
                if cand.is_file() {
                    return Ok(cand);
                }
            }
        }

        Err(TagisanError::Execution(format!(
            "Binary '{}' not found in system PATH or filesystem",
            target
        )))
    }

    /// Executes `--help`, `-h`, or `help` to capture command signatures
    async fn probe_help(binary_path: &Path) -> Result<String> {
        for flag in &["--help", "-h", "help"] {
            let mut cmd = Command::new(binary_path);
            cmd.arg(flag);
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            if let Ok(output) = cmd.output().await {
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                let err_str = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = if out_str.trim().is_empty() {
                    err_str
                } else {
                    out_str
                };
                if !combined.trim().is_empty() {
                    return Ok(combined);
                }
            }
        }

        Err(TagisanError::Execution(format!(
            "Failed to capture help documentation from binary '{}'",
            binary_path.display()
        )))
    }

    /// Probes help for a specific subcommand
    async fn probe_subcommand_help(binary_path: &Path, subcmd: &str) -> Option<String> {
        let mut cmd = Command::new(binary_path);
        cmd.arg(subcmd);
        cmd.arg("--help");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Ok(output) = cmd.output().await {
            let out_str = String::from_utf8_lossy(&output.stdout).to_string();
            let err_str = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if out_str.trim().is_empty() { err_str } else { out_str };
            if !combined.trim().is_empty() {
                return Some(combined);
            }
        }
        None
    }

    /// Parses help text and extracts subcommands, options, and arguments into a HarnessSpec
    pub async fn parse_help_to_spec(
        name: &str,
        binary_path: &Path,
        help_text: &str,
    ) -> Result<HarnessSpec> {
        let mut spec = HarnessSpec::new(name, binary_path, SourceType::Auto);
        spec.description = format!("Agent-native CLI wrapper for system binary {}", name);

        // 1. Detect subcommands
        let detected_subcmds = Self::extract_subcommands(help_text);

        if !detected_subcmds.is_empty() {
            for (sub_name, sub_desc) in detected_subcmds {
                let mut cmd = CommandSpec::new(&sub_name, &sub_name).with_description(&sub_desc);

                // Try to get subcmd options if available
                if let Some(sub_help) = Self::probe_subcommand_help(binary_path, &sub_name).await {
                    let sub_args = Self::extract_options(&sub_help);
                    for arg in sub_args {
                        cmd = cmd.with_argument(arg);
                    }
                }

                // Add generic positional argument or flags if empty
                if cmd.arguments.is_empty() {
                    cmd = cmd.with_argument(
                        ArgumentSpec::new("args", ArgumentType::List(Box::new(ArgumentType::String)))
                            .with_description("Optional arguments passed directly to the binary")
                            .with_positional(false),
                    );
                }

                spec = spec.with_command(cmd);
            }
        } else {
            // Single-command binary (e.g. curl, jq, grep)
            let options = Self::extract_options(help_text);
            let mut main_cmd = CommandSpec::new("run", "run")
                .with_description(format!("Execute {} with arguments", name));

            for opt in options {
                main_cmd = main_cmd.with_argument(opt);
            }

            // Always add a raw positional target or query arg
            main_cmd = main_cmd.with_argument(
                ArgumentSpec::new("target", ArgumentType::String)
                    .with_description("Target URL, file, or pattern")
                    .with_positional(true),
            );

            spec = spec.with_command(main_cmd);
        }

        Ok(spec)
    }

    /// Extracts subcommand names and descriptions from help text
    fn extract_subcommands(help_text: &str) -> Vec<(String, String)> {
        let mut subcommands = Vec::new();
        let mut in_command_section = false;

        for line in help_text.lines() {
            let trimmed = line.trim();
            let lower = trimmed.to_lowercase();

            if lower.contains("commands:")
                || lower.contains("available commands:")
                || lower.contains("common git commands")
                || lower.contains("management commands:")
                || lower.contains("subcommands:")
            {
                in_command_section = true;
                continue;
            }

            if in_command_section {
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.ends_with(':') && !trimmed.starts_with('-') {
                    // Entered a new section
                    in_command_section = false;
                    continue;
                }

                // Match format: "   command    Description"
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Some(cmd_cand) = parts.first() {
                    let cmd_name = cmd_cand.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                    if !cmd_name.is_empty()
                        && !cmd_name.starts_with('-')
                        && cmd_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
                        && cmd_name.len() >= 2
                    {
                        let desc = if parts.len() > 1 {
                            parts[1..].join(" ")
                        } else {
                            format!("Run {} operation", cmd_name)
                        };
                        subcommands.push((cmd_name.to_string(), desc));
                    }
                }
            }
        }

        // Limit to 20 subcommands to keep CLI crisp and focused
        subcommands.truncate(20);
        subcommands
    }

    /// Extracts options and flags from help text
    fn extract_options(help_text: &str) -> Vec<ArgumentSpec> {
        let mut options = Vec::new();

        for line in help_text.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('-') {
                continue;
            }

            // Example line: "-o, --output <file>    Output file path"
            // or: "--verbose               Enable verbose mode"
            let mut short_flag = None;
            let mut long_flag = None;
            let mut arg_type = ArgumentType::Boolean;
            let mut desc = String::new();

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let mut idx = 0;

            while idx < parts.len() {
                let part = parts[idx];
                if part.starts_with("--") {
                    let clean = part.trim_start_matches("--").trim_end_matches(',');
                    long_flag = Some(clean.to_string());
                } else if part.starts_with('-') && part.len() == 2 {
                    short_flag = part.chars().nth(1);
                } else if part.starts_with('<') || part.starts_with('[') || part.to_uppercase() == part && part.len() > 1 {
                    // Takes argument value
                    let val_name = part.to_lowercase();
                    if val_name.contains("int") || val_name.contains("num") || val_name.contains("port") {
                        arg_type = ArgumentType::Integer;
                    } else if val_name.contains("file") || val_name.contains("path") {
                        arg_type = ArgumentType::FilePath;
                    } else {
                        arg_type = ArgumentType::String;
                    }
                } else {
                    // Rest of line is description
                    desc = parts[idx..].join(" ");
                    break;
                }
                idx += 1;
            }

            if let Some(flag_name) = long_flag {
                let clean_name = flag_name.replace('-', "_");
                if clean_name != "help" && clean_name != "version" && clean_name != "json" {
                    let mut arg = ArgumentSpec::new(clean_name, arg_type).with_description(desc);
                    if let Some(s) = short_flag {
                        arg = arg.with_short(s);
                    }
                    options.push(arg);
                }
            }
        }

        options.truncate(15);
        options
    }

    /// Generates standalone binary wrapper CLI script
    fn generate_binary_wrapper(spec: &HarnessSpec, binary_path: &Path) -> Result<GeneratedHarness> {
        let cli_filename = format!("{}_cli.py", spec.name.replace('-', "_"));
        let test_filename = format!("test_{}_cli.py", spec.name.replace('-', "_"));

        let mut cli_code = String::new();
        cli_code.push_str("#!/usr/bin/env python3\n");
        cli_code.push_str(&format!(
            "# Agent-Native Binary Wrapper for '{}'\n",
            spec.name
        ));
        cli_code.push_str("# Synthesized autonomously by Tagisan (tgs harness ingest)\n\n");

        cli_code.push_str("import argparse\n");
        cli_code.push_str("import datetime\n");
        cli_code.push_str("import json\n");
        cli_code.push_str("import os\n");
        cli_code.push_str("import subprocess\n");
        cli_code.push_str("import sys\n");
        cli_code.push_str("import traceback\n\n");

        let bin_escaped = binary_path.to_string_lossy().replace('\\', "/");
        cli_code.push_str(&format!("TARGET_BINARY = \"{}\"\n\n", bin_escaped));

        // Subcommand handlers
        for cmd in &spec.commands {
            let func_name = format!("handle_{}", cmd.name.replace('-', "_"));
            cli_code.push_str(&format!("def {}(args):\n", func_name));
            cli_code.push_str("    cmd = [TARGET_BINARY]\n");
            if cmd.name != "run" && cmd.name != "execute" {
                cli_code.push_str(&format!("    cmd.append('{}')\n", cmd.name));
            }

            for arg in &cmd.arguments {
                let var = arg.python_var_name();
                if arg.positional {
                    cli_code.push_str(&format!(
                        "    if hasattr(args, '{}') and getattr(args, '{}'):\n",
                        var, var
                    ));
                    cli_code.push_str(&format!(
                        "        cmd.append(str(getattr(args, '{}')))\n",
                        var
                    ));
                } else if matches!(arg.arg_type, ArgumentType::Boolean) {
                    cli_code.push_str(&format!(
                        "    if getattr(args, '{}', False):\n",
                        var
                    ));
                    cli_code.push_str(&format!("        cmd.append('{}')\n", arg.cli_flag()));
                } else {
                    cli_code.push_str(&format!(
                        "    if getattr(args, '{}', None) is not None:\n",
                        var
                    ));
                    cli_code.push_str(&format!("        cmd.append('{}')\n", arg.cli_flag()));
                    cli_code.push_str(&format!(
                        "        cmd.append(str(getattr(args, '{}')))\n",
                        var
                    ));
                }
            }

            // Run process
            cli_code.push_str("    proc = subprocess.run(cmd, capture_output=True, text=True)\n");
            cli_code.push_str("    # Attempt to parse stdout as JSON if structured\n");
            cli_code.push_str("    parsed_data = None\n");
            cli_code.push_str("    try:\n");
            cli_code.push_str("        if proc.stdout.strip().startswith('{') or proc.stdout.strip().startswith('['):\n");
            cli_code.push_str("            parsed_data = json.loads(proc.stdout.strip())\n");
            cli_code.push_str("    except Exception:\n");
            cli_code.push_str("        pass\n\n");

            cli_code.push_str("    return {\n");
            cli_code.push_str("        'stdout': proc.stdout,\n");
            cli_code.push_str("        'stderr': proc.stderr,\n");
            cli_code.push_str("        'exit_code': proc.returncode,\n");
            cli_code.push_str("        'parsed': parsed_data,\n");
            cli_code.push_str("        'command_line': cmd,\n");
            cli_code.push_str("    }\n\n");
        }

        // Build parser
        cli_code.push_str("def build_parser():\n");
        cli_code.push_str("    parser = argparse.ArgumentParser(prog='");
        cli_code.push_str(&spec.name);
        cli_code.push_str("', description='Agent-native wrapper')\n");
        cli_code.push_str("    parser.add_argument('--json', action='store_true', help='Emit structured JSON')\n");
        cli_code.push_str("    parser.add_argument('--version', action='version', version='1.0.0')\n");
        cli_code.push_str("    subparsers = parser.add_subparsers(dest='subcommand', title='subcommands')\n\n");

        for cmd in &spec.commands {
            let p_var = format!("p_{}", cmd.name.replace('-', "_"));
            cli_code.push_str(&format!(
                "    {} = subparsers.add_parser('{}', help='{}')\n",
                p_var,
                cmd.name,
                cmd.description.replace('\'', "\\'")
            ));

            for arg in &cmd.arguments {
                if arg.positional {
                    cli_code.push_str(&format!(
                        "    {}.add_argument('{}', nargs='?', default='')\n",
                        p_var, arg.name
                    ));
                } else if matches!(arg.arg_type, ArgumentType::Boolean) {
                    cli_code.push_str(&format!(
                        "    {}.add_argument('{}', action='store_true')\n",
                        p_var,
                        arg.cli_flag()
                    ));
                } else {
                    cli_code.push_str(&format!(
                        "    {}.add_argument('{}', type=str, default=None)\n",
                        p_var,
                        arg.cli_flag()
                    ));
                }
            }
        }

        cli_code.push_str("    return parser\n\n");

        // Main
        cli_code.push_str("def main():\n");
        cli_code.push_str("    parser = build_parser()\n");
        cli_code.push_str("    args = parser.parse_args()\n");
        cli_code.push_str("    if not args.subcommand:\n");
        cli_code.push_str("        parser.print_help()\n");
        cli_code.push_str("        sys.exit(0)\n\n");

        cli_code.push_str("    handlers = {\n");
        for cmd in &spec.commands {
            cli_code.push_str(&format!(
                "        '{}': handle_{},\n",
                cmd.name,
                cmd.name.replace('-', "_")
            ));
        }
        cli_code.push_str("    }\n\n");

        cli_code.push_str("    handler = handlers.get(args.subcommand)\n");
        cli_code.push_str("    if not handler:\n");
        cli_code.push_str("        sys.exit(1)\n\n");

        cli_code.push_str("    result = handler(args)\n");
        cli_code.push_str("    if getattr(args, 'json', False):\n");
        cli_code.push_str("        envelope = {\n");
        cli_code.push_str("            'status': 'success' if result.get('exit_code', 0) == 0 else 'error',\n");
        cli_code.push_str("            'command': args.subcommand,\n");
        cli_code.push_str("            'result': result,\n");
        cli_code.push_str("            'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),\n");
        cli_code.push_str("        }\n");
        cli_code.push_str("        print(json.dumps(envelope, indent=2))\n");
        cli_code.push_str("    else:\n");
        cli_code.push_str("        sys.stdout.write(result.get('stdout', ''))\n");
        cli_code.push_str("        sys.stderr.write(result.get('stderr', ''))\n");
        cli_code.push_str("    sys.exit(result.get('exit_code', 0))\n\n");

        cli_code.push_str("if __name__ == '__main__':\n");
        cli_code.push_str("    main()\n");

        // Tests
        let mut test_code = String::new();
        test_code.push_str("#!/usr/bin/env python3\n");
        test_code.push_str("import json, os, subprocess, sys, unittest\n\n");
        test_code.push_str(&format!("CLI_SCRIPT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '{}')\n\n", cli_filename));
        test_code.push_str("class TestBinaryWrapper(unittest.TestCase):\n");
        test_code.push_str("    def test_help(self):\n");
        test_code.push_str("        res = subprocess.run([sys.executable, CLI_SCRIPT, '--help'], capture_output=True, text=True)\n");
        test_code.push_str("        self.assertEqual(res.returncode, 0)\n\n");

        if let Some(first_cmd) = spec.commands.first() {
            test_code.push_str("    def test_subcommand_json(self):\n");
            test_code.push_str(&format!(
                "        res = subprocess.run([sys.executable, CLI_SCRIPT, '{}', '--json'], capture_output=True, text=True)\n",
                first_cmd.name
            ));
            test_code.push_str("        self.assertIn(res.returncode, [0, 1, 2])\n");
            test_code.push_str("        data = json.loads(res.stdout)\n");
            test_code.push_str(&format!(
                "        self.assertEqual(data.get('command'), '{}')\n\n",
                first_cmd.name
            ));
        }

        test_code.push_str("if __name__ == '__main__':\n    unittest.main()\n");

        Ok(GeneratedHarness {
            cli_code,
            test_code,
            cli_filename,
            test_filename,
            spec: spec.clone(),
        })
    }
}
