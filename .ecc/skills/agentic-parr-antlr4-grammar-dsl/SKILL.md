---
name: agentic-parr-antlr4-grammar-dsl
description: "ALL(*) adaptive LL grammar parsing, listener vs visitor AST traversal patterns, lexical modes, and grammar ambiguity resolution."
triggers: ["parr", "antlr4", "adaptive-ll-star", "ast-visitor", "ast-listener", "lexical-modes", "grammar-engineering"]
---

# agentic-parr-antlr4-grammar-dsl
> Based on **The Definitive ANTLR 4 Reference - Terence Parr**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **ALL(*) Parsing Algorithm: Dynamically explores lookahead paths at runtime using deterministic finite automata (DFA), handling complex grammar recursion.**
2. **Visitor vs Listener Pattern: Listeners walk ASTs passively via event callbacks (enterRule/exitRule); Visitors explicitly control traversal order and return values.**
3. **Lexical Mode Switching: Switching token rules contextually (e.g. entering string interpolation or embedded SQL blocks).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Generate robust language parsers using ANTLR4 grammars. Implement the Visitor pattern when traversing code structures for type checking and transpilation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Introducing left-recursive grammar rules that cause infinite loops in non-adaptive parsers.**
- **Embedding arbitrary target language code actions directly into grammar files, destroying portability.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "parr"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
