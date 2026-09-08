use clap::{Parser, Subcommand};
use colored::Colorize;
use std::env;
use std::sync::Arc;
use tagisan::{
    AnthropicProvider, CollaborationStrategy, DialecticalDebateStrategy, EngineContext,
    GeminiProvider, MixtureOfAgentsStrategy, OllamaProvider, OpenAiCompatibleProvider,
    StrategyInput,
};

#[derive(Parser)]
#[command(name = "tagisan")]
#[command(
    about = "Tagisan ng Talino: High-Performance Multi-LLM Collaboration & Debate Engine in Rust",
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
    /// Check available registered providers and API key status
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

    // Register Local Ollama by default
    ctx.register_provider(Arc::new(OllamaProvider::default_local()));

    ctx
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
            println!("{}", "\nChecking Configured LLM Providers:".bold());
            let providers = [
                ("Anthropic (Claude)", "ANTHROPIC_API_KEY"),
                ("xAI (Grok)", "XAI_API_KEY"),
                ("OpenAI (GPT / o-series)", "OPENAI_API_KEY"),
                ("Google Gemini", "GEMINI_API_KEY"),
                ("DeepSeek (R1 / V3)", "DEEPSEEK_API_KEY"),
                ("Local Ollama (Offline)", "No key required (localhost:11434)"),
            ];

            for (name, var) in providers {
                if var.starts_with("No key") {
                    println!("  [✓] {:<25} -> Available (Local daemon)", name.green());
                } else if env::var(var).map(|k| !k.is_empty()).unwrap_or(false) {
                    println!("  [✓] {:<25} -> Configured ({})", name.green(), var);
                } else {
                    println!("  [✗] {:<25} -> Missing {}", name.red(), var);
                }
            }
            println!("\nTip: Add your keys to a .env file in this directory to enable all providers.\n");
        }

        Commands::Moa { prompt } => {
            let ctx = build_engine_context(cli.max_budget);
            println!("\n{}", "🚀 Starting Mixture-of-Agents (MoA) Collaboration...".bold().magenta());
            println!("Prompt: \"{}\"\n", prompt.italic());

            // Build dynamic proposer list based on available providers
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

            // Choose aggregator: Claude 3.5 Sonnet if available, else Gemini or GPT-4o
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

            // Proponent (Thesis)
            let proponent = if env::var("ANTHROPIC_API_KEY").is_ok() {
                ("anthropic".to_string(), "claude-3-5-sonnet-20241022".to_string())
            } else if env::var("OPENAI_API_KEY").is_ok() {
                ("openai".to_string(), "gpt-4o".to_string())
            } else {
                ("ollama".to_string(), "llama3.2".to_string())
            };

            // Adversary (Antithesis & Red Team)
            let adversary = if env::var("DEEPSEEK_API_KEY").is_ok() {
                ("deepseek".to_string(), "deepseek-reasoner".to_string())
            } else if env::var("XAI_API_KEY").is_ok() {
                ("xai".to_string(), "grok-2-latest".to_string())
            } else if env::var("GEMINI_API_KEY").is_ok() {
                ("gemini".to_string(), "gemini-2.0-flash".to_string())
            } else {
                ("ollama".to_string(), "llama3.2".to_string())
            };

            // Adjudicator (Lakandiwa / Master Synthesis)
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
