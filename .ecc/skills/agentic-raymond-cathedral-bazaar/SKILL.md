---
name: agentic-raymond-cathedral-bazaar
description: "Release early and often, Linus's Law (many eyeballs make bugs shallow), treating users as co-developers, decentralized design, and open-source dynamics."
triggers: ["raymond", "cathedral-bazaar", "release-early-often", "linus-law", "decentralized-development", "open-source-patterns"]
---

# agentic-raymond-cathedral-bazaar
> Based on **The Cathedral and the Bazaar - Eric S. Raymond**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Linus's Law: Given enough eyeballs, all bugs are shallow (deploying automated testing and multi-agent review sweeps).**
2. **Release Early, Release Often: Short release cadences minimize integration divergence and accelerate empirical feedback.**
3. **Smart Data Structures: Smart data structures and dumb code work a lot better than the other way around.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Commit small, frequent, atomic changes that keep the build green. Use automated multi-agent code reviews to uncover hidden edge cases.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Massive multi-week PRs that are impossible to review or debug.**
- **Hoarding uncommitted changes locally, risking devastating merge conflicts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "raymond"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
