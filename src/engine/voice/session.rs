//! Voice streaming session management, barge-in detection, and real-time event bus.

use super::audio::{AudioBuffer, AudioFormat, AudioFrame};
use super::vad::{SpeechSegment, SpeechSegmenter, VadConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Voice session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSessionConfig {
    pub format: AudioFormat,
    pub vad_config: VadConfig,
    pub allow_barge_in: bool,
    pub echo_cancellation: bool,
}

impl Default for VoiceSessionConfig {
    fn default() -> Self {
        Self {
            format: AudioFormat::pcm16_mono(16000),
            vad_config: VadConfig::default(),
            allow_barge_in: true,
            echo_cancellation: true,
        }
    }
}

/// Operational state of the Voice Session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceSessionState {
    Idle,
    Listening,
    UserSpeaking,
    Processing,
    AgentSpeaking,
    Interrupted,
}

/// Events emitted during voice streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceEvent {
    SessionStarted,
    SessionEnded,
    StateChanged { from: VoiceSessionState, to: VoiceSessionState },
    SpeechStarted { timestamp_ms: u64 },
    SpeechEnded { segment: SpeechSegment },
    AgentSpeakingStarted,
    AgentSpeakingEnded,
    AgentAudioChunk { frame: AudioFrame },
    Interrupted { timestamp_ms: u64 },
    Error { message: String },
}

/// Real-time bidirectional Voice Session
pub struct VoiceSession {
    pub config: VoiceSessionConfig,
    state: VoiceSessionState,
    segmenter: SpeechSegmenter,
    current_time_ms: u64,
    frame_sequence: u64,
    outgoing_queue: Vec<AudioFrame>,
    event_tx: broadcast::Sender<VoiceEvent>,
}

impl VoiceSession {
    pub fn new(config: VoiceSessionConfig) -> Self {
        let (event_tx, _) = broadcast::channel(128);
        let segmenter = SpeechSegmenter::new(config.vad_config.clone(), config.format);
        Self {
            config,
            state: VoiceSessionState::Idle,
            segmenter,
            current_time_ms: 0,
            frame_sequence: 0,
            outgoing_queue: Vec::new(),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<VoiceEvent> {
        self.event_tx.subscribe()
    }

    pub fn state(&self) -> VoiceSessionState {
        self.state
    }

    fn transition_to(&mut self, new_state: VoiceSessionState) -> Option<VoiceEvent> {
        if self.state != new_state {
            let from = self.state;
            self.state = new_state;
            let ev = VoiceEvent::StateChanged { from, to: new_state };
            let _ = self.event_tx.send(ev.clone());
            Some(ev)
        } else {
            None
        }
    }

    /// Feeds microphone PCM16 audio into the session and returns generated events
    pub fn feed_input_pcm(&mut self, samples: &[i16]) -> Vec<VoiceEvent> {
        let mut events = Vec::new();
        let frame_duration_ms = self.config.format.samples_to_duration_ms(samples.len());
        self.current_time_ms += frame_duration_ms;

        let frame = AudioFrame {
            format: self.config.format,
            seq: self.frame_sequence,
            timestamp_ms: self.current_time_ms,
            samples: samples.to_vec(),
        };
        self.frame_sequence += 1;

        // Barge-in check: If user speaks while agent is speaking
        if self.state == VoiceSessionState::AgentSpeaking && self.config.allow_barge_in {
            let rms = AudioBuffer { format: self.config.format, samples: samples.to_vec() }.rms_energy();
            if rms > self.config.vad_config.energy_threshold * 1.5 {
                self.outgoing_queue.clear();
                self.transition_to(VoiceSessionState::Interrupted);
                let ev = VoiceEvent::Interrupted { timestamp_ms: self.current_time_ms };
                let _ = self.event_tx.send(ev.clone());
                events.push(ev);
            }
        }

        if let Some(segment) = self.segmenter.process_frame(&frame) {
            self.transition_to(VoiceSessionState::Processing);
            let ev = VoiceEvent::SpeechEnded { segment };
            let _ = self.event_tx.send(ev.clone());
            events.push(ev);
        } else if self.segmenter.is_in_speech() {
            if self.state != VoiceSessionState::UserSpeaking {
                self.transition_to(VoiceSessionState::UserSpeaking);
                let ev = VoiceEvent::SpeechStarted { timestamp_ms: self.current_time_ms };
                let _ = self.event_tx.send(ev.clone());
                events.push(ev);
            }
        } else if self.state == VoiceSessionState::UserSpeaking {
            self.transition_to(VoiceSessionState::Listening);
        }

        events
    }

    /// Queues synthesized audio to be output to the speaker
    pub fn queue_agent_audio(&mut self, frame: AudioFrame) {
        if self.state != VoiceSessionState::AgentSpeaking {
            self.transition_to(VoiceSessionState::AgentSpeaking);
            let _ = self.event_tx.send(VoiceEvent::AgentSpeakingStarted);
        }
        self.outgoing_queue.push(frame.clone());
        let _ = self.event_tx.send(VoiceEvent::AgentAudioChunk { frame });
    }

    /// Pulls next audio frame for speaker playback
    pub fn poll_playback_frame(&mut self) -> Option<AudioFrame> {
        let frame = if !self.outgoing_queue.is_empty() {
            Some(self.outgoing_queue.remove(0))
        } else {
            None
        };

        if self.outgoing_queue.is_empty() && self.state == VoiceSessionState::AgentSpeaking {
            self.transition_to(VoiceSessionState::Listening);
            let _ = self.event_tx.send(VoiceEvent::AgentSpeakingEnded);
        }

        frame
    }

    pub fn interrupt(&mut self) {
        self.outgoing_queue.clear();
        self.transition_to(VoiceSessionState::Interrupted);
        let _ = self.event_tx.send(VoiceEvent::Interrupted { timestamp_ms: self.current_time_ms });
    }
}
