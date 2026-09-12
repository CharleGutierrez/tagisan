---
name: qual-edmondson-fearless-organization
description: "Psychological safety: Creating environments where people feel safe to take interpersonal risks, voice concerns, admit errors, and challenge the status quo."
triggers: ["amy-edmondson", "fearless-organization", "psychological-safety", "blameless-culture", "interpersonal-risk", "error-reporting", "learning-organization"]
---

# qual-edmondson-fearless-organization
> Based on **The Fearless Organization: Creating Psychological Safety in the Workplace - Amy C. Edmondson**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Design collaboration, code review, and issue tracking systems with blameless language, positive feedback loops, and private drafting spaces.**
2. **ALWAYS: Celebrate early bug reporting and architectural vulnerability disclosures as valuable team contributions.**
3. **NEVER: Implement public shame leaderboards, punitive defect metrics, or surveillance telemetry that targets individual engineers.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct collaborative development workflows that foster psychological safety: normalize inquiries, support blameless reviews, and protect risk-taking.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Displaying 'Bug Creator of the Week' leaderboards.**
- **Locking down pull requests with aggressive, punitive automated commentary.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-edmondson-fearless-organization"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
