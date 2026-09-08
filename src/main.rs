use clap::{Parser, Subcommand};
use colored::Colorize;
use futures::StreamExt;
use std::env;
use std::io::Write;
use std::sync::Arc;
use tagisan::{
    AnthropicProvider, ChatSession, CollaborationStrategy, CompletionRequest,
    DialecticalDebateStrategy, EngineContext, GeminiProvider, MixtureOfAgentsStrategy,
    OllamaProvider, OpenAiCompatibleProvider, ProviderCapabilities, StrategyInput, StreamChunkDelta,
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
        /// Provider ID: anthropic, openai, xai, deepseek, gemini, ollama
        #[arg(short, long, default_value = "ollama")]
        provider: String,

        /// Model name (e.g. claude-3-5-sonnet-20241022, grok-2-latest, gemini-2.0-flash, llama3.2)
        #[arg(short, long)]
        model: Option<String>,

        /// User query or prompt
        prompt: String,
    },
    /// Direct single completion query to any provider
    Ask {
        /// Provider ID: anthropic, openai, xai, deepseek, gemini, ollama
        #[arg(short, long, default_value = "ollama")]
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
        /// The problem or architecture decision to debate
        prompt: String,
    },
    /// Check configured LLM providers, API keys, and model capability bitflags
    Status,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    println!("{}", "=========================================================".cyan());
    println!("{}", "  🇵🇭 TAGISAN: Multi-LLM Collaboration Engine in Rust".bold().yellow());
    println!("{}", "=========================================================".cyan());

    match cli.command {
        Commands::Status => {
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
            let prov = ctx.get_provider(&provider)?;

            let model_name = model.unwrap_or_else(|| match provider.as_str() {
                "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
                "xai" => "grok-2-latest".to_string(),
                "openai" => "gpt-4o".to_string(),
                "gemini" => "gemini-2.0-flash".to_string(),
                "deepseek" => "deepseek-reasoner".to_string(),
                _ => "llama3.2".to_string(),
            });

            println!(
                "\n{} [{}: {}]...",
                "Streaming from".bold().magenta(),
                provider.cyan().bold(),
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
            let prov = ctx.get_provider(&provider)?;

            let model_name = model.unwrap_or_else(|| match provider.as_str() {
                "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
                "xai" => "grok-2-latest".to_string(),
                "openai" => "gpt-4o".to_string(),
                "gemini" => "gemini-2.0-flash".to_string(),
                "deepseek" => "deepseek-chat".to_string(),
                _ => "llama3.2".to_string(),
            });

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

        Commands::Debate { prompt } => {
            let ctx = build_engine_context(cli.max_budget);
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
    }

    Ok(())
}
