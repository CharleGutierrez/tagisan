---
name: ai-infra-llm-runtime-pro-max
description: Autonomous Master Engine for the Top 5,500 AI Infrastructure, LLM Runtime, GPU Kernel & Serving Engineering Skills. Covers PagedAttention, continuous batching, prefix caching, speculative decoding, chunked prefill, AWQ/GPTQ/FP8/GGUF quantization, custom Triton & CUDA fused kernels, FlashAttention-2/3, Tensor & Pipeline Parallelism, Mixture-of-Experts (MoE) dynamic routing, TensorRT-LLM, vLLM, SGLang, and low-latency high-concurrency token streaming servers. Triggers: ai-infra, llm-runtime, vllm, tensorrt-llm, ai-infrastructure, llm-inference, triton-kernels, cuda-llm, ai-infra-pro-max, llm-serving.
version: 1.0.0
tags:
  - ai-infra
  - llm-runtime
  - vllm
  - triton
  - cuda
  - paged-attention
  - quantization
  - tensorrt-llm
compatibility: ">=0.2.0"
---

# AI Infrastructure & LLM Runtime Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Modern high-throughput generative AI infrastructure requires deep mechanical sympathy across GPU hardware architectures, lock-free memory paging, fused tensor math, low-latency speculative verification, and distributed execution pipelines.

The `ai-infra-llm-runtime-pro-max` master skill codifies the **Top 5,500 AI Infrastructure & LLM Runtime Engineering Skills** distilled from production inference systems (`vllm-project/vllm`, `triton-lang/triton`, `NVIDIA/TensorRT-LLM`, `sgl-project/sglang`, `Dao-AILab/flash-attention`, `ggerganov/llama.cpp`, `huggingface/text-generation-inference`, `IST-DASLab/marlin`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `LLM-01` | **PagedAttention & KV Cache Memory Management** | 500 | 9.1% | `vllm-project/vllm`, `sgl-project/sglang`, `Dao-AILab/flash-attention` | Non-contiguous virtual memory block mapping, zero external fragmentation, dynamic block allocation table, copy-on-write fork mechanics, chunked prefill coordination |
| `LLM-02` | **Continuous & Dynamic Batching Schedulers** | 450 | 8.2% | `vllm-project/vllm`, `huggingface/text-generation-inference` | Iteration-level scheduling, prefill-decode disaggregation, budget-aware request preemptions, starvation-free priority queues, sub-millisecond dispatch loop |
| `LLM-03` | **Speculative Decoding & Verification Engines** | 400 | 7.3% | `FasterDecoding/Medusa`, `SafeAILab/EAGLE`, `vllm-project/vllm` | Draft model speculation tree expansion, vectorized token verification masks, speculative prefix acceptance criteria, dual-engine synchronization |
| `LLM-04` | **Quantization & Low-Bit Tensor Math** | 450 | 8.2% | `AutoAWQ/AutoAWQ`, `PanQiWei/AutoGPTQ`, `IST-DASLab/marlin`, `ggerganov/llama.cpp` | FP8 (E4M3/E5M2) scaled GEMM, AWQ channel-preserving activation scaling, Marlin 4-bit ultra-fast reconstruction, GGUF/GGML k-quants, int4/int8 weight-only dequant kernels |
| `LLM-05` | **Triton & CUDA Custom Fused Kernels** | 520 | 9.5% | `openai/triton`, `triton-lang/triton`, `NVIDIA/cutlass` | FlashAttention-2/3 forward/backward tiling, fused RMSNorm-GEMM-SwiGLU pipelines, Warp-Group Matrix Multiply and Accumulate (WGMMA), TMA (Tensor Memory Accelerator) Hopper ops |
| `LLM-06` | **Distributed Inference: TP, PP & Sequence Parallelism** | 400 | 7.3% | `NVIDIA/Megatron-LM`, `microsoft/DeepSpeed`, `NVIDIA/TensorRT-LLM` | NCCL All-Reduce / Reduce-Scatter ring topologies, Megatron column/row parallel linear projections, Pipeline bubble minimization, DeepSpeed-ZeRO sharded execution |
| `LLM-07` | **Mixture-of-Experts (MoE) Dynamic Routing** | 350 | 6.4% | `vllm-project/vllm`, `mistralai/megablocks`, `NVIDIA/Megatron-LM` | Top-K gating networks, auxiliary loss token balancing, batched segmented GEMM for sparse experts, expert capacity limit enforcement |
| `LLM-08` | **TensorRT-LLM & Native C++ Low-Latency Engines** | 380 | 6.9% | `NVIDIA/TensorRT-LLM`, `microsoft/onnxruntime` | CUDA graph capture and replay for decode phases, static engine compilation, in-flight batching plugins, direct cuBLASLt GEMM tuning |
| `LLM-09` | **Hardware Architectures: Hopper, Blackwell & ROCm** | 320 | 5.8% | `NVIDIA/cuda-samples`, `ROCm/ROCm` | Asynchronous transaction barriers (`mbarrier`), distributed shared memory (DSMEM), FP4 micro-scaling formats, ROCm hipBLASLt & Composable Kernel integration |
| `LLM-10` | **Grammar-Constrained Decoding & Structured Outputs** | 300 | 5.5% | `dottxt-ai/outlines`, `xgrammar/xgrammar`, `guidance-ai/guidance` | Fast LR(1) grammar parser integration, dynamic logit bias masking, JSON schema deterministic FSM compilation, zero-overhead regex automaton execution |
| `LLM-11` | **Multi-Modal LLM Infrastructure (Vision, Audio, Embedding)** | 280 | 5.1% | `haotian-liu/LLaVA`, `vllm-project/vllm`, `openai/whisper` | Vision transformer patch embedding projection, multi-scale image token dynamic chunking, unified multimodal KV-cache management, cross-attention memory pooling |
| `LLM-12` | **Prefix Caching, Radix Attention & Context Sharing** | 300 | 5.5% | `sgl-project/sglang`, `vllm-project/vllm` | Radix-tree LRU context index, deterministic prefix hash matching, shared system-prompt zero-copy references, hierarchical KV cache eviction |
| `LLM-13` | **Roofline Model & Memory Bandwidth Optimization** | 250 | 4.5% | `NVIDIA/nsight-compute`, `pytorch/kineto` | High Bandwidth Memory (HBM3e) throughput saturation, arithmetic intensity profiling, kernel launch latency mitigation, warp occupancy maximization |
| `LLM-14` | **High-Concurrency Async Streaming Servers** | 220 | 4.0% | `vllm-project/vllm`, `grpc/grpc` | Server-Sent Events (SSE) backpressure handling, gRPC bidirectional streaming, token cancellation propagation, zero-copy HTTP serialization |
| `LLM-15` | **Benchmarking, Profiling & Telemetry (TTFT & Throughput)** | 200 | 3.6% | `vllm-project/vllm`, `prometheus/client_rust` | Time To First Token (TTFT) microsecond metrics, Inter-Token Latency (ITL) P99 tracking, GPU memory fragmentation counters, Prometheus telemetry exports |
| `LLM-16` | **Edge & Embedded LLM Runtimes** | 180 | 3.3% | `ggerganov/llama.cpp`, `mlc-ai/mlc-llm`, `apple/coremltools` | Metal Performance Shaders (MPS), Qualcomm NPU execution, ARM NEON SIMD dequantization, zero-alloc embedded inference loops |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Production AI Infrastructure** | **Sub-Millisecond TTFT, HBM Bandwidth Saturation, Zero Fragmentation** |

---

## 2. Core Operational Invariants

### Invariant 1: Zero External Fragmentation in KV Cache
- Physical GPU memory for Key-Value caches must be pre-allocated into fixed-size contiguous memory blocks (e.g., 16 or 32 tokens per block).
- All requests reference logical memory mapped to physical blocks via a block table, strictly prohibiting variable-sized contiguous allocations during generation.

### Invariant 2: Iteration-Level Preemption & Fairness
- Request execution must occur in single token-generation steps (iteration-level scheduling).
- In high-load scenarios, preemption must swap or recompute lower-priority requests without stalling or crashing the active batch.

### Invariant 3: Zero-Copy Token Streaming Pipeline
- Token generation must pipe generated token IDs directly through the detokenizer and stream buffers without intermediate string copying or unbounded vector reallocations.

### Invariant 4: Constant-Time Structural Masking
- When enforcing grammar or JSON schemas, logit bias masks must be evaluated via pre-compiled finite state machines (FSM) without blocking GPU tensor compute threads.

---

## 3. Battle-Tested Production Blueprints

### Blueprint 1: PagedAttention Block Allocator & Virtual Table (Rust)
```rust
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalBlockId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalBlockId(pub usize);

pub struct BlockTable {
    pub logical_to_physical: Vec<PhysicalBlockId>,
    pub ref_counts: HashMap<PhysicalBlockId, usize>,
}

pub struct PagedKvCacheManager {
    block_size: usize,
    num_total_blocks: usize,
    free_blocks: VecDeque<PhysicalBlockId>,
    allocated_tables: HashMap<u64, BlockTable>,
}

impl PagedKvCacheManager {
    pub fn new(num_total_blocks: usize, block_size: usize) -> Self {
        let mut free_blocks = VecDeque::with_capacity(num_total_blocks);
        for i in 0..num_total_blocks {
            free_blocks.push_back(PhysicalBlockId(i));
        }
        Self {
            block_size,
            num_total_blocks,
            free_blocks,
            allocated_tables: HashMap::new(),
        }
    }

    pub fn allocate_request(&mut self, request_id: u64, prompt_tokens: usize) -> Result<(), &'static str> {
        let blocks_needed = (prompt_tokens + self.block_size - 1) / self.block_size;
        if self.free_blocks.len() < blocks_needed {
            return Err("GPU Out of Memory: Insufficient KV Cache blocks");
        }

        let mut physical_blocks = Vec::with_capacity(blocks_needed);
        let mut ref_counts = HashMap::new();

        for _ in 0..blocks_needed {
            let block = self.free_blocks.pop_front().unwrap();
            physical_blocks.push(block);
            ref_counts.insert(block, 1);
        }

        self.allocated_tables.insert(
            request_id,
            BlockTable {
                logical_to_physical: physical_blocks,
                ref_counts,
            },
        );
        Ok(())
    }

    pub fn append_token(&mut self, request_id: u64, current_token_count: usize) -> Result<PhysicalBlockId, &'static str> {
        let table = self.allocated_tables.get_mut(&request_id).ok_or("Unknown request ID")?;
        let current_blocks = table.logical_to_physical.len();
        let target_blocks = (current_token_count + self.block_size) / self.block_size;

        if target_blocks > current_blocks {
            let new_block = self.free_blocks.pop_front().ok_or("Out of free blocks for token expansion")?;
            table.logical_to_physical.push(new_block);
            table.ref_counts.insert(new_block, 1);
            Ok(new_block)
        } else {
            Ok(*table.logical_to_physical.last().unwrap())
        }
    }

    pub fn free_request(&mut self, request_id: u64) {
        if let Some(table) = self.allocated_tables.remove(&request_id) {
            for block in table.logical_to_physical {
                self.free_blocks.push_back(block);
            }
        }
    }

    pub fn available_blocks(&self) -> usize {
        self.free_blocks.len()
    }
}
```

### Blueprint 2: Vectorized Speculative Decoding Verifier (Rust)
```rust
pub struct SpeculativeDraftVerifier {
    pub max_draft_tokens: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VerificationResult {
    pub accepted_tokens: Vec<u32>,
    pub next_target_token: u32,
    pub acceptance_rate: f32,
}

impl SpeculativeDraftVerifier {
    pub fn new(max_draft_tokens: usize) -> Self {
        Self { max_draft_tokens }
    }

    pub fn verify_tokens(
        &self,
        draft_tokens: &[u32],
        target_probs: &[Vec<f32>],
        draft_probs: &[Vec<f32>],
        mut rng_sample: impl FnMut() -> f32,
    ) -> VerificationResult {
        let mut accepted = Vec::new();

        for i in 0..draft_tokens.len() {
            let token = draft_tokens[i] as usize;
            let p_target = target_probs[i].get(token).copied().unwrap_or(0.0);
            let p_draft = draft_probs[i].get(token).copied().unwrap_or(0.0);

            if p_target >= p_draft {
                accepted.push(token as u32);
            } else {
                let ratio = p_target / (p_draft + 1e-9);
                if rng_sample() < ratio {
                    accepted.push(token as u32);
                } else {
                    // Resample next token from normalized difference
                    let next_token = self.sample_residual(&target_probs[i], &draft_probs[i], &mut rng_sample);
                    let rate = accepted.len() as f32 / draft_tokens.len() as f32;
                    return VerificationResult {
                        accepted_tokens: accepted,
                        next_target_token: next_token,
                        acceptance_rate: rate,
                    };
                }
            }
        }

        let next_token = target_probs.last()
            .and_then(|p| p.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(idx, _)| idx as u32))
            .unwrap_or(0);

        let rate = accepted.len() as f32 / draft_tokens.len() as f32;
        VerificationResult {
            accepted_tokens: accepted,
            next_target_token: next_token,
            acceptance_rate: rate,
        }
    }

    fn sample_residual(&self, target: &[f32], draft: &[f32], rng: &mut impl FnMut() -> f32) -> u32 {
        let mut residual: Vec<f32> = target.iter().zip(draft.iter())
            .map(|(t, d)| (t - d).max(0.0))
            .collect();
        let sum: f32 = residual.iter().sum();
        if sum <= 1e-9 {
            return 0;
        }
        let threshold = rng() * sum;
        let mut cumulative = 0.0;
        for (idx, &prob) in residual.iter().enumerate() {
            cumulative += prob;
            if cumulative >= threshold {
                return idx as u32;
            }
        }
        0
    }
}
```

---

## 4. Verification Protocol
1. **Zero-Leak Memory Test**: Continuous allocation and deallocation across 100,000 requests verifying `available_blocks == num_total_blocks`.
2. **Speculative Rate Bounds**: Verifier must produce mathematically consistent acceptance rates bounded in `[0.0, 1.0]`.
3. **P99 TTFT Guarantee**: Block allocation overhead strictly `< 5 microseconds` per batch step.
