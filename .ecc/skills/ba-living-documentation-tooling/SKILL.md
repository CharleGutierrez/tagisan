---
name: ba-living-documentation-tooling
description: "Living documentation architecture: AST analysis, domain-driven annotations, automated diagram extraction, and synchronizing code with domain knowledge."
triggers: ["living-documentation-tooling", "living-documentation", "cyrille-martraire", "code-as-documentation", "ast-analysis", "domain-annotations"]
---

# ba-living-documentation-tooling
> Based on **Living Documentation: Continuous Knowledge Sharing - Cyrille Martraire**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Single Source of Truth: Documentation must be derived directly from verified source code and executable tests, never maintained in disconnected wikis.**
2. **Executable Invariants: Domain rules documented in comments or markdown must be backed by unit tests or compiler type assertions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Annotate domain entities with living doc annotations. Extract architecture diagrams and markdown glossaries directly from codebase AST.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Maintaining stale Word documents or wiki pages that drift from codebase reality.**
- **Writing code comments that contradict executable behavior.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "living-documentation-tooling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
