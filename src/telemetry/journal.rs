//! JSONL-based trace journal for persistent, append-only event storage.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use tracing::debug;

/// A single recorded trace event persisted to the JSONL journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: String,
    pub span_name: String,
    pub timestamp: u64,
    pub fields: serde_json::Value,
    pub duration_ms: Option<u64>,
}

/// Append-only JSONL journal that persists trace events to disk.
#[derive(Debug, Clone)]
pub struct TraceJournal {
    pub path: PathBuf,
}

impl Default for TraceJournal {
    fn default() -> Self {
        Self {
            path: PathBuf::from(".tagisan").join("traces.jsonl"),
        }
    }
}

impl TraceJournal {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Append a single trace event as a JSON line to the journal file.
    pub fn record(&self, event: &TraceEvent) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TagisanError::Execution(format!("Failed to create telemetry dir: {e}"))
            })?;
        }

        let line =
            serde_json::to_string(event).map_err(TagisanError::Serialization)? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| {
                TagisanError::Execution(format!("Failed to open trace journal: {e}"))
            })?;

        file.write_all(line.as_bytes()).map_err(|e| {
            TagisanError::Execution(format!("Failed to write trace event: {e}"))
        })?;

        debug!(span = %event.span_name, id = %event.id, "Recorded trace event");
        Ok(())
    }

    /// Read all trace events from the journal file.
    pub fn read_all(&self) -> Result<Vec<TraceEvent>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&self.path).map_err(|e| {
            TagisanError::Execution(format!("Failed to open trace journal for reading: {e}"))
        })?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                TagisanError::Execution(format!("Failed to read trace journal line {i}: {e}"))
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let event: TraceEvent = serde_json::from_str(&line).map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to parse trace event on line {i}: {e}"
                ))
            })?;
            events.push(event);
        }

        Ok(events)
    }

    /// Clear all events from the journal file.
    pub fn clear(&self) -> Result<()> {
        if self.path.exists() {
            fs::write(&self.path, b"").map_err(|e| {
                TagisanError::Execution(format!("Failed to clear trace journal: {e}"))
            })?;
        }
        Ok(())
    }

    /// Export all trace events to a SQLite database.
    pub fn export_sqlite(&self, db_path: &Path) -> Result<usize> {
        let events = self.read_all()?;
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TagisanError::Execution(format!("Failed to create SQLite dir: {e}"))
            })?;
        }

        let conn = rusqlite::Connection::open(db_path).map_err(|e| {
            TagisanError::Execution(format!("Failed to open SQLite database {:?}: {e}", db_path))
        })?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS traces (
                id TEXT PRIMARY KEY,
                span_name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                duration_ms INTEGER,
                fields TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| TagisanError::Execution(format!("SQLite table init error: {e}")))?;

        let mut stmt = conn
            .prepare(
                "INSERT OR REPLACE INTO traces (id, span_name, timestamp, duration_ms, fields)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(|e| TagisanError::Execution(format!("SQLite prepare error: {e}")))?;

        let count = events.len();
        for event in &events {
            let fields_str =
                serde_json::to_string(&event.fields).unwrap_or_else(|_| "{}".to_string());
            let duration = event.duration_ms.map(|d| d as i64);
            stmt.execute(rusqlite::params![
                event.id,
                event.span_name,
                event.timestamp as i64,
                duration,
                fields_str
            ])
            .map_err(|e| TagisanError::Execution(format!("SQLite insert error: {e}")))?;
        }

        Ok(count)
    }

    /// Read trace events from a SQLite database.
    pub fn read_sqlite(db_path: &Path) -> Result<Vec<TraceEvent>> {
        let conn = rusqlite::Connection::open(db_path).map_err(|e| {
            TagisanError::Execution(format!("Failed to open SQLite database {:?}: {e}", db_path))
        })?;

        let mut stmt = conn
            .prepare(
                "SELECT id, span_name, timestamp, duration_ms, fields FROM traces ORDER BY timestamp ASC",
            )
            .map_err(|e| TagisanError::Execution(format!("SQLite prepare error: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let span_name: String = row.get(1)?;
                let timestamp: i64 = row.get(2)?;
                let duration_ms: Option<i64> = row.get(3)?;
                let fields_str: String = row.get(4)?;
                let fields = serde_json::from_str(&fields_str).unwrap_or(serde_json::Value::Null);

                Ok(TraceEvent {
                    id,
                    span_name,
                    timestamp: timestamp as u64,
                    fields,
                    duration_ms: duration_ms.map(|d| d as u64),
                })
            })
            .map_err(|e| TagisanError::Execution(format!("SQLite query error: {e}")))?;

        let mut result = Vec::new();
        for row in rows {
            if let Ok(evt) = row {
                result.push(evt);
            }
        }
        Ok(result)
    }

    /// Return the path to the underlying JSONL file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}
