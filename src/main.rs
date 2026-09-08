use clap::{Parser, Subcommand};
use colored::Colorize;
use futures::StreamExt;
use std::env;
use std::io::Write;
use std::sync::Arc;
use tagisan::{
    all_ecc_presets, all_ecc_skills, build_ecc_pipeline, load_ecc_agents_from_dir,
    load_ecc_skills_from_dir, resolve_ecc_agent, resolve_ecc_skill, AgentShieldScanner,
    AnthropicProvider, AutonomousAgent, CalculatorTool, ChatSession, CollaborationStrategy,
    CompletionRequest, ContentBlock, DagScheduler, DialecticalDebateStrategy, EccAuditDebate,
    EngineContext, GeminiProvider, LlmProvider, MixtureOfAgentsStrategy, OllamaProvider,
    OpenAiCompatibleProvider, ProviderCapabilities, ReadFileTool, RunCommandTool, StrategyInput,
    StreamChunkDelta, TagisanError, ToolRegistry, WorkflowEvent, WorkflowPlanner, WriteFileTool,
};

#[derive(Parser)]
#[command(name = "tagisan")]
#[command(
    about = "🇵🇭 Tagisan ng Talino: High-Performance Multi-LLM Collaboration Engine in Rust",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Maximum session budget in USD (default: $5.00)
    #[arg(long, default_value = "5.00", global = true)]
    max_budget: f64,
}

#[derive(Subcommand)]
enum Commands {
    /// Stream real-time token output from any specific provider
    Stream {
        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama (default: auto-detected)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name (defaults to best model for selected provider)
        #[arg(short, long)]
        model: Option<String>,

        /// User query or prompt
        prompt: String,
    },
    /// Direct single completion query to any provider
    Ask {
        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama (default: auto-detected)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// User prompt
        prompt: String,
    },
    /// Execute Mixture-of-Agents (Parallel Proposers -> Master Aggregator)
    Moa {
        /// The query or coding task prompt
        prompt: String,
    },
    /// Execute Dialectical Debate (Thesis -> Antithesis -> Lakandiwa Synthesis)
    Debate {
        /// Launch interactive multi-pane Terminal User Interface (TUI)
        #[arg(long)]
        tui: bool,

        /// The problem or architecture decision to debate
        prompt: String,
    },
    /// Run an Autonomous Multi-Turn Agent with tools (Milestone 2)
    Agent {
        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama (default: auto-detected)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// Comma-separated list of tools to enable: read_file, write_file, run_command, calculator, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum autonomous feedback iterations (default: 10)
        #[arg(long, default_value = "10")]
        max_iterations: usize,

        /// The agent goal or task prompt
        prompt: String,
    },
    /// Dynamic Multi-Agent DAG Workflow Engine (Milestone 3)
    Workflow {
        #[command(subcommand)]
        action: Option<WorkflowAction>,

        /// Decompose a complex objective into an optimal DAG and execute it
        #[arg(long)]
        plan: Option<String>,

        /// Execute a sequential/parallel pipeline expression (e.g. "research -> analyze -> synthesize")
        #[arg(long)]
        run: Option<String>,

        /// Direct goal / pipeline positional argument
        #[arg(index = 1)]
        objective: Option<String>,

        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama (default: auto-detected)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name
        #[arg(short, long)]
        model: Option<String>,

        /// Comma-separated list of tools to enable: read_file, write_file, run_command, calculator, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum concurrency limit for parallel DAG tasks
        #[arg(long)]
        concurrency: Option<usize>,
    },
    /// ECC (Everything Coding Cloud) Autonomous Multi-Agent Engineering Operating System
    Ecc {
        #[command(subcommand)]
        action: EccAction,
    },
    /// Check configured LLM providers, API keys, and model capability bitflags
    Status,
}

#[derive(Subcommand, Debug)]
enum EccAction {
    /// List all available built-in ECC agent presets and discovered agents
    List {
        /// Directory containing custom ECC agent definitions (defaults to .ecc/agents)
        #[arg(long)]
        dir: Option<String>,
    },
    /// List all available built-in and discovered ECC engineering skills
    Skills {
        /// Directory containing custom ECC skills (defaults to .ecc/skills)
        #[arg(long)]
        dir: Option<String>,
    },
    /// Run a specialized ECC agent persona with tool calling
    Run {
        /// Name of the ECC agent (e.g. architect, tdd-engineer, code-reviewer, security-auditor, build-resolver)
        agent: String,

        /// Goal or prompt for the ECC agent
        prompt: String,

        /// Optional ECC skill to attach to the agent context (e.g. tdd-workflow, security-review)
        #[arg(long)]
        skill: Option<String>,

        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name override
        #[arg(short, long)]
        model: Option<String>,

        /// Comma-separated list of tools: read_file, write_file, run_command, calculator, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum autonomous iterations
        #[arg(long, default_value = "10")]
        max_iterations: usize,

        /// Directory containing custom ECC agent definitions
        #[arg(long)]
        dir: Option<String>,
    },
    /// Execute the 5-stage ECC Engineering Workflow Pipeline (Plan -> Test -> Implement -> Review/Security -> Verify)
    Pipeline {
        /// High-level engineering objective or feature to build
        objective: String,

        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Model name override
        #[arg(short, long)]
        model: Option<String>,

        /// Comma-separated list of tools: read_file, write_file, run_command, calculator, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum concurrency limit for parallel DAG tasks
        #[arg(long)]
        concurrency: Option<usize>,
    },
    /// Run an Adversarial ECC Engineering Audit (Architect vs Security Auditor -> Chief Adjudicator)
    Audit {
        /// Launch interactive multi-pane Terminal User Interface (TUI)
        #[arg(long)]
        tui: bool,

        /// Architectural problem or code to audit
        prompt: String,
    },
}

#[derive(Subcommand, Debug)]
enum WorkflowAction {
    /// Plan and execute a dynamic workflow from an objective
    Plan {
        /// Complex objective to decompose
        goal: String,
    },
    /// Run a pipeline expression (e.g. "task1 -> task2")
    Run {
        /// Pipeline expression
        pipeline: String,
    },
}

fn build_engine_context(max_budget: f64) -> EngineContext {
    let mut ctx = EngineContext::new(max_budget);

    // Register Anthropic if key exists
    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        if !key.trim().is_empty() {
            ctx.register_provider(Arc::new(AnthropicProvider::new(key)));
        }
    }

    // Register OpenAI if key exists
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        if !key.trim().is_empty() {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::openai(key)));
        }
    }

    // Register xAI (Grok) if key exists
    if let Ok(key) = env::var("XAI_API_KEY") {
        if !key.trim().is_empty() {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::xai(key)));
        }
    }

    // Register DeepSeek if key exists
    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        if !key.trim().is_empty() {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::deepseek(key)));
        }
    }

    // Register Google Gemini if key exists
    if let Ok(key) = env::var("GEMINI_API_KEY") {
        if !key.trim().is_empty() {
            ctx.register_provider(Arc::new(GeminiProvider::new(key)));
        }
    }

    // Register Local Ollama
    ctx.register_provider(Arc::new(OllamaProvider::default_local()));

    ctx
}

fn format_capabilities(caps: ProviderCapabilities) -> String {
    let mut features = Vec::new();
    if caps.contains(ProviderCapabilities::STREAMING) {
        features.push("Streaming");
    }
    if caps.contains(ProviderCapabilities::REASONING_EXTRACTION) {
        features.push("Reasoning Extraction");
    }
    if caps.contains(ProviderCapabilities::PROMPT_CACHING) {
        features.push("Prompt Caching");
    }
    if caps.contains(ProviderCapabilities::VISION) {
        features.push("Vision");
    }
    if caps.contains(ProviderCapabilities::FUNCTION_CALLING) {
        features.push("Tool Calling");
    }
    features.join(", ")
}

fn default_model_for_provider(provider_id: &str) -> String {
    match provider_id {
        "gemini" => "gemini-2.0-flash".to_string(),
        "deepseek" => "deepseek-chat".to_string(),
        "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
        "openai" => "gpt-4o".to_string(),
        "xai" => "grok-2-latest".to_string(),
        _ => std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:1.5b".to_string()),
    }
}

fn resolve_provider_and_model(
    ctx: &EngineContext,
    user_provider: &str,
    user_model: Option<String>,
) -> Result<(String, String, Arc<dyn LlmProvider>), TagisanError> {
    if user_provider != "auto" {
        let prov = ctx.get_provider(user_provider)?;
        let model = user_model.unwrap_or_else(|| default_model_for_provider(user_provider));
        return Ok((user_provider.to_string(), model, prov));
    }

    // Auto-detection strategy in priority order:
    // 1. Gemini (100% Free Cloud Tier via Google AI Studio)
    // 2. DeepSeek (Ultra-cheap Cloud)
    // 3. Anthropic (Claude 3.5 Sonnet)
    // 4. OpenAI (GPT-4o)
    // 5. xAI (Grok-2)
    // 6. Local Ollama (100% Free Offline)
    let candidate_keys = [
        ("gemini", "GEMINI_API_KEY"),
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
        ("xai", "XAI_API_KEY"),
    ];

    for (id, key_var) in candidate_keys {
        if std::env::var(key_var).is_ok() {
            if let Ok(prov) = ctx.get_provider(id) {
                let model = user_model.unwrap_or_else(|| default_model_for_provider(id));
                return Ok((id.to_string(), model, prov));
            }
        }
    }

    // Fallback to local Ollama
    let prov = ctx.get_provider("ollama")?;
    let model = user_model.unwrap_or_else(|| {
        std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder:1.5b".to_string())
    });
    Ok(("ollama".to_string(), model, prov))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🇵🇭 TAGISAN: Multi-LLM Collaboration Engine in Rust".bold().yellow());
            println!("{}", "=========================================================".cyan());
            println!("{}", "\nChecking Configured LLM Providers & Capabilities:".bold());
            let ctx = build_engine_context(cli.max_budget);

            let provider_specs = [
                ("anthropic", "Anthropic (Claude 3.5)", "ANTHROPIC_API_KEY", "claude-3-5-sonnet-20241022"),
                ("xai", "xAI (Grok 2 / Grok 3)", "XAI_API_KEY", "grok-2-latest"),
                ("openai", "OpenAI (GPT-4o / o1)", "OPENAI_API_KEY", "gpt-4o"),
                ("gemini", "Google Gemini (2.0 Flash)", "GEMINI_API_KEY", "gemini-2.0-flash"),
                ("deepseek", "DeepSeek (R1 / V3)", "DEEPSEEK_API_KEY", "deepseek-reasoner"),
                ("ollama", "Local Ollama (Offline)", "No key required (localhost:11434)", "llama3.2"),
            ];

            for (id, name, var, sample_model) in provider_specs {
                if let Ok(p) = ctx.get_provider(id) {
                    let caps = p.capabilities(sample_model);
                    println!(
                        "  [✓] {:<26} -> Ready ({}) [{}]\n      ↳ Features: {}",
                        name.green().bold(),
                        id.cyan(),
                        sample_model,
                        format_capabilities(caps).italic()
                    );
                } else {
                    println!("  [✗] {:<26} -> Missing {}", name.red(), var);
                }
            }
            println!("\nTip: Configure API keys in your .env file to enable cloud providers.\n");
        }

        Commands::Stream { provider, model, prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            println!(
                "\n{} [{}: {}]...",
                "Streaming from".bold().magenta(),
                provider_id.cyan().bold(),
                model_name.yellow()
            );
            println!("Prompt: \"{}\"\n", prompt.italic());

            let req = CompletionRequest::new(model_name.clone(), prompt)
                .with_stream(true)
                .with_cancellation(ctx.cancellation_token.clone());
            let mut stream = prov.stream(req).await?;

            let start = std::time::Instant::now();
            let mut is_thinking = false;
            let mut last_usage = None;

            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        if let Some(u) = chunk.usage {
                            last_usage = Some(u);
                        }
                        match chunk.delta {
                            StreamChunkDelta::Thinking(thought) => {
                                if !is_thinking {
                                    print!("\n{}\n", "--- Model Thinking Block ---".italic().dimmed());
                                    is_thinking = true;
                                }
                                print!("{}", thought.dimmed());
                                std::io::stdout().flush().ok();
                            }
                            StreamChunkDelta::Text(text) => {
                                if is_thinking {
                                    print!("\n{}\n", "--- Response Output ---".italic().green());
                                    is_thinking = false;
                                }
                                print!("{}", text);
                                std::io::stdout().flush().ok();
                            }
                            StreamChunkDelta::ToolCallDelta { index, id, name, arguments_delta } => {
                                if let Some(n) = name {
                                    print!("\n[Tool Call #{}: {}", index, n);
                                    if let Some(id_str) = id {
                                        print!(" (ID: {})", id_str);
                                    }
                                    print!("] ");
                                }
                                if let Some(args) = arguments_delta {
                                    print!("{}", args);
                                }
                                std::io::stdout().flush().ok();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\n{}: {:?}", "Stream Error".red().bold(), e);
                        break;
                    }
                }
            }

            println!("\n\n{} (Stream finished in {:.2}s)", "✔ Done".green().bold(), start.elapsed().as_secs_f32());

            if let Some(u) = last_usage {
                let cost_res = ctx.budget_tracker.record(&model_name, u.prompt_tokens, u.completion_tokens);
                let total_spent = ctx.budget_tracker.current_spent_usd();
                println!(
                    "Tokens: {} (Prompt: {}, Output: {}) | Session Spent: ${:.4} USD",
                    u.prompt_tokens + u.completion_tokens,
                    u.prompt_tokens,
                    u.completion_tokens,
                    total_spent
                );
                if let Err(e) = cost_res {
                    eprintln!("{}: {:?}", "Budget Alert".yellow().bold(), e);
                }
            }
        }

        Commands::Ask { provider, model, prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            println!(
                "\n{} [{}: {}]...",
                "Querying".bold().magenta(),
                provider_id.cyan().bold(),
                model_name.yellow()
            );

            let mut session = ChatSession::new();
            session.add_user_message(prompt);

            let req = session
                .build_request(model_name.clone())
                .with_cancellation(ctx.cancellation_token.clone());
            let resp = prov.complete(req).await?;

            ctx.budget_tracker.record(&model_name, resp.usage.prompt_tokens, resp.usage.completion_tokens)?;

            if let Some(thinking) = resp.message.extract_thinking() {
                println!("\n{}", "--- Model Thinking / Reasoning ---".dimmed().italic());
                println!("{}\n", thinking.dimmed());
            }

            println!("\n{}", "--- Response ---".bold().green());
            println!("{}\n", resp.message.extract_text());
            println!(
                "Latency: {:.2}s | Tokens: {} (Prompt: {}, Output: {}) | Total Spent: ${:.4} USD",
                resp.latency.as_secs_f32(),
                resp.usage.prompt_tokens + resp.usage.completion_tokens,
                resp.usage.prompt_tokens,
                resp.usage.completion_tokens,
                ctx.budget_tracker.current_spent_usd()
            );
        }

        Commands::Moa { prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            println!("\n{}", "🚀 Starting Mixture-of-Agents (MoA) Collaboration...".bold().magenta());
            println!("Prompt: \"{}\"\n", prompt.italic());

            let mut proposers = Vec::new();
            if env::var("XAI_API_KEY").is_ok() {
                proposers.push(("xai".to_string(), "grok-2-latest".to_string()));
            }
            if env::var("GEMINI_API_KEY").is_ok() {
                proposers.push(("gemini".to_string(), "gemini-2.0-flash".to_string()));
            }
            if env::var("DEEPSEEK_API_KEY").is_ok() {
                proposers.push(("deepseek".to_string(), "deepseek-chat".to_string()));
            }
            if env::var("OPENAI_API_KEY").is_ok() && proposers.is_empty() {
                proposers.push(("openai".to_string(), "gpt-4o-mini".to_string()));
            }
            if proposers.is_empty() {
                proposers.push(("ollama".to_string(), "llama3.2".to_string()));
            }

            let aggregator = if env::var("ANTHROPIC_API_KEY").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if env::var("GEMINI_API_KEY").is_ok() {
                ("gemini".to_string(), "gemini-1.5-pro".to_string())
            } else if env::var("OPENAI_API_KEY").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else {
                ("ollama".to_string(), "llama3.2".to_string())
            };

            let moa = MixtureOfAgentsStrategy::new(proposers, aggregator);
            let input = StrategyInput {
                prompt,
                system_instruction: None,
            };

            let output = moa.execute(input, &ctx).await?;

            for step in &output.intermediate_steps {
                println!(
                    "{} [{} / {}] (took {:.2}s)",
                    "  ✓".green().bold(),
                    step.provider.bold(),
                    step.model.cyan(),
                    step.latency.as_secs_f32()
                );
            }

            println!("\n{}", "================ FINAL SYNTHESIS ================".bold().green());
            println!("{}\n", output.final_answer);
            println!("{}", "=================================================".green());
            println!(
                "Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
                output.total_usage.prompt_tokens + output.total_usage.completion_tokens,
                output.total_cost_usd,
                output.total_latency.as_secs_f32()
            );
        }

        Commands::Debate { tui, prompt } => {
            let ctx = build_engine_context(cli.max_budget);

            if tui {
                // Interactive Ratatui / Crossterm TUI
                tagisan::run_debate_tui(prompt, &ctx).await?;
                return Ok(());
            }

            println!("\n{}", "⚔️  Starting Dialectical Debate (Tagisan ng Talino)...".bold().magenta());
            println!("Topic: \"{}\"\n", prompt.italic());

            let proponent = if env::var("ANTHROPIC_API_KEY").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if env::var("OPENAI_API_KEY").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else {
                ("ollama".to_string(), "llama3.2".to_string())
            };

            let adversary = if env::var("DEEPSEEK_API_KEY").is_ok() {
                ("deepseek".to_string(), "deepseek-reasoner".to_string())
            } else if env::var("XAI_API_KEY").is_ok() {
                ("xai".to_string(), "grok-2-latest".to_string())
            } else if env::var("GEMINI_API_KEY").is_ok() {
                ("gemini".to_string(), "gemini-2.0-flash".to_string())
            } else {
                ("ollama".to_string(), "llama3.2".to_string())
            };

            let adjudicator = if env::var("GEMINI_API_KEY").is_ok() {
                ("gemini".to_string(), "gemini-1.5-pro".to_string())
            } else if env::var("ANTHROPIC_API_KEY").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else {
                ("openai".to_string(), "gpt-4o".to_string())
            };

            let debate = DialecticalDebateStrategy::new(proponent, adversary, adjudicator);
            let input = StrategyInput {
                prompt,
                system_instruction: None,
            };

            let output = debate.execute(input, &ctx).await?;

            for step in &output.intermediate_steps {
                println!("\n{}", format!("--- {} ---", step.step_name).bold().cyan());
                println!("Agent: {} ({}) | Latency: {:.2}s", step.provider.bold(), step.model.yellow(), step.latency.as_secs_f32());
                println!("{}\n", step.message.extract_text());
            }

            println!("{}", "================ LAKANDIWA VERDICT ================".bold().green());
            println!("{}\n", output.final_answer);
            println!("{}", "===================================================".green());
            println!(
                "Total Tokens: {} | Estimated Cost: ${:.4} USD | Total Time: {:.2}s",
                output.total_usage.prompt_tokens + output.total_usage.completion_tokens,
                output.total_cost_usd,
                output.total_latency.as_secs_f32()
            );
        }

        Commands::Agent {
            provider,
            model,
            tools,
            max_iterations,
            prompt,
        } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            // Build Tool Registry
            let mut registry = ToolRegistry::new();
            let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
            let enable_all = tool_list.contains(&"all");

            if enable_all || tool_list.contains(&"read_file") {
                registry.register_tool(ReadFileTool::new());
            }
            if enable_all || tool_list.contains(&"write_file") {
                registry.register_tool(WriteFileTool::new());
            }
            if enable_all || tool_list.contains(&"run_command") {
                registry.register_tool(RunCommandTool::default());
            }
            if enable_all || tool_list.contains(&"calculator") {
                registry.register_tool(CalculatorTool::new());
            }

            println!("\n{}", "🤖 Starting Tagisan Autonomous Agent...".bold().magenta());
            println!("Provider: {} | Model: {}", provider_id.cyan().bold(), model_name.yellow().bold());
            println!("Active Tools: [{}]", registry.names().join(", ").green());
            println!("Max Iterations: {}", max_iterations);
            println!("Goal: \"{}\"\n", prompt.italic());

            let agent = AutonomousAgent::new(prov, model_name, registry)
                .with_max_iterations(max_iterations);

            let result = agent.run(&prompt, &ctx).await?;

            println!("\n{}", "================ AGENT EXECUTION TRACE ================".bold().cyan());
            for step in &result.steps {
                println!("\n{}", format!("--- Iteration {} ---", step.iteration).bold().yellow());
                for (id, name, args) in step.assistant_message.extract_tool_calls() {
                    println!("🔧 Called Tool: {} (ID: {})", name.green().bold(), id.dimmed());
                    println!("   Args: {}", args);
                }
                for res in &step.tool_results {
                    if let ContentBlock::ToolResult { tool_call_id, content, is_error } = res {
                        if *is_error {
                            println!("❌ Result [{}]: {}", tool_call_id.dimmed(), content.red());
                        } else {
                            println!("✔ Result [{}]: {}", tool_call_id.dimmed(), content.dimmed());
                        }
                    }
                }
            }

            println!("\n{}", "================ FINAL ANSWER ================".bold().green());
            println!("{}\n", result.final_answer);
            println!("{}", "==============================================".green());
            println!(
                "Iterations: {} | Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
                result.iterations,
                result.total_usage.prompt_tokens + result.total_usage.completion_tokens,
                result.total_cost_usd,
                result.total_latency.as_secs_f32()
            );
        }

        Commands::Workflow {
            action,
            plan,
            run,
            objective,
            provider,
            model,
            tools,
            concurrency,
        } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;
            println!("Active Engine: {} [{}]", provider_id.cyan().bold(), model_name.yellow().bold());

            // Build Tool Registry
            let mut registry = ToolRegistry::new();
            let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
            let enable_all = tool_list.contains(&"all");

            if enable_all || tool_list.contains(&"read_file") {
                registry.register_tool(ReadFileTool::new());
            }
            if enable_all || tool_list.contains(&"write_file") {
                registry.register_tool(WriteFileTool::new());
            }
            if enable_all || tool_list.contains(&"run_command") {
                registry.register_tool(RunCommandTool::default());
            }
            if enable_all || tool_list.contains(&"calculator") {
                registry.register_tool(CalculatorTool::new());
            }

            // Determine execution mode (Plan vs Run pipeline vs Positional Objective)
            let mut workflow_graph = match (action, plan, run, objective) {
                (Some(WorkflowAction::Plan { goal }), _, _, _) => {
                    println!("\n{}", "🧠 Decomposing Goal with Autonomous Planner...".bold().magenta());
                    println!("Objective: \"{}\"\n", goal.italic());
                    let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                    planner.plan(&goal, &ctx).await?
                }
                (Some(WorkflowAction::Run { pipeline }), _, _, _) => {
                    println!("\n{}", "⚙️ Building Workflow from Pipeline Specification...".bold().cyan());
                    println!("Pipeline: \"{}\"\n", pipeline.italic());
                    let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                    planner.from_pipeline_str(&pipeline, prov.clone(), &model_name, registry.clone())?
                }
                (_, Some(goal), _, _) => {
                    println!("\n{}", "🧠 Decomposing Goal with Autonomous Planner...".bold().magenta());
                    println!("Objective: \"{}\"\n", goal.italic());
                    let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                    planner.plan(&goal, &ctx).await?
                }
                (_, _, Some(pipeline), _) => {
                    println!("\n{}", "⚙️ Building Workflow from Pipeline Specification...".bold().cyan());
                    println!("Pipeline: \"{}\"\n", pipeline.italic());
                    let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                    planner.from_pipeline_str(&pipeline, prov.clone(), &model_name, registry.clone())?
                }
                (_, _, _, Some(obj)) => {
                    if obj.contains("->") {
                        println!("\n{}", "⚙️ Building Workflow from Pipeline Specification...".bold().cyan());
                        println!("Pipeline: \"{}\"\n", obj.italic());
                        let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                        planner.from_pipeline_str(&obj, prov.clone(), &model_name, registry.clone())?
                    } else {
                        println!("\n{}", "🧠 Decomposing Goal with Autonomous Planner...".bold().magenta());
                        println!("Objective: \"{}\"\n", obj.italic());
                        let planner = WorkflowPlanner::new(prov.clone(), model_name.clone()).with_tools(registry.clone());
                        planner.plan(&obj, &ctx).await?
                    }
                }
                (None, None, None, None) => {
                    eprintln!("{}", "Error: Please provide a goal with --plan \"<goal>\" or a pipeline with --run \"step1 -> step2\"".red().bold());
                    std::process::exit(1);
                }
            };

            // Display DAG topology
            println!("{}", "════════════════ WORKFLOW TOPOLOGY ════════════════".bold().blue());
            let topo = workflow_graph.validate()?;
            for (i, task_id) in topo.iter().enumerate() {
                let task = workflow_graph.get_task(task_id).unwrap();
                let deps = workflow_graph.upstream_dependencies(task_id)?;
                let deps_str = if deps.is_empty() {
                    "None (Root Task)".italic().dimmed().to_string()
                } else {
                    deps.join(", ").yellow().to_string()
                };
                println!(
                    "  {}. [{}] {} | Depends on: [{}]",
                    i + 1,
                    task.id.cyan().bold(),
                    task.name.bold(),
                    deps_str
                );
            }
            println!("{}\n", "═══════════════════════════════════════════════════".bold().blue());

            // Initialize Scheduler & Event Stream
            let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
            let mut scheduler = DagScheduler::new()
                .with_id("tagisan_wf")
                .with_event_sender(event_tx);

            if let Some(limit) = concurrency {
                scheduler = scheduler.with_concurrency_limit(limit);
            }

            let event_printer = tokio::spawn(async move {
                while let Some(evt) = event_rx.recv().await {
                    match evt {
                        WorkflowEvent::WorkflowStarted { workflow_id, total_tasks } => {
                            println!(
                                "{} [{}] ({} tasks scheduled)",
                                "🚀 Workflow Execution Started:".bold().magenta(),
                                workflow_id.cyan(),
                                total_tasks
                            );
                        }
                        WorkflowEvent::TaskStarted { task_id, task_name, attempt } => {
                            println!(
                                "  {} {} ({}) [Attempt {}]",
                                "⏳ Starting Task:".yellow().bold(),
                                task_name.bold(),
                                task_id.dimmed(),
                                attempt
                            );
                        }
                        WorkflowEvent::TaskProgress { task_id, message } => {
                            println!("    ↳ [{}] {}", task_id.dimmed(), message.italic());
                        }
                        WorkflowEvent::TaskRetry { task_id, attempt, max_retries, delay, error } => {
                            println!(
                                "  {} Task '{}' (attempt {}/{}) retrying in {:.1}s: {}",
                                "🔄 Retry:".yellow().bold(),
                                task_id.cyan(),
                                attempt,
                                max_retries,
                                delay.as_secs_f32(),
                                error.red()
                            );
                        }
                        WorkflowEvent::TaskCompleted { task_id, output } => {
                            println!(
                                "  {} Task '{}' completed in {:.2}s (Tokens: {})",
                                "✅ Task Succeeded:".green().bold(),
                                task_id.cyan(),
                                output.latency.as_secs_f32(),
                                output.usage.prompt_tokens + output.usage.completion_tokens
                            );
                        }
                        WorkflowEvent::TaskFailed { task_id, error, attempts } => {
                            println!(
                                "  {} Task '{}' failed after {} attempt(s): {}",
                                "❌ Task Failed:".red().bold(),
                                task_id.cyan(),
                                attempts,
                                error.red()
                            );
                        }
                        WorkflowEvent::TaskSkipped { task_id, reason } => {
                            println!(
                                "  {} Task '{}' skipped: {}",
                                "⚠️  Task Skipped:".yellow(),
                                task_id.dimmed(),
                                reason
                            );
                        }
                        WorkflowEvent::WorkflowCompleted { workflow_id, total_tasks, completed_tasks, total_latency, total_cost_usd, .. } => {
                            println!(
                                "\n{} [{}] (Completed: {}/{}, Time: {:.2}s, Spent: ${:.4} USD)",
                                "🎉 Workflow Finished Successfully!".green().bold(),
                                workflow_id.cyan(),
                                completed_tasks,
                                total_tasks,
                                total_latency.as_secs_f32(),
                                total_cost_usd
                            );
                        }
                        WorkflowEvent::WorkflowFailed { workflow_id, error } => {
                            println!(
                                "\n{} [{}] Error: {}",
                                "💥 Workflow Failed!".red().bold(),
                                workflow_id.cyan(),
                                error.red()
                            );
                        }
                    }
                }
            });

            let result = scheduler.run(&mut workflow_graph, &ctx).await;
            let _ = event_printer.await;

            match result {
                Ok(wf_res) => {
                    println!("\n{}", "════════════════ WORKFLOW TASK RESULTS ════════════════".bold().green());
                    for (task_id, output) in &wf_res.task_outputs {
                        println!("\n{}", format!("--- Task [{}] ---", task_id).bold().cyan());
                        println!("Latency: {:.2}s | Tokens: {}", output.latency.as_secs_f32(), output.usage.prompt_tokens + output.usage.completion_tokens);
                        println!("{}\n", output.text);
                    }

                    if let Some(final_text) = wf_res.final_output {
                        println!("{}", "════════════════ FINAL WORKFLOW SYNTHESIS ════════════════".bold().yellow());
                        println!("{}\n", final_text);
                        println!("{}", "══════════════════════════════════════════════════════════".bold().yellow());
                    }

                    println!(
                        "Completed Tasks: {}/{} | Total Tokens: {} | Total Cost: ${:.4} USD | Total Time: {:.2}s",
                        wf_res.completed_tasks,
                        wf_res.completed_tasks + wf_res.failed_tasks,
                        wf_res.total_usage.prompt_tokens + wf_res.total_usage.completion_tokens,
                        wf_res.total_cost_usd,
                        wf_res.total_latency.as_secs_f32()
                    );
                }
                Err(e) => {
                    eprintln!("\n{}: {:?}", "Workflow Execution Error".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Ecc { action } => {
            let ctx = build_engine_context(cli.max_budget);

            match action {
                EccAction::List { dir } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  🏛️  ECC (Everything Coding Cloud) Autonomous Swarm".bold().yellow());
                    println!("{}", "=========================================================".cyan());
                    println!("\n{}", "Built-in Canonical ECC Agent Presets:".bold());

                    for agent in all_ecc_presets() {
                        let model_str = agent.recommended_model.as_deref().unwrap_or("default");
                        println!(
                            "  [•] {:<20} -> {} [{}]\n      ↳ Tools: [{}]",
                            agent.name.green().bold(),
                            agent.description.italic(),
                            model_str.cyan(),
                            agent.tools.join(", ").yellow()
                        );
                    }

                    // Check directory
                    let custom_path = dir
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from(".ecc/agents"));

                    if custom_path.exists() {
                        println!("\n{}", format!("Discovered Agents in '{}':", custom_path.display()).bold());
                        let custom_agents = load_ecc_agents_from_dir(&custom_path);
                        if custom_agents.is_empty() {
                            println!("  (No .md agent files found)");
                        } else {
                            for agent in custom_agents {
                                let model_str = agent.recommended_model.as_deref().unwrap_or("default");
                                println!(
                                    "  [+] {:<20} -> {} [{}]\n      ↳ Tools: [{}]",
                                    agent.name.magenta().bold(),
                                    agent.description.italic(),
                                    model_str.cyan(),
                                    agent.tools.join(", ").yellow()
                                );
                            }
                        }
                    } else {
                        println!("\nTip: Place custom ECC markdown files in '.ecc/agents/*.md' to discover them automatically.\n");
                    }
                }

                EccAction::Skills { dir } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📚  ECC (Everything Coding Cloud) Skills Catalog".bold().yellow());
                    println!("{}", "=========================================================".cyan());
                    println!("\n{}", "Built-in Standard ECC Skills:".bold());

                    for skill in all_ecc_skills() {
                        println!(
                            "  [•] {:<22} -> {}",
                            skill.name.green().bold(),
                            skill.description.italic()
                        );
                    }

                    let custom_path = dir
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from(".ecc/skills"));

                    if custom_path.exists() {
                        println!("\n{}", format!("Discovered Skills in '{}':", custom_path.display()).bold());
                        let custom_skills = load_ecc_skills_from_dir(&custom_path);
                        if custom_skills.is_empty() {
                            println!("  (No skill files found)");
                        } else {
                            for skill in custom_skills {
                                println!(
                                    "  [+] {:<22} -> {}",
                                    skill.name.magenta().bold(),
                                    skill.description.italic()
                                );
                            }
                        }
                    } else {
                        println!("\nTip: Place custom ECC skills in '.ecc/skills/<skill>/SKILL.md' to discover them automatically.\n");
                    }
                }

                EccAction::Run {
                    agent,
                    prompt,
                    skill,
                    provider,
                    model,
                    tools,
                    max_iterations,
                    dir,
                } => {
                    let custom_dir = dir.as_ref().map(std::path::Path::new);
                    let mut ecc_agent = match resolve_ecc_agent(&agent, custom_dir) {
                        Some(a) => a,
                        None => {
                            let default_dir = std::path::Path::new(".ecc/agents");
                            match resolve_ecc_agent(&agent, Some(default_dir)) {
                                Some(a) => a,
                                None => {
                                    eprintln!(
                                        "{}: ECC agent '{}' not found. Run 'tagisan ecc list' to see available agents.",
                                        "Error".red().bold(),
                                        agent
                                    );
                                    std::process::exit(1);
                                }
                            }
                        }
                    };

                    // If a skill was attached, inject it into the agent's system prompt
                    if let Some(ref skill_name) = skill {
                        let skills_dir = std::path::Path::new(".ecc/skills");
                        if let Some(attached_skill) = resolve_ecc_skill(skill_name, Some(skills_dir)) {
                            println!("Attached Skill: {} ({})", attached_skill.name.cyan().bold(), attached_skill.description.italic());
                            ecc_agent.system_prompt.push_str(&format!(
                                "\n\n--- Attached ECC Skill: {} ---\n{}",
                                attached_skill.name, attached_skill.instructions
                            ));
                        } else {
                            eprintln!(
                                "{}: ECC skill '{}' not found. Run 'tagisan ecc skills' to view available skills.",
                                "Warning".yellow().bold(),
                                skill_name
                            );
                        }
                    }

                    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

                    // Build Tool Registry
                    let mut registry = ToolRegistry::new();
                    let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
                    let enable_all = tool_list.contains(&"all");

                    if enable_all || tool_list.contains(&"read_file") || ecc_agent.tools.contains(&"read_file".to_string()) {
                        registry.register_tool(ReadFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"write_file") || ecc_agent.tools.contains(&"write_file".to_string()) {
                        registry.register_tool(WriteFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"run_command") || ecc_agent.tools.contains(&"run_command".to_string()) {
                        registry.register_tool(RunCommandTool::default());
                    }
                    if enable_all || tool_list.contains(&"calculator") || ecc_agent.tools.contains(&"calculator".to_string()) {
                        registry.register_tool(CalculatorTool::new());
                    }

                    println!("\n{}", "🏛️ Launching Autonomous ECC Agent...".bold().magenta());
                    println!("Agent Persona: {} ({})", ecc_agent.name.yellow().bold(), ecc_agent.description.italic());
                    println!("Engine: {} [{}]", provider_id.cyan().bold(), model_name.yellow().bold());
                    println!("Active Tools: [{}]", registry.names().join(", ").green());
                    println!("Prompt: \"{}\"\n", prompt.italic());

                    let autonomous_agent = ecc_agent
                        .into_autonomous_agent(prov, Some(model_name), registry)
                        .with_max_iterations(max_iterations);

                    let result = autonomous_agent.run(&prompt, &ctx).await?;

                    println!("\n{}", "================ ECC AGENT EXECUTION TRACE ================".bold().cyan());
                    for step in &result.steps {
                        println!("\n{}", format!("--- Iteration {} ---", step.iteration).bold().yellow());
                        for (id, name, args) in step.assistant_message.extract_tool_calls() {
                            println!("🔧 Called Tool: {} (ID: {})", name.green().bold(), id.dimmed());
                            println!("   Args: {}", args);
                        }
                        for res in &step.tool_results {
                            if let ContentBlock::ToolResult { tool_call_id, content, is_error } = res {
                                if *is_error {
                                    println!("❌ Result [{}]: {}", tool_call_id.dimmed(), content.red());
                                } else {
                                    println!("✔ Result [{}]: {}", tool_call_id.dimmed(), content.dimmed());
                                }
                            }
                        }
                    }

                    println!("\n{}", "================ FINAL ANSWER ================".bold().green());
                    println!("{}\n", result.final_answer);
                    println!("{}", "==============================================".green());
                    println!(
                        "Iterations: {} | Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
                        result.iterations,
                        result.total_usage.prompt_tokens + result.total_usage.completion_tokens,
                        result.total_cost_usd,
                        result.total_latency.as_secs_f32()
                    );
                }

                EccAction::Pipeline {
                    objective,
                    provider,
                    model,
                    tools,
                    concurrency,
                } => {
                    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

                    println!("{}", "═══════════════════════════════════════════════════════════".bold().blue());
                    println!("{}", "  🏛️  ECC 5-STAGE MULTI-AGENT ENGINEERING PIPELINE".bold().yellow());
                    println!("  Plan -> Test -> Implement -> (Review || Security) -> Verify");
                    println!("{}", "═══════════════════════════════════════════════════════════".bold().blue());
                    println!("Objective: \"{}\"", objective.italic());
                    println!("Active Engine: {} [{}]\n", provider_id.cyan().bold(), model_name.yellow().bold());

                    // Build Tool Registry
                    let mut registry = ToolRegistry::new();
                    let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
                    let enable_all = tool_list.contains(&"all");

                    if enable_all || tool_list.contains(&"read_file") {
                        registry.register_tool(ReadFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"write_file") {
                        registry.register_tool(WriteFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"run_command") {
                        registry.register_tool(RunCommandTool::default());
                    }
                    if enable_all || tool_list.contains(&"calculator") {
                        registry.register_tool(CalculatorTool::new());
                    }

                    let mut pipeline_graph = build_ecc_pipeline(&objective, prov, &model_name, registry)?;

                    // Display DAG topology
                    println!("{}", "════════════════ PIPELINE TOPOLOGY ════════════════".bold().blue());
                    let topo = pipeline_graph.validate()?;
                    for (i, task_id) in topo.iter().enumerate() {
                        let task = pipeline_graph.get_task(task_id).unwrap();
                        let deps = pipeline_graph.upstream_dependencies(task_id)?;
                        let deps_str = if deps.is_empty() {
                            "None (Root Task)".italic().dimmed().to_string()
                        } else {
                            deps.join(", ").yellow().to_string()
                        };
                        println!(
                            "  {}. [{}] {} | Depends on: [{}]",
                            i + 1,
                            task.id.cyan().bold(),
                            task.name.bold(),
                            deps_str
                        );
                    }
                    println!("{}\n", "═══════════════════════════════════════════════════".bold().blue());

                    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
                    let mut scheduler = DagScheduler::new()
                        .with_id("ecc_pipeline")
                        .with_event_sender(event_tx);

                    if let Some(limit) = concurrency {
                        scheduler = scheduler.with_concurrency_limit(limit);
                    }

                    let event_printer = tokio::spawn(async move {
                        while let Some(evt) = event_rx.recv().await {
                            match evt {
                                WorkflowEvent::WorkflowStarted { workflow_id, total_tasks } => {
                                    println!(
                                        "{} [{}] ({} stages scheduled)",
                                        "🚀 Pipeline Execution Started:".bold().magenta(),
                                        workflow_id.cyan(),
                                        total_tasks
                                    );
                                }
                                WorkflowEvent::TaskStarted { task_id, task_name, attempt } => {
                                    println!(
                                        "  {} {} ({}) [Attempt {}]",
                                        "⏳ Starting Stage:".yellow().bold(),
                                        task_name.bold(),
                                        task_id.dimmed(),
                                        attempt
                                    );
                                }
                                WorkflowEvent::TaskCompleted { task_id, output } => {
                                    println!(
                                        "  {} [{}] Finished in {:.2}s ({} tokens)",
                                        "✔ Completed:".green().bold(),
                                        task_id.cyan(),
                                        output.latency.as_secs_f32(),
                                        output.usage.prompt_tokens + output.usage.completion_tokens
                                    );
                                }
                                WorkflowEvent::TaskFailed { task_id, error, attempts } => {
                                    println!(
                                        "  {} [{}] Failed after {} attempts: {}",
                                        "❌ Failed:".red().bold(),
                                        task_id.red(),
                                        attempts,
                                        error
                                    );
                                }
                                _ => {}
                            }
                        }
                    });

                    let result = scheduler.run(&mut pipeline_graph, &ctx).await;
                    let _ = event_printer.await;

                    match result {
                        Ok(wf_res) => {
                            println!("\n{}", "════════════════ ECC PIPELINE STAGE OUTPUTS ════════════════".bold().green());
                            for (task_id, output) in &wf_res.task_outputs {
                                println!("\n{}", format!("--- Stage [{}] ---", task_id).bold().cyan());
                                println!("Latency: {:.2}s | Tokens: {}", output.latency.as_secs_f32(), output.usage.prompt_tokens + output.usage.completion_tokens);
                                println!("{}\n", output.text);
                            }

                            if let Some(final_text) = wf_res.final_output {
                                println!("{}", "════════════════ FINAL VERIFIED & SYNTHESIZED DELIVERABLE ════════════════".bold().yellow());
                                println!("{}\n", final_text);
                                println!("{}", "══════════════════════════════════════════════════════════════════════════".bold().yellow());
                            }

                            println!(
                                "Completed Stages: {}/{} | Total Tokens: {} | Total Cost: ${:.4} USD | Total Time: {:.2}s",
                                wf_res.completed_tasks,
                                wf_res.completed_tasks + wf_res.failed_tasks,
                                wf_res.total_usage.prompt_tokens + wf_res.total_usage.completion_tokens,
                                wf_res.total_cost_usd,
                                wf_res.total_latency.as_secs_f32()
                            );
                        }
                        Err(e) => {
                            eprintln!("\n{}: {:?}", "ECC Pipeline Execution Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }

                EccAction::Audit { tui, prompt } => {
                    if tui {
                        tagisan::run_debate_tui(prompt, &ctx).await?;
                        return Ok(());
                    }

                    println!("\n{}", "🛡️  Starting ECC Adversarial Engineering Audit...".bold().magenta());
                    println!("Architectural Problem / Code: \"{}\"\n", prompt.italic());

                    let architect_spec = if env::var("ANTHROPIC_API_KEY").is_ok() {
                        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                    } else if env::var("OPENAI_API_KEY").is_ok() {
                        ("openai".to_string(), "gpt-4o".to_string())
                    } else if env::var("GEMINI_API_KEY").is_ok() {
                        ("gemini".to_string(), "gemini-1.5-pro".to_string())
                    } else {
                        ("ollama".to_string(), "llama3.2".to_string())
                    };

                    let security_spec = if env::var("DEEPSEEK_API_KEY").is_ok() {
                        ("deepseek".to_string(), "deepseek-reasoner".to_string())
                    } else if env::var("XAI_API_KEY").is_ok() {
                        ("xai".to_string(), "grok-2-latest".to_string())
                    } else if env::var("GEMINI_API_KEY").is_ok() {
                        ("gemini".to_string(), "gemini-2.0-flash".to_string())
                    } else {
                        ("ollama".to_string(), "llama3.2".to_string())
                    };

                    let adjudicator_spec = if env::var("GEMINI_API_KEY").is_ok() {
                        ("gemini".to_string(), "gemini-1.5-pro".to_string())
                    } else if env::var("ANTHROPIC_API_KEY").is_ok() {
                        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                    } else {
                        ("openai".to_string(), "gpt-4o".to_string())
                    };

                    let audit = EccAuditDebate::new(architect_spec, security_spec, adjudicator_spec);
                    let input = StrategyInput {
                        prompt,
                        system_instruction: None,
                    };

                    let output = audit.execute(input, &ctx).await?;

                    for step in &output.intermediate_steps {
                        println!("\n{}", format!("--- {} ---", step.step_name).bold().cyan());
                        println!("Agent: {} ({}) | Latency: {:.2}s", step.provider.bold(), step.model.yellow(), step.latency.as_secs_f32());
                        println!("{}\n", step.message.extract_text());
                    }

                    println!("{}", "================ DEFINITIVE AUDIT VERDICT ================".bold().green());
                    println!("{}\n", output.final_answer);
                    println!("{}", "==========================================================".green());
                    println!(
                        "Total Tokens: {} | Estimated Cost: ${:.4} USD | Total Time: {:.2}s",
                        output.total_usage.prompt_tokens + output.total_usage.completion_tokens,
                        output.total_cost_usd,
                        output.total_latency.as_secs_f32()
                    );
                }
            }
        }
    }

    Ok(())
}
