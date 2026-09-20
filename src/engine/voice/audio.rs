//! Audio buffer, format conversions, resampling, and framing for real-time voice streaming.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};

/// Audio format specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

impl Default for AudioFormat {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bits_per_sample: 16,
        }
    }
}

impl AudioFormat {
    pub fn pcm16_mono(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            channels: 1,
            bits_per_sample: 16,
        }
    }

    pub fn bytes_per_sample(&self) -> usize {
        (self.bits_per_sample / 8) as usize
    }

    pub fn bytes_per_frame(&self) -> usize {
        self.bytes_per_sample() * (self.channels as usize)
    }

    pub fn duration_to_samples(&self, duration_ms: u64) -> usize {
        ((self.sample_rate as u64 * duration_ms) / 1000) as usize
    }

    pub fn samples_to_duration_ms(&self, samples_count: usize) -> u64 {
        if self.sample_rate == 0 {
            0
        } else {
            (samples_count as u64 * 1000) / (self.sample_rate as u64)
        }
    }
}

/// PCM16 Audio Buffer for streaming audio accumulation and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBuffer {
    pub format: AudioFormat,
    pub samples: Vec<i16>,
}

impl AudioBuffer {
    pub fn new(format: AudioFormat) -> Self {
        Self {
            format,
            samples: Vec::new(),
        }
    }

    pub fn with_capacity(format: AudioFormat, capacity: usize) -> Self {
        Self {
            format,
            samples: Vec::with_capacity(capacity),
        }
    }

    pub fn from_pcm16(samples: Vec<i16>, format: AudioFormat) -> Self {
        Self { format, samples }
    }

    pub fn from_bytes_le(bytes: &[u8], format: AudioFormat) -> Result<Self> {
        if bytes.len() % 2 != 0 {
            return Err(TagisanError::Execution(
                "PCM16 byte buffer length must be a multiple of 2".to_string(),
            ));
        }
        let mut samples = Vec::with_capacity(bytes.len() / 2);
        for chunk in bytes.chunks_exact(2) {
            samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
        Ok(Self { format, samples })
    }

    pub fn append(&mut self, other: &[i16]) {
        self.samples.extend_from_slice(other);
    }

    pub fn append_bytes_le(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes.len() % 2 != 0 {
            return Err(TagisanError::Execution(
                "PCM16 byte buffer length must be a multiple of 2".to_string(),
            ));
        }
        for chunk in bytes.chunks_exact(2) {
            self.samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
        Ok(())
    }

    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.samples.len() * 2);
        for &s in &self.samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes
    }

    pub fn duration_ms(&self) -> u64 {
        self.format.samples_to_duration_ms(self.samples.len())
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Peak absolute amplitude in range [0, 32767]
    pub fn peak_amplitude(&self) -> i16 {
        self.samples
            .iter()
            .map(|&s| s.abs())
            .max()
            .unwrap_or(0)
    }

    /// Root-Mean-Square (RMS) energy normalized to [0.0, 1.0]
    pub fn rms_energy(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f64 = self
            .samples
            .iter()
            .map(|&s| {
                let norm = (s as f64) / 32768.0;
                norm * norm
            })
            .sum();
        ((sum_squares / (self.samples.len() as f64)).sqrt()) as f32
    }
}

/// A discrete audio frame suitable for network streaming and real-time VAD processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub seq: u64,
    pub timestamp_ms: u64,
    pub samples: Vec<i16>,
    pub format: AudioFormat,
}

impl AudioFrame {
    pub fn new(seq: u64, timestamp_ms: u64, samples: Vec<i16>, format: AudioFormat) -> Self {
        Self {
            seq,
            timestamp_ms,
            samples,
            format,
        }
    }

    pub fn duration_ms(&self) -> u64 {
        self.format.samples_to_duration_ms(self.samples.len())
    }

    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.samples.len() * 2);
        for &s in &self.samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes
    }

    pub fn to_pcm_bytes(&self) -> Vec<u8> {
        self.to_bytes_le()
    }
}

/// Resampler using linear interpolation for audio sample rate conversion
pub struct AudioResampler;

impl AudioResampler {
    pub fn resample(samples: &[i16], src_rate: u32, dst_rate: u32) -> Vec<i16> {
        if src_rate == dst_rate || samples.is_empty() {
            return samples.to_vec();
        }

        let ratio = (src_rate as f64) / (dst_rate as f64);
        let dst_len = ((samples.len() as f64) / ratio).round() as usize;
        let mut result = Vec::with_capacity(dst_len);

        for i in 0..dst_len {
            let src_idx = (i as f64) * ratio;
            let idx_floor = src_idx.floor() as usize;
            let frac = (src_idx - (idx_floor as f64)) as f32;

            if idx_floor + 1 < samples.len() {
                let s0 = samples[idx_floor] as f32;
                let s1 = samples[idx_floor + 1] as f32;
                let interpolated = s0 + frac * (s1 - s0);
                result.push(interpolated.clamp(-32768.0, 32767.0) as i16);
            } else if idx_floor < samples.len() {
                result.push(samples[idx_floor]);
            } else {
                result.push(0);
            }
        }

        result
    }
}

/// Standard RIFF/WAV header and file encoder for PCM16 audio
pub struct WavEncoder;

impl WavEncoder {
    pub fn encode_pcm16(samples: &[i16], format: &AudioFormat) -> Vec<u8> {
        let num_samples = samples.len();
        let bytes_per_sample = 2usize;
        let data_size = (num_samples * bytes_per_sample) as u32;
        let file_size = 36u32 + data_size;
        let byte_rate = (format.sample_rate * (format.channels as u32) * 2) as u32;
        let block_align = (format.channels * 2) as u16;

        let mut out = Vec::with_capacity(44 + (num_samples * 2));

        // 1. RIFF Header
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(b"WAVE");

        // 2. fmt Subchunk
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size = 16 for PCM
        out.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat = 1 (PCM)
        out.extend_from_slice(&format.channels.to_le_bytes());
        out.extend_from_slice(&format.sample_rate.to_le_bytes());
        out.extend_from_slice(&byte_rate.to_le_bytes());
        out.extend_from_slice(&block_align.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample = 16

        // 3. data Subchunk
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_size.to_le_bytes());
        for &s in samples {
            out.extend_from_slice(&s.to_le_bytes());
        }

        out
    }
}

/// Lightweight packetized Opus-style audio frame container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpusFrame {
    pub seq: u64,
    pub payload: Vec<u8>,
    pub duration_ms: u32,
    pub sample_rate: u32,
}

impl OpusFrame {
    pub fn new(seq: u64, payload: Vec<u8>, duration_ms: u32, sample_rate: u32) -> Self {
        Self {
            seq,
            payload,
            duration_ms,
            sample_rate,
        }
    }

    /// Mock encoder wrapping PCM16 into an Opus packet structure with header
    pub fn encode_mock(samples: &[i16], seq: u64, sample_rate: u32) -> Self {
        let duration_ms = ((samples.len() as u64 * 1000) / (sample_rate as u64)) as u32;
        let mut payload = Vec::with_capacity(4 + samples.len() * 2);
        payload.extend_from_slice(&(samples.len() as u32).to_be_bytes());
        for &s in samples {
            payload.extend_from_slice(&s.to_le_bytes());
        }
        Self {
            seq,
            payload,
            duration_ms,
            sample_rate,
        }
    }

    /// Mock decoder unpacking PCM16 samples from Opus packet
    pub fn decode_mock(&self) -> Result<Vec<i16>> {
        if self.payload.len() < 4 {
            return Err(TagisanError::Execution("Opus packet too short".into()));
        }
        let num_samples = u32::from_be_bytes([
            self.payload[0],
            self.payload[1],
            self.payload[2],
            self.payload[3],
        ]) as usize;

        let mut samples = Vec::with_capacity(num_samples);
        for chunk in self.payload[4..].chunks_exact(2) {
            samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
        }
        Ok(samples)
    }
}
