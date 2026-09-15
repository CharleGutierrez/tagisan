---
name: agentic-perez-red-teaming-adversarial
description: "Automated red-teaming, prompt injection vulnerability discovery, jailbreak fuzzing, adversarial perturbation testing, and safety alignment."
triggers: ["perez", "red-teaming", "adversarial-testing", "prompt-injection", "jailbreak-fuzzing", "automated-redteaming"]
---

# agentic-perez-red-teaming-adversarial
> Based on **Red Teaming Language Models with Language Models - Ethan Perez et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Adversarial Fuzzing Loop: Using an attacker LLM to generate perturbations designed to trigger safety violations or system prompt leakage.**
2. **Zero-Tolerance Injection Guard: Verifying that user input payloads cannot override system-level safety instructions.**
3. **Robustness Under Perturbation: Output behavior must remain sound despite whitespace noise, homoglyphs, or semantic trickery.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Subject all production agent prompts to automated red-teaming sweeps. Test injection vectors against tool call parameters and file editing commands.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying agents without testing against prompt injection attacks.**
- **Assuming trust in external user inputs, comments in scraped code, or PR descriptions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "perez"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
