---
name: agentic-tunstall-nlp-transformers
description: "Parameter-efficient fine-tuning (PEFT/LoRA), quantization (int8/int4), model deployment pipelines, instruction tuning, and local LLM execution."
triggers: ["tunstall", "von-werra", "wolf", "huggingface", "peft-lora", "quantization", "instruction-tuning", "local-llm"]
---

# agentic-tunstall-nlp-transformers
> Based on **Natural Language Processing with Transformers - Lewis Tunstall, Leandro von Werra & Thomas Wolf**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Low-Rank Adaptation (LoRA): W_updated = W_0 + (alpha / r) * B * A, where rank r << min(d_in, d_out).**
2. **Quantization Trade-offs: Quantizing weights to 4-bit (AWQ/GPTQ) reduces VRAM by ~70% with negligible perplexity penalty on code tasks.**
3. **Instruction Dataset Curation: Clean, deduplicated, verified test-passing code samples maximize transfer learning during alignment.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deploy quantized local models for offline agent coding loops. Use LoRA adapters tailored to specific internal frameworks and domain-specific APIs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying unquantized 32-bit float models for CLI agents, causing GPU out-of-memory crashes.**
- **Fine-tuning on unverified code containing syntax errors and security vulnerabilities.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "tunstall"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
