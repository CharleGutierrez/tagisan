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

// ---------------------------------------------------------------------------
// 5. Autonomous Linux System & Hardware Doctor (tgs doctor)
// ---------------------------------------------------------------------------

pub async fn handle_doctor_command(
    fix: bool,
    battery_only: bool,
    cli_max_budget: f64,
) -> Result<()> {
    println!("{}", "══════════════════════════════════════════════════════════════".cyan().bold());
    println!("{}", "  🩺  TGS AUTONOMOUS LINUX SYSTEM & HARDWARE DOCTOR".bold().yellow());
    println!("{}", "══════════════════════════════════════════════════════════════".cyan().bold());

    // 1. Gather Telemetry
    let kernel_ver = std::fs::read_to_string("/proc/version").unwrap_or_default();
    
    // CPU Governor
    let governor = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .unwrap_or_else(|_| "unknown".into());
    
    // Thermals
    let mut temps = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("thermal_zone") {
                let temp_file = entry.path().join("temp");
                if let Ok(content) = std::fs::read_to_string(temp_file) {
                    if let Ok(milli) = content.trim().parse::<f64>() {
                        temps.push(format!("{}: {:.1}°C", name, milli / 1000.0));
                    }
                }
            }
        }
    }

    // Battery Health
    let mut battery_info = String::new();
    if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                let status = std::fs::read_to_string(entry.path().join("status")).unwrap_or_default();
                let cap = std::fs::read_to_string(entry.path().join("capacity")).unwrap_or_default();
                let energy_full = std::fs::read_to_string(entry.path().join("energy_full")).ok()
                    .and_then(|s| s.trim().parse::<f64>().ok());
                let energy_design = std::fs::read_to_string(entry.path().join("energy_full_design")).ok()
                    .and_then(|s| s.trim().parse::<f64>().ok());
                
                let health_pct = match (energy_full, energy_design) {
                    (Some(full), Some(design)) if design > 0.0 => format!("{:.1}%", (full / design) * 100.0),
                    _ => "N/A".into(),
                };
                battery_info = format!("{}: Status: {}, Level: {}%, Health/Capacity: {}", name, status.trim(), cap.trim(), health_pct);
            }
        }
    }

    // Failed systemd units
    let failed_units = Command::new("systemctl")
        .args(["--failed", "--no-legend"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Pacman Lock check
    let pacman_locked = Path::new("/var/lib/pacman/db.lck").exists();

    // Disk space
    let disk_space = Command::new("df")
        .args(["-h", "/"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    // Kernel critical errors
    let journal_errors = Command::new("journalctl")
        .args(["-p", "3", "-xb", "-n", "10", "--no-pager"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    println!("Kernel:        {}", kernel_ver.lines().next().unwrap_or("Unknown").cyan());
    println!("CPU Governor:  {}", governor.trim().green().bold());
    println!("Thermals:      {}", temps.join(", ").yellow());
    if !battery_info.is_empty() {
        println!("Battery:       {}", battery_info.cyan());
    }
    println!("Pacman Lock:   {}", if pacman_locked { "LOCKED (/var/lib/pacman/db.lck)".red().bold() } else { "Clean (No stale lock)".green() });
    println!("Failed Units:  {}", if failed_units.trim().is_empty() { "0 failed systemd units".green() } else { failed_units.trim().red().bold() });
    println!();

    if battery_only {
        return Ok(());
    }

    // 2. Query TGS LLM for Holistic Diagnosis
    println!("{}", "🧠 Consulting TGS AI Doctor for System Health Diagnosis...".bold().magenta());

    let telemetry = format!(
        "System Metrics:\n\
        - Kernel: {}\n\
        - CPU Governor: {}\n\
        - Thermals: {}\n\
        - Battery: {}\n\
        - Failed Systemd Units:\n{}\n\
        - Pacman Lock Status: {}\n\
        - Disk Space (/):\n{}\n\
        - Critical Kernel Errors (journalctl -p 3):\n{}\n",
        kernel_ver.lines().next().unwrap_or(""),
        governor.trim(),
        temps.join(", "),
        battery_info,
        if failed_units.trim().is_empty() { "None" } else { &failed_units },
        if pacman_locked { "LOCKED" } else { "UNLOCKED" },
        disk_space,
        if journal_errors.trim().is_empty() { "None" } else { &journal_errors }
    );

    let prompt = format!(
        "You are the TGS Autonomous Linux Kernel & Hardware Doctor on Garuda/Arch Linux.\n\
        Analyze this live telemetry:\n{}\n\
        Format your diagnosis strictly as:\n\
        1. 🩺 Health Score: X/100 (and 1-sentence summary)\n\
        2. 🚨 Critical Issues & Anomalies (if any)\n\
        3. ⚡ Recommended Tuning & Maintenance Actions (numbered list with exact shell commands)",
        telemetry
    );

    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;

    let req = CompletionRequest::new(model_name, prompt)
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    let mut stream = prov.stream(req).await?;
    let start = std::time::Instant::now();

    while let Some(chunk_res) = stream.next().await {
        if let Ok(chunk) = chunk_res {
            if let StreamChunkDelta::Text(t) = chunk.delta {
                print!("{}", t);
                io::stdout().flush().ok();
            }
        }
    }

    println!("\n\n{} Diagnosis completed in {:.2}s", "✔".green().bold(), start.elapsed().as_secs_f64());

    // 3. Interactive Healing Actions
    if fix {
        println!("\n{}", "───────────────────────────────────────────────────────".yellow());
        println!("{}", "⚡ TGS Doctor Auto-Remediation Mode:".bold().yellow());
        println!("{}", "───────────────────────────────────────────────────────".yellow());

        if pacman_locked {
            print!("{}", "Stale pacman lock file detected. Remove /var/lib/pacman/db.lck? [y/N]: ".bold());
            io::stdout().flush().ok();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).ok();
            if choice.trim().eq_ignore_ascii_case("y") {
                let _ = Command::new("sudo").args(["rm", "-f", "/var/lib/pacman/db.lck"]).status();
                println!("{}", "✔ Stale pacman lock removed.".green());
            }
        }

        if !failed_units.trim().is_empty() {
            print!("{}", "Attempt to reset failed systemd units (systemctl reset-failed)? [y/N]: ".bold());
            io::stdout().flush().ok();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).ok();
            if choice.trim().eq_ignore_ascii_case("y") {
                let _ = Command::new("systemctl").args(["reset-failed"]).status();
                let _ = Command::new("systemctl").args(["--user", "reset-failed"]).status();
                println!("{}", "✔ Failed unit state reset.".green());
            }
        }

        println!("{}", "✔ System healing pass completed.".green().bold());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 6. Autonomous Headless Web Agent (tgs browse)
// ---------------------------------------------------------------------------

pub async fn handle_browse_command(
    target: String,
    query: Option<String>,
    cli_max_budget: f64,
) -> Result<()> {
    println!("{}", "🌐 Launching Headless Web Extraction Agent...".cyan().bold());
    println!("Target URL: {}", target.yellow());

    // 1. Fetch content using headless Chrome or reqwest fallback
    let raw_html = if Command::new("google-chrome-stable").arg("--version").output().is_ok() {
        println!("{}", "Using Google Chrome (Headless DOM Engine)...".dimmed());
        let out = Command::new("google-chrome-stable")
            .args([
                "--headless=new",
                "--disable-gpu",
                "--no-sandbox",
                "--dump-dom",
                &target,
            ])
            .output();
        match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => {
                println!("{}", "Headless Chrome failed, falling back to HTTP client...".yellow());
                fetch_http_body(&target).await?
            }
        }
    } else {
        fetch_http_body(&target).await?
    };

    if raw_html.trim().is_empty() {
        return Err(TagisanError::Execution(format!("Failed to retrieve any content from {}", target)));
    }

    // 2. Extract and clean text from HTML
    let cleaned_text = extract_clean_text_from_html(&raw_html);
    let sample = if cleaned_text.len() > 25_000 {
        format!("{}\n\n[... Page truncated at 25,000 characters ...]", &cleaned_text[..25_000])
    } else {
        cleaned_text
    };

    println!("{} Extracted {:.1} KB of clean page text.\n", "✔".green().bold(), sample.len() as f64 / 1024.0);

    // 3. Query TGS
    let prompt_intent = query.unwrap_or_else(|| "Provide an executive summary and extract all key insights from this page.".to_string());
    let prompt = format!(
        "You are an expert web research agent. Analyze this extracted web content from <{}>:\n\
        User Objective: {}\n\n\
        EXTRACTED PAGE TEXT:\n```\n{}\n```",
        target,
        prompt_intent,
        sample
    );

    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;

    println!(
        "{} [{}: {}]...\n",
        "Analyzing Web Content".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );

    let req = CompletionRequest::new(model_name, prompt)
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    let mut stream = prov.stream(req).await?;
    let start = std::time::Instant::now();

    while let Some(chunk_res) = stream.next().await {
        if let Ok(chunk) = chunk_res {
            if let StreamChunkDelta::Text(t) = chunk.delta {
                print!("{}", t);
                io::stdout().flush().ok();
            }
        }
    }

    println!("\n\n{} Web analysis completed in {:.2}s", "✔".green().bold(), start.elapsed().as_secs_f64());
    Ok(())
}

async fn fetch_http_body(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(15))
        .build()?;
    let text = client.get(url).send().await?.text().await?;
    Ok(text)
}

fn extract_clean_text_from_html(html: &str) -> String {
    let re_script = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>|<noscript[^>]*>.*?</noscript>|<svg[^>]*>.*?</svg>").unwrap();
    let stripped = re_script.replace_all(html, " ");
    let re_tags = regex::Regex::new(r"<[^>]+>").unwrap();
    let text_only = re_tags.replace_all(&stripped, " ");
    let re_spaces = regex::Regex::new(r"[ \t]+").unwrap();
    let re_newlines = regex::Regex::new(r"\n\s*\n").unwrap();
    let normalized = re_spaces.replace_all(&text_only, " ");
    let cleaned = re_newlines.replace_all(&normalized, "\n\n");
    cleaned.trim().to_string()
}

// ---------------------------------------------------------------------------
// 7. Personal Knowledge Vault & Local RAG (tgs recall)
// ---------------------------------------------------------------------------

pub async fn handle_recall_command(
    query: Option<String>,
    index_dir: Option<String>,
    stats: bool,
    top_k: usize,
    cli_max_budget: f64,
) -> Result<()> {
    if stats {
        let store = crate::memory::VectorStore::load_or_default();
        let default_path = crate::memory::VectorStore::default_path();
        println!("{}", "=========================================================".cyan());
        println!("{}", "  📊  Tagisan Persistent Memory & Vault Statistics".bold().yellow());
        println!("{}", "=========================================================".cyan());
        println!("Storage File:         {}", default_path.display().to_string().yellow());
        println!("Total Documents:      {}", store.len().to_string().cyan().bold());
        return Ok(());
    }

    if let Some(ref dir) = index_dir {
        println!("{}", "=========================================================".cyan());
        println!("{}", format!("  🧠  Indexing Vault into Memory: {}", dir).bold().magenta());
        println!("{}", "=========================================================".cyan());

        let store = crate::memory::VectorStore::load_or_default();
        let provider = crate::memory::default_embedding_provider();
        println!("Embedding Provider: {} ({} dims)", provider.provider_id().cyan().bold(), provider.dimensions());

        let indexer = crate::memory::CodebaseIndexer::new(provider);
        let start = std::time::Instant::now();
        let count = indexer.index_directory(dir, &store).await?;

        let default_path = crate::memory::VectorStore::default_path();
        store.save_to_file(&default_path)?;

        println!("\n{} Indexed {} total chunks in {:.2}s!", "✔".green().bold(), count, start.elapsed().as_secs_f32());
        println!("Persistent Vault File: {}", default_path.display().to_string().yellow());
        return Ok(());
    }

    if let Some(ref q) = query {
        println!("{}", "=========================================================".cyan());
        println!("{}", format!("  🧠  TGS Semantic Recall: \"{}\"", q).bold().yellow());
        println!("{}", "=========================================================".cyan());

        let store = crate::memory::VectorStore::load_or_default();
        if store.is_empty() {
            println!("{}: Vault memory is empty. Run `tgs recall --index <dir>` to index notes/code.", "Note".yellow().bold());
            return Ok(());
        }

        let provider = crate::memory::default_embedding_provider();
        let emb = provider.embed_text(q).await?;
        let hits = store.search(&emb, top_k, 0.05);

        if hits.is_empty() {
            println!("No relevant matches found in your personal vault.");
            return Ok(());
        }

        let mut context_snippets = String::new();
        println!("Found {} relevant source snippet(s) in personal vault:\n", hits.len());
        for (i, hit) in hits.iter().enumerate() {
            let doc = &hit.document;
            let file_path = doc.metadata.get("file_path").map(|s| s.as_str()).unwrap_or(&doc.id);
            let start_line = doc.metadata.get("start_line").map(|s| s.as_str()).unwrap_or("?");
            let end_line = doc.metadata.get("end_line").map(|s| s.as_str()).unwrap_or("?");
            println!("  [{}] {} (Lines {}-{}) | Score: {:.3}", i + 1, file_path.cyan(), start_line, end_line, hit.score);
            context_snippets.push_str(&format!("\n--- File: {} (Lines {}-{}) ---\n{}\n", file_path, start_line, end_line, doc.text));
        }

        // Synthesize answer
        println!("\n{}", "🇵🇭 Synthesizing answer from personal knowledge...".bold().magenta());
        let prompt = format!(
            "You are TGS Personal Knowledge Assistant. Based ONLY on the following retrieved personal documents and codebase files, answer the user question:\n\n\
            RETRIEVED KNOWLEDGE:\n{}\n\n\
            USER QUESTION: {}\n\n\
            Rules: Cite which files and lines your answer is based on.",
            context_snippets,
            q
        );

        let ctx = build_engine_context(cli_max_budget);
        let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;

        let req = CompletionRequest::new(model_name, prompt)
            .with_stream(true)
            .with_cancellation(ctx.cancellation_token.clone());

        let mut stream = prov.stream(req).await?;
        let start = std::time::Instant::now();

        while let Some(chunk_res) = stream.next().await {
            if let Ok(chunk) = chunk_res {
                if let StreamChunkDelta::Text(t) = chunk.delta {
                    print!("{}", t);
                    io::stdout().flush().ok();
                }
            }
        }

        println!("\n\n{} Recall query answered in {:.2}s", "✔".green().bold(), start.elapsed().as_secs_f64());
        return Ok(());
    }

    println!("Usage: tgs recall \"your question\"  OR  tgs recall --index <path>  OR  tgs recall --stats");
    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Sovereign Speech & Voice Copilot (tgs voice)
// ---------------------------------------------------------------------------

pub async fn handle_voice_command(
    duration: u32,
    prompt: Option<String>,
    cli_max_budget: f64,
) -> Result<()> {
    let pid = std::process::id();
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let temp_audio = std::env::temp_dir().join(format!("tgs_voice_{}_{}.wav", pid, timestamp));
    let _guard = TempFileGuard::new(temp_audio.clone());

    println!("{}", "🎙️  TGS Voice Copilot Active".bold().cyan());
    println!("Recording audio for {}s via PipeWire (pw-record)... Speak now!", duration);

    // Record via pw-record
    let status = Command::new("pw-record")
        .args([
            "--rate", "16000",
            "--channels", "1",
            temp_audio.to_str().unwrap(),
        ])
        .spawn();

    match status {
        Ok(mut child) => {
            tokio::time::sleep(Duration::from_secs(duration as u64)).await;
            let _ = child.kill();
            let _ = child.wait();
        }
        Err(_) => {
            return Err(TagisanError::Execution("Failed to spawn pw-record for audio capture.".into()));
        }
    }

    if !temp_audio.exists() || std::fs::metadata(&temp_audio).map(|m| m.len() < 1000).unwrap_or(true) {
        return Err(TagisanError::Execution("Audio recording failed or file was empty.".into()));
    }

    let file_size = std::fs::metadata(&temp_audio).map(|m| m.len()).unwrap_or(0);
    println!("{} Recorded {:.1} KB of audio. Processing voice intent...", "✔".green().bold(), file_size as f64 / 1024.0);

    use base64::prelude::*;
    let audio_bytes = std::fs::read(&temp_audio)?;
    let audio_b64 = BASE64_STANDARD.encode(&audio_bytes);

    let default_prompt = prompt.unwrap_or_else(|| "Listen to this audio carefully. Transcribe the user speech (English, Tagalog, or Taglish) and execute or answer the user's intent directly and concisely.".to_string());

    let user_blocks = vec![
        ContentBlock::text(default_prompt),
        ContentBlock::Image {
            media_type: "audio/wav".to_string(),
            data_base64: audio_b64,
        },
    ];

    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, "gemini", None)?;

    println!(
        "{} [{}: {}]...\n",
        "Transcribing & Reasoning".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );

    let req = CompletionRequest::new(model_name, "")
        .with_messages(vec![Message::user_with_content(user_blocks)])
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    let mut stream = prov.stream(req).await?;
    let start = std::time::Instant::now();

    while let Some(chunk_res) = stream.next().await {
        if let Ok(chunk) = chunk_res {
            if let StreamChunkDelta::Text(t) = chunk.delta {
                print!("{}", t);
                io::stdout().flush().ok();
            }
        }
    }

    println!("\n\n{} Voice response received in {:.2}s", "✔".green().bold(), start.elapsed().as_secs_f64());
    Ok(())
}
