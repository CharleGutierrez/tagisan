---
name: agentic-alammar-transformer-mechanics
description: "Multi-head self-attention mechanics, KV-cache management, rotary positional embeddings (RoPE), token logits, and internal transformer layer projections."
triggers: ["alammar", "illustrated-transformer", "transformer-mechanics", "multi-head-attention", "kv-cache", "rope-embeddings", "token-logits"]
---

# agentic-alammar-transformer-mechanics
> Based on **Hands-On Large Language Models / The Illustrated Transformer - Jay Alammar & Maarten Grootendorst**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Scaled Dot-Product Attention: Attention(Q, K, V) = softmax(Q * K^T / sqrt(d_k)) * V.**
2. **KV-Cache Memory Footprint: Memory = 2 * n_layers * n_heads * d_head * seq_len * batch_size * precision_bytes.**
3. **Rotary Positional Embeddings (RoPE): Embeds positional information through complex rotation matrices preserving relative token distances.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize agent prompts for KV-cache reuse by placing static system prompts and tools at the absolute beginning of context. Manage token budget strictly relative to context window limits.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Dynamic prefix mutation that breaks KV-cache prefix sharing across turns, multiplying inference latency.**
- **Exceeding attention budget leading to needle-in-a-haystack retrieval degradation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "alammar"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
