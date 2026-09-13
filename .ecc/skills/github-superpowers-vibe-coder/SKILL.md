---
name: "github-superpowers-vibe-coder"
description: "Master GitHub Open-Source Engineering Superpowers skill for vibe code developers: Formal Verification, Linux eBPF, AST Mutation, GPU Triton Kernels, Low-Latency HFT, Columnar Vector DBs, Kubernetes Operators, Binary Fuzzing, Compilers, and Swarm Intelligence across 500 canonical GitHub projects."
---
# GitHub Superpower Skills Engine for the Vibe Code Developer

## Overview & Escaping Fragile Vibe Coding
When generative AI assists developers in writing code, naive prompts produce **Catastrophic Fragility & AI Slop**:
- Brittle, unverified glue code with zero formal correctness guarantees.
- Slow, blocking REST and synchronous I/O loops that bottleneck on CPU context switches.
- Inability to mutate or refactor code structurally, resorting to broken regex string replacements.
- Naive model serving that runs unquantized tensors with zero KV-cache optimization.
- Shallow test suites with 0% mutation coverage that pass only because assertions are trivial.

**This skill equips the vibe coder and TGS swarms with 500 elite open-source superpowers across 10 engineering pillars.**

---

## The 5 Iron Laws of GitHub Engineering Superpowers

```
1. VERIFY BEFORE RUNNING: Always prefer formal model checking (Kani/Z3) or property fuzzing over anecdotal assertions.
2. BYPASS THE KERNEL WHEN LATENCY MATTERS: Use io_uring, DPDK, or XDP instead of standard synchronous POSIX sockets.
3. MUTATE SYNTAX TREES, NOT STRINGS: Execute structural refactors using Tree-sitter ASTs or Comby patterns.
4. TILE MEMORY FOR AI TENSORS: Never write naive loops over matrix multiplication; fuse GPU kernels via Triton and FlashAttention.
5. EPISODIC REFLEXION ON FAILURE: Always log stack traces and compiler errors to the Reflexion Vault to achieve self-healing code.
```

---

## The 10 Superpower Pillars (500-Project Canon)

- **Pillar I**: Formal Verification, SMT Solvers & Provable Mathematics (001 - 050)
- **Pillar II**: Linux Kernel, eBPF Telemetry, Low-Latency & Hardware Bypass (051 - 100)
- **Pillar III**: Autonomous Software Engineering, AST Mutation & Self-Healing (101 - 150)
- **Pillar IV**: High-Performance GPU Inference, Tensor Kernels & Quantization (151 - 200)
- **Pillar V**: High-Frequency Trading, Financial Engineering & Microstructure (201 - 250)
- **Pillar VI**: Distributed Databases, Vector Storage & Columnar Analytics (251 - 300)
- **Pillar VII**: Cloud-Native, Kubernetes Operators, Service Mesh & Chaos Engineering (301 - 350)
- **Pillar VIII**: Binary Exploitation, Security Auditing, Fuzzing & Forensics (351 - 400)
- **Pillar IX**: Compiler Toolchains, Language Runtimes & Polyglot Virtual Machines (401 - 450)
- **Pillar X**: Swarm Intelligence, Meta-Cognition & Autonomous Agentics (451 - 500)

*Refer to `docs/TOP_500_GITHUB_SKILLS_FOR_TGS_SUPERPOWERS.md` for the complete 500-item mapping.*

---

## Dynamic Skill Synthesis via `tgs harness`

Any of the 500 projects can be synthesized into an autonomous CLI tool or skill package:
```bash
tgs harness --github <OWNER/REPO> --skill <SKILL_IDENTIFIER>
```
