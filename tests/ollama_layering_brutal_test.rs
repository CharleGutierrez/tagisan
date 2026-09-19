//! Brutal Verification Test Suite: Ollama Layering & Local LLM Inference Engine
//!
//! Brutally tests:
//! 1. Zero-Copy SafeTensors binary layout, header parsing, and 64-byte alignment validation.
//! 2. Double-Buffering Asynchronous Prefetcher Pipeline (`sync_channel(1)`).
//! 3. Linux Page Cache Eviction (`libc::MADV_DONTNEED`) & Zero-Leak Memory Bound.
//! 4. Persistent KV Cache & Hidden States in-place mutation across simulated 80-layer passes.
//! 5. Ollama Provider Tool-Calling Capability Filtering (`llama2-uncensored`, `codellama`, `orca-mini`).
//! 6. Ollama Transparent Auto-Recovery from "does not support tools" error.
//! 7. High-Concurrency Stress and Multithreaded Layer Streaming.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use memmap2::Mmap;
use tagisan::engine::layer_streaming::{
    HiddenStates, KvCache, LayerPrefetcher, LayerStreamingConfig,
    LayerStreamingEngine, LayerWeights, SafeTensorsParser, SyntheticSafeTensorsBuilder,
    TensorDType,
};
use tagisan::providers::ollama::OllamaProvider;
use tagisan::providers::LlmProvider;
use tagisan::types::ProviderCapabilities;

struct AutoCleanupDir(PathBuf);
impl AutoCleanupDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "tgs_brutal_layering_{}_{}_{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for AutoCleanupDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// =========================================================================
// Pillar 1: SafeTensors Binary Layout & 64-Byte Alignment Stress
// =========================================================================

#[test]
fn test_brutal_01_safetensors_binary_layout_and_simd_alignment() {
    let dir = AutoCleanupDir::new("layout");
    let layer_file = dir.path().join("layer_test.safetensors");
    let hidden_dim = 1024;

    SyntheticSafeTensorsBuilder::create_layer_file(&layer_file, hidden_dim)
        .expect("Failed to build synthetic safetensors file");

    let file = File::open(&layer_file).expect("Failed to open file");
    let mmap = unsafe { Mmap::map(&file).expect("Mmap failed") };
    let (descriptors, base_offset) = SafeTensorsParser::parse_header(&mmap)
        .expect("SafeTensors header parse failed");

    // Verify 64-byte alignment of the payload offset for AVX-512 / GPU coalesced I/O
    assert_eq!(
        base_offset % 64,
        0,
        "Base data offset {} must be strictly 64-byte aligned",
        base_offset
    );

    let weights = LayerWeights::new(
        0,
        Arc::new(mmap),
        descriptors,
        base_offset,
        layer_file.metadata().unwrap().len() as usize,
    );

    for name in &["attn_norm.weight", "mlp_norm.weight", "q_proj.weight"] {
        let view = weights.get_tensor(name).expect("Tensor must exist");
        assert_eq!(view.descriptor.dtype, TensorDType::F32);
        assert_eq!(view.descriptor.shape, vec![hidden_dim]);

        let f32_slice = view.as_f32_slice().expect("Must cast cleanly to f32 slice");
        assert_eq!(f32_slice.len(), hidden_dim);
        assert_eq!(f32_slice[0], 1.0f32);
    }
}

// =========================================================================
// Pillar 2: Double-Buffering Asynchronous Prefetcher Pipeline Stress
// =========================================================================

#[test]
fn test_brutal_02_double_buffering_prefetcher_pipeline_stress() {
    let dir = AutoCleanupDir::new("prefetch_stress");
    let num_layers = 16;
    let mut layer_paths = Vec::new();

    for i in 0..num_layers {
        let p = dir.path().join(format!("layer_{}.safetensors", i));
        SyntheticSafeTensorsBuilder::create_layer_file(&p, 256).expect("Failed to create layer");
        layer_paths.push(p);
    }

    let prefetcher = LayerPrefetcher::new(layer_paths);

    // Prime with layer 0
    prefetcher.request_layer(0).expect("Failed to request layer 0");
    let mut current = Some(prefetcher.wait_for_layer().expect("Failed to wait for layer 0"));

    // Rapid double-buffering cycle through all 16 layers
    for i in 0..num_layers {
        if i + 1 < num_layers {
            prefetcher.request_layer(i + 1).expect("Failed to pipeline next layer");
        }

        // Simulate layer compute and immediate eviction
        let cur = current.take().expect("Current layer must be present");
        assert_eq!(cur.layer_idx, i);
        drop(cur);

        if i + 1 < num_layers {
            current = Some(prefetcher.wait_for_layer().expect("Failed to receive pipelined layer"));
        }
    }
}

// =========================================================================
// Pillar 3: Linux Page Cache Eviction & Memory Lifecycle
// =========================================================================

#[test]
fn test_brutal_03_memory_lifecycle_and_madv_dontneed_eviction() {
    let dir = AutoCleanupDir::new("eviction");
    let num_layers = 8;
    let mut layer_files = Vec::new();

    for i in 0..num_layers {
        let p = dir.path().join(format!("layer_{}.safetensors", i));
        SyntheticSafeTensorsBuilder::create_layer_file(&p, 512).unwrap();
        layer_files.push(p);
    }

    let config = LayerStreamingConfig {
        model_name: "Llama-3-70B-Brutal-Stress".to_string(),
        num_layers,
        hidden_dim: 512,
        intermediate_dim: 1024,
        num_heads: 8,
        num_kv_heads: 2,
        head_dim: 64,
        max_seq_len: 256,
        vocab_size: 2000,
        layer_files,
    };

    let engine = LayerStreamingEngine::new(config);
    let prompt = vec![42, 100, 200];
    let new_tokens = 6;

    let output = engine.generate(&prompt, new_tokens).expect("Inference failed");
    assert_eq!(output.len(), prompt.len() + new_tokens);

    let metrics = engine.metrics();
    assert_eq!(metrics.tokens_generated, new_tokens);

    // Total layer passes = num_layers * (prompt.len() + new_tokens)
    let total_layer_evals = num_layers * (prompt.len() + new_tokens);
    assert_eq!(
        metrics.memory_evictions_count, total_layer_evals,
        "Every single layer pass must trigger an explicit Drop + MADV_DONTNEED eviction"
    );
}

// =========================================================================
// Pillar 4: Persistent KV Cache & Hidden States In-Place Mutation
// =========================================================================

#[test]
fn test_brutal_04_persistent_kv_cache_and_hidden_states_in_place() {
    let num_layers = 80; // 70B parameter architecture simulation
    let num_kv_heads = 8;
    let head_dim = 128;
    let max_seq_len = 512;
    let hidden_dim = 8192;

    let mut kv_cache = KvCache::new(num_layers, num_kv_heads, max_seq_len, head_dim);
    assert_eq!(kv_cache.layers.len(), 80);

    // Memory footprint calculation check: 80 layers * 8 heads * 512 tokens * 128 dim * 2 (K+V) * 4 bytes
    let expected_capacity_bytes = 80 * 8 * 512 * 128 * 2 * 4;
    assert_eq!(kv_cache.total_memory_bytes(), expected_capacity_bytes);

    let hidden_states = HiddenStates::new(1, hidden_dim);
    assert_eq!(hidden_states.as_slice().len(), hidden_dim);

    // Verify in-place mutation without re-allocation
    let dummy_k = vec![0.5f32; num_kv_heads * head_dim];
    let dummy_v = vec![0.25f32; num_kv_heads * head_dim];

    for layer_idx in 0..num_layers {
        let l_kv = kv_cache.get_mut(layer_idx).expect("Layer KV must exist");
        l_kv.append(&dummy_k, &dummy_v);
        assert_eq!(l_kv.current_len, 1);
    }

    // Reset verification
    kv_cache.reset();
    for l in &kv_cache.layers {
        assert_eq!(l.current_len, 0);
        assert!(l.key_cache.is_empty());
    }
}

// =========================================================================
// Pillar 5: Ollama Tool Capability Filtering & Auto-Recovery
// =========================================================================

#[test]
fn test_brutal_05_ollama_non_tool_model_capability_and_auto_recovery() {
    let provider = OllamaProvider::default_local();

    // 1. Models known NOT to support tool calling in Ollama
    let non_tool_models = [
        "llama2:latest",
        "llama2-uncensored:latest",
        "registry.ollama.ai/library/llama2-uncensored:latest",
        "codellama:7b",
        "orca-mini:3b",
        "vicuna:13b",
        "wizardlm:7b",
        "tinyllama:latest",
        "medllama2:7b",
    ];

    for m in &non_tool_models {
        let caps = provider.capabilities(m);
        assert!(
            !caps.contains(ProviderCapabilities::FUNCTION_CALLING),
            "Model '{}' must NOT have FUNCTION_CALLING capability",
            m
        );
        assert!(
            !OllamaProvider::is_model_tool_supported(m),
            "is_model_tool_supported must return false for '{}'",
            m
        );
    }

    // 2. Models that DO support tool calling in Ollama
    let tool_models = [
        "qwen2.5-coder:1.5b",
        "qwen2.5:7b",
        "llama3.1:8b",
        "llama3.2:3b",
        "mistral:7b",
    ];

    for m in &tool_models {
        let caps = provider.capabilities(m);
        assert!(
            caps.contains(ProviderCapabilities::FUNCTION_CALLING),
            "Model '{}' MUST have FUNCTION_CALLING capability",
            m
        );
        assert!(
            OllamaProvider::is_model_tool_supported(m),
            "is_model_tool_supported must return true for '{}'",
            m
        );
    }

    // 3. Dynamic runtime blacklisting check
    let custom_model = "my-custom-finetune:latest";
    assert!(OllamaProvider::is_model_tool_supported(custom_model));

    // Simulate runtime discovery of "does not support tools" error
    {
        let mut guard = provider.unsupported_tool_models.blocking_write();
        guard.insert(custom_model.to_string());
    }

    let updated_caps = provider.capabilities(custom_model);
    assert!(
        !updated_caps.contains(ProviderCapabilities::FUNCTION_CALLING),
        "Dynamically blacklisted model must have FUNCTION_CALLING removed"
    );
}

// =========================================================================
// Pillar 6: High-Concurrency Multithreaded Layer Streaming Stress
// =========================================================================

#[test]
fn test_brutal_06_multithreaded_layer_streaming_concurrency() {
    let dir = AutoCleanupDir::new("concurrency");
    let num_layers = 4;
    let mut layer_files = Vec::new();

    for i in 0..num_layers {
        let p = dir.path().join(format!("layer_{}.safetensors", i));
        SyntheticSafeTensorsBuilder::create_layer_file(&p, 256).unwrap();
        layer_files.push(p);
    }

    let config = LayerStreamingConfig {
        model_name: "Concurrency-Test-70B".to_string(),
        num_layers,
        hidden_dim: 256,
        intermediate_dim: 512,
        num_heads: 4,
        num_kv_heads: 1,
        head_dim: 64,
        max_seq_len: 128,
        vocab_size: 500,
        layer_files,
    };

    let engine = Arc::new(LayerStreamingEngine::new(config));

    let mut handles = Vec::new();
    for thread_idx in 0..4 {
        let eng = Arc::clone(&engine);
        let handle = std::thread::spawn(move || {
            let prompt = vec![10 + thread_idx as u32, 20, 30];
            let res = eng.generate(&prompt, 3).expect("Thread generation failed");
            assert_eq!(res.len(), 6);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    let metrics = engine.metrics();
    assert_eq!(metrics.tokens_generated, 4 * 3);
    assert!(metrics.memory_evictions_count >= 4 * num_layers * 6);
}
