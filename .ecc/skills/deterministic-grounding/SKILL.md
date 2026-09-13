---
name: deterministic-grounding
description: Deterministic closed-loop grounding, AST invariant extraction, adversarial dialectical critique, and ephemeral compiler verification pipeline.
---

# ECC Deterministic Grounding & Verification Skill

## Protocol
1. **Context Grounding**: Synthesize petgraph AST symbol signatures and trait constraints before hypothesis formulation to bind the LLM to ground truth.
2. **Adversarial Dialectical Audit**: Scrutinize code hypotheses against static rules detecting `.unwrap()` panics, data races, unchecked indexing, and leaked file descriptors.
3. **Deterministic Sandbox Verification**: Compile candidate code in an ephemeral environment using `cargo check`, `py_compile`, `tsc`, or `go vet`.
4. **Closed-Loop Self-Healing**: Parse compiler diagnostic spans and iteratively apply surgical replacements until zero errors and zero critical findings remain.
5. **Certification**: Verify 100% compiler pass rate and emit calibrated confidence score.
