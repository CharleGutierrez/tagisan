---
name: qual-vaughan-challenger-launch
description: "The Normalization of Deviance: How operational anomalies and out-of-spec signals become incrementally accepted as normal until catastrophic system failure occurs."
triggers: ["diane-vaughan", "challenger-launch", "normalization-of-deviance", "safety-culture", "incremental-risk", "anomaly-acceptance", "operational-drift"]
---

# qual-vaughan-challenger-launch
> Based on **The Challenger Launch Decision: Risky Technology, Culture, and Deviance at NASA - Diane Vaughan**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Treat intermittent system warnings, flaky tests, and recurring background errors as critical technical debt; maintain zero-tolerance alerts.**
2. **ALWAYS: Explicitly track operational drift by measuring deviations between specified behavior and actual runtime telemetry.**
3. **NEVER: Silence or permanently ignore recurring alerts simply because 'the system hasn't crashed yet'.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce strict anomaly accountability: prevent normalization of deviance by tracking alert creep, flaky test drift, and degraded thresholds as blocking defects.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Muting a failing alert channel because 'it fires all the time'.**
- **Retrying flaky tests 5 times until they accidentally pass in CI.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-vaughan-challenger-launch"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
