//! Gemini Live API bidirectional real-time audio client and protocol framing.

use super::audio::{AudioBuffer, AudioFormat, AudioFrame};
use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};

/// Configuration for Gemini Live API session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiLiveConfig {
    pub api_key: String,
    pub model: String,
    pub voice_name: String,
    pub system_instruction: Option<String>,
    pub endpoint: String,
}

impl Default for GeminiLiveConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("GEMINI_API_KEY").unwrap_or_default(),
            model: "models/gemini-2.0-flash-exp".to_string(),
            voice_name: "Aoede".to_string(),
            system_instruction: Some("You are Tagisan Voice Copilot, an ultra-fast, concise, and expert coding pair programmer.".to_string()),
            endpoint: "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1alpha.GenerativeService.BidiGenerateContent".to_string(),
        }
    }
}

/// Outgoing message to Gemini Live WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BidiClientMessage {
    Setup {
        setup: BidiSetupConfig,
    },
    RealtimeInput {
        realtime_input: BidiRealtimeInput,
    },
    ClientContent {
        client_content: BidiClientContent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiSetupConfig {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<BidiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<BidiContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiGenerationConfig {
    pub response_modalities: Vec<String>,
    pub speech_config: Option<BidiSpeechConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiSpeechConfig {
    pub voice_config: BidiVoiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiVoiceConfig {
    pub prebuilt_voice_config: BidiPrebuiltVoiceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiPrebuiltVoiceConfig {
    pub voice_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiRealtimeInput {
    pub media_chunks: Vec<BidiBlob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiBlob {
    pub mime_type: String,
    pub data: String, // Base64 encoded PCM16 or Opus
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiClientContent {
    pub turns: Vec<BidiContent>,
    pub turn_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiContent {
    pub role: String,
    pub parts: Vec<BidiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BidiPart {
    Text { text: String },
    InlineData { inline_data: BidiBlob },
}

/// Incoming message from Gemini Live WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiServerMessage {
    #[serde(default)]
    pub server_content: Option<BidiServerContent>,
    #[serde(default)]
    pub tool_call: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiServerContent {
    #[serde(default)]
    pub model_turn: Option<BidiContent>,
    #[serde(default)]
    pub turn_complete: Option<bool>,
    #[serde(default)]
    pub interrupted: Option<bool>,
}

/// Helper to serialize setup message
pub fn build_setup_message(config: &GeminiLiveConfig) -> BidiClientMessage {
    BidiClientMessage::Setup {
        setup: BidiSetupConfig {
            model: config.model.clone(),
            generation_config: Some(BidiGenerationConfig {
                response_modalities: vec!["AUDIO".to_string()],
                speech_config: Some(BidiSpeechConfig {
                    voice_config: BidiVoiceConfig {
                        prebuilt_voice_config: BidiPrebuiltVoiceConfig {
                            voice_name: config.voice_name.clone(),
                        },
                    },
                }),
            }),
            system_instruction: config.system_instruction.as_ref().map(|s| BidiContent {
                role: "system".to_string(),
                parts: vec![BidiPart::Text { text: s.clone() }],
            }),
        },
    }
}

/// Helper to encode PCM16 audio frame to Base64 realtime_input message
pub fn build_audio_input_message(frame: &AudioFrame) -> BidiClientMessage {
    let pcm_bytes = frame.to_pcm_bytes();
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &pcm_bytes);
    BidiClientMessage::RealtimeInput {
        realtime_input: BidiRealtimeInput {
            media_chunks: vec![BidiBlob {
                mime_type: format!("audio/pcm;rate={}", frame.format.sample_rate),
                data: b64,
            }],
        },
    }
}

/// Helper to extract PCM audio from a server turn message
pub fn extract_server_audio(msg: &BidiServerMessage, format: AudioFormat) -> Vec<AudioFrame> {
    let mut frames = Vec::new();
    if let Some(ref sc) = msg.server_content {
        if let Some(ref mt) = sc.model_turn {
            for part in &mt.parts {
                if let BidiPart::InlineData { ref inline_data } = part {
                    if inline_data.mime_type.starts_with("audio/pcm") {
                        if let Ok(bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &inline_data.data) {
                            let mut samples = Vec::with_capacity(bytes.len() / 2);
                            for chunk in bytes.chunks_exact(2) {
                                samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
                            }
                            frames.push(AudioFrame {
                                format,
                                seq: 0,
                                timestamp_ms: 0,
                                samples,
                            });
                        }
                    }
                }
            }
        }
    }
    frames
}
