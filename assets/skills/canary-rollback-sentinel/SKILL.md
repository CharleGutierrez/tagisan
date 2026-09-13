---
name: canary-rollback-sentinel
description: Autonomous canary deployment & rollback. Stepwise canary traffic shifting; continuous p99/error telemetry gating; automated rollback with AST diff blame attribution.
version: 1.0.0
tags:
  - canary-deployment
  - automated-rollback
  - progressive-delivery
  - telemetry-gating
  - p99-monitoring
  - ast-diff-blame
  - blast-radius-containment
triggers:
  - canary-deployment
  - automated-rollback
  - canary-sentinel
  - traffic-shifting
  - p99-latency
  - error-budget
  - ast-diff-blame
  - deployment-telemetry
  - progressive-rollout
compatibility: ">=0.2.0"
---

# Canary Rollback Sentinel: Autonomous Progressive Delivery & AST Blame Attribution

## Purpose & Scope
Deploying software changes directly to 100% of production traffic risks catastrophic outages, degraded user experience, and revenue loss when latent bugs escape staging environments. Manual monitoring during release cycles is slow, error-prone, and reactive.

This skill equips autonomous agents with progressive canary deployment, real-time telemetry gating, instantaneous automated rollback, and AST diff blame attribution. By routing fractional production traffic to canary releases, evaluating SLA/SLO health gates continuously, rolling back within seconds of an invariant breach, and localizing the root cause to specific AST code modifications, the Sentinel guarantees continuous system reliability.

---

## 1. Operational Invariants (Canary & Rollback Protocol)

### Invariant 1: Stepwise Canary Traffic Shifting
- **MANDATORY**: Release progression must adhere to discrete, stepwise traffic intervals (e.g., 2% -> 10% -> 25% -> 50% -> 100%). Each increment requires a mandatory soak/bake evaluation window.
- **NEVER**: Promote a canary release from 0% directly to 100% traffic without successfully passing all intermediary telemetry gates.

### Invariant 2: Continuous p99 & Error Telemetry Gating
- **ALWAYS**: Continuously evaluate telemetry across 5 core reliability metrics during every canary phase:
  1. **HTTP 5xx Error Rate**: Must not exceed 0.5% or 2x the baseline rate.
  2. **Latency p95 & p99**: Must not degrade by more than 15% relative to baseline.
  3. **Crash / Panic Frequency**: Zero tolerance for panics, segfaults, or unhandled exceptions.
  4. **Resource Saturation**: CPU and memory consumption must remain within <= 120% of baseline profile.
  5. **Error Budget Burn Rate**: Release must immediately halt if 1-hour error budget burn exceeds 2x threshold.
- **STRICT_REJECT**: Any telemetry violation triggers an immediate, non-negotiable deployment abort.

### Invariant 3: Instantaneous Automated Rollback
- **MANDATORY**: When a telemetry gate fails, the Sentinel must execute an atomic rollback within < 5 seconds, shifting 100% of ingress traffic back to the stable baseline release.
- **NEVER**: Await human manual confirmation or terminal intervention to execute an emergency rollback when production health gates are breached.

### Invariant 4: AST Diff Blame Attribution & Case-Law Persistence
- **ALWAYS**: Immediately following an automated rollback, the Sentinel must analyze the git commit range between the baseline and canary builds, performing AST syntactic diffing to attribute failure blame to specific symbols, functions, or dependencies modified in the release.
- **MANDATORY**: Record the incident, failing metrics, and AST blame attribution into the system's episodic reflexion vault to prevent identical regressions in subsequent iterations.

---

## 2. Progressive Canary Gating Matrix

| Stage | Traffic % | Soak Duration | Invariant Criteria | Action on Breach |
|---|---|---|---|---|
| **Phase 1: Canary Probe** | 2% | 5 minutes | Error rate < 0.1%, Zero panics | Immediate Rollback to 0% |
| **Phase 2: Micro-Load** | 10% | 15 minutes | p99 latency <= 110% baseline | Immediate Rollback to 0% |
| **Phase 3: Half-Capacity**| 50% | 30 minutes | CPU/Memory within budget | Immediate Rollback to 0% |
| **Phase 4: Full Promotion**| 100% | Continuous | Steady-state SLA preserved | Record successful deployment |

---

## 3. Tool & Command Workflows

```bash
# Verify canary deployment status and telemetry gates
tgs telemetry status --release canary-v2.1

# Execute simulated canary evaluation and AST blame audit
cargo test --test nextgen_skills_enhancement_test
```
