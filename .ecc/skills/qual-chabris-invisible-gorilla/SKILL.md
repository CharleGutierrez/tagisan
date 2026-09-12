---
name: qual-chabris-invisible-gorilla
description: "Inattentional blindness, change blindness, and the illusion of attention: Human focus tunnels vision, causing critical peripheral alerts to be ignored."
triggers: ["chabris", "invisible-gorilla", "inattentional-blindness", "change-blindness", "focal-attention", "banner-blindness", "inline-validation"]
---

# qual-chabris-invisible-gorilla
> Based on **The Invisible Gorilla: How Our Intuitions Deceive Us - Christopher Chabris & Daniel Simons**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Anchor critical validation errors and status updates directly inside the user's active foveal gaze zone (inline inputs, cursor anchor).**
2. **ALWAYS: Use localized visual motion or contrast shifts only when an immediate user intervention is strictly required.**
3. **NEVER: Rely on peripheral toast messages, far-off notification bars, or bottom-corner badges for critical blocking errors.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Place feedback directly within the active task focus area. Prevent inattentional blindness by co-locating warnings with the causative input control.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Flashing error messages in the upper-right header while user types at the bottom of the page.**
- **Assuming users notice passive banner alerts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-chabris-invisible-gorilla"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
