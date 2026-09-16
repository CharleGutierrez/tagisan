---
name: copilot-prompt-jailbreak-canary-tokens
description: "Canary token injection into system prompts to detect and abort prompt extraction attacks."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Defending Enterprise AI Against Prompt Injection and Jailbreaks - Simon Willison"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["jailbreak-canary-tokens", "prompt-extraction-defense", "canary-token-monitoring", "system-prompt-protection"]
---

# copilot-prompt-jailbreak-canary-tokens
> Based on **Defending Enterprise AI Against Prompt Injection and Jailbreaks - Simon Willison** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Canary Invariant: Inject unique high-entropy random UUID canary tokens into confidential system instructions.**
2. **Exfiltration Detection: Monitor model output stream; if canary token appears in output, abort transmission immediately.**
3. **MANDATORY alerting and blacklisting of offending user session.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-prompt-jailbreak-canary-tokens.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-prompt-jailbreak-canary-tokens.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Protect proprietary system prompts and confidential instructions using canary tokens and output exfiltration tripwires.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-prompt-jailbreak-canary-tokens.**
- **Unmonitored runtime execution without telemetry in copilot-prompt-jailbreak-canary-tokens.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jailbreak-canary-tokens"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
