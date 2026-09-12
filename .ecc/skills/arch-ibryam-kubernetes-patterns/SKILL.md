---
name: arch-ibryam-kubernetes-patterns
description: "Container orchestration patterns: Sidecar, Ambassador, Adapter, Init Container, Health Probes (Liveness, Readiness, Startup), and Controller reconciliation loops."
triggers: ["kubernetes-patterns", "sidecar-pattern", "ambassador-pattern", "init-container", "liveness-readiness-probes", "controller-loop", "declarative-reconcile"]
---

# arch-ibryam-kubernetes-patterns
> Based on **Kubernetes Patterns: Reusable Elements for Designing Cloud-Native Applications - Bilgin Ibryam & Roland Huß**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Separate core business containers from auxiliary concerns (logging, proxying, metric scraping) using the Sidecar pattern.**
2. **ALWAYS: Expose explicit `/livez` (liveness: is process deadlocked?) and `/readyz` (readiness: can process accept traffic?) HTTP probe endpoints.**
3. **NEVER: Mark readiness probe as healthy before database connection pools and warm caches are initialized.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Sidecar/Ambassador patterns for peripheral concerns. Define distinct liveness, readiness, and startup probes with calibrated failure thresholds.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Conflating liveness with readiness, causing Kubernetes to reboot containers that are simply warming up.**
- **Bundling monitoring proxies and business servers in a single monolithic container.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-ibryam-kubernetes-patterns"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
