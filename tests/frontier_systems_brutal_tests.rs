//! Brutal Integration Tests for Frontier Systems in Tagisan (TGS)
//!
//! Tests the 5 lacking frontier areas fulfilled with 1000% reality and 1000x reliability:
//! 1. NVMe Layer-by-Layer Disk Streaming (TransformerLayerStreamer, SyntheticSafeTensorsBuilder, PinnedLayerBuffer)
//! 2. Real-Time Voice & Audio Engine (AudioResampler, WavEncoder, VAD, Barge-In, GeminiLive)
//! 3. Interactive Visual Swarm & Computer-Use Web Dashboard (State, WS framing, HTTP API)
//! 4. Unified Cross-Platform Sandboxing (Linux Landlock/Seccomp, Mac Seatbelt, Windows AppContainer)
//! 5. Automated Prompt Distillation & LoRA Dataset Generator (ReflexionDistiller, Alpaca, ShareGPT, DPO)

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Simple RAII temporary directory for tests
struct TestTempDir {
    path: PathBuf,
}

impl TestTempDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).expect("Failed to create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

// ---------------------------------------------------------------------------
// 1. NVMe Layer-by-Layer Disk Streaming Tests
// ---------------------------------------------------------------------------

#[test]
fn test_nvme_layer_streaming_synthetic_and_streamer() {
    use tagisan::engine::layer_streaming::{
        LayerStreamConfig, SyntheticSafeTensorsBuilder, TransformerLayerStreamer,
    };

    let dir = TestTempDir::new("tgs_layer_streaming");
    let hidden_dim = 256;
    let num_layers = 4;
    let mut layer_files = Vec::new();

    // 1. Generate synthetic SafeTensors layers
    for i in 0..num_layers {
        let layer_path = dir.path().join(format!("layer_{i}.safetensors"));
        SyntheticSafeTensorsBuilder::create_layer_file(&layer_path, hidden_dim)
            .expect("Failed to build synthetic SafeTensors layer file");
        assert!(layer_path.exists());
        layer_files.push(layer_path);
    }

    // 2. Initialize streamer with 2 maximum resident layers (double-buffering)
    let config = LayerStreamConfig {
        model_name: "Test-Streaming-Model".to_string(),
        num_layers,
        hidden_dim,
        layer_files,
        max_pinned_layers: 2,
        prefetch_depth: 1,
        ..Default::default()
    };

    let mut streamer = TransformerLayerStreamer::new(config);
    streamer.init_pipeline().expect("Failed to initialize streaming pipeline");

    // 3. Sequentially advance through all layers
    for expected_idx in 0..num_layers {
        let pinned = streamer.advance_layer()
            .unwrap_or_else(|e| panic!("Failed to advance to layer {expected_idx}: {e}"));

        assert_eq!(pinned.layer_idx, expected_idx);
        assert!(pinned.is_pinned());
        assert!(pinned.memory_bytes() > 0);

        // Verify that resident layers never exceed max_pinned_layers (2)
        assert!(
            streamer.active_layer_count() <= 2,
            "Active layer count ({}) exceeded limit (2)",
            streamer.active_layer_count()
        );
    }

    // After 4 layers with max_pinned_layers = 2, at least 2 layers must have been evicted from RAM
    assert!(
        streamer.total_evictions() >= 2,
        "Expected at least 2 layer evictions, found {}",
        streamer.total_evictions()
    );

    // 4. Finish pass and verify full cleanup
    streamer.finish_pass();
    assert_eq!(streamer.active_layer_count(), 0);
}

// ---------------------------------------------------------------------------
// 2. Real-Time Voice & Audio Engine Tests
// ---------------------------------------------------------------------------

#[test]
fn test_audio_resampler_and_wav_encoder() {
    use tagisan::engine::voice::audio::{AudioFormat, AudioResampler, WavEncoder};

    // 1. Generate a 48kHz mono sine wave (100ms = 4800 samples)
    let sample_rate_in = 48000;
    let duration_secs = 0.1f32;
    let num_samples_in = (sample_rate_in as f32 * duration_secs) as usize;
    let mut samples_in = Vec::with_capacity(num_samples_in);

    for i in 0..num_samples_in {
        let s = (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / (sample_rate_in as f32)).sin();
        samples_in.push((s * 16384.0) as i16);
    }

    // 2. Resample from 48kHz to 16kHz
    let resampled = AudioResampler::resample(&samples_in, 48000, 16000);
    // 100ms at 16kHz is 1600 samples
    assert!((resampled.len() as i64 - 1600).abs() <= 5);

    // 3. Encode to standard RIFF/WAV format
    let format = AudioFormat::pcm16_mono(16000);
    let wav_bytes = WavEncoder::encode_pcm16(&resampled, &format);

    assert!(wav_bytes.len() > 44);
    assert_eq!(&wav_bytes[0..4], b"RIFF");
    assert_eq!(&wav_bytes[8..12], b"WAVE");
    assert_eq!(&wav_bytes[12..16], b"fmt ");
}

#[test]
fn test_voice_activity_detector_and_speech_segmenter() {
    use tagisan::engine::voice::audio::{AudioFormat, AudioFrame};
    use tagisan::engine::voice::vad::{SpeechSegmenter, VadConfig, VadState, VoiceActivityDetector};

    let config = VadConfig {
        energy_threshold: 0.05,
        zcr_speech_threshold: 0.05,
        min_speech_duration_ms: 60,
        silence_timeout_ms: 100,
        frame_duration_ms: 20,
    };

    let mut vad = VoiceActivityDetector::new(config.clone());
    let format = AudioFormat::pcm16_mono(16000);
    let mut segmenter = SpeechSegmenter::new(config, format);

    // 1. Silence frame (amplitude 0)
    let silence_samples = vec![0i16; 320]; // 20ms at 16kHz = 320 samples
    let (is_voice, state) = vad.process_frame(&silence_samples);
    assert!(!is_voice);
    assert_eq!(state, VadState::Silence);

    let silence_frame = AudioFrame::new(0, 0, silence_samples, format);
    let seg = segmenter.process_frame(&silence_frame);
    assert!(seg.is_none());

    // 2. Speech frame (amplitude 16000)
    let speech_samples = vec![16000i16; 320];
    let (is_voice2, _state2) = vad.process_frame(&speech_samples);
    assert!(is_voice2);

    let speech_frame = AudioFrame::new(1, 20, speech_samples, format);
    let seg2 = segmenter.process_frame(&speech_frame);
    assert!(seg2.is_none()); // Still gathering speech turn
}

#[tokio::test]
async fn test_voice_session_barge_in_and_gemini_live() {
    use tagisan::engine::voice::audio::{AudioFormat, AudioFrame};
    use tagisan::engine::voice::gemini_live::{
        build_audio_input_message, build_setup_message, BidiClientMessage, GeminiLiveConfig,
    };
    use tagisan::engine::voice::session::{VoiceEvent, VoiceSession, VoiceSessionConfig, VoiceSessionState};

    // 1. Test VoiceSession with barge-in interruption
    let mut session = VoiceSession::new(VoiceSessionConfig::default());

    // Agent starts speaking
    let agent_frame = AudioFrame::new(0, 0, vec![1000i16; 320], AudioFormat::pcm16_mono(16000));
    session.queue_agent_audio(agent_frame);
    assert_eq!(session.state(), VoiceSessionState::AgentSpeaking);

    // User interrupts with loud speech (barge-in)
    let user_speech = vec![25000i16; 320];
    let events = session.feed_input_pcm(&user_speech);

    // Assert barge-in detected and session transitioned to Interrupted
    assert!(
        events.iter().any(|e| matches!(e, VoiceEvent::Interrupted { .. })),
        "VoiceSession must emit Interrupted event when user interrupts agent!"
    );
    assert_eq!(session.state(), VoiceSessionState::Interrupted);

    // 2. Test GeminiLive message serialization
    let live_config = GeminiLiveConfig {
        model: "models/gemini-2.0-flash-exp".to_string(),
        voice_name: "Aoede".to_string(),
        api_key: "dummy-key".to_string(),
        system_instruction: Some("Be fast and concise.".to_string()),
        endpoint: "wss://test.endpoint".to_string(),
    };

    let setup_msg = build_setup_message(&live_config);
    match setup_msg {
        BidiClientMessage::Setup { setup } => {
            assert_eq!(setup.model, "models/gemini-2.0-flash-exp");
            assert!(setup.generation_config.is_some());
        }
        _ => panic!("Expected Setup message"),
    }

    let input_frame = AudioFrame::new(1, 100, vec![100i16, 200, 300], AudioFormat::pcm16_mono(16000));
    let audio_msg = build_audio_input_message(&input_frame);
    match audio_msg {
        BidiClientMessage::RealtimeInput { realtime_input } => {
            assert_eq!(realtime_input.media_chunks.len(), 1);
            assert!(realtime_input.media_chunks[0].mime_type.contains("audio/pcm"));
            assert!(!realtime_input.media_chunks[0].data.is_empty());
        }
        _ => panic!("Expected RealtimeInput message"),
    }
}

// ---------------------------------------------------------------------------
// 3. Interactive Visual Swarm & Computer-Use Web Dashboard Tests
// ---------------------------------------------------------------------------

#[test]
fn test_dashboard_state_and_ws_framing() {
    use tagisan::engine::dashboard::state::HostTelemetrySnapshot;
    use tagisan::engine::dashboard::ws::{compute_accept_key, encode_text_frame};

    // 1. Test WebSocket accept key against RFC 6455 test vector
    let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept_val = compute_accept_key(test_key);
    assert_eq!(accept_val, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");

    // 2. Test text frame encoding (Opcode 0x1)
    let payload = r#"{"type":"pong"}"#;
    let frame = encode_text_frame(payload);
    assert_eq!(frame[0], 0x81); // FIN bit + Opcode 1
    assert_eq!(frame[1], payload.len() as u8);
    assert_eq!(&frame[2..], payload.as_bytes());

    // 3. Test HostTelemetrySnapshot current readings
    let telem = HostTelemetrySnapshot::current();
    assert!(telem.mem_total_mb > 0);
    assert!(telem.cpu_cores >= 1);
}

#[tokio::test]
async fn test_dashboard_http_server_and_rest_endpoints() {
    use tagisan::engine::dashboard::server::SwarmDashboardServer;
    use tagisan::engine::dashboard::state::SharedDashboardState;

    let state = SharedDashboardState::new();
    state.update_screen_frame("data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==".to_string()).await;
    state.update_telemetry().await;

    // Bind to port 0 (ephemeral port chosen by OS)
    let server = SwarmDashboardServer::start("127.0.0.1", 0, state).await
        .expect("Failed to start SwarmDashboardServer");

    let port = server.port;
    assert!(port > 0);

    // Give the listener 50ms to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = reqwest::Client::new();

    // 1. Test GET /
    let res_index = client.get(&format!("http://127.0.0.1:{port}/"))
        .send()
        .await
        .expect("Failed to GET /");
    assert_eq!(res_index.status(), 200);
    let html = res_index.text().await.unwrap();
    assert!(html.contains("Tagisan Swarm & Computer-Use Dashboard"));

    // 2. Test GET /api/telemetry
    let res_telem = client.get(&format!("http://127.0.0.1:{port}/api/telemetry"))
        .send()
        .await
        .expect("Failed to GET /api/telemetry");
    assert_eq!(res_telem.status(), 200);
    let telem_json: serde_json::Value = res_telem.json().await.unwrap();
    assert!(telem_json["mem_total_mb"].as_u64().unwrap_or(0) > 0);

    // 3. Test GET /api/state
    let res_state = client.get(&format!("http://127.0.0.1:{port}/api/state"))
        .send()
        .await
        .expect("Failed to GET /api/state");
    assert_eq!(res_state.status(), 200);
    let state_json: serde_json::Value = res_state.json().await.unwrap();
    assert!(state_json["latest_screen_base64"].is_string());

    server.shutdown();
}

// ---------------------------------------------------------------------------
// 4. Unified Cross-Platform Sandboxing Tests
// ---------------------------------------------------------------------------

#[test]
fn test_cross_platform_sandbox_policy_and_isolation() {
    use tagisan::agent::cross_sandbox::{
        create_host_sandbox, PathAccess, SandboxPolicy,
    };

    let dir = TestTempDir::new("tgs_sandbox_test");
    let allowed_write = dir.path().to_path_buf();
    let forbidden_dir = PathBuf::from("/etc");

    let policy = SandboxPolicy {
        read_only_paths: vec![PathBuf::from("/usr"), PathBuf::from("/bin")],
        read_write_paths: vec![allowed_write.clone()],
        allow_network: false,
        max_memory_bytes: Some(512 * 1024 * 1024),
        max_cpu_shares: Some(512),
        allow_child_processes: true,
    };

    let sandbox = create_host_sandbox(policy);

    // Verify path access rules
    assert!(sandbox.is_path_allowed(&allowed_write, PathAccess::Write));
    assert!(sandbox.is_path_allowed(&allowed_write, PathAccess::Read));
    assert!(!sandbox.is_path_allowed(&forbidden_dir, PathAccess::Write));

    // Verify platform profile generation
    let profile = sandbox.generate_platform_profile();
    assert!(!profile.is_empty());
}

// ---------------------------------------------------------------------------
// 5. Automated Prompt Distillation & LoRA Dataset Generator Tests
// ---------------------------------------------------------------------------

#[test]
fn test_prompt_distillation_and_dataset_export() {
    use tagisan::engine::distill::{
        DatasetExporter, DistillationFormat, ReflexionDistiller,
    };
    use tagisan::tools::builtin::ReflexionEntry;

    let dir = TestTempDir::new("tgs_distill_test");

    // 1. Create synthetic episodic reflexion entries
    let mock_entries = vec![
        ReflexionEntry {
            id: "ref-001".to_string(),
            timestamp: "2026-09-20T10:00:00Z".to_string(),
            error_signature: "Kernel swap thrashing on Ollama model load".to_string(),
            root_cause: "RAM requirement (6GB) exceeded available RAM (1GB)".to_string(),
            fix_applied: "Invoked HostMemoryGovernor and clamped num_ctx to 512".to_string(),
            preventative_invariant: "Always run pre-flight memory admission checks before loading".to_string(),
            tags: vec!["ollama".to_string(), "memory".to_string()],
        },
        ReflexionEntry {
            id: "ref-002".to_string(),
            timestamp: "2026-09-20T10:05:00Z".to_string(),
            error_signature: "Cargo build exhausted all 2 CPU cores".to_string(),
            root_cause: "Parallel compilation without -j 1 flag".to_string(),
            fix_applied: "Passed -j 1 to all cargo commands".to_string(),
            preventative_invariant: "Always pass -j 1 to cargo commands on dual-core systems".to_string(),
            tags: vec!["cargo".to_string(), "cpu".to_string()],
        },
    ];

    let distiller = ReflexionDistiller::new(mock_entries);

    // 2. Export Alpaca format
    let alpaca_path = dir.path().join("alpaca.jsonl");
    let count_alpaca = DatasetExporter::export_to_jsonl(
        &distiller,
        DistillationFormat::Alpaca,
        &alpaca_path,
    ).expect("Failed to export Alpaca JSONL");
    assert_eq!(count_alpaca, 2);
    assert!(alpaca_path.exists());

    // 3. Export ShareGPT format
    let sharegpt_path = dir.path().join("sharegpt.jsonl");
    let count_sharegpt = DatasetExporter::export_to_jsonl(
        &distiller,
        DistillationFormat::ShareGpt,
        &sharegpt_path,
    ).expect("Failed to export ShareGPT JSONL");
    assert_eq!(count_sharegpt, 2);
    assert!(sharegpt_path.exists());

    // 4. Export DPO format
    let dpo_path = dir.path().join("dpo.jsonl");
    let count_dpo = DatasetExporter::export_to_jsonl(
        &distiller,
        DistillationFormat::Dpo,
        &dpo_path,
    ).expect("Failed to export DPO JSONL");
    assert_eq!(count_dpo, 2);
    assert!(dpo_path.exists());

    // 5. Verify line count and format correctness
    let dpo_contents = std::fs::read_to_string(&dpo_path).unwrap();
    let dpo_lines: Vec<&str> = dpo_contents.lines().collect();
    assert_eq!(dpo_lines.len(), 2);
    assert!(dpo_lines[0].contains("Kernel swap thrashing"));
    assert!(dpo_lines[0].contains("chosen"));
    assert!(dpo_lines[0].contains("rejected"));
}
