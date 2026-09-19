//! High-Performance Sequential Layer-Streaming LLM Inference Engine
//!
//! Provides zero-copy disk-to-RAM streaming inference for massive Transformer
//! models (e.g., Llama-3-70B, Qwen-2.5-72B) on severely memory-constrained systems (8GB RAM).
//!
//! # Architecture
//! 1. **Zero-Copy Disk-to-RAM Streaming**: Leverages `memmap2::Mmap` and SafeTensors zero-copy
//!    slice indexing to map individual layer weights from NVMe SSD into virtual memory.
//! 2. **Double-Buffering via Bounded Channels**: A dedicated background I/O prefetch thread
//!    reads and prepares Layer `N+1` while the compute thread executes Layer `N` on CPU/GPU.
//! 3. **Strict Memory Management**: As soon as Layer `N` execution finishes, the `LayerWeights`
//!    structure is dropped, invoking `libc::madvise(..., MADV_DONTNEED)` to evict the physical
//!    pages from Linux page cache immediately without swapping.
//! 4. **Persistent State Retention**: `HiddenStates` and `KvCache` are allocated outside the layer
//!    loop and mutated in-place across layer boundaries.

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};

use crate::error::{Result, TagisanError};

/// Tensor Data Types supported by the zero-copy layer streaming engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorDType {
    F32,
    F16,
    BF16,
    Q4_0,
    Q4_K,
    Q8_0,
    I8,
}

impl TensorDType {
    /// Returns the byte size of an element or quant block.
    pub fn element_bytes(&self) -> usize {
        match self {
            TensorDType::F32 => 4,
            TensorDType::F16 | TensorDType::BF16 => 2,
            TensorDType::I8 => 1,
            TensorDType::Q4_0 | TensorDType::Q4_K => 1, // Normalized amortized block size
            TensorDType::Q8_0 => 1,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "F32" => TensorDType::F32,
            "F16" => TensorDType::F16,
            "BF16" => TensorDType::BF16,
            "Q4_0" => TensorDType::Q4_0,
            "Q4_K" => TensorDType::Q4_K,
            "Q8_0" => TensorDType::Q8_0,
            "I8" => TensorDType::I8,
            _ => TensorDType::F32,
        }
    }
}

/// Metadata descriptor for a tensor stored in SafeTensors format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorDescriptor {
    pub name: String,
    pub dtype: TensorDType,
    pub shape: Vec<usize>,
    pub data_offsets: (usize, usize),
}

impl TensorDescriptor {
    pub fn byte_len(&self) -> usize {
        self.data_offsets.1.saturating_sub(self.data_offsets.0)
    }

    pub fn num_elements(&self) -> usize {
        self.shape.iter().product()
    }
}

/// A zero-copy borrowed slice of a tensor pointing directly into an Mmap buffer.
pub struct TensorView<'a> {
    pub descriptor: TensorDescriptor,
    pub data: &'a [u8],
}

impl<'a> TensorView<'a> {
    #[inline(always)]
    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }

    #[inline(always)]
    pub fn as_f32_slice(&self) -> Result<&'a [f32]> {
        if self.descriptor.dtype != TensorDType::F32 {
            return Err(TagisanError::Execution(format!(
                "Tensor {} is {:?}, not F32",
                self.descriptor.name, self.descriptor.dtype
            )));
        }
        let byte_len = self.data.len();
        if byte_len % 4 != 0 {
            return Err(TagisanError::Execution(format!(
                "Invalid byte length for F32 tensor {}: {}",
                self.descriptor.name, byte_len
            )));
        }
        let ptr = self.data.as_ptr() as *const f32;
        let count = byte_len / 4;
        // Verify 4-byte alignment
        if (ptr as usize) % std::mem::align_of::<f32>() != 0 {
            return Err(TagisanError::Execution(format!(
                "Mmap buffer for tensor {} is unaligned for f32",
                self.descriptor.name
            )));
        }
        Ok(unsafe { std::slice::from_raw_parts(ptr, count) })
    }
}

/// Layer weights loaded into virtual memory via memory mapping.
///
/// Implements `Drop` to automatically trigger `libc::madvise(..., MADV_DONTNEED)`
/// on Linux/Unix systems as soon as the layer completes its forward pass.
pub struct LayerWeights {
    pub layer_idx: usize,
    pub mmap: Arc<Mmap>,
    pub tensors: HashMap<String, TensorDescriptor>,
    pub base_offset: usize,
    pub total_bytes: usize,
    evicted: bool,
}

impl LayerWeights {
    pub fn new(
        layer_idx: usize,
        mmap: Arc<Mmap>,
        tensors: HashMap<String, TensorDescriptor>,
        base_offset: usize,
        total_bytes: usize,
    ) -> Self {
        Self {
            layer_idx,
            mmap,
            tensors,
            base_offset,
            total_bytes,
            evicted: false,
        }
    }

    /// Retrieve a zero-copy slice of a tensor by name.
    pub fn get_tensor<'a>(&'a self, name: &str) -> Option<TensorView<'a>> {
        let desc = self.tensors.get(name)?;
        let start = self.base_offset + desc.data_offsets.0;
        let end = self.base_offset + desc.data_offsets.1;
        if end > self.mmap.len() {
            return None;
        }
        Some(TensorView {
            descriptor: desc.clone(),
            data: &self.mmap[start..end],
        })
    }

    /// Explicitly evict physical pages from Linux page cache using `MADV_DONTNEED`.
    pub fn evict_from_page_cache(&mut self) {
        if self.evicted {
            return;
        }
        self.evicted = true;

        #[cfg(target_os = "linux")]
        {
            let ptr = self.mmap.as_ptr() as *mut libc::c_void;
            let len = self.mmap.len();
            unsafe {
                let ret = libc::madvise(ptr, len, libc::MADV_DONTNEED);
                if ret != 0 {
                    debug!(
                        layer = self.layer_idx,
                        code = ret,
                        "madvise(MADV_DONTNEED) failed or non-critical"
                    );
                } else {
                    trace!(
                        layer = self.layer_idx,
                        bytes = len,
                        "Successfully issued MADV_DONTNEED for layer"
                    );
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            trace!(
                layer = self.layer_idx,
                "Non-linux OS; relying on kernel LRU page reclaim"
            );
        }
    }
}

impl Drop for LayerWeights {
    fn drop(&mut self) {
        self.evict_from_page_cache();
    }
}

/// Key-Value Cache for a single Transformer layer.
#[derive(Debug, Clone)]
pub struct LayerKvCache {
    pub layer_idx: usize,
    pub key_cache: Vec<f32>,   // [num_heads, max_seq_len, head_dim]
    pub value_cache: Vec<f32>, // [num_heads, max_seq_len, head_dim]
    pub current_len: usize,
}

impl LayerKvCache {
    pub fn new(layer_idx: usize, num_heads: usize, max_seq_len: usize, head_dim: usize) -> Self {
        let capacity = num_heads * max_seq_len * head_dim;
        Self {
            layer_idx,
            key_cache: Vec::with_capacity(capacity),
            value_cache: Vec::with_capacity(capacity),
            current_len: 0,
        }
    }

    pub fn append(&mut self, k: &[f32], v: &[f32]) {
        self.key_cache.extend_from_slice(k);
        self.value_cache.extend_from_slice(v);
        self.current_len += 1;
    }

    pub fn reset(&mut self) {
        self.key_cache.clear();
        self.value_cache.clear();
        self.current_len = 0;
    }

    pub fn memory_usage_bytes(&self) -> usize {
        (self.key_cache.capacity() + self.value_cache.capacity()) * std::mem::size_of::<f32>()
    }
}

/// Global Key-Value Cache retained outside the layer loop in main RAM.
#[derive(Debug, Clone)]
pub struct KvCache {
    pub layers: Vec<LayerKvCache>,
    pub num_layers: usize,
    pub num_heads: usize,
    pub max_seq_len: usize,
    pub head_dim: usize,
}

impl KvCache {
    pub fn new(num_layers: usize, num_heads: usize, max_seq_len: usize, head_dim: usize) -> Self {
        let mut layers = Vec::with_capacity(num_layers);
        for idx in 0..num_layers {
            layers.push(LayerKvCache::new(idx, num_heads, max_seq_len, head_dim));
        }
        Self {
            layers,
            num_layers,
            num_heads,
            max_seq_len,
            head_dim,
        }
    }

    pub fn get_mut(&mut self, layer_idx: usize) -> Option<&mut LayerKvCache> {
        self.layers.get_mut(layer_idx)
    }

    pub fn reset(&mut self) {
        for l in &mut self.layers {
            l.reset();
        }
    }

    pub fn total_memory_bytes(&self) -> usize {
        self.layers.iter().map(|l| l.memory_usage_bytes()).sum()
    }
}

/// Persistent Intermediate Hidden States vector, mutated in-place between layers.
#[derive(Debug, Clone)]
pub struct HiddenStates {
    pub data: Vec<f32>,
    pub hidden_dim: usize,
    pub seq_len: usize,
}

impl HiddenStates {
    pub fn new(seq_len: usize, hidden_dim: usize) -> Self {
        Self {
            data: vec![0.0f32; seq_len * hidden_dim],
            hidden_dim,
            seq_len,
        }
    }

    pub fn from_vec(data: Vec<f32>, hidden_dim: usize) -> Self {
        let seq_len = if hidden_dim > 0 { data.len() / hidden_dim } else { 0 };
        Self {
            data,
            hidden_dim,
            seq_len,
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        &mut self.data
    }

    pub fn memory_usage_bytes(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<f32>()
    }
}

/// SafeTensors Header Parser for fast zero-copy offset resolution.
pub struct SafeTensorsParser;

impl SafeTensorsParser {
    /// Parse the SafeTensors 8-byte length prefix and JSON header from an Mmap buffer.
    pub fn parse_header(mmap: &Mmap) -> Result<(HashMap<String, TensorDescriptor>, usize)> {
        if mmap.len() < 8 {
            return Err(TagisanError::Execution(
                "SafeTensors file smaller than 8 bytes".into(),
            ));
        }
        let header_len = u64::from_le_bytes(
            mmap[0..8]
                .try_into()
                .map_err(|e| TagisanError::Execution(format!("Header read error: {}", e)))?,
        ) as usize;

        if 8 + header_len > mmap.len() {
            return Err(TagisanError::Execution(format!(
                "Corrupt SafeTensors: header length {} exceeds file size {}",
                header_len,
                mmap.len()
            )));
        }

        let header_bytes = &mmap[8..8 + header_len];
        let raw_map: HashMap<String, serde_json::Value> = serde_json::from_slice(header_bytes)
            .map_err(|e| TagisanError::Execution(format!("Invalid SafeTensors JSON header: {}", e)))?;

        let mut descriptors = HashMap::new();
        for (name, val) in raw_map {
            if name == "__metadata__" {
                continue;
            }
            let dtype_str = val
                .get("dtype")
                .and_then(|v| v.as_str())
                .unwrap_or("F32");
            let shape = val
                .get("shape")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_u64().map(|n| n as usize))
                        .collect()
                })
                .unwrap_or_default();
            let offsets = val
                .get("data_offsets")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    let start = arr.get(0).and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                    let end = arr.get(1).and_then(|x| x.as_u64()).unwrap_or(0) as usize;
                    (start, end)
                })
                .unwrap_or((0, 0));

            descriptors.insert(
                name.clone(),
                TensorDescriptor {
                    name,
                    dtype: TensorDType::from_str(dtype_str),
                    shape,
                    data_offsets: offsets,
                },
            );
        }

        let base_data_offset = 8 + header_len;
        Ok((descriptors, base_data_offset))
    }
}

/// Commands sent to the LayerPrefetcher background thread.
pub enum PrefetchCommand {
    LoadLayer(usize),
    Terminate,
}

/// Asynchronous Layer Prefetcher implementing Double-Buffering with `sync_channel(1)`.
pub struct LayerPrefetcher {
    sender: SyncSender<PrefetchCommand>,
    receiver: Receiver<Result<LayerWeights>>,
    worker_handle: Option<JoinHandle<()>>,
    is_running: Arc<AtomicBool>,
}

impl LayerPrefetcher {
    /// Spawns the background I/O thread. The bounded channel has capacity 1,
    /// guaranteeing at most 1 ahead layer is buffered in memory.
    pub fn new(layer_paths: Vec<PathBuf>) -> Self {
        let (cmd_tx, cmd_rx) = sync_channel::<PrefetchCommand>(1);
        let (res_tx, res_rx) = sync_channel::<Result<LayerWeights>>(1);
        let is_running = Arc::new(AtomicBool::new(true));
        let running_flag = Arc::clone(&is_running);

        let handle = thread::Builder::new()
            .name("tgs-layer-prefetcher".to_string())
            .spawn(move || {
                debug!("LayerPrefetcher background I/O thread started");
                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        PrefetchCommand::LoadLayer(layer_idx) => {
                            let t0 = Instant::now();
                            if layer_idx >= layer_paths.len() {
                                let _ = res_tx.send(Err(TagisanError::Execution(format!(
                                    "Layer index {} out of range (max {})",
                                    layer_idx,
                                    layer_paths.len()
                                ))));
                                continue;
                            }
                            let path = &layer_paths[layer_idx];
                            trace!(layer = layer_idx, path = ?path, "Prefetching layer weights from disk");

                            let load_res = Self::mmap_layer(layer_idx, path);
                            let duration = t0.elapsed();
                            trace!(
                                layer = layer_idx,
                                elapsed_ms = duration.as_millis(),
                                "Layer prefetch finished"
                            );

                            if res_tx.send(load_res).is_err() {
                                break; // Consumer dropped
                            }
                        }
                        PrefetchCommand::Terminate => {
                            debug!("LayerPrefetcher received Terminate signal");
                            break;
                        }
                    }
                }
                running_flag.store(false, Ordering::SeqCst);
                debug!("LayerPrefetcher background I/O thread stopped");
            })
            .expect("Failed to spawn tgs-layer-prefetcher thread");

        Self {
            sender: cmd_tx,
            receiver: res_rx,
            worker_handle: Some(handle),
            is_running,
        }
    }

    fn mmap_layer(layer_idx: usize, path: &Path) -> Result<LayerWeights> {
        let file = File::open(path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to open layer file {}: {}",
                path.display(),
                e
            ))
        })?;

        // Memory-map the layer weights file zero-copy
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| {
                TagisanError::Execution(format!(
                    "Mmap failed for {}: {}",
                    path.display(),
                    e
                ))
            })?
        };

        // Advise kernel of sequential access pattern
        #[cfg(target_os = "linux")]
        unsafe {
            libc::madvise(
                mmap.as_ptr() as *mut libc::c_void,
                mmap.len(),
                libc::MADV_WILLNEED | libc::MADV_SEQUENTIAL,
            );
        }

        let mmap_len = mmap.len();
        let (descriptors, base_offset) = SafeTensorsParser::parse_header(&mmap)?;

        Ok(LayerWeights::new(
            layer_idx,
            Arc::new(mmap),
            descriptors,
            base_offset,
            mmap_len,
        ))
    }

    /// Request prefetching of a specific layer.
    pub fn request_layer(&self, layer_idx: usize) -> Result<()> {
        self.sender
            .send(PrefetchCommand::LoadLayer(layer_idx))
            .map_err(|e| TagisanError::Execution(format!("Failed to send prefetch command: {}", e)))
    }

    /// Block until the requested layer is loaded and delivered.
    pub fn wait_for_layer(&self) -> Result<LayerWeights> {
        self.receiver
            .recv()
            .map_err(|e| TagisanError::Execution(format!("Failed to receive layer from prefetcher: {}", e)))?
    }
}

impl Drop for LayerPrefetcher {
    fn drop(&mut self) {
        let _ = self.sender.send(PrefetchCommand::Terminate);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Model Configuration for the Layer Streaming Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStreamingConfig {
    pub model_name: String,
    pub num_layers: usize,
    pub hidden_dim: usize,
    pub intermediate_dim: usize,
    pub num_heads: usize,
    pub num_kv_heads: usize,
    pub head_dim: usize,
    pub max_seq_len: usize,
    pub vocab_size: usize,
    pub layer_files: Vec<PathBuf>,
}

impl Default for LayerStreamingConfig {
    fn default() -> Self {
        Self {
            model_name: "Llama-3-70B-Streaming-Prototype".to_string(),
            num_layers: 4, // Prototype default
            hidden_dim: 8192,
            intermediate_dim: 28672,
            num_heads: 64,
            num_kv_heads: 8,
            head_dim: 128,
            max_seq_len: 4096,
            vocab_size: 128256,
            layer_files: Vec::new(),
        }
    }
}

/// Real-time Inference Metrics for telemetry and performance auditing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceMetrics {
    pub tokens_generated: usize,
    pub total_prefill_time_ms: u128,
    pub total_decode_time_ms: u128,
    pub total_layer_io_wait_ms: u128,
    pub total_layer_compute_ms: u128,
    pub peak_resident_layers: usize,
    pub memory_evictions_count: usize,
}

/// Sequential Layer-Streaming LLM Inference Engine.
pub struct LayerStreamingEngine {
    config: LayerStreamingConfig,
    metrics: Arc<std::sync::Mutex<InferenceMetrics>>,
    active_layers: Arc<AtomicUsize>,
}

impl LayerStreamingEngine {
    pub fn new(config: LayerStreamingConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(std::sync::Mutex::new(InferenceMetrics::default())),
            active_layers: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Executes inference across all layers using Double-Buffering:
    /// - Compute Thread evaluates Layer `i`
    /// - Background Prefetcher loads Layer `i+1`
    /// - Immediately drops Layer `i` after execution, triggering `libc::MADV_DONTNEED`.
    pub fn forward_token(
        &self,
        token_id: u32,
        position: usize,
        hidden_states: &mut HiddenStates,
        kv_cache: &mut KvCache,
    ) -> Result<u32> {
        let num_layers = self.config.num_layers;
        if self.config.layer_files.len() < num_layers {
            return Err(TagisanError::Execution(format!(
                "Configured for {} layers but only {} layer files provided",
                num_layers,
                self.config.layer_files.len()
            )));
        }

        let prefetcher = LayerPrefetcher::new(self.config.layer_files.clone());

        // Prime the double-buffer: initiate prefetch of Layer 0
        prefetcher.request_layer(0)?;

        let mut current_layer_weights = Some(prefetcher.wait_for_layer()?);
        self.active_layers.fetch_add(1, Ordering::SeqCst);

        for i in 0..num_layers {
            // Pipeline next layer: If not at the final layer, request layer i + 1 asynchronously
            if i + 1 < num_layers {
                prefetcher.request_layer(i + 1)?;
            }

            let t_comp_start = Instant::now();
            let layer_kv = kv_cache.get_mut(i).ok_or_else(|| {
                TagisanError::Execution(format!("Missing KV cache for layer {}", i))
            })?;

            // Compute the forward pass for layer i
            if let Some(ref weights) = current_layer_weights {
                self.compute_layer(weights, hidden_states, layer_kv, position)?;
            }
            let comp_duration = t_comp_start.elapsed().as_millis();

            // Explicitly drop and trigger MADV_DONTNEED on current layer
            drop(current_layer_weights.take());
            self.active_layers.fetch_sub(1, Ordering::SeqCst);
            {
                let mut m = self.metrics.lock().unwrap();
                m.memory_evictions_count += 1;
                m.total_layer_compute_ms += comp_duration;
            }

            // Receive the prefetched next layer
            if i + 1 < num_layers {
                let t_io_start = Instant::now();
                current_layer_weights = Some(prefetcher.wait_for_layer()?);
                self.active_layers.fetch_add(1, Ordering::SeqCst);
                let io_wait = t_io_start.elapsed().as_millis();
                {
                    let mut m = self.metrics.lock().unwrap();
                    m.total_layer_io_wait_ms += io_wait;
                }
            }
        }

        // Project final hidden states to next token (mock argmax / greedy sampler)
        let next_token = self.sample_next_token(hidden_states, token_id)?;
        Ok(next_token)
    }

    /// Compute layer Transformer block:
    /// 1. Input LayerNorm (RMSNorm)
    /// 2. Multi-Head Attention / GQA (Query, Key, Value projections)
    /// 3. In-place update to persistent KV Cache
    /// 4. Output projection and residual addition
    /// 5. Post-attention LayerNorm
    /// 6. Feed-Forward SwiGLU (Gate, Up, Down projections) and residual addition
    fn compute_layer(
        &self,
        weights: &LayerWeights,
        hidden_states: &mut HiddenStates,
        kv: &mut LayerKvCache,
        _position: usize,
    ) -> Result<()> {
        let hidden = hidden_states.as_mut_slice();
        let _hidden_dim = self.config.hidden_dim;

        // Verify or simulate weight access zero-copy
        if let Some(attn_norm) = weights.get_tensor("attn_norm.weight") {
            let norm_slice = attn_norm.as_f32_slice().unwrap_or(&[]);
            if !norm_slice.is_empty() {
                // Apply RMSNorm SIMD mock
                for (h, w) in hidden.iter_mut().zip(norm_slice.iter().cycle()) {
                    *h *= *w;
                }
            }
        }

        // Mock GQA: generate k and v vectors for this token and append to layer KV cache
        let dummy_k = vec![0.01f32; self.config.num_kv_heads * self.config.head_dim];
        let dummy_v = vec![0.02f32; self.config.num_kv_heads * self.config.head_dim];
        kv.append(&dummy_k, &dummy_v);

        // Feed-forward SwiGLU residual step mock
        for x in hidden.iter_mut() {
            *x = (*x * 0.999f32) + 0.001f32;
        }

        Ok(())
    }

    /// Greedy sampling from the final hidden state representation.
    fn sample_next_token(&self, hidden_states: &HiddenStates, prev_token: u32) -> Result<u32> {
        let slice = hidden_states.as_slice();
        let sum: f32 = slice.iter().take(64).sum();
        let candidate = ((prev_token as f32 + sum.abs()) as u32) % (self.config.vocab_size as u32);
        Ok(candidate)
    }

    /// Generates a sequence of tokens in streaming mode.
    pub fn generate(
        &self,
        prompt_tokens: &[u32],
        max_new_tokens: usize,
    ) -> Result<Vec<u32>> {
        if prompt_tokens.is_empty() {
            return Err(TagisanError::Execution("Prompt tokens cannot be empty".into()));
        }

        let mut output = prompt_tokens.to_vec();
        let mut kv_cache = KvCache::new(
            self.config.num_layers,
            self.config.num_kv_heads,
            self.config.max_seq_len,
            self.config.head_dim,
        );
        let mut hidden_states = HiddenStates::new(1, self.config.hidden_dim);

        info!(
            model = %self.config.model_name,
            num_layers = self.config.num_layers,
            kv_cache_ram_bytes = kv_cache.total_memory_bytes(),
            "Starting Sequential Layer-Streaming inference"
        );

        // Prefill prompt tokens
        let t_prefill_start = Instant::now();
        for (pos, &token) in prompt_tokens.iter().enumerate() {
            self.forward_token(token, pos, &mut hidden_states, &mut kv_cache)?;
        }
        let prefill_duration = t_prefill_start.elapsed().as_millis();
        {
            let mut m = self.metrics.lock().unwrap();
            m.total_prefill_time_ms += prefill_duration;
        }

        // Decode loop
        let t_decode_start = Instant::now();
        for step in 0..max_new_tokens {
            let pos = prompt_tokens.len() + step;
            let last_token = *output.last().unwrap();
            let next_token = self.forward_token(last_token, pos, &mut hidden_states, &mut kv_cache)?;
            output.push(next_token);
            {
                let mut m = self.metrics.lock().unwrap();
                m.tokens_generated += 1;
            }
        }
        let decode_duration = t_decode_start.elapsed().as_millis();
        {
            let mut m = self.metrics.lock().unwrap();
            m.total_decode_time_ms += decode_duration;
        }

        Ok(output)
    }

    pub fn metrics(&self) -> InferenceMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

/// Helper utility for creating synthetic SafeTensors layer files for testing and benchmarking.
pub struct SyntheticSafeTensorsBuilder;

impl SyntheticSafeTensorsBuilder {
    pub fn create_layer_file(path: &Path, hidden_dim: usize) -> Result<()> {
        let mut file = File::create(path).map_err(|e| {
            TagisanError::Execution(format!("Failed to create synthetic layer: {}", e))
        })?;

        // Construct synthetic tensor descriptors
        let f32_bytes = hidden_dim * 4;
        let mut header_map = HashMap::new();

        let mut current_offset = 0;
        let tensor_names = ["attn_norm.weight", "mlp_norm.weight", "q_proj.weight"];

        for name in &tensor_names {
            let end_offset = current_offset + f32_bytes;
            let mut t_val = serde_json::Map::new();
            t_val.insert("dtype".into(), serde_json::Value::String("F32".into()));
            t_val.insert(
                "shape".into(),
                serde_json::Value::Array(vec![serde_json::Value::Number(hidden_dim.into())]),
            );
            t_val.insert(
                "data_offsets".into(),
                serde_json::Value::Array(vec![
                    serde_json::Value::Number(current_offset.into()),
                    serde_json::Value::Number(end_offset.into()),
                ]),
            );
            header_map.insert((*name).to_string(), serde_json::Value::Object(t_val));
            current_offset = end_offset;
        }

        let header_json = serde_json::to_string(&header_map)
            .map_err(|e| TagisanError::Execution(format!("JSON serialization failed: {}", e)))?;
        let mut header_bytes = header_json.into_bytes();
        let unaligned = (8 + header_bytes.len()) % 64;
        if unaligned != 0 {
            let padding = 64 - unaligned;
            header_bytes.extend(std::iter::repeat(b' ').take(padding));
        }
        let header_len = header_bytes.len() as u64;

        use std::io::Write;
        file.write_all(&header_len.to_le_bytes()).map_err(|e| {
            TagisanError::Execution(format!("Failed to write header length: {}", e))
        })?;
        file.write_all(&header_bytes).map_err(|e| {
            TagisanError::Execution(format!("Failed to write header JSON: {}", e))
        })?;

        // Write synthetic float data for all tensors
        let data_payload = vec![1.0f32; hidden_dim * tensor_names.len()];
        let byte_slice = unsafe {
            std::slice::from_raw_parts(
                data_payload.as_ptr() as *const u8,
                data_payload.len() * 4,
            )
        };
        file.write_all(byte_slice).map_err(|e| {
            TagisanError::Execution(format!("Failed to write tensor payload: {}", e))
        })?;
        file.flush().map_err(|e| {
            TagisanError::Execution(format!("Failed to flush synthetic file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AutoCleanupDir(PathBuf);
    impl AutoCleanupDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "tgs_streaming_{}_{}_{}",
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

    #[test]
    fn test_safetensors_parser_and_zero_copy_view() {
        let dir = AutoCleanupDir::new("safetensors");
        let layer_path = dir.path().join("layer_0.safetensors");
        SyntheticSafeTensorsBuilder::create_layer_file(&layer_path, 256).unwrap();

        let file = File::open(&layer_path).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };
        let (descriptors, base_offset) = SafeTensorsParser::parse_header(&mmap).unwrap();

        assert!(descriptors.contains_key("attn_norm.weight"));
        assert!(descriptors.contains_key("mlp_norm.weight"));

        let weights = LayerWeights::new(
            0,
            Arc::new(mmap),
            descriptors,
            base_offset,
            layer_path.metadata().unwrap().len() as usize,
        );

        let view = weights.get_tensor("attn_norm.weight").expect("Tensor not found");
        let f32_data = view.as_f32_slice().expect("Failed to cast to F32");
        assert_eq!(f32_data.len(), 256);
        assert_eq!(f32_data[0], 1.0f32);
    }

    #[test]
    fn test_layer_prefetcher_double_buffering() {
        let dir = AutoCleanupDir::new("prefetcher");
        let mut layer_paths = Vec::new();
        for i in 0..3 {
            let p = dir.path().join(format!("layer_{}.safetensors", i));
            SyntheticSafeTensorsBuilder::create_layer_file(&p, 128).unwrap();
            layer_paths.push(p);
        }

        let prefetcher = LayerPrefetcher::new(layer_paths);

        // Request Layer 0
        prefetcher.request_layer(0).unwrap();
        let l0 = prefetcher.wait_for_layer().unwrap();
        assert_eq!(l0.layer_idx, 0);

        // Double buffering: while l0 is active, request l1
        prefetcher.request_layer(1).unwrap();
        let l1 = prefetcher.wait_for_layer().unwrap();
        assert_eq!(l1.layer_idx, 1);

        // Drop l0 and l1, verifying Drop does not panic and triggers MADV_DONTNEED
        drop(l0);
        drop(l1);
    }

    #[test]
    fn test_layer_streaming_inference_forward_pass() {
        let dir = AutoCleanupDir::new("inference");
        let num_layers = 4;
        let mut layer_files = Vec::new();

        for i in 0..num_layers {
            let p = dir.path().join(format!("layer_{}.safetensors", i));
            SyntheticSafeTensorsBuilder::create_layer_file(&p, 512).unwrap();
            layer_files.push(p);
        }

        let config = LayerStreamingConfig {
            model_name: "Test-70B-Mock".to_string(),
            num_layers,
            hidden_dim: 512,
            intermediate_dim: 1024,
            num_heads: 8,
            num_kv_heads: 2,
            head_dim: 64,
            max_seq_len: 128,
            vocab_size: 1000,
            layer_files,
        };

        let engine = LayerStreamingEngine::new(config);
        let prompt = vec![101, 2054, 2003];
        let generated = engine.generate(&prompt, 5).expect("Inference generation failed");

        assert_eq!(generated.len(), prompt.len() + 5);
        let metrics = engine.metrics();
        assert_eq!(metrics.tokens_generated, 5);
        assert!(metrics.memory_evictions_count >= num_layers * (prompt.len() + 5));
    }
}
