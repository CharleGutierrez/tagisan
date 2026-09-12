use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::harness::healer::HarnessHealer;
use crate::harness::ingester::BinaryIngester;
use crate::harness::mcp_importer::McpImporter;
use crate::harness::pipeline::{HarnessPipeline, HarnessPipelineOptions};
use crate::harness::runner::HarnessRunner;
use crate::harness::spec::SourceType;
use crate::tui::Spinner;

/// Handle CLI `tgs harness` subcommands with rich ANSI output, spinners, and badges
pub async fn handle_harness_command(
    action: crate::cli::HarnessAction,
    _cli_max_budget: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        crate::cli::HarnessAction::Generate {
            source,
            name,
            lang,
            output_dir,
            install,
            test,
            swarm,
            tier,
        } => {
            println!(
                "\n{}",
                "┌─────────────────────────── 🛠️  CLI-ANYTHING AUTONOMOUS SYNTHESIS ENGINE ───────────────────────────┐".cyan().bold()
            );
            println!(
                "│ Source Codebase : {} │",
                format!("{:<76}", source).yellow()
            );
            println!(
                "│ Target Language : {} │",
                format!("{:<76}", lang).magenta()
            );
            println!(
                "│ Swarm Synthesis : {} │",
                format!("{:<76}", if swarm { format!("Enabled (Tier: {})", tier) } else { "Disabled (Deterministic AST)".to_string() }).green()
            );
            println!(
                "│ Auto-Install    : {} │",
                format!("{:<76}", if install { "Yes (Installing into .ecc/skills/)" } else { "No (Available locally in .tagisan/harness/)" }).blue()
            );
            println!(
                "{}\n",
                "└───────────────────────────────────────────────────────────────────────────────────────────────────┘".cyan().bold()
            );

            let source_path = PathBuf::from(&source);
            if !source_path.exists() {
                eprintln!(
                    "{} Source path does not exist: '{}'",
                    "✖ Error:".red().bold(),
                    source
                );
                std::process::exit(1);
            }

            let source_type = SourceType::from_str(&lang).unwrap_or(SourceType::Auto);
            let mut options = HarnessPipelineOptions::new(source_path)
                .with_lang(source_type)
                .with_install(install)
                .with_run_tests(test)
                .with_swarm(swarm)
                .with_tier(tier);

            if let Some(n) = name {
                options = options.with_name(n);
            }
            if let Some(out) = output_dir {
                options = options.with_output_dir(PathBuf::from(out));
            }

            let spinner = Spinner::start("Executing 7-phase CLI-Anything synthesis pipeline...");

            let pipeline = HarnessPipeline::new();
            let result = match pipeline.execute(options).await {
                Ok(r) => {
                    spinner.success("CLI synthesis pipeline completed successfully!");
                    r
                }
                Err(e) => {
                    spinner.failure(format!("CLI synthesis failed: {}", e));
                    return Err(Box::new(e));
                }
            };

            // Print Phase Breakdown
            println!("\n{}", "--- SYNTHESIS PIPELINE EXECUTION BREAKDOWN ---".bold());
            for phase in &result.phase_results {
                let badge = if phase.success {
                    "✔ [PASS]".green().bold()
                } else {
                    "✖ [FAIL]".red().bold()
                };
                println!(
                    "{} {} ({}ms)\n  ↳ {}",
                    badge,
                    phase.title.bold(),
                    phase.elapsed_ms,
                    phase.message.dimmed()
                );
            }

            // Print Subcommand Table
            println!("\n{}", "--- SYNTHESIZED AGENT-NATIVE SUBCOMMANDS ---".bold());
            println!(
                "{:<24} {:<28} {:<12} {}",
                "Subcommand".cyan().bold(),
                "Target Function".yellow().bold(),
                "Required".magenta().bold(),
                "Description".dimmed()
            );
            println!("{}", "─".repeat(88).dimmed());

            for cmd in &result.spec.commands {
                let req_count = cmd.arguments.iter().filter(|a| a.required).count();
                println!(
                    "{:<24} {:<28} {:<12} {}",
                    cmd.name.cyan(),
                    cmd.function_name.yellow(),
                    format!("{} args", req_count).magenta(),
                    if cmd.description.is_empty() { "-" } else { &cmd.description }
                );
            }

            // Print Output Artifacts
            println!("\n{}", "--- GENERATED HARNESS ARTIFACTS ---".bold());
            println!(
                "• Standalone CLI : {}",
                result.output_dir.join(&result.generated.cli_filename).display().to_string().green().bold()
            );
            println!(
                "• Test Harness   : {}",
                result.output_dir.join(&result.generated.test_filename).display().to_string().green()
            );
            if let Some(ref inst) = result.installed_path {
                println!(
                    "• Installed Skill: {} (SKILL.md registered in .ecc/skills/)",
                    inst.display().to_string().cyan().bold()
                );
            }

            println!(
                "\n{} Total Pipeline Duration: {}ms. Ready for agent execution with '--json'.\n",
                "✨ Done!".green().bold(),
                result.total_elapsed_ms
            );
        }
        crate::cli::HarnessAction::List => {
            println!(
                "\n{}",
                "┌──────────────────────────── 📋 SYNTHESIZED CLI HARNESSES & SKILLS ────────────────────────────┐".cyan().bold()
            );

            let mut harnesses = Vec::new();

            // 1. Scan .tagisan/harness
            let tagisan_harness_dir = Path::new(".tagisan").join("harness");
            if tagisan_harness_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&tagisan_harness_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                            let cli_file = p.join(format!("{}_cli.py", name.replace('-', "_")));
                            let exists = cli_file.is_file();
                            harnesses.push((name, p, exists, "Local Harness"));
                        }
                    }
                }
            }

            // 2. Scan .ecc/skills
            let skills_dir = Path::new(".ecc").join("skills");
            if skills_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&skills_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                            let cli_file = p.join(format!("{}_cli.py", name.replace('-', "_")));
                            if cli_file.is_file() {
                                harnesses.push((name, p, true, "Installed Skill"));
                            }
                        }
                    }
                }
            }

            if harnesses.is_empty() {
                println!("  No synthesized harnesses found in .tagisan/harness or .ecc/skills/.");
                println!("  Synthesize a new one using: tgs harness generate <SOURCE>\n");
            } else {
                println!(
                    "{:<24} {:<18} {:<10} {}",
                    "Harness Name".cyan().bold(),
                    "Type".magenta().bold(),
                    "Status".green().bold(),
                    "Directory Path".dimmed()
                );
                println!("{}", "─".repeat(88).dimmed());

                for (name, path, valid, h_type) in harnesses {
                    let status = if valid { "Ready".green() } else { "Incomplete".yellow() };
                    println!(
                        "{:<24} {:<18} {:<10} {}",
                        name.cyan().bold(),
                        h_type.magenta(),
                        status,
                        path.display().to_string().dimmed()
                    );
                }
                println!();
            }
        }
        crate::cli::HarnessAction::Run { name, args } => {
            let script_path = resolve_harness_script(&name)?;
            println!(
                "{} Executing harness '{}' via AgentShield security gate...\n",
                "⚡ [AgentShield]".yellow().bold(),
                name.cyan().bold()
            );

            let runner = HarnessRunner::new()?;
            let result = runner.run(&script_path, &args, None).await?;

            println!("{}", result.display_summary());

            if !result.success {
                std::process::exit(result.exit_code);
            }
        }
        crate::cli::HarnessAction::Test { name } => {
            let test_path = resolve_test_script(&name)?;
            println!(
                "{} Running automated validation test suite for '{}'...\n",
                "🧪 [Test Runner]".green().bold(),
                name.cyan().bold()
            );

            let runner = HarnessRunner::new()?;
            let result = runner.run_tests(&test_path, None).await?;

            println!("{}", result.display_summary());

            if result.success {
                println!("\n{} All validation tests passed cleanly!\n", "✔ Success:".green().bold());
            } else {
                println!("\n{} Validation tests failed.\n", "✖ Failure:".red().bold());
                std::process::exit(result.exit_code);
            }
        }
        crate::cli::HarnessAction::Heal { name, attempts } => {
            let cli_path = resolve_harness_script(&name)?;
            let test_path = resolve_test_script(&name)?;

            println!(
                "\n{}",
                "┌──────────────────────────── 🩺 AUTONOMOUS HARNESS SELF-HEALING ────────────────────────────┐".magenta().bold()
            );
            println!(
                "│ Target Harness  : {} │",
                format!("{:<76}", name).yellow()
            );
            println!(
                "│ Max Heal Passes : {} │",
                format!("{:<76}", attempts).cyan()
            );
            println!(
                "{}\n",
                "└─────────────────────────────────────────────────────────────────────────────────────────────┘".magenta().bold()
            );

            let spinner = Spinner::start("Diagnosing test failure and synthesizing autonomous AST fixes...");
            let healer = HarnessHealer::new()?.with_max_attempts(attempts);
            let report = healer.heal(&name, &cli_path, &test_path).await?;

            if report.healed {
                spinner.success(format!("Autonomous self-healing succeeded in {} attempt(s)!", report.attempts_made));
            } else if report.fixes_applied.is_empty() {
                spinner.success("Harness is already healthy and all validation tests pass!");
            } else {
                spinner.failure(format!("Autonomous self-healing could not resolve all issues after {} attempts", attempts));
            }

            println!("\n{}", report.display_summary());

            if !report.healed && !report.fixes_applied.is_empty() {
                std::process::exit(1);
            }
        }
        crate::cli::HarnessAction::Ingest { binary, name, output_dir, install } => {
            println!(
                "\n{}",
                "┌─────────────────────────── 📥 BLACK-BOX BINARY INGESTION ENGINE ───────────────────────────┐".cyan().bold()
            );
            println!(
                "│ Target Binary   : {} │",
                format!("{:<76}", binary).yellow()
            );
            println!(
                "│ Auto-Install    : {} │",
                format!("{:<76}", if install { "Yes (Installing into .ecc/skills/)" } else { "No (Available locally in .tagisan/harness/)" }).blue()
            );
            println!(
                "{}\n",
                "└─────────────────────────────────────────────────────────────────────────────────────────────┘".cyan().bold()
            );

            let spinner = Spinner::start("Probing binary --help, extracting subcommands, and generating agent-native CLI...");
            let res = BinaryIngester::ingest(
                &binary,
                name,
                output_dir.map(PathBuf::from),
                install,
            ).await?;
            spinner.success("Binary ingestion completed successfully!");

            println!("\n{}", res.display_summary());
        }
        crate::cli::HarnessAction::ImportMcp { spec, name, output_dir, install } => {
            println!(
                "\n{}",
                "┌───────────────────────────── 🌐 MCP TO CLI TRANSPILATION ENGINE ────────────────────────────┐".magenta().bold()
            );
            println!(
                "│ MCP Spec Schema : {} │",
                format!("{:<76}", spec).yellow()
            );
            println!(
                "│ Auto-Install    : {} │",
                format!("{:<76}", if install { "Yes (Installing into .ecc/skills/)" } else { "No (Available locally in .tagisan/harness/)" }).blue()
            );
            println!(
                "{}\n",
                "└─────────────────────────────────────────────────────────────────────────────────────────────┘".magenta().bold()
            );

            let spinner = Spinner::start("Parsing MCP tool schema and synthesizing standalone CLI with RFC-004 SKILL.md...");
            let res = McpImporter::import(
                &spec,
                name,
                output_dir.map(PathBuf::from),
                install,
            ).await?;
            spinner.success("MCP transpilation completed successfully!");

            println!("\n{}", res.display_summary());
        }
    }

    Ok(())
}

fn resolve_harness_script(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let direct = PathBuf::from(name);
    if direct.is_file() {
        return Ok(direct);
    }

    let norm_name = name.to_lowercase().replace('_', "-");
    let script_filename = format!("{}_cli.py", norm_name.replace('-', "_"));

    // Check .tagisan/harness/<name>/
    let in_tagisan = Path::new(".tagisan").join("harness").join(&norm_name).join(&script_filename);
    if in_tagisan.is_file() {
        return Ok(in_tagisan);
    }

    // Check .ecc/skills/<name>/
    let in_ecc = Path::new(".ecc").join("skills").join(&norm_name).join(&script_filename);
    if in_ecc.is_file() {
        return Ok(in_ecc);
    }

    Err(format!(
        "Synthesized harness '{}' not found in .tagisan/harness or .ecc/skills. Run 'tgs harness generate' first.",
        name
    ).into())
}

fn resolve_test_script(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let direct = PathBuf::from(name);
    if direct.is_file() {
        return Ok(direct);
    }

    let norm_name = name.to_lowercase().replace('_', "-");
    let test_filename = format!("test_{}_cli.py", norm_name.replace('-', "_"));

    // Check .tagisan/harness/<name>/
    let in_tagisan = Path::new(".tagisan").join("harness").join(&norm_name).join(&test_filename);
    if in_tagisan.is_file() {
        return Ok(in_tagisan);
    }

    // Check .ecc/skills/<name>/
    let in_ecc = Path::new(".ecc").join("skills").join(&norm_name).join(&test_filename);
    if in_ecc.is_file() {
        return Ok(in_ecc);
    }

    Err(format!(
        "Validation test script for '{}' not found in .tagisan/harness or .ecc/skills.",
        name
    ).into())
}
