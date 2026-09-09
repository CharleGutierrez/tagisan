use clap::{Parser, Subcommand};
use colored::Colorize;
use futures::StreamExt;
use std::env;
use std::io::Write;
use std::sync::Arc;
use crate::{
    all_ecc_presets, all_ecc_skills, build_ecc_pipeline, load_ecc_agents_from_dir,
    load_ecc_skills_from_dir, resolve_ecc_agent, resolve_ecc_skill,
    AnthropicProvider, AutonomousAgent, CalculatorTool, ChatSession, CollaborationStrategy,
    CompletionRequest, ContentBlock, DagScheduler, DialecticalDebateStrategy, EccAuditDebate,
    EngineContext, GeminiProvider, LlmProvider, MixtureOfAgentsStrategy, OllamaProvider,
    OpenAiCompatibleProvider, ProviderCapabilities, ReadFileTool, RunCommandTool, StrategyInput,
    StreamChunkDelta, TagisanError, ToolRegistry, ViewImageTool, WorkflowEvent, WorkflowPlanner, WriteFileTool,
    McpManager, WorktreeSandbox,
    InteractiveRepl, SessionStore,
    SwarmCoordinator, SwarmMember, TeamConsensusEngine, VotingRule,
};

#[derive(Parser)]
#[command(name = "tgs", bin_name = "tgs")]
#[command(
    about = "🇵🇭 TGS (Tagisan ng Talino): High-Performance Multi-LLM Collaboration, Adversarial Debate & ECC Swarm in Rust",
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

        /// Optional path to an image file for multimodal vision analysis (.png, .jpg, .jpeg, .webp, .gif)
        #[arg(short, long)]
        image: Option<String>,

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

        /// Optional path to an image file for multimodal vision analysis (.png, .jpg, .jpeg, .webp, .gif)
        #[arg(short, long)]
        image: Option<String>,

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

        /// Comma-separated list of tools to enable: read_file, write_file, run_command, calculator, view_image, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum autonomous feedback iterations (default: 10)
        #[arg(long, default_value = "10")]
        max_iterations: usize,

        /// Optional path to an image file for multimodal vision analysis (.png, .jpg, .jpeg, .webp, .gif)
        #[arg(short, long)]
        image: Option<String>,

        /// Disable AgentShield security interception (safety is enabled by default)
        #[arg(long)]
        no_shield: bool,

        /// Optional path to mcp.json configuration file to load external MCP tools
        #[arg(long)]
        mcp_config: Option<String>,

        /// Enable loading external tools from discovered mcp.json
        #[arg(long)]
        mcp: bool,

        /// Enable long-term vector memory (.tagisan/memory.json)
        #[arg(long)]
        memory: bool,

        /// Run the agent in an isolated Git worktree sandbox
        #[arg(long)]
        sandbox: bool,

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

        /// Optional path to mcp.json configuration file to load external MCP tools
        #[arg(long)]
        mcp_config: Option<String>,

        /// Enable loading external tools from discovered mcp.json
        #[arg(long)]
        mcp: bool,

        /// Enable long-term vector memory (.tagisan/memory.json)
        #[arg(long)]
        memory: bool,
    },
    /// ECC (Everything Coding Cloud) Autonomous Multi-Agent Engineering Operating System
    Ecc {
        #[command(subcommand)]
        action: EccAction,
    },
    /// Model Context Protocol (MCP) Client: inspect servers, list tools, and execute calls
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
    /// Long-term vector memory and semantic codebase RAG subsystem (Milestone 6)
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Check configured LLM providers, API keys, and model capability bitflags
    Status,
    /// Start a Model Context Protocol (MCP) Server over stdio JSON-RPC 2.0 (Milestone 7)
    #[command(name = "serve-mcp")]
    ServeMcp,
    /// Multi-Agent Swarm Orchestration & Team Collaboration (Milestone 8)
    Swarm {
        #[command(subcommand)]
        action: SwarmAction,
    },
    /// Multi-Agent Peer Review & Consensus Voting Protocol (Milestone 8)
    Consensus {
        /// Voting rule: majority, unanimous, supermajority, borda (default: majority)
        #[arg(short, long, default_value = "majority")]
        rule: String,

        /// Path to an artifact file or direct proposal text to review
        artifact: String,
    },
    /// Interactive Multi-Turn Agent Session REPL (Milestone 8)
    Repl {
        /// Initial agent persona (e.g. architect, tdd-engineer, code-reviewer, security-auditor)
        #[arg(short, long)]
        agent: Option<String>,

        /// Model name (defaults to auto-detected)
        #[arg(short, long)]
        model: Option<String>,

        /// Provider ID: auto, anthropic, openai, xai, deepseek, gemini, ollama (default: auto)
        #[arg(short, long, default_value = "auto")]
        provider: String,

        /// Enable long-term vector memory
        #[arg(long)]
        memory: bool,

        /// Run in an isolated Git worktree sandbox
        #[arg(long)]
        sandbox: bool,

        /// Resume a previously saved session by ID
        #[arg(long)]
        resume: Option<String>,
    },
    /// Persistent Agent Session Management (Milestone 8)
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
}

#[derive(Subcommand, Debug)]
enum McpAction {
    /// Run Tagisan as a standard Model Context Protocol (MCP) server over stdio JSON-RPC 2.0
    Serve,
    /// List all configured MCP servers and discover their published tools
    List {
        /// Optional path to mcp.json configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Test connection and initialize handshake with a specific MCP server
    Test {
        /// Server name as defined in mcp.json
        server: String,

        /// Optional path to mcp.json configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Directly execute an MCP tool on a specified server
    Call {
        /// Server name as defined in mcp.json
        server: String,

        /// Tool name to execute
        tool: String,

        /// Arguments as JSON string (e.g. '{"path":"."}')
        #[arg(default_value = "{}")]
        arguments: String,

        /// Optional path to mcp.json configuration file
        #[arg(short, long)]
        config: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum MemoryAction {
    /// Index source files in a directory into long-term vector memory
    Index {
        /// Path to directory to index (defaults to current directory '.')
        #[arg(default_value = ".")]
        path: String,
    },
    /// Search indexed memory and codebase semantically
    Search {
        /// Search query
        query: String,

        /// Maximum number of results to display
        #[arg(short, long, default_value = "5")]
        top_k: usize,

        /// Minimum similarity threshold between 0.0 and 1.0
        #[arg(long, default_value = "0.05")]
        threshold: f32,
    },
    /// Display statistics about indexed long-term memory
    Stats,
    /// Clear and wipe persistent memory
    Clear,
}

#[derive(Subcommand, Debug)]
enum SwarmAction {
    /// Execute task via Lead Agent with dynamic specialist delegation
    Run {
        /// High-level goal or task prompt
        prompt: String,

        /// Comma-separated list of agent personas in swarm (default: architect,tdd-engineer,security-auditor)
        #[arg(short, long, default_value = "architect,tdd-engineer,security-auditor")]
        agents: String,

        /// Lead agent name (default: architect)
        #[arg(long, default_value = "architect")]
        lead: String,
    },
    /// Execute sequential multi-stage pipeline across agents
    Pipeline {
        /// Initial task description or input
        prompt: String,

        /// Comma-separated list of agent stages (default: architect,tdd-engineer,code-reviewer,security-auditor)
        #[arg(short, long, default_value = "architect,tdd-engineer,code-reviewer,security-auditor")]
        stages: String,
    },
    /// Broadcast prompt to all swarm members concurrently and collect evaluations
    Broadcast {
        /// Prompt or code to broadcast
        prompt: String,

        /// Comma-separated list of agents (default: architect,tdd-engineer,security-auditor)
        #[arg(short, long, default_value = "architect,tdd-engineer,security-auditor")]
        agents: String,
    },
}

#[derive(Subcommand, Debug)]
enum SessionAction {
    /// List all saved sessions
    List,
    /// Resume an interactive session
    Resume {
        /// Session ID to resume
        id: String,
    },
    /// Export session transcript to Markdown
    Export {
        /// Session ID to export
        id: String,

        /// Optional output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Delete a saved session
    Delete {
        /// Session ID to delete
        id: String,
    },
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

        /// Comma-separated list of tools: read_file, write_file, run_command, calculator, view_image, all
        #[arg(short, long, default_value = "all")]
        tools: String,

        /// Maximum autonomous iterations
        #[arg(long, default_value = "10")]
        max_iterations: usize,

        /// Disable AgentShield security interception (safety is enabled by default)
        #[arg(long)]
        no_shield: bool,

        /// Optional path to mcp.json configuration file to load external MCP tools
        #[arg(long)]
        mcp_config: Option<String>,

        /// Enable loading external tools from discovered mcp.json
        #[arg(long)]
        mcp: bool,

        /// Enable long-term vector memory (.tagisan/memory.json)
        #[arg(long)]
        memory: bool,

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

        /// Optional path to mcp.json configuration file to load external MCP tools
        #[arg(long)]
        mcp_config: Option<String>,

        /// Enable loading external tools from discovered mcp.json
        #[arg(long)]
        mcp: bool,

        /// Enable long-term vector memory (.tagisan/memory.json)
        #[arg(long)]
        memory: bool,
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

async fn load_and_register_mcp_tools(
    mcp_enabled: bool,
    mcp_config_path: Option<&str>,
    registry: &mut ToolRegistry,
) -> Result<Option<McpManager>, Box<dyn std::error::Error>> {
    if !mcp_enabled && mcp_config_path.is_none() {
        return Ok(None);
    }

    let mut manager = match McpManager::load(mcp_config_path.map(std::path::Path::new)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: Failed to load MCP config: {}", "Warning".yellow().bold(), e);
            return Ok(None);
        }
    };

    if manager.is_empty() {
        println!("No MCP servers found in configuration.");
        return Ok(None);
    }

    println!("{}", "🔌 Connecting to configured MCP server(s)...".bold().cyan());
    match manager.connect_all().await {
        Ok(discovered) => {
            for (server_name, tools) in &discovered {
                println!("  [✓] Server '{}' connected ({} tool(s) discovered)", server_name.green().bold(), tools.len());
                for t in tools {
                    let desc = t.description.as_deref().unwrap_or("No description");
                    println!("      ↳ {}: {}", t.name.cyan(), desc);
                }
            }
            let count = manager.populate_tool_registry(registry, true);
            println!("Registered {} external MCP tool(s) into tool registry.\n", count);
        }
        Err(e) => {
            eprintln!("{}: MCP connection error: {}", "Warning".yellow().bold(), e);
        }
    }

    Ok(Some(manager))
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
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

        Commands::Stream { provider, model, image, prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            println!(
                "\n{} [{}: {}]...",
                "Streaming from".bold().magenta(),
                provider_id.cyan().bold(),
                model_name.yellow()
            );
            println!("Prompt: \"{}\"", prompt.italic());

            let mut user_blocks = vec![ContentBlock::text(prompt)];
            if let Some(ref img_path) = image {
                let img_block = ContentBlock::from_image_file(img_path)?;
                println!("Attached Multimodal Image: {}", img_path.cyan().bold());
                user_blocks.push(img_block);
            }
            println!();

            let req = CompletionRequest::new(model_name.clone(), "")
                .with_messages(vec![crate::types::Message::user_with_content(user_blocks)])
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
                let cost_res = ctx.budget_tracker.record_usage(&model_name, &u);
                let total_spent = ctx.budget_tracker.current_spent_usd();
                println!(
                    "Tokens: {} (Prompt: {}, Output: {}, Cached: {}) | Session Spent: ${:.4} USD",
                    u.prompt_tokens + u.completion_tokens,
                    u.prompt_tokens,
                    u.completion_tokens,
                    u.cached_prompt_tokens.unwrap_or(0),
                    total_spent
                );
                if let Err(e) = cost_res {
                    eprintln!("{}: {:?}", "Budget Alert".yellow().bold(), e);
                }
            }
        }

        Commands::Ask { provider, model, image, prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            println!(
                "\n{} [{}: {}]...",
                "Querying".bold().magenta(),
                provider_id.cyan().bold(),
                model_name.yellow()
            );

            let mut user_blocks = vec![ContentBlock::text(prompt)];
            if let Some(ref img_path) = image {
                let img_block = ContentBlock::from_image_file(img_path)?;
                println!("Attached Multimodal Image: {}\n", img_path.cyan().bold());
                user_blocks.push(img_block);
            }

            let mut session = ChatSession::new();
            session.add_user_message_with_blocks(user_blocks);

            let req = session
                .build_request(model_name.clone())
                .with_cancellation(ctx.cancellation_token.clone());
            let resp = prov.complete(req).await?;

            ctx.budget_tracker.record_usage(&model_name, &resp.usage)?;

            if let Some(thinking) = resp.message.extract_thinking() {
                println!("\n{}", "--- Model Thinking / Reasoning ---".dimmed().italic());
                println!("{}\n", thinking.dimmed());
            }

            println!("\n{}", "--- Response ---".bold().green());
            println!("{}\n", resp.message.extract_text());
            println!(
                "Latency: {:.2}s | Tokens: {} (Prompt: {}, Output: {}, Cached: {}) | Total Spent: ${:.4} USD",
                resp.latency.as_secs_f32(),
                resp.usage.prompt_tokens + resp.usage.completion_tokens,
                resp.usage.prompt_tokens,
                resp.usage.completion_tokens,
                resp.usage.cached_prompt_tokens.unwrap_or(0),
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
                crate::run_debate_tui(prompt, &ctx).await?;
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
            image,
            no_shield,
            mcp_config,
            mcp,
            memory,
            sandbox,
            prompt,
        } => {
            let ctx = build_engine_context(cli.max_budget);
            let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

            // Build Tool Registry & Optional Worktree Sandbox
            let mut sandbox_holder = None;
            let mut registry = if sandbox {
                let branch_name = format!("tgs-sandbox-{}", std::process::id());
                match WorktreeSandbox::create(".", &branch_name) {
                    Ok(sb) => {
                        println!("🔒 Git Worktree Sandbox: Provisioned branch '{}' at '{}'", branch_name.yellow(), sb.path().display().to_string().cyan());
                        let reg = ToolRegistry::with_builtins_in_dir(sb.path());
                        sandbox_holder = Some(sb);
                        reg
                    }
                    Err(e) => {
                        eprintln!("⚠️ Failed to initialize Git worktree sandbox ({e}), falling back to local workspace");
                        let mut reg = ToolRegistry::new();
                        let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
                        let enable_all = tool_list.contains(&"all");

                        if enable_all || tool_list.contains(&"read_file") {
                            reg.register_tool(ReadFileTool::new());
                        }
                        if enable_all || tool_list.contains(&"write_file") {
                            reg.register_tool(WriteFileTool::new());
                        }
                        if enable_all || tool_list.contains(&"run_command") {
                            reg.register_tool(RunCommandTool::default());
                        }
                        if enable_all || tool_list.contains(&"calculator") {
                            reg.register_tool(CalculatorTool::new());
                        }
                        if enable_all || tool_list.contains(&"view_image") {
                            reg.register_tool(ViewImageTool::new());
                        }
                        reg
                    }
                }
            } else {
                let mut reg = ToolRegistry::new();
                let tool_list: Vec<&str> = tools.split(',').map(|s| s.trim()).collect();
                let enable_all = tool_list.contains(&"all");

                if enable_all || tool_list.contains(&"read_file") {
                    reg.register_tool(ReadFileTool::new());
                }
                if enable_all || tool_list.contains(&"write_file") {
                    reg.register_tool(WriteFileTool::new());
                }
                if enable_all || tool_list.contains(&"run_command") {
                    reg.register_tool(RunCommandTool::default());
                }
                if enable_all || tool_list.contains(&"calculator") {
                    reg.register_tool(CalculatorTool::new());
                }
                if enable_all || tool_list.contains(&"view_image") {
                    reg.register_tool(ViewImageTool::new());
                }
                reg
            };

            let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

            let shield_active = !no_shield;
            println!("\n{}", "🤖 Starting Tagisan Autonomous Agent...".bold().magenta());
            println!("Provider: {} | Model: {}", provider_id.cyan().bold(), model_name.yellow().bold());
            println!("Active Tools: [{}]", registry.names().join(", ").green());
            println!(
                "AgentShield Guardrails: {}",
                if shield_active {
                    "ACTIVE (Enabled by Default)".green().bold()
                } else {
                    "DISABLED (--no-shield)".red().bold()
                }
            );
            println!("Max Iterations: {}", max_iterations);
            if let Some(ref img_path) = image {
                println!("Attached Multimodal Image: {}", img_path.cyan().bold());
            }
            println!("Goal: \"{}\"\n", prompt.italic());

            let mut agent = AutonomousAgent::new(prov, model_name, registry)
                .with_agentshield(shield_active)
                .with_max_iterations(max_iterations);

            if let Some(ref sb) = sandbox_holder {
                agent = agent.with_working_dir(sb.path());
            }

            if memory {
                let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                let emb_prov = crate::memory::default_embedding_provider();
                println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                agent = agent.with_memory(mem_store, emb_prov);
            }

            let result = if let Some(ref img_path) = image {
                let img_block = ContentBlock::from_image_file(img_path)?;
                agent.run_with_content(vec![ContentBlock::text(&prompt), img_block], &ctx).await?
            } else {
                agent.run(&prompt, &ctx).await?
            };

            let sanitized_result = if shield_active {
                AutonomousAgent::sanitize_agent_result(result)
            } else {
                result
            };

            println!("\n{}", "================ AGENT EXECUTION TRACE ================".bold().cyan());
            for step in &sanitized_result.steps {
                println!("\n{}", format!("--- Iteration {} ---", step.iteration).bold().yellow());
                for (id, name, args) in step.assistant_message.extract_tool_calls() {
                    let sanitized_args = if shield_active {
                        AutonomousAgent::sanitize_text(&args.to_string())
                    } else {
                        args.to_string()
                    };
                    println!("🔧 Called Tool: {} (ID: {})", name.green().bold(), id.dimmed());
                    println!("   Args: {}", sanitized_args);
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
            println!("{}\n", sanitized_result.final_answer);
            println!("{}", "==============================================".green());
            println!(
                "Iterations: {} | Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
                sanitized_result.iterations,
                sanitized_result.total_usage.prompt_tokens + sanitized_result.total_usage.completion_tokens,
                sanitized_result.total_cost_usd,
                sanitized_result.total_latency.as_secs_f32()
            );

            if let Some(mut sb) = sandbox_holder {
                if let Ok(diff) = sb.diff() {
                    if !diff.trim().is_empty() {
                        println!("\n{}", "================ SANDBOX GIT DIFF ================".bold().yellow());
                        println!("{}", diff.trim());
                        println!("{}", "==================================================".yellow());
                    }
                }
                let _ = sb.cleanup();
                println!("🧹 Git Worktree Sandbox cleaned up.");
            }
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
            mcp_config,
            mcp,
            memory,
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

            let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

            if memory {
                let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                let emb_prov = crate::memory::default_embedding_provider();
                println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                registry.register_tool(crate::tools::builtin::SearchMemoryTool::new(mem_store.clone(), emb_prov.clone()));
                registry.register_tool(crate::tools::builtin::SaveMemoryTool::new(mem_store, emb_prov));
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
                    no_shield,
                    mcp_config,
                    mcp,
                    memory,
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
                    if enable_all || tool_list.contains(&"view_image") || ecc_agent.tools.contains(&"view_image".to_string()) {
                        registry.register_tool(ViewImageTool::new());
                    }

                    let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

                    let shield_active = !no_shield;
                    println!("\n{}", "🏛️ Launching Autonomous ECC Agent...".bold().magenta());
                    println!("Agent Persona: {} ({})", ecc_agent.name.yellow().bold(), ecc_agent.description.italic());
                    println!("Engine: {} [{}]", provider_id.cyan().bold(), model_name.yellow().bold());
                    println!("Active Tools: [{}]", registry.names().join(", ").green());
                    println!(
                        "AgentShield Guardrails: {}",
                        if shield_active {
                            "ACTIVE (Enabled by Default)".green().bold()
                        } else {
                            "DISABLED (--no-shield)".red().bold()
                        }
                    );
                    println!("Prompt: \"{}\"\n", prompt.italic());

                    let mut autonomous_agent = ecc_agent
                        .into_autonomous_agent(prov, Some(model_name), registry)
                        .with_agentshield(shield_active)
                        .with_max_iterations(max_iterations);

                    if memory {
                        let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                        let emb_prov = crate::memory::default_embedding_provider();
                        println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                        autonomous_agent = autonomous_agent.with_memory(mem_store, emb_prov);
                    }

                    let result = autonomous_agent.run(&prompt, &ctx).await?;

                    let sanitized_result = if shield_active {
                        AutonomousAgent::sanitize_agent_result(result)
                    } else {
                        result
                    };

                    println!("\n{}", "================ ECC AGENT EXECUTION TRACE ================".bold().cyan());
                    for step in &sanitized_result.steps {
                        println!("\n{}", format!("--- Iteration {} ---", step.iteration).bold().yellow());
                        for (id, name, args) in step.assistant_message.extract_tool_calls() {
                            let sanitized_args = if shield_active {
                                AutonomousAgent::sanitize_text(&args.to_string())
                            } else {
                                args.to_string()
                            };
                            println!("🔧 Called Tool: {} (ID: {})", name.green().bold(), id.dimmed());
                            println!("   Args: {}", sanitized_args);
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
                    println!("{}\n", sanitized_result.final_answer);
                    println!("{}", "==============================================".green());
                    println!(
                        "Iterations: {} | Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
                        sanitized_result.iterations,
                        sanitized_result.total_usage.prompt_tokens + sanitized_result.total_usage.completion_tokens,
                        sanitized_result.total_cost_usd,
                        sanitized_result.total_latency.as_secs_f32()
                    );
                }

                EccAction::Pipeline {
                    objective,
                    provider,
                    model,
                    tools,
                    concurrency,
                    mcp_config,
                    mcp,
                    memory,
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

                    let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

                    if memory {
                        let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                        let emb_prov = crate::memory::default_embedding_provider();
                        println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                        registry.register_tool(crate::tools::builtin::SearchMemoryTool::new(mem_store.clone(), emb_prov.clone()));
                        registry.register_tool(crate::tools::builtin::SaveMemoryTool::new(mem_store, emb_prov));
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
                        crate::run_debate_tui(prompt, &ctx).await?;
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

        Commands::ServeMcp => {
            let server = crate::mcp::McpServer::default_server();
            server.run_default_stdio().await?;
        }

        Commands::Mcp { action } => {
            match action {
                McpAction::Serve => {
                    let server = crate::mcp::McpServer::default_server();
                    server.run_default_stdio().await?;
                }
                McpAction::List { config } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  🔌  Model Context Protocol (MCP) Server Discovery".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let mut manager = match McpManager::load(config.as_deref().map(std::path::Path::new)) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("{}: Failed to load MCP configuration: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let server_names = manager.server_names();
                    if server_names.is_empty() {
                        println!("\nNo MCP servers configured.");
                        println!("Tip: Define servers in 'mcp.json' or 'tagisan.mcp.json'. Example:\n");
                        println!("{{\n  \"mcpServers\": {{\n    \"sqlite\": {{\n      \"command\": \"uvx\",\n      \"args\": [\"mcp-server-sqlite\", \"--db-path\", \"test.db\"]\n    }}\n  }}\n}}");
                        return Ok(());
                    }

                    println!("\nConfigured MCP Servers ({}):", server_names.len());
                    for name in &server_names {
                        if let Some(srv_cfg) = manager.config().mcp_servers.get(name) {
                            println!("  [•] {:<20} -> {} {}", name.cyan().bold(), srv_cfg.command.green(), srv_cfg.args.join(" ").dimmed());
                        }
                    }

                    println!("\nConnecting and discovering published tools...");
                    match manager.connect_all().await {
                        Ok(discovered) => {
                            for (server_name, tools) in discovered {
                                println!("\n{}", format!("Server [{}] ({} tools):", server_name, tools.len()).bold().green());
                                if tools.is_empty() {
                                    println!("  (No tools published)");
                                } else {
                                    for t in tools {
                                        let desc = t.description.as_deref().unwrap_or("No description");
                                        println!("  [+] {:<24} -> {}", t.name.yellow().bold(), desc);
                                        let schema_str = serde_json::to_string(&t.input_schema).unwrap_or_default();
                                        println!("      ↳ Schema: {}", schema_str.dimmed());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}: Failed to query MCP tools: {}", "Error".red().bold(), e);
                        }
                    }
                    manager.shutdown_all().await;
                }

                McpAction::Test { server, config } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🔌  Testing MCP Server: {}", server).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let mut manager = match McpManager::load(config.as_deref().map(std::path::Path::new)) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("{}: Failed to load MCP configuration: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let start = std::time::Instant::now();
                    match manager.connect_server(&server).await {
                        Ok(client) => {
                            let duration = start.elapsed();
                            println!("{} Handshake successful in {:.2}s!", "✔".green().bold(), duration.as_secs_f32());
                            println!("Server Name: {}", client.server_name.cyan().bold());
                            println!("Protocol Version: {}", client.protocol_version.yellow());
                            if let Some(ref info) = client.server_info {
                                let ver = info.version.as_deref().unwrap_or("unknown");
                                println!("Implementation: {} (v{})", info.name.green(), ver.dimmed());
                            }

                            // Perform standard MCP ping probe
                            let ping_start = std::time::Instant::now();
                            match client.ping().await {
                                Ok(()) => {
                                    println!("{} MCP ping probe successful in {:.2}ms!", "✔".green().bold(), ping_start.elapsed().as_secs_f32() * 1000.0);
                                }
                                Err(e) => {
                                    println!("{} MCP ping probe warning/unsupported: {}", "⚠".yellow().bold(), e);
                                }
                            }

                            match client.list_tools().await {
                                Ok(tools) => {
                                    println!("\nPublished Tools ({}):", tools.len());
                                    for t in tools {
                                        let desc = t.description.as_deref().unwrap_or("No description");
                                        println!("  • {:<20} - {}", t.name.cyan().bold(), desc);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to list tools: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("\n{} Connection failed: {}", "❌".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                    manager.shutdown_all().await;
                }

                McpAction::Call {
                    server,
                    tool,
                    arguments,
                    config,
                } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🛠️  Executing MCP Tool: {}::{}", server, tool).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let parsed_args: serde_json::Value = match serde_json::from_str(&arguments) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("{}: Invalid arguments JSON: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let mut manager = match McpManager::load(config.as_deref().map(std::path::Path::new)) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("{}: Failed to load MCP configuration: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let client = match manager.connect_server(&server).await {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}: Could not connect to server '{}': {}", "Error".red().bold(), server, e);
                            std::process::exit(1);
                        }
                    };

                    println!("Server: {}", server.cyan().bold());
                    println!("Tool:   {}", tool.yellow().bold());
                    println!("Args:   {}\n", arguments.dimmed());

                    // AgentShield scanning on tool call
                    let verdict = crate::AgentShieldScanner::scan_tool_call(&tool, &parsed_args);
                    if let crate::AgentShieldVerdict::Block { reason, threat_level } = verdict {
                        eprintln!("{}: Call blocked by AgentShield [{:?}]: {}", "Security Alert".red().bold(), threat_level, reason);
                        std::process::exit(1);
                    }
                    if let Some(cmd) = parsed_args.get("command").and_then(|v| v.as_str()) {
                        if let crate::AgentShieldVerdict::Block { reason, threat_level } = crate::AgentShieldScanner::scan_command(cmd) {
                            eprintln!("{}: Command argument blocked by AgentShield [{:?}]: {}", "Security Alert".red().bold(), threat_level, reason);
                            std::process::exit(1);
                        }
                    }

                    let start = std::time::Instant::now();
                    match client.call_tool(&tool, parsed_args).await {
                        Ok(call_result) => {
                            let duration = start.elapsed();
                            let sanitized = AutonomousAgent::sanitize_text(&call_result.extract_text());
                            if call_result.is_error {
                                println!("{} Tool execution reported error (took {:.2}s):\n{}", "❌".red().bold(), duration.as_secs_f32(), sanitized.red());
                            } else {
                                println!("{} Execution succeeded (took {:.2}s):\n", "✔".green().bold(), duration.as_secs_f32());
                                println!("{}\n", sanitized);
                            }
                        }
                        Err(e) => {
                            eprintln!("{} Execution failed: {}", "❌".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                    manager.shutdown_all().await;
                }
            }
        }

        Commands::Memory { action } => {
            match action {
                MemoryAction::Index { path } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🧠  Indexing Codebase into Memory: {}", path).bold().magenta());
                    println!("{}", "=========================================================".cyan());

                    let store = crate::memory::VectorStore::load_or_default();
                    let provider = crate::memory::default_embedding_provider();
                    println!("Embedding Provider: {} ({} dims)", provider.provider_id().cyan().bold(), provider.dimensions());

                    let indexer = crate::memory::CodebaseIndexer::new(provider);
                    let start = std::time::Instant::now();
                    let count = indexer.index_directory(&path, &store).await?;

                    let default_path = crate::memory::VectorStore::default_path();
                    store.save_to_file(&default_path)?;

                    println!("\n{} Indexed {} total chunks in {:.2}s!", "✔".green().bold(), count, start.elapsed().as_secs_f32());
                    println!("Persistent Memory File: {}", default_path.display().to_string().yellow());
                }
                MemoryAction::Search { query, top_k, threshold } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🔍  Semantic Memory Search: \"{}\"", query).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let store = crate::memory::VectorStore::load_or_default();
                    if store.is_empty() {
                        println!("{}: Memory store is empty. Index your codebase first with `tgs memory index`.", "Note".yellow().bold());
                        return Ok(());
                    }

                    let provider = crate::memory::default_embedding_provider();
                    let emb = provider.embed_text(&query).await?;
                    let hits = store.search(&emb, top_k, threshold);

                    if hits.is_empty() {
                        println!("No relevant matches found above threshold {threshold:.2}.");
                        return Ok(());
                    }

                    println!("\nTop {} Match(es):\n", hits.len());
                    for (i, hit) in hits.iter().enumerate() {
                        let doc = &hit.document;
                        let file_path = doc.metadata.get("file_path").map(|s| s.as_str()).unwrap_or(&doc.id);
                        let start_line = doc.metadata.get("start_line").map(|s| s.as_str()).unwrap_or("?");
                        let end_line = doc.metadata.get("end_line").map(|s| s.as_str()).unwrap_or("?");
                        let lang = doc.metadata.get("language").map(|s| s.as_str()).unwrap_or("text");

                        println!(
                            "  {}. {} (Lines {}-{}, [{}]) | Similarity: {}",
                            i + 1,
                            file_path.cyan().bold(),
                            start_line,
                            end_line,
                            lang.yellow(),
                            format!("{:.3}", hit.score).green().bold()
                        );
                        let preview: String = doc.text.lines().take(4).collect::<Vec<_>>().join("\n");
                        println!("     {}\n", preview.dimmed());
                    }
                }
                MemoryAction::Stats => {
                    let default_path = crate::memory::VectorStore::default_path();
                    let store = crate::memory::VectorStore::load_or_default();
                    let stats = store.stats(Some(&default_path));

                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📊  Tagisan Persistent Memory Statistics".bold().magenta());
                    println!("{}", "=========================================================".cyan());
                    println!("Storage File:         {}", stats.file_path.unwrap_or_else(|| "none".to_string()).yellow());
                    println!("Total Documents:      {}", stats.total_documents.to_string().cyan().bold());
                    println!("Embedding Dimension:  {}", stats.embedding_dimensions.to_string().green());
                    println!("File Size on Disk:    {} bytes", stats.storage_bytes.to_string().yellow());
                }
                MemoryAction::Clear => {
                    let default_path = crate::memory::VectorStore::default_path();
                    if default_path.exists() {
                        let _ = std::fs::remove_file(&default_path);
                    }
                    println!("{} Cleared Tagisan persistent memory file: {}", "✔".green().bold(), default_path.display());
                }
            }
        }

        Commands::Swarm { action } => {
            let ctx = build_engine_context(cli.max_budget);
            match action {
                SwarmAction::Run { prompt, agents, lead } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  🐝  Tagisan Swarm: Lead Agent Orchestration".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let agent_names: Vec<&str> = agents.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    let mut coordinator = SwarmCoordinator::new();

                    for name in &agent_names {
                        let preset = crate::ecc::find_preset(name);
                        let role = preset.as_ref().map(|p| p.description.as_str()).unwrap_or("General Engineering Specialist");
                        let sys_prompt = preset.as_ref().map(|p| p.system_prompt.clone());
                        let (_, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;
                        let member = SwarmMember::new(*name, role, model_name, prov)
                            .with_tools(ToolRegistry::with_builtins());
                        let member = if let Some(sys) = sys_prompt {
                            member.with_system_prompt(sys)
                        } else {
                            member
                        };
                        coordinator.register_member(member);
                    }
                    coordinator.set_lead(&lead);

                    println!("Lead Agent:  {}", lead.bold().green());
                    println!("Specialists: {}", agents.cyan());
                    println!("Task:        {}\n", prompt.italic());

                    let start = std::time::Instant::now();
                    let res = coordinator.run_lead(&prompt, &ctx).await?;

                    println!("\n{}", "=========================================================".cyan());
                    println!("{}", "  🏁  Swarm Deliverable Summary".bold().green());
                    println!("{}", "=========================================================".cyan());
                    println!("Completed in {} iteration(s) ({:.2}s)", res.iterations, start.elapsed().as_secs_f32());
                    println!("Total Tokens: {} | Total Cost: ${:.4} USD",
                        res.total_usage.prompt_tokens + res.total_usage.completion_tokens,
                        res.total_cost_usd
                    );
                    println!("\n{}\n", res.final_answer);
                }
                SwarmAction::Pipeline { prompt, stages } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  ⛓️   Tagisan Swarm: Sequential Pipeline Execution".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let stage_names: Vec<&str> = stages.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    let mut coordinator = SwarmCoordinator::new();

                    for name in &stage_names {
                        let preset = crate::ecc::find_preset(name);
                        let role = preset.as_ref().map(|p| p.description.as_str()).unwrap_or("Specialist");
                        let sys_prompt = preset.as_ref().map(|p| p.system_prompt.clone());
                        let (_, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;
                        let member = SwarmMember::new(*name, role, model_name, prov)
                            .with_tools(ToolRegistry::with_builtins());
                        let member = if let Some(sys) = sys_prompt {
                            member.with_system_prompt(sys)
                        } else {
                            member
                        };
                        coordinator.register_member(member);
                    }

                    println!("Stages: {}\nTask:   {}\n", stages.cyan().bold(), prompt.italic());
                    let res = coordinator.execute_pipeline(&stage_names, &prompt, &ctx).await?;

                    for stage in &res.stages {
                        println!("{}", "---------------------------------------------------------".dimmed());
                        println!("Stage {}: {} ({})", stage.stage_index + 1, stage.agent_name.bold().green(), stage.role.cyan());
                        let preview: String = stage.result.final_answer.lines().take(5).collect::<Vec<_>>().join("\n");
                        println!("{}\n", preview);
                    }

                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  🏁  Pipeline Final Output".bold().green());
                    println!("{}", "=========================================================".cyan());
                    println!("{}\n", res.final_answer);
                }
                SwarmAction::Broadcast { prompt, agents } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📡  Tagisan Swarm: Concurrent Multi-Agent Broadcast".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let agent_names: Vec<&str> = agents.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    let mut coordinator = SwarmCoordinator::new();

                    for name in &agent_names {
                        let preset = crate::ecc::find_preset(name);
                        let role = preset.as_ref().map(|p| p.description.as_str()).unwrap_or("Specialist");
                        let sys_prompt = preset.as_ref().map(|p| p.system_prompt.clone());
                        let (_, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;
                        let member = SwarmMember::new(*name, role, model_name, prov)
                            .with_tools(ToolRegistry::with_builtins());
                        let member = if let Some(sys) = sys_prompt {
                            member.with_system_prompt(sys)
                        } else {
                            member
                        };
                        coordinator.register_member(member);
                    }

                    println!("Broadcast Members: {}", agents.cyan().bold());
                    println!("Prompt:            {}\n", prompt.italic());

                    let results = coordinator.execute_broadcast(&prompt, &ctx).await?;
                    for (name, res) in results {
                        println!("{}", "---------------------------------------------------------".dimmed());
                        println!("Agent: {}", name.bold().green());
                        println!("Tokens: {} | Latency: {:.2}s",
                            res.total_usage.prompt_tokens + res.total_usage.completion_tokens,
                            res.total_latency.as_secs_f32()
                        );
                        println!("{}\n", res.final_answer);
                    }
                }
            }
        }

        Commands::Consensus { rule, artifact } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚖️   Tagisan Team Consensus & Peer Review Engine".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let ctx = build_engine_context(cli.max_budget);
            let voting_rule = match rule.to_lowercase().as_str() {
                "unanimous" => VotingRule::Unanimous,
                "supermajority" | "super" => VotingRule::SuperMajority(0.66),
                "borda" | "weighted_borda" => VotingRule::WeightedBorda,
                _ => VotingRule::Majority,
            };

            let artifact_content = if std::path::Path::new(&artifact).exists() {
                println!("Loading artifact from file: {}", artifact.cyan().bold());
                std::fs::read_to_string(&artifact)?
            } else {
                artifact.clone()
            };

            println!("Voting Rule:  {:?}", voting_rule);
            println!("Artifact:     {} byte(s)\n", artifact_content.len());

            let mut coordinator = SwarmCoordinator::new();
            for name in &["architect", "security-auditor", "code-reviewer"] {
                let preset = crate::ecc::find_preset(name);
                let role = preset.as_ref().map(|p| p.description.as_str()).unwrap_or("Reviewer");
                let sys_prompt = preset.as_ref().map(|p| p.system_prompt.clone());
                let (_, model_name, prov) = resolve_provider_and_model(&ctx, "auto", None)?;
                let mut member = SwarmMember::new(*name, role, model_name, prov);
                if let Some(sys) = sys_prompt {
                    member = member.with_system_prompt(sys);
                }
                coordinator.register_member(member);
            }

            let engine = TeamConsensusEngine::new();
            let verdict = engine.run_consensus_review(&coordinator, &artifact_content, voting_rule, &ctx).await?;

            println!("{}", "=========================================================".cyan());
            println!("{}", "  🗳️   Consensus Evaluation Results".bold().magenta());
            println!("{}", "=========================================================".cyan());

            let status_badge = if verdict.approved {
                "APPROVED".bold().green()
            } else {
                "REJECTED / REVISION REQUIRED".bold().red()
            };
            println!("Final Verdict:        {}", status_badge);
            println!("Approval Ratio:       {:.1}% ({}/{} reviewers)", verdict.approval_ratio * 100.0,
                verdict.reviews.iter().filter(|r| r.approved).count(), verdict.voters_count);
            println!("Average Quality:      {:.2} / 10", verdict.average_score);

            if !verdict.criterion_averages.is_empty() {
                println!("\nCriterion Breakdown:");
                for (crit, avg) in &verdict.criterion_averages {
                    println!("  - {:<18}: {:.2} / 10", crit.bold().cyan(), avg);
                }
            }

            if let Some(ref win) = verdict.winning_option {
                println!("\nPreferred Option (Borda): {}", win.bold().yellow());
            }

            if !verdict.action_items.is_empty() {
                println!("\nAction Items & Risks ({}):", verdict.action_items.len());
                for item in &verdict.action_items {
                    println!("  ⚠️  {}", item.yellow());
                }
            }

            println!("\nSynthesis Summary:\n{}\n", verdict.synthesis);
        }

        Commands::Repl { agent, model, provider, memory, sandbox, resume } => {
            let ctx = build_engine_context(cli.max_budget);
            let store = SessionStore::new();

            let mut repl = if let Some(ref session_id) = resume {
                let session_record = store.load(session_id)?;
                let (_, model_name, prov) = resolve_provider_and_model(&ctx, &provider, Some(session_record.model.clone()))?;
                let mut autonomous_agent = AutonomousAgent::new(prov, model_name, ToolRegistry::with_builtins());
                if let Some(ref sys) = session_record.system_prompt {
                    autonomous_agent = autonomous_agent.with_system_prompt(sys.clone());
                }
                InteractiveRepl::new(autonomous_agent, session_id.clone(), session_record.model.clone(), ctx.clone())
            } else {
                let session_id = format!("repl-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
                let (_, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;
                let mut autonomous_agent = AutonomousAgent::new(prov, model_name.clone(), ToolRegistry::with_builtins());

                if let Some(ref persona) = agent {
                    if let Some(preset) = crate::ecc::find_preset(persona) {
                        autonomous_agent = autonomous_agent.with_system_prompt(preset.system_prompt.clone());
                    }
                }

                if memory {
                    let vec_store = std::sync::Arc::new(crate::memory::VectorStore::load_or_default());
                    let emb_prov = crate::memory::default_embedding_provider();
                    autonomous_agent = autonomous_agent.with_memory(vec_store, emb_prov);
                }

                let mut rep = InteractiveRepl::new(autonomous_agent, session_id, model_name, ctx.clone());
                if let Some(ref persona) = agent {
                    rep.session_record.agent_persona = Some(persona.clone());
                }
                rep
            };

            if sandbox {
                let sb = WorktreeSandbox::new(".")?;
                repl = repl.with_sandbox(sb);
            }

            repl.start().await?;
        }

        Commands::Session { action } => {
            let store = SessionStore::new();
            match action {
                SessionAction::List => {
                    let list = store.list()?;
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📁  Tagisan Saved Agent Sessions".bold().magenta());
                    println!("{}", "=========================================================".cyan());

                    if list.is_empty() {
                        println!("No saved sessions found in {:?}.", store.base_dir());
                        return Ok(());
                    }

                    println!("{:<24} {:<20} {:<10} {:<10} {:<24}",
                        "Session ID".bold().cyan(),
                        "Model".bold().yellow(),
                        "Messages".bold().green(),
                        "Cost (USD)".bold().green(),
                        "Last Updated".bold().white()
                    );
                    println!("{}", "-".repeat(90).dimmed());

                    for s in list {
                        println!("{:<24} {:<20} {:<10} ${:<9.4} {:<24}",
                            s.id.cyan(),
                            s.model.yellow(),
                            s.message_count,
                            s.total_cost_usd,
                            s.updated_at.dimmed()
                        );
                    }
                }
                SessionAction::Resume { id } => {
                    let ctx = build_engine_context(cli.max_budget);
                    let session_record = store.load(&id)?;
                    let (_, model_name, prov) = resolve_provider_and_model(&ctx, "auto", Some(session_record.model.clone()))?;
                    let mut autonomous_agent = AutonomousAgent::new(prov, model_name.clone(), ToolRegistry::with_builtins());
                    if let Some(ref sys) = session_record.system_prompt {
                        autonomous_agent = autonomous_agent.with_system_prompt(sys.clone());
                    }
                    let mut repl = InteractiveRepl::new(autonomous_agent, id.clone(), model_name, ctx);
                    repl.session_record = session_record;
                    repl.start().await?;
                }
                SessionAction::Export { id, output } => {
                    let md = store.export_markdown(&id)?;
                    if let Some(out_path) = output {
                        std::fs::write(&out_path, &md)?;
                        println!("{} Exported session '{}' to Markdown at: {}", "✔".green().bold(), id.cyan(), out_path.yellow().bold());
                    } else {
                        println!("{md}");
                    }
                }
                SessionAction::Delete { id } => {
                    if store.delete(&id)? {
                        println!("{} Deleted session '{}' successfully.", "✔".green().bold(), id.cyan());
                    } else {
                        println!("{}: Session '{}' was not found.", "Warning".yellow().bold(), id);
                    }
                }
            }
        }
    }

    Ok(())
}
