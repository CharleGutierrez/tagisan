use crate::agent::{AutonomousAgent, WorktreeSandbox};
use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use crate::swarm::session::{SessionRecord, SessionStore};
use crate::tui::Spinner;
use crate::types::{Message, Role};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
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

        // Execute agent turn with Ctrl+C interrupt handler
        let exec_result = tokio::select! {
            res = self.agent.execute_session(&mut chat_session, &self.context) => res,
            _ = tokio::signal::ctrl_c() => {
                spinner.failure("Aborted by user (Ctrl+C)");
                return Ok(format!("\n{}", "⚠️  Agent execution cancelled by user (Ctrl+C). Ready for next instruction.".yellow().bold()));
            }
        };

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
                    {}       Propose rigorous step-by-step implementation plan\n\
                    {}       Autonomously execute toward objective\n\
                    {}       Exit interactive session\n\n\
                    {}\n\
                    {}          Recall previous / next commands from history\n\
                    {}         Reverse incremental history search\n\
                    {}         Clear terminal screen & reprint AGY banner\n\
                    {}           Context-aware auto-complete (commands, personas, skills)\n\
                    {}   Move cursor backward / forward by word\n\
                    {}   Jump to line start / line end\n\
                    {}   Cut text to line start / line end into kill ring\n\
                    {}   Cut previous word into kill ring / Yank (paste)\n\
                    {}         Cancel current prompt or abort thinking agent\n\
                    {}         Exit session on empty line (or delete char under cursor)\n\
                    {}     Multi-line prompt continuation (creates  │  block)\n",
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
                    "/plan [goal]".bold().green(),
                    "/goal <goal>".bold().green(),
                    "/exit".bold().green(),
                    "Google Antigravity (AGY) Keyboard Controls:".bold().yellow(),
                    "↑ / ↓".bold().bright_cyan(),
                    "Ctrl+R".bold().bright_cyan(),
                    "Ctrl+L".bold().bright_cyan(),
                    "Tab".bold().bright_cyan(),
                    "Ctrl/Alt + ← / →".bold().bright_cyan(),
                    "Home / End (Ctrl+A / Ctrl+E)".bold().bright_cyan(),
                    "Ctrl+U / Ctrl+K".bold().bright_cyan(),
                    "Ctrl+W / Ctrl+Y".bold().bright_cyan(),
                    "Ctrl+C".bold().bright_cyan(),
                    "Ctrl+D".bold().bright_cyan(),
                    "\\ + Enter (Alt+Enter)".bold().bright_cyan(),
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

    /// Print AGY Dual-Core UX banner for current session
    pub fn print_banner(&self) {
        Self::print_banner_static(&self.agent.model, &self.session_record.id);
    }

    /// Static banner renderer supporting raw-mode screen repaints
    pub fn print_banner_static(model: &str, session_id: &str) {
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
            model.green().bold()
        );
        println!(
            "│  ⚡ Session:   {:<54} │",
            session_id.cyan().bold()
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
        println!(
            "│  ⌨️  Keys:      {} hist • {} search • {} clear • {} multi-line │",
            "↑/↓".bright_white().bold(),
            "Ctrl+R".bright_cyan().bold(),
            "Ctrl+L".bright_yellow().bold(),
            "\\+Enter".bright_green().bold()
        );
        println!("{}", "╰──────────────────────────────────────────────────────────────────────────╯".cyan().bold());
    }

    /// Launch the live interactive terminal loop with AGY keyboard controls & readline engine
    pub async fn start(&mut self) -> Result<()> {
        self.running = true;
        self.print_banner();

        let mut editor = ReplEditor::new();

        while self.running {
            let prompt = format!("{} {} ", "▲".bold().magenta(), "tgs ❯".bold().cyan());
            let current_model = self.agent.model.clone();
            let current_session_id = self.session_record.id.clone();

            let repaint_banner = move || {
                Self::print_banner_static(&current_model, &current_session_id);
            };

            let line_result = editor.read_line(&prompt, &repaint_banner)?;

            let input = match line_result {
                ReadlineResult::Submit(s) => s,
                ReadlineResult::Eof => {
                    self.running = false;
                    break;
                }
                ReadlineResult::Interrupted => {
                    continue;
                }
            };

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            let cmd = Self::parse_command(trimmed);
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

/// Result of an interactive line edit
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadlineResult {
    Submit(String),
    Eof,
    Interrupted,
}

/// RAII Guard for terminal raw mode ensuring raw mode is disabled on drop
pub struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self { active: true })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
            self.active = false;
        }
    }
}

/// Interactive readline and AGY keyboard navigation editor
pub struct ReplEditor {
    pub history: Vec<String>,
    pub history_index: usize,
    pub draft: Vec<char>,
    pub kill_ring: String,
    pub history_file: Option<PathBuf>,
}

impl Default for ReplEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplEditor {
    pub fn new() -> Self {
        let history_file = Self::resolve_history_path();
        let history = if let Some(ref path) = history_file {
            Self::load_history_from_file(path)
        } else {
            Vec::new()
        };
        let history_index = history.len();
        Self {
            history,
            history_index,
            draft: Vec::new(),
            kill_ring: String::new(),
            history_file,
        }
    }

    pub fn with_history(history: Vec<String>) -> Self {
        let history_index = history.len();
        Self {
            history,
            history_index,
            draft: Vec::new(),
            kill_ring: String::new(),
            history_file: None,
        }
    }

    fn resolve_history_path() -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(home).join(".tagisan");
            let _ = std::fs::create_dir_all(&dir);
            Some(dir.join("repl_history.txt"))
        } else {
            let dir = PathBuf::from(".tagisan");
            let _ = std::fs::create_dir_all(&dir);
            Some(dir.join("repl_history.txt"))
        }
    }

    fn load_history_from_file(path: &PathBuf) -> Vec<String> {
        if !path.exists() {
            return Vec::new();
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = content
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .collect();
        if lines.len() > 5000 {
            lines = lines.split_off(lines.len() - 5000);
        }
        lines
    }

    pub fn add_history(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.history.last().map(|s| s.as_str()) == Some(trimmed) {
            self.history_index = self.history.len();
            return;
        }
        self.history.push(trimmed.to_string());
        self.history_index = self.history.len();

        if let Some(ref path) = self.history_file {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{}", trimmed);
            }
        }
    }

    pub fn find_word_backward(buffer: &[char], cursor: usize) -> usize {
        if cursor == 0 {
            return 0;
        }
        let mut i = cursor;
        while i > 0 && buffer[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !buffer[i - 1].is_whitespace() {
            i -= 1;
        }
        i
    }

    pub fn find_word_forward(buffer: &[char], cursor: usize) -> usize {
        let len = buffer.len();
        if cursor >= len {
            return len;
        }
        let mut i = cursor;
        while i < len && !buffer[i].is_whitespace() {
            i += 1;
        }
        while i < len && buffer[i].is_whitespace() {
            i += 1;
        }
        i
    }

    pub fn longest_common_prefix(strings: &[String]) -> String {
        if strings.is_empty() {
            return String::new();
        }
        let first = &strings[0];
        let mut len = 0;
        for (i, c) in first.chars().enumerate() {
            if strings.iter().all(|s| s.chars().nth(i) == Some(c)) {
                len += c.len_utf8();
            } else {
                break;
            }
        }
        first[..len].to_string()
    }

    pub fn get_completions(prefix: &str) -> Vec<(String, usize)> {
        let mut results = Vec::new();
        let trimmed_prefix = prefix.trim_start();

        // 1. Slash commands: if typing `/...` and no space yet
        if trimmed_prefix.starts_with('/') && !trimmed_prefix.contains(' ') {
            let slash_cmds = [
                ("/help", "Display help manual"),
                ("/agent", "Switch agent persona"),
                ("/skill", "Attach ECC skill"),
                ("/forge", "Forge skill from binary/code/mcp"),
                ("/model", "Switch active model"),
                ("/tools", "List registered tools"),
                ("/memory", "Display memory stats"),
                ("/sandbox", "Inspect git worktree sandbox"),
                ("/bun", "Evaluate TS/JS via Bun"),
                ("/vella", "Vella Sovereign OS & E-Stop"),
                ("/save", "Save session checkpoint"),
                ("/load", "Load saved session"),
                ("/clear", "Clear conversational history"),
                ("/budget", "Display token usage and cost"),
                ("/history", "Show conversational history"),
                ("/plan", "Propose implementation plan"),
                ("/goal", "Autonomous execution toward goal"),
                ("/exit", "Exit interactive session"),
            ];

            for (cmd, _) in slash_cmds {
                if cmd.starts_with(trimmed_prefix) {
                    let completed = format!("{cmd} ");
                    let len = completed.chars().count();
                    results.push((completed, len));
                }
            }
            return results;
        }

        // 2. /agent <persona>
        if let Some(rest) = trimmed_prefix.strip_prefix("/agent ") {
            let arg = rest.trim_start();
            for preset in crate::ecc::all_presets() {
                if preset.name.starts_with(arg) {
                    let completed = format!("/agent {} ", preset.name);
                    let len = completed.chars().count();
                    results.push((completed, len));
                }
            }
            return results;
        }

        // 3. /skill <name>
        if let Some(rest) = trimmed_prefix.strip_prefix("/skill ") {
            let arg = rest.trim_start();
            for skill in crate::ecc::all_built_in_skills() {
                if skill.name.starts_with(arg) {
                    let completed = format!("/skill {} ", skill.name);
                    let len = completed.chars().count();
                    results.push((completed, len));
                }
            }
            return results;
        }

        // 4. /vella <subcmd>
        if let Some(rest) = trimmed_prefix.strip_prefix("/vella ") {
            let arg = rest.trim_start();
            let subcmds = ["status", "estop", "clear_estop", "schemas", "audit"];
            for sc in subcmds {
                if sc.starts_with(arg) {
                    let completed = format!("/vella {} ", sc);
                    let len = completed.chars().count();
                    results.push((completed, len));
                }
            }
            return results;
        }

        results
    }

    fn redraw_line(prompt: &str, buffer: &[char], cursor: usize) -> io::Result<()> {
        let mut stdout = io::stdout();
        let text: String = buffer.iter().collect();
        write!(stdout, "\r\x1b[2K{}{}", prompt, text)?;

        let backtracks = buffer.len().saturating_sub(cursor);
        if backtracks > 0 {
            write!(stdout, "\x1b[{}D", backtracks)?;
        }
        stdout.flush()?;
        Ok(())
    }

    fn run_reverse_search(
        history: &[String],
        buffer: &mut Vec<char>,
        cursor: &mut usize,
        active_prompt: &str,
    ) -> io::Result<()> {
        let original_buf = buffer.clone();
        let original_cursor = *cursor;

        let mut query = String::new();
        let mut search_idx: Option<usize> = None;

        let find_match = |q: &str, start_from: Option<usize>| -> Option<usize> {
            if q.is_empty() || history.is_empty() {
                return None;
            }
            let end = start_from.unwrap_or(history.len().saturating_sub(1));
            for i in (0..=end).rev() {
                if history[i].contains(q) {
                    return Some(i);
                }
            }
            None
        };

        loop {
            let mut stdout = io::stdout();
            let match_display = if let Some(idx) = search_idx {
                &history[idx]
            } else {
                ""
            };
            let prompt_str = if search_idx.is_some() || query.is_empty() {
                format!("(reverse-i-search)`{}': {}", query.bold().green(), match_display)
            } else {
                format!("(failed reverse-i-search)`{}': {}", query.bold().red(), match_display)
            };
            write!(stdout, "\r\x1b[2K{}", prompt_str)?;
            stdout.flush()?;

            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }
                match key_event.code {
                    KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(idx) = search_idx {
                            if idx > 0 {
                                search_idx = find_match(&query, Some(idx - 1));
                            }
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('g') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        *buffer = original_buf;
                        *cursor = original_cursor;
                        Self::redraw_line(active_prompt, buffer, *cursor)?;
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        *buffer = original_buf;
                        *cursor = original_cursor;
                        Self::redraw_line(active_prompt, buffer, *cursor)?;
                        return Ok(());
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = search_idx {
                            *buffer = history[idx].chars().collect();
                            *cursor = buffer.len();
                        }
                        return Ok(());
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        search_idx = find_match(&query, None);
                    }
                    KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) && !key_event.modifiers.contains(KeyModifiers::ALT) => {
                        query.push(c);
                        search_idx = find_match(&query, None);
                    }
                    _ => {
                        if let Some(idx) = search_idx {
                            *buffer = history[idx].chars().collect();
                            *cursor = buffer.len();
                        }
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Read an interactive line with AGY keyboard functionalities, history, and completions
    pub fn read_line(
        &mut self,
        prompt: &str,
        on_repaint: &dyn Fn(),
    ) -> Result<ReadlineResult> {
        if !io::stdin().is_terminal() {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => return Ok(ReadlineResult::Eof),
                Ok(_) => {
                    let trimmed = line.trim_end_matches(&['\r', '\n'][..]).to_string();
                    self.add_history(&trimmed);
                    return Ok(ReadlineResult::Submit(trimmed));
                }
                Err(e) => return Err(TagisanError::Execution(format!("Stdin read error: {e}"))),
            }
        }

        let mut buffer: Vec<char> = Vec::new();
        let mut cursor: usize = 0;
        let mut continuation_lines: Vec<String> = Vec::new();
        self.history_index = self.history.len();
        self.draft.clear();

        let _raw_guard = match RawModeGuard::enter() {
            Ok(guard) => Some(guard),
            Err(_) => None,
        };

        if _raw_guard.is_none() {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => return Ok(ReadlineResult::Eof),
                Ok(_) => {
                    let trimmed = line.trim_end_matches(&['\r', '\n'][..]).to_string();
                    self.add_history(&trimmed);
                    return Ok(ReadlineResult::Submit(trimmed));
                }
                Err(e) => return Err(TagisanError::Execution(format!("Stdin read error: {e}"))),
            }
        }

        let continuation_prompt = "  │ ";
        let active_prompt = |is_cont: bool| -> &str {
            if is_cont {
                continuation_prompt
            } else {
                prompt
            }
        };

        Self::redraw_line(prompt, &buffer, cursor)
            .map_err(|e| TagisanError::Execution(format!("Terminal draw error: {e}")))?;

        loop {
            let is_continuation = !continuation_lines.is_empty();
            let cur_prompt = active_prompt(is_continuation);

            let event = match event::read() {
                Ok(ev) => ev,
                Err(e) => return Err(TagisanError::Execution(format!("Failed to read terminal event: {e}"))),
            };

            match event {
                Event::Paste(pasted_text) => {
                    for ch in pasted_text.chars() {
                        if ch == '\r' || ch == '\n' {
                            let cur_line: String = buffer.iter().collect();
                            continuation_lines.push(cur_line);
                            buffer.clear();
                            cursor = 0;
                            let _ = write!(io::stdout(), "\r\n");
                        } else {
                            buffer.insert(cursor, ch);
                            cursor += 1;
                        }
                    }
                    let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                }
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }

                    match key_event.code {
                        // ── Screen & Interrupt Handling ──────────────────
                        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !buffer.is_empty() || !continuation_lines.is_empty() {
                                buffer.clear();
                                cursor = 0;
                                continuation_lines.clear();
                                let _ = write!(io::stdout(), "^C\r\n");
                                let _ = Self::redraw_line(prompt, &buffer, cursor);
                            } else {
                                let _ = write!(io::stdout(), "^C\r\n  💡 Press Ctrl+D or type /exit to quit\r\n");
                                let _ = Self::redraw_line(prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if buffer.is_empty() && continuation_lines.is_empty() {
                                let _ = write!(io::stdout(), "\r\n");
                                return Ok(ReadlineResult::Eof);
                            } else if cursor < buffer.len() {
                                buffer.remove(cursor);
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('l') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            let _ = write!(io::stdout(), "\x1b[2J\x1b[1;1H");
                            on_repaint();
                            for prev in &continuation_lines {
                                let _ = writeln!(io::stdout(), "  │ {}\r", prev);
                            }
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }

                        // ── Inline Editing & Navigation ──────────────────
                        KeyCode::Char('a') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            cursor = 0;
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Char('e') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            cursor = buffer.len();
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Home => {
                            cursor = 0;
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::End => {
                            cursor = buffer.len();
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Left => {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) || key_event.modifiers.contains(KeyModifiers::ALT) {
                                cursor = Self::find_word_backward(&buffer, cursor);
                            } else if cursor > 0 {
                                cursor -= 1;
                            }
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Right => {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) || key_event.modifiers.contains(KeyModifiers::ALT) {
                                cursor = Self::find_word_forward(&buffer, cursor);
                            } else if cursor < buffer.len() {
                                cursor += 1;
                            }
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Char('b') if key_event.modifiers.contains(KeyModifiers::ALT) => {
                            cursor = Self::find_word_backward(&buffer, cursor);
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }
                        KeyCode::Char('f') if key_event.modifiers.contains(KeyModifiers::ALT) => {
                            cursor = Self::find_word_forward(&buffer, cursor);
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }

                        // ── Kill Ring & Deletion ────────────────────────
                        KeyCode::Char('k') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if cursor < buffer.len() {
                                self.kill_ring = buffer[cursor..].iter().collect();
                                buffer.truncate(cursor);
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('u') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if cursor > 0 {
                                self.kill_ring = buffer[..cursor].iter().collect();
                                buffer.drain(..cursor);
                                cursor = 0;
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('w') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if cursor > 0 {
                                let kill_start = Self::find_word_backward(&buffer, cursor);
                                self.kill_ring = buffer[kill_start..cursor].iter().collect();
                                buffer.drain(kill_start..cursor);
                                cursor = kill_start;
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::ALT) => {
                            let kill_end = Self::find_word_forward(&buffer, cursor);
                            if kill_end > cursor {
                                self.kill_ring = buffer[cursor..kill_end].iter().collect();
                                buffer.drain(cursor..kill_end);
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('y') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if !self.kill_ring.is_empty() {
                                for ch in self.kill_ring.chars() {
                                    buffer.insert(cursor, ch);
                                    cursor += 1;
                                }
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Backspace | KeyCode::Char('h') if key_event.code == KeyCode::Backspace || key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            if cursor > 0 {
                                buffer.remove(cursor - 1);
                                cursor -= 1;
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Delete => {
                            if cursor < buffer.len() {
                                buffer.remove(cursor);
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }

                        // ── History Navigation ──────────────────────────
                        KeyCode::Up => {
                            if self.history_index == self.history.len() {
                                self.draft = buffer.clone();
                            }
                            if self.history_index > 0 {
                                self.history_index -= 1;
                                buffer = self.history[self.history_index].chars().collect();
                                cursor = buffer.len();
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Down => {
                            if self.history_index + 1 < self.history.len() {
                                self.history_index += 1;
                                buffer = self.history[self.history_index].chars().collect();
                                cursor = buffer.len();
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            } else if self.history_index + 1 == self.history.len() {
                                self.history_index = self.history.len();
                                buffer = self.draft.clone();
                                cursor = buffer.len();
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }
                        KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                            let _ = Self::run_reverse_search(&self.history, &mut buffer, &mut cursor, cur_prompt);
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }

                        // ── Tab Autocompletion ──────────────────────────
                        KeyCode::Tab => {
                            let current_text: String = buffer.iter().collect();
                            let prefix = &current_text[..cursor];

                            let completions = Self::get_completions(prefix);
                            if completions.is_empty() {
                                // No match
                            } else if completions.len() == 1 {
                                let (replacement, new_cursor) = completions[0].clone();
                                buffer = replacement.chars().collect();
                                cursor = new_cursor;
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            } else {
                                let replacement_strings: Vec<String> = completions.iter().map(|(r, _)| r.clone()).collect();
                                let common = Self::longest_common_prefix(&replacement_strings);
                                if common.len() > prefix.len() {
                                    buffer = common.chars().collect();
                                    cursor = buffer.len();
                                }

                                let mut stdout = io::stdout();
                                let _ = write!(stdout, "\r\n");
                                let pills: Vec<String> = completions
                                    .iter()
                                    .map(|(r, _)| {
                                        let label = r.split_whitespace().last().unwrap_or(r.as_str());
                                        format!("  {} {}", "▸".cyan(), label.bold().white())
                                    })
                                    .collect();

                                for chunk in pills.chunks(3) {
                                    let _ = writeln!(stdout, "{}\r", chunk.join("    "));
                                }
                                let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                            }
                        }

                        // ── Submission & Multi-Line Continuation ────────
                        KeyCode::Enter => {
                            let is_continuation_trigger = buffer.ends_with(&['\\'])
                                || key_event.modifiers.contains(KeyModifiers::ALT)
                                || key_event.modifiers.contains(KeyModifiers::SHIFT);

                            if is_continuation_trigger {
                                if buffer.ends_with(&['\\']) {
                                    buffer.pop();
                                }
                                let cur_line: String = buffer.iter().collect();
                                continuation_lines.push(cur_line);
                                buffer.clear();
                                cursor = 0;
                                let _ = write!(io::stdout(), "\r\n");
                                let _ = Self::redraw_line(continuation_prompt, &buffer, cursor);
                            } else {
                                let _ = write!(io::stdout(), "\r\n");
                                let final_line = if continuation_lines.is_empty() {
                                    buffer.iter().collect()
                                } else {
                                    let mut full = continuation_lines.join("\n");
                                    full.push('\n');
                                    full.push_str(&buffer.iter().collect::<String>());
                                    full
                                };
                                self.add_history(&final_line);
                                return Ok(ReadlineResult::Submit(final_line));
                            }
                        }

                        // ── Regular Characters ──────────────────────────
                        KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) && !key_event.modifiers.contains(KeyModifiers::ALT) => {
                            buffer.insert(cursor, c);
                            cursor += 1;
                            let _ = Self::redraw_line(cur_prompt, &buffer, cursor);
                        }

                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_word_backward() {
        let text: Vec<char> = "hello beautiful world".chars().collect();
        // At end: cursor = 21 ('d')
        let pos1 = ReplEditor::find_word_backward(&text, 21);
        assert_eq!(pos1, 16); // start of "world"

        // Cursor at 16 ('w')
        let pos2 = ReplEditor::find_word_backward(&text, 16);
        assert_eq!(pos2, 6); // start of "beautiful"

        // Cursor at 6 ('b')
        let pos3 = ReplEditor::find_word_backward(&text, 6);
        assert_eq!(pos3, 0); // start of "hello"

        // Cursor at 0
        let pos4 = ReplEditor::find_word_backward(&text, 0);
        assert_eq!(pos4, 0);
    }

    #[test]
    fn test_find_word_forward() {
        let text: Vec<char> = "hello beautiful world".chars().collect();
        // At start: cursor = 0 ('h')
        let pos1 = ReplEditor::find_word_forward(&text, 0);
        assert_eq!(pos1, 6); // start of "beautiful"

        // Cursor at 6 ('b')
        let pos2 = ReplEditor::find_word_forward(&text, 6);
        assert_eq!(pos2, 16); // start of "world"

        // Cursor at 16 ('w')
        let pos3 = ReplEditor::find_word_forward(&text, 16);
        assert_eq!(pos3, 21); // end of "world"

        // Cursor at end
        let pos4 = ReplEditor::find_word_forward(&text, 21);
        assert_eq!(pos4, 21);
    }

    #[test]
    fn test_longest_common_prefix() {
        let list1 = vec!["/help".to_string(), "/history".to_string()];
        assert_eq!(ReplEditor::longest_common_prefix(&list1), "/h");

        let list2 = vec!["/plan".to_string(), "/prompt".to_string()];
        assert_eq!(ReplEditor::longest_common_prefix(&list2), "/p");

        let list3 = vec!["single".to_string()];
        assert_eq!(ReplEditor::longest_common_prefix(&list3), "single");

        let list4: Vec<String> = vec![];
        assert_eq!(ReplEditor::longest_common_prefix(&list4), "");
    }

    #[test]
    fn test_get_completions_slash_commands() {
        let comp_h = ReplEditor::get_completions("/h");
        let names: Vec<String> = comp_h.into_iter().map(|(s, _)| s).collect();
        assert!(names.contains(&"/help ".to_string()));
        assert!(names.contains(&"/history ".to_string()));

        let comp_p = ReplEditor::get_completions("/p");
        let names: Vec<String> = comp_p.into_iter().map(|(s, _)| s).collect();
        assert!(names.contains(&"/plan ".to_string()));

        let comp_g = ReplEditor::get_completions("/g");
        let names: Vec<String> = comp_g.into_iter().map(|(s, _)| s).collect();
        assert!(names.contains(&"/goal ".to_string()));
    }

    #[test]
    fn test_get_completions_agent_presets() {
        let comp_arch = ReplEditor::get_completions("/agent arch");
        assert!(!comp_arch.is_empty());
        assert_eq!(comp_arch[0].0, "/agent architect ");
    }

    #[test]
    fn test_get_completions_skills() {
        let comp_sec = ReplEditor::get_completions("/skill security");
        assert!(!comp_sec.is_empty());
        let names: Vec<String> = comp_sec.into_iter().map(|(s, _)| s).collect();
        assert!(names.contains(&"/skill security-review ".to_string()));
    }

    #[test]
    fn test_get_completions_vella() {
        let comp_estop = ReplEditor::get_completions("/vella est");
        assert!(!comp_estop.is_empty());
        assert_eq!(comp_estop[0].0, "/vella estop ");
    }

    #[test]
    fn test_repl_editor_history_and_dedup() {
        let mut editor = ReplEditor::with_history(vec!["first".to_string(), "second".to_string()]);
        assert_eq!(editor.history.len(), 2);
        assert_eq!(editor.history_index, 2);

        // Deduplicate adjacent
        editor.add_history("second");
        assert_eq!(editor.history.len(), 2);

        // Add distinct
        editor.add_history("third");
        assert_eq!(editor.history.len(), 3);
        assert_eq!(editor.history[2], "third");
    }

    #[test]
    fn test_kill_ring_and_line_editing() {
        let mut buffer: Vec<char> = "the quick brown fox".chars().collect();
        let mut kill_ring = String::new();
        let cursor = 10; // at 'b'

        // Kill to end (Ctrl+K)
        kill_ring = buffer[cursor..].iter().collect();
        buffer.truncate(cursor);
        assert_eq!(kill_ring, "brown fox");
        assert_eq!(buffer.iter().collect::<String>(), "the quick ");

        // Yank (Ctrl+Y)
        for ch in kill_ring.chars() {
            buffer.push(ch);
        }
        assert_eq!(buffer.iter().collect::<String>(), "the quick brown fox");
    }
}
