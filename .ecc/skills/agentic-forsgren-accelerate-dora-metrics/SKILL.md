---
name: agentic-forsgren-accelerate-dora-metrics
description: "Four DORA metrics (Deployment Frequency, Lead Time for Changes, Change Failure Rate, Time to Restore Service), and continuous delivery practices."
triggers: ["forsgren", "humble", "gene-kim", "accelerate", "dora-metrics", "continuous-delivery", "lead-time-for-changes"]
---

# agentic-forsgren-accelerate-dora-metrics
> Based on **Accelerate: Building and Scaling High Performing Technology Organizations - Nicole Forsgren, Jez Humble & Gene Kim**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Four DORA Metrics: Deployment Frequency, Lead Time for Changes, Change Failure Rate (< 15%), Mean Time to Recovery (< 1 hour).**
2. **Continuous Delivery Capabilities: Version control for all artifacts, trunk-based development, automated testing, loosely coupled architecture.**
3. **Transformational Leadership: Empowering engineering teams with autonomous decision-making and psychological safety.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize agent-assisted development to drive elite DORA metrics: sub-hour lead time from prompt to production, zero-regression trunk commits.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Long-lived feature branches that delay feedback and trigger painful merge conflicts.**
- **Deploying unverified changes that spike Change Failure Rates above healthy thresholds.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "forsgren"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
