//! Telemetry span builder for structured trace recording.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A single in-flight trace span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub id: String,
    pub name: String,
    pub start_ms: u64,
    pub fields: serde_json::Value,
}

impl TraceSpan {
    /// Create a new span with the given name, recording the current timestamp.
    pub fn new(name: impl Into<String>) -> Self {
        let start_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let id = format!("{:x}", start_ms ^ (std::ptr::addr_of!(start_ms) as u64));

        Self {
            id,
            name: name.into(),
            start_ms,
            fields: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Attach a key-value field to this span.
    pub fn set_field(&mut self, key: impl Into<String>, value: serde_json::Value) -> &mut Self {
        if let serde_json::Value::Object(ref mut map) = self.fields {
            map.insert(key.into(), value);
        }
        self
    }

    /// Finish the span, compute duration, and persist to the given journal.
    pub fn finish(self, journal: &super::journal::TraceJournal) -> Result<()> {
        let end_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let duration_ms = end_ms.saturating_sub(self.start_ms);

        let event = super::journal::TraceEvent {
            id: self.id,
            span_name: self.name,
            timestamp: self.start_ms,
            fields: self.fields,
            duration_ms: Some(duration_ms),
        };

        journal.record(&event)
    }
}
