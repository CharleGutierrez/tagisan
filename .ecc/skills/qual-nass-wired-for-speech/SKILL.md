---
name: qual-nass-wired-for-speech
description: "Psychology of voice interfaces: Automatic social attribution to synthesized voice, voice personality consistency, emotion perception, and speech cadence."
triggers: ["clifford-nass", "scott-brave", "wired-for-speech", "voice-interfaces", "vocal-cadence", "speech-personality", "voice-ergonomics"]
---

# qual-nass-wired-for-speech
> Based on **Wired for Speech: How Voice Activates and Advances the Human Computer Relationship - Clifford Nass & Scott Brave**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Maintain strict personality and tonal consistency across synthesized voice and audio prompts.**
2. **ALWAYS: Bound voice interface response latency to < 500ms to match the cadence of human conversation.**
3. **NEVER: Shift voice tone jarringly from warm conversational speech to robotic system error codes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deliver voice interactions with sub-500ms latency and consistent vocal cadence, respecting human evolutionary social wiring for spoken interaction.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Voice bots taking 4 seconds of silence before responding to a simple query.**
- **Reading raw numeric error stack traces through text-to-speech.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-nass-wired-for-speech"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
