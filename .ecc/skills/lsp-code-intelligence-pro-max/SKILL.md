---
name: lsp-code-intelligence-pro-max
description: Autonomous Master Engine for Language Server Protocol (LSP) Code Intelligence, Real-time Compiler Diagnostics, Definition & Cross-Reference Tracking, Workspace Symbol Indexing, and Hover Documentation. Enforces compiler-accurate AST analysis, cargo check/tsc/pyright integration, zero-hallucination code navigation, and resilient diagnostic self-healing across multi-language enterprise codebases. Triggers: lsp, code-intelligence, language-server, diagnostics, compiler-check, symbol-lookup, definition, references, hover-docs, ast-analysis, typecheck, lsp-pro-max.
version: 1.0.0
tags:
  - lsp
  - code-intelligence
  - diagnostics
  - ast
  - rustc
  - typescript
  - pyright
  - symbols
  - navigation
  - self-healing
compatibility: ">=0.2.0"
---

# LSP Code Intelligence Pro Max: Autonomous Compiler-Accurate Navigation & Diagnostic Engine

## Purpose & Scope
Autonomous software engineering agents must never speculate on type signatures, symbol definitions, or compilation diagnostics. Guessing at code types or assuming that a syntax edit passes without verifying compiler output causes cascading regressions, hallucinated APIs, and broken builds.

The `lsp-code-intelligence-pro-max` skill establishes Tagisan's authoritative language server and code intelligence architecture. It codifies the 6 architectural pillars governing compiler-accurate diagnostics, precise definition resolution, workspace-wide reference graph tracking, symbol indexing, rich hover documentation, and closed-loop diagnostic self-healing across Rust, TypeScript, JavaScript, Python, Go, and C/C++.

---

## Pillar LSP-01: Compiler-Accurate Multi-Language Diagnostics

### 1. Authoritative Compiler Subprocess Orchestration
Compiler diagnostics represent ground truth. Syntactic linters alone fail to catch type mismatches, lifetime errors, or missing trait implementations:
- **Rust Projects**: Execute `cargo check --message-format=json` or `cargo clippy --message-format=json` within a sandboxed timeout window. Parse newline-delimited JSON compiler streams.
  - Extract compiler diagnostic level (`error`, `warning`, `note`, `help`).
  - Extract machine-applicable compiler suggestion spans (`suggestion_applicability`, `replacement`).
  - Extract primary span coordinates: file path, start/end line, start/end column.
- **TypeScript / JavaScript**: Execute `npx tsc --noEmit` or `bun x tsc --noEmit`. Parse standard diagnostic format `file(line,col): error TS<code>: <message>`.
- **Python**: Orchestrate `pyright --outputjson` or `mypy --json` with fallback to `ruff check --output-format=json` and `python -m py_compile`.
- **Go**: Execute `go vet` and `go build -o /dev/null`.

### 2. Resilient Native AST & Heuristic Diagnostics Fallback
In offline, air-gapped, or resource-constrained environments where external language servers or compilers are unavailable:
- Tokenizer balances braces (`{}`, `()`, `[]`) across all files and flags unclosed structural scopes.
- String literal delimiter parser flags unescaped newlines or unmatched quotes (`"`, `'`, `` ` ``).
- Unresolved import detector compares imported modules against workspace crate/package roots.
- Zero-crash fallback ensures diagnostics always return structured JSON with actionable error counts.

---

## Pillar LSP-02: Deterministic Definition & Cross-File Jump Navigation

### 1. AST-Aware Symbol Lookup Hierarchy
When an agent or developer requests the definition of a symbol `fn_or_type`:
1. **Target File Scope**: Scan the active file for declarations matching the symbol name.
2. **Module Scope**: Resolve relative module references (`use crate::...`, `import ... from ...`, `from . import ...`).
3. **Workspace Scope**: Search across all workspace source files for declaration patterns:
   - Rust: `\b(pub\s+)?(fn|struct|enum|trait|type|const|static)\s+<symbol>\b`, `impl(\s+<.*>)?\s+<symbol>`
   - TypeScript / JS: `\b(export\s+)?(function|class|interface|type|const|let|var)\s+<symbol>\b`
   - Python: `\b(def|class)\s+<symbol>\b`
   - Go: `\bfunc\s+(?:\([^)]+\)\s+)?<symbol>\b|\btype\s+<symbol>\b`
4. **Ranking & Disambiguation**: Rank definition sites by exact name match, export visibility, and proximity to call sites.

### 2. Output Schema & Context Window
Definition lookups return:
- Canonical file path
- Zero-indexed or 1-indexed line and column numbers
- Declaration signature
- 5-line surrounding context snippet
- Containing scope (module, struct, class)

---

## Pillar LSP-03: Workspace-Wide Reference Tracking & Impact Analysis

### 1. Word-Boundary Reference Graph Mining
To calculate the blast radius of a proposed refactoring or API change:
- Scan all workspace files matching language extensions (`.rs`, `.ts`, `.tsx`, `.js`, `.py`, `.go`).
- Perform high-speed regex search with word boundary delimiters (`\b<symbol>\b`).
- Classify reference occurrences into:
  - Definition / Declaration site
  - Direct call / instantiation site
  - Type annotation / import site
  - Comment or string literal (filtered out or labeled as non-semantic)

### 2. Blast Radius Integration
Reference counts directly feed Tagisan's `CalculateBlastRadiusTool`:
- Low Blast: $< 3$ references in single module.
- Medium Blast: $3 - 15$ references across $1 - 3$ modules.
- Critical Blast: $> 15$ references or public API trait method change requiring automated regression test suite run.

---

## Pillar LSP-04: Deep Workspace & Document Symbol Extraction

### 1. Hierarchical Symbol Tree Introspection
Document symbol extraction converts raw text into structured outline trees:
- **Functions & Methods**: Parameters, return types, async/generator qualifiers.
- **Classes, Structs & Traits**: Fields, methods, associated constants, trait bounds.
- **Enums & Variants**: Discriminants, associated tuples/structs.
- **Constants & Type Aliases**: Base type mappings.

### 2. Fast Workspace Search Index
Workspace symbol search enables fuzzy querying across thousands of symbols in sub-millisecond response time:
- Symbol name, kind (`function`, `struct`, `interface`, etc.), location file/line.
- Container name (e.g. `McpServer::handle_tool_call`).

---

## Pillar LSP-05: Contextual Hover Documentation & Type Signatures

### 1. Rich Markdown Hover Synthesis
Hover operations combine signature definitions with associated docstrings:
- Rust: `///` and `//!` outer and inner doc comments.
- TypeScript / JS: `/** JSDoc */` comments including `@param`, `@returns`, `@throws`.
- Python: `"""Docstrings"""` including PEP 257 summary and Google/NumPy style sections.
- Markdown output provides clean, syntax-highlighted code blocks with descriptive text.

---

## Pillar LSP-06: Closed-Loop Diagnostic Self-Healing

### 1. Compile -> Diagnose -> Patch -> Verify Cycle
When an edit is applied:
1. Trigger `diagnostics` action immediately.
2. If `count > 0`:
   - Group diagnostics by severity (`error` prioritized over `warning`).
   - Extract primary error file, line, and compiler suggestion.
   - Formulate targeted replacement chunk.
   - Apply patch and re-run `diagnostics`.
3. Terminate loop only when `count == 0` (clean compilation) or circuit breaker threshold reached (max 3 iterations).
