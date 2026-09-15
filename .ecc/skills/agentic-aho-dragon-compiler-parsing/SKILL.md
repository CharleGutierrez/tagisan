---
name: agentic-aho-dragon-compiler-parsing
description: "Lexical analysis, LL/LR parsing tables, abstract syntax trees (AST), syntax-directed translation, symbol tables, and compiler frontends."
triggers: ["aho", "dragon-book", "compiler-parsing", "abstract-syntax-tree", "syntax-directed-translation", "symbol-table", "lr-parsing"]
---

# agentic-aho-dragon-compiler-parsing
> Based on **Compilers: Principles, Techniques, and Tools (Dragon Book) - Alfred V. Aho, Monica S. Lam, Ravi Sethi & Jeffrey D. Ullman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Grammar Classification: Context-Free Grammar G = (V, Sigma, R, S) parsed via deterministic LR(1) or LALR tables without shift-reduce conflicts.**
2. **AST Construction: Generating an Abstract Syntax Tree that abstracts away concrete punctuation while preserving hierarchical semantic structure.**
3. **Symbol Table Scope Stack: Maintaining lexical scope hierarchies mapping identifier symbols to type signatures and memory offsets.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Parse agent-generated code into formal ASTs before saving to disk. Catch syntax and lexical errors immediately at the compiler frontend level.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on naive regex string matching to inspect or refactor nested programming language constructs.**
- **Ignoring lexical scope rules, causing duplicate symbol declarations or shadow variable bugs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "aho"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
