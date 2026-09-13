---
name: rust-gguf-engine
description: Low-level zero-copy GGUF v2/v3 binary parsing, Ollama blob storage resolution, native Tokio HTTP streaming server, and in-process tensor inference in Tagisan.
triggers:
  - gguf
  - ollama
  - tensor engine
  - zero-copy mmap
  - tgs serve
  - tgs engine
tier: systems
category: ai-engine
---

# Rust GGUF Tensor Engine & Ollama Compatibility Skill

## Overview
Tagisan's native Tensor Engine reverse-engineers Ollama's container storage model and pairs it with zero-copy memory-mapped GGUF v2/v3 parsing and native Tokio async HTTP streaming.

## Architectural Components

### 1. GGUF Binary Parser (`src/engine/gguf.rs`)
- **Memory Mapping**: Maps models into process memory via `memmap2::Mmap` with sub-5ms latency.
- **Magic & Version**: Validates header magic `b"GGUF"` (`0x46554747`) and versions 2 & 3.
- **Metadata KV**: Dynamic typed parser (`Uint8`..`Float64`, `String`, `Array`).
- **Tensor Info**: Extracts tensor names, shapes, GGML quantization types (`Q4_K_M`, `Q8_0`, `F16`, etc.), offsets, and byte spans.

### 2. Ollama Blob Resolver (`src/engine/gguf.rs`)
- Auto-discovers local models in `~/.ollama/models/manifests/` or `$OLLAMA_MODELS`.
- Resolves manifest layers:
  - `application/vnd.ollama.image.model` -> GGUF model blob (`sha256-<hash>`)
  - `application/vnd.ollama.image.template` -> Jinja/Go chat template
  - `application/vnd.ollama.image.params` -> generation parameters & stop sequences
  - `application/vnd.ollama.image.system` -> model system prompt
- Intelligent query fuzzy matching (e.g. `"abliterated"`, `"llama3.2-abliterate:3b-instruct"`).

### 3. Embedded In-Process LLM Provider (`src/engine/embedded.rs`)
- Implements `LlmProvider` trait with ID `"embedded"` and pricing `$0.00`.
- Formats prompts according to model chat templates (Llama 3, ChatML).
- Streams tokens with 3-second heartbeat pulses to eliminate client timeout disconnections.

### 4. Native Tokio Ollama Server (`src/engine/server.rs`)
- Native HTTP/1.1 async server running over `tokio::net::TcpListener`.
- Supports `/`, `/api/version`, `/api/tags`, `/api/show`, `/api/chat`, and `/api/generate`.
- Handles Chunked Transfer-Encoding streaming NDJSON.
- Automatic port fallback if port 11434 is occupied.

## CLI Usage

```bash
# List discovered Ollama models
tgs engine list

# Inspect model binary tensors & metadata
tgs engine inspect abliterated

# Start native Ollama-compatible server
tgs serve --port 11434
```
