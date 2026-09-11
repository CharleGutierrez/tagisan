//! Tagisan telemetry subsystem — OpenTelemetry-compatible tracing and JSONL journal.
//!
//! # Quick Start
//! ```rust,no_run
//! use tagisan::telemetry::TagisanTracer;
//!
//! let tracer = TagisanTracer::new("tgs", None);
//! tracer.record_dag_node("plan", "claude", 200, 800, 1200);
//! ```
//!
//! # CLI
//! ```bash
//! tgs trace --live          # Stream last 20 events from .tagisan/traces.jsonl
//! tgs trace --export out.jsonl
//! tgs trace --clear
//! ```

pub mod journal;
pub mod otel;
pub mod span;
pub mod tui;

pub use journal::{TraceEvent, TraceJournal};
pub use otel::TagisanTracer;
pub use span::TraceSpan;
pub use tui::run_trace_tui;

use crate::error::{Result, TagisanError};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

/// CLI arguments for the `tgs trace` subcommand.
#[derive(Debug, Args, Clone)]
pub struct TraceArgs {
    /// Stream the last N trace events from the journal to stdout.
    #[arg(long, default_value = "false")]
    pub live: bool,

    /// Launch interactive Ratatui TUI live trace tree viewer.
    #[arg(long, default_value = "false")]
    pub tui: bool,

    /// Export the trace journal to the specified file path (JSONL or SQLite).
    #[arg(long)]
    pub export: Option<PathBuf>,

    /// Export traces directly to a SQLite database file (.db).
    #[arg(long)]
    pub sqlite: Option<PathBuf>,

    /// Number of recent events to display with --live.
    #[arg(long, default_value = "20")]
    pub last: usize,

    /// Clear all events from the trace journal.
    #[arg(long, default_value = "false")]
    pub clear: bool,
}

/// Execute the `tgs trace` command.
pub fn run_trace_command(args: &TraceArgs) -> Result<()> {
    let journal = TraceJournal::default();

    if args.clear {
        journal.clear()?;
        println!("{}", "✓ Trace journal cleared.".green());
        return Ok(());
    }

    if let Some(ref db_path) = args.sqlite {
        let count = journal.export_sqlite(db_path)?;
        println!(
            "{}",
            format!("✓ Exported {} trace event(s) to SQLite database: {:?}", count, db_path).green()
        );
        return Ok(());
    }

    if let Some(ref export_path) = args.export {
        if !journal.path().exists() {
            return Err(TagisanError::Execution(
                "No trace journal found at .tagisan/traces.jsonl".to_string(),
            ));
        }

        let is_sqlite = export_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext == "db" || ext == "sqlite" || ext == "sqlite3")
            .unwrap_or(false);

        if is_sqlite {
            let count = journal.export_sqlite(export_path)?;
            println!(
                "{}",
                format!("✓ Exported {} trace event(s) to SQLite database: {:?}", count, export_path).green()
            );
        } else {
            std::fs::copy(journal.path(), export_path).map_err(|e| {
                TagisanError::Execution(format!("Failed to export trace journal: {e}"))
            })?;
            println!(
                "{}",
                format!("✓ Trace journal exported to {:?}", export_path).green()
            );
        }
        return Ok(());
    }

    if args.tui {
        return run_trace_tui(&journal);
    }

    if args.live {
        let events = journal.read_all()?;
        let total = events.len();
        let display: Vec<_> = events.into_iter().rev().take(args.last).rev().collect();

        println!(
            "\n{} (showing {}/{} events from {:?})\n",
            "Tagisan Trace Journal".bold().cyan(),
            display.len(),
            total,
            journal.path()
        );

        println!(
            "{:<36} {:<30} {:<10} {:<12} {}",
            "ID".bold(),
            "Span".bold(),
            "Duration".bold(),
            "Timestamp".bold(),
            "Fields".bold()
        );
        println!("{}", "─".repeat(110).dimmed());

        for event in &display {
            let duration = event
                .duration_ms
                .map(|d| format!("{d}ms"))
                .unwrap_or_else(|| "—".to_string());

            let fields_preview = match &event.fields {
                serde_json::Value::Object(map) => {
                    let pairs: Vec<String> = map
                        .iter()
                        .take(3)
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect();
                    pairs.join(", ")
                }
                other => other.to_string(),
            };

            println!(
                "{:<36} {:<30} {:<10} {:<12} {}",
                event.id.dimmed(),
                event.span_name.yellow(),
                duration.cyan(),
                event.timestamp,
                fields_preview.dimmed()
            );
        }
        println!();
        return Ok(());
    }

    // Default: show help hint
    println!(
        "{}",
        "Use --live to view events, --export <path> to export, --clear to wipe.".dimmed()
    );
    Ok(())
}
