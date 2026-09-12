---
name: qual-holtzblatt-contextual-inquiry
description: "Contextual Design: Apprenticeship model, observing work in its real situated context, work models (flow, sequence, artifact, culture, physical), and affinity diagrams."
triggers: ["holtzblatt", "beyer", "contextual-inquiry", "contextual-design", "situated-work", "affinity-diagram", "work-modeling", "interruption-safe"]
---

# qual-holtzblatt-contextual-inquiry
> Based on **Contextual Inquiry and Design - Karen Holtzblatt & Hugh Beyer**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Design interfaces to be interruption-safe: auto-save drafts locally in client storage and seamlessly restore state across reloads and tab closures.**
2. **ALWAYS: Account for environmental noise, multi-monitor setups, and physical context in UI layout and contrast ratios.**
3. **NEVER: Erase unsubmitted user input upon network disconnection, session timeout, or accidental navigation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ensure zero data loss during multi-step workflows. Cache state continuously in local client storage to survive real-world workplace interruptions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Clearing an entire 15-field form when validation fails.**
- **Session timeout that permanently wipes half-completed data entry.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-holtzblatt-contextual-inquiry"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
