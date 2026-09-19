use crate::agent::{AutonomousAgent, WorktreeSandbox};
use crate::engine::EngineContext;
use crate::error::Result;
use crate::swarm::session::{SessionRecord, SessionStore};
use crate::tui::Spinner;
use crate::types::{Message, Role};
use colored::Colorize;
use std::io::{self, Write};
use std::time::Instant;

/// Slash commands supported inside the Interactive Agent REPL
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    Help,
    Agent(String),
    Skill(String),
    Forge(String),
    Model(String),
    Tools,
    Memory,
    Sandbox,
    Save(Option<String>),
    Load(String),
    Clear,
    Budget,
    History,
    Bun(String),
    Vella(String),
    Plan(String),
    Goal(String),
    Exit,
    UserPrompt(String),
}

/// Interactive REPL managing multi-turn agent sessions, slash commands, and persistence
pub struct InteractiveRepl {
    pub agent: AutonomousAgent,
    pub session_record: SessionRecord,
    pub session_store: SessionStore,
    pub context: EngineContext,
    pub sandbox: Option<WorktreeSandbox>,
    pub running: bool,
}

impl InteractiveRepl {
    pub fn new(
        agent: AutonomousAgent,
        session_id: impl Into<String>,
        model: impl Into<String>,
        context: EngineContext,
    ) -> Self {
        let sid = session_id.into();
        let m = model.into();
        let session_record = SessionRecord::new(sid.clone(), format!("Interactive Session {sid}"), m);
        Self {
            agent,
            session_record,
            session_store: SessionStore::new(),
            context,
            sandbox: None,
            running: false,
        }
    }

    pub fn with_sandbox(mut self, sandbox: WorktreeSandbox) -> Self {
        self.agent = self.agent.with_working_dir(sandbox.path());
        self.sandbox = Some(sandbox);
        self
    }

    /// Parse user input into structured ReplCommand
    pub fn parse_command(input: &str) -> ReplCommand {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return ReplCommand::UserPrompt(String::new());
        }

        if !trimmed.starts_with('/') {
            return ReplCommand::UserPrompt(trimmed.to_string());
        }

        let (cmd, arg) = if let Some(space_idx) = trimmed.find(char::is_whitespace) {
            (&trimmed[..space_idx], trimmed[space_idx..].trim().to_string())
        } else {
            (trimmed, String::new())
        };

        match cmd.to_lowercase().as_str() {
            "/help" | "/h" | "/?" => ReplCommand::Help,
            "/agent" | "/persona" => ReplCommand::Agent(arg),
            "/skill" => ReplCommand::Skill(arg),
            "/forge" => ReplCommand::Forge(arg),
            "/model" | "/m" => ReplCommand::Model(arg),
            "/tools" | "/t" => ReplCommand::Tools,
            "/memory" | "/mem" => ReplCommand::Memory,
            "/sandbox" | "/box" => ReplCommand::Sandbox,
            "/bun" => ReplCommand::Bun(arg),
            "/vella" => ReplCommand::Vella(arg),
            "/save" | "/s" => {
                let name = if arg.is_empty() { None } else { Some(arg) };
                ReplCommand::Save(name)
            }
            "/load" | "/l" => ReplCommand::Load(arg),
            "/clear" | "/c" => ReplCommand::Clear,
            "/budget" | "/b" => ReplCommand::Budget,
            "/history" | "/hist" => ReplCommand::History,
            "/plan" | "/p" => ReplCommand::Plan(arg),
            "/goal" | "/g" => ReplCommand::Goal(arg),
            "/exit" | "/quit" | "/q" => ReplCommand::Exit,
            _ => ReplCommand::UserPrompt(trimmed.to_string()),
        }
    }

    /// Execute a single turn on the agent session with live animation and appealing formatting
    pub async fn run_turn(&mut self, prompt: &str) -> Result<String> {
        if prompt.trim().is_empty() {
            return Ok(String::new());
        }

        self.session_record.add_message(Message::user(prompt));

        // Reconstruct active ChatSession from session_record history
        let mut chat_session = crate::types::ChatSession::new();
        if let Some(ref sys) = self.agent.system_prompt {
            chat_session = chat_session.with_system(sys.clone());
        }
        for msg in &self.session_record.messages {
            chat_session.add_message(msg.clone());
        }

        let start_time = Instant::now();

        // 🌟 Start animated spinner with vibrant cycling colors and live timer
        let spinner_msg = format!("🧠 Thinking & synthesizing with {}...", self.agent.model.bold().cyan());
        let spinner = Spinner::start(spinner_msg);

        // Execute agent turn
        let exec_result = self.agent.execute_session(&mut chat_session, &self.context).await;

        match &exec_result {
            Ok(_) => spinner.stop(),
            Err(e) => {
                spinner.failure(format!("Generation failed: {e}"));
            }
        }
        let result = exec_result?;

        let elapsed = start_time.elapsed().as_secs_f64();
        let prompt_tokens = result.total_usage.prompt_tokens;
        let comp_tokens = result.total_usage.completion_tokens;

        // Update session record with results
        self.session_record.messages = chat_session.history;
        self.session_record.total_usage.prompt_tokens += prompt_tokens;
        self.session_record.total_usage.completion_tokens += comp_tokens;
        if let Some(rt) = result.total_usage.reasoning_tokens {
            self.session_record.total_usage.reasoning_tokens =
                Some(self.session_record.total_usage.reasoning_tokens.unwrap_or(0) + rt);
        }
        self.session_record.total_cost_usd = self.context.budget_tracker.current_spent_usd();

        // Auto-save checkpoint
        let _ = self.session_store.save(&self.session_record);

        // Format result with appealing colors, card frame, and emojis
        let formatted = format_appealing_repl_response(
            &result.final_answer,
            &self.agent.model,
            elapsed,
            prompt_tokens,
            comp_tokens,
            self.session_record.total_cost_usd,
            &self.session_record.id,
        );

        Ok(formatted)
    }

    /// Process a parsed command
    pub async fn execute_command(&mut self, cmd: ReplCommand) -> Result<Option<String>> {
        match cmd {
            ReplCommand::Help => {
                let help_text = format!(
                    "{}\n\
                    {}       Display this help manual\n\
                    {} Switch agent persona (e.g. architect, tdd-engineer)\n\
                    {}   Dynamically attach an ECC skill\n\
                    {}   Forge a skill live from binary, code, or MCP spec\n\
                    {}   Switch model name\n\
                    {}       List registered tools\n\
                    {}      Display memory stats or search memory\n\
                    {}     Inspect git worktree sandbox status & diff\n\
                    {}  Evaluate TypeScript/JavaScript on the fly via Bun\n\
                    {} Interact with Vella Sovereign OS & hardware E-Stop\n\
                    {}   Save session checkpoint\n\
                    {}     Load previously saved session\n\
                    {}      Clear conversational history\n\
                    {}      Display token usage and USD cost\n\
                    {}    Show conversational history overview\n\
                    {}       Exit interactive session\n",
                    "Tagisan Interactive Agent REPL Commands:".bold().cyan(),
                    "/help".bold().green(),
                    "/agent <name>".bold().green(),
                    "/skill <name>".bold().green(),
                    "/forge <path>".bold().green(),
                    "/model <name>".bold().green(),
                    "/tools".bold().green(),
                    "/memory".bold().green(),
                    "/sandbox".bold().green(),
                    "/bun <ts_code>".bold().green(),
                    "/vella <cmd>".bold().green(),
                    "/save [id]".bold().green(),
                    "/load <id>".bold().green(),
                    "/clear".bold().green(),
                    "/budget".bold().green(),
                    "/history".bold().green(),
                    "/exit".bold().green(),
                );
                Ok(Some(help_text))
            }
            ReplCommand::Agent(name) => {
                if name.is_empty() {
                    return Ok(Some("Usage: /agent <persona_name> (e.g. architect, tdd-engineer, code-reviewer, security-auditor)".to_string()));
                }
                if let Some(preset) = crate::ecc::find_preset(&name) {
                    self.agent.system_prompt = Some(preset.system_prompt.clone());
                    self.session_record.agent_persona = Some(preset.name.clone());
                    Ok(Some(format!("Switched persona to '{}' ({})", preset.name.bold().green(), preset.description)))
                } else {
                    Ok(Some(format!("Agent persona '{}' not recognized.", name.bold().red())))
                }
            }
            ReplCommand::Skill(name) => {
                if name.is_empty() {
                    return Ok(Some("Usage: /skill <skill_name> (e.g. tdd-workflow, security-review, api-design)".to_string()));
                }
                if let Some(skill) = crate::ecc::find_built_in_skill(&name) {
                    let mut current_sys = self.agent.system_prompt.clone().unwrap_or_default();
                    current_sys.push_str(&format!("\n\n## Attached Skill: {}\n{}", skill.name, skill.instructions));
                    self.agent.system_prompt = Some(current_sys);
                    Ok(Some(format!("Attached skill '{}' to current agent session.", skill.name.bold().cyan())))
                } else {
                    Ok(Some(format!("Skill '{}' not found in catalog.", name.bold().red())))
                }
            }
            ReplCommand::Forge(target) => {
                if target.is_empty() {
                    return Ok(Some("Usage: /forge <path_to_code_or_mcp_or_binary> (e.g. /forge curl, /forge script.py, /forge schema.json)".to_string()));
                }
                let target_path = std::path::Path::new(&target);

                // 1. MCP Spec JSON
                let is_json = target.ends_with(".json") || (target_path.is_file() && target.ends_with(".json"));
                if is_json {
                    match crate::harness::mcp_importer::McpImporter::import(&target, None, None, true).await {
                        Ok(res) => {
                            if let Some(ref skill_dir) = res.skill_path {
                                let skill_md_file = skill_dir.join("SKILL.md");
                                if let Ok(content) = std::fs::read_to_string(&skill_md_file) {
                                    let mut current_sys = self.agent.system_prompt.clone().unwrap_or_default();
                                    current_sys.push_str(&format!("\n\n## Forged MCP Skill: {}\n{}", res.harness_name, content));
                                    self.agent.system_prompt = Some(current_sys);
                                }
                            }
                            return Ok(Some(format!(
                                "✨ Forged MCP skill '{}' ({} tools) and attached to active session!",
                                res.harness_name.bold().green(),
                                res.tools_imported
                            )));
                        }
                        Err(e) => {
                            return Ok(Some(format!("✖ Failed to forge MCP tool: {}", e)));
                        }
                    }
                }

                // 2. Source Code or Codebase Directory
                let is_source_file = target.ends_with(".py") || target.ends_with(".rs") || target.ends_with(".ts") || target.ends_with(".js") || (target_path.is_dir() && !target_path.is_file());
                if is_source_file && target_path.exists() {
                    let options = crate::harness::pipeline::HarnessPipelineOptions::new(target_path.to_path_buf())
                        .with_install(true)
                        .with_run_tests(false);
                    let pipeline = crate::harness::pipeline::HarnessPipeline::new();
                    match pipeline.execute(options).await {
                        Ok(res) => {
                            if let Some(ref skill_dir) = res.installed_path {
                                let skill_md_file = skill_dir.join("SKILL.md");
                                if let Ok(content) = std::fs::read_to_string(&skill_md_file) {
                                    let mut current_sys = self.agent.system_prompt.clone().unwrap_or_default();
                                    current_sys.push_str(&format!("\n\n## Forged Code Skill: {}\n{}", res.spec.name, content));
                                    self.agent.system_prompt = Some(current_sys);
                                }
                            }
                            return Ok(Some(format!(
                                "✨ Forged CLI harness '{}' ({} subcommands) and attached to active session!",
                                res.spec.name.bold().green(),
                                res.spec.commands.len()
                            )));
                        }
                        Err(e) => {
                            return Ok(Some(format!("✖ Failed to forge codebase harness: {}", e)));
                        }
                    }
                }

                // 3. System Binary Ingestion (e.g. curl, git, jq)
                match crate::harness::ingester::BinaryIngester::ingest(&target, None, None, true).await {
                    Ok(res) => {
                        if let Some(ref skill_dir) = res.skill_path {
                            let skill_md_file = skill_dir.join("SKILL.md");
                            if let Ok(content) = std::fs::read_to_string(&skill_md_file) {
                                let mut current_sys = self.agent.system_prompt.clone().unwrap_or_default();
                                current_sys.push_str(&format!("\n\n## Forged Binary Skill: {}\n{}", res.binary_name, content));
                                self.agent.system_prompt = Some(current_sys);
                            }
                        }
                        Ok(Some(format!(
                            "✨ Forged binary skill '{}' ({} commands) from '{}' and attached to active session!",
                            res.binary_name.bold().green(),
                            res.subcommands_count,
                            res.binary_path.display()
                        )))
                    }
                    Err(e) => {
                        Ok(Some(format!("✖ Failed to forge binary '{}': {}", target, e)))
                    }
                }
            }
            ReplCommand::Model(name) => {
                if name.is_empty() {
                    return Ok(Some(format!("Current model: {}", self.agent.model.bold().yellow())));
                }
                self.agent.model = name.clone();
                self.session_record.model = name.clone();
                Ok(Some(format!("Switched active model to '{}'", name.bold().green())))
            }
            ReplCommand::Tools => {
                let defs = self.agent.tools.definitions();
                let mut out = format!("Registered Tools ({}):\n", defs.len());
                for d in defs {
                    out.push_str(&format!("  - {}: {}\n", d.name.bold().cyan(), d.description));
                }
                Ok(Some(out))
            }
            ReplCommand::Memory => {
                if let Some(ref mem) = self.agent.memory {
                    let stats = mem.stats(None);
                    Ok(Some(format!(
                        "Memory Store: {} documents indexed, {} embedding dimensions.",
                        stats.total_documents.to_string().bold().green(),
                        stats.embedding_dimensions.to_string().bold().cyan()
                    )))
                } else {
                    Ok(Some("Long-term memory is not enabled for this session.".to_string()))
                }
            }
            ReplCommand::Sandbox => {
                if let Some(ref sb) = self.sandbox {
                    let diff = sb.diff().unwrap_or_default();
                    let diff_display = if diff.trim().is_empty() {
                        "No uncommitted changes in sandbox.".italic().to_string()
                    } else {
                        diff
                    };
                    Ok(Some(format!(
                        "Active Sandbox Path: {:?}\nBase Commit: {}\nDiff:\n{}",
                        sb.path(),
                        sb.base_commit(),
                        diff_display
                    )))
                } else {
                    Ok(Some("Git worktree sandbox is not active for this session.".to_string()))
                }
            }
            ReplCommand::Bun(code) => {
                if code.trim().is_empty() {
                    return Ok(Some("Usage: /bun <ts_code> (e.g. /bun console.log(Math.sqrt(42)))".to_string()));
                }
                let runtime = match crate::bun::BunRuntime::new() {
                    Ok(rt) => rt,
                    Err(e) => return Ok(Some(format!("Bun runtime not available: {e}"))),
                };
                let cwd = self.sandbox.as_ref().map(|s| s.path().to_path_buf());
                match runtime.eval(&code, std::time::Duration::from_secs(30), None, cwd).await {
                    Ok(res) => {
                        let mut output = format!("Bun execution completed in {}ms (exit code: {})\n", res.duration_ms, res.exit_code);
                        if !res.stdout.is_empty() {
                            output.push_str("--- STDOUT ---\n");
                            output.push_str(&res.stdout);
                            if !res.stdout.ends_with('\n') {
                                output.push('\n');
                            }
                        }
                        if !res.stderr.is_empty() {
                            output.push_str("--- STDERR ---\n");
                            output.push_str(&res.stderr);
                            if !res.stderr.ends_with('\n') {
                                output.push('\n');
                            }
                        }
                        if res.stdout.is_empty() && res.stderr.is_empty() {
                            output.push_str("(no output)");
                        }
                        Ok(Some(output))
                    }
                    Err(e) => Ok(Some(format!("Bun execution error: {e}"))),
                }
            }
            ReplCommand::Vella(arg) => {
                let parts: Vec<&str> = arg.split_whitespace().collect();
                let subcmd = parts.first().copied().unwrap_or("status");
                let mgr = crate::vella::VellaAppManager::with_default_schemas();

                match subcmd {
                    "status" => {
                        let e_stop = mgr.governor.is_e_stop_active().await;
                        Ok(Some(format!(
                            "{}\n  - E-Stop Active: {}\n  - Max Order Value: ${:.2}\n  - Max Robot Velocity: {:.1} m/s\n  - Allowed SCADA Coils: [{}, {}]",
                            "⚡ Vella Sovereign Control Plane:".bold().cyan(),
                            if e_stop { "YES (TRIPPED)".bold().red() } else { "NO (ARMED)".bold().green() },
                            mgr.governor.max_order_value_usd,
                            mgr.governor.max_robot_velocity_ms,
                            mgr.governor.allowed_scada_coils.0,
                            mgr.governor.allowed_scada_coils.1
                        )))
                    }
                    "estop" => {
                        mgr.governor.set_e_stop(true, "Triggered via REPL /vella estop").await;
                        Ok(Some("🚨 EMERGENCY STOP LATCHED across all physical actuators and drone kinematic buses.".bold().red().to_string()))
                    }
                    "clear_estop" => {
                        mgr.governor.set_e_stop(false, "Cleared via REPL /vella clear_estop").await;
                        Ok(Some("✅ EMERGENCY STOP CLEARED. Actuation pathways re-armed.".bold().green().to_string()))
                    }
                    "schemas" => {
                        #[cfg(feature = "vella")]
                        {
                            let reg = mgr.schema_registry.read().await;
                            let schemas = reg.all();
                            let mut out = format!("Vella Active Schemas ({}):\n", schemas.len());
                            for s in schemas {
                                out.push_str(&format!("  - {} (table: {}, category: {}, fields: {})\n",
                                    s.name.bold().green(), s.table_name, s.category, s.fields.len()
                                ));
                            }
                            Ok(Some(out))
                        }
                        #[cfg(not(feature = "vella"))]
                        {
                            Ok(Some("Vella schemas disabled without feature.".to_string()))
                        }
                    }
                    "audit" => {
                        let log = mgr.governor.audit_log.read().await;
                        let mut out = format!("Vella Policy Audit Entries ({}):\n", log.len());
                        for entry in log.iter() {
                            out.push_str(&format!("  {}\n", entry));
                        }
                        Ok(Some(out))
                    }
                    _ => Ok(Some(format!(
                        "Usage: /vella <status | estop | clear_estop | schemas | audit>"
                    ))),
                }
            }
            ReplCommand::Save(opt_name) => {
                if let Some(name) = opt_name {
                    self.session_record.id = name;
                }
                self.session_store.save(&self.session_record)?;
                Ok(Some(format!(
                    "Session checkpoint saved successfully as '{}'.",
                    self.session_record.id.bold().green()
                )))
            }
            ReplCommand::Load(id) => {
                if id.is_empty() {
                    return Ok(Some("Usage: /load <session_id>".to_string()));
                }
                let loaded = self.session_store.load(&id)?;
                self.session_record = loaded;
                self.agent.model = self.session_record.model.clone();
                Ok(Some(format!(
                    "Loaded session '{}' with {} messages. Estimated cost: ${:.4}",
                    self.session_record.id.bold().green(),
                    self.session_record.messages.len(),
                    self.session_record.total_cost_usd
                )))
            }
            ReplCommand::Clear => {
                self.session_record.messages.clear();
                Ok(Some("Cleared conversation history for this session.".to_string()))
            }
            ReplCommand::Budget => {
                let spent = self.context.budget_tracker.current_spent_usd();
                let limit = self.context.budget_tracker.max_budget_usd();
                let tokens = &self.session_record.total_usage;
                Ok(Some(format!(
                    "Session Budget Status:\n  - Cost: ${:.4} / ${:.2} USD\n  - Prompt Tokens: {}\n  - Completion Tokens: {}\n  - Total Tokens: {}",
                    spent,
                    limit,
                    tokens.prompt_tokens,
                    tokens.completion_tokens,
                    tokens.prompt_tokens + tokens.completion_tokens
                )))
            }
            ReplCommand::History => {
                let mut out = format!("History Overview ({} messages):\n", self.session_record.messages.len());
                for (i, m) in self.session_record.messages.iter().enumerate() {
                    let role_str = match m.role {
                        Role::User => "User".cyan(),
                        Role::Assistant => "Assistant".green(),
                        Role::System => "System".yellow(),
                        Role::Tool => "Tool".magenta(),
                        Role::Reasoning => "Reasoning".blue(),
                    };
                    let snippet: String = m.extract_text().chars().take(60).collect();
                    out.push_str(&format!("  #{}: [{}] {}\n", i + 1, role_str, snippet.trim()));
                }
                Ok(Some(out))
            }
            ReplCommand::Plan(goal) => {
                let prompt = if goal.is_empty() {
                    "Please inspect current workspace and propose a rigorous step-by-step implementation plan with milestones and verification tests.".to_string()
                } else {
                    format!("Create a comprehensive, step-by-step implementation plan for the following objective:\n{}\nDetail architectural design, files to modify, edge cases, and test strategy.", goal)
                };
                let response = self.run_turn(&prompt).await?;
                Ok(Some(response))
            }
            ReplCommand::Goal(objective) => {
                if objective.is_empty() {
                    return Ok(Some("💡 Usage: /goal <objective> (e.g. /goal implement auth middleware)".yellow().to_string()));
                }
                let prompt = format!("Autonomously execute and verify the following goal step-by-step until completely satisfied:\n{}", objective);
                let response = self.run_turn(&prompt).await?;
                Ok(Some(response))
            }
            ReplCommand::Exit => {
                self.running = false;
                Ok(Some("Exiting interactive session. Goodbye!".to_string()))
            }
            ReplCommand::UserPrompt(prompt) => {
                let response = self.run_turn(&prompt).await?;
                Ok(Some(response))
            }
        }
    }

    /// Launch the live interactive terminal loop
    pub async fn start(&mut self) -> Result<()> {
        self.running = true;
        let cwd_display = std::env::current_dir()
            .unwrap_or_default()
            .display()
            .to_string();
        let short_cwd = if cwd_display.len() > 50 {
            format!("...{}", &cwd_display[cwd_display.len() - 47..])
        } else {
            cwd_display
        };

        println!("{}", "╭──────────────────────────────────────────────────────────────────────────╮".cyan().bold());
        println!("{}", "│  ▲  TAGISAN INTERACTIVE CLI — Antigravity (AGY) Dual-Core UX            │".yellow().bold());
        println!("{}", "├──────────────────────────────────────────────────────────────────────────┤".cyan());
        println!(
            "│  🤖 Model:     {:<54} │",
            self.agent.model.green().bold()
        );
        println!(
            "│  ⚡ Session:   {:<54} │",
            self.session_record.id.cyan().bold()
        );
        println!(
            "│  📁 Workspace: {:<54} │",
            short_cwd.dimmed()
        );
        println!(
            "│  💡 Shortcuts: {} help  •  {} plan  •  {} goal  •  {} exit │",
            "/help".magenta().bold(),
            "/plan".cyan().bold(),
            "/goal".yellow().bold(),
            "/exit".dimmed()
        );
        println!("{}", "╰──────────────────────────────────────────────────────────────────────────╯".cyan().bold());

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        while self.running {
            print!("\n{} {} ", "▲".bold().magenta(), "tgs ❯".bold().cyan());
            let _ = stdout.flush();

            let mut line = String::new();
            if stdin.read_line(&mut line).is_err() {
                break;
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            let cmd = Self::parse_command(input);
            match self.execute_command(cmd).await {
                Ok(Some(output)) => {
                    println!("{output}");
                }
                Ok(None) => {}
                Err(err) => {
                    eprintln!("\n{} {err}", "❌ Error:".bold().red());
                }
            }
        }

        // Clean up sandbox if active
        if let Some(mut sb) = self.sandbox.take() {
            let _ = sb.cleanup();
        }

        println!("\n{}", "👋 Session concluded. Thank you for building with Tagisan!".green().bold());
        Ok(())
    }
}

/// Format the agent's turn result with appealing borders, emojis, and vibrant syntax cues
pub fn format_appealing_repl_response(
    content: &str,
    model: &str,
    elapsed_secs: f64,
    prompt_tokens: u32,
    completion_tokens: u32,
    cost_usd: f64,
    session_id: &str,
) -> String {
    let mut out = String::new();

    // Appealing Header with AGY Insignia
    let model_tag = format!("[{model}]").bold().yellow();
    out.push_str(&format!(
        "\n{}\n",
        format!("╭── ▲ ✦ Tagisan AI  {} ──────────────────────────────────────────", model_tag).bold().cyan()
    ));

    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Code block entry / exit
        if trimmed.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                let lang = trimmed.trim_start_matches("```").trim();
                let lang_display = if lang.is_empty() { "code" } else { lang };
                out.push_str(&format!(
                    "│  {}\n",
                    format!("📦 [{}] ──────────────────────────────────", lang_display).bold().yellow()
                ));
            } else {
                in_code_block = false;
                out.push_str(&format!(
                    "│  {}\n",
                    "────────────────────────────────────────────────".dimmed()
                ));
            }
            continue;
        }

        if in_code_block {
            // Code block content: indented with soft cyan
            out.push_str(&format!("│    {}\n", line.cyan()));
            continue;
        }

        // Markdown Headers with distinct vibrant emojis
        if let Some(h1) = trimmed.strip_prefix("# ") {
            out.push_str(&format!("│\n│  {} {}\n│\n", "📌".bold(), h1.bold().bright_yellow()));
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            out.push_str(&format!("│\n│  {} {}\n│\n", "⚡".bold(), h2.bold().bright_cyan()));
            continue;
        }
        if let Some(h3) = trimmed.strip_prefix("### ") {
            out.push_str(&format!("│  {} {}\n", "✨".bold(), h3.bold().bright_magenta()));
            continue;
        }

        // Bullet lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let bullet_text = &trimmed[2..];
            let colored_bullet = format_inline_markdown(bullet_text);
            out.push_str(&format!("│    {} {}\n", "▸".bold().bright_green(), colored_bullet));
            continue;
        }

        // Numbered lists
        if let Some(dot_idx) = trimmed.find(". ") {
            let prefix = &trimmed[..dot_idx];
            if prefix.chars().all(|c| c.is_ascii_digit()) && !prefix.is_empty() {
                let rest = &trimmed[dot_idx + 2..];
                let colored_rest = format_inline_markdown(rest);
                out.push_str(&format!("│    {} {}\n", format!("{}.", prefix).bold().bright_cyan(), colored_rest));
                continue;
            }
        }

        // Blank lines
        if trimmed.is_empty() {
            out.push_str("│\n");
            continue;
        }

        // Standard prose with inline formatting & emoji callouts
        let colored_line = format_inline_markdown(line);
        out.push_str(&format!("│  {}\n", colored_line));
    }

    // Appealing Footer with real-time stats, emojis, and cost
    let elapsed_str = if elapsed_secs >= 60.0 {
        format!("{:.1}m", elapsed_secs / 60.0)
    } else {
        format!("{:.2}s", elapsed_secs)
    };
    let total_tokens = prompt_tokens + completion_tokens;
    let footer_stats = format!(
        "⏱️ {} │ 🪙 {} tokens (${:.4}) │ 💬 {}",
        elapsed_str.bold().bright_white(),
        total_tokens.to_string().bold().bright_green(),
        cost_usd,
        session_id.dimmed()
    );

    out.push_str(&format!(
        "│\n{}\n",
        format!("╰── ▲ {} ─────────────────────────", footer_stats).bold().cyan()
    ));

    out
}

/// Helper to colorize inline markdown elements and add emoji callouts
fn format_inline_markdown(text: &str) -> String {
    let mut res = text.to_string();

    // Callout emoji enhancers
    if res.starts_with("Note:") || res.starts_with("NOTE:") {
        res = res.replacen("Note:", &format!("{} {}", "📝", "Note:".bold().bright_blue()), 1);
    } else if res.starts_with("Tip:") || res.starts_with("TIP:") {
        res = res.replacen("Tip:", &format!("{} {}", "💡", "Tip:".bold().bright_green()), 1);
    } else if res.starts_with("Important:") || res.starts_with("IMPORTANT:") {
        res = res.replacen("Important:", &format!("{} {}", "🔥", "Important:".bold().bright_red()), 1);
    } else if res.starts_with("Warning:") || res.starts_with("WARNING:") {
        res = res.replacen("Warning:", &format!("{} {}", "⚠️ ", "Warning:".bold().bright_yellow()), 1);
    } else if res.starts_with("Result:") || res.starts_with("RESULT:") {
        res = res.replacen("Result:", &format!("{} {}", "🎯", "Result:".bold().bright_cyan()), 1);
    } else if res.starts_with("Summary:") || res.starts_with("SUMMARY:") {
        res = res.replacen("Summary:", &format!("{} {}", "📊", "Summary:".bold().bright_cyan()), 1);
    } else if res.starts_with("Solution:") || res.starts_with("SOLUTIONS:") {
        res = res.replacen("Solution:", &format!("{} {}", "🛠️ ", "Solution:".bold().bright_green()), 1);
    }

    // Bold formatting: **text** -> text.bold().bright_white()
    while let Some(start) = res.find("**") {
        if let Some(end) = res[start + 2..].find("**") {
            let actual_end = start + 2 + end;
            let inner = &res[start + 2..actual_end];
            let replacement = format!("{}", inner.bold().bright_white());
            res.replace_range(start..actual_end + 2, &replacement);
        } else {
            break;
        }
    }

    // Inline code: `code` -> code.bold().yellow()
    while let Some(start) = res.find('`') {
        if let Some(end) = res[start + 1..].find('`') {
            let actual_end = start + 1 + end;
            let inner = &res[start + 1..actual_end];
            let replacement = format!("{}", inner.bold().yellow());
            res.replace_range(start..actual_end + 1, &replacement);
        } else {
            break;
        }
    }

    res
}
