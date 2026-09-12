---
name: ba-nfr-quality-attributes
description: "Non-Functional Requirements & Quality Attribute Scenarios: 6-part scenarios (Source, Stimulus, Artifact, Environment, Response, Response Measure), SLA/SLO metrics."
triggers: ["nfr-quality-attributes", "quality-attribute-scenarios", "iso-25010", "non-functional-requirements", "bass-clements-kazman", "sla-slo"]
---

# ba-nfr-quality-attributes
> Based on **Software Architecture in Practice (4th Ed) & ISO 25010 - Bass, Clements, Kazman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **6-Part Quality Attribute Scenario: Every NFR must define Source of Stimulus, Stimulus, Artifact, Environment, Response, and quantifiable Response Measure.**
2. **SLO Quantifiability: Performance, Availability, Security, and Scalability must be formulated with objective numerical thresholds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify all NFRs as 6-part scenarios: Source of Stimulus, Stimulus, Artifact, Environment, Response, and quantifiable Response Measure.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing vague NFRs like 'System must be high-performance'.**
- **Ignoring degraded environment modes in NFR specifications.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "nfr-quality-attributes"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
