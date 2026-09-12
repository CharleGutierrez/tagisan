---
name: qual-weick-sensemaking
description: "Organizational sensemaking: Retrospective, grounded in identity construction, social, continuous, extracted cues, driven by plausibility rather than accuracy."
triggers: ["karl-weick", "sensemaking", "high-reliability-organizations", "retrospective-sensemaking", "extracted-cues", "plausibility-over-accuracy", "shared-narrative"]
---

# qual-weick-sensemaking
> Based on **Sensemaking in Organizations - Karl E. Weick**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Provide unified chronological event timelines during production incidents so all cross-functional responders share identical situational sensemaking.**
2. **ALWAYS: Support real-time collaborative annotations directly attached to system state changes.**
3. **NEVER: Fragment incident telemetry across disjointed tools without a central, synchronized narrative timeline.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct collaborative observability timelines that synthesize disparate system cues into an actionable, plausible shared narrative for incident teams.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Isolating audit logs in separate databases with non-synchronized timestamps.**
- **Lacking collaborative shared notes during incident response.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-weick-sensemaking"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
