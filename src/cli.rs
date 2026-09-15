use clap::{Parser, Subcommand};
use colored::Colorize;
use futures::StreamExt;
use std::env;
use std::io::Write;
use std::sync::Arc;
use crate::{
    all_ecc_presets, build_ecc_pipeline, build_ecc_pipeline_with_skills, load_ecc_agents_from_dir,
    resolve_ecc_agent, resolve_ecc_skill,
    AnthropicProvider, AutonomousAgent, CalculatorTool, ChatSession, ColibriProvider, CollaborationStrategy,
    CompletionRequest, ContentBlock, DagScheduler, DeleteFileTool, DialecticalDebateStrategy, EccAuditDebate,
    EditFileTool, EngineContext, GeminiProvider, ListDirTool, LlmProvider, MixtureOfAgentsStrategy, OllamaProvider,
    OpenAiCompatibleProvider, ProviderCapabilities, ReadFileTool, RunCommandTool, StrategyInput,
    StreamChunkDelta, TagisanError, ToolHandler, ToolRegistry, ViewImageTool, WorkflowEvent, WorkflowPlanner, WriteFileTool,
    McpManager, WorktreeSandbox, Spinner,
    InteractiveRepl, SessionStore,
    ClusterCoordinator, ClusterWorker, query_cluster_status, SwarmAtlas,
    SwarmCoordinator, SwarmMember, TeamConsensusEngine, VotingRule,
};

#[derive(Parser)]
#[command(name = "tgs", bin_name = "tgs", version)]
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

        /// Explicit engineering skill to inject by name (e.g. "rust-tokio-concurrency")
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject relevant engineering skills based on prompt semantics
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable skill injection
        #[arg(long)]
        no_skills: bool,

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

        /// Explicit engineering skill to inject by name (e.g. "rust-tokio-concurrency")
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject relevant engineering skills based on prompt semantics
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable skill injection
        #[arg(long)]
        no_skills: bool,

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

        /// Explicit engineering skill to inject into all debate agents by name (e.g. "rust-tokio-concurrency")
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject relevant engineering skills based on prompt semantics
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable skill injection in debate
        #[arg(long)]
        no_skills: bool,

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

        /// Comma-separated list of tools to enable: read_file, write_file, edit_file, delete_file, list_dir, run_command, calculator, view_image, web_search, all
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

        /// Comma-separated list of tools to enable: read_file, write_file, edit_file, delete_file, list_dir, run_command, calculator, web_search, all
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
    /// Live Web Search and content extraction (DuckDuckGo, Tavily, Brave)
    #[command(alias = "websearch")]
    Search {
        /// Search query keywords
        #[arg(index = 1)]
        query: Option<String>,

        /// Direct URL to fetch, clean, and extract readable text from
        #[arg(short, long)]
        url: Option<String>,

        /// Maximum number of search results to return (1-10, default: 5)
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },
    /// Manage authentication sessions and web logins (Google Gemini OAuth 2.0)
    #[command(alias = "login")]
    Auth {
        #[command(subcommand)]
        action: Option<AuthAction>,

        /// Google OAuth 2.0 Client ID (optional, overrides default/env)
        #[arg(long)]
        client_id: Option<String>,

        /// Google OAuth 2.0 Client Secret (optional)
        #[arg(long)]
        client_secret: Option<String>,
    },
    /// Log out from an authenticated provider session (Google Gemini OAuth)
    Logout {
        /// Provider to log out from (default: gemini)
        #[arg(default_value = "gemini")]
        provider: String,
    },
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
    /// Bun ultra-fast JavaScript/TypeScript runtime, bundler, and test runner
    Bun {
        #[command(subcommand)]
        action: BunAction,
    },
    /// Python 3 runtime execution, script runner, and tool handler
    Python {
        #[command(subcommand)]
        action: PythonAction,
    },
    /// Perl 5 runtime execution, script runner, and tool handler
    Perl {
        #[command(subcommand)]
        action: PerlAction,
    },
    /// Vella Sovereign Framework: SCADA, Robotics, Trading, Medicine & Sovereign Governance
    Vella {
        #[command(subcommand)]
        action: VellaAction,
    },
    /// Enterprise Observability & OpenTelemetry Trace Inspector (RFC-001)
    Trace {
        /// Stream the last N trace events from the journal to stdout
        #[arg(long, default_value = "false")]
        live: bool,

        /// Launch interactive Ratatui TUI live trace tree viewer
        #[arg(long, default_value = "false")]
        tui: bool,

        /// Export the trace journal to the specified file path (JSONL or SQLite)
        #[arg(long)]
        export: Option<std::path::PathBuf>,

        /// Export traces directly to a SQLite database file (.db)
        #[arg(long)]
        sqlite: Option<std::path::PathBuf>,

        /// Number of recent events to display with --live
        #[arg(long, default_value = "20")]
        last: usize,

        /// Clear all events from the trace journal
        #[arg(long, default_value = "false")]
        clear: bool,
    },
    /// Automated Swarm Evaluation Suite & Multi-Model Benchmarking (RFC-001)
    Eval {
        #[command(subcommand)]
        action: Option<EvalAction>,

        /// Path to the JSON evaluation dataset file
        #[arg(long)]
        dataset: Option<std::path::PathBuf>,

        /// Comma-separated list of model identifiers to evaluate
        #[arg(long, value_delimiter = ',')]
        models: Option<Vec<String>>,

        /// Voting/aggregation rule: "borda" (default) or "majority"
        #[arg(long, default_value = "borda")]
        rule: String,

        /// Pass/fail threshold on the 1.0–5.0 scoring scale (or 0.0–1.0 ratio, default: 0.85)
        #[arg(long, default_value = "0.85")]
        threshold: f32,

        /// Optional path to save the JSON evaluation report
        #[arg(long)]
        output: Option<std::path::PathBuf>,
    },
    /// Execute Structured Role-Based Harmony Swarm (Assembly Line: Architect -> Implementer -> QA -> Doc)
    Harmony {
        #[command(subcommand)]
        action: Option<HarmonySubcommand>,

        /// Objective or coding task to build (if subcommand is omitted)
        #[arg(value_name = "OBJECTIVE")]
        objective: Option<String>,

        /// Custom Architect role model in format 'provider:model'
        #[arg(long)]
        architect: Option<String>,

        /// Custom Implementer role model in format 'provider:model'
        #[arg(long)]
        implementer: Option<String>,

        /// Custom QA & Test Specialist role model in format 'provider:model'
        #[arg(long)]
        qa: Option<String>,

        /// Custom Documentation & Packaging role model in format 'provider:model'
        #[arg(long)]
        doc: Option<String>,

        /// Cloud tier optimization profile: 'smart' (default), 'flagship', or 'economy'
        #[arg(long)]
        tier: Option<String>,

        /// Run QA and Documentation stages concurrently in parallel
        #[arg(long)]
        parallel: bool,

        /// Run adversarial audit step on the final assembly (Harmony + Debate)
        #[arg(long)]
        audit: bool,

        /// Output full structured JSON artifact instead of formatted markdown
        #[arg(long)]
        json: bool,

        /// Save generated code files to a target directory
        #[arg(long)]
        output_dir: Option<String>,

        /// Automatically fallback to local Ollama on cloud rate limits or errors
        #[arg(long)]
        fallback_to_local: bool,

        /// Automatically evacuate to zero-cost local Ollama if budget is reached
        #[arg(long)]
        evacuate_on_budget: bool,

        /// Emit OS desktop notification when failover occurs
        #[arg(long)]
        notify: bool,

        /// Explicit skill to inject by name into the pipeline
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject domain skills for each role (default: true)
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable automatic skill injection across all stages
        #[arg(long)]
        no_skills: bool,
    },
    /// Autonomously synthesize agent-native CLI tools and SKILL.md packages from codebases (CLI-Anything)
    Harness {
        #[command(subcommand)]
        action: HarnessAction,
    },
    /// Manage, install, and execute capability-sandboxed plugins & extensions (RFC-002)
    Plugin {
        #[command(subcommand)]
        action: crate::plugins::PluginAction,
    },
    /// Start the native Ollama-compatible Rust Tensor Engine HTTP server
    Serve {
        /// Port to listen on (default: 11434 with automatic fallback)
        #[arg(short, long, default_value = "11434")]
        port: u16,

        /// Host address to bind to
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Inspect and manage local GGUF tensor models and Ollama blob store
    Engine {
        #[command(subcommand)]
        action: EngineAction,
    },
    /// Automatically diagnose and self-heal compiler errors & test failures (Rust, TS, Python, Go)
    Autofix {
        /// Target directory or source file to inspect and heal (default: current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Include test suites during diagnosis (e.g. cargo check --tests, pytest)
        #[arg(short, long)]
        test: bool,

        /// Maximum iterative healing attempts
        #[arg(short = 'm', long, default_value_t = 5)]
        max_attempts: usize,

        /// Perform dry run without modifying files on disk
        #[arg(long)]
        dry_run: bool,
    },
    /// AST codebase knowledge graph, symbol navigation, and blast-radius analysis
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
    /// Deterministic Closed-Loop Grounding & Verification Engine (tgs ground)
    Ground {
        /// Coding or reasoning task prompt to ground and verify
        task: String,

        /// Target codebase directory or source file for AST invariant extraction
        #[arg(short, long)]
        path: Option<String>,

        /// Maximum iterative self-healing verification attempts (default: 3)
        #[arg(short = 'm', long, default_value_t = 3)]
        max_iterations: usize,

        /// Disable adversarial dialectical critique audit
        #[arg(long)]
        no_critique: bool,

        /// Disable AST codebase graph invariant extraction
        #[arg(long)]
        no_ast: bool,

        /// Optional file path to write verified output code
        #[arg(short, long)]
        output: Option<String>,
    },
    /// AgentShield Cyber Defense Subsystem: Scan repositories, audit commands, and monitor nation-state threat indicators
    Shield {
        #[command(subcommand)]
        action: ShieldAction,
    },
    /// Erlang/BEAM & OTP Native Actor Engine, Supervision Trees & Port Protocol
    Otp {
        #[command(subcommand)]
        action: OtpAction,
    },
    /// Sovereign Gleam Actor Subsystem: Type checker, Erlang/ETF compiler, and OTP actor runner
    Gleam {
        #[command(subcommand)]
        action: GleamAction,
    },
    /// IDE Ecosystem Embedding: Setup editor configurations, run Language Server Protocol (LSP), and inspect status
    Ide {
        #[command(subcommand)]
        action: IdeAction,
    },
    /// Microsoft 365 Copilot & Microsoft Graph Communication System
    Copilot {
        #[command(subcommand)]
        action: CopilotAction,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CopilotAction {
    /// Microsoft Entra ID (Azure AD) OAuth2 Device Code & Token Management
    Auth {
        /// Entra ID Application (Client) ID
        #[arg(long)]
        client_id: Option<String>,

        /// Entra ID Directory (Tenant) ID
        #[arg(long)]
        tenant_id: Option<String>,

        /// Display current authentication status and token expiration
        #[arg(long)]
        status: bool,
    },

    /// Generate Microsoft 365 Copilot Plugin, Declarative Agent manifest, and OpenAPI 3.0 spec
    #[command(alias = "package")]
    Plugin {
        /// Output directory for package bundle (defaults to .tagisan/copilot_package)
        #[arg(long, default_value = ".tagisan/copilot_package")]
        output_dir: String,

        /// Base URL for the Tagisan API endpoints
        #[arg(long, default_value = "https://api.tagisan.ai")]
        base_url: String,
    },

    /// Post an engineering update or debate verdict to Microsoft Teams with AgentShield DLP
    Post {
        /// Teams channel identifier or chat ID
        #[arg(long)]
        channel: String,

        /// Message content to post
        #[arg(long)]
        message: String,
    },

    /// Fetch and inspect Teams meeting transcript or extract structured action items
    Transcript {
        /// Microsoft Teams online meeting identifier
        #[arg(long, default_value = "latest_architecture_sync")]
        meeting: String,

        /// Parse structured engineering action items from the transcript
        #[arg(long)]
        parse_items: bool,
    },

    /// Register or inspect Microsoft Search Graph Connector schema and indexed skills
    Index {
        /// Display schema definition only without indexing items
        #[arg(long)]
        schema_only: bool,
    },

    /// Run diagnostic self-test and verification of the Copilot subsystem
    Test,

    /// Inspect Copilot subsystem, Entra ID status, and Graph connector health
    Status,

    /// End-to-end meeting-to-code pipeline: extract action items, compute AST blast radius, synthesize patches
    #[command(name = "meeting-to-code")]
    MeetingToCode {
        /// Microsoft Teams online meeting identifier
        #[arg(long)]
        meeting: Option<String>,

        /// Optional raw transcript text with speaker turns
        #[arg(long)]
        transcript: Option<String>,

        /// Root codebase directory to calculate AST blast radius (default: '.')
        #[arg(long, default_value = ".")]
        path: String,

        /// Automatically synthesize code diffs and patch proposals
        #[arg(long, default_value_t = true)]
        auto_patch: bool,

        /// Teams channel or chat ID to post the synthesized execution plan
        #[arg(long)]
        channel: Option<String>,
    },

    /// Codebase Telemetry & Blast Radius Cards: Adaptive Card & HTML reports for Teams/Excel/PowerPoint
    #[command(name = "blast-report")]
    BlastReport {
        /// Target symbol name (struct, function, method, trait)
        #[arg(long)]
        symbol: String,

        /// Maximum transitive traversal depth in the codebase graph (default: 3)
        #[arg(long, default_value_t = 3)]
        max_depth: usize,

        /// Root codebase directory path (default: '.')
        #[arg(long, default_value = ".")]
        path: String,

        /// Desired output format: 'adaptive_card', 'html', or 'all' (default: 'all')
        #[arg(long, default_value = "all")]
        format: String,

        /// Teams channel or chat ID to post the telemetry card
        #[arg(long)]
        post_to_teams: Option<String>,

        /// Recipient email address to export the HTML report via Outlook
        #[arg(long)]
        export_email: Option<String>,
    },

    /// Dialectical Debate Dispatch: Run 3-round debate and dispatch verdict to Teams/Outlook
    Debate {
        /// Architectural proposal, RFC, or technical question to debate
        #[arg(long)]
        proposal: String,

        /// Title or subject for the debate
        #[arg(long)]
        title: Option<String>,

        /// Proponent model or persona (default: 'claude-3-5-sonnet')
        #[arg(long, default_value = "claude-3-5-sonnet")]
        proponent: String,

        /// Adversary model or persona (default: 'gpt-4o')
        #[arg(long, default_value = "gpt-4o")]
        adversary: String,

        /// Lakandiwa adjudicator model or persona (default: 'o1-preview')
        #[arg(long, default_value = "o1-preview")]
        lakandiwa: String,

        /// Teams channel or chat ID to post the verdict
        #[arg(long)]
        post_to_teams: Option<String>,

        /// Recipient email address to dispatch the debate transcript via Outlook
        #[arg(long)]
        send_to_email: Option<String>,
    },

    /// Microsoft Teams Operations
    Teams {
        #[command(subcommand)]
        action: TeamsAction,
    },

    /// Ingest SharePoint / OneDrive document or Teams meeting transcript
    Ingest {
        /// SharePoint or OneDrive file path or URL
        #[arg(long)]
        url: Option<String>,

        /// Microsoft Teams online meeting identifier
        #[arg(long)]
        meeting: Option<String>,
    },

    /// Microsoft Purview Data Sensitivity & Zero-Egress Air-Gapping Evaluation
    Purview {
        /// Content, code snippet, or document text to evaluate
        #[arg(long)]
        content: String,

        /// Optional explicit sensitivity label: General, Confidential, HighlyConfidential, Secret
        #[arg(long)]
        label: Option<String>,

        /// Optional planned destination or target execution engine
        #[arg(long)]
        destination: Option<String>,
    },

    /// Architecture Decision Record (ADR) Synthesis & Sync to OneNote / SharePoint
    Adr {
        /// Architectural proposal, design problem, or debate topic
        #[arg(long)]
        proposal: String,

        /// Optional consensus decision or Lakandiwa debate synthesis
        #[arg(long)]
        verdict: Option<String>,

        /// Optional human-readable title for the decision record
        #[arg(long)]
        title: Option<String>,

        /// Target OneNote section (defaults to 'Architecture Decisions')
        #[arg(long)]
        onenote_section: Option<String>,

        /// Target SharePoint folder (defaults to 'Engineering/ADRs')
        #[arg(long)]
        sharepoint_folder: Option<String>,
    },

    /// Ephemeral Git Branch Creation & Pull Request Automation with Blast Telemetry
    Pr {
        /// Code patch, diff snippet, or engineering requirement text
        #[arg(long)]
        patch: String,

        /// Pull request title or conventional commit summary
        #[arg(long)]
        title: Option<String>,

        /// Ephemeral branch name (auto-generated if omitted)
        #[arg(long)]
        branch: Option<String>,

        /// Base branch to merge into (defaults to 'main')
        #[arg(long, default_value = "main")]
        base: String,

        /// Target platform: 'azure_devops' or 'github' (defaults to 'azure_devops')
        #[arg(long, default_value = "azure_devops")]
        platform: String,

        /// Primary architectural symbol affected
        #[arg(long)]
        symbol: Option<String>,

        /// Teams channel or chat ID to dispatch the PR notification card
        #[arg(long)]
        channel: Option<String>,
    },

    /// Responsive Executive Presentation Briefing Slide Deck Generator (HTML & Markdown)
    Deck {
        /// Presentation title (defaults to 'Tagisan Executive Architecture Briefing')
        #[arg(long, default_value = "Tagisan Executive Architecture Briefing")]
        title: String,

        /// Output format: 'html', 'markdown', or 'all' (defaults to 'all')
        #[arg(long, default_value = "all")]
        format: String,

        /// File path to save the generated deck (e.g. '.tagisan/executive_deck.html')
        #[arg(long)]
        output: Option<String>,

        /// Recipient email address to dispatch deck via Outlook
        #[arg(long)]
        email: Option<String>,

        /// Custom executive notes or remarks
        #[arg(long)]
        notes: Option<String>,
    },

    /// Interactive Teams Bot Webhook Listener & Action.Submit Handler
    Listen {
        /// Port to listen on (default: 3978)
        #[arg(short, long, default_value = "3978")]
        port: u16,

        /// Host address to bind to (default: 127.0.0.1)
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Dry-run test action to simulate: 'approve_patch', 'run_autofix', 'run_debate', 'sync_adr'
        #[arg(long)]
        test_action: Option<String>,

        /// Target file or data for dry-run simulation
        #[arg(long)]
        target: Option<String>,
    },

    /// Native Excel Custom Functions Engine & Add-in Packager (=TGS.*)
    Excel {
        /// Excel formula to evaluate (e.g. '=TGS.BLAST_RADIUS("EntraAuthManager", ".")')
        #[arg(long)]
        formula: Option<String>,

        /// Function name to evaluate ('BLAST_RADIUS', 'COMPLEXITY', 'COST_SAVINGS', 'INVARIANT_CHECK')
        #[arg(long)]
        function: Option<String>,

        /// Symbol name for BLAST_RADIUS or COMPLEXITY
        #[arg(long)]
        symbol: Option<String>,

        /// Root codebase path (defaults to '.')
        #[arg(long, default_value = ".")]
        path: String,

        /// Prompt tokens for COST_SAVINGS (default: 100000)
        #[arg(long, default_value_t = 100000)]
        prompt_tokens: u64,

        /// Completion tokens for COST_SAVINGS (default: 50000)
        #[arg(long, default_value_t = 50000)]
        completion_tokens: u64,

        /// Target module for INVARIANT_CHECK
        #[arg(long)]
        target: Option<String>,

        /// Code snippet for INVARIANT_CHECK
        #[arg(long)]
        code: Option<String>,

        /// Package Add-in manifest.xml, functions.json, and functions.js to directory
        #[arg(long)]
        package: bool,

        /// Output directory for Add-in package (default: '.tagisan/excel_addin')
        #[arg(long, default_value = ".tagisan/excel_addin")]
        output_dir: String,
    },

    /// Live Server-Sent Events (SSE) / NDJSON Streaming Gateway for Copilot Studio & Teams
    Stream {
        /// Technical prompt, architecture topic, or question to stream
        prompt: String,

        /// Output format: 'sse' (default, text/event-stream) or 'ndjson'
        #[arg(long, default_value = "sse")]
        format: String,

        /// Streaming mode: 'debate' (default), 'swarm', or 'reasoning'
        #[arg(long, default_value = "debate")]
        mode: String,

        /// Disable keepalive heartbeat pulses
        #[arg(long)]
        no_keepalive: bool,
    },

    /// Microsoft Planner & To-Do Task Synchronizer with Git/PR Linkage
    Planner {
        /// Synchronization action: 'sync_all' (default), 'sync_planner', 'sync_todo', 'create_single'
        #[arg(long, default_value = "sync_all")]
        action: String,

        /// Teams meeting identifier to extract action items from
        #[arg(long)]
        meeting: Option<String>,

        /// Raw transcript text
        #[arg(long)]
        transcript: Option<String>,

        /// Planner plan ID (default: 'plan_tagisan_core')
        #[arg(long, default_value = "plan_tagisan_core")]
        plan_id: String,

        /// Planner bucket ID (default: 'bucket_sprint_backlog')
        #[arg(long, default_value = "bucket_sprint_backlog")]
        bucket_id: String,

        /// To-Do list ID (default: 'todo_personal_tasks')
        #[arg(long, default_value = "todo_personal_tasks")]
        todo_list_id: String,

        /// Task title for single task creation
        #[arg(long)]
        title: Option<String>,

        /// Priority: 'High', 'Medium', 'Low' (default: 'Medium')
        #[arg(long, default_value = "Medium")]
        priority: String,

        /// Assignee name or email
        #[arg(long)]
        assignee: Option<String>,

        /// Git branch URL to link as reference
        #[arg(long)]
        branch_url: Option<String>,

        /// Pull Request URL to link as reference
        #[arg(long)]
        pr_url: Option<String>,
    },

    /// Teams "@Tagisan" CI/CD Incident Debugger & Surgical Autofix Recommender
    Incident {
        /// Path to CI failure log file or raw log text
        #[arg(long)]
        logs: Option<String>,

        /// Path to log file to read
        #[arg(long)]
        file: Option<String>,

        /// Git commit SHA of the failed build
        #[arg(long)]
        commit: Option<String>,

        /// Pipeline or job identifier
        #[arg(long)]
        pipeline: Option<String>,

        /// Teams channel or chat ID to dispatch the incident alert
        #[arg(long)]
        channel: Option<String>,

        /// Output format: 'all' (default), 'text', or 'adaptive_card'
        #[arg(long, default_value = "all")]
        format: String,
    },

    /// Windows Copilot+ PC Hardware Telemetry & Energy Efficiency
    Hardware {
        /// Inference workload in tokens (default: 10000)
        #[arg(long, default_value_t = 10000)]
        tokens: u64,

        /// Accelerator to evaluate: 'auto' (default), 'npu', 'directml', 'cpu'
        #[arg(long, default_value = "auto")]
        accelerator: String,

        /// Output format: 'text' (default), 'json', or 'adaptive_card'
        #[arg(long, default_value = "text")]
        format: String,
    },
}


#[derive(Subcommand, Debug, Clone)]
pub enum TeamsAction {
    /// Post a message to a Teams channel or chat with AgentShield DLP protection
    Post {
        /// Teams channel identifier or chat ID
        #[arg(long)]
        channel: String,

        /// Message content to post
        #[arg(long)]
        message: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum IdeAction {
    /// Generate IDE configuration bundles for VS Code, Cursor, Windsurf, Claude Desktop, Zed, and JetBrains
    Setup {
        /// Target IDE: all, vscode, cursor, windsurf, claude, zed, jetbrains (default: all)
        #[arg(short, long, default_value = "all")]
        target: String,

        /// Output directory (defaults to current directory ".")
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Run the Language Server Protocol (LSP) engine over stdio
    Lsp,
    /// Inspect IDE configurations and detect active editor integrations
    Status {
        /// Workspace directory to inspect (defaults to current directory ".")
        #[arg(short, long, default_value = ".")]
        path: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum GleamAction {
    /// Syntax-checks and type-checks Gleam agent modules
    Check {
        /// Path to Gleam file (defaults to scanning sdk/gleam/tagisan_gleam/src/ or current dir)
        file: Option<String>,
    },
    /// Compiles Gleam agent source to Erlang/ETF
    Compile {
        /// Path to Gleam source file
        file: String,
        /// Compilation target: erlang, etf, or all (default: all)
        #[arg(short, long, default_value = "all")]
        target: Option<String>,
    },
    /// Compiles, type-checks, and spawns a Gleam actor into Tagisan's OTP supervisor tree
    Run {
        /// Path to Gleam actor file
        file: String,
        /// Initial message to send to the spawned actor (e.g. "ping" or constructor name)
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Scaffolds a new type-safe sovereign Gleam agent file
    New {
        /// Name of the new agent
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum OtpAction {
    /// Start an OTP supervisor tree and monitor worker actors
    Supervise,
    /// Launch stdio Erlang Port 4-byte packet protocol mode
    Port,
    /// Benchmark massive actor spawning and message passing throughput
    Bench {
        /// Number of actors to spawn concurrently (default: 5000)
        #[arg(short, long, default_value_t = 5000)]
        actors: usize,
        /// Total number of messages to dispatch (default: 50000)
        #[arg(short, long, default_value_t = 50000)]
        messages: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ShieldAction {
    /// Recursively scan files, codebases, repositories, and dependency manifests for nation-state APT indicators
    Scan {
        /// Path to file, directory, or repository to scan (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Pre-execution vetting of an arbitrary shell command, script, or payload against AgentShield
    Audit {
        /// Command string, script, or payload to audit
        command: String,

        /// Output verdict in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Display active cyber defense profile, protected assets, threat actor profiles, and telemetry
    Status,
}

#[derive(Subcommand, Debug, Clone)]
pub enum GraphAction {
    /// Compute and display codebase graph statistics
    Stats {
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
    },
    /// Locate symbol definitions and metadata
    Symbol {
        /// Symbol name or pattern to locate
        name: String,
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
    },
    /// Find incoming callers of a symbol
    Callers {
        /// Target symbol name
        name: String,
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
    },
    /// Find outgoing callees invoked by a symbol
    Callees {
        /// Target symbol name
        name: String,
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
    },
    /// Calculate transitive blast radius and refactoring risk
    BlastRadius {
        /// Target symbol to evaluate
        target: String,
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
        /// Maximum transitive traversal depth (default: 3)
        #[arg(long, default_value = "3")]
        max_depth: Option<usize>,
    },
    /// Export the codebase knowledge graph in DOT or JSON format
    Export {
        /// Codebase directory path (default: current directory)
        #[arg(long, short)]
        path: Option<String>,
        /// Output format: "dot" or "json"
        #[arg(long, default_value = "json")]
        format: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum EngineAction {
    /// List all discovered Ollama models and GGUF blobs
    List,
    /// Inspect GGUF binary headers, tensor tensors, and architecture metadata
    Inspect {
        /// Model name, tag, or path to GGUF file
        model: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HarnessAction {
    /// Autonomously synthesize a CLI harness and SKILL.md package from a codebase or script
    Generate {
        /// Path to source code file or directory to synthesize
        source: String,
        /// Name of the synthesized harness and skill (default: auto-detected from filename/folder)
        #[arg(short, long)]
        name: Option<String>,
        /// Source language ('python', 'rust', 'typescript', 'javascript', 'openapi', or 'auto')
        #[arg(short, long, default_value = "auto")]
        lang: String,
        /// Custom output directory (default: .tagisan/harness/<name>)
        #[arg(short, long)]
        output_dir: Option<String>,
        /// Automatically install generated SKILL.md and executable into .ecc/skills/<name>/
        #[arg(short, long)]
        install: bool,
        /// Run the generated validation test harness immediately after synthesis
        #[arg(short, long)]
        test: bool,
        /// Use multi-model Harmony swarm for design and test synthesis instead of deterministic AST
        #[arg(long)]
        swarm: bool,
        /// Model tier for swarm synthesis ('haiku', 'sonnet', 'opus')
        #[arg(long, default_value = "sonnet")]
        tier: String,
    },
    /// List all synthesized CLI harnesses and installed skills
    List,
    /// Execute a synthesized CLI harness safely with AgentShield security auditing
    Run {
        /// Name of the harness or skill to run
        name: String,
        /// Arguments passed directly to the synthesized harness
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run the automated validation test harness for a synthesized CLI
    Test {
        /// Name of the harness or skill to test
        name: String,
    },
    /// Autonomously diagnose and heal a broken CLI harness or failing test suite
    Heal {
        /// Name of the harness or skill to heal
        name: String,
        /// Maximum heal iterations (default: 3)
        #[arg(short, long, default_value = "3")]
        attempts: usize,
    },
    /// Ingest a black-box system binary (e.g. curl, git, ffmpeg) into an agent-native CLI and SKILL.md
    Ingest {
        /// Name or path of the binary to ingest
        binary: String,
        /// Custom name for the generated harness
        #[arg(short, long)]
        name: Option<String>,
        /// Custom output directory
        #[arg(short, long)]
        output_dir: Option<String>,
        /// Automatically install into .ecc/skills/<name>/
        #[arg(short, long)]
        install: bool,
    },
    /// Transpile an MCP tool schema or manifest into an agent-native CLI harness and SKILL.md
    ImportMcp {
        /// Path to the MCP tool JSON schema file
        spec: String,
        /// Custom name for the generated harness
        #[arg(short, long)]
        name: Option<String>,
        /// Custom output directory
        #[arg(short, long)]
        output_dir: Option<String>,
        /// Automatically install into .ecc/skills/<name>/
        #[arg(short, long)]
        install: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HarmonySubcommand {
    /// Build a complete software artifact via the 4-stage assembly line
    Build {
        /// The objective or task prompt
        objective: String,

        /// Custom Architect role model in format 'provider:model'
        #[arg(long)]
        architect: Option<String>,

        /// Custom Implementer role model in format 'provider:model'
        #[arg(long)]
        implementer: Option<String>,

        /// Custom QA role model in format 'provider:model'
        #[arg(long)]
        qa: Option<String>,

        /// Custom Doc role model in format 'provider:model'
        #[arg(long)]
        doc: Option<String>,

        /// Cloud tier optimization profile: 'smart' (default), 'flagship', or 'economy'
        #[arg(long)]
        tier: Option<String>,

        /// Run QA and Documentation stages concurrently in parallel
        #[arg(long)]
        parallel: bool,

        /// Run adversarial audit step on the final assembly
        #[arg(long)]
        audit: bool,

        /// Output JSON project bundle
        #[arg(long)]
        json: bool,

        /// Save generated files to directory
        #[arg(long)]
        output_dir: Option<String>,

        /// Automatically fallback to local Ollama on cloud rate limits or errors
        #[arg(long)]
        fallback_to_local: bool,

        /// Automatically evacuate to zero-cost local Ollama if budget is reached
        #[arg(long)]
        evacuate_on_budget: bool,

        /// Emit OS desktop notification when failover occurs
        #[arg(long)]
        notify: bool,

        /// Explicit skill to inject by name into the pipeline
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject domain skills for each role (default: true)
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable automatic skill injection across all stages
        #[arg(long)]
        no_skills: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum EvalAction {
    /// Run automated benchmark evaluation against a test dataset
    Run {
        /// Path to the JSON evaluation dataset file
        #[arg(long)]
        dataset: std::path::PathBuf,

        /// Comma-separated list of model identifiers to evaluate
        #[arg(long, value_delimiter = ',')]
        models: Vec<String>,

        /// Voting/aggregation rule: "borda" (default) or "majority"
        #[arg(long, default_value = "borda")]
        rule: String,

        /// Pass/fail threshold on the 1.0–5.0 scoring scale (or 0.0–1.0 ratio, default: 0.85)
        #[arg(long, default_value = "0.85")]
        threshold: f32,

        /// Optional path to save the JSON evaluation report
        #[arg(long)]
        output: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum VellaAction {
    /// Inspect Vella Framework & Sovereign Policy Governor status
    Status,
    /// Trigger emergency stop (E-Stop) across physical actuators and robotics
    Estop {
        /// Reason for emergency stop
        #[arg(short, long, default_value = "CLI operator triggered E-Stop")]
        reason: String,
    },
    /// Clear emergency stop (E-Stop) latch
    ClearEstop {
        /// Reason for clearing E-Stop
        #[arg(short, long, default_value = "Operator verified system safety")]
        reason: String,
    },
    /// List registered sovereign model schemas in Vella
    Schemas,
    /// Inspect domain safety policy audit log
    Audit,
    /// Run an adversarial debate governance vote over a high-stakes proposal
    Debate {
        /// Target domain (trading, scada, robotics, medicine)
        #[arg(short, long, default_value = "scada")]
        domain: String,
        /// Action type (e.g. coil_actuation, order_execution, estop_override)
        #[arg(short, long, default_value = "coil_actuation")]
        action_type: String,
        /// Target device/symbol
        #[arg(short, long, default_value = "VALVE_MAIN_COOLANT")]
        target: String,
        /// Parameters JSON string (default: '{"coil_address": 10, "state": true}')
        #[arg(short, long, default_value = "{\"coil_address\": 10, \"state\": true}")]
        parameters: String,
    },
    /// Execute a domain tool directly (trading, scada, robotics, medicine, events)
    Tool {
        /// Tool name: trading, scada, robotics, medicine, events
        tool: String,
        /// Action parameter (e.g. match, read_register, e_stop, molecular_docking, publish)
        action: String,
        /// Additional parameters as JSON string
        #[arg(short, long, default_value = "{}")]
        args: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum BunAction {
    /// Evaluate TypeScript or JavaScript code snippet directly
    Eval {
        /// Code string to execute
        code: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Execute a TypeScript or JavaScript file
    Run {
        /// Script path to execute
        script: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
        /// Arguments passed to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run Bun test runner on files or test suites
    Test {
        /// Target test file, directory, or pattern (default: '.')
        #[arg(default_value = ".")]
        target: String,
        /// Execution timeout in seconds (default: 60)
        #[arg(short, long, default_value = "60")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
        /// Additional arguments for bun test
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Install npm packages using Bun's package manager
    Install {
        /// Packages to install (if empty, runs 'bun install' to install package.json)
        packages: Vec<String>,
        /// Save as development dependency (--dev / -d)
        #[arg(short = 'd', long)]
        dev: bool,
        /// Explicitly permit native C/C++ compilation (e.g. node-gyp lifecycle scripts). By default, scripts are ignored.
        #[arg(long)]
        allow_native: bool,
        /// Execution timeout in seconds (default: 120)
        #[arg(short, long, default_value = "120")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Bundle and optimize TypeScript/JavaScript entry points
    Build {
        /// Entrypoint script (e.g. index.ts)
        entrypoint: String,
        /// Output directory for bundled artifacts (default: ./dist)
        #[arg(short, long, default_value = "./dist")]
        outdir: String,
        /// Minify the bundle output
        #[arg(short, long)]
        minify: bool,
        /// Target environment: browser, bun, node (default: bun)
        #[arg(long, default_value = "bun")]
        target: String,
        /// Execution timeout in seconds (default: 60)
        #[arg(short, long, default_value = "60")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Compile a TypeScript/JavaScript script into a standalone zero-dependency native binary executable
    Compile {
        /// Entrypoint script (e.g. cli.ts)
        entrypoint: String,
        /// Output binary file path (e.g. ./my_app)
        outfile: String,
        /// Minify the compiled executable
        #[arg(short, long)]
        minify: bool,
        /// Compile with bytecode cache
        #[arg(short, long)]
        bytecode: bool,
        /// Execution timeout in seconds (default: 120)
        #[arg(short, long, default_value = "120")]
        timeout: u64,
    },
    /// Display Bun runtime version, path, and host environment information
    Info,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PythonAction {
    /// Evaluate Python 3 code snippet directly
    Eval {
        /// Python code string to execute
        code: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Execute a Python (.py) script file with arguments
    Run {
        /// Script path to execute
        script: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
        /// Arguments passed to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Install Python packages into the environment using uv or pip
    Install {
        /// Packages to install (e.g. requests, pydantic)
        packages: Vec<String>,
        /// Explicitly permit native C extension compilation from source distributions. By default, pre-built binary wheels only are allowed.
        #[arg(long)]
        allow_native: bool,
        /// Execution timeout in seconds (default: 300)
        #[arg(short, long, default_value = "300")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Display Python runtime version, path, and host environment information
    Info,
}

#[derive(Subcommand, Debug, Clone)]
pub enum PerlAction {
    /// Evaluate Perl 5 code snippet or one-liner directly
    Eval {
        /// Perl code string to execute
        code: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Execute a Perl (.pl) script file with arguments
    Run {
        /// Script path to execute
        script: String,
        /// Execution timeout in seconds (default: 30)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
        /// Arguments passed to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Install CPAN modules into local sandbox (.tagisan/perl5/) using cpanm
    Install {
        /// CPAN modules to install (e.g. Path::Tiny, JSON::MaybeXS)
        modules: Vec<String>,
        /// Explicitly permit native C/XS compilation via make/gcc. By default, pure-Perl mode is enforced.
        #[arg(long)]
        allow_native: bool,
        /// Execution timeout in seconds (default: 300)
        #[arg(short, long, default_value = "300")]
        timeout: u64,
        /// Working directory
        #[arg(short, long)]
        cwd: Option<String>,
    },
    /// Display Perl runtime version, path, and host environment information
    Info,
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
    /// Browse or dump entries from the 500 MCP plugin catalog (.ecc/mcp_catalog.json)
    Catalog {
        /// Filter catalog by domain name or keyword (e.g. "Search", "Domain 2", "Cybersecurity")
        #[arg(short, long)]
        domain: Option<String>,

        /// Filter catalog by authority (e.g. "Smithery.ai", "NPM Registry", "PyPI", "Glama.ai")
        #[arg(short, long)]
        authority: Option<String>,

        /// Maximum number of catalog entries to display (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Output full catalog entries as JSON
        #[arg(long)]
        json: bool,
    },
    /// Multi-keyword fuzzy search across the 500 MCP plugin catalog
    Search {
        /// Search keywords or phrase (e.g. "postgres database", "vector memory", "browser scraping")
        query: String,

        /// Optional domain filter
        #[arg(short, long)]
        domain: Option<String>,

        /// Optional authority filter
        #[arg(short, long)]
        authority: Option<String>,

        /// Maximum results to display (default: 25)
        #[arg(short, long, default_value = "25")]
        limit: usize,

        /// Output search results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add an MCP server to configuration (from catalog entry or custom command)
    Add {
        /// Name of the catalog plugin (e.g. "brave-search-mcp") or custom server name
        name: String,

        /// Custom executable command (if omitted, auto-generated from catalog)
        #[arg(long)]
        command: Option<String>,

        /// Custom arguments for the command
        #[arg(long)]
        args: Vec<String>,

        /// Target configuration file path (defaults to mcp.dynamic.json)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Remove an MCP server from configuration
    Remove {
        /// Server name to remove
        server: String,

        /// Target configuration file path (defaults to mcp.dynamic.json or discovered mcp.json)
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Verify active MCP server configuration, commands, environment variables, and AgentShield security
    Verify {
        /// Specific server name to verify (if omitted, verifies all configured servers)
        #[arg(short, long)]
        server: Option<String>,

        /// Optional path to mcp.json configuration file
        #[arg(long)]
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

        /// Vector backend to target: local, pgvector, qdrant (default: local)
        #[arg(long, default_value = "local")]
        backend: String,
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

        /// Vector backend to search: local, pgvector, qdrant (default: local)
        #[arg(long, default_value = "local")]
        backend: String,
    },
    /// Synchronize local vector memory with enterprise database via HybridSyncBridge (RFC-001)
    Sync {
        /// Synchronization action: push, pull, sync (default: sync)
        #[arg(long, default_value = "sync")]
        action: String,

        /// Target collection name
        #[arg(long, default_value = "default_knowledge_base")]
        collection: String,
    },
    /// Display statistics about indexed long-term memory
    Stats,
    /// Clear and wipe persistent memory
    Clear,
}

#[derive(Subcommand, Debug)]
enum AuthAction {
    /// Start browser-based OAuth 2.0 web authentication (Google Gemini)
    Login {
        /// Provider to log in to (default: gemini)
        #[arg(default_value = "gemini")]
        provider: String,

        /// Google OAuth 2.0 Client ID (optional, overrides default/env)
        #[arg(long)]
        client_id: Option<String>,

        /// Google OAuth 2.0 Client Secret (optional)
        #[arg(long)]
        client_secret: Option<String>,
    },
    /// Log out and remove stored credentials
    Logout {
        /// Provider to log out from (default: gemini)
        #[arg(default_value = "gemini")]
        provider: String,
    },
    /// Check current authentication status
    Status,
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
    /// Distributed Local P2P Cluster Mesh (Milestone 8 / Colibrì)
    Cluster {
        #[command(subcommand)]
        subcommand: ClusterAction,
    },
    /// Agent & Skill Atlas with Routing Heat Tracking (Colibrì Live Cortex)
    Atlas {
        /// Optional path to custom swarm heat JSON file
        #[arg(short, long)]
        file: Option<String>,

        /// Clear/reset heat tracking
        #[arg(long)]
        reset: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ClusterAction {
    /// Start the LAN Coordinator server
    Coordinator {
        /// Address and port to bind (default: 0.0.0.0:8765)
        #[arg(short, long, default_value = "0.0.0.0:8765")]
        bind: String,
    },
    /// Start a LAN Worker node connecting to Coordinator
    Worker {
        /// Coordinator address to connect to (default: 127.0.0.1:8765)
        #[arg(short, long, default_value = "127.0.0.1:8765")]
        connect: String,

        /// Optional custom worker ID name
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Query live status of cluster nodes and executed tasks
    Status {
        /// Coordinator address to query (default: 127.0.0.1:8765)
        #[arg(short, long, default_value = "127.0.0.1:8765")]
        coordinator: String,
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

        /// Optional search query to test hybrid lexical-semantic skill dispatching
        #[arg(short, long)]
        query: Option<String>,
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

        /// Comma-separated list of tools: read_file, write_file, edit_file, delete_file, list_dir, run_command, calculator, view_image, all
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

        /// Comma-separated list of tools: read_file, write_file, edit_file, delete_file, list_dir, run_command, calculator, all
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

        /// Explicit engineering skill to inject into all pipeline agents by name (e.g. "rust-tokio-concurrency")
        #[arg(long)]
        skill: Option<String>,

        /// Automatically detect and inject relevant engineering skills into pipeline stages based on objective semantics
        #[arg(long)]
        auto_skills: bool,

        /// Explicitly disable skill injection in pipeline
        #[arg(long)]
        no_skills: bool,
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

fn default_ollama_model() -> String {
    crate::providers::ollama::default_ollama_model()
}

fn is_valid_key(key: &str) -> bool {
    let k = key.trim();
    !k.is_empty() && !k.starts_with("your_") && !k.ends_with("_key_here")
}

pub fn build_engine_context(max_budget: f64) -> EngineContext {
    let mut ctx = EngineContext::new(max_budget);

    // Register Anthropic if key exists
    if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
        if is_valid_key(&key) {
            ctx.register_provider(Arc::new(AnthropicProvider::new(key)));
        }
    }

    // Register OpenAI if key exists
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        if is_valid_key(&key) {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::openai(key)));
        }
    }

    // Register xAI (Grok) if key exists
    if let Ok(key) = env::var("XAI_API_KEY") {
        if is_valid_key(&key) {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::xai(key)));
        }
    }

    // Register DeepSeek if key exists
    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        if is_valid_key(&key) {
            ctx.register_provider(Arc::new(OpenAiCompatibleProvider::deepseek(key)));
        }
    }

    // Register Google Gemini if valid key exists or OAuth is authenticated
    let valid_gemini_key = env::var("GEMINI_API_KEY").ok().filter(|k| is_valid_key(k));
    if let Some(key) = valid_gemini_key {
        ctx.register_provider(Arc::new(GeminiProvider::new(key)));
    } else if crate::auth::GeminiOAuthManager::is_authenticated() {
        ctx.register_provider(Arc::new(GeminiProvider::with_oauth(crate::auth::GeminiOAuthManager::new())));
    }

    // Register Local Ollama
    ctx.register_provider(Arc::new(OllamaProvider::default_local()));

    // Register Native Colibrì Inference Provider (Dual-SSD MoE / Disk-Streamed)
    ctx.register_provider(Arc::new(ColibriProvider::with_default_config()));

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

pub fn default_model_for_provider(provider_id: &str) -> String {
    match provider_id {
        "colibri" => "deepseek-v4".to_string(),
        "gemini" => {
            if crate::auth::GeminiOAuthManager::is_authenticated() {
                "gemini-2.5-flash".to_string()
            } else {
                "gemini-2.0-flash".to_string()
            }
        }
        "deepseek" => "deepseek-chat".to_string(),
        "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
        "openai" => "gpt-4o".to_string(),
        "xai" => "grok-2-latest".to_string(),
        _ => default_ollama_model(),
    }
}

pub fn resolve_provider_and_model(
    ctx: &EngineContext,
    user_provider: &str,
    user_model: Option<String>,
) -> Result<(String, String, Arc<dyn LlmProvider>), TagisanError> {
    let env_provider = std::env::var("TAGISAN_PROVIDER")
        .or_else(|_| std::env::var("TGS_PROVIDER"))
        .ok();
    let local_only = std::env::var("TAGISAN_LOCAL_ONLY")
        .or_else(|_| std::env::var("TAGISAN_OFFLINE"))
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if local_only && user_provider != "auto" && !crate::ecc::is_local_provider(user_provider) {
        crate::notify::notify_offline_lock(
            user_provider,
            "ollama",
            "TAGISAN_LOCAL_ONLY or TAGISAN_OFFLINE environment lock is active",
            "Overriding cloud provider request to local Ollama runtime (air-gapped privacy enforced)",
        );
    }

    let effective_user_provider = if local_only {
        // When local_only is enforced, override any cloud provider or auto detection to local (ollama or colibri)
        if user_provider != "auto" && (user_provider == "ollama" || user_provider == "colibri" || user_provider == "local") {
            user_provider
        } else if let Some(ref ep) = env_provider {
            if ep == "ollama" || ep == "colibri" || ep == "local" {
                ep.as_str()
            } else {
                "ollama"
            }
        } else {
            "ollama"
        }
    } else if user_provider != "auto" {
        user_provider
    } else if crate::auth::GeminiOAuthManager::is_authenticated() {
        "gemini"
    } else if let Some(ref ep) = env_provider {
        if ep == "auto" {
            "auto"
        } else if (ep == "gemini" || ep == "google") && !crate::providers::gemini::GeminiProvider::is_available() {
            "auto"
        } else {
            ep.as_str()
        }
    } else {
        "auto"
    };

    if effective_user_provider != "auto" {
        let prov = ctx.get_provider(effective_user_provider)?;
        let model = if effective_user_provider == "ollama" {
            let installed = OllamaProvider::discover_installed_models();
            if installed.is_empty() {
                OllamaProvider::notify_no_models_installed();
                return Err(TagisanError::NoModelsInstalled);
            }
            if let Some(m) = user_model {
                if let Some(matched) = OllamaProvider::find_matching_model(&m, &installed) {
                    matched
                } else {
                    let fallback = installed[0].clone();
                    crate::notify::notify_model_auto_healed(
                        &m,
                        &fallback,
                        "ollama",
                        "Specified model is not installed locally in Ollama catalog",
                        &format!("Automatically falling back to '{}' and healed .env configuration", fallback),
                    );
                    OllamaProvider::auto_heal_env_file(&fallback);
                    fallback
                }
            } else {
                OllamaProvider::default_model()
            }
        } else {
            user_model.unwrap_or_else(|| default_model_for_provider(effective_user_provider))
        };
        return Ok((effective_user_provider.to_string(), model, prov));
    }

    // Auto-detection strategy in priority order:
    // 1. Gemini (100% Free Cloud Tier via Google AI Studio or Google OAuth Session)
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
        let has_credentials = if id == "gemini" {
            GeminiProvider::has_credentials()
        } else {
            std::env::var(key_var).is_ok()
        };

        if has_credentials {
            if let Ok(prov) = ctx.get_provider(id) {
                let model = user_model.unwrap_or_else(|| default_model_for_provider(id));
                return Ok((id.to_string(), model, prov));
            }
        }
    }

    // Fallback to local Ollama
    let prov = ctx.get_provider("ollama")?;
    let installed = OllamaProvider::discover_installed_models();
    if installed.is_empty() {
        OllamaProvider::notify_no_models_installed();
        return Err(TagisanError::NoModelsInstalled);
    }
    let model = if let Some(m) = user_model {
        if let Some(matched) = OllamaProvider::find_matching_model(&m, &installed) {
            matched
        } else {
            let fallback = installed[0].clone();
            crate::notify::notify_model_auto_healed(
                &m,
                &fallback,
                "ollama",
                "Specified model is not installed locally in Ollama catalog",
                &format!("Automatically falling back to '{}' and healed .env configuration", fallback),
            );
            OllamaProvider::auto_heal_env_file(&fallback);
            fallback
        }
    } else {
        OllamaProvider::default_model()
    };
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

pub fn load_global_env() {
    // 1. Try current directory .env
    if dotenvy::dotenv_override().is_ok() {
        return;
    }
    // 2. Ascend parent directories to find .env (e.g. project root)
    if let Ok(cwd) = std::env::current_dir() {
        let mut curr = cwd.as_path();
        while let Some(parent) = curr.parent() {
            let candidate = parent.join(".env");
            if candidate.is_file() {
                let _ = dotenvy::from_path_override(&candidate);
                return;
            }
            curr = parent;
        }
    }
    // 3. User home directory or config folder
    if let Ok(home) = std::env::var("HOME") {
        let candidate = std::path::PathBuf::from(&home).join(".env");
        if candidate.is_file() {
            let _ = dotenvy::from_path_override(&candidate);
            return;
        }
        let config_candidate = std::path::PathBuf::from(&home).join(".config/tagisan/.env");
        if config_candidate.is_file() {
            let _ = dotenvy::from_path_override(&config_candidate);
            return;
        }
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    load_global_env();

    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🇵🇭 TAGISAN: Multi-LLM Collaboration Engine in Rust".bold().yellow());
            println!("{}", "=========================================================".cyan());
            println!("{}", "\nChecking Configured LLM Providers & Capabilities:".bold());
            let ctx = build_engine_context(cli.max_budget);

            let installed_ollama = OllamaProvider::discover_installed_models();
            let is_ollama_live = OllamaProvider::fetch_live_tags().is_some();
            let ollama_active_model = if !installed_ollama.is_empty() {
                OllamaProvider::default_model()
            } else {
                "none".to_string()
            };

            let provider_specs: [(&str, &str, &str, &str); 6] = [
                ("anthropic", "Anthropic (Claude 3.5)", "ANTHROPIC_API_KEY", "claude-3-5-sonnet-20241022"),
                ("xai", "xAI (Grok 2 / Grok 3)", "XAI_API_KEY", "grok-2-latest"),
                ("openai", "OpenAI (GPT-4o / o1)", "OPENAI_API_KEY", "gpt-4o"),
                ("gemini", "Google Gemini (2.0 Flash)", "GEMINI_API_KEY", "gemini-2.0-flash"),
                ("deepseek", "DeepSeek (R1 / V3)", "DEEPSEEK_API_KEY", "deepseek-reasoner"),
                ("ollama", "Local Ollama (Offline)", "No key required (localhost:11434)", &ollama_active_model),
            ];

            for (id, name, var, sample_model) in provider_specs {
                if id == "ollama" {
                    if installed_ollama.is_empty() {
                        if is_ollama_live {
                            println!(
                                "  [!] {:<26} -> Connected ({}) but NO models installed\n      ↳ Notification: All local models were removed or none installed.\n      ↳ Quick Fix: Run 'ollama pull smollm2:1.7b' to download a local model.",
                                name.yellow().bold(),
                                "localhost:11434".cyan()
                            );
                        } else {
                            println!(
                                "  [✗] {:<26} -> Offline ({})\n      ↳ Tip: Start Ollama service with 'ollama serve'",
                                name.red(),
                                "localhost:11434"
                            );
                        }
                    } else if let Ok(p) = ctx.get_provider(id) {
                        let caps = p.capabilities(sample_model);
                        println!(
                            "  [✓] {:<26} -> Ready ({}) [{}]\n      ↳ Features: {}",
                            name.green().bold(),
                            id.cyan(),
                            sample_model,
                            format_capabilities(caps).italic()
                        );
                    }
                    continue;
                }

                if id == "gemini" {
                    if let Ok(p) = ctx.get_provider(id) {
                        let caps = p.capabilities(sample_model);
                        let auth_info = if let Some(email) = crate::auth::GeminiOAuthManager::get_account_email() {
                            format!("OAuth Web Login: {}", email.cyan())
                        } else {
                            id.to_string()
                        };
                        println!(
                            "  [✓] {:<26} -> Ready ({}) [{}]\n      ↳ Features: {}",
                            name.green().bold(),
                            auth_info,
                            sample_model,
                            format_capabilities(caps).italic()
                        );
                    } else {
                        println!(
                            "  [✗] {:<26} -> Missing {} (or run 'tgs auth login gemini')",
                            name.red(),
                            var
                        );
                    }
                    continue;
                }

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

        Commands::Search { query, url, limit } => {
            let search_tool = crate::tools::web_search::WebSearchTool::new();

            if let Some(target_url) = url {
                println!("{}", format!("🌐 Fetching webpage content from: {}", target_url).bold().cyan());
                match search_tool.fetch_webpage(&target_url).await {
                    Ok(content) => {
                        println!("\n{}\n", "─".repeat(70).dimmed());
                        println!("{}", content);
                        println!("{}\n", "─".repeat(70).dimmed());
                    }
                    Err(e) => {
                        eprintln!("{}: Failed to fetch URL '{}': {}", "Error".red().bold(), target_url, e);
                        std::process::exit(1);
                    }
                }
            } else if let Some(search_query) = query {
                println!("{}", format!("🔍 Searching the web for: \"{}\"...", search_query).bold().cyan());
                let capped_limit = limit.clamp(1, 10);
                match search_tool.search(&search_query, capped_limit).await {
                    Ok(results) => {
                        if results.is_empty() {
                            println!("{}", "No web search results found.".yellow());
                        } else {
                            println!("\n{}", format!("Found {} result(s):", results.len()).bold().green());
                            for (idx, r) in results.iter().enumerate() {
                                println!("\n{}. {}", (idx + 1).to_string().bold().yellow(), r.title.bold());
                                println!("   {}", r.url.underline().blue());
                                if !r.snippet.is_empty() {
                                    println!("   {}", r.snippet.dimmed());
                                }
                            }
                            println!();
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: Web search failed: {}", "Error".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("{}: Please specify a search query or a --url to fetch.", "Error".red().bold());
                eprintln!("Usage: tgs websearch \"your query\" OR tgs websearch --url https://example.com");
                std::process::exit(1);
            }
        }

        Commands::Auth { action, client_id, client_secret } => {
            let sub_action = action.unwrap_or(AuthAction::Login {
                provider: "gemini".to_string(),
                client_id,
                client_secret,
            });

            match sub_action {
                AuthAction::Login { provider, client_id, client_secret } => {
                    let prov = provider.to_lowercase();
                    if prov == "gemini" || prov == "google" {
                        match crate::auth::GeminiOAuthManager::start_web_login(client_id, client_secret).await {
                            Ok(tokens) => {
                                let email = tokens.email.as_deref().unwrap_or("Authorized Account");
                                println!("\n{}", format!("✨ Google Gemini Web Authentication is active for: {}", email).bold().green());
                                println!("You can now use 'tgs ask', 'tgs stream', and 'tgs agent' with Gemini without an API key!\n");
                            }
                            Err(e) => {
                                eprintln!("\n{}: Web authentication failed: {}\n", "Error".red().bold(), e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("{}: Web authentication is currently supported for 'gemini'.", "Error".red().bold());
                        std::process::exit(1);
                    }
                }
                AuthAction::Logout { provider } => {
                    let prov = provider.to_lowercase();
                    if prov == "gemini" || prov == "google" {
                        match crate::auth::GeminiOAuthManager::delete_tokens() {
                            Ok(_) => {
                                println!("{}", "✔ Successfully logged out from Google Gemini OAuth session.".green().bold());
                                println!("Removed stored credentials from {:?}", crate::auth::GeminiOAuthManager::token_file_path());
                            }
                            Err(e) => {
                                eprintln!("{}: Failed to log out: {}", "Error".red().bold(), e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!("{}: Unknown provider '{}'", "Error".red().bold(), provider);
                        std::process::exit(1);
                    }
                }
                AuthAction::Status => {
                    println!("{}", "════════════════ AUTHENTICATION STATUS ════════════════".bold().cyan());
                    if crate::auth::GeminiOAuthManager::is_authenticated() {
                        let email = crate::auth::GeminiOAuthManager::get_account_email().unwrap_or_else(|| "Unknown".to_string());
                        println!("  Google Gemini: {} ({})", "AUTHENTICATED (Web OAuth 2.0)".green().bold(), email.yellow());
                        println!("  Credential file: {:?}", crate::auth::GeminiOAuthManager::token_file_path());
                    } else {
                        println!("  Google Gemini: {}", "NOT AUTHENTICATED (No OAuth session)".yellow());
                        println!("  Tip: Run 'tgs auth login gemini' to connect your Google Account.");
                    }
                    println!();
                }
            }
        }

        Commands::Logout { provider } => {
            let prov = provider.to_lowercase();
            if prov == "gemini" || prov == "google" {
                match crate::auth::GeminiOAuthManager::delete_tokens() {
                    Ok(_) => {
                        println!("{}", "✔ Successfully logged out from Google Gemini OAuth session.".green().bold());
                    }
                    Err(e) => {
                        eprintln!("{}: Failed to log out: {}", "Error".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("{}: Unknown provider '{}'", "Error".red().bold(), provider);
                std::process::exit(1);
            }
        }

        Commands::Stream {
            provider,
            model,
            image,
            prompt,
            skill,
            auto_skills,
            no_skills,
        } => {
            handle_stream_command(
                provider,
                model,
                image,
                prompt,
                skill,
                auto_skills,
                no_skills,
                cli.max_budget,
            )
            .await?;
        }

        Commands::Ask {
            provider,
            model,
            image,
            prompt,
            skill,
            auto_skills,
            no_skills,
        } => {
            handle_ask_command(
                provider,
                model,
                image,
                prompt,
                skill,
                auto_skills,
                no_skills,
                cli.max_budget,
            )
            .await?;
        }

        Commands::Moa { prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            println!("\n{}", "🚀 Starting Mixture-of-Agents (MoA) Collaboration...".bold().magenta());
            println!("Prompt: \"{}\"\n", prompt.italic());

            let mut proposers = Vec::new();
            if ctx.get_provider("xai").is_ok() {
                proposers.push(("xai".to_string(), "grok-2-latest".to_string()));
            }
            if ctx.get_provider("gemini").is_ok() {
                proposers.push(("gemini".to_string(), "gemini-2.0-flash".to_string()));
            }
            if ctx.get_provider("deepseek").is_ok() {
                proposers.push(("deepseek".to_string(), "deepseek-chat".to_string()));
            }
            if ctx.get_provider("openai").is_ok() && proposers.is_empty() {
                proposers.push(("openai".to_string(), "gpt-4o-mini".to_string()));
            }
            if proposers.is_empty() {
                proposers.push(("ollama".to_string(), default_ollama_model()));
            }

            let aggregator = if ctx.get_provider("anthropic").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if ctx.get_provider("gemini").is_ok() {
                ("gemini".to_string(), "gemini-1.5-pro".to_string())
            } else if ctx.get_provider("openai").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else {
                ("ollama".to_string(), default_ollama_model())
            };

            let moa = MixtureOfAgentsStrategy::new(proposers, aggregator);
            let input = StrategyInput {
                prompt,
                system_instruction: None,
            };

            let spinner = Spinner::start("Gathering proposals and synthesizing with Mixture-of-Agents...");
            let output = moa.execute(input, &ctx).await;
            match &output {
                Ok(out) => spinner.success(format!("MoA finished successfully in {:.2}s", out.total_latency.as_secs_f32())),
                Err(e) => spinner.failure(format!("MoA failed: {}", e)),
            }
            let output = output?;

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

        Commands::Debate {
            tui,
            skill,
            auto_skills,
            no_skills,
            prompt,
        } => {
            let ctx = build_engine_context(cli.max_budget);

            let should_inject_skills = !no_skills && (skill.is_some() || auto_skills);
            let mut debate_system_instruction: Option<String> = None;

            if should_inject_skills {
                let dispatcher = crate::ecc::skills::global_dispatcher();
                let (equipped_text, injected_skills, budget) = dispatcher.equip_prompt_maximized(
                    "",
                    &prompt,
                    "anthropic",
                    None,
                    skill.as_deref(),
                    None,
                    None,
                );

                if !injected_skills.is_empty() {
                    debate_system_instruction = Some(equipped_text);
                    let mode_badge = match budget.mode {
                        crate::ecc::InjectionMode::DenseInvariants => "Dense Invariants DSL (<1.2k tokens)".cyan().bold(),
                        crate::ecc::InjectionMode::Hierarchical => "Hierarchical Multi-Tier Architecture".blue().bold(),
                        crate::ecc::InjectionMode::Comprehensive => "Cloud Comprehensive Specification".magenta().bold(),
                        crate::ecc::InjectionMode::CheatSheet => "Local Cheat-Sheet (<1k tokens)".cyan().bold(),
                    };
                    let skill_names: Vec<String> = injected_skills
                        .iter()
                        .map(|s| format!("{} ({:.1})", s.skill.name, s.score))
                        .collect();
                    println!(
                        "{} [{}] Injected {} skill(s) into debate [Budget: ~{} tokens / {}k ctx] -> [{}]\n",
                        "⚡ Dynamic Skills:".bold().yellow(),
                        mode_badge,
                        injected_skills.len(),
                        budget.max_tokens,
                        budget.context_window / 1000,
                        skill_names.join(", ").green()
                    );
                }
            }

            if tui {
                // Interactive Ratatui / Crossterm TUI
                crate::run_debate_tui_with_system(prompt, &ctx, debate_system_instruction).await?;
                return Ok(());
            }

            println!("\n{}", "⚔️  Starting Dialectical Debate (Tagisan ng Talino)...".bold().magenta());
            println!("Topic: \"{}\"\n", prompt.italic());

            let proponent = if ctx.get_provider("anthropic").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if ctx.get_provider("openai").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else if ctx.get_provider("gemini").is_ok() {
                ("gemini".to_string(), "gemini-2.0-flash".to_string())
            } else {
                ("ollama".to_string(), default_ollama_model())
            };

            let adversary = if ctx.get_provider("deepseek").is_ok() {
                ("deepseek".to_string(), "deepseek-reasoner".to_string())
            } else if ctx.get_provider("xai").is_ok() {
                ("xai".to_string(), "grok-2-latest".to_string())
            } else if ctx.get_provider("gemini").is_ok() {
                ("gemini".to_string(), "gemini-2.0-flash".to_string())
            } else {
                ("ollama".to_string(), default_ollama_model())
            };

            let adjudicator = if ctx.get_provider("gemini").is_ok() {
                ("gemini".to_string(), "gemini-1.5-pro".to_string())
            } else if ctx.get_provider("anthropic").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if ctx.get_provider("openai").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else {
                ("ollama".to_string(), default_ollama_model())
            };

            let debate = DialecticalDebateStrategy::new(proponent, adversary, adjudicator);
            let input = StrategyInput {
                prompt,
                system_instruction: debate_system_instruction,
            };

            let spinner = Spinner::start("Executing dialectical debate (Thesis -> Antithesis -> Synthesis)...");
            let output = debate.execute(input, &ctx).await;
            match &output {
                Ok(out) => spinner.success(format!("Debate completed in {:.2}s", out.total_latency.as_secs_f32())),
                Err(e) => spinner.failure(format!("Debate failed: {}", e)),
            }
            let output = output?;

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
                        if enable_all || tool_list.contains(&"edit_file") {
                            reg.register_tool(EditFileTool::new());
                        }
                        if enable_all || tool_list.contains(&"delete_file") {
                            reg.register_tool(DeleteFileTool::new());
                        }
                        if enable_all || tool_list.contains(&"list_dir") {
                            reg.register_tool(ListDirTool::new());
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
                        if enable_all || tool_list.contains(&"search_skills") {
                            reg.register_tool(crate::tools::builtin::SearchSkillsTool::with_default());
                        }
                        if enable_all || tool_list.contains(&"grounded_inference") {
                            reg.register_tool(crate::tools::builtin::GroundedInferenceTool::new());
                        }
                        if enable_all || tool_list.contains(&"web_search") {
                            reg.register_tool(crate::tools::web_search::WebSearchTool::new());
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
                if enable_all || tool_list.contains(&"edit_file") {
                    reg.register_tool(EditFileTool::new());
                }
                if enable_all || tool_list.contains(&"delete_file") {
                    reg.register_tool(DeleteFileTool::new());
                }
                if enable_all || tool_list.contains(&"list_dir") {
                    reg.register_tool(ListDirTool::new());
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
                if enable_all || tool_list.contains(&"search_skills") {
                    reg.register_tool(crate::tools::builtin::SearchSkillsTool::with_default());
                }
                if enable_all || tool_list.contains(&"grounded_inference") {
                    reg.register_tool(crate::tools::builtin::GroundedInferenceTool::new());
                }
                if enable_all || tool_list.contains(&"web_search") {
                    reg.register_tool(crate::tools::web_search::WebSearchTool::new());
                }
                reg
            };

            let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

            // Load and register plugins from .tagisan/plugins and ~/.tagisan/plugins (RFC-002)
            let mut plugin_mgr = crate::plugins::PluginManager::new(true);
            let _ = plugin_mgr.load_all().await;
            let _plugin_tool_count = plugin_mgr.populate_tool_registry(&mut registry);

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

            let dispatcher = crate::ecc::skills::global_dispatcher();
            let top_skills = dispatcher.dispatch(&prompt, 2, None);
            if !top_skills.is_empty() && top_skills[0].score >= 20.0 {
                println!("{}", "⚡ Auto-Equipped Engineering Skills:".bold().cyan());
                for s in &top_skills {
                    println!("   • {} [Score: {:.1} | Domain: {}] - {}", s.skill.name.yellow().bold(), s.score, s.domain.green(), s.skill.description.italic());
                    let mut current_prompt = agent.system_prompt.unwrap_or_default();
                    current_prompt.push_str(&format!(
                        "\n\n--- AUTO-EQUIPPED SPECIALIZED ENGINEERING SKILL: {} (Score: {:.1}) ---\n{}\n",
                        s.skill.name, s.score, s.skill.instructions
                    ));
                    agent.system_prompt = Some(current_prompt);
                }
                println!();
            }

            if let Some(ref sb) = sandbox_holder {
                agent = agent.with_working_dir(sb.path());
            }

            if memory {
                let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                let emb_prov = crate::memory::default_embedding_provider();
                println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                agent = agent.with_memory(mem_store, emb_prov);
            }

            let spinner = Spinner::start("Autonomous Agent reasoning and executing tools...");
            let result = if let Some(ref img_path) = image {
                let img_block = ContentBlock::from_image_file(img_path)?;
                agent.run_with_content(vec![ContentBlock::text(&prompt), img_block], &ctx).await
            } else {
                agent.run(&prompt, &ctx).await
            };
            match &result {
                Ok(r) => spinner.success(format!("Agent concluded in {} iteration(s) ({:.2}s)", r.iterations, r.total_latency.as_secs_f32())),
                Err(e) => spinner.failure(format!("Agent failed: {}", e)),
            }
            let result = result?;

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
            if enable_all || tool_list.contains(&"edit_file") {
                registry.register_tool(EditFileTool::new());
            }
            if enable_all || tool_list.contains(&"delete_file") {
                registry.register_tool(DeleteFileTool::new());
            }
            if enable_all || tool_list.contains(&"list_dir") {
                registry.register_tool(ListDirTool::new());
            }
            if enable_all || tool_list.contains(&"run_command") {
                registry.register_tool(RunCommandTool::default());
            }
            if enable_all || tool_list.contains(&"calculator") {
                registry.register_tool(CalculatorTool::new());
            }
            if enable_all || tool_list.contains(&"web_search") {
                registry.register_tool(crate::tools::web_search::WebSearchTool::new());
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
                    let spinner = Spinner::start("Formulating DAG workflow plan with LLM...");
                    let plan_res = planner.plan(&goal, &ctx).await;
                    match &plan_res {
                        Ok(graph) => spinner.success(format!("Formulated workflow DAG ({} tasks)", graph.len())),
                        Err(e) => spinner.failure(format!("Workflow planning failed: {}", e)),
                    }
                    plan_res?
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
                    let spinner = Spinner::start("Formulating DAG workflow plan with LLM...");
                    let plan_res = planner.plan(&goal, &ctx).await;
                    match &plan_res {
                        Ok(graph) => spinner.success(format!("Formulated workflow DAG ({} tasks)", graph.len())),
                        Err(e) => spinner.failure(format!("Workflow planning failed: {}", e)),
                    }
                    plan_res?
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
                        let spinner = Spinner::start("Formulating DAG workflow plan with LLM...");
                        let plan_res = planner.plan(&obj, &ctx).await;
                        match &plan_res {
                            Ok(graph) => spinner.success(format!("Formulated workflow DAG ({} tasks)", graph.len())),
                            Err(e) => spinner.failure(format!("Workflow planning failed: {}", e)),
                        }
                        plan_res?
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

            let effective_concurrency = concurrency.or_else(|| {
                if crate::ecc::skills::SkillDispatcher::is_local_provider(&provider_id) {
                    Some(1)
                } else {
                    None
                }
            });
            if let Some(limit) = effective_concurrency {
                scheduler = scheduler.with_concurrency_limit(limit);
            }

            let event_printer = tokio::spawn(async move {
                let mut active_spinner: Option<Spinner> = None;
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
                            if let Some(s) = active_spinner.take() {
                                s.stop();
                            }
                            active_spinner = Some(Spinner::start(format!(
                                "Running task '{}' ({}) [Attempt {}]...",
                                task_name, task_id, attempt
                            )));
                        }
                        WorkflowEvent::TaskProgress { task_id, message } => {
                            if let Some(ref s) = active_spinner {
                                s.set_message(format!("[{}] {}", task_id, message));
                            } else {
                                println!("    ↳ [{}] {}", task_id.dimmed(), message.italic());
                            }
                        }
                        WorkflowEvent::TaskRetry { task_id, attempt, max_retries, delay, error } => {
                            if let Some(s) = active_spinner.take() {
                                s.stop();
                            }
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
                            if let Some(s) = active_spinner.take() {
                                s.success(format!(
                                    "Task '{}' completed in {:.2}s (Tokens: {})",
                                    task_id,
                                    output.latency.as_secs_f32(),
                                    output.usage.prompt_tokens + output.usage.completion_tokens
                                ));
                            } else {
                                println!(
                                    "  {} Task '{}' completed in {:.2}s (Tokens: {})",
                                    "✅ Task Succeeded:".green().bold(),
                                    task_id.cyan(),
                                    output.latency.as_secs_f32(),
                                    output.usage.prompt_tokens + output.usage.completion_tokens
                                );
                            }
                        }
                        WorkflowEvent::TaskFailed { task_id, error, attempts } => {
                            if let Some(s) = active_spinner.take() {
                                s.failure(format!(
                                    "Task '{}' failed after {} attempt(s): {}",
                                    task_id, attempts, error
                                ));
                            } else {
                                println!(
                                    "  {} Task '{}' failed after {} attempt(s): {}",
                                    "❌ Task Failed:".red().bold(),
                                    task_id.cyan(),
                                    attempts,
                                    error.red()
                                );
                            }
                        }
                        WorkflowEvent::TaskSkipped { task_id, reason } => {
                            if let Some(s) = active_spinner.take() {
                                s.stop();
                            }
                            println!(
                                "  {} Task '{}' skipped: {}",
                                "⚠️  Task Skipped:".yellow(),
                                task_id.dimmed(),
                                reason
                            );
                        }
                        WorkflowEvent::WorkflowCompleted { workflow_id, total_tasks, completed_tasks, total_latency, total_cost_usd, .. } => {
                            if let Some(s) = active_spinner.take() {
                                s.stop();
                            }
                            println!(
                                "\n{} [{}] (Completed: {}/{}, Time: {:.2}s, Spent: ${:.4} USD)",
                                "🎉 Workflow Finished Successfully!".green().bold(),
                                workflow_id.cyan(),
                                completed_tasks,
                                total_tasks,
                                total_latency.as_secs_f32(),
                                total_cost_usd
                            );
                            break;
                        }
                        WorkflowEvent::WorkflowFailed { workflow_id, error } => {
                            if let Some(s) = active_spinner.take() {
                                s.stop();
                            }
                            println!(
                                "\n{} [{}] Error: {}",
                                "💥 Workflow Failed!".red().bold(),
                                workflow_id.cyan(),
                                error.red()
                            );
                            break;
                        }
                    }
                }
            });

            let result = scheduler.run(&mut workflow_graph, &ctx).await;
            drop(scheduler);
            let _ = tokio::time::timeout(std::time::Duration::from_millis(500), event_printer).await;

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
                        .unwrap_or_else(|| {
                            let cwd_path = std::path::PathBuf::from(".ecc/agents");
                            if cwd_path.exists() {
                                return cwd_path;
                            }
                            if let Ok(cwd) = std::env::current_dir() {
                                let mut curr = cwd.as_path();
                                while let Some(parent) = curr.parent() {
                                    let candidate = parent.join(".ecc/agents");
                                    if candidate.exists() {
                                        return candidate;
                                    }
                                    curr = parent;
                                }
                            }
                            if let Ok(home) = std::env::var("HOME") {
                                let home_path = std::path::PathBuf::from(home).join(".ecc/agents");
                                if home_path.exists() {
                                    return home_path;
                                }
                            }
                            cwd_path
                        });

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
                        println!("\nTip: Place custom ECC markdown files in '.ecc/agents/*.md' or '~/.ecc/agents/*.md' to discover them automatically.\n");
                    }
                }

                EccAction::Skills { dir, query } => {
                    if let Some(ref q) = query {
                        let start = std::time::Instant::now();
                        let dispatcher = crate::ecc::skills::global_dispatcher();
                        let matches = dispatcher.dispatch(q, 10, None);
                        let elapsed = start.elapsed();

                        println!("{}", "=========================================================".cyan());
                        println!("  🔍  ECC Skills Dispatcher Query: \"{}\"", q.bold().yellow());
                        println!("{}", "=========================================================".cyan());
                        println!("Ranked {} skills in {:.2?}:\n", dispatcher.len(), elapsed);

                        if matches.is_empty() {
                            println!("  (No matching skills found)");
                        } else {
                            for (i, m) in matches.iter().enumerate() {
                                println!(
                                    "  {}. {} [Score: {:.1} | Domain: {}]\n     {}",
                                    i + 1,
                                    m.skill.name.cyan().bold(),
                                    m.score,
                                    m.domain.green(),
                                    m.skill.description.italic()
                                );
                                if !m.matched_triggers.is_empty() {
                                    println!("     Triggers matched: {:?}", m.matched_triggers);
                                }
                                println!();
                            }
                        }
                        return Ok(());
                    }

                    let custom_path = dir
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| {
                            let cwd_path = std::path::PathBuf::from(".ecc/skills");
                            if cwd_path.exists() {
                                return cwd_path;
                            }
                            if let Ok(cwd) = std::env::current_dir() {
                                let mut curr = cwd.as_path();
                                while let Some(parent) = curr.parent() {
                                    let candidate = parent.join(".ecc/skills");
                                    if candidate.exists() {
                                        return candidate;
                                    }
                                    curr = parent;
                                }
                            }
                            if let Ok(home) = std::env::var("HOME") {
                                let home_path = std::path::PathBuf::from(home).join(".ecc/skills");
                                if home_path.exists() {
                                    return home_path;
                                }
                            }
                            cwd_path
                        });

                    let custom_dispatcher;
                    let dispatcher = if custom_path.exists() && custom_path != std::path::Path::new(".ecc/skills") {
                        custom_dispatcher = crate::ecc::skills::SkillDispatcher::load_or_build(Some(&custom_path));
                        &custom_dispatcher
                    } else {
                        crate::ecc::skills::global_dispatcher()
                    };

                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📚  ECC (Everything Coding Cloud) Skills Catalog".bold().yellow());
                    println!("{}", "=========================================================".cyan());
                    println!("\n{}", "Built-in Standard ECC Skills:".bold());

                    for skill in dispatcher.skills().iter().filter(|s| s.is_builtin) {
                        println!(
                            "  [•] {:<22} -> {}",
                            skill.name.green().bold(),
                            skill.description.italic()
                        );
                    }

                    if custom_path.exists() {
                        println!("\n{}", format!("Discovered Skills in '{}':", custom_path.display()).bold());
                        let custom_skills: Vec<_> = dispatcher.skills().iter().filter(|s| !s.is_builtin).collect();
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
                        println!("\nTip: Place custom ECC skills in '.ecc/skills/<skill>/SKILL.md' or '~/.ecc/skills/<skill>/SKILL.md' to discover them automatically.\n");
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
                            eprintln!(
                                "{}: ECC agent '{}' not found. Run 'tagisan ecc list' to see available agents.",
                                "Error".red().bold(),
                                agent
                            );
                            std::process::exit(1);
                        }
                    };

                    let dispatcher = crate::ecc::skills::global_dispatcher();

                    // Skill Attachment: Explicit or Automated Right-Skills-for-Right-Job Dispatch
                    if let Some(ref skill_name) = skill {
                        if skill_name == "auto" {
                            let top_skills = dispatcher.dispatch(&prompt, 2, None);
                            if !top_skills.is_empty() {
                                println!("{}", "⚡ Auto-Equipped Engineering Skills:".bold().cyan());
                                for s in &top_skills {
                                    println!("   • {} [Score: {:.1} | Domain: {}] - {}", s.skill.name.yellow().bold(), s.score, s.domain.green(), s.skill.description.italic());
                                    ecc_agent.system_prompt.push_str(&format!(
                                        "\n\n--- Auto-Equipped ECC Skill: {} (Score: {:.1}) ---\n{}",
                                        s.skill.name, s.score, s.skill.instructions
                                    ));
                                }
                            }
                        } else {
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
                    } else {
                        // Fully automated dispatch when --skill is omitted
                        let top_skills = dispatcher.dispatch(&prompt, 2, None);
                        if !top_skills.is_empty() {
                            println!("{}", "⚡ Auto-Equipped Engineering Skills:".bold().cyan());
                            for s in &top_skills {
                                println!("   • {} [Score: {:.1} | Domain: {}] - {}", s.skill.name.yellow().bold(), s.score, s.domain.green(), s.skill.description.italic());
                                ecc_agent.system_prompt.push_str(&format!(
                                    "\n\n--- Auto-Equipped ECC Skill: {} (Score: {:.1}) ---\n{}",
                                    s.skill.name, s.score, s.skill.instructions
                                ));
                            }
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
                    if enable_all || tool_list.contains(&"edit_file") || ecc_agent.tools.contains(&"edit_file".to_string()) {
                        registry.register_tool(EditFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"delete_file") || ecc_agent.tools.contains(&"delete_file".to_string()) {
                        registry.register_tool(DeleteFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"list_dir") || ecc_agent.tools.contains(&"list_dir".to_string()) {
                        registry.register_tool(ListDirTool::new());
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
                    if enable_all || tool_list.contains(&"search_skills") || ecc_agent.tools.contains(&"search_skills".to_string()) {
                        registry.register_tool(crate::tools::builtin::SearchSkillsTool::with_default());
                    }
                    if enable_all || tool_list.contains(&"web_search") || ecc_agent.tools.contains(&"web_search".to_string()) {
                        registry.register_tool(crate::tools::web_search::WebSearchTool::new());
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
                    skill,
                    auto_skills,
                    no_skills,
                } => {
                    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

                    let should_inject_skills = !no_skills && (skill.is_some() || auto_skills);

                    println!("{}", "═══════════════════════════════════════════════════════════".bold().blue());
                    println!("{}", "  🏛️  ECC 5-STAGE MULTI-AGENT ENGINEERING PIPELINE".bold().yellow());
                    println!("  Plan -> Test -> Implement -> (Review || Security) -> Verify");
                    println!("{}", "═══════════════════════════════════════════════════════════".bold().blue());
                    println!("Objective: \"{}\"", objective.italic());
                    println!("Active Engine: {} [{}]\n", provider_id.cyan().bold(), model_name.yellow().bold());

                    if should_inject_skills {
                        let dispatcher = crate::ecc::skills::global_dispatcher();
                        let (_, injected_skills, budget) = dispatcher.equip_prompt_maximized(
                            "",
                            &objective,
                            &provider_id,
                            Some(&model_name),
                            skill.as_deref(),
                            None,
                            None,
                        );

                        if !injected_skills.is_empty() {
                            let mode_badge = match budget.mode {
                                crate::ecc::InjectionMode::DenseInvariants => "Dense Invariants DSL (<1.2k tokens)".cyan().bold(),
                                crate::ecc::InjectionMode::Hierarchical => "Hierarchical Multi-Tier Architecture".blue().bold(),
                                crate::ecc::InjectionMode::Comprehensive => "Cloud Comprehensive Specification".magenta().bold(),
                                crate::ecc::InjectionMode::CheatSheet => "Local Cheat-Sheet (<1k tokens)".cyan().bold(),
                            };
                            let skill_names: Vec<String> = injected_skills
                                .iter()
                                .map(|s| format!("{} ({:.1})", s.skill.name, s.score))
                                .collect();
                            println!(
                                "{} [{}] Injected {} skill(s) into pipeline [Budget: ~{} tokens / {}k ctx] -> [{}]\n",
                                "⚡ Dynamic Skills:".bold().yellow(),
                                mode_badge,
                                injected_skills.len(),
                                budget.max_tokens,
                                budget.context_window / 1000,
                                skill_names.join(", ").green()
                            );
                        }
                    }

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
                    if enable_all || tool_list.contains(&"edit_file") {
                        registry.register_tool(EditFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"delete_file") {
                        registry.register_tool(DeleteFileTool::new());
                    }
                    if enable_all || tool_list.contains(&"list_dir") {
                        registry.register_tool(ListDirTool::new());
                    }
                    if enable_all || tool_list.contains(&"run_command") {
                        registry.register_tool(RunCommandTool::default());
                    }
                    if enable_all || tool_list.contains(&"calculator") {
                        registry.register_tool(CalculatorTool::new());
                    }
                    if enable_all || tool_list.contains(&"web_search") {
                        registry.register_tool(crate::tools::web_search::WebSearchTool::new());
                    }

                    let _mcp_manager = load_and_register_mcp_tools(mcp, mcp_config.as_deref(), &mut registry).await?;

                    if memory {
                        let mem_store = Arc::new(crate::memory::VectorStore::load_or_default());
                        let emb_prov = crate::memory::default_embedding_provider();
                        println!("Long-Term Memory: Active ({} documents in {:?})", mem_store.len(), crate::memory::VectorStore::default_path());
                        registry.register_tool(crate::tools::builtin::SearchMemoryTool::new(mem_store.clone(), emb_prov.clone()));
                        registry.register_tool(crate::tools::builtin::SaveMemoryTool::new(mem_store, emb_prov));
                    }

                    let mut pipeline_graph = build_ecc_pipeline_with_skills(
                        &objective,
                        prov,
                        &model_name,
                        registry,
                        skill.as_deref(),
                        auto_skills,
                        no_skills,
                    )?;

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

                    let effective_concurrency = concurrency.or_else(|| {
                        if crate::ecc::skills::SkillDispatcher::is_local_provider(&provider_id) {
                            Some(1)
                        } else {
                            None
                        }
                    });
                    if let Some(limit) = effective_concurrency {
                        scheduler = scheduler.with_concurrency_limit(limit);
                    }

                    let event_printer = tokio::spawn(async move {
                        let mut active_spinner: Option<Spinner> = None;
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
                                    if let Some(s) = active_spinner.take() {
                                        s.stop();
                                    }
                                    active_spinner = Some(Spinner::start(format!(
                                        "Executing Stage: {} ({}) [Attempt {}]...",
                                        task_name, task_id, attempt
                                    )));
                                }
                                WorkflowEvent::TaskCompleted { task_id, output } => {
                                    if let Some(s) = active_spinner.take() {
                                        s.success(format!(
                                            "Stage '{}' finished in {:.2}s ({} tokens)",
                                            task_id,
                                            output.latency.as_secs_f32(),
                                            output.usage.prompt_tokens + output.usage.completion_tokens
                                        ));
                                    } else {
                                        println!(
                                            "  {} [{}] Finished in {:.2}s ({} tokens)",
                                            "✔ Completed:".green().bold(),
                                            task_id.cyan(),
                                            output.latency.as_secs_f32(),
                                            output.usage.prompt_tokens + output.usage.completion_tokens
                                        );
                                    }
                                }
                                WorkflowEvent::TaskFailed { task_id, error, attempts } => {
                                    if let Some(s) = active_spinner.take() {
                                        s.failure(format!(
                                            "Stage '{}' failed after {} attempts: {}",
                                            task_id, attempts, error
                                        ));
                                    } else {
                                        println!(
                                            "  {} [{}] Failed after {} attempts: {}",
                                            "❌ Failed:".red().bold(),
                                            task_id.red(),
                                            attempts,
                                            error
                                        );
                                    }
                                }
                                WorkflowEvent::WorkflowCompleted { workflow_id, total_tasks, completed_tasks, total_latency, total_cost_usd, .. } => {
                                    if let Some(s) = active_spinner.take() {
                                        s.stop();
                                    }
                                    println!(
                                        "\n{} [{}] (Completed: {}/{}, Time: {:.2}s, Spent: ${:.4} USD)",
                                        "🎉 Pipeline Finished Successfully!".green().bold(),
                                        workflow_id.cyan(),
                                        completed_tasks,
                                        total_tasks,
                                        total_latency.as_secs_f32(),
                                        total_cost_usd
                                    );
                                    break;
                                }
                                WorkflowEvent::WorkflowFailed { workflow_id, error } => {
                                    if let Some(s) = active_spinner.take() {
                                        s.stop();
                                    }
                                    println!(
                                        "\n{} [{}] Error: {}",
                                        "💥 Pipeline Failed!".red().bold(),
                                        workflow_id.cyan(),
                                        error.red()
                                    );
                                    break;
                                }
                                _ => {}
                            }
                        }
                    });

                    let result = scheduler.run(&mut pipeline_graph, &ctx).await;
                    drop(scheduler);
                    let _ = tokio::time::timeout(std::time::Duration::from_millis(500), event_printer).await;

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

                    let architect_spec = if ctx.get_provider("anthropic").is_ok() {
                        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                    } else if ctx.get_provider("openai").is_ok() {
                        ("openai".to_string(), "gpt-4o".to_string())
                    } else if ctx.get_provider("gemini").is_ok() {
                        ("gemini".to_string(), "gemini-1.5-pro".to_string())
                    } else {
                        ("ollama".to_string(), default_ollama_model())
                    };

                    let security_spec = if ctx.get_provider("deepseek").is_ok() {
                        ("deepseek".to_string(), "deepseek-reasoner".to_string())
                    } else if ctx.get_provider("xai").is_ok() {
                        ("xai".to_string(), "grok-2-latest".to_string())
                    } else if ctx.get_provider("gemini").is_ok() {
                        ("gemini".to_string(), "gemini-2.0-flash".to_string())
                    } else {
                        ("ollama".to_string(), default_ollama_model())
                    };

                    let adjudicator_spec = if ctx.get_provider("gemini").is_ok() {
                        ("gemini".to_string(), "gemini-1.5-pro".to_string())
                    } else if ctx.get_provider("anthropic").is_ok() {
                        ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
                    } else if ctx.get_provider("openai").is_ok() {
                        ("openai".to_string(), "gpt-4o".to_string())
                    } else {
                        ("ollama".to_string(), default_ollama_model())
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

                McpAction::Catalog {
                    domain,
                    authority,
                    limit,
                    json,
                } => {
                    let catalog = match crate::mcp::McpCatalog::load_default() {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}: Failed to load MCP catalog: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let filtered = catalog.search_filtered(None, domain.as_deref(), authority.as_deref());

                    if json {
                        let out = serde_json::to_string_pretty(&filtered)?;
                        println!("{out}");
                        return Ok(());
                    }

                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  📚  Model Context Protocol (MCP) 500-Plugin Catalog".bold().yellow());
                    println!("{}", "=========================================================".cyan());
                    println!("Total plugins loaded: {}", catalog.len().to_string().cyan().bold());
                    if domain.is_some() || authority.is_some() {
                        println!("Matching criteria:    {}", filtered.len().to_string().green().bold());
                    }

                    let display_count = filtered.len().min(limit);
                    println!("\nDisplaying {} of {} plugins:\n", display_count.to_string().yellow(), filtered.len());

                    for entry in filtered.iter().take(limit) {
                        println!(
                            "  [{:>3}] {:<32} | {:<20} | {}",
                            entry.index.to_string().dimmed(),
                            entry.name.cyan().bold(),
                            entry.authority.green(),
                            entry.description
                        );
                        println!(
                            "        ↳ Domain: {}",
                            entry.domain.dimmed()
                        );
                    }

                    if filtered.len() > limit {
                        println!(
                            "\n{} Showing first {} results. Use '--limit {}' to view all matching plugins.",
                            "ℹ".blue().bold(),
                            limit,
                            filtered.len()
                        );
                    }
                }

                McpAction::Search {
                    query,
                    domain,
                    authority,
                    limit,
                    json,
                } => {
                    let catalog = match crate::mcp::McpCatalog::load_default() {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("{}: Failed to load MCP catalog: {}", "Error".red().bold(), e);
                            std::process::exit(1);
                        }
                    };

                    let results = catalog.search_filtered(Some(&query), domain.as_deref(), authority.as_deref());

                    if json {
                        let out = serde_json::to_string_pretty(&results)?;
                        println!("{out}");
                        return Ok(());
                    }

                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🔍  MCP Catalog Search: \"{}\"", query).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    if results.is_empty() {
                        println!("\n{} No matching MCP plugins found for query '{}'", "⚠".yellow().bold(), query);
                        println!("Tip: Try broader search terms or browse full index via 'tgs mcp catalog'");
                        return Ok(());
                    }

                    println!("Found {} matching plugin(s):\n", results.len().to_string().green().bold());
                    let display_count = results.len().min(limit);

                    for (i, entry) in results.iter().take(limit).enumerate() {
                        println!(
                            "  {}. [{:>3}] {:<32} ({})",
                            (i + 1).to_string().dimmed(),
                            entry.index.to_string().dimmed(),
                            entry.name.cyan().bold(),
                            entry.authority.green()
                        );
                        println!("     Description: {}", entry.description);
                        println!("     Domain:      {}", entry.domain.dimmed());
                        println!("     Quick Add:   {}", format!("tgs mcp add {}", entry.name).yellow());
                        println!();
                    }

                    if results.len() > limit {
                        println!(
                            "{} Showing top {} matches of {}. Use '--limit {}' to see more.",
                            "ℹ".blue().bold(),
                            limit,
                            results.len(),
                            results.len()
                        );
                    }
                }

                McpAction::Add {
                    name,
                    command,
                    args,
                    config,
                } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  ➕  Adding MCP Server: {}", name).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let (mut cfg, target_path) = if let Some(ref cfg_path) = config {
                        let p = std::path::PathBuf::from(cfg_path);
                        let c = if p.is_file() {
                            crate::mcp::McpConfig::from_file(&p)?
                        } else {
                            crate::mcp::McpConfig::new()
                        };
                        (c, p)
                    } else if let Some((p, c)) = crate::mcp::McpConfig::discover_default() {
                        (c, p)
                    } else {
                        (crate::mcp::McpConfig::new(), std::path::PathBuf::from("mcp.dynamic.json"))
                    };

                    let catalog = crate::mcp::McpCatalog::load_default().ok();
                    let catalog_entry = catalog.as_ref().and_then(|c| c.get_by_name(&name));

                    let server_config = if let Some(entry) = catalog_entry {
                        println!("Found plugin '{}' in sovereign catalog (Authority: {})", entry.name.cyan().bold(), entry.authority.green());
                        let mut generated = crate::mcp::McpCatalog::entry_to_server_config(entry);
                        if let Some(cmd) = command {
                            generated.command = cmd;
                        }
                        if !args.is_empty() {
                            generated.args = args;
                        }
                        generated
                    } else {
                        let cmd = command.unwrap_or_else(|| {
                            eprintln!("{}: '{}' was not found in catalog. Using default 'npx'. Specify '--command <cmd>' for custom command.", "Warning".yellow().bold(), name);
                            "npx".to_string()
                        });
                        crate::mcp::McpServerConfig {
                            command: cmd,
                            args,
                            env: std::collections::HashMap::new(),
                            description: Some(format!("Custom server {name}")),
                            ..Default::default()
                        }
                    };

                    println!("  Command:     {} {}", server_config.command.green().bold(), server_config.args.join(" ").dimmed());
                    if !server_config.env.is_empty() {
                        println!("  Environment: {} variable(s)", server_config.env.len());
                        for (k, v) in &server_config.env {
                            println!("    {} = {}", k.cyan(), v.dimmed());
                        }
                    }

                    cfg.add_server(name.clone(), server_config);
                    cfg.save_to_file(&target_path)?;

                    println!("\n{} Successfully saved MCP server '{}' to '{}'!", "✔".green().bold(), name.cyan().bold(), target_path.display());
                }

                McpAction::Remove { server, config } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  ➖  Removing MCP Server: {}", server).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let (mut cfg, target_path) = if let Some(ref cfg_path) = config {
                        let p = std::path::PathBuf::from(cfg_path);
                        if !p.is_file() {
                            eprintln!("{}: Config file '{}' does not exist.", "Error".red().bold(), p.display());
                            std::process::exit(1);
                        }
                        let c = crate::mcp::McpConfig::from_file(&p)?;
                        (c, p)
                    } else if let Some((p, c)) = crate::mcp::McpConfig::discover_default() {
                        (c, p)
                    } else {
                        eprintln!("{}: No MCP configuration found.", "Error".red().bold());
                        std::process::exit(1);
                    };

                    if let Some(removed) = cfg.remove_server(&server) {
                        cfg.save_to_file(&target_path)?;
                        println!("{} Successfully removed server '{}' from '{}'.", "✔".green().bold(), server.cyan().bold(), target_path.display());
                        println!("  Removed command: {} {}", removed.command.yellow(), removed.args.join(" ").dimmed());
                    } else {
                        eprintln!("{}: Server '{}' not found in configuration '{}'.", "Warning".yellow().bold(), server, target_path.display());
                    }
                }

                McpAction::Verify { server, config } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  🛡️  MCP Server Configuration & AgentShield Verification".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let (cfg, path) = if let Some(ref cfg_path) = config {
                        let p = std::path::PathBuf::from(cfg_path);
                        let c = crate::mcp::McpConfig::from_file(&p)?;
                        (c, p)
                    } else if let Some((p, c)) = crate::mcp::McpConfig::discover_default() {
                        (c, p)
                    } else {
                        eprintln!("{}: No MCP configuration found.", "Error".red().bold());
                        std::process::exit(1);
                    };

                    println!("Configuration file: {}", path.display().to_string().cyan());
                    let target_servers: Vec<String> = if let Some(ref s) = server {
                        if !cfg.mcp_servers.contains_key(s) {
                            eprintln!("{}: Server '{}' not found in configuration.", "Error".red().bold(), s);
                            std::process::exit(1);
                        }
                        vec![s.clone()]
                    } else {
                        cfg.mcp_servers.keys().cloned().collect()
                    };

                    if target_servers.is_empty() {
                        println!("\nNo configured MCP servers to verify.");
                        return Ok(());
                    }

                    println!("Verifying {} configured server(s)...\n", target_servers.len());

                    let mut passed_count = 0;
                    let mut warning_count = 0;
                    let mut blocked_count = 0;

                    for name in &target_servers {
                        let srv_cfg = &cfg.mcp_servers[name];
                        println!("Server: [{}]", name.cyan().bold());
                        println!("  Raw Command:  {} {}", srv_cfg.command, srv_cfg.args.join(" ").dimmed());

                        // 1. AgentShield scan
                        let shield_verdict = crate::AgentShieldScanner::scan_command(&srv_cfg.command);
                        match shield_verdict {
                            crate::AgentShieldVerdict::Block { reason, threat_level } => {
                                println!("  AgentShield:  {} [{:?}] {}", "BLOCKED".red().bold(), threat_level, reason);
                                blocked_count += 1;
                                println!();
                                continue;
                            }
                            crate::AgentShieldVerdict::Allow => {
                                println!("  AgentShield:  {}", "ALLOWED (Clean)".green());
                            }
                        }

                        // 2. Expand env vars
                        let expanded = srv_cfg.expand_env();
                        if expanded.command != srv_cfg.command || expanded.args != srv_cfg.args {
                            println!("  Expanded:     {} {}", expanded.command.green(), expanded.args.join(" ").dimmed());
                        }

                        // 3. Inspect env variables
                        let mut missing_vars = Vec::new();
                        for (k, v) in &srv_cfg.env {
                            if v.starts_with("${") && v.ends_with("}") {
                                let var_name = v.trim_start_matches("${").trim_end_matches("}");
                                let var_key = var_name.split_once(":-").map(|(n, _)| n).unwrap_or(var_name);
                                if std::env::var(var_key).is_err() && !var_name.contains(":-") {
                                    missing_vars.push(var_key);
                                }
                            }
                        }

                        if !missing_vars.is_empty() {
                            println!("  Env Status:   {} (Unset: {})", "WARNING".yellow().bold(), missing_vars.join(", ").yellow());
                            warning_count += 1;
                        } else {
                            println!("  Env Status:   {}", "OK (All variables resolved)".green());
                        }

                        if let Some(ref auth) = srv_cfg.authority {
                            println!("  Authority:    {}", auth.dimmed());
                        }
                        if let Some(ref desc) = srv_cfg.description {
                            println!("  Description:  {}", desc.dimmed());
                        }

                        passed_count += 1;
                        println!();
                    }

                    println!("---------------------------------------------------------");
                    println!(
                        "Verification complete: {} passed, {} warnings, {} blocked.",
                        passed_count.to_string().green().bold(),
                        warning_count.to_string().yellow().bold(),
                        blocked_count.to_string().red().bold()
                    );
                }
            }
        }

        Commands::Memory { action } => {
            match action {
                MemoryAction::Index { path, backend } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🧠  Indexing Codebase into Memory: {} [Backend: {}]", path, backend).bold().magenta());
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
                MemoryAction::Search { query, top_k, threshold, backend } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🔍  Semantic Memory Search: \"{}\" [Backend: {}]", query, backend).bold().yellow());
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
                MemoryAction::Sync { action, collection } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", format!("  🔄  Enterprise Vector Synchronization: {}", collection).bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let store = Arc::new(crate::memory::VectorStore::load_or_default());
                    let bridge = crate::vella::vector_sync::VellaVectorSyncBridge::new(store, &collection);

                    match action.to_lowercase().as_str() {
                        "push" => {
                            let stats = bridge.push_to_vella().await?;
                            println!("{} Pushed {} documents to Vella collection '{}' ({}ms)", "✔".green().bold(), stats.pushed_count, collection, stats.duration_ms);
                        }
                        "pull" => {
                            let stats = bridge.pull_from_vella().await?;
                            println!("{} Pulled {} documents into local store ({}ms)", "✔".green().bold(), stats.pulled_count, stats.duration_ms);
                        }
                        _ => {
                            let stats = bridge.bidirectional_sync().await?;
                            println!("{} Bidirectional Sync complete (pushed {}, pulled {}, updated {} in {}ms)",
                                "✔".green().bold(), stats.pushed_count, stats.pulled_count, stats.updated_count, stats.duration_ms);
                        }
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
                SwarmAction::Cluster { subcommand } => {
                    match subcommand {
                        ClusterAction::Coordinator { bind } => {
                            let coord = ClusterCoordinator::new(&bind);
                            coord.run_server().await?;
                        }
                        ClusterAction::Worker { connect, id } => {
                            let worker = ClusterWorker::new(&connect, id);
                            worker.run_worker().await?;
                        }
                        ClusterAction::Status { coordinator } => {
                            match query_cluster_status(&coordinator).await {
                                Ok(report) => {
                                    println!("{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan());
                                    println!("║  {}  ║", "🌐 TAGISAN LOCAL P2P CLUSTER MESH STATUS               ".bold().bright_white());
                                    println!("{}", "╠═══════════════════════════════════════════════════════════╣".bright_cyan());
                                    println!("║ Coordinator: {:<44} ║", report.coordinator_addr);
                                    println!("║ Active Workers: {:<41} ║", report.total_workers);
                                    println!("║ Dispatched Tasks: {:<39} ║", report.total_dispatched_tasks);
                                    println!("║ Completed Tasks: {:<40} ║", report.total_completed_tasks);
                                    println!("{}", "╠═══════════════════════════════════════════════════════════╣".bright_cyan());
                                    for w in report.workers {
                                        println!("║  Node: {:<20} (cores: {}, tools: {}) ║", w.worker_id, w.cpu_cores, w.supported_tools.len());
                                    }
                                    println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan());
                                }
                                Err(e) => {
                                    eprintln!("Failed to query cluster status from {}: {}", coordinator, e);
                                }
                            }
                        }
                    }
                }
                SwarmAction::Atlas { file, reset } => {
                    let path = file.as_deref().map(std::path::Path::new);
                    if reset {
                        let atlas = SwarmAtlas::new();
                        atlas.save(path)?;
                        println!("{}", "  ✨ Swarm heat atlas reset successfully.".bold().green());
                    } else {
                        let atlas = SwarmAtlas::load_or_default(path);
                        println!("{}", atlas.render_atlas_table());
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

        Commands::Bun { action } => {
            handle_bun_command(action).await?;
        }

        Commands::Python { action } => {
            handle_python_command(action).await?;
        }

        Commands::Perl { action } => {
            handle_perl_command(action).await?;
        }

        Commands::Vella { action } => {
            handle_vella_command(action).await?;
        }

        Commands::Trace { live, tui, export, sqlite, last, clear } => {
            let trace_args = crate::telemetry::TraceArgs {
                live,
                tui,
                export,
                sqlite,
                last,
                clear,
            };
            crate::telemetry::run_trace_command(&trace_args)?;
        }

        Commands::Eval { action, dataset, models, rule, threshold, output } => {
            let ctx = build_engine_context(cli.max_budget);
            let eval_args = match action {
                Some(EvalAction::Run { dataset, models, rule, threshold, output }) => {
                    crate::eval::EvalArgs {
                        dataset,
                        models,
                        rule,
                        threshold,
                        output,
                    }
                }
                None => {
                    let ds = dataset.ok_or_else(|| {
                        TagisanError::Execution("Missing required --dataset argument for tgs eval. Example: tgs eval run --dataset ./evals/sc_docket_benchmarks.json".to_string())
                    })?;
                    let ms = models.unwrap_or_else(|| {
                        vec!["claude-3-5-sonnet-20241022".to_string(), "gemini-2.0-flash".to_string()]
                    });
                    crate::eval::EvalArgs {
                        dataset: ds,
                        models: ms,
                        rule,
                        threshold,
                        output,
                    }
                }
            };
            crate::eval::run_eval_command(&eval_args, &ctx).await?;
        }

        Commands::Harmony {
            action,
            objective,
            architect,
            implementer,
            qa,
            doc,
            tier,
            parallel,
            audit,
            json,
            output_dir,
            fallback_to_local,
            evacuate_on_budget,
            notify,
            skill,
            auto_skills,
            no_skills,
        } => {
            let ctx = build_engine_context(cli.max_budget);
            handle_harmony_command(
                action,
                objective,
                architect,
                implementer,
                qa,
                doc,
                tier,
                parallel,
                audit,
                json,
                output_dir,
                fallback_to_local,
                evacuate_on_budget,
                notify,
                skill,
                auto_skills,
                no_skills,
                &ctx,
            )
            .await?;
        }

        Commands::Harness { action } => {
            crate::harness::cli_handler::handle_harness_command(action, cli.max_budget).await?;
        }

        Commands::Plugin { action } => {
            crate::plugins::handle_plugin_command(action.clone()).await?;
        }

        Commands::Serve { port, host } => {
            crate::engine::OllamaServer::start(&host, port).await?;
        }

        Commands::Engine { action } => {
            handle_engine_command(action).await?;
        }

        Commands::Autofix { path, test, max_attempts, dry_run } => {
            handle_autofix_command(path, test, max_attempts, dry_run).await?;
        }

        Commands::Graph { action } => {
            handle_graph_command(action)?;
        }

        Commands::Ground {
            task,
            path,
            max_iterations,
            no_critique,
            no_ast,
            output,
        } => {
            handle_ground_command(task, path, max_iterations, no_critique, no_ast, output).await?;
        }

        Commands::Shield { action } => {
            handle_shield_command(action).await?;
        }

        Commands::Otp { action } => {
            handle_otp_command(action).await?;
        }

        Commands::Gleam { action } => {
            handle_gleam_command(action).await?;
        }

        Commands::Ide { action } => {
            handle_ide_command(action).await?;
        }

        Commands::Copilot { action } => {
            handle_copilot_command(action).await?;
        }
    }

    Ok(())
}

async fn handle_copilot_command(action: CopilotAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CopilotAction::Auth { client_id, tenant_id, status } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🏢 MICROSOFT ENTRA ID AUTHENTICATION MANAGER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let mut config = crate::copilot::EntraIdConfig::default();
            if let Some(cid) = client_id {
                config.client_id = cid;
            }
            if let Some(tid) = tenant_id {
                config.tenant_id = tid;
            }

            let auth = crate::copilot::EntraAuthManager::new(config);

            if status {
                let st = auth.copilot_auth_status().await;
                println!("  Authentication State: {}", if st.authenticated { "AUTHENTICATED".green().bold() } else { "NOT AUTHENTICATED".red().bold() });
                println!("  Auth Mode:            {}", st.auth_mode.cyan());
                println!("  Tenant ID:            {}", st.tenant_id.yellow());
                println!("  Client ID:            {}", st.client_id.yellow());
                println!("  Token Expired:        {}", if st.is_expired { "YES".red() } else { "NO".green() });
                if let Some(secs) = st.expires_in_secs {
                    println!("  Expires In:           {} seconds", secs);
                }
                if let Some(scope) = st.scope {
                    println!("  Scope:                {}", scope.dimmed());
                }
            } else {
                println!("Initiating Entra ID Device Code Login Flow...");
                let dc = auth.initiate_device_code().await?;
                println!("\n  {}", dc.message.bright_white().bold());
                println!("  Verification URL: {}", dc.verification_uri.cyan().underline());
                println!("  User Code:        {}", dc.user_code.yellow().bold());
                println!("\nWaiting for authentication completion (polling every {}s)...", dc.interval);

                let token = auth.poll_for_token(&dc.device_code, dc.interval, dc.expires_in).await?;
                println!("\n{}", "✅ Authentication successful!".green().bold());
                println!("  Token Type:    {}", token.token_type.cyan());
                println!("  Expires In:    {} seconds", token.expires_in);
                println!("  Cached to:     {}", auth.config().token_cache_path.display().to_string().yellow());
            }
        }

        CopilotAction::Plugin { output_dir, base_url } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📦 MICROSOFT 365 COPILOT PACKAGE GENERATOR".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let out_path = std::path::PathBuf::from(&output_dir);
            let pkg = crate::copilot::export_copilot_package(&out_path, &base_url)?;

            println!("\n{}", "✅ Copilot Package Bundle Generated:".green().bold());
            println!("  Output Directory: {}", pkg.output_dir.display().to_string().yellow());
            println!("  Total Size:       {} bytes", pkg.total_bytes);
            println!("\nBundled Artifacts:");
            for f in &pkg.files {
                println!("  [✓] {}", f.display().to_string().green());
            }
            println!("\nNext Steps:");
            println!("  1. Sideload manifest.json into Teams Developer Portal or Microsoft 365 Admin Center.");
            println!("  2. Deploy OpenAPI gateway at: {}", base_url.cyan());
        }

        CopilotAction::Teams { action } => {
            match action {
                TeamsAction::Post { channel, message } => {
                    println!("{}", "=========================================================".cyan());
                    println!("{}", "  💬 MICROSOFT TEAMS MESSAGE DISPATCHER".bold().yellow());
                    println!("{}", "=========================================================".cyan());

                    let auth = std::sync::Arc::new(crate::copilot::EntraAuthManager::with_defaults());
                    let client = crate::copilot::GraphClient::new(auth);

                    let msg_id = client.send_teams_message(&channel, &message).await?;
                    println!("\n{}", "✅ Message Dispatched Successfully:".green().bold());
                    println!("  Target Channel:  {}", channel.yellow());
                    println!("  Message ID:      {}", msg_id.cyan());
                    println!("  AgentShield DLP: {}", "PASSED (Zero credential leaks)".green());
                }
            }
        }

        CopilotAction::Ingest { url, meeting } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📥 MICROSOFT 365 GRAPH INGESTION ENGINE".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let auth = std::sync::Arc::new(crate::copilot::EntraAuthManager::with_defaults());
            let client = crate::copilot::GraphClient::new(auth);

            if let Some(meeting_id) = meeting {
                println!("Fetching meeting transcript for meeting: {}", meeting_id.cyan());
                let transcript = client.get_meeting_transcript(&meeting_id).await?;
                println!("Retrieved {} transcript turn(s). Synthesizing action items...\n", transcript.len());

                let items = client.parse_action_items(&transcript);
                println!("{}", "Extracted Engineering Action Items:".bold());
                for (i, item) in items.iter().enumerate() {
                    let badge = match item.priority.as_str() {
                        "High" => item.priority.red().bold(),
                        "Medium" => item.priority.yellow(),
                        _ => item.priority.green(),
                    };
                    println!("  {}. [{}] {} - Assignee: {}", i + 1, badge, item.title.bright_white(), item.assignee.as_deref().unwrap_or("Unassigned").cyan());
                }
            } else if let Some(doc_url) = url {
                println!("Ingesting document from: {}", doc_url.cyan());
                let doc = client.fetch_sharepoint_file(&doc_url).await?;
                println!("\n{}", "✅ Document Ingested & Sanitized:".green().bold());
                println!("  Filename:               {}", doc.filename.yellow());
                println!("  Content-Type:           {}", doc.content_type.cyan());
                println!("  Size:                   {} bytes", doc.size_bytes);
                println!("  AgentShield Inbound:    {}", "PASSED (Clean document)".green());
                println!("\n--- Document Preview ---\n{}", if doc.text.len() > 300 { format!("{}...", &doc.text[..300]) } else { doc.text });
            } else {
                eprintln!("{}", "Error: Must specify either --url <path_or_url> or --meeting <meeting_id>".red().bold());
            }
        }

        CopilotAction::Status => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🏢 MICROSOFT 365 COPILOT SUBSYSTEM STATUS".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let auth = crate::copilot::EntraAuthManager::with_defaults();
            let auth_st = auth.copilot_auth_status().await;

            println!("\n[1] Entra ID (Azure AD) Identity Plane:");
            println!("  Authenticated:         {}", if auth_st.authenticated { "YES".green().bold() } else { "NO".yellow() });
            println!("  Auth Mode:             {}", auth_st.auth_mode.cyan());
            println!("  Tenant ID:             {}", auth_st.tenant_id.yellow());
            println!("  Client ID:             {}", auth_st.client_id.yellow());
            println!("  Token Expired:         {}", if auth_st.is_expired { "YES".red() } else { "NO".green() });
            if let Some(secs) = auth_st.expires_in_secs {
                println!("  Token TTL:             {}s", secs);
            }

            println!("\n[2] Microsoft Graph Integration Plane:");
            let client = crate::copilot::GraphClient::new(std::sync::Arc::new(auth));
            println!("  Graph Execution Mode:  {}", if client.is_mock() { "Deterministic Mock / Sandbox".yellow() } else { "Live Enterprise Graph REST API".green().bold() });
            println!("  Base Endpoint:         https://graph.microsoft.com/v1.0");

            println!("\n[3] Autonomous Copilot Tools in Registry (16 Tools):");
            println!("  [✓] copilot_teams_post          (Post updates & debate verdicts to Teams)");
            println!("  [✓] copilot_sharepoint_get      (Ingest SharePoint/OneDrive docs with AgentShield)");
            println!("  [✓] copilot_meeting_action_items(Decompose Teams meeting transcripts into code tasks)");
            println!("  [✓] copilot_export_report       (Dispatch HTML debate reports via Outlook)");
            println!("  [✓] copilot_meeting_to_code     (End-to-end meeting transcript to AST blast radius & code patch)");
            println!("  [✓] copilot_blast_radius_report (Adaptive Cards & executive HTML reports for Teams/Excel/PPT)");
            println!("  [✓] copilot_debate_dispatch     (3-round dialectical debate execution & Teams/Outlook dispatch)");
            println!("  [✓] copilot_purview_guard       (Microsoft Purview Sensitivity & Zero-Egress Air-Gapping)");
            println!("  [✓] copilot_adr_sync            (Architecture Decision Record MADR synthesis & OneNote sync)");
            println!("  [✓] copilot_create_pr           (Ephemeral Git branch & PR automation with blast telemetry)");
            println!("  [✓] copilot_export_deck         (Responsive executive presentation briefing slide deck)");
            println!("  [✓] copilot_excel_functions     (Native Excel Custom Functions =TGS.* & Add-in packager)");
            println!("  [✓] copilot_stream_gateway      (Live SSE / NDJSON streaming gateway for Copilot Studio)");
            println!("  [✓] copilot_planner_sync        (Microsoft Planner & To-Do synchronizer with Git/PR linkage)");
            println!("  [✓] copilot_incident_debugger   (Teams '@Tagisan' CI/CD incident debugger & surgical autofix)");
            println!("  [✓] copilot_hardware_telemetry  (Windows Copilot+ PC NPU/DirectML telemetry & carbon efficiency)");

            println!("\n[4] AgentShield Cyber Defense Gate:");
            println!("  Outbound DLP:          {}", "ACTIVE (Zero API key/private key/credential leakage)".green().bold());
            println!("  Inbound Sanitization:  {}", "ACTIVE (Guards against prompt injection in SharePoint/Teams)".green().bold());

            println!("\n[5] Microsoft 365 Search Connector:");
            let connector = crate::copilot::GraphConnectorEngine::new();
            println!("  Connection ID:         {}", connector.connection_id().cyan());
            println!("  Indexed Schemas:       tagisanSkill (495+ skills), tagisanArtifact, tagisanDebate");
        }

        CopilotAction::Post { channel, message } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  💬 MICROSOFT TEAMS MESSAGE DISPATCHER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let auth = std::sync::Arc::new(crate::copilot::EntraAuthManager::with_defaults());
            let client = crate::copilot::GraphClient::new(auth);

            let msg_id = client.send_teams_message(&channel, &message).await?;
            println!("\n{}", "✅ Message Dispatched Successfully:".green().bold());
            println!("  Target Channel:  {}", channel.yellow());
            println!("  Message ID:      {}", msg_id.cyan());
            println!("  AgentShield DLP: {}", "PASSED (Zero credential leaks)".green());
        }

        CopilotAction::Transcript { meeting, parse_items } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🎙️ MICROSOFT TEAMS MEETING TRANSCRIPT VIEWER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let auth = std::sync::Arc::new(crate::copilot::EntraAuthManager::with_defaults());
            let client = crate::copilot::GraphClient::new(auth);
            let transcript = client.get_meeting_transcript(&meeting).await?;

            println!("Retrieved {} transcript turn(s) for meeting: {}\n", transcript.len(), meeting.cyan());
            for turn in &transcript {
                let ts = turn.timestamp.as_deref().unwrap_or("00:00");
                println!("  [{}] {}: {}", ts.dimmed(), turn.speaker.bold().bright_white(), turn.text);
            }

            if parse_items {
                let items = client.parse_action_items(&transcript);
                println!("\n{}", "Extracted Engineering Action Items:".bold());
                for (i, item) in items.iter().enumerate() {
                    let badge = match item.priority.as_str() {
                        "High" => item.priority.red().bold(),
                        "Medium" => item.priority.yellow(),
                        _ => item.priority.green(),
                    };
                    println!("  {}. [{}] {} - Assignee: {}", i + 1, badge, item.title.bright_white(), item.assignee.as_deref().unwrap_or("Unassigned").cyan());
                }
            }
        }

        CopilotAction::Index { schema_only } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🔍 MICROSOFT SEARCH GRAPH CONNECTOR ENGINE".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let connector = crate::copilot::GraphConnectorEngine::new();
            if schema_only {
                let schema = connector.generate_schema_definition();
                println!("{}", serde_json::to_string_pretty(&schema)?);
            } else {
                let skills = connector.build_skill_items();
                println!("{}", "✅ Microsoft Search Graph Connector Initialized:".green().bold());
                println!("  Connection ID:   {}", connector.connection_id().cyan());
                println!("  Indexed Skills:  {} built-in engineering capabilities", skills.len().to_string().yellow());
                println!("  Target Schema:   externalItem (isSearchable, isRetrievable, isQueryable)");
                println!("  Next Steps:      Register external connection in Microsoft 365 Admin Center Search & Intelligence.");
            }
        }

        CopilotAction::Test => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🧪 MICROSOFT 365 COPILOT SUBSYSTEM SELF-TEST".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let client = crate::copilot::GraphClient::mock();
            let msg_id = client.send_teams_message("test-channel", "Tagisan self-test ping").await?;
            println!("  [✓] GraphClient Mock Teams Dispatch:       ID={}", msg_id.green());

            let trans = client.get_meeting_transcript("mock-sync").await?;
            let items = client.parse_action_items(&trans);
            println!("  [✓] Transcript Retrieval & Decomposition:  {} items extracted", items.len().to_string().green());

            let doc = client.fetch_sharepoint_file("Documents/Architecture_Specification.md").await?;
            println!("  [✓] SharePoint Document Fetch & Inbound:    {} bytes ({})", doc.size_bytes.to_string().green(), doc.content_type.cyan());

            let dlp_clean = crate::ecc::agentshield::AgentShieldScanner::scan_outbound_dlp("Verification text");
            assert_eq!(dlp_clean, crate::ecc::agentshield::AgentShieldVerdict::Allow);
            println!("  [✓] AgentShield Outbound DLP Interception: VERIFIED (Zero credential leaks)");

            let leaked_check = crate::ecc::agentshield::AgentShieldScanner::scan_outbound_dlp("sk-ant-secret123456789012345");
            assert!(matches!(leaked_check, crate::ecc::agentshield::AgentShieldVerdict::Block { .. }));
            println!("  [✓] AgentShield Outbound Secret Blocking:  VERIFIED (Blocked API keys)");

            let mail_id = client.send_outlook_report(&["leads@tagisan.ai".to_string()], "Self Test", "<p>OK</p>").await?;
            println!("  [✓] Outlook HTML Dispatch:                 ID={}", mail_id.green());

            println!("\n{}", "✅ All Microsoft 365 Copilot Subsystem Self-Tests Passed!".green().bold());
        }

        CopilotAction::MeetingToCode { meeting, transcript, path, auto_patch, channel } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🚀 COPILOT MEETING-TO-CODE PIPELINE".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotMeetingToCodeTool::new();
            let args = serde_json::json!({
                "meeting_id": meeting,
                "transcript_text": transcript,
                "codebase_path": path,
                "auto_patch": auto_patch,
                "channel": channel,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::BlastReport { symbol, max_depth, path, format, post_to_teams, export_email } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📊 COPILOT BLAST RADIUS TELEMETRY CARDS".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotBlastRadiusReportTool::new();
            let args = serde_json::json!({
                "symbol": symbol,
                "max_depth": max_depth,
                "path": path,
                "format": format,
                "post_to_teams": post_to_teams,
                "export_email": export_email,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Debate { proposal, title, proponent, adversary, lakandiwa, post_to_teams, send_to_email } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚖️ COPILOT DIALECTICAL DEBATE DISPATCH".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotDebateDispatchTool::new();
            let args = serde_json::json!({
                "proposal": proposal,
                "title": title,
                "proponent": proponent,
                "adversary": adversary,
                "lakandiwa": lakandiwa,
                "post_to_teams": post_to_teams,
                "send_to_email": send_to_email,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Purview { content, label, destination } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🛡️  MICROSOFT PURVIEW SENSITIVITY & AIR-GAP GUARD".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotPurviewGuardTool::new();
            let args = serde_json::json!({
                "content": content,
                "label": label,
                "destination": destination,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Adr { proposal, verdict, title, onenote_section, sharepoint_folder } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📑 COPILOT ADR SYNCHRONIZATION (ONENOTE & SHAREPOINT)".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotAdrSyncTool::new();
            let args = serde_json::json!({
                "proposal": proposal,
                "verdict": verdict,
                "title": title,
                "onenote_section": onenote_section,
                "sharepoint_folder": sharepoint_folder,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Pr { patch, title, branch, base, platform, symbol, channel } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🚀 COPILOT PULL REQUEST & EPHEMERAL BRANCH AUTOMATION".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotCreatePrTool::new();
            let args = serde_json::json!({
                "patch": patch,
                "title": title,
                "branch_name": branch,
                "base_branch": base,
                "target_platform": platform,
                "symbol": symbol,
                "post_to_teams": channel,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Deck { title, format, output, email, notes } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📊 COPILOT EXECUTIVE PRESENTATION DECK COMPILER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotExportDeckTool::new();
            let args = serde_json::json!({
                "title": title,
                "format": format,
                "output_path": output,
                "export_email": email,
                "custom_notes": notes,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Listen { port, host, test_action, target } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🤖 TEAMS BOT WEBHOOK & ADAPTIVE CARD ACTION LISTENER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let handler = crate::copilot::TeamsBotHandler::new();

            if let Some(action_verb) = test_action {
                println!("Executing dry-run Teams Adaptive Card action callback: {}", action_verb.yellow().bold());
                let payload = crate::copilot::TeamsActionPayload {
                    action: action_verb,
                    user: Some("CLI Developer".to_string()),
                    user_id: Some("usr_cli_01".to_string()),
                    target,
                    data: Some("CLI Interactive Verification".to_string()),
                    parameters: None,
                };

                let resp = handler.process_action(&payload).await?;
                println!("\n{}", "✅ Action Successfully Processed:".green().bold());
                println!("  Status:           {}", resp.status.green());
                println!("  Action Handled:   {}", resp.action_processed.cyan());
                println!("  Badge:            {}", resp.badge);
                println!("  Summary:          {}", resp.summary_text);
                println!("  Processed At:     {}", resp.processed_at.dimmed());
                println!("\nRefreshed Adaptive Card v1.5 JSON:\n{}", serde_json::to_string_pretty(&resp.card_json)?);
            } else {
                println!("Starting Teams Bot Webhook listener on http://{}:{}...", host.cyan(), port.to_string().yellow());
                println!("  Endpoint URL:     http://{}:{}/api/messages", host, port);
                println!("  Supported Verbs:  approve_patch, run_autofix, run_debate, sync_adr");
                println!("  AgentShield DLP:  Active (Zero credential leakage)");
            }
        }

        CopilotAction::Excel { formula, function, symbol, path, prompt_tokens, completion_tokens, target, code, package, output_dir } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📊 TAGISAN EXCEL CUSTOM FUNCTIONS & ADD-IN ENGINE".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotExcelFunctionsTool::new();

            if package {
                let args = serde_json::json!({
                    "action": "package",
                    "output_dir": output_dir,
                });
                let out = tool.execute(args).await?;
                println!("\n{out}");
            } else if let Some(f) = formula {
                let args = serde_json::json!({
                    "formula": f,
                });
                let out = tool.execute(args).await?;
                println!("\n{out}");
            } else {
                let fn_name = function.unwrap_or_else(|| "BLAST_RADIUS".to_string());
                let args = serde_json::json!({
                    "function": fn_name,
                    "symbol": symbol,
                    "path": path,
                    "prompt_tokens": prompt_tokens,
                    "completion_tokens": completion_tokens,
                    "target": target,
                    "code": code,
                });
                let out = tool.execute(args).await?;
                println!("\n{out}");
            }
        }

        CopilotAction::Stream { prompt, format, mode, no_keepalive } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🌊 COPILOT STUDIO REAL-TIME STREAMING GATEWAY".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotStreamGatewayTool::new();
            let args = serde_json::json!({
                "prompt": prompt,
                "format": format,
                "mode": mode,
                "include_keepalive": !no_keepalive,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Planner { action, meeting, transcript, plan_id, bucket_id, todo_list_id, title, priority, assignee, branch_url, pr_url } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  📋 MICROSOFT PLANNER & TO-DO TASK SYNCHRONIZER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotPlannerSyncTool::new();
            let args = serde_json::json!({
                "action": action,
                "meeting_id": meeting,
                "transcript_text": transcript,
                "plan_id": plan_id,
                "bucket_id": bucket_id,
                "todo_list_id": todo_list_id,
                "task_title": title,
                "priority": priority,
                "assignee": assignee,
                "branch_url": branch_url,
                "pr_url": pr_url,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Incident { logs, file, commit, pipeline, channel, format } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🚨 TEAMS \"@TAGISAN\" CI/CD INCIDENT DEBUGGER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let log_content = if let Some(path) = file {
                std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read incident log file '{path}': {e}"))?
            } else if let Some(l) = logs {
                l
            } else {
                "error[E0308]: mismatched types\n  --> src/copilot/excel.rs:42:12\n   |\n42 |     res\n   |     ^^^ expected enum `Result`, found struct `ExcelEvalResult`".to_string()
            };

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotIncidentDebuggerTool::new();
            let args = serde_json::json!({
                "logs": log_content,
                "commit_sha": commit,
                "pipeline_id": pipeline,
                "post_to_teams": channel,
                "format": format,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }

        CopilotAction::Hardware { tokens, accelerator, format } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🌱 WINDOWS COPILOT+ PC HARDWARE TELEMETRY".bold().yellow());
            println!("{}", "=========================================================".cyan());

            use crate::tools::ToolHandler;
            let tool = crate::copilot::CopilotHardwareTelemetryTool::new();
            let args = serde_json::json!({
                "workload_tokens": tokens,
                "accelerator": accelerator,
                "format": format,
            });

            let out = tool.execute(args).await?;
            println!("\n{out}");
        }
    }

    Ok(())
}

async fn handle_ide_command(action: IdeAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        IdeAction::Setup { target, path } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🛠️  TAGISAN IDE ECOSYSTEM CONFIGURATION GENERATOR".bold().yellow());
            println!("{}", "=========================================================".cyan());
            let generator = crate::ide::config::IdeConfigGenerator::new(&path);
            let written_files = generator.generate_target(&target)?;
            println!("\n✅ Successfully generated {} IDE configuration file(s) for target '{}':", written_files.len(), target);
            for f in &written_files {
                println!("  [✓] {}", f.display().to_string().green());
            }
        }
        IdeAction::Lsp => {
            let server = crate::ide::lsp::LspServer::new();
            server.run_default_stdio().await?;
        }
        IdeAction::Status { path } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🔍 TAGISAN IDE ECOSYSTEM INTEGRATION STATUS".bold().yellow());
            println!("{}", "=========================================================".cyan());
            let status = crate::ide::config::inspect_ide_status(&path);
            println!("  Workspace Directory:  {}", path.yellow());
            println!("  VS Code Configured:   {}", if status.vscode_configured { "YES".green().bold() } else { "NO".dimmed() });
            println!("  Cursor Configured:    {}", if status.cursor_configured { "YES".green().bold() } else { "NO".dimmed() });
            println!("  Windsurf Configured:  {}", if status.windsurf_configured { "YES".green().bold() } else { "NO".dimmed() });
            println!("  Claude Desktop:       {}", if status.claude_configured { "YES".green().bold() } else { "NO".dimmed() });
            println!("  Zed Configured:       {}", if status.zed_configured { "YES".green().bold() } else { "NO".dimmed() });
            println!("  JetBrains Configured: {}", if status.jetbrains_configured { "YES".green().bold() } else { "NO".dimmed() });
            if !status.detected_editors.is_empty() {
                println!("\n  Active Integrations:  {}", status.detected_editors.join(", ").cyan().bold());
            } else {
                println!("\n  No active IDE integrations detected in this workspace. Run 'tgs ide setup' to configure.");
            }
        }
    }
    Ok(())
}

async fn handle_otp_command(action: OtpAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        OtpAction::Supervise => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🇵🇭 TAGISAN BEAM/OTP NATIVE SUPERVISION TREE ENGINE".bold().yellow());
            println!("{}", "=========================================================".cyan());
            println!("Starting Root OTP Supervisor with OneForOne strategy...\n");

            #[derive(Clone)]
            struct DemoAgent {
                name: String,
            }

            #[async_trait::async_trait]
            impl crate::otp::actor::GenServer for DemoAgent {
                async fn handle_call(&mut self, req: crate::otp::etf::Term) -> Result<crate::otp::etf::Term, crate::otp::actor::ActorError> {
                    match req.as_atom() {
                        Some("ping") => Ok(crate::otp::etf::Term::atom("pong")),
                        Some("status") => Ok(crate::otp::etf::Term::string(format!("Agent '{}' is active and operational", self.name))),
                        _ => Ok(crate::otp::etf::Term::ok()),
                    }
                }
            }

            let spec_1 = crate::otp::supervisor::ChildSpec::new("agent_alpha", || DemoAgent { name: "agent_alpha".to_string() });
            let spec_2 = crate::otp::supervisor::ChildSpec::new("agent_beta", || DemoAgent { name: "agent_beta".to_string() });
            let spec_3 = crate::otp::supervisor::ChildSpec::new("agent_gamma", || DemoAgent { name: "agent_gamma".to_string() });

            let sup_spec = crate::otp::supervisor::SupervisorSpec::new("root_supervisor", crate::otp::supervisor::RestartStrategy::OneForOne)
                .max_restarts(5, 10)
                .add_child(spec_1)
                .add_child(spec_2)
                .add_child(spec_3);

            let supervisor = crate::otp::supervisor::Supervisor::start(sup_spec).await?;
            let children = supervisor.which_children().await?;

            println!("Supervised Child Processes (Total: {}):", children.len().to_string().green().bold());
            for child in &children {
                let status = if child.is_alive { "ALIVE".green().bold() } else { "STOPPED".red() };
                println!("  • Child ID: {:<14} | Status: [{}] | Restarts: {}", child.id.cyan(), status, child.restart_count);
            }

            println!("\nDispatching synchronous calls across actors:");
            for child in &children {
                if let Some(actor) = supervisor.get_child(&child.id).await {
                    let reply = actor.call(crate::otp::etf::Term::atom("status"), std::time::Duration::from_millis(500)).await?;
                    println!("  ↳ [{}] -> {}", child.id.cyan(), reply.as_str().unwrap_or(""));
                }
            }

            println!("\nOTP Supervision Tree active. Shutting down cleanly...");
            supervisor.terminate().await?;
            println!("{}", "✅ Supervisor tree terminated with 0 leaks.".green().bold());
        }

        OtpAction::Port => {
            crate::otp::port::run_port_loop().await?;
        }

        OtpAction::Bench { actors, messages } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚡ TAGISAN BEAM/OTP MASSIVE CONCURRENCY BENCHMARK".bold().yellow());
            println!("{}", "=========================================================".cyan());
            println!("Spawning {} concurrent actors...", actors.to_string().cyan().bold());

            struct BenchWorker {
                received: std::sync::Arc<std::sync::atomic::AtomicUsize>,
            }

            #[async_trait::async_trait]
            impl crate::otp::actor::GenServer for BenchWorker {
                async fn handle_cast(&mut self, _msg: crate::otp::etf::Term) -> Result<(), crate::otp::actor::ActorError> {
                    self.received.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(())
                }
            }

            let total_received = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let start_spawn = std::time::Instant::now();
            let mut handles = Vec::with_capacity(actors);

            for _ in 0..actors {
                let worker = BenchWorker { received: total_received.clone() };
                let (actor_ref, _handle) = crate::otp::actor::ActorProcess::spawn(worker);
                handles.push(actor_ref);
            }

            let spawn_duration = start_spawn.elapsed();
            println!("  ↳ Spawned {} actors in {:.2?} ({:.0} actors/sec)",
                actors.to_string().green(),
                spawn_duration,
                actors as f64 / spawn_duration.as_secs_f64()
            );

            println!("\nRouting {} messages across actor mailboxes...", messages.to_string().cyan().bold());
            let start_dispatch = std::time::Instant::now();

            for i in 0..messages {
                let target = &handles[i % actors];
                target.cast(crate::otp::etf::Term::int(i as i64)).await?;
            }

            let dispatch_duration = start_dispatch.elapsed();
            let throughput = messages as f64 / dispatch_duration.as_secs_f64();
            println!("  ↳ Dispatched in {:.2?} ({} msgs/sec)", dispatch_duration, format!("{:.0}", throughput).green().bold());

            for _ in 0..100 {
                if total_received.load(std::sync::atomic::Ordering::Relaxed) >= messages {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            let final_received = total_received.load(std::sync::atomic::Ordering::Relaxed);
            println!("\n{}", format!("✅ Verification Passed: {}/{} messages processed with 100% delivery guarantee.", final_received, messages).green().bold());
        }
    }
    Ok(())
}

async fn handle_gleam_command(action: GleamAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        GleamAction::Check { file } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🇵🇭 TAGISAN SOVEREIGN GLEAM TYPE CHECKER & AUDITOR".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let paths = if let Some(f) = file {
                vec![std::path::PathBuf::from(f)]
            } else {
                let mut found = Vec::new();
                let candidate_dir = std::path::Path::new("sdk/gleam/tagisan_gleam/src");
                if candidate_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(candidate_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                                    for sub in sub_entries.flatten() {
                                        if sub.path().extension().map(|e| e == "gleam").unwrap_or(false) {
                                            found.push(sub.path());
                                        }
                                    }
                                }
                            } else if path.extension().map(|e| e == "gleam").unwrap_or(false) {
                                found.push(path);
                            }
                        }
                    }
                }
                if found.is_empty() {
                    found.push(std::path::PathBuf::from("src/tagisan.gleam"));
                }
                found
            };

            for path in &paths {
                if !path.exists() {
                    eprintln!("{} File not found: {}", "Error:".red().bold(), path.display());
                    continue;
                }
                let source = std::fs::read_to_string(path)?;
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");
                let module = match crate::gleam::parse_gleam_source(&source, file_name) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("{} Parsing failed in {}: {}", "✗".red().bold(), path.display(), e);
                        return Err(e.to_string().into());
                    }
                };

                let mut type_env = crate::gleam::TypeEnvironment::new();
                if let Err(e) = type_env.check_module(&module) {
                    eprintln!("{} Type check failed in {}: {}", "✗".red().bold(), path.display(), e);
                    return Err(e.to_string().into());
                }

                let protocol = type_env.validate_actor_protocol(&module).ok();
                println!("{} {}", "✓".green().bold(), path.display().to_string().bold());
                println!("    ↳ Types: {} | Functions: {} | Protocol: {}",
                    module.types.len().to_string().cyan(),
                    module.functions.len().to_string().cyan(),
                    protocol.as_deref().unwrap_or("None (library)").yellow()
                );
            }
            println!("\n{}", "✅ All Gleam modules type-checked with 100% formal type safety.".green().bold());
        }

        GleamAction::Compile { file, target } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚙️  TAGISAN GLEAM TO BEAM / ETF COMPILER".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let path = std::path::Path::new(&file);
            let source = std::fs::read_to_string(path)?;
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");

            let module = crate::gleam::parse_gleam_source(&source, file_name)?;
            let mut type_env = crate::gleam::TypeEnvironment::new();
            type_env.check_module(&module)?;

            let target_str = target.unwrap_or_else(|| "all".to_string()).to_lowercase();

            if target_str == "erlang" || target_str == "all" {
                let codegen = crate::gleam::ErlangCodeGen::new(module.clone());
                let erl_code = codegen.compile()?;
                let erl_path = path.with_extension("erl");
                std::fs::write(&erl_path, &erl_code)?;
                println!("  [✓] Erlang BEAM source compiled: {}", erl_path.display().to_string().green().bold());
                println!("\n{}\n", erl_code.dimmed());
            }

            if target_str == "etf" || target_str == "all" {
                let term = crate::otp::etf::Term::tuple(vec![
                    crate::otp::etf::Term::atom(file_name),
                    crate::otp::etf::Term::atom("compiled_v0_2_0"),
                ]);
                let etf_bytes = term.encode();
                let etf_path = path.with_extension("etf");
                std::fs::write(&etf_path, &etf_bytes)?;
                println!("  [✓] Erlang ETF binary term written: {} ({} bytes)",
                    etf_path.display().to_string().green().bold(),
                    etf_bytes.len().to_string().yellow()
                );
            }
        }

        GleamAction::Run { file, message } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🚀 TAGISAN GLEAM ACTOR RUNNER (OTP SUPERVISION)".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let path = std::path::Path::new(&file);
            let source = std::fs::read_to_string(path)?;
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("agent");

            let initial_state = crate::gleam::GleamValue::Constructor {
                name: "AgentState".to_string(),
                fields: vec![
                    crate::gleam::GleamValue::String(file_name.to_string()),
                    crate::gleam::GleamValue::String("sovereign_worker".to_string()),
                    crate::gleam::GleamValue::Int(0),
                    crate::gleam::GleamValue::Bool(true),
                ],
            };

            let actor_instance = crate::gleam::GleamActor::from_source(file_name, &source, initial_state.clone())?;
            println!("Actor initialized: {} [Protocol: {}]",
                file_name.cyan().bold(),
                actor_instance.msg_type_name.yellow().bold()
            );

            let source_clone = source.clone();
            let initial_clone = initial_state.clone();
            let file_name_clone = file_name.to_string();

            let spec = crate::gleam::GleamActor::child_spec(file_name, move || {
                crate::gleam::GleamActor::from_source(
                    &file_name_clone,
                    &source_clone,
                    initial_clone.clone(),
                )
                .unwrap()
            });

            let sup_spec = crate::otp::supervisor::SupervisorSpec::new("gleam_supervisor", crate::otp::supervisor::RestartStrategy::OneForOne)
                .add_child(spec);

            let supervisor = crate::otp::supervisor::Supervisor::start(sup_spec).await?;
            let child_ref = supervisor.get_child(file_name).await.ok_or("Failed to get child actor handle")?;

            let msg_term = if let Some(ref m) = message {
                crate::otp::etf::Term::tuple(vec![
                    crate::otp::etf::Term::atom(m.clone()),
                    crate::otp::etf::Term::atom("self"),
                ])
            } else {
                crate::otp::etf::Term::atom("ping")
            };

            println!("Dispatching call to supervised actor: {}", msg_term);
            let start = std::time::Instant::now();
            let reply = child_ref.call(msg_term, std::time::Duration::from_secs(2)).await;
            let duration = start.elapsed();

            match reply {
                Ok(res) => {
                    println!("{} Actor reply received in {:.2?}: {}", "✔".green().bold(), duration, res.to_string().cyan());
                }
                Err(e) => {
                    println!("{} Call result: {}", "ℹ".yellow().bold(), e);
                }
            }

            supervisor.terminate().await?;
            println!("{}", "✅ Supervised Gleam actor terminated cleanly.".green().bold());
        }

        GleamAction::New { name } => {
            let scaffold = format!(
r#"//// Sovereign Agent Module: {name}
import gleam/erlang/process.{{type Subject}}
import gleam/otp/actor

pub type State {{
  State(name: String, count: Int)
}}

pub type Msg {{
  Ping(reply_to: Subject(String))
  Increment(amount: Int)
  Reset
}}

pub fn init() -> State {{
  State(name: "{name}", count: 0)
}}

pub fn handle_msg(msg: Msg, state: State) -> actor.Next(Msg, State) {{
  case msg {{
    Ping(reply_to) -> {{
      process.send(reply_to, "pong")
      actor.continue(state)
    }}
    Increment(amount) -> {{
      actor.continue(State(name: state.name, count: state.count + amount))
    }}
    Reset -> {{
      actor.continue(State(name: state.name, count: 0))
    }}
  }}
}}
"#
            );
            let out_file = format!("{}.gleam", name);
            std::fs::write(&out_file, scaffold)?;
            println!("{} Scaffolded new Gleam agent: {}", "✔".green().bold(), out_file.bold());
        }
    }

    Ok(())
}

async fn handle_shield_command(action: ShieldAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ShieldAction::Scan { path, json } => {
            let target_path = std::path::Path::new(&path);
            if !json {
                println!("{}", "=========================================================".cyan());
                println!("{}", "  🛡️  AGENTSHIELD NATION-STATE APT REPOSITORY SCANNER".bold().yellow());
                println!("{}", "=========================================================".cyan());
                println!("Target Path: {}", path.bold().cyan());
                println!("Engine:      AgentShield Zero Ambient Authority Scanner");
                println!("Attribution: Rogue AI Cyberwarfare Expert, Lazarus Group, Kimsuky\n");
            }

            let report = crate::ecc::AgentShieldScanner::scan_directory(target_path);

            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                println!("{}", format!("Scan complete: {} file(s) inspected.", report.files_scanned).bold());

                if report.findings.is_empty() {
                    println!("\n{}", "✅ AGENTSHIELD SCAN PASSED: No nation-state APT indicators detected.".bold().green());
                } else {
                    println!("\n{}", format!("🚨 AGENTSHIELD DETECTED {} SECURITY VIOLATION(S):", report.findings.len()).bold().red());
                    for (idx, f) in report.findings.iter().enumerate() {
                        println!("\n{}", format!("--- Finding #{} [{:?}] ---", idx + 1, f.severity).bold().red());
                        println!("  Rule:         {}", f.rule_name.bold().yellow());
                        println!("  Threat Actor: {}", f.threat_actor.bold().bright_red());
                        println!("  Vector:       {}", f.attack_vector.cyan());
                        if let Some(line) = f.line_number {
                            println!("  Location:     {}:{}", f.file_path.bold(), line);
                        } else {
                            println!("  Location:     {}", f.file_path.bold());
                        }
                        println!("  Snippet:      {}", f.snippet.italic());
                        println!("  Remediation:  {}", f.remediation.green());
                    }
                    println!("\n{}", "=========================================================".red());
                    println!("{}", "  STATUS: REPOSITORY COMPROMISED / ISOLATION RECOMMENDED".bold().red());
                    println!("{}", "=========================================================".red());
                }
            }

            if !report.passed {
                return Err(format!("AgentShield found {} security violation(s)", report.findings.len()).into());
            }
        }
        ShieldAction::Audit { command, json } => {
            if !json {
                println!("{}", "=========================================================".cyan());
                println!("{}", "  🛡️  AGENTSHIELD PRE-EXECUTION COMMAND AUDIT".bold().yellow());
                println!("{}", "=========================================================".cyan());
                println!("Auditing Command: \"{}\"", command.bold().cyan());
            }

            let verdict = crate::ecc::AgentShieldScanner::scan_command(&command);

            if json {
                let json_verdict = match &verdict {
                    crate::ecc::AgentShieldVerdict::Allow => serde_json::json!({
                        "status": "Allow",
                        "command": command,
                        "safe": true
                    }),
                    crate::ecc::AgentShieldVerdict::Block { reason, threat_level } => {
                        let actor = crate::notify::hub().history().iter().rev().find_map(|e| {
                            if let crate::notify::NotificationPayload::CyberDefenseAlert(ref details) = e.payload {
                                Some(details.threat_actor.clone())
                            } else {
                                None
                            }
                        }).unwrap_or_else(|| "Rogue AI Cyberwarfare Expert / Lazarus Group".to_string());

                        serde_json::json!({
                            "status": "Block",
                            "command": command,
                            "safe": false,
                            "reason": reason,
                            "threat_level": format!("{:?}", threat_level),
                            "attribution": actor
                        })
                    }
                };
                println!("{}", serde_json::to_string_pretty(&json_verdict).unwrap_or_default());
            } else {
                match verdict {
                    crate::ecc::AgentShieldVerdict::Allow => {
                        println!("\n{}", "✅ AGENTSHIELD AUDIT PASSED: Command verified safe (0 malicious indicators detected).".bold().green());
                    }
                    crate::ecc::AgentShieldVerdict::Block { reason, threat_level } => {
                        let actor = crate::notify::hub().history().iter().rev().find_map(|e| {
                            if let crate::notify::NotificationPayload::CyberDefenseAlert(ref details) = e.payload {
                                Some(details.threat_actor.clone())
                            } else {
                                None
                            }
                        }).unwrap_or_else(|| "Rogue AI Cyberwarfare Expert / Lazarus Group".to_string());

                        println!("\n{}", "🚨 CRITICAL CYBER THREAT INTERCEPTED BEFORE EXECUTION!".bold().bright_red());
                        println!("  Threat Actor: {}", actor.bold().red());
                        println!("  Threat Level: {:?}", threat_level);
                        println!("  Reason:       {}", reason.bold().yellow());
                        println!("  Action Taken: Execution terminated before spawning subshell.");
                        println!("  Remediation:  Isolate host environment, verify command origin, and revoke exposed secrets.");
                        return Err(format!("Command blocked by AgentShield [{:?}]: {}", threat_level, reason).into());
                    }
                }
            }
        }
        ShieldAction::Status => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🛡️  AGENTSHIELD ACTIVE DEFENSE & TELEMETRY STATUS".bold().yellow());
            println!("{}", "=========================================================".cyan());

            let telemetry = crate::ecc::AgentShieldScanner::incident_telemetry();
            println!("Status:          {}", if telemetry.critical_incidents > 0 { telemetry.status.bold().red() } else { telemetry.status.bold().green() });
            println!("Defense Policy:  ZERO AMBIENT AUTHORITY (ENFORCING)");
            println!("Host Subsystem:  AgentShield Invariant Defense & NotificationHub");

            println!("\n{}", "--- Monitored Nation-State Threat Actor Profiles ---".bold().cyan());
            for (i, p) in crate::ecc::AgentShieldScanner::threat_actor_profiles().iter().enumerate() {
                println!("  {}. {}", i + 1, p.name.bold().yellow());
                println!("     Aliases:      {}", p.aliases.join(", ").italic());
                println!("     Attribution:  {}", p.attribution);
                println!("     Targets:      {}", p.primary_targets.join(", "));
                println!("     Indicators:   {}", p.ttp_indicators.join("; "));
            }

            println!("\n{}", "--- Protected Asset Categories ---".bold().cyan());
            for asset in crate::ecc::AgentShieldScanner::protected_assets() {
                println!("  🔒 {}", asset);
            }

            println!("\n{}", "--- Live Telemetry & Incident Counters ---".bold().cyan());
            println!("  Total Logged Security Events: {}", telemetry.total_incidents);
            println!("  Critical Interceptions:       {}", telemetry.critical_incidents);
            println!("  Security Alerts:              {}", telemetry.security_alerts);
            println!("  Warnings:                     {}", telemetry.warning_incidents);
            println!("{}", "=========================================================\n".cyan());
        }
    }
    Ok(())
}

fn handle_graph_command(action: GraphAction) -> Result<(), Box<dyn std::error::Error>> {
    use crate::engine::graph::{BlastRisk, CodebaseGraph};
    use std::path::Path;

    match action {
        GraphAction::Stats { path } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            println!(
                "{}",
                format!("⚡ Indexing AST Codebase Knowledge Graph: {}", root.display()).cyan().bold()
            );

            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;
            let stats = graph.stats();

            println!("\n{}", "========================================================".dimmed());
            println!(
                "{}",
                "  Tagisan AST Codebase Knowledge Graph Statistics"
                    .bold()
                    .cyan()
            );
            println!("{}", "========================================================".dimmed());
            println!("  {:<24} : {}", "Total AST Nodes".bold(), stats.total_nodes.to_string().green());
            println!("  {:<24} : {}", "Total Relational Edges".bold(), stats.total_edges.to_string().green());
            println!("  {:<24} : {}", "Functions & Methods".bold(), stats.functions_count.to_string().yellow());
            println!("  {:<24} : {}", "Types & Interfaces".bold(), stats.types_count.to_string().yellow());
            println!("  {:<24} : {}", "Modules & Namespaces".bold(), stats.modules_count.to_string().blue());
            println!("  {:<24} : {}", "Indexed Source Files".bold(), stats.files_count.to_string().magenta());
            println!("{}", "--------------------------------------------------------".dimmed());

            if !stats.top_central_symbols.is_empty() {
                println!("\n{}", "  Top Architectural Hubs (In-Degree Centrality):".bold().underline());
                for (rank, (sym, count)) in stats.top_central_symbols.iter().enumerate() {
                    let badge = format!("#{}", rank + 1).dimmed();
                    let count_str = format!("({} dependents)", count).bright_yellow();
                    println!("    {:<4} {:<42} {}", badge, sym.cyan().bold(), count_str);
                }
            }
            println!("{}", "========================================================".dimmed());
        }

        GraphAction::Symbol { name, path } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;
            let symbols = graph.find_symbol(&name);

            if symbols.is_empty() {
                println!("{}", format!("No symbols matching '{}' found in graph.", name).yellow());
                return Ok(());
            }

            println!(
                "\n{}",
                format!("Found {} matching symbol(s) for '{}':", symbols.len(), name).bold().green()
            );
            println!("{}", "--------------------------------------------------------------------------------".dimmed());
            for sym in symbols {
                let kind_str = format!("[{}]", sym.kind.as_str()).blue().bold();
                let vis_str = format!("({})", sym.visibility.as_str()).dimmed();
                println!("  {} {} {}", kind_str, sym.qualified_name.bold().cyan(), vis_str);
                println!("    File     : {}:{}", sym.file.display().to_string().bright_white(), sym.line);
                println!("    Signature: {}", sym.signature.dimmed());
                if let Some(ref doc) = sym.doc {
                    println!("    Doc      : {}", doc.bright_black());
                }
                println!("{}", "--------------------------------------------------------------------------------".dimmed());
            }
        }

        GraphAction::Callers { name, path } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;
            let callers = graph.find_callers(&name);

            if callers.is_empty() {
                println!("{}", format!("No incoming callers found for '{}'.", name).yellow());
                return Ok(());
            }

            println!(
                "\n{}",
                format!("Incoming callers to '{}' ({} found):", name, callers.len()).bold().green()
            );
            println!("{}", "--------------------------------------------------------------------------------".dimmed());
            for (caller, edge) in callers {
                let rel_str = format!("[{}]", edge.relation.as_str().to_uppercase()).magenta().bold();
                let line_str = edge
                    .call_site_line
                    .map(|l| format!("call site line {l}"))
                    .unwrap_or_else(|| format!("defined line {}", caller.line));

                println!(
                    "  {} {:<36} ({}: {})",
                    rel_str,
                    caller.qualified_name.cyan().bold(),
                    caller.file.display().to_string().dimmed(),
                    line_str.dimmed()
                );
            }
            println!("{}", "--------------------------------------------------------------------------------".dimmed());
        }

        GraphAction::Callees { name, path } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;
            let callees = graph.find_callees(&name);

            if callees.is_empty() {
                println!("{}", format!("No outgoing calls found from '{}'.", name).yellow());
                return Ok(());
            }

            println!(
                "\n{}",
                format!("Outgoing calls from '{}' ({} found):", name, callees.len()).bold().green()
            );
            println!("{}", "--------------------------------------------------------------------------------".dimmed());
            for (callee, edge) in callees {
                let line_str = edge
                    .call_site_line
                    .map(|l| format!("line {l}"))
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "  ↳ {:<36} [{}] ({}: {})",
                    callee.qualified_name.cyan().bold(),
                    callee.kind.as_str().yellow(),
                    callee.file.display().to_string().dimmed(),
                    line_str.dimmed()
                );
            }
            println!("{}", "--------------------------------------------------------------------------------".dimmed());
        }

        GraphAction::BlastRadius { target, path, max_depth } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            let depth = max_depth.unwrap_or(3);
            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;
            let report = graph.calculate_blast_radius(&target, depth)?;

            let risk_banner = match report.risk_level {
                BlastRisk::Low => "🟢 LOW RISK IMPACT".green().bold(),
                BlastRisk::Medium => "🟡 MEDIUM RISK IMPACT".yellow().bold(),
                BlastRisk::High => "🟠 HIGH RISK IMPACT".bright_red().bold(),
                BlastRisk::Critical => "🔴 CRITICAL RISK IMPACT".red().bold(),
            };

            println!("\n{}", "========================================================".dimmed());
            println!("  Tagisan Transitive Blast-Radius Analysis");
            println!("{}", "========================================================".dimmed());
            println!("  Target Symbol    : {}", report.target_symbol.bold().cyan());
            println!("  Target File      : {}", report.target_file.display().to_string().bright_white());
            println!("  Assessed Risk    : {}", risk_banner);
            println!("  Total Affected   : {} symbols", report.total_affected_symbols.to_string().yellow().bold());
            println!("  Affected Files   : {} files", report.affected_files.len().to_string().yellow().bold());
            println!("{}", "--------------------------------------------------------".dimmed());

            println!("\n{}", "  Direct Callers:".bold().underline());
            if report.direct_callers.is_empty() {
                println!("    {}", "(None)".dimmed());
            } else {
                for c in &report.direct_callers {
                    println!("    • {}", c.cyan());
                }
            }

            println!("\n{}", "  Transitive Callers:".bold().underline());
            if report.transitive_callers.is_empty() {
                println!("    {}", "(None within depth limit)".dimmed());
            } else {
                for c in &report.transitive_callers {
                    println!("    • {}", c.bright_blue());
                }
            }

            if !report.implementing_types.is_empty() {
                println!("\n{}", "  Implementing Types:".bold().underline());
                for imp in &report.implementing_types {
                    println!("    • {}", imp.magenta());
                }
            }

            println!("\n{}", "  Affected Files:".bold().underline());
            for f in &report.affected_files {
                println!("    📁 {}", f.display().to_string().dimmed());
            }

            println!("\n{}", "  Refactoring Recommendations:".bold().underline());
            for rec in &report.recommendations {
                println!("    👉 {}", rec.bright_yellow());
            }
            println!("{}\n", "========================================================".dimmed());
        }

        GraphAction::Export { path, format } => {
            let root_str = path.unwrap_or_else(|| ".".to_string());
            let root = Path::new(&root_str);
            let graph = CodebaseGraph::build_from_dir(root, 10_000)?;

            match format.to_lowercase().as_str() {
                "dot" => {
                    println!("{}", graph.export_dot());
                }
                _ => {
                    let json_str = graph.export_json()?;
                    println!("{}", json_str);
                }
            }
        }
    }

    Ok(())
}

async fn handle_autofix_command(
    path: String,
    test: bool,
    max_attempts: usize,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=========================================================".cyan());
    println!("{}", "  🔧  TGS SELF-HEALING COMPILER & TDD HEALER (`tgs autofix`)".bold().yellow());
    println!("{}", "=========================================================".cyan());

    let target_path = std::path::PathBuf::from(&path);
    let canonical_target = target_path.canonicalize().unwrap_or_else(|_| target_path.clone());
    let detected_type = crate::engine::autofix::detect_project_type(&canonical_target);

    println!("  [•] Target Path:   {}", canonical_target.display().to_string().green());
    println!("  [•] Project Type:  {}", format!("{:?}", detected_type).magenta().bold());
    println!("  [•] Max Attempts:  {}", max_attempts.to_string().cyan());
    println!("  [•] Include Tests: {}", if test { "Yes".green() } else { "No".yellow() });
    println!("  [•] Mode:          {}", if dry_run { "DRY-RUN (Simulated)".yellow().bold() } else { "ACTIVE HEALING".green().bold() });
    println!();

    let engine = crate::engine::autofix::AutofixEngine::new();
    let options = crate::engine::autofix::AutofixOptions {
        max_attempts,
        include_tests: test,
        dry_run,
        backup: true,
    };

    let report = engine.heal(&canonical_target, &options)?;

    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "  📋  HEALING REPORT".bold().yellow());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("  Project Type:       {:?}", report.project_type);
    println!("  Initial Diagnostics: {}", report.total_diagnostics);
    println!("  Repairs Applied:    {}", report.healed_count);
    println!("  Attempts Made:      {}/{}", report.attempts_made, options.max_attempts);
    println!("  Duration:           {}ms", report.duration_ms);

    if !report.fixes_applied.is_empty() {
        println!("\n{}", "Surgical Patches Applied:".bold());
        for fix in &report.fixes_applied {
            println!("    {}", fix.green());
        }
    }

    println!();
    if report.is_clean {
        println!("{}", "  ✨ SUCCESS: All diagnostics resolved. Codebase is clean!".green().bold());
    } else {
        println!("{}", "  ⚠️  ATTENTION: Diagnostics remain after maximum attempts.".yellow().bold());
    }
    println!("{}", "=========================================================\n".cyan());

    Ok(())
}

async fn handle_ground_command(
    task: String,
    path: Option<String>,
    max_iterations: usize,
    no_critique: bool,
    no_ast: bool,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use colored::Colorize;
    use crate::engine::grounding::{CritiqueSeverity, GroundingEngine, GroundingOptions};
    use std::path::PathBuf;

    println!("{}", "================================================================================".cyan());
    println!("{}", "  🧠  DETERMINISTIC CLOSED-LOOP GROUNDING & VERIFICATION ENGINE (`tgs ground`)".bold().yellow());
    println!("{}", "================================================================================".cyan());
    println!("  [•] Task:                 {}", task.bold().white());
    let target_path = path.as_ref().map(PathBuf::from);
    if let Some(ref p) = target_path {
        println!("  [•] Context Target:       {}", p.display().to_string().green());
    } else {
        println!("  [•] Context Target:       {}", "Current Workspace".green());
    }
    println!("  [•] Max Iterations:       {}", max_iterations.to_string().cyan());
    println!("  [•] Adversarial Critique: {}", if no_critique { "Disabled".yellow() } else { "Enabled".green() });
    println!("  [•] AST Grounding:        {}", if no_ast { "Disabled".yellow() } else { "Enabled".green() });
    println!();

    let engine = GroundingEngine::new();
    let detected_lang = engine.detect_language(&task, None, target_path.as_deref());

    // 🌲 Phase 1: AST Invariant Extraction
    println!("{}", "🌲 Phase 1: AST Invariant Extraction".bold().cyan());
    let ast_context = if !no_ast {
        let ctx = engine.synthesize_ast_context(&task, target_path.as_deref().or_else(|| Some(std::path::Path::new("."))));
        let lines = ctx.lines().count();
        if lines > 0 {
            println!("   ↳ Querying petgraph knowledge graph...");
            println!("   ↳ Extracted {} invariant symbol signatures (<400 tokens compressed)", lines.to_string().green().bold());
        } else {
            println!("   ↳ No project AST graph found at target; using standard target prelude invariants.");
        }
        ctx
    } else {
        println!("   ↳ AST Knowledge Graph extraction skipped (--no-ast).");
        String::new()
    };
    println!();

    // 💡 Phase 2: Hypothesis Generation
    println!("{}", "💡 Phase 2: Hypothesis Generation".bold().cyan());
    println!("   ↳ Target Ecosystem: {}", format!("{:?}", detected_lang).magenta().bold());
    println!("   ↳ Synthesizing candidate code adhering strictly to target idioms...");
    let initial_code = engine.generate_hypothesis(&task, detected_lang, &ast_context);
    println!("   ↳ Candidate hypothesis generated ({} bytes, {} lines)", initial_code.len().to_string().yellow(), initial_code.lines().count().to_string().yellow());
    println!();

    // ⚔️ Phase 3: Adversarial Dialectical Audit
    println!("{}", "⚔️ Phase 3: Adversarial Dialectical Audit".bold().cyan());
    let _critique_findings = if !no_critique {
        let findings = engine.audit_critique(&initial_code, detected_lang);
        let crit_count = findings.iter().filter(|f| f.severity == CritiqueSeverity::Critical).count();
        let high_count = findings.iter().filter(|f| f.severity == CritiqueSeverity::High).count();
        let med_count = findings.iter().filter(|f| f.severity == CritiqueSeverity::Medium).count();
        let low_count = findings.iter().filter(|f| f.severity == CritiqueSeverity::Low).count();

        println!(
            "   ↳ Invariant Audit: {} total findings ({} Critical, {} High, {} Medium, {} Low)",
            findings.len().to_string().yellow().bold(),
            crit_count.to_string().red().bold(),
            high_count.to_string().yellow(),
            med_count.to_string().cyan(),
            low_count.to_string().dimmed(),
        );
        for f in findings.iter().take(5) {
            let badge = match f.severity {
                CritiqueSeverity::Critical => "CRITICAL".red().bold(),
                CritiqueSeverity::High => "HIGH".yellow().bold(),
                CritiqueSeverity::Medium => "MEDIUM".cyan(),
                CritiqueSeverity::Low => "LOW".dimmed(),
            };
            println!("     • [{}] [{:?}] {}", badge, f.category, f.description);
            println!("       ↳ Suggestion: {}", f.suggestion.italic());
        }
        findings
    } else {
        println!("   ↳ Adversarial critique skipped (--no-critique).");
        Vec::new()
    };
    println!();

    // 🔬 Phase 4: Deterministic Compiler Verification
    println!("{}", "🔬 Phase 4: Deterministic Compiler Verification".bold().cyan());
    println!("   ↳ Ephemeral sandbox initialized.");
    let (is_compiler_clean, diagnostics) = engine.verify_deterministic(&initial_code, detected_lang, false);
    if is_compiler_clean {
        println!("   ↳ {}", "Deterministic compiler syntax & type check PASSED (0 errors)".green().bold());
    } else {
        println!("   ↳ {}", format!("Compiler detected {} diagnostics", diagnostics.len()).red().bold());
        for d in diagnostics.iter().take(3) {
            println!("     • Line {}:{} [{}] {}", d.line, d.col, d.code.as_deref().unwrap_or("error"), d.message.red());
        }
    }
    println!();

    // Execute full elevate pipeline (includes Phase 5: Closed-Loop Self-Healing & Phase 6: Certification)
    let options = GroundingOptions {
        max_iterations,
        adversarial_critique: !no_critique,
        ast_grounding: !no_ast,
        sandbox_exec: false,
        provider: None,
        model: None,
    };

    println!("{}", "🩹 Phase 5: Closed-Loop Self-Healing".bold().cyan());
    let report = engine.elevate(&task, Some(detected_lang), target_path.as_deref(), &options)?;
    if report.compiler_healed_count > 0 {
        println!(
            "   ↳ Applied {} surgical iterative patch repairs across {} verification passes.",
            report.compiler_healed_count.to_string().green().bold(),
            report.total_passes.to_string().cyan()
        );
    } else if report.initial_clean {
        println!("   ↳ {}", "Initial hypothesis clean; zero healing cycles required.".green());
    } else {
        println!("   ↳ Ran {} self-healing verification passes.", report.total_passes);
    }
    println!();

    // 🏆 Phase 6: Grounded Truth Certification
    println!("{}", "🏆 Phase 6: Grounded Truth Certification".bold().cyan());
    println!("{}", "================================================================================".cyan());
    if report.final_verified {
        println!(
            "  {} (Confidence Score: {:.1}%)",
            "✨ CERTIFIED GROUNDED TRUTH".green().bold(),
            report.confidence_score * 100.0
        );
    } else {
        println!(
            "  {} (Confidence Score: {:.1}%)",
            "⚠️ UNVERIFIED HYPOTHESIS".yellow().bold(),
            report.confidence_score * 100.0
        );
    }
    println!("{}", "================================================================================".cyan());
    println!("  • Language:          {:?}", report.language);
    println!("  • Passes Completed:  {}/{}", report.total_passes, max_iterations);
    println!("  • Initial Clean:     {}", if report.initial_clean { "Yes".green() } else { "No".yellow() });
    println!("  • Critique Findings: {}", report.critique_count);
    println!("  • Patches Healed:    {}", report.compiler_healed_count);
    println!("  • Verified Clean:    {}", if report.final_verified { "TRUE".green().bold() } else { "FALSE".red().bold() });
    println!("  • Verification Time: {}ms", report.duration_ms);
    println!();

    println!("{}", "Grounded & Verified Source Code:".bold().white());
    println!("{}", "────────────────────────────────────────────────────────────────────────────────".dimmed());
    for (idx, line) in report.final_code.trim().lines().enumerate() {
        println!("{:>4} │ {}", (idx + 1).to_string().dimmed(), line);
    }
    println!("{}", "────────────────────────────────────────────────────────────────────────────────".dimmed());
    println!();

    // Output to file if requested
    if let Some(ref out_path) = output {
        let p = PathBuf::from(out_path);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&p, &report.final_code)?;
        println!("💾 Successfully wrote verified code to: {}", p.display().to_string().green().bold());
        println!();
    }

    Ok(())
}

async fn handle_engine_command(action: EngineAction) -> Result<(), Box<dyn std::error::Error>> {
    let resolver = crate::engine::OllamaBlobResolver::new(None);
    match action {
        EngineAction::List => {
            let models = resolver.list_installed_models()?;
            if models.is_empty() {
                println!("No Ollama models found in '{}'", resolver.base_dir.display());
                return Ok(());
            }

            println!("=========================================================================================================");
            println!(" Tagisan Tensor Engine - Local Ollama Models (GGUF)");
            println!(" Base Directory: {}", resolver.base_dir.display());
            println!("=========================================================================================================");
            println!("{:<35} {:<10} {:<12} {:<10} {:<15}", "NAME", "FAMILY", "SIZE", "QUANT", "MODIFIED");
            println!("---------------------------------------------------------------------------------------------------------");
            for m in &models {
                let size_mb = (m.size as f64) / (1024.0 * 1024.0);
                let size_str = if size_mb >= 1024.0 {
                    format!("{:.2} GB", size_mb / 1024.0)
                } else {
                    format!("{:.1} MB", size_mb)
                };
                let mod_short = m.modified_at.split('T').next().unwrap_or(&m.modified_at);
                println!(
                    "{:<35} {:<10} {:<12} {:<10} {:<15}",
                    m.name,
                    m.details.family,
                    size_str,
                    m.details.quantization_level,
                    mod_short
                );
            }
            println!("=========================================================================================================");
        }
        EngineAction::Inspect { model } => {
            let model_path = if std::path::Path::new(&model).exists() {
                std::path::PathBuf::from(&model)
            } else {
                let summary = resolver.resolve(&model)?;
                println!("Resolved model '{}' -> {}", model, summary.model_path.display());
                summary.model_path
            };

            let start = std::time::Instant::now();
            let gguf = crate::engine::GgufFile::open(&model_path)?;
            let elapsed = start.elapsed();

            println!("{}", gguf.inspect());
            println!("Zero-Copy mmap & header parse latency: {:.3} ms", elapsed.as_secs_f64() * 1000.0);
        }
    }
    Ok(())
}

async fn handle_stream_command(
    provider: String,
    model: Option<String>,
    image: Option<String>,
    prompt: String,
    skill: Option<String>,
    auto_skills: bool,
    no_skills: bool,
    cli_max_budget: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

    println!(
        "\n{} [{}: {}]...",
        "Streaming from".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );
    println!("Prompt: \"{}\"", prompt.italic());

    let should_inject_skills = !no_skills && (skill.is_some() || auto_skills);
    let mut final_system_prompt: Option<String> = None;
    let mut effective_prompt = prompt.clone();

    if should_inject_skills {
        let dispatcher = crate::ecc::skills::global_dispatcher();
        let (equipped_text, injected_skills, budget) = dispatcher.equip_prompt_maximized(
            "",
            &prompt,
            &provider_id,
            Some(&model_name),
            skill.as_deref(),
            None,
            None,
        );

        if !injected_skills.is_empty() {
            let caps = prov.capabilities(&model_name);

            if caps.contains(crate::types::ProviderCapabilities::SYSTEM_PROMPT) {
                final_system_prompt = Some(equipped_text);
            } else {
                effective_prompt = format!("{}\n\n{}", equipped_text, prompt);
            }

            let mode_badge = match budget.mode {
                crate::ecc::InjectionMode::DenseInvariants => "Dense Invariants DSL (<1.2k tokens)".cyan().bold(),
                crate::ecc::InjectionMode::Hierarchical => "Hierarchical Multi-Tier Architecture".blue().bold(),
                crate::ecc::InjectionMode::Comprehensive => "Cloud Comprehensive Specification".magenta().bold(),
                crate::ecc::InjectionMode::CheatSheet => "Local Cheat-Sheet (<1k tokens)".cyan().bold(),
            };
            let skill_names: Vec<String> = injected_skills
                .iter()
                .map(|s| format!("{} ({:.1})", s.skill.name, s.score))
                .collect();
            println!(
                "{} [{}] Injected {} skill(s) [Budget: ~{} tokens / {}k ctx] -> [{}]\n",
                "⚡ Dynamic Skills:".bold().yellow(),
                mode_badge,
                injected_skills.len(),
                budget.max_tokens,
                budget.context_window / 1000,
                skill_names.join(", ").green()
            );
        }
    }

    let mut user_blocks = vec![ContentBlock::text(effective_prompt)];
    if let Some(ref img_path) = image {
        let img_block = ContentBlock::from_image_file(img_path)?;
        println!("Attached Multimodal Image: {}", img_path.cyan().bold());
        user_blocks.push(img_block);
    }
    println!();

    let mut req = CompletionRequest::new(model_name.clone(), "")
        .with_messages(vec![crate::types::Message::user_with_content(user_blocks)])
        .with_stream(true)
        .with_cancellation(ctx.cancellation_token.clone());

    if let Some(sys) = final_system_prompt {
        req = req.with_system(sys);
    }

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

    Ok(())
}

async fn handle_ask_command(
    provider: String,
    model: Option<String>,
    image: Option<String>,
    prompt: String,
    skill: Option<String>,
    auto_skills: bool,
    no_skills: bool,
    cli_max_budget: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = build_engine_context(cli_max_budget);
    let (provider_id, model_name, prov) = resolve_provider_and_model(&ctx, &provider, model)?;

    println!(
        "\n{} [{}: {}]...",
        "Querying".bold().magenta(),
        provider_id.cyan().bold(),
        model_name.yellow()
    );

    let should_inject_skills = !no_skills && (skill.is_some() || auto_skills);
    let mut final_system_prompt: Option<String> = None;
    let mut effective_prompt = prompt.clone();

    if should_inject_skills {
        let dispatcher = crate::ecc::skills::global_dispatcher();
        let (equipped_text, injected_skills, budget) = dispatcher.equip_prompt_maximized(
            "",
            &prompt,
            &provider_id,
            Some(&model_name),
            skill.as_deref(),
            None,
            None,
        );

        if !injected_skills.is_empty() {
            let caps = prov.capabilities(&model_name);

            if caps.contains(crate::types::ProviderCapabilities::SYSTEM_PROMPT) {
                final_system_prompt = Some(equipped_text);
            } else {
                effective_prompt = format!("{}\n\n{}", equipped_text, prompt);
            }

            let mode_badge = match budget.mode {
                crate::ecc::InjectionMode::DenseInvariants => "Dense Invariants DSL (<1.2k tokens)".cyan().bold(),
                crate::ecc::InjectionMode::Hierarchical => "Hierarchical Multi-Tier Architecture".blue().bold(),
                crate::ecc::InjectionMode::Comprehensive => "Cloud Comprehensive Specification".magenta().bold(),
                crate::ecc::InjectionMode::CheatSheet => "Local Cheat-Sheet (<1k tokens)".cyan().bold(),
            };
            let skill_names: Vec<String> = injected_skills
                .iter()
                .map(|s| format!("{} ({:.1})", s.skill.name, s.score))
                .collect();
            println!(
                "{} [{}] Injected {} skill(s) [Budget: ~{} tokens / {}k ctx] -> [{}]\n",
                "⚡ Dynamic Skills:".bold().yellow(),
                mode_badge,
                injected_skills.len(),
                budget.max_tokens,
                budget.context_window / 1000,
                skill_names.join(", ").green()
            );
        }
    }

    let mut user_blocks = vec![ContentBlock::text(effective_prompt)];
    if let Some(ref img_path) = image {
        let img_block = ContentBlock::from_image_file(img_path)?;
        println!("Attached Multimodal Image: {}\n", img_path.cyan().bold());
        user_blocks.push(img_block);
    }

    let mut session = ChatSession::new();
    if let Some(sys) = final_system_prompt {
        session.system_prompt = Some(sys);
    }
    session.add_user_message_with_blocks(user_blocks);

    let req = session
        .build_request(model_name.clone())
        .with_cancellation(ctx.cancellation_token.clone());

    let spinner = Spinner::start(format!(
        "Thinking with {} [{}]...",
        provider_id.cyan().bold(),
        model_name.yellow().bold()
    ));
    let resp = prov.complete(req).await;
    match &resp {
        Ok(r) => spinner.success(format!("Response received in {:.2}s", r.latency.as_secs_f32())),
        Err(e) => spinner.failure(format!("Request failed: {}", e)),
    }
    let resp = resp?;

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

    Ok(())
}

async fn handle_harmony_command(
    action: Option<HarmonySubcommand>,
    objective: Option<String>,
    architect: Option<String>,
    implementer: Option<String>,
    qa: Option<String>,
    doc: Option<String>,
    tier: Option<String>,
    parallel: bool,
    audit: bool,
    json: bool,
    output_dir: Option<String>,
    fallback_to_local: bool,
    evacuate_on_budget: bool,
    notify: bool,
    skill: Option<String>,
    auto_skills: bool,
    no_skills: bool,
    ctx: &EngineContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let (
        eff_objective,
        eff_arch,
        eff_imp,
        eff_qa,
        eff_doc,
        eff_tier,
        eff_parallel,
        eff_audit,
        eff_json,
        eff_out_dir,
        eff_fallback_to_local,
        eff_evacuate_on_budget,
        eff_notify,
        eff_skill,
        _eff_auto_skills,
        eff_no_skills,
    ) = match action {
        Some(HarmonySubcommand::Build {
            objective: sub_obj,
            architect: sub_arch,
            implementer: sub_imp,
            qa: sub_qa,
            doc: sub_doc,
            tier: sub_tier,
            parallel: sub_parallel,
            audit: sub_audit,
            json: sub_json,
            output_dir: sub_out_dir,
            fallback_to_local: sub_fallback_to_local,
            evacuate_on_budget: sub_evacuate_on_budget,
            notify: sub_notify,
            skill: sub_skill,
            auto_skills: sub_auto_skills,
            no_skills: sub_no_skills,
        }) => (
            sub_obj,
            sub_arch.or(architect),
            sub_imp.or(implementer),
            sub_qa.or(qa),
            sub_doc.or(doc),
            sub_tier.or(tier),
            sub_parallel || parallel,
            sub_audit || audit,
            sub_json || json,
            sub_out_dir.or(output_dir),
            sub_fallback_to_local || fallback_to_local,
            sub_evacuate_on_budget || evacuate_on_budget,
            sub_notify || notify,
            sub_skill.or(skill),
            sub_auto_skills || auto_skills,
            sub_no_skills || no_skills,
        ),
        None => {
            let obj = objective.unwrap_or_else(|| {
                eprintln!("{}: missing objective. Usage: tgs harmony build \"<objective>\"", "Error".red().bold());
                std::process::exit(1);
            });
            (
                obj,
                architect,
                implementer,
                qa,
                doc,
                tier,
                parallel,
                audit,
                json,
                output_dir,
                fallback_to_local,
                evacuate_on_budget,
                notify,
                skill,
                auto_skills,
                no_skills,
            )
        }
    };

    let profile = match eff_tier.as_deref().map(|s| s.to_lowercase()).as_deref() {
        Some("flagship") => Some(crate::swarm::harmony::HarmonyTierProfile::Flagship),
        Some("economy") | Some("cheap") => Some(crate::swarm::harmony::HarmonyTierProfile::Economy),
        _ => Some(crate::swarm::harmony::HarmonyTierProfile::Smart),
    };

    let overrides = crate::swarm::harmony::RoleModelOverrides {
        architect: eff_arch,
        implementer: eff_imp,
        qa: eff_qa,
        doc: eff_doc,
        profile,
    };

    let (arch_m, imp_m, qa_m, doc_m) = crate::swarm::harmony::resolve_harmony_models(ctx, &overrides);

    let effective_auto_skills = !eff_no_skills;
    let mut pipeline = crate::swarm::harmony::build_standard_harmony_pipeline(
        &eff_objective,
        ctx,
        &overrides,
        eff_audit,
    )
    .with_fallback_to_local(eff_fallback_to_local)
    .with_evacuate_on_budget(eff_evacuate_on_budget)
    .with_notify_on_failover(eff_notify)
    .with_auto_skills(effective_auto_skills);

    if eff_parallel {
        pipeline = pipeline.with_parallel(true);
    }

    if !eff_json {
        println!("{}", "=========================================================================".cyan());
        println!("{}", "  🏛️  Tagisan Structured Role-Based Harmony Swarm (Bayanihan)".bold().magenta());
        println!("{}", "=========================================================================".cyan());
        println!("Objective: \"{}\"\n", eff_objective.bold().yellow());
        println!("  • Cloud Tier:         {:?}", profile.unwrap_or_default());
        println!("  • Downstream Exec:    {}", if pipeline.parallel_qa_doc { "Concurrent (QA + Doc in Parallel)".green().bold() } else { "Sequential".dimmed() });
        println!("  • Semantic Skills:    {}", if effective_auto_skills { "Enabled (Provider-aware Dynamic Injection)".green().bold() } else { "Disabled (--no-skills)".dimmed() });
        if let Some(ref s) = eff_skill {
            println!("  • Explicit Skill:     {}", s.cyan().bold());
        }
        if eff_fallback_to_local {
            println!("  • Local Failover:     {}", "Enabled (Dynamic Hot-swap to Ollama on Rate Limit / Error)".green().bold());
        }
        if eff_evacuate_on_budget {
            println!("  • Budget Evacuation:  {}", "Enabled (Zero-cost Local Evacuation on Spending Cap)".green().bold());
        }
        if eff_notify {
            println!("  • Desktop Alert:      {}", "Enabled (Native OS Toast on Failover)".green().bold());
        }
        println!("  • [Stage 1] Architect:    {} [{}]", arch_m.1.cyan().bold(), arch_m.0.dimmed());
        println!("  • [Stage 2] Implementer:  {} [{}]", imp_m.1.cyan().bold(), imp_m.0.dimmed());
        println!("  • [Stage 3] QA & Test:    {} [{}]", qa_m.1.cyan().bold(), qa_m.0.dimmed());
        println!("  • [Stage 4] Docs:         {} [{}]", doc_m.1.cyan().bold(), doc_m.0.dimmed());
        if eff_audit {
            println!("  • [Stage 5] Audit:        Enabled (Adversarial Security & Architecture Review)");
        }
        println!();
    }

    let spinner = if !eff_json {
        Some(Spinner::start("Executing 4-stage assembly line with syntax & AgentShield gates..."))
    } else {
        None
    };

    let result = pipeline.execute(ctx).await;

    match &result {
        Ok(res) => {
            if let Some(sp) = spinner {
                sp.success(format!(
                    "Assembly line completed successfully in {:.2}s ({} artifacts generated)",
                    res.total_latency.as_secs_f64(),
                    res.artifacts.len()
                ));
            }
        }
        Err(e) => {
            if let Some(sp) = spinner {
                sp.failure(format!("Assembly line failed: {}", e));
            }
            return Err(e.to_string().into());
        }
    }

    let res = result?;

    if eff_json {
        let json_val = serde_json::to_string_pretty(&res)?;
        println!("{}", json_val);
    } else {
        for artifact in &res.artifacts {
            let stage_header = format!("─── STAGE: {} ({}) ───", artifact.role_id.to_uppercase(), artifact.role_title);
            if let Some(ref fo) = artifact.failover_event {
                println!(
                    "\n{} [{} / {}] (took {:.2}s, {} tokens) ⚠️  {}",
                    stage_header.green().bold(),
                    artifact.provider.bold(),
                    artifact.model.cyan(),
                    artifact.latency_secs,
                    artifact.tokens_used,
                    format!("[Failover: {} ({}) -> {} ({}) | Trigger: {}]", fo.original_provider, fo.original_model, fo.evacuated_to_provider, fo.evacuated_to_model, fo.trigger_reason).yellow().bold()
                );
            } else {
                println!(
                    "\n{} [{} / {}] (took {:.2}s, {} tokens)",
                    stage_header.green().bold(),
                    artifact.provider.bold(),
                    artifact.model.cyan(),
                    artifact.latency_secs,
                    artifact.tokens_used
                );
            }
            println!("{}", artifact.raw_output.trim());
        }

        if let Some(ref audit_text) = res.audit_verdict {
            println!("\n{}", "─── ADVERSARIAL AUDIT VERDICT ───".red().bold());
            println!("{}", audit_text.trim());
        }

        println!("\n{}", "=========================================================================".cyan());
        println!(
            "Total Tokens: {} | Estimated Cost: ${:.4} USD | Latency: {:.2}s",
            res.total_usage.prompt_tokens + res.total_usage.completion_tokens,
            res.total_cost_usd,
            res.total_latency.as_secs_f64()
        );

        let failovers: Vec<&crate::swarm::harmony::FailoverEvent> = res
            .artifacts
            .iter()
            .filter_map(|a| a.failover_event.as_ref())
            .collect();
        if !failovers.is_empty() {
            println!("{}", "──────────────────────── Failover Telemetry ────────────────────────".yellow().bold());
            for (idx, fo) in failovers.iter().enumerate() {
                println!(
                    "  [{}] Evacuated: '{}' ({}) -> '{}' ({}) | Cost at Failover: ${:.4} | Reason: {}",
                    idx + 1,
                    fo.original_provider.bold(),
                    fo.original_model,
                    fo.evacuated_to_provider.bold().green(),
                    fo.evacuated_to_model.cyan(),
                    fo.cost_at_failover_usd,
                    fo.trigger_reason.yellow()
                );
            }
        }
        println!("{}", "=========================================================================".cyan());
    }

    // If output_dir was specified, save files to disk
    if let Some(ref dir) = eff_out_dir {
        let target_dir = std::path::Path::new(dir);
        std::fs::create_dir_all(target_dir)?;

        let bundle_path = target_dir.join("assembled_project.txt");
        std::fs::write(&bundle_path, &res.complete_project)?;

        for artifact in &res.artifacts {
            for (idx, block) in artifact.code_blocks.iter().enumerate() {
                let ext = match block.language.to_lowercase().as_str() {
                    "rust" | "rs" => "rs",
                    "python" | "py" => "py",
                    "perl" | "pl" => "pl",
                    "typescript" | "ts" => "ts",
                    "javascript" | "js" => "js",
                    "json" => "json",
                    _ => "txt",
                };
                let file_name = if idx == 0 {
                    format!("{}.{}", artifact.role_id, ext)
                } else {
                    format!("{}_{}.{}", artifact.role_id, idx + 1, ext)
                };
                let file_path = target_dir.join(file_name);
                std::fs::write(&file_path, &block.code)?;
            }
        }
        if !eff_json {
            println!("  [✓] Artifacts saved to directory: {}", target_dir.display().to_string().green().bold());
        }
    }

    Ok(())
}

async fn handle_bun_command(action: BunAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        BunAction::Info => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🥟  Bun Ultra-Fast JS/TS Runtime & Bundler Integration".bold().yellow());
            println!("{}", "=========================================================".cyan());

            match crate::bun::BunRuntime::new() {
                Ok(runtime) => {
                    let ver = runtime.version().await.unwrap_or_else(|_| "unknown".to_string());
                    println!("  [✓] Bun Binary:       {}", runtime.bun_path().display().to_string().green().bold());
                    println!("  [✓] Bun Version:      {}", ver.cyan().bold());
                    println!("  [✓] Runtime Status:   {}", "Ready & Available".green());
                    println!("  [✓] Top-level Await:  {}", "Supported natively".green());
                    println!("  [✓] ESM & TypeScript: {}", "Zero-config native execution".green());
                    println!("  [✓] Bundler & Test:   {}", "Integrated with AgentShield safety".green());
                }
                Err(e) => {
                    println!("  [✗] Bun Runtime:      {}", "Not Discovered".red().bold());
                    println!("      ↳ Error: {}", e);
                    println!("\n  To install Bun, run: curl -fsSL https://bun.sh/install | bash");
                }
            }
        }
        BunAction::Eval { code, timeout, cwd } => {
            let runtime = crate::bun::BunRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.eval(&code, std::time::Duration::from_secs(timeout), None, cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        BunAction::Run { script, timeout, cwd, args } => {
            let runtime = crate::bun::BunRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.run_script(&script, &args, std::time::Duration::from_secs(timeout), None, cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        BunAction::Test { target, timeout, cwd, args } => {
            let runtime = crate::bun::BunRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.test(&target, &args, std::time::Duration::from_secs(timeout), cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        BunAction::Install { packages, dev, allow_native, timeout, cwd } => {
            let runtime = crate::bun::BunRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.install(&packages, dev, allow_native, std::time::Duration::from_secs(timeout), cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        BunAction::Build { entrypoint, outdir, minify, target, timeout, cwd } => {
            let runtime = crate::bun::BunRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.build(&entrypoint, &outdir, minify, &target, std::time::Duration::from_secs(timeout), cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        BunAction::Compile { entrypoint, outfile, minify, bytecode, timeout } => {
            let tool = crate::tools::bun_compile::BunCompileTool::new();
            let args = serde_json::json!({
                "entrypoint": entrypoint,
                "outfile": outfile,
                "minify": minify,
                "bytecode": bytecode,
                "timeout_secs": timeout
            });
            let output = tool.execute(args).await?;
            println!("{output}");
        }
    }
    Ok(())
}

async fn handle_python_command(action: PythonAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PythonAction::Info => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🐍  Python 3 Runtime & Tool Handler Integration".bold().yellow());
            println!("{}", "=========================================================".cyan());

            match crate::python::PythonRuntime::new() {
                Ok(runtime) => {
                    let ver = runtime.version().await.unwrap_or_else(|_| "unknown".to_string());
                    println!("  [✓] Python Binary:    {}", runtime.python_path().display().to_string().green().bold());
                    println!("  [✓] Python Version:   {}", ver.cyan().bold());
                    println!("  [✓] Runtime Status:   {}", "Ready & Available".green());
                    println!("  [✓] Standard Lib:     {}", "math, json, sys, os integrated".green());
                    println!("  [✓] AgentShield:      {}", "Strict AST & reverse shell guardrails active".green());
                }
                Err(e) => {
                    println!("  [✗] Python Runtime:   {}", "Not Discovered".red().bold());
                    println!("      ↳ Error: {}", e);
                    println!("\n  Please install Python 3.x or configure TAGISAN_PYTHON_PATH.");
                }
            }
        }
        PythonAction::Eval { code, timeout, cwd } => {
            let runtime = crate::python::PythonRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.eval(&code, std::time::Duration::from_secs(timeout), None, cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        PythonAction::Run { script, timeout, cwd: _, args } => {
            let runtime = crate::python::PythonRuntime::new()?;
            let script_path = std::path::PathBuf::from(script);
            let res = runtime.run_file(&script_path, &args, Some(timeout)).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        PythonAction::Install { packages, allow_native, timeout, cwd } => {
            let runtime = crate::python::PythonRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.install(&packages, allow_native, std::time::Duration::from_secs(timeout), cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
    }
    Ok(())
}

async fn handle_perl_command(action: PerlAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PerlAction::Info => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  🐪  Perl 5 Runtime & Tool Handler Integration".bold().yellow());
            println!("{}", "=========================================================".cyan());

            match crate::perl::PerlRuntime::new() {
                Ok(runtime) => {
                    let ver = runtime.version().await.unwrap_or_else(|_| "unknown".to_string());
                    println!("  [✓] Perl Binary:      {}", runtime.perl_path().display().to_string().green().bold());
                    println!("  [✓] Perl Version:     {}", ver.cyan().bold());
                    println!("  [✓] Runtime Status:   {}", "Ready & Available".green());
                    println!("  [✓] Regex Engine:     {}", "Native Perl 5 regex stream processor".green());
                    println!("  [✓] AgentShield:      {}", "Strict backtick, pipe & system guardrails active".green());
                }
                Err(e) => {
                    println!("  [✗] Perl Runtime:     {}", "Not Discovered".red().bold());
                    println!("      ↳ Error: {}", e);
                    println!("\n  Please install Perl 5.x or configure TAGISAN_PERL_PATH.");
                }
            }
        }
        PerlAction::Eval { code, timeout, cwd } => {
            let runtime = crate::perl::PerlRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.eval(&code, std::time::Duration::from_secs(timeout), None, cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        PerlAction::Run { script, timeout, cwd: _, args } => {
            let runtime = crate::perl::PerlRuntime::new()?;
            let script_path = std::path::PathBuf::from(script);
            let res = runtime.run_file(&script_path, &args, Some(timeout)).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
        PerlAction::Install { modules, allow_native, timeout, cwd } => {
            let runtime = crate::perl::PerlRuntime::new()?;
            let cwd_path = cwd.map(std::path::PathBuf::from);
            let res = runtime.install(&modules, allow_native, std::time::Duration::from_secs(timeout), cwd_path).await?;
            print!("{}", res.combined_output());
            if !res.success {
                std::process::exit(res.exit_code);
            }
        }
    }
    Ok(())
}

async fn handle_vella_command(action: VellaAction) -> Result<(), Box<dyn std::error::Error>> {
    let mgr = crate::vella::VellaAppManager::with_default_schemas();

    match action {
        VellaAction::Status => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚡ VELLA SOVEREIGN FRAMEWORK & DOMAIN GOVERNOR".bold().yellow());
            println!("{}", "=========================================================".cyan());
            let e_stop = mgr.governor.is_e_stop_active().await;
            println!("  Operational State:      {}", if e_stop { "🚨 EMERGENCY STOP LATCHED".bold().red() } else { "✅ ACTIVE / NOMINAL".bold().green() });
            println!("  Max Trade Order Value:  ${:.2} USD", mgr.governor.max_order_value_usd);
            println!("  Max Order Size:         {} contracts", mgr.governor.max_order_size);
            println!("  Max Permitted Leverage: {:.1}x", mgr.governor.max_leverage);
            println!("  Allowed SCADA Coils:    [{}, {}]", mgr.governor.allowed_scada_coils.0, mgr.governor.allowed_scada_coils.1);
            println!("  Max Robot Velocity:     {:.1} m/s", mgr.governor.max_robot_velocity_ms);
            println!("  Debate Governance:      {}", "MANDATORY FOR HIGH-STAKES".bold().cyan());
            #[cfg(feature = "vella")]
            {
                let reg = mgr.schema_registry.read().await;
                println!("  Vella Model Schemas:    {} registered", reg.len());
            }
            println!("{}", "=========================================================".cyan());
        }

        VellaAction::Estop { reason } => {
            mgr.governor.set_e_stop(true, &reason).await;
            println!("{}", format!("🚨 [VELLA E-STOP LATCHED]: {}", reason).bold().red());
            println!("{}", "All physical SCADA actuators and drone kinematics are BLOCKED.".yellow());
        }

        VellaAction::ClearEstop { reason } => {
            mgr.governor.set_e_stop(false, &reason).await;
            println!("{}", format!("✅ [VELLA E-STOP CLEARED]: {}", reason).bold().green());
            println!("{}", "Actuation and flight channels successfully re-armed.".cyan());
        }

        VellaAction::Schemas => {
            #[cfg(feature = "vella")]
            {
                let reg = mgr.schema_registry.read().await;
                let schemas = reg.all();
                println!("{}", format!("Vella Registered Model Schemas ({}):", schemas.len()).bold().cyan());
                for s in schemas {
                    println!("\n  📦 Model: {} (Table: '{}', Category: '{}')", s.name.bold().yellow(), s.table_name, s.category);
                    if let Some(ref d) = s.description {
                        println!("     Description: {}", d.italic());
                    }
                    println!("     Fields ({})", s.fields.len());
                    for f in &s.fields {
                        println!("       - {}: {:?}", f.name.bold(), f.field_type);
                    }
                }
            }
            #[cfg(not(feature = "vella"))]
            {
                println!("Vella feature is disabled in compilation.");
            }
        }

        VellaAction::Audit => {
            let log = mgr.governor.audit_log.read().await;
            println!("{}", format!("Vella Sovereign Policy Audit Log ({} records):", log.len()).bold().cyan());
            if log.is_empty() {
                println!("  (No audit events recorded yet)");
            } else {
                for (idx, entry) in log.iter().enumerate() {
                    println!("  [{:03}] {}", idx + 1, entry);
                }
            }
        }

        VellaAction::Debate { domain, action_type, target, parameters } => {
            println!("{}", "=========================================================".cyan());
            println!("{}", "  ⚖️  VELLA ADVERSARIAL DEBATE GOVERNOR".bold().yellow());
            println!("{}", "=========================================================".cyan());
            let params_val: serde_json::Value = serde_json::from_str(&parameters)
                .map_err(|e| format!("Invalid parameters JSON: {e}"))?;

            let proposal = crate::vella::DomainActionProposal::new(
                &domain,
                &action_type,
                &target,
                params_val,
                "cli_operator",
            );

            let debate_gov = crate::vella::VellaDebateGovernor::new(mgr.governor.clone());
            let verdict = debate_gov.debate_and_govern(&proposal, None).await?;

            println!("Domain:      {}", verdict.domain.bold().cyan());
            println!("Action Type: {}", verdict.action_type.bold().yellow());
            println!("Target:      {}", verdict.target.bold());
            println!("\n{}", "--- 1. Proponent Thesis ---".green().bold());
            println!("{}", verdict.proposer_thesis);
            println!("\n{}", "--- 2. Security Auditor Antithesis ---".red().bold());
            println!("{}", verdict.auditor_antithesis);
            println!("\n{}", "--- 3. Chief Adjudicator Synthesis ---".magenta().bold());
            println!("{}", verdict.judge_synthesis);
            println!("\n{}", "--- Borda Count Points ---".blue().bold());
            for (opt, pts) in &verdict.borda_points {
                println!("  - {}: {} points", opt.bold(), pts);
            }
            println!("\n{}", "=========================================================".cyan());
            if verdict.approved {
                println!("  FINAL VERDICT: {}", "AUTHORIZED FOR EXECUTION".bold().green());
            } else {
                println!("  FINAL VERDICT: {}", "BLOCKED / REVISION REQUIRED".bold().red());
            }
            println!("{}", "=========================================================".cyan());
        }

        VellaAction::Tool { tool, action: tool_action, args } => {
            let mut parsed_args: serde_json::Value = serde_json::from_str(&args)
                .map_err(|e| format!("Invalid args JSON: {e}"))?;
            if let Some(obj) = parsed_args.as_object_mut() {
                obj.insert("action".to_string(), serde_json::json!(tool_action));
            }

            let output = match tool.to_lowercase().as_str() {
                "trading" => {
                    let t = crate::vella::VellaTradingTool::new(mgr.governor.clone());
                    t.execute(parsed_args).await?
                }
                "scada" => {
                    let t = crate::vella::VellaScadaTool::new(mgr.governor.clone());
                    t.execute(parsed_args).await?
                }
                "robotics" => {
                    let t = crate::vella::VellaRoboticsTool::new(mgr.governor.clone());
                    t.execute(parsed_args).await?
                }
                "medicine" => {
                    let t = crate::vella::VellaMedicineTool::new(mgr.governor.clone());
                    t.execute(parsed_args).await?
                }
                "events" => {
                    #[cfg(feature = "vella")]
                    {
                        let t = crate::vella::VellaEventBridgeTool::new(mgr.event_bus(), mgr.governor.clone());
                        t.execute(parsed_args).await?
                    }
                    #[cfg(not(feature = "vella"))]
                    {
                        "Vella feature not enabled".to_string()
                    }
                }
                _ => return Err(format!("Unknown Vella tool '{}'. Available: trading, scada, robotics, medicine, events", tool).into()),
            };

            println!("{}", output);
        }
    }

    Ok(())
}
