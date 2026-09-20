//! Voice Activity Detection (VAD) with energy and zero-crossing analysis,
//! silence thresholding, and continuous speech segmenting.

use super::audio::{AudioBuffer, AudioFormat, AudioFrame};
use serde::{Deserialize, Serialize};

/// Configuration parameters for Voice Activity Detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Normalized RMS energy threshold above which audio is considered speech (0.0 - 1.0)
    pub energy_threshold: f32,
    /// Minimum Zero Crossing Rate to classify high-frequency unvoiced speech/fricatives
    pub zcr_speech_threshold: f32,
    /// Minimum duration of continuous speech to trigger a valid speech turn (ms)
    pub min_speech_duration_ms: u64,
    /// Silence duration after speech before marking the speech turn as completed (ms)
    pub silence_timeout_ms: u64,
    /// Duration of each discrete analysis frame (ms)
    pub frame_duration_ms: u64,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            energy_threshold: 0.015,
            zcr_speech_threshold: 0.05,
            min_speech_duration_ms: 150,
            silence_timeout_ms: 600,
            frame_duration_ms: 20,
        }
    }
}

/// State of the VAD finite state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VadState {
    /// Background ambient silence or non-speech noise
    Silence,
    /// Onset of speech detected
    SpeechStart,
    /// Sustained active speech
    InSpeech,
    /// Speech ended; silence threshold reached
    SpeechEnd,
}

/// A detected continuous segment of spoken audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub audio: AudioBuffer,
    pub peak_rms: f32,
}

impl SpeechSegment {
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }
}

/// Real-time Voice Activity Detector implementing energy & zero-crossing analysis
pub struct VoiceActivityDetector {
    config: VadConfig,
    state: VadState,
    speech_start_ms: Option<u64>,
    last_speech_ms: Option<u64>,
    current_time_ms: u64,
    consecutive_speech_frames: usize,
    consecutive_silence_frames: usize,
}

impl VoiceActivityDetector {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            state: VadState::Silence,
            speech_start_ms: None,
            last_speech_ms: None,
            current_time_ms: 0,
            consecutive_speech_frames: 0,
            consecutive_silence_frames: 0,
        }
    }

    /// Compute Root-Mean-Square (RMS) energy normalized to [0.0, 1.0]
    pub fn compute_rms(samples: &[i16]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = samples
            .iter()
            .map(|&s| {
                let n = (s as f64) / 32768.0;
                n * n
            })
            .sum();
        ((sum_sq / (samples.len() as f64)).sqrt()) as f32
    }

    /// Compute Zero Crossing Rate (ZCR)
    pub fn compute_zcr(samples: &[i16]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }
        let mut crossings = 0usize;
        for i in 1..samples.len() {
            if (samples[i] >= 0 && samples[i - 1] < 0) || (samples[i] < 0 && samples[i - 1] >= 0) {
                crossings += 1;
            }
        }
        (crossings as f32) / (samples.len() as f32)
    }

    /// Evaluate an incoming audio frame and return `(is_speech_frame, current_state)`
    pub fn process_frame(&mut self, samples: &[i16]) -> (bool, VadState) {
        let frame_duration = self.config.frame_duration_ms;
        self.current_time_ms += frame_duration;

        let rms = Self::compute_rms(samples);
        let zcr = Self::compute_zcr(samples);

        // Frame is speech if RMS exceeds energy threshold or ZCR indicates fricative speech with moderate energy
        let is_voice_frame = rms >= self.config.energy_threshold
            || (rms >= (self.config.energy_threshold * 0.5) && zcr >= self.config.zcr_speech_threshold);

        if is_voice_frame {
            self.consecutive_speech_frames += 1;
            self.consecutive_silence_frames = 0;
            self.last_speech_ms = Some(self.current_time_ms);
        } else {
            self.consecutive_silence_frames += 1;
            self.consecutive_speech_frames = 0;
        }

        match self.state {
            VadState::Silence | VadState::SpeechEnd => {
                if self.consecutive_speech_frames >= 2 {
                    self.state = VadState::SpeechStart;
                    self.speech_start_ms = Some(self.current_time_ms.saturating_sub(frame_duration * 2));
                } else {
                    self.state = VadState::Silence;
                }
            }
            VadState::SpeechStart => {
                if is_voice_frame {
                    self.state = VadState::InSpeech;
                } else if self.consecutive_silence_frames > 3 {
                    self.state = VadState::Silence;
                    self.speech_start_ms = None;
                }
            }
            VadState::InSpeech => {
                if let Some(last_sp) = self.last_speech_ms {
                    let silence_duration = self.current_time_ms.saturating_sub(last_sp);
                    if silence_duration >= self.config.silence_timeout_ms {
                        let total_speech_dur = self
                            .speech_start_ms
                            .map(|s| last_sp.saturating_sub(s))
                            .unwrap_or(0);

                        if total_speech_dur >= self.config.min_speech_duration_ms {
                            self.state = VadState::SpeechEnd;
                        } else {
                            self.state = VadState::Silence;
                        }
                        self.speech_start_ms = None;
                    }
                }
            }
        }

        (is_voice_frame, self.state)
    }

    pub fn reset(&mut self) {
        self.state = VadState::Silence;
        self.speech_start_ms = None;
        self.last_speech_ms = None;
        self.consecutive_speech_frames = 0;
        self.consecutive_silence_frames = 0;
    }

    pub fn current_state(&self) -> VadState {
        self.state
    }
}

/// Accumulates audio frames and segments continuous speech turns
pub struct SpeechSegmenter {
    vad: VoiceActivityDetector,
    format: AudioFormat,
    current_segment_samples: Vec<i16>,
    segment_start_ms: u64,
    segment_counter: u64,
    peak_rms: f32,
}

impl SpeechSegmenter {
    pub fn new(config: VadConfig, format: AudioFormat) -> Self {
        Self {
            vad: VoiceActivityDetector::new(config),
            format,
            current_segment_samples: Vec::new(),
            segment_start_ms: 0,
            segment_counter: 0,
            peak_rms: 0.0,
        }
    }

    /// Ingest an audio frame; if a speech segment is completed, returns `Some(SpeechSegment)`
    pub fn ingest_frame(&mut self, frame: &AudioFrame) -> Option<SpeechSegment> {
        let (is_speech, state) = self.vad.process_frame(&frame.samples);
        let frame_rms = VoiceActivityDetector::compute_rms(&frame.samples);

        if frame_rms > self.peak_rms {
            self.peak_rms = frame_rms;
        }

        match state {
            VadState::SpeechStart => {
                if self.current_segment_samples.is_empty() {
                    self.segment_start_ms = frame.timestamp_ms;
                }
                self.current_segment_samples.extend_from_slice(&frame.samples);
                None
            }
            VadState::InSpeech => {
                self.current_segment_samples.extend_from_slice(&frame.samples);
                None
            }
            VadState::SpeechEnd => {
                self.current_segment_samples.extend_from_slice(&frame.samples);
                let end_ms = frame.timestamp_ms + frame.duration_ms();
                self.segment_counter += 1;

                let segment = SpeechSegment {
                    id: format!("speech-seg-{}", self.segment_counter),
                    start_ms: self.segment_start_ms,
                    end_ms,
                    audio: AudioBuffer::from_pcm16(
                        std::mem::take(&mut self.current_segment_samples),
                        self.format,
                    ),
                    peak_rms: self.peak_rms,
                };

                self.peak_rms = 0.0;
                self.vad.reset();
                Some(segment)
            }
            VadState::Silence => {
                if is_speech {
                    self.current_segment_samples.extend_from_slice(&frame.samples);
                } else if !self.current_segment_samples.is_empty() && self.current_segment_samples.len() > 16000 {
                    // Prevent memory leakage from prolonged noise
                    self.current_segment_samples.clear();
                }
                None
            }
        }
    }

    pub fn flush(&mut self, timestamp_ms: u64) -> Option<SpeechSegment> {
        if self.current_segment_samples.is_empty() {
            return None;
        }
        self.segment_counter += 1;
        let segment = SpeechSegment {
            id: format!("speech-seg-flush-{}", self.segment_counter),
            start_ms: self.segment_start_ms,
            end_ms: timestamp_ms,
            audio: AudioBuffer::from_pcm16(
                std::mem::take(&mut self.current_segment_samples),
                self.format,
            ),
            peak_rms: self.peak_rms,
        };
        self.peak_rms = 0.0;
        self.vad.reset();
        Some(segment)
    }

    pub fn process_frame(&mut self, frame: &AudioFrame) -> Option<SpeechSegment> {
        self.ingest_frame(frame)
    }

    pub fn is_in_speech(&self) -> bool {
        self.vad.current_state() == VadState::InSpeech || self.vad.current_state() == VadState::SpeechStart
    }
}
