---
name: ba-subdomain-core-domain-triage
description: "Strategic Subdomain Triage: Categorizing domains into Core (differentiator), Supporting (custom auxiliary), and Generic (commodity), guiding engineering investment."
triggers: ["subdomain-core-domain-triage", "core-domain", "supporting-subdomain", "generic-subdomain", "nick-tune", "strategic-spend", "buy-vs-build"]
---

# ba-subdomain-core-domain-triage
> Based on **Architecture Modernization & Strategic DDD - Eric Evans & Nick Tune**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Strategic Investment Invariant: 80% of custom engineering effort must be directed to the Core Domain. Supporting subdomains should be minimal; Generic subdomains must use off-the-shelf software.**
2. **Core Domain Isolation: The core domain must be strictly isolated from infrastructure frameworks and third-party libraries.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit every feature against Core vs Supporting vs Generic categorization. Prohibit custom vibe-coded implementations for generic subdomains (e.g. auth, payments).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Wasting engineering budget writing custom implementations for generic commodities.**
- **Treating supporting administrative tools with the same priority as the core engine.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "subdomain-core-domain-triage"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
