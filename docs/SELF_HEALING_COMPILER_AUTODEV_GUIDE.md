# Self-Healing Compiler & TDD Healer Guide (`tgs autofix`)

## 1. Overview & Architecture

The **Self-Healing Compiler & TDD Healer (`tgs autofix`)** provides Tagisan (`tgs`) with autonomous polyglot compiler diagnostic parsing, surgical AST patch synthesis, and iterative closed-loop verification.

```
┌─────────────────────────────────────────────────────────────┐
│                    tgs autofix Loop                         │
└──────────────────────────────┬──────────────────────────────┘
                               │
                ┌──────────────▼──────────────┐
                │ 1. Project Type Detection   │
                │ (Rust / TS / Python / Go)   │
                └──────────────┬──────────────┘
                               │
                ┌──────────────▼──────────────┐
                │ 2. Polyglot Diagnostics     │
                │ (cargo / tsc / py_compile)  │
                └──────────────┬──────────────┘
                               │
             ┌─────────────────┴─────────────────┐
     [Clean / 0 Diags]                 [Diagnostics Found]
             │                                   │
    ┌────────▼────────┐                 ┌────────▼────────┐
    │ Return Report   │                 │ 3. Group by File│
    │ (is_clean: true)│                 │ & Create .bak   │
    └─────────────────┘                 └────────┬────────┘
                                                 │
                                        ┌────────▼────────┐
                                        │ 4. Descending   │
                                        │ Surgical Patches│
                                        └────────┬────────┘
                                                 │
                                        ┌────────▼────────┐
                                        │ 5. Write to Disk│
                                        │ (Loop back)     │
                                        └─────────────────┘
```

## 2. Core Components in `src/engine/autofix.rs`

### `ProjectType`
Detects codebase nature (`Rust`, `TypeScript`, `Python`, `Go`, or `Unknown`) based on manifests (`Cargo.toml`, `tsconfig.json`, `package.json`, `pyproject.toml`, `go.mod`) and file extensions.

### `CompilerDiagnostic`
Structured representation containing:
- `file: PathBuf`: Target file path.
- `line`, `col`, `end_line`, `end_col`: 1-based source coordinates.
- `code: Option<String>`: Compiler error code (e.g. `E0382`, `unused_variables`, `TS2322`, `SyntaxError`).
- `message: String`: Human-readable compiler diagnostic.
- `suggested_replacement: Option<String>`: Machine-applicable replacement string.
- `level: DiagnosticLevel`: `Error`, `Warning`, `Note`, `Help`.

### Diagnostic Parsers
- `parse_cargo_json`: Parses `cargo check --message-format=json`, extracting primary spans, machine-applicable compiler suggestions, and children help diagnostic spans.
- `parse_tsc_output`: Parses TypeScript `tsc` error and warning lines.
- `parse_python_diagnostics`: Parses Python `py_compile` tracebacks, syntax errors, and `pytest` assertion failures.
- `parse_go_diagnostics`: Parses `go vet` and `go build` compiler error lines.

### Surgical Patch Synthesis (`apply_span_replacement`)
Applies replacements cleanly:
- Multi-span diagnostics are applied in **descending order** of `(line, col)`.
- Modifying a span towards the bottom or right of a file preserves the exact 1-based coordinates of earlier spans in the same file.
- Guarantees zero panic slicing by clamping and aligning to UTF-8 character boundaries.
- Trims redundant whitespace when removing keywords (e.g., `mut ` removal for `unused_mut`).

## 3. CLI Command: `tgs autofix`

```bash
# Basic usage on current workspace
tgs autofix .

# Run with test diagnostics included (cargo check --tests, pytest)
tgs autofix . --test

# Perform dry-run simulation without modifying files
tgs autofix . --dry-run

# Limit healing cycles (default: 5)
tgs autofix . --max-attempts 3
```

## 4. ECC Agent & Skill Integration

### ECC Agent: `compiler-healer-expert`
Defined in `.ecc/agents/compiler-healer-expert.md`.
Enables the autonomous Tagisan Swarm to assign compiler diagnosis and AST self-healing tasks to a dedicated deep reasoning agent.

### ECC Skill: `compiler-autofix`
Defined in `.ecc/skills/compiler-autofix/SKILL.md`.
Provides standardized rules, invariants (zero panics, atomic `.bak` snapshots, descending order patch application), and execution patterns for autonomous compiler repair.
