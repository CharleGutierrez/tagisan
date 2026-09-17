//! Frontier Skills for Tagisan (`tgs`):
//! - `tgs screen`: Multimodal Vision Screen Perception
//! - `tgs commit`: Autonomous Git conventional commits with AgentShield secret scanning
//! - `tgs review`: Triad multi-agent PR code review with Lakandiwa verdict
//! - `tgs watch`: Proactive background compiler & test sentinel watchdog

use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;
use colored::Colorize;
use futures::StreamExt;
use chrono::Local;

use crate::cli::{build_engine_context, resolve_provider_and_model};
use crate::error::{Result, TagisanError};
use crate::types::{CompletionRequest, ContentBlock, Message, StreamChunkDelta};
use crate::ecc::agentshield::AgentShieldScanner;
use crate::engine::autofix::{detect_project_type, ProjectType};

// ---------------------------------------------------------------------------
// 1. Multimodal Screen Perception (tgs screen)
// ---------------------------------------------------------------------------

struct TempFileGuard {
    path: PathBuf,
    armed: bool,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.armed && self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

pub async fn handle_screen_command(
    mode: String,
    provider: String,
    prompt: String,
    output: Option<String>,
    cli_max_budget: f64,
) -> Result<()> {
    let (temp_path, _guard) = match output {
        Some(ref p) => (PathBuf::from(p), None),
        None => {
            let pid = std::process::id();
            let timestamp = Local::now().format("%Y%m%d_%H%M%S");
            let p = std::env::temp_dir().join(format!("tgs_screen_{}_{}.png", pid, timestamp));
            let g = TempFileGuard::new(p.clone());
            (p, Some(g))
        }
    };

    println!("{}", "📸 Capturing screen via Spectacle/Wayland...".cyan().bold());

    // Execute screenshot capture
    let capture_status = if Command::new("spectacle").arg("--version").output().is_ok() {
        let mut cmd = Command::new("spectacle");
        cmd.arg("-b").arg("-n");
        match mode.to_lowercase().as_str() {
            "active" | "window" => {
                cmd.arg("-a");
            }
            "region" | "area" => {
                println!("{}", "👉 Please drag to select a region on screen...".yellow());
                cmd.arg("-r");
            }
            _ => {
                cmd.arg("-f");
            }
        }
        cmd.arg("-o").arg(&temp_path);
        cmd.status()
    } else if Command::new("grim").arg("-version").output().is_ok() {
        Command::new("grim").arg(&temp_path).status()
    } else if Command::new("import").arg("-version").output().is_ok() {
        Command::new("import").arg("-window").arg("root").arg(&temp_path).status()
    } else {
        return Err(TagisanError::Execution(
            "No screenshot utility found. Please install 'spectacle', 'grim', or 'imagemagick'.".into(),
        ));
    };

    match capture_status {
        Ok(st) if st.success() && temp_path.exists() && std::fs::metadata(&temp_path).map(|m| m.len() > 0).unwrap_or(false) => {
            let file_size = std::fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0);
            println!(
                "{} Captured screenshot: {} ({:.1} KB)\n",
                "✔".green().bold(),
                temp_path.display().to_string().cyan(),
                file_size as f64 / 1024.0
            );
        }
        Ok(_) | Err(_) => {
            return Err(TagisanError::Execution(
                format!("Failed to capture screenshot to {:?}", temp_path),
            ));
        }
    }

    // Connect to multimodal provider (Gemini 2.5 Flash is preferred for vision)
    let ctx = build_engine_context(cli_max_budget);
    let target_provider = if provider == "auto" { "gemini" } else { &provider };
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, target_provider, None)?;

    println!(
        "{} [{}: {}] with image attachment...",
        "Querying Vision Model".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );
    println!("Prompt: \"{}\"\n", prompt.italic());

    let user_blocks = vec![
        ContentBlock::text(prompt.clone()),
        ContentBlock::from_image_file(&temp_path)?,
    ];

    let req = CompletionRequest::new(model_name.clone(), "")
        .with_messages(vec![Message::user_with_content(user_blocks)])
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    let mut stream = prov.stream(req).await?;
    let start = std::time::Instant::now();
    let mut full_response = String::new();

    while let Some(chunk_res) = stream.next().await {
        match chunk_res {
            Ok(chunk) => {
                match chunk.delta {
                    StreamChunkDelta::Text(text) => {
                        print!("{}", text);
                        io::stdout().flush().ok();
                        full_response.push_str(&text);
                    }
                    StreamChunkDelta::Thinking(th) => {
                        print!("{}", th.dimmed());
                        io::stdout().flush().ok();
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("\n{}: Vision stream error: {}", "Error".red().bold(), e);
                break;
            }
        }
    }

    let elapsed = start.elapsed();
    println!(
        "\n\n{} Response received in {:.2}s",
        "✔".green().bold(),
        elapsed.as_secs_f64()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Autonomous Git Conventional Commit (tgs commit)
// ---------------------------------------------------------------------------

pub async fn handle_commit_command(
    yes: bool,
    hint: Option<String>,
    cli_max_budget: f64,
) -> Result<()> {
    // 1. Verify inside git repo
    let git_check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    if git_check.is_err() || !git_check.unwrap().status.success() {
        return Err(TagisanError::Execution(
            "Current directory is not inside a git repository.".into(),
        ));
    }

    // 2. Check staged diff
    let mut staged_diff = get_git_output(&["diff", "--cached"])?;

    if staged_diff.trim().is_empty() {
        let status = get_git_output(&["status", "--porcelain"])?;
        if status.trim().is_empty() {
            println!("{}", "✔ Working tree clean, nothing to commit.".green().bold());
            return Ok(());
        }

        println!("{}", "No changes currently staged in git.".yellow());
        print!("{}", "Would you like to stage all tracked modified files (git add -u)? [Y/n]: ".bold());
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let trimmed = input.trim().to_lowercase();
        if trimmed.is_empty() || trimmed == "y" || trimmed == "yes" {
            let add_status = Command::new("git").args(["add", "-u"]).status()?;
            if !add_status.success() {
                return Err(TagisanError::Execution("Failed to stage tracked files with 'git add -u'".into()));
            }
            staged_diff = get_git_output(&["diff", "--cached"])?;
        } else {
            println!("Commit cancelled.");
            return Ok(());
        }
    }

    if staged_diff.trim().is_empty() {
        println!("{}", "No staged changes to commit.".yellow());
        return Ok(());
    }

    // 3. AgentShield Cyber Defense & Secret Leak Inspection
    println!("{}", "🛡️  Running AgentShield Security & Secret Leak Audit on staged diff...".cyan());
    
    // Quick regex scan for common API keys, tokens, and private keys
    let secret_patterns = [
        ("AIza[0-9A-Za-z-_]{35}", "Google Gemini / Cloud API Key"),
        ("ghp_[0-9A-Za-z]{36}", "GitHub Personal Access Token"),
        ("sk-[0-9A-Za-z]{32,}", "OpenAI API Secret Key"),
        ("BEGIN (?:RSA|OPENSSH|EC|DSA|PGP) PRIVATE KEY", "Private Cryptographic Key"),
        ("aws_secret_access_key\\s*=", "AWS Secret Access Key"),
    ];

    for (pat, desc) in secret_patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            if re.is_match(&staged_diff) {
                eprintln!("\n{}", "══════════════════════════════════════════════════════════════".red().bold());
                eprintln!("{}", "🚨 CRITICAL SECURITY VIOLATION: LEAK DETECTED IN STAGED DIFF".bold().red());
                eprintln!("{}", "══════════════════════════════════════════════════════════════".red().bold());
                eprintln!("Pattern Match: {}", desc.yellow().bold());
                eprintln!("Commit blocked by AgentShield to prevent publishing sensitive credentials.\n");
                return Err(TagisanError::Security(
                    format!("AgentShield blocked commit: Potential {} found in staged diff", desc),
                ));
            }
        }
    }

    let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(&staged_diff);
    if let crate::ecc::AgentShieldVerdict::Block { reason, threat_level } = dlp_verdict {
        eprintln!("\n{}", "🚨 AgentShield Blocked Staged Commit:".bold().red());
        eprintln!("Threat Level: {:?} | Reason: {}", threat_level, reason.yellow());
        return Err(TagisanError::Security(format!("AgentShield DLP: {}", reason)));
    }
    println!("{}", "✔ AgentShield: Clean staged diff (0 security violations detected).".green());

    // 4. Generate Semantic Conventional Commit Message
    println!("{}", "⚡ Generating Conventional Commit Message via TGS...".cyan());
    
    // Truncate diff if oversized to preserve token budget
    let effective_diff = if staged_diff.len() > 25_000 {
        format!("{}\n\n[... Diff truncated at 25,000 characters ...]", &staged_diff[..25_000])
    } else {
        staged_diff
    };

    let hint_text = hint.map(|h| format!("\nUser context/intent hint: {}", h)).unwrap_or_default();

    let prompt = format!(
        "You are an expert software engineer generating a high-quality, professional Conventional Commit message for this git diff.\n\
        Rules:\n\
        1. First line: <type>(<scope>): <short imperative subject in 50 chars or less>\n\
        2. Blank line\n\
        3. 2-4 concise bullet points explaining WHAT changed and WHY.\n\
        4. Valid types: feat, fix, refactor, perf, test, docs, chore, ci.\n\
        5. Output ONLY the raw commit message text. Do NOT wrap in backticks or markdown code blocks.\n\
        {hint_text}\n\n\
        STAGED DIFF:\n{effective_diff}"
    );

    let ctx = build_engine_context(cli_max_budget);
    let (_provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;

    let req = CompletionRequest::new(model_name, prompt);
    let resp = prov.complete(req).await?;
    let raw_msg = resp.message.extract_text();
    let commit_msg = raw_msg.trim_matches('`').trim().to_string();

    println!("\n{}", "───────────────────────────────────────────────────────".cyan());
    println!("{}", "📝 Proposed Conventional Commit Message:".bold().yellow());
    println!("{}", "───────────────────────────────────────────────────────".cyan());
    println!("{}", commit_msg.green().bold());
    println!("{}\n", "───────────────────────────────────────────────────────".cyan());

    if yes {
        execute_git_commit(&commit_msg)?;
        return Ok(());
    }

    print!("{}", "Execute this commit? [Y/n/e(edit)]: ".bold());
    io::stdout().flush().ok();

    let mut user_choice = String::new();
    io::stdin().read_line(&mut user_choice).ok();
    let choice = user_choice.trim().to_lowercase();

    if choice.is_empty() || choice == "y" || choice == "yes" {
        execute_git_commit(&commit_msg)?;
    } else if choice == "e" || choice == "edit" {
        let temp_msg_file = std::env::temp_dir().join(format!("tgs_commit_msg_{}.txt", std::process::id()));
        std::fs::write(&temp_msg_file, &commit_msg)?;
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let _ = Command::new(&editor).arg(&temp_msg_file).status();
        let edited_msg = std::fs::read_to_string(&temp_msg_file).unwrap_or(commit_msg);
        let _ = std::fs::remove_file(&temp_msg_file);
        if !edited_msg.trim().is_empty() {
            execute_git_commit(edited_msg.trim())?;
        } else {
            println!("Empty commit message. Aborted.");
        }
    } else {
        println!("Commit aborted.");
    }

    Ok(())
}

fn execute_git_commit(msg: &str) -> Result<()> {
    let st = Command::new("git")
        .args(["commit", "-m", msg])
        .status()?;
    if st.success() {
        println!("\n{}", "✔ Commit successfully created!".green().bold());
    } else {
        eprintln!("\n{}", "✗ Git commit failed.".red().bold());
    }
    Ok(())
}

fn get_git_output(args: &[&str]) -> Result<String> {
    let out = Command::new("git").args(args).output()?;
    if !out.status.success() {
        return Err(TagisanError::Execution(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

// ---------------------------------------------------------------------------
// 3. Multi-Agent Peer Review Triad (tgs review)
// ---------------------------------------------------------------------------

pub async fn handle_review_command(
    target: String,
    cli_max_budget: f64,
) -> Result<()> {
    // 1. Verify inside git repo
    let git_check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    if git_check.is_err() || !git_check.unwrap().status.success() {
        return Err(TagisanError::Execution(
            "Current directory is not inside a git repository.".into(),
        ));
    }

    println!(
        "{} Comparing current branch against target '{}'...",
        "🔍 Fetching git diff:".bold().cyan(),
        target.yellow()
    );

    // Try diffing against target, or fall back to HEAD~1
    let mut diff = get_git_output(&["diff", &format!("{}...HEAD", target)])
        .or_else(|_| get_git_output(&["diff", &target]))
        .or_else(|_| get_git_output(&["diff", "HEAD~1"]))
        .or_else(|_| get_git_output(&["diff"]))?;

    if diff.trim().is_empty() {
        println!(
            "{} No code differences found against '{}'.",
            "✔".green().bold(),
            target
        );
        return Ok(());
    }

    let total_lines = diff.lines().count();
    println!(
        "Found {} lines of diff to review.\n",
        total_lines.to_string().yellow().bold()
    );

    // AgentShield DLP check on review diff
    let dlp_verdict = AgentShieldScanner::scan_outbound_dlp(&diff);
    if let crate::ecc::AgentShieldVerdict::Block { reason, threat_level } = dlp_verdict {
        eprintln!("\n{}", "🚨 AgentShield Blocked Outbound Code Review:".bold().red());
        eprintln!("Threat Level: {:?} | Reason: {}", threat_level, reason.yellow());
        return Err(TagisanError::Security(format!("AgentShield DLP: Sensitive code blocked from outbound review ({})", reason)));
    }

    if diff.len() > 35_000 {
        diff = format!("{}\n\n[... Diff truncated at 35,000 characters ...]", &diff[..35_000]);
    }

    let review_prompt = format!(
        "You are the Tagisan Peer Review Triad conducting a rigorous code review.\n\
        Roles:\n\
        1. 🏗️ ARCHITECT: Analyze code structure, abstractions, API design, and maintainability.\n\
        2. 🛡️ SECURITY AUDITOR: Check for memory safety, credential leaks, unhandled errors, injection, and resource exhaustion.\n\
        3. 🧪 QA LEAD: Check edge cases, missing test scenarios, error propagation, and regression risks.\n\
        4. 🇵🇭 LAKANDIWA SYNTHESIS & VERDICT:\n\
           - Final Verdict: [APPROVE | REQUEST_CHANGES | REJECT]\n\
           - Overall Risk Rating: [Low | Medium | High | Critical] (Score: X/10)\n\
           - Actionable Remediation Checklist (Top 3 priority items).\n\n\
        GIT DIFF:\n```diff\n{}\n```",
        diff
    );

    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;

    println!(
        "{} [{}: {}]...\n",
        "🇵🇭 Convening Peer Review Triad".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );

    let req = CompletionRequest::new(model_name, review_prompt)
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    let mut stream = prov.stream(req).await?;
    let start = std::time::Instant::now();

    while let Some(chunk_res) = stream.next().await {
        match chunk_res {
            Ok(chunk) => {
                match chunk.delta {
                    StreamChunkDelta::Text(text) => {
                        print!("{}", text);
                        io::stdout().flush().ok();
                    }
                    StreamChunkDelta::Thinking(th) => {
                        print!("{}", th.dimmed());
                        io::stdout().flush().ok();
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("\n{}: Review stream error: {}", "Error".red().bold(), e);
                break;
            }
        }
    }

    let elapsed = start.elapsed();
    println!(
        "\n\n{} Peer Review completed in {:.2}s",
        "✔".green().bold(),
        elapsed.as_secs_f64()
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Proactive Background Sentinel Watchdog (tgs watch)
// ---------------------------------------------------------------------------

pub async fn handle_watch_command(
    path: String,
    cli_max_budget: f64,
) -> Result<()> {
    let watch_dir = PathBuf::from(&path);
    if !watch_dir.exists() {
        return Err(TagisanError::Execution(format!("Path does not exist: {:?}", watch_dir)));
    }

    let project_type = detect_project_type(&watch_dir);
    println!("{}", "══════════════════════════════════════════════════════════════".cyan().bold());
    println!("{}", "  ⚡ TGS PROACTIVE SENTINEL & FILE WATCHDOG".bold().yellow());
    println!("{}", "══════════════════════════════════════════════════════════════".cyan().bold());
    println!("Project Type: {:?}", project_type);
    println!("Watch Directory: {:?}", watch_dir.canonicalize().unwrap_or(watch_dir.clone()));
    println!("Monitoring filesystem for changes. Press Ctrl+C to stop.\n");

    let mut last_scan_time = std::time::SystemTime::now();

    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Scan for recent modifications
        let changed = has_modified_files(&watch_dir, last_scan_time);
        if !changed {
            continue;
        }

        last_scan_time = std::time::SystemTime::now();
        let timestamp = Local::now().format("%H:%M:%S");

        println!(
            "\n[{}] {} Running validation checks...",
            timestamp.to_string().dimmed(),
            "🔄 Changes detected:".cyan().bold()
        );

        // Run validation based on project type
        let check_res = match project_type {
            ProjectType::Rust => {
                Command::new("cargo")
                    .current_dir(&watch_dir)
                    .args(["check", "--message-format=short"])
                    .output()
            }
            ProjectType::TypeScript => {
                if Command::new("bun").arg("--version").output().is_ok() {
                    Command::new("bun").current_dir(&watch_dir).args(["test"]).output()
                } else {
                    Command::new("npm").current_dir(&watch_dir).args(["test"]).output()
                }
            }
            ProjectType::Python => {
                Command::new("python3").current_dir(&watch_dir).args(["-m", "py_compile"]).output()
            }
            _ => {
                Command::new("git").current_dir(&watch_dir).args(["status", "--short"]).output()
            }
        };

        match check_res {
            Ok(output) => {
                if output.status.success() {
                    println!(
                        "[{}] {} All checks and invariants satisfied.",
                        timestamp.to_string().dimmed(),
                        "✔ BUILD PASSING:".green().bold()
                    );
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let combined_errors = format!("{}\n{}", stdout, stderr);

                    println!(
                        "[{}] {} Errors found during validation!",
                        timestamp.to_string().dimmed(),
                        "✗ BUILD FAILED:".red().bold()
                    );

                    // Emit desktop notification
                    let _ = Command::new("notify-send")
                        .args([
                            "-a", "TGS Sentinel",
                            "-i", "dialog-error",
                            "TGS Sentinel: Build Failed",
                            "Computing AI diagnosis and autofix...",
                        ])
                        .status();

                    // Query TGS for instant diagnosis
                    println!("{}", "🧠 Diagnosing error with TGS AI...".yellow());
                    let diag_prompt = format!(
                        "A background build failure occurred in a {:?} project.\n\
                        Error log:\n```\n{}\n```\n\
                        Provide a concise diagnosis (1-2 sentences) and the exact corrected code snippet or command to fix it.",
                        project_type,
                        &combined_errors[..combined_errors.len().min(4000)]
                    );

                    let ctx = build_engine_context(cli_max_budget);
                    if let Ok((_, model_name, prov)) = resolve_provider_and_model(&ctx, "auto", None) {
                        let req = CompletionRequest::new(model_name, diag_prompt);
                        if let Ok(res) = prov.complete(req).await {
                            let text = res.message.extract_text();
                            println!("\n{}", "─".repeat(50).yellow());
                            println!("{}", text.trim());
                            println!("{}\n", "─".repeat(50).yellow());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to execute validation tool: {}", e);
            }
        }
    }
}

fn has_modified_files(dir: &Path, since: std::time::SystemTime) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        let p = entry.path();
        let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
            continue;
        }

        if p.is_dir() {
            if has_modified_files(&p, since) {
                return true;
            }
        } else if let Ok(meta) = p.metadata() {
            if let Ok(mod_time) = meta.modified() {
                if mod_time > since {
                    return true;
                }
            }
        }
    }
    false
}
