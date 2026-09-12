---
name: qual-dekker-field-guide-human-error
description: "The New View of human error: Moving from 'who did it' to 'what made their actions make sense at the time'. Resilience engineering and systemic safety."
triggers: ["dekker-field-guide", "new-view-safety", "resilience-engineering", "local-rationality", "blameless-post-mortem", "safety-differently"]
---

# qual-dekker-field-guide-human-error
> Based on **The Field Guide to Understanding 'Human Error' - Sidney Dekker**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Design system limits to degrade gracefully and provide soft boundaries rather than catastrophic cliff-like failures.**
2. **ALWAYS: Reconstruct operational breakdowns from the perspective of the operator's local rationality at that exact moment.**
3. **NEVER: Design punitive system features that lock out, fine, or publicly shame operators for operational mistakes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement resilient safety envelopes with soft boundaries and graceful degradation, understanding the local rationality of human operators under pressure.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Hard-failing an entire transaction pipeline because a single non-critical validation field was formatted unusually.**
- **Punitive error handling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-dekker-field-guide-human-error"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
