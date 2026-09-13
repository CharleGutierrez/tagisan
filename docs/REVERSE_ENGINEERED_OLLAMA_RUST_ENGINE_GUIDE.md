# Comprehensive Guide: Reverse-Engineered Ollama Rust Tensor Engine

## 1. Architectural Architecture & Design Principles

Tagisan (`tgs`) features an in-process, zero-copy Rust Tensor Engine designed to eliminate the latency, resource overhead, and external dependency requirements of running third-party LLM daemon processes. By reverse-engineering the Ollama container and blob storage architecture and combining it with low-level GGUF binary parsing over memory-mapped files (`mmap`), Tagisan achieves:

1. **Sub-5ms Model Loading**: By using `memmap2::Mmap`, virtual address space is mapped directly to the GGUF blob on NVMe/SSD storage without allocating multiple gigabytes on the heap or copying data.
2. **Native GGUF v2 & v3 Binary Specification**: Complete binary parser supporting variable-length headers, typed metadata key-value stores (with 13 scalar and array types), and GGML quantization types (`Q4_K_M`, `Q8_0`, `F16`, etc.).
3. **Ollama Container Blob Resolution**: Direct parsing of Ollama OCI image manifests and layer digests, automatically resolving GGUF model blobs, Jinja/Go chat templates, generation parameters, and system prompts from `~/.ollama/models/` or `$OLLAMA_MODELS`.
4. **Heartbeat-Pulsed Streaming**: Low-overhead HTTP 1.1 streaming using Chunked Transfer-Encoding and NDJSON, emitting 3-second heartbeat pulses to eliminate client socket timeouts during complex multi-step reasoning.
5. **Zero-Cost Inference**: Provider ID `"embedded"` runs with pricing `$0.00` per token.

---

## 2. Directory Layout & Module Structure

```
tagisan/src/engine/
├── mod.rs          # Engine context, budget tracker, and re-exports
├── budget.rs       # Token budget tracking and telemetry
├── gguf.rs         # Zero-copy GGUF v2/v3 parser & OllamaBlobResolver
├── embedded.rs     # EmbeddedLlmProvider implementing LlmProvider
└── server.rs       # Native Tokio HTTP 1.1 Ollama-compatible daemon
```

---

## 3. GGUF Binary Specification & Zero-Copy Parser

GGUF containers begin with a 24-byte binary header:

```
+---------------+---------------+--------------------+-----------------------+
|  Magic (4B)   | Version (4B)  | Tensor Count (8B)  | Metadata KV Count (8B)|
|   "GGUF"      |   uint32 (3)  |      uint64        |        uint64         |
+---------------+---------------+--------------------+-----------------------+
```

### Key-Value Metadata Types (`GgufValueType`)
- `0: Uint8`, `1: Int8`
- `2: Uint16`, `3: Int16`
- `4: Uint32`, `5: Int32`
- `6: Float32`, `7: Bool`
- `8: String` (length uint64 + UTF-8 payload)
- `9: Array` (type uint32 + length uint64 + items)
- `10: Uint64`, `11: Int64`, `12: Float64`

### Memory-Mapped Tensor Extraction
Tensor data starts at an offset aligned to `general.alignment` (default 32 bytes):
$$\text{Offset}_{\text{data}} = (\text{Offset}_{\text{header}} + (\text{alignment} - 1)) \ \& \ \neg(\text{alignment} - 1)$$

Individual tensor data can be accessed with zero copies:
```rust
let slice = gguf_file.tensor_slice("blk.0.attn_q.weight")?;
```

---

## 4. Ollama Blob Storage & Manifest Resolution

Ollama stores models in an OCI-inspired image format:
- Manifests: `~/.ollama/models/manifests/<registry>/<namespace>/<model>/<tag>`
- Blobs: `~/.ollama/models/blobs/sha256-<digest>`

The manifest JSON contains layer descriptors:
```json
{
  "layers": [
    { "mediaType": "application/vnd.ollama.image.model", "digest": "sha256:fdc5784e2c12..." },
    { "mediaType": "application/vnd.ollama.image.template", "digest": "sha256:966de95c..." },
    { "mediaType": "application/vnd.ollama.image.params", "digest": "sha256:56bb8bd4..." }
  ]
}
```

The `OllamaBlobResolver` inspects these layers, resolves symbolic model references (such as `"abliterated"` or `"llama3.2-abliterate:3b-instruct"`), and pairs the model with its exact chat template and stop tokens.

---

## 5. CLI Operations

### Listing Installed Models
```bash
tgs engine list
```
Output:
```
=========================================================================================================
 Tagisan Tensor Engine - Local Ollama Models (GGUF)
 Base Directory: /home/dyna/.ollama/models
=========================================================================================================
NAME                                FAMILY     SIZE         QUANT      MODIFIED       
---------------------------------------------------------------------------------------------------------
llama3.2-abliterate:3b-instruct     llama      2.09 GB      Q4_K_M     2026-09-13     
=========================================================================================================
```

### Inspecting Model Headers & Tensor Tensors
```bash
tgs engine inspect abliterated
```
Output:
```
=======================================================
 Tagisan GGUF Inspector: .../sha256-fdc5784e2c129015e9adc9a9f9af7ffa8b76ef6d032ecc8fe50a58fdb96ecb23
=======================================================
Format Version:      GGUF v3
Total Tensors:       256
Metadata KV Pairs:   35
Architecture:        llama
Context Length:      131072
Embedding Length:    3072
Block / Layer Count: 28
Attention Heads:     24
KV Heads:            8
Vocab Size:          128256
Tensor Data Offset:  0x12FE0 (77792 bytes)
-------------------------------------------------------
Sample Tensors (First 15):
  [000] token_embd.weight                        Q4_K     [128256 x 3072] (219.73 MB)
  [001] blk.0.attn_q.weight                      Q4_K     [3072 x 3072] (5.25 MB)
  ...
=======================================================
Zero-Copy mmap & header parse latency: 1.412 ms
```

### Launching the Native HTTP 1.1 Daemon
```bash
# Start server on default port 11434 (with auto-fallback if busy)
tgs serve

# Or specify a custom port
tgs serve --port 11439
```

---

## 6. HTTP API Compatibility

| Endpoint | Method | Description |
|---|---|---|
| `/` | `GET` | Probes engine health (`200 OK`) |
| `/api/version` | `GET` | Returns engine release (`{"version":"0.2.0-tgs-rust"}`) |
| `/api/tags` | `GET` | Lists discovered local models with full details |
| `/api/show` | `POST` | Inspects model template, parameters, and modelfile |
| `/api/chat` | `POST` | Streaming/non-streaming NDJSON chat completions |
| `/api/generate` | `POST` | Raw prompt completions |

### Example Streaming Chat Request:
```bash
curl -N http://127.0.0.1:11439/api/chat -d '{
  "model": "abliterated",
  "messages": [{"role": "user", "content": "Hello!"}],
  "stream": true
}'
```
