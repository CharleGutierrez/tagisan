//! Real-time Voice & Audio Streaming Engine for Tagisan (TGS)
//!
//! Features bidirectional audio streaming, energy/ZCR Voice Activity Detection (VAD),
//! barge-in detection, PCM16/Opus resampling, and Gemini Live API protocol integration.

pub mod audio;
pub mod vad;
pub mod session;
pub mod gemini_live;
pub mod tools;

pub use audio::{
    AudioBuffer, AudioFormat, AudioFrame, AudioResampler, OpusFrame, WavEncoder,
};
pub use vad::{
    SpeechSegment, SpeechSegmenter, VadConfig, VadState, VoiceActivityDetector,
};
pub use session::{
    VoiceEvent, VoiceSession, VoiceSessionConfig, VoiceSessionState,
};
pub use gemini_live::{
    build_audio_input_message, build_setup_message, extract_server_audio,
    BidiClientMessage, BidiPart, BidiServerMessage, GeminiLiveConfig,
};
pub use tools::{
    VoiceListenTool, VoiceSpeakTool,
};
