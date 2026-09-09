use crate::agent::{AutonomousAgent, WorktreeSandbox};
use crate::engine::EngineContext;
use crate::error::Result;
use crate::swarm::session::{SessionRecord, SessionStore};
use crate::types::{Message, Role};
use colored::Colorize;
use std::io::{self, Write};

/// Slash commands supported inside the Interactive Agent REPL
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    Help,
    Agent(String),
    Skill(String),
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
            "/model" | "/m" => ReplCommand::Model(arg),
            "/tools" | "/t" => ReplCommand::Tools,
            "/memory" | "/mem" => ReplCommand::Memory,
            "/sandbox" | "/box" => ReplCommand::Sandbox,
            "/bun" => ReplCommand::Bun(arg),
            "/save" | "/s" => {
                let name = if arg.is_empty() { None } else { Some(arg) };
                ReplCommand::Save(name)
            }
            "/load" | "/l" => ReplCommand::Load(arg),
            "/clear" | "/c" => ReplCommand::Clear,
            "/budget" | "/b" => ReplCommand::Budget,
            "/history" | "/hist" => ReplCommand::History,
            "/exit" | "/quit" | "/q" => ReplCommand::Exit,
            _ => ReplCommand::UserPrompt(trimmed.to_string()),
        }
    }

    /// Execute a single turn on the agent session
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

        // Execute agent turn
        let result = self.agent.execute_session(&mut chat_session, &self.context).await?;

        // Update session record with results
        self.session_record.messages = chat_session.history;
        self.session_record.total_usage.prompt_tokens += result.total_usage.prompt_tokens;
        self.session_record.total_usage.completion_tokens += result.total_usage.completion_tokens;
        if let Some(rt) = result.total_usage.reasoning_tokens {
            self.session_record.total_usage.reasoning_tokens =
                Some(self.session_record.total_usage.reasoning_tokens.unwrap_or(0) + rt);
        }
        self.session_record.total_cost_usd = self.context.budget_tracker.current_spent_usd();

        // Auto-save checkpoint
        let _ = self.session_store.save(&self.session_record);

        Ok(result.final_answer)
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
                    {}   Switch model name\n\
                    {}       List registered tools\n\
                    {}      Display memory stats or search memory\n\
                    {}     Inspect git worktree sandbox status & diff\n\
                    {}  Evaluate TypeScript/JavaScript on the fly via Bun\n\
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
                    "/model <name>".bold().green(),
                    "/tools".bold().green(),
                    "/memory".bold().green(),
                    "/sandbox".bold().green(),
                    "/bun <ts_code>".bold().green(),
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
        println!("{}", "═".repeat(70).cyan());
        println!(
            "{}",
            "🇵🇭 Tagisan Interactive Agent REPL — Milestone 8 Swarm & Session Core"
                .bold()
                .yellow()
        );
        println!(
            "Model: {} | Session: {} | Type {} for command manual",
            self.agent.model.bold().green(),
            self.session_record.id.bold().cyan(),
            "/help".bold().magenta()
        );
        println!("{}", "═".repeat(70).cyan());

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        while self.running {
            print!("\n{} ", "tgs>".bold().green());
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
                    println!("\n{output}");
                }
                Ok(None) => {}
                Err(err) => {
                    eprintln!("\n{} {err}", "Error:".bold().red());
                }
            }
        }

        // Clean up sandbox if active
        if let Some(mut sb) = self.sandbox.take() {
            let _ = sb.cleanup();
        }

        Ok(())
    }
}
