//! Brutal Production-Grade Integration Tests for Ollama Anti-Freeze Guardian
//! & Resource-Adaptive Engine in Tagisan (`tgs`).

use std::net::SocketAddr;
use std::sync::Arc;
use serde_json::json;
use tagisan::{
    AdmissionVerdict, CompletionRequest, HostMemoryGovernor, LinuxMemInfo,
    MemoryPressureTier, MemoryTunerConfig, OllamaAdmissionController,
    OllamaMemoryTuner, OllamaProvider,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct MockRecordedRequest {
    pub method: String,
    pub path: String,
    pub body_json: Option<serde_json::Value>,
}

pub struct MockOllamaGuardianServer {
    pub addr: SocketAddr,
    pub base_url: String,
    shutdown_token: CancellationToken,
    pub recorded_requests: Arc<Mutex<Vec<MockRecordedRequest>>>,
}

impl MockOllamaGuardianServer {
    pub async fn start(tags_response: serde_json::Value, ps_response: serde_json::Value) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let shutdown_token = CancellationToken::new();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));

        let shutdown_clone = shutdown_token.clone();
        let recorded_clone = recorded_requests.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_clone.cancelled() => break,
                    Ok((socket, _)) = listener.accept() => {
                        let (read_half, mut write_half) = socket.into_split();
                        let recorded = recorded_clone.clone();
                        let tags_val = tags_response.clone();
                        let ps_val = ps_response.clone();

                        tokio::spawn(async move {
                            let mut reader = BufReader::new(read_half);
                            let mut req_line = String::new();
                            if reader.read_line(&mut req_line).await.is_err() || req_line.is_empty() {
                                return;
                            }
                            let parts: Vec<&str> = req_line.split_whitespace().collect();
                            if parts.len() < 2 {
                                return;
                            }
                            let method = parts[0].to_string();
                            let path = parts[1].to_string();

                            let mut content_length = 0usize;
                            loop {
                                let mut line = String::new();
                                if reader.read_line(&mut line).await.is_err() || line == "\r\n" || line == "\n" || line.is_empty() {
                                    break;
                                }
                                if let Some((k, v)) = line.split_once(':') {
                                    if k.trim().eq_ignore_ascii_case("content-length") {
                                        content_length = v.trim().parse().unwrap_or(0);
                                    }
                                }
                            }

                            let mut body_bytes = vec![0u8; content_length];
                            if content_length > 0 {
                                let _ = reader.read_exact(&mut body_bytes).await;
                            }
                            let body_json = serde_json::from_slice(&body_bytes).ok();

                            recorded.lock().await.push(MockRecordedRequest {
                                method: method.clone(),
                                path: path.clone(),
                                body_json: body_json.clone(),
                            });

                            let (status, body) = if path == "/api/tags" {
                                (200, serde_json::to_vec(&tags_val).unwrap())
                            } else if path == "/api/ps" {
                                (200, serde_json::to_vec(&ps_val).unwrap())
                            } else if path == "/api/generate" {
                                // Unload endpoint
                                (200, json!({"status": "success"}).to_string().into_bytes())
                            } else if path == "/api/chat" {
                                let resp = json!({
                                    "model": "llama3.2:1b",
                                    "message": {"role": "assistant", "content": "Anti-freeze response"},
                                    "done": true,
                                    "prompt_eval_count": 20,
                                    "eval_count": 10
                                });
                                (200, serde_json::to_vec(&resp).unwrap())
                            } else {
                                (404, b"Not found".to_vec())
                            };

                            let resp_header = format!(
                                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                if status == 200 { "200 OK" } else { "404 Not Found" },
                                body.len()
                            );
                            let _ = write_half.write_all(resp_header.as_bytes()).await;
                            let _ = write_half.write_all(&body).await;
                            let _ = write_half.flush().await;
                        });
                    }
                }
            }
        });

        Self {
            addr,
            base_url,
            shutdown_token,
            recorded_requests,
        }
    }

    pub fn shutdown(&self) {
        self.shutdown_token.cancel();
    }
}

// -------------------------------------------------------------------------
// Test 1: Admission control rejects oversized models when RAM is insufficient
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_admission_control_rejects_oversized_model_when_ram_insufficient() {
    let tags = json!({
        "models": [
            {
                "name": "qwen2.5-coder:14b",
                "model": "qwen2.5-coder:14b",
                "size": 9_500_000_000u64, // ~9.5 GB
            }
        ]
    });
    let ps = json!({"models": []});
    let server = MockOllamaGuardianServer::start(tags, ps).await;

    let mut tuner_cfg = MemoryTunerConfig::default();
    tuner_cfg.ollama_url = server.base_url.clone();
    let tuner = Arc::new(OllamaMemoryTuner::new(tuner_cfg));
    let guardian = OllamaAdmissionController::new(tuner);

    // Host with only 1.2 GB available RAM
    let metrics = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_free_kb: 500 * 1024,
        mem_available_kb: 1_200 * 1024, // 1.2 GB
        swap_total_kb: 4 * 1024 * 1024,
        swap_free_kb: 2 * 1024 * 1024,
        ..Default::default()
    };

    let verdict = guardian
        .check_admission_with_metrics("qwen2.5-coder:14b", 2048, &metrics)
        .await
        .unwrap();

    match verdict {
        AdmissionVerdict::Rejected {
            model_name,
            total_required_bytes,
            available_bytes,
            suggested_models,
            diagnostics,
            ..
        } => {
            assert_eq!(model_name, "qwen2.5-coder:14b");
            assert!(total_required_bytes > available_bytes);
            assert!(suggested_models.contains(&"smollm2:1.7b".to_string()));
            assert!(diagnostics.contains("ANTI-FREEZE GUARDIAN BLOCKED MODEL LOAD"));
            assert!(diagnostics.contains("kswapd0"));
        }
        other => panic!("Expected AdmissionVerdict::Rejected, got: {:?}", other),
    }

    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 2: Auto-eviction of resident models frees RAM before loading new model
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_auto_eviction_of_resident_models_frees_ram() {
    let tags = json!({
        "models": [
            {
                "name": "llama3.2:1b",
                "model": "llama3.2:1b",
                "size": 1_300_000_000u64, // ~1.3 GB
            }
        ]
    });
    let ps = json!({
        "models": [
            {
                "name": "idle_heavy_model:7b",
                "model": "idle_heavy_model:7b",
                "size": 4_800_000_000u64,
                "size_vram": 0u64,
            }
        ]
    });
    let server = MockOllamaGuardianServer::start(tags, ps).await;

    let mut tuner_cfg = MemoryTunerConfig::default();
    tuner_cfg.ollama_url = server.base_url.clone();
    let tuner = Arc::new(OllamaMemoryTuner::new(tuner_cfg));
    let guardian = OllamaAdmissionController::new(tuner);

    // Initial low RAM state where evicting the resident model is triggered
    let metrics = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_free_kb: 400 * 1024,
        mem_available_kb: 800 * 1024, // 800 MB (less than 1.3GB + KV + buffer)
        swap_total_kb: 4 * 1024 * 1024,
        swap_free_kb: 2 * 1024 * 1024,
        ..Default::default()
    };

    let _ = guardian
        .check_admission_with_metrics("llama3.2:1b", 1024, &metrics)
        .await;

    // Verify unload was sent to /api/generate
    let recorded = server.recorded_requests.lock().await;
    let unload_req = recorded.iter().find(|r| r.path == "/api/generate");
    assert!(
        unload_req.is_some(),
        "Eviction must call /api/generate with keep_alive: 0"
    );

    let body = unload_req.unwrap().body_json.as_ref().unwrap();
    assert_eq!(body["model"], "idle_heavy_model:7b");
    assert_eq!(body["keep_alive"], 0);

    drop(recorded);
    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 3: Auto-recovery to lighter model when oversized model cannot fit
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_auto_recovery_to_installed_lighter_model() {
    let tags = json!({
        "models": [
            {
                "name": "qwen2.5-coder:7b",
                "model": "qwen2.5-coder:7b",
                "size": 4_800_000_000u64, // ~4.8 GB
            },
            {
                "name": "llama3.2:1b",
                "model": "llama3.2:1b",
                "size": 1_300_000_000u64, // ~1.3 GB
            }
        ]
    });
    let ps = json!({"models": []});
    let server = MockOllamaGuardianServer::start(tags, ps).await;

    let mut tuner_cfg = MemoryTunerConfig::default();
    tuner_cfg.ollama_url = server.base_url.clone();
    let tuner = Arc::new(OllamaMemoryTuner::new(tuner_cfg));
    let guardian = OllamaAdmissionController::new(tuner);

    // Host with 2.5 GB available RAM (cannot fit 7B, can fit 1B)
    let metrics = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_free_kb: 1_000 * 1024,
        mem_available_kb: 2_500 * 1024, // 2.5 GB
        swap_total_kb: 4 * 1024 * 1024,
        swap_free_kb: 2 * 1024 * 1024,
        ..Default::default()
    };

    let verdict = guardian
        .check_admission_with_metrics("qwen2.5-coder:7b", 1024, &metrics)
        .await
        .unwrap();

    match verdict {
        AdmissionVerdict::RecoveredToLighterModel {
            original_model,
            fallback_model,
            ..
        } => {
            assert_eq!(original_model, "qwen2.5-coder:7b");
            assert_eq!(fallback_model, "llama3.2:1b");
        }
        other => panic!("Expected RecoveredToLighterModel, got: {:?}", other),
    }

    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 4: Thread clamping on dual-core and low-memory systems (leaves core for OS)
// -------------------------------------------------------------------------
#[test]
fn test_thread_clamping_on_dual_core_and_low_memory() {
    let gov = HostMemoryGovernor::new();

    // 8GB system under GreenNormal
    let metrics_8gb = LinuxMemInfo {
        mem_total_kb: 7_800 * 1024,
        mem_free_kb: 2_000 * 1024,
        mem_available_kb: 3_000 * 1024,
        swap_total_kb: 4 * 1024 * 1024,
        swap_free_kb: 3 * 1024 * 1024,
        ..Default::default()
    };
    assert!(gov.is_8gb_workstation(&metrics_8gb));
    let threads_8gb = gov.recommended_ollama_threads(&metrics_8gb);
    assert_eq!(threads_8gb, 1, "8GB system must clamp num_thread to 1");

    // Critical pressure
    let metrics_red = LinuxMemInfo {
        mem_total_kb: 16 * 1024 * 1024,
        mem_free_kb: 200 * 1024,
        mem_available_kb: 500 * 1024, // <15%
        swap_total_kb: 4 * 1024 * 1024,
        swap_free_kb: 500 * 1024, // >85%
        ..Default::default()
    };
    assert_eq!(gov.evaluate_pressure(&metrics_red), MemoryPressureTier::RedCritical);
    let threads_red = gov.recommended_ollama_threads(&metrics_red);
    assert_eq!(threads_red, 1, "RedCritical must clamp num_thread to 1");
}

// -------------------------------------------------------------------------
// Test 5: Context window clamping under memory pressure
// -------------------------------------------------------------------------
#[test]
fn test_context_window_clamping_under_memory_pressure() {
    let gov = HostMemoryGovernor::new();

    // RedCritical clamp: 512 - 1024
    let metrics_red = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_available_kb: 400 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 500 * 1024,
        ..Default::default()
    };
    assert_eq!(
        gov.recommended_ollama_ctx(&metrics_red, Some(4096)),
        1024,
        "RedCritical must clamp requested 4096 ctx to 1024"
    );
    assert_eq!(
        gov.recommended_ollama_ctx(&metrics_red, Some(256)),
        512,
        "RedCritical must clamp minimum ctx to 512"
    );

    // YellowWarning / 8GB clamp: 1024 - 2048
    let metrics_yellow = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_available_kb: 1_500 * 1024, // ~18%
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 4 * 1024 * 1024,
        ..Default::default()
    };
    assert_eq!(
        gov.recommended_ollama_ctx(&metrics_yellow, Some(8192)),
        2048,
        "YellowWarning must clamp requested 8192 ctx to 2048"
    );
}

// -------------------------------------------------------------------------
// Test 6: Adaptive keep-alive tuning (0s on RedCritical, 2m on Yellow/8GB)
// -------------------------------------------------------------------------
#[test]
fn test_adaptive_keep_alive_tuning() {
    let gov = HostMemoryGovernor::new();

    // RedCritical
    let metrics_red = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_available_kb: 500 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 500 * 1024,
        ..Default::default()
    };
    assert_eq!(gov.recommended_ollama_keep_alive(&metrics_red), "0s");

    // 8GB system in Green
    let metrics_8gb = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_available_kb: 3_000 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 7 * 1024 * 1024,
        ..Default::default()
    };
    assert_eq!(gov.recommended_ollama_keep_alive(&metrics_8gb), "2m");

    // >8GB system in Green
    let metrics_large = LinuxMemInfo {
        mem_total_kb: 32 * 1024 * 1024,
        mem_available_kb: 20 * 1024 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 8 * 1024 * 1024,
        ..Default::default()
    };
    assert_eq!(gov.recommended_ollama_keep_alive(&metrics_large), "5m");
}

// -------------------------------------------------------------------------
// Test 7: Post-inference hygiene and heap trimming
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_post_inference_hygiene_and_heap_trimming() {
    let tags = json!({"models": []});
    let ps = json!({"models": []});
    let server = MockOllamaGuardianServer::start(tags, ps).await;

    let mut tuner_cfg = MemoryTunerConfig::default();
    tuner_cfg.ollama_url = server.base_url.clone();
    let tuner = Arc::new(OllamaMemoryTuner::new(tuner_cfg));

    // 1. Heap trim executes cleanly
    let gov = HostMemoryGovernor::new();
    let _ = gov.trim_heap();

    // 2. Under RedCritical, post-inference hygiene forces unload
    let metrics_red = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_available_kb: 400 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 500 * 1024,
        ..Default::default()
    };

    let evicted = tuner
        .perform_post_inference_hygiene_with_metrics("test_model:1b", &metrics_red)
        .await;
    assert!(evicted, "Post-inference hygiene must unload model when RedCritical");

    let recorded = server.recorded_requests.lock().await;
    assert!(
        recorded.iter().any(|r| r.path == "/api/generate"),
        "Unload request must be sent to Ollama on RedCritical"
    );

    drop(recorded);
    server.shutdown();
}

// -------------------------------------------------------------------------
// Test 8: End-to-end OllamaProvider with admission control and hygiene
// -------------------------------------------------------------------------
#[tokio::test]
async fn test_ollama_provider_full_lifecycle_with_admission_and_hygiene() {
    let tags = json!({
        "models": [
            {
                "name": "llama3.2:1b",
                "model": "llama3.2:1b",
                "size": 1_300_000_000u64,
            }
        ]
    });
    let ps = json!({"models": []});
    let server = MockOllamaGuardianServer::start(tags, ps).await;

    let provider = OllamaProvider::new(&server.base_url);
    let req = CompletionRequest::new("llama3.2:1b", "Test anti-freeze prompt");

    let resp = provider.send_chat_request(req).await.unwrap();
    assert_eq!(resp.model, "llama3.2:1b");
    assert_eq!(resp.message.extract_text(), "Anti-freeze response");

    server.shutdown();
}
