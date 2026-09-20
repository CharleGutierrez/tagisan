//! Voice tools for Tagisan agents: VoiceSpeakTool and VoiceListenTool.

use crate::error::{Result, TagisanError};
use crate::tools::ToolHandler;
use async_trait::async_trait;
use serde_json::{json, Value};

/// Tool enabling agents to synthesize spoken audio response
pub struct VoiceSpeakTool;

impl Default for VoiceSpeakTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for VoiceSpeakTool {
    fn name(&self) -> &'static str {
        "voice_speak"
    }

    fn description(&self) -> &'static str {
        "Speak a message aloud to the user using Tagisan voice synthesis."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The exact spoken text to synthesize and play."
                },
                "voice": {
                    "type": "string",
                    "description": "Optional voice identifier (e.g., 'Aoede', 'Puck', 'Charon').",
                    "default": "Aoede"
                }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let text = arguments
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TagisanError::Execution("Missing 'text' parameter".to_string()))?;

        let voice = arguments
            .get("voice")
            .and_then(|v| v.as_str())
            .unwrap_or("Aoede");

        let out = json!({
            "status": "spoken",
            "text": text,
            "voice": voice,
            "timestamp_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        });
        Ok(out.to_string())
    }
}

/// Tool enabling agents to transcribe or listen to user speech
pub struct VoiceListenTool;

impl Default for VoiceListenTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl ToolHandler for VoiceListenTool {
    fn name(&self) -> &'static str {
        "voice_listen"
    }

    fn description(&self) -> &'static str {
        "Listen for a user spoken turn or transcribe incoming microphone stream."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "timeout_ms": {
                    "type": "integer",
                    "description": "Maximum duration to wait for user speech in milliseconds.",
                    "default": 5000
                }
            }
        })
    }

    async fn execute(&self, arguments: Value) -> Result<String> {
        let timeout_ms = arguments
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(5000);

        let out = json!({
            "status": "listening",
            "timeout_ms": timeout_ms,
            "note": "Awaiting user spoken turn via VoiceSession"
        });
        Ok(out.to_string())
    }
}
