//! Tagisan automated swarm evaluation suite — `tgs eval`.
//!
//! Runs multi-model evaluation against JSON benchmark datasets using
//! Borda count or majority voting consensus scoring.
//!
//! # CLI Usage
//! ```bash
//! tgs eval --dataset ./evals/bench.json \
//!          --models claude-3-5-sonnet-20241022,gemini-1.5-pro \
//!          --rule borda \
//!          --threshold 3.4
//! ```

pub mod dataset;
pub mod runner;
pub mod scorer;

pub use dataset::{EvalCase, EvalDataset};
pub use runner::EvalRunner;
pub use scorer::{
    cosine_score, evaluate_criteria, groundedness_score, safety_score, threshold_pass, EvalReport,
    EvalScore,
};

use crate::engine::EngineContext;
use crate::error::{Result, TagisanError};
use clap::Args;
use std::path::PathBuf;

/// CLI arguments for the `tgs eval` subcommand.
#[derive(Debug, Args, Clone)]
pub struct EvalArgs {
    /// Path to the JSON evaluation dataset file.
    #[arg(long)]
    pub dataset: PathBuf,

    /// Comma-separated list of model identifiers to evaluate.
    #[arg(long, value_delimiter = ',')]
    pub models: Vec<String>,

    /// Voting/aggregation rule: "borda" (default) or "majority".
    #[arg(long, default_value = "borda")]
    pub rule: String,

    /// Pass/fail threshold on the 1.0–5.0 scoring scale (default: 3.4).
    #[arg(long, default_value = "3.4")]
    pub threshold: f32,

    /// Optional path to save the JSON evaluation report.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Execute the `tgs eval` command.
pub async fn run_eval_command(args: &EvalArgs, ctx: &EngineContext) -> Result<()> {
    if args.models.is_empty() {
        return Err(TagisanError::Execution(
            "At least one model must be specified via --models".to_string(),
        ));
    }

    let dataset = EvalDataset::load_from_file(&args.dataset)?;

    println!(
        "🧪 Running eval on dataset '{}' ({} cases) with {} model(s)…",
        dataset.name,
        dataset.cases.len(),
        args.models.len()
    );

    let runner = EvalRunner::new(args.models.clone(), args.rule.clone(), args.threshold);
    let report = runner.run(&dataset, ctx).await?;

    report.print_summary();

    if let Some(ref output_path) = args.output {
        report.save_to_file(output_path)?;
        println!("📄 Report saved to {:?}", output_path);
    }

    Ok(())
}
