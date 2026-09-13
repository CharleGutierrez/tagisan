---
name: triton-cuda-tensor-kernel-fuser
description: OpenAI Triton and CUDA/PTX fused GPU tensor kernel engineering. Optimizes 2D/3D block tiling hierarchies, eliminates 32-way shared memory bank conflicts, maximizes tensor core warp shuffle utilization, and synthesizes high-throughput FlashAttention and fused GEMM operators.
version: 1.0.0
tags:
  - triton
  - cuda
  - gpu-kernel
  - tensor-cores
  - flash-attention
  - bank-conflicts
  - warp-shuffle
  - block-tiling
  - fused-gemm
triggers:
  - triton-cuda-tensor-kernel-fuser
  - triton-kernel
  - cuda-kernel-fuser
  - flash-attention-kernel
  - shared-memory-banks
  - warp-shuffle-gemm
compatibility: ">=0.2.0"
---

# OpenAI Triton & CUDA Fused GPU Tensor Kernel Engineering

## Purpose & Scope
Modern deep learning workloads (Transformer attention layers, quantization dequant-GEMMs, and RMSNorm/RoPE fusions) are severely memory-bandwidth bound. Standard framework kernels trigger repeated high-latency roundtrips to GPU High Bandwidth Memory (HBM/VRAM).

This skill provides an autonomous engineering framework for synthesizing, optimizing, and verifying high-performance GPU tensor kernels using OpenAI Triton and CUDA/PTX. It enforces 2D/3D block tiling hierarchies, eliminates 32-way shared memory bank conflicts via XOR swizzling, maximizes tensor core arithmetic intensity, and implements memory-efficient online softmax renormalization (FlashAttention).

---

## 1. Operational Invariants (GPU Tensor Kernel Protocol)

### Invariant 1: Coalesced Global Memory Load/Store & Vectorized Transfers
- **MANDATORY**: Global memory accesses must be coalesced across warp threads: contiguous threads must access contiguous 128-bit memory words (`float4`, `int4`, or 8x `fp16`).
- **ALWAYS**: Compute global memory pointer offsets using vectorized stride products (`tl.arange(0, BLOCK_SIZE)`).
- **MANDATORY**: Guard boundary tiles with explicit predicates/masks (`tl.load(ptr, mask=mask, other=0.0)`).
- **STRICT_REJECT**: Reject kernels that use strided non-coalesced memory load patterns that serialize global memory bus transactions.

### Invariant 2: 32-Way Shared Memory Bank Conflict Elimination & Swizzling
- **MANDATORY**: Shared memory allocations (SRAM) across 32 banks (4 bytes per bank per cycle) must prevent bank conflicts.
- **ALWAYS**: Apply XOR swizzling or stride padding (`stride = K + 1` or `(col ^ (row / 4)) % 32`) on shared memory tiles.
- **NEVER**: Allow unpadded power-of-two strides that cause 16-way or 32-way shared memory bank serialization stalls.
- **STRICT_REJECT**: Reject any shared memory indexing pattern that incurs 4-way, 8-way, 16-way, or 32-way bank conflict serialization stalls.

### Invariant 3: Tensor Core Warp Shuffle Communication & Register Tiling
- **MANDATORY**: Inner-loop matrix reductions and accumulate stages must reside entirely in GPU registers (RF), utilizing tensor core warp shuffle instructions (`__shfl_xor_sync`, `tl.dot`) rather than intermediate shared memory roundtrips.
- **ALWAYS**: Accumulate intermediate GEMM sums in FP32 precision (`tl.float32`) to prevent numerical underflow/overflow before truncating to FP16/BF16 on final store.
- **NEVER**: Spill intermediate accumulators from registers to local device memory (HBM).
- **STRICT_REJECT**: Reject implementations that spill registers to local memory (SRAM/HBM) due to unmanaged tile dimensions.

### Invariant 4: FlashAttention Tiling & Online Softmax Normalization
- **MANDATORY**: Attention computation must fuse Query-Key dot product, online softmax scaling, and Value reduction into a single kernel pass without materializing the $O(N^2)$ attention matrix in global memory.
- **ALWAYS**: Implement online running maximum and sum renormalization (Dao et al. FlashAttention algorithm) to maintain numerical stability and exact mathematical equivalence.
- **STRICT_REJECT**: Reject attention kernel designs that allocate global intermediate attention score buffers.

---

## 2. Canonical Triton Implementation Patterns

### Fused GEMM Kernel with Swizzled Block Tiling (Triton Python)
```python
import triton
import triton.language as tl

@triton.jit
def fused_gemm_kernel(
    a_ptr, b_ptr, c_ptr,
    M, N, K,
    stride_am, stride_ak,
    stride_bk, stride_bn,
    stride_cm, stride_cn,
    BLOCK_SIZE_M: tl.constexpr,
    BLOCK_SIZE_N: tl.constexpr,
    BLOCK_SIZE_K: tl.constexpr,
    GROUP_SIZE_M: tl.constexpr,
):
    pid = tl.program_id(axis=0)
    num_pid_m = tl.cdiv(M, BLOCK_SIZE_M)
    num_pid_n = tl.cdiv(N, BLOCK_SIZE_N)
    num_pid_in_group = GROUP_SIZE_M * num_pid_n
    group_id = pid // num_pid_in_group
    first_pid_m = group_id * GROUP_SIZE_M
    group_size_m = min(num_pid_m - first_pid_m, GROUP_SIZE_M)
    pid_m = first_pid_m + (pid % group_size_m)
    pid_n = (pid % num_pid_in_group) // group_size_m

    offs_am = (pid_m * BLOCK_SIZE_M + tl.arange(0, BLOCK_SIZE_M)) % M
    offs_bn = (pid_n * BLOCK_SIZE_N + tl.arange(0, BLOCK_SIZE_N)) % N
    offs_k = tl.arange(0, BLOCK_SIZE_K)

    a_ptrs = a_ptr + (offs_am[:, None] * stride_am + offs_k[None, :] * stride_ak)
    b_ptrs = b_ptr + (offs_k[:, None] * stride_bk + offs_bn[None, :] * stride_bn)

    accumulator = tl.zeros((BLOCK_SIZE_M, BLOCK_SIZE_N), dtype=tl.float32)
    for k in range(0, tl.cdiv(K, BLOCK_SIZE_K)):
        a = tl.load(a_ptrs, mask=offs_k[None, :] < K - k * BLOCK_SIZE_K, other=0.0)
        b = tl.load(b_ptrs, mask=offs_k[:, None] < K - k * BLOCK_SIZE_K, other=0.0)
        accumulator += tl.dot(a, b)
        a_ptrs += BLOCK_SIZE_K * stride_ak
        b_ptrs += BLOCK_SIZE_K * stride_bk

    c = accumulator.to(tl.float16)
    offs_cm = pid_m * BLOCK_SIZE_M + tl.arange(0, BLOCK_SIZE_M)
    offs_cn = pid_n * BLOCK_SIZE_N + tl.arange(0, BLOCK_SIZE_N)
    c_ptrs = c_ptr + stride_cm * offs_cm[:, None] + stride_cn * offs_cn[None, :]
    c_mask = (offs_cm[:, None] < M) & (offs_cn[None, :] < N)
    tl.store(c_ptrs, c, mask=c_mask)
```
