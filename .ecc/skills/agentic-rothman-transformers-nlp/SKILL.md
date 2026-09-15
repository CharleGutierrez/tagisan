---
name: agentic-rothman-transformers-nlp
description: "Transformer tokenization, attention visualization, encoder-decoder vs decoder-only routing, few-shot conditioning, and downstream agent specialization."
triggers: ["rothman", "transformers-nlp", "attention-visualization", "encoder-decoder", "decoder-only", "few-shot-conditioning"]
---

# agentic-rothman-transformers-nlp
> Based on **Transformers for Natural Language Processing and Computer Vision - Denis Rothman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Subword Tokenization (BPE/WordPiece): Text tokenization maps vocabulary IDs preserving morphological subword units.**
2. **Attention Map Analysis: Inspecting cross-attention weights to audit whether the agent attends to relevant codebase context.**
3. **Few-Shot Exemplar Anchoring: Injecting input-output exemplars with explicit reasoning traces to constrain output variance.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Provide 2-3 precise input/output exemplars within agent instructions to anchor tone, formatting, and structural invariants. Verify tokenization boundaries on specialized code syntax.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming word-level tokenization for code identifiers, causing token fragmentation in camelCase/snake_case.**
- **Providing conflicting few-shot examples that induce high epistemic entropy.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "rothman"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
