---
name: qual-demarco-peopleware
description: "The sociology of software development: Flow state, environmental factors, uninterrupted work time, the cost of context switching, and team jelling."
triggers: ["demarco-lister", "peopleware", "flow-state", "context-switching", "uninterrupted-time", "team-jelling", "developer-ergonomics"]
---

# qual-demarco-peopleware
> Based on **Peopleware: Productive Projects and Teams - Tom DeMarco & Timothy Lister**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Protect developer and user flow states: batch notifications asynchronously and provide dedicated 'Do Not Disturb' focus modes.**
2. **ALWAYS: Preserve full workspace state across app sessions to eliminate context reconstitution overhead.**
3. **NEVER: Trigger synchronous modal popups or marketing banners that steal cursor focus during active typing or coding.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Protect deep flow state: buffer asynchronous notifications, eliminate focus-stealing popups, and guarantee instantaneous workspace state resumption.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Stealing keyboard focus with an update modal while user is typing.**
- **Sending immediate notifications for every minor background sync.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-demarco-peopleware"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
