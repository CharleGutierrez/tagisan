use crate::error::{Result, TagisanError};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use tracing::warn;

/// A reusable engineering skill defined according to the ECC specification
#[derive(Debug, Clone, PartialEq)]
pub struct EccSkill {
    pub name: String,
    pub description: String,
    pub instructions: String,
}

impl EccSkill {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            instructions: instructions.into(),
        }
    }

    /// Parse a SKILL.md file with YAML frontmatter
    pub fn parse(content: &str) -> Result<Self> {
        Self::parse_with_default_name(content, None)
    }

    /// Parse a SKILL.md file with YAML frontmatter and an optional fallback name
    pub fn parse_with_default_name(content: &str, default_name: Option<&str>) -> Result<Self> {
        let trimmed = content.trim_start_matches('\u{feff}').trim_start();
        if !trimmed.starts_with("---") {
            if let Some(def) = default_name {
                let first_line = trimmed.lines().find(|l| !l.trim().is_empty()).unwrap_or(def);
                let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                return Ok(Self {
                    name: def.to_string(),
                    description: if desc.is_empty() { def.to_string() } else { desc },
                    instructions: trimmed.to_string(),
                });
            }
            return Err(TagisanError::Execution(
                "Invalid ECC skill format: missing leading '---' frontmatter delimiter".to_string(),
            ));
        }

        let rest = &trimmed[3..];
        let end_idx = match rest.find("\n---").or_else(|| rest.find("\r\n---")) {
            Some(idx) => idx,
            None => {
                if let Some(def) = default_name {
                    let first_line = rest.lines().find(|l| !l.trim().is_empty()).unwrap_or(def);
                    let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                    return Ok(Self {
                        name: def.to_string(),
                        description: if desc.is_empty() { def.to_string() } else { desc },
                        instructions: rest.to_string(),
                    });
                }
                return Err(TagisanError::Execution(
                    "Invalid ECC skill format: missing closing '---' frontmatter delimiter".to_string(),
                ));
            }
        };

        let frontmatter_str = &rest[..end_idx];
        let after_close = &rest[end_idx..];
        let delim_pos = after_close.find("---").unwrap_or(0);
        let mut body_start_offset = delim_pos + 3;
        if let Some(nl_pos) = after_close[body_start_offset..].find('\n') {
            body_start_offset += nl_pos + 1;
        } else {
            body_start_offset = after_close.len();
        }
        let body = after_close.get(body_start_offset..).unwrap_or("").trim().to_string();

        let mut name = String::new();
        let mut description = String::new();

        let lines: Vec<&str> = frontmatter_str.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let raw_line = lines[i];
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim().trim_matches('"').trim_matches('\'').trim();

                match key.as_str() {
                    "name" => {
                        if name.is_empty() {
                            name = val.to_string();
                        }
                    }
                    "description" => {
                        if description.is_empty() {
                            if val == "|" || val == "|-" || val == ">" || val == ">-" || val.is_empty() {
                                let mut desc_lines = Vec::new();
                                i += 1;
                                while i < lines.len() {
                                    let next_raw = lines[i];
                                    if next_raw.starts_with(' ') || next_raw.starts_with('\t') {
                                        desc_lines.push(next_raw.trim());
                                        i += 1;
                                    } else if next_raw.trim().is_empty() {
                                        i += 1;
                                    } else {
                                        break;
                                    }
                                }
                                description = desc_lines.join(" ");
                                continue;
                            } else {
                                description = val.to_string();
                            }
                        }
                    }
                    _ => {}
                }
            }
            i += 1;
        }

        if name.trim().is_empty() {
            if let Some(def) = default_name {
                name = def.to_string();
            } else {
                return Err(TagisanError::Execution(
                    "Invalid ECC skill format: missing 'name' field in frontmatter".to_string(),
                ));
            }
        }

        if description.trim().is_empty() {
            let first_line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or(&name);
            description = first_line.trim().trim_start_matches('#').trim().to_string();
            if description.is_empty() {
                description = name.clone();
            }
        }

        let instructions = if body.trim().is_empty() {
            description.clone()
        } else {
            body
        };

        Ok(Self {
            name,
            description,
            instructions,
        })
    }

    /// Load an ECC skill from a file on disk
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read ECC skill file '{}': {}",
                path_ref.display(),
                e
            ))
        })?;
        let default_name = path_ref
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str());
        Self::parse_with_default_name(&content, default_name)
    }
}

/// Return all built-in ECC engineering skills
pub fn all_built_in_skills() -> Vec<EccSkill> {
    vec![
        tdd_workflow(),
        security_review(),
        api_design(),
        verification_loop(),
        tokio_async_tuning(),
        rust_idiomatic_hygiene(),
        rust_proptest_fuzzing(),
        criterion_benchmarking(),
        cargo_deny_security(),
        llm_as_a_judge_rubrics(),
        hallucination_detector(),
        swe_bench_decomposition(),
        prompt_injection_defense(),
        secret_entropy_scanner(),
        posix_shell_sanitizer(),
        bun_native_apis(),
        seccomp_sandbox_rules(),
        hexagonal_architecture(),
        backend_patterns(),
        coding_standards(),
        structured_analysis_yourdon(),
        modular_coupling_cohesion(),
        data_dictionary_minispecs(),
        domain_driven_design(),
        design_by_contract(),
        statechart_fsm_modeling(),
        data_intensive_architecture(),
        deep_modules_complexity(),
        legacy_seams_characterization(),
        catalog_refactoring_smells(),
        production_resilience_release_it(),
        evolutionary_fitness_functions(),
        temporal_invariants_tla(),
        conceptual_integrity_systems(),
        grokking_algorithms(),
        ostep_mechanical_sympathy(),
        system_design_building_blocks(),
        pragmatic_programmer_craft(),
        site_reliability_engineering(),
        // Advanced UX & UI Vibe Engineering Skills
        refactoring_ui(),
        microinteractions_design(),
        laws_of_ux(),
        design_systems_tokens(),
        about_face_interaction_design(),
        designing_for_emotion(),
        // Quantitative & Financial AI Foundation Model Skills
        kronos_kline_modeling(),
        qlib_alpha_engineering(),
        chronos_moirai_forecasting(),
        fingpt_multimodal_nlp(),
        rdagent_factor_mining(),
        // Mathematics for Vibe Code Developers (Top 20 Foundational Books)
        math_nature_of_code(),
        math_linear_algebra_savov(),
        math_high_dimensional_data(),
        math_information_inference(),
        math_causal_inference(),
        math_category_theory(),
        math_game_engine_geometry(),
        math_code_first_calculus(),
        math_bayesian_reasoning(),
        math_ml_foundations(),
        math_networks_crowds_markets(),
        math_convex_optimization(),
        math_probability_logic(),
        math_programmers_discrete(),
        math_intuitive_calculus(),
        math_concrete_discrete(),
        math_visual_topology(),
        math_chaos_fractals(),
        math_numerical_methods(),
        math_algorithmic_game_theory(),
    ]
}

/// Retrieve a built-in skill by name
pub fn find_built_in_skill(name: &str) -> Option<EccSkill> {
    let lower = name.to_lowercase().replace('_', "-");
    // Math Vibe Skills Aliases
    if lower == "nature-of-code" || lower == "boids" || lower == "flocking" || lower == "autonomous-steering" {
        return find_built_in_skill("math-nature-of-code");
    }
    if lower == "linear-algebra" || lower == "savov" || lower == "axler" || lower == "matrix-projections" {
        return find_built_in_skill("math-linear-algebra-savov");
    }
    if lower == "high-dimensional-data" || lower == "hopcroft" || lower == "curse-of-dimensionality" {
        return find_built_in_skill("math-high-dimensional-data");
    }
    if lower == "information-theory" || lower == "shannon-entropy" || lower == "mackay" || lower == "token-entropy" {
        return find_built_in_skill("math-information-inference");
    }
    if lower == "causal-inference" || lower == "book-of-why" || lower == "do-calculus" || lower == "pearl-causality" {
        return find_built_in_skill("math-causal-inference");
    }
    if lower == "category-theory" || lower == "milewski" || lower == "monads" || lower == "functors" {
        return find_built_in_skill("math-category-theory");
    }
    if lower == "game-engine-geometry" || lower == "quaternions" || lower == "slerp" || lower == "lengyel" {
        return find_built_in_skill("math-game-engine-geometry");
    }
    if lower == "code-first-calculus" || lower == "paul-orland" || lower == "gradient-descent-math" {
        return find_built_in_skill("math-code-first-calculus");
    }
    if lower == "bayesian-reasoning" || lower == "will-kurt" || lower == "thompson-sampling" || lower == "bayes" {
        return find_built_in_skill("math-bayesian-reasoning");
    }
    if lower == "ml-foundations" || lower == "deisenroth" || lower == "matrix-calculus" {
        return find_built_in_skill("math-ml-foundations");
    }
    if lower == "networks-crowds-markets" || lower == "pagerank" || lower == "kleinberg" || lower == "easley" {
        return find_built_in_skill("math-networks-crowds-markets");
    }
    if lower == "convex-optimization" || lower == "boyd" || lower == "kkt" || lower == "kkt-conditions" {
        return find_built_in_skill("math-convex-optimization");
    }
    if lower == "probability-logic" || lower == "jaynes" || lower == "maximum-entropy" || lower == "maxent" {
        return find_built_in_skill("math-probability-logic");
    }
    if lower == "programmers-discrete" || lower == "jeremy-kun" || lower == "fft" || lower == "graph-laplacian" {
        return find_built_in_skill("math-programmers-discrete");
    }
    if lower == "intuitive-calculus" || lower == "calculus-made-easy" || lower == "pid-controller" {
        return find_built_in_skill("math-intuitive-calculus");
    }
    if lower == "concrete-discrete" || lower == "concrete-math" || lower == "knuth-math" || lower == "recurrences" {
        return find_built_in_skill("math-concrete-discrete");
    }
    if lower == "visual-topology" || lower == "hilbert-geometry" || lower == "gaussian-curvature" {
        return find_built_in_skill("math-visual-topology");
    }
    if lower == "chaos-fractals" || lower == "lorenz-attractor" || lower == "fractals" || lower == "mandelbrot" {
        return find_built_in_skill("math-chaos-fractals");
    }
    if lower == "numerical-methods" || lower == "numerical-recipes" || lower == "rk4" || lower == "runge-kutta" {
        return find_built_in_skill("math-numerical-methods");
    }
    if lower == "algorithmic-game-theory" || lower == "nash-equilibrium" || lower == "shapley-value" || lower == "vcg-auction" {
        return find_built_in_skill("math-algorithmic-game-theory");
    }
    if lower == "kronos" || lower == "kronos-bsq" || lower == "kline-modeling" || lower == "k-line-modeling" {
        return find_built_in_skill("kronos-kline-modeling");
    }
    if lower == "qlib" || lower == "qlib-alpha" || lower == "qlib-factor-neutralization" {
        return find_built_in_skill("qlib-alpha-engineering");
    }
    if lower == "chronos" || lower == "moirai" || lower == "chronos-moirai" || lower == "moirai-forecasting" {
        return find_built_in_skill("chronos-moirai-forecasting");
    }
    if lower == "fingpt" || lower == "finrl" || lower == "fingpt-finrl" || lower == "financial-nlp" {
        return find_built_in_skill("fingpt-multimodal-nlp");
    }
    if lower == "rdagent" || lower == "rd-agent" || lower == "rdagent-mining" || lower == "rdagent-alpha" {
        return find_built_in_skill("rdagent-factor-mining");
    }
    if lower == "rust-tokio-concurrency" || lower == "rust-concurrency" || lower == "tokio-concurrency" {
        if let Some(mut s) = find_built_in_skill("tokio-async-tuning") {
            s.name = "rust-tokio-concurrency".to_string();
            return Some(s);
        }
    }
    all_built_in_skills().into_iter().find(|s| s.name == lower)
}

/// 1. TDD Workflow Skill
pub fn tdd_workflow() -> EccSkill {
    EccSkill::new(
        "tdd-workflow",
        "Test-Driven Development discipline: write failing assertions and reproduction cases before code. Triggers: test driven development, unit test, assertions, regression, tdd workflow, failing tests.",
        r#"# ECC TDD Workflow

## Phase 1: Test Formulation
- Always formulate concrete, failing unit tests before authoring implementation logic.
- Identify the public interface, inputs, expected outputs, and error variants.
- Write tests that capture boundary limits (0, 1, MAX, empty, null/None, unicode).

## Phase 2: Minimal Implementation
- Write the minimum amount of code required to make the tests pass.
- Resist premature optimization during greening phase.

## Phase 3: Refactor
- Eliminate duplicate code (DRY).
- Improve naming and readability without altering observable external behavior.
- Ensure all tests continue to pass.
"#,
    )
}

/// 2. Security Review Skill
pub fn security_review() -> EccSkill {
    EccSkill::new(
        "security-review",
        "Rigorous offensive and defensive threat modeling and vulnerability scanning. Triggers: security threat model, injection, memory safety, vulnerability audit, security review.",
        r#"# ECC Security Review

## Threat Checklist
1. Input Sanitization: Validate and sanitize all external inputs (CLI flags, file inputs, network bytes).
2. Injection Prevention: Avoid shell concatenation or string interpolation in command execution.
3. Concurrency Safety: Check for race conditions, deadlock cycles, and time-of-check to time-of-use (TOCTOU).
4. Secret Protection: Ensure API keys, tokens, and credentials are never logged or echoed to stdout.
5. Memory & Resource Safety: Validate array bounds, recursion depths, and allocation limits to prevent denial-of-service.
"#,
    )
}

/// 3. API Design Skill
pub fn api_design() -> EccSkill {
    EccSkill::new(
        "api-design",
        "Robust API design patterns emphasizing backward compatibility and intuitive ergonomic interfaces",
        r#"# ECC API Design

## Principles
1. Explicit over Implicit: Design function signatures where failure modes are represented in types (`Result`, `Option`).
2. Least Astonishment: Follow canonical idioms of the host programming language.
3. Extensibility: Use the builder pattern or options structs for functions with many parameters.
4. Documentation: Document every public struct, enum, and function with doc comments and usage examples.
"#,
    )
}

/// 4. Verification Loop Skill
pub fn verification_loop() -> EccSkill {
    EccSkill::new(
        "verification-loop",
        "Continuous verification, automated test runs, and regression monitoring",
        r#"# ECC Verification Loop

## Protocol
1. Baseline: Run the existing test suite to ensure an unpolluted baseline.
2. Reproduction: If fixing a bug, write a test that fails reliably without the fix.
3. Application: Apply the targeted, minimal fix.
4. Confirmation: Rerun tests to confirm the reproduction test passes AND zero regressions occur in existing tests.
"#,
    )
}

/// 5. Tokio Async Tuning Skill
pub fn tokio_async_tuning() -> EccSkill {
    EccSkill::new(
        "tokio-async-tuning",
        "Tokio async concurrency, lock contention avoidance, bounded channels, cancellation tokens, and JoinSet task scheduling. Triggers: tokio, concurrency, async, rust-tokio-concurrency, joinset, mpsc, mutex, await, blocking, cancellation.",
        r#"# Tokio Async & Concurrency Optimization

## Core Concurrency Protocols
1. Channel Bounding: Always use bounded channels (`tokio::sync::mpsc::channel(N)`); never unbounded in production hot paths.
2. Lock Contention Minimization: Never hold a `MutexGuard` across an `.await` boundary. Use message passing or atomic primitives (`AtomicBool`, `AtomicUsize`) for scalars.
3. Task Orchestration: Prefer `tokio::task::JoinSet` over loose `tokio::spawn` calls to ensure structured concurrency, cancellation propagation, and clean resource cleanup.
4. Blocking Operations: Offload heavy synchronous computations, CPU-bound parsing, or blocking filesystem I/O to `tokio::task::spawn_blocking`.
5. Clean Cancellation: Use tokio_util::sync::CancellationToken for cooperative shutdown and timeout propagation.
"#,
    )
}

/// 6. Rust Idiomatic Hygiene Skill
pub fn rust_idiomatic_hygiene() -> EccSkill {
    EccSkill::new(
        "rust-idiomatic-hygiene",
        "Idiomatic Rust patterns, RAII memory safety, thiserror error trees, and zero-copy Cow abstractions",
        r#"# Rust Idiomatic Hygiene & Memory Architecture

## Core Guidelines
1. Error Architecture: Represent domain failure modes with explicit enum variants derived via `thiserror::Error`. Never discard errors with `.unwrap()` in library code.
2. Zero-Copy Ergonomics: Use `std::borrow::Cow<'a, str>` or `&str` when data transformation is optional.
3. RAII & Clean Teardown: Leverage the `Drop` trait to guarantee deterministic cleanup of file handles, temporary directories, worktrees, and sockets.
4. Type-Driven Design: Make illegal states unrepresentable using Rust's algebraic data types (`enum`, `struct`, newtype pattern).
"#,
    )
}

/// 7. Rust Property-Based Testing & Fuzzing Skill
pub fn rust_proptest_fuzzing() -> EccSkill {
    EccSkill::new(
        "rust-proptest-fuzzing",
        "Property-based invariant testing, fuzzing with proptest/quickcheck, and minimal failure shrink analysis",
        r#"# Rust Property-Based Testing & Invariant Fuzzing

## Fuzzing Protocols
1. Invariant Identification: Define mathematical or structural invariants (e.g., `decode(encode(x)) == x`, `sort(x).is_sorted()`, `a + b == b + a`).
2. Generator Strategies: Author custom `proptest::strategy::Strategy` implementations that span entire domains including edge cases (empty collections, MAX/MIN bounds, NaN, UTF-8 surrogate code points).
3. Minimal Failing Vector Analysis: Utilize proptest's automated shrinking algorithm to isolate the exact minimal reproduction input on failure.
"#,
    )
}

/// 8. Criterion Benchmarking Skill
pub fn criterion_benchmarking() -> EccSkill {
    EccSkill::new(
        "criterion-benchmarking",
        "Statistical micro-benchmarking with Criterion, throughput measurement, and allocation profiling",
        r#"# Criterion Statistical Micro-Benchmarking

## Benchmarking Protocol
1. Statistical Isolation: Use `criterion::black_box` to prevent LLVM dead-code elimination and constant folding.
2. Throughput Metrics: Configure benchmarks with `Throughput::Bytes` or `Throughput::Elements` for realistic MB/s evaluations.
3. Warmup & Outlier Detection: Enforce minimum 3-second warmup and 5-second measurement periods across at least 100 samples.
4. Flamegraph Integration: Pair benchmarks with `pprof` or `cargo-flamegraph` to visualize CPU bottlenecks in hot loops.
"#,
    )
}

/// 9. Cargo Deny & Supply Chain Security Skill
pub fn cargo_deny_security() -> EccSkill {
    EccSkill::new(
        "cargo-deny-security",
        "Supply chain crate security, license compliance, duplicate dependencies, and cargo-audit scanning",
        r#"# Cargo Supply Chain Security & Dependency Hygiene

## Audit Guidelines
1. Vulnerability Scanning: Audit all transitive dependencies with `cargo audit` and RustSec Advisory Database.
2. License Enforcement: Reject non-permissive or ambiguous licenses (enforce MIT, Apache-2.0, BSD-3-Clause).
3. Duplicate Elimination: Detect and consolidate duplicate versions of common crates (`serde`, `tokio`, `syn`).
4. Ban Lists: Block unmaintained or unsound crates with known CVEs or memory safety violations.
"#,
    )
}

/// 10. LLM as a Judge Scoring Rubrics Skill
pub fn llm_as_a_judge_rubrics() -> EccSkill {
    EccSkill::new(
        "llm-as-a-judge-rubrics",
        "Structured evaluation scoring rubrics, Borda count ranking, and adversarial debate adjudication",
        r#"# LLM-as-a-Judge Evaluation & Adjudication Rubrics

## Evaluation Rubrics
1. Correctness (Weight 40%): Mathematical soundness, constraint satisfaction, compile-time validity, edge-case coverage.
2. Security & Guardrails (Weight 30%): Resistance to injection, privilege escalation, credential exposure, buffer exhaustion.
3. Maintainability (Weight 15%): Modularity, idiomatic naming, documentation density, clear separation of concerns.
4. Performance (Weight 15%): Algorithmic time/space complexity, memory footprint, cache-friendliness.
5. Consensus Synthesis: Aggregate individual agent assessments using Positional Borda Count or Supermajority voting.
"#,
    )
}

/// 11. Hallucination Detector Skill
pub fn hallucination_detector() -> EccSkill {
    EccSkill::new(
        "hallucination-detector",
        "Context grounding validation, fact checking against retrieved vector memory, and claim verification",
        r#"# Hallucination Detection & Context Grounding

## Verification Procedure
1. Claim Extraction: Decompose LLM output into atomic, verifiable technical assertions.
2. Grounding Cross-Check: Verify each claim against source code chunks and episodic memory retrieved via RAG.
3. Factuality Scoring: Flag and reject claims that cite non-existent functions, imaginary API endpoints, or phantom types.
4. Contradiction Analysis: Detect if proposed logic violates previously established architectural invariants.
"#,
    )
}

/// 12. SWE-Bench Issue Decomposition Skill
pub fn swe_bench_decomposition() -> EccSkill {
    EccSkill::new(
        "swe-bench-decomposition",
        "Decomposition of complex real-world software engineering issues into test-first DAG execution plans",
        r#"# SWE-Bench Problem Decomposition & Resolution

## Structured Workflow
1. Issue Ingestion: Parse bug reports, stack traces, and reproduction scripts.
2. Localization: Use vector search and codebase indexing to identify the minimal set of modified files.
3. Failing Regression Test: Author a dedicated unit/integration test reproducing the exact failure.
4. Surgical Remediation: Apply targeted modifications with zero side-effects to unrelated subsystems.
5. Verification Pass: Run the full test suite and confirm zero regressions.
"#,
    )
}

/// 13. Prompt Injection Defense Skill
pub fn prompt_injection_defense() -> EccSkill {
    EccSkill::new(
        "prompt-injection-defense",
        "Indirect prompt injection defense, token smuggling mitigations, and untrusted payload containment",
        r#"# Prompt Injection Defense & Input Armor

## Defense Guidelines
1. Boundary Enclosure: Wrap untrusted user inputs, external MCP payloads, and file contents in distinct XML-style delimiters (`<user_data>...</user_data>`).
2. Instruction Isolation: Explicitly instruct models to treat enclosed blocks as passive data, never as system instructions.
3. Token Smuggling Scanning: Strip or escape homoglyphs, zero-width characters, and base64-encoded jailbreak preambles.
4. AgentShield Interception: Verify all outbound tool calls generated by models against policy rules before OS execution.
"#,
    )
}

/// 14. Secret Entropy Scanner Skill
pub fn secret_entropy_scanner() -> EccSkill {
    EccSkill::new(
        "secret-entropy-scanner",
        "Shannon entropy secret detection, token format matching, and automated credential redaction",
        r#"# Secret & Credential Entropy Scanner

## Scanning Rules
1. Pattern Matching: Scan for known secret prefixes (`sk-ant-`, `sk-proj-`, `AIzaSy`, `xai-`, `ghp_`, `gho_`, `AWS_SECRET`).
2. Shannon Entropy Analysis: Flag high-entropy base64/hex strings (>4.5 bits/char) appearing in unexpected string literals.
3. Redaction Protocol: Automatically replace discovered tokens with `[REDACTED_SECRET_KEY]` before logging or stdout exposure.
4. Workspace Isolation: Block tools from reading `.env`, `.pem`, `id_rsa`, `id_ed25519`, and `.gnupg` directory files.
"#,
    )
}

/// 15. POSIX Shell Sanitizer Skill
pub fn posix_shell_sanitizer() -> EccSkill {
    EccSkill::new(
        "posix-shell-sanitizer",
        "POSIX shell AST validation, command injection prevention, and destructive execution interceptors",
        r#"# POSIX Shell Sanitizer & Command Interception

## Safety Rules
1. Prohibited Commands: Block destructive operations (`rm -rf /`, `mkfs`, `dd if=/dev/zero`, `:(){ :|:& };:`).
2. Argument Whitelisting: Enforce strict parameter validation on shell commands.
3. Path Confinement: Reject commands with traversal escapes (`../../../`) targeting root filesystems.
4. Subshell Isolation: Execute commands in dedicated subprocesses with strict timeouts and resource limits.
"#,
    )
}

/// 16. Bun Native APIs Skill
pub fn bun_native_apis() -> EccSkill {
    EccSkill::new(
        "bun-native-apis",
        "High-performance Bun native APIs, bun:sqlite vector storage, Bun.serve, and zero-transpile TypeScript",
        r#"# Bun Native Performance & APIs

## Best Practices
1. Fast I/O: Prefer `Bun.file(path)` and `Bun.write(path, content)` over legacy `node:fs` streams.
2. Built-in SQLite: Leverage `import { Database } from "bun:sqlite"` with prepared statements for zero-overhead persistence.
3. High-Throughput HTTP: Use `Bun.serve({ fetch(req) { ... } })` with native WebSocket pub/sub for sub-millisecond latency.
4. TypeScript Runtime: Rely on Bun's built-in transpiler without separate `tsc` build steps during development.
"#,
    )
}

/// 17. Seccomp Sandbox Rules Skill
pub fn seccomp_sandbox_rules() -> EccSkill {
    EccSkill::new(
        "seccomp-sandbox-rules",
        "Linux kernel system call filtering, namespace/cgroup isolation, and process jail safety policies",
        r#"# Linux Seccomp & Kernel Sandbox Rules

## Sandboxing Policy
1. System Call Whitelisting: Allow essential POSIX calls (`read`, `write`, `futex`, `epoll_wait`, `nanosleep`); block dangerous primitives (`ptrace`, `kexec_load`, `mount`, `reboot`).
2. Resource Quotas: Enforce memory quotas (`RLIMIT_AS`), CPU time caps (`RLIMIT_CPU`), and open descriptor limits (`RLIMIT_NOFILE`).
3. Filesystem Jails: Restrict write access strictly to designated temporary working directories.
"#,
    )
}

/// 18. Hexagonal Architecture Skill
pub fn hexagonal_architecture() -> EccSkill {
    EccSkill::new(
        "hexagonal-architecture",
        "Ports & Adapters clean architecture, decoupling business domain logic from databases and external APIs",
        r#"# Hexagonal (Ports & Adapters) Architecture

## Architectural Invariants
1. Core Domain Independence: Pure domain logic must have zero dependencies on frameworks, databases, or network protocols.
2. Ports as Traits/Interfaces: Define primary (driving) and secondary (driven) ports as language-native abstractions (`trait` / `interface`).
3. Adapters in Isolation: Database clients, HTTP endpoints, and CLI handlers live in distinct adapter modules wrapping ports.
4. Testability: The domain layer must be 100% unit-testable in memory without databases or external servers.
"#,
    )
}

/// 19. Backend Patterns Skill
pub fn backend_patterns() -> EccSkill {
    EccSkill::new(
        "backend-patterns",
        "Enterprise backend resilience: connection pooling, idempotent operations, circuit breakers, and graceful shutdown",
        r#"# Enterprise Backend Resilience Patterns

## Patterns
1. Idempotency: Enforce idempotency keys on mutating endpoints to handle network retries safely.
2. Connection Pooling: Use robust pools with connection health checks, max lifespans, and acquisition timeouts.
3. Circuit Breakers: Guard upstream microservices with circuit breakers to prevent cascading system failures.
4. Graceful Shutdown: Handle `SIGINT`/`SIGTERM` by draining active in-flight requests before terminating worker pools.
"#,
    )
}

/// 20. Coding Standards Skill
pub fn coding_standards() -> EccSkill {
    EccSkill::new(
        "coding-standards",
        "Strict code hygiene, cyclomatic complexity limits, zero dead code, and maintainability enforcement",
        r#"# Engineering Coding Standards

## Standards Checklist
1. Cyclomatic Complexity: Keep function complexity under 10; decompose complex branched logic into small, testable helpers.
2. Zero Dead Code: Eliminate unused variables, dead imports, and obsolete comments.
3. Consistent Formatting: Follow standard linters and formatters (`cargo fmt`, `cargo clippy`, `prettier`).
4. Self-Documenting Code: Name identifiers for intent rather than implementation mechanics.
"#,
    )
}

/// 21. Structured Analysis Yourdon Skill
pub fn structured_analysis_yourdon() -> EccSkill {
    EccSkill::new(
        "structured-analysis-yourdon",
        "Modern Structured Analysis (Edward Yourdon): environmental modeling, context diagrams, event-response lists, leveled DFDs, and state transition diagrams",
        r#"# Modern Structured Analysis & Design (Edward Yourdon)

## 1. The Environmental Model (System Boundary Definition)
- Statement of Purpose: Define a concise 1-2 sentence statement defining exact objective and boundaries.
- Context Diagram (DFD Level-0): Represent the system as a single process 0 surrounded by external terminators.
- Event-Response List: Categorize stimuli into External Events (actors), Temporal Events (time), and State Events (internal thresholds).

## 2. The Behavioral Model (Leveled DFDs)
- Event Partitioning (DFD Level-1): Draw exactly one process bubble per event in the event list.
- Data Stores: Shared memory/databases between processes.
- Level-2 Decomposition: Decompose complex process bubbles until leaf nodes represent single cohesive transformations.

## 3. State Transition Diagrams (STDs)
- Model time-dependent behavior: States, Transitions, Guard Conditions, and Actions.
- Ensure exhaustive event handling with zero unhandled state deadlocks.
"#,
    )
}

/// 22. Modular Coupling & Cohesion Skill
pub fn modular_coupling_cohesion() -> EccSkill {
    EccSkill::new(
        "modular-coupling-cohesion",
        "Structured Systems Design (Meilir Page-Jones): module cohesion hierarchy, coupling reduction, fan-in/fan-out bounds, and transform/transaction factoring",
        r#"# Modular Systems Design: Cohesion & Coupling (Meilir Page-Jones)

## 1. The 7 Levels of Module Cohesion (Target: Functional Cohesion)
1. Functional (Highest): Performs exactly one problem-related task. Every line contributes to that task.
2. Sequential: Output of one step is direct input to the next step.
3. Communicational: Operates on the same shared input data set.
4. Procedural (Avoid): Grouped solely by execution order.
5. Temporal (Avoid): Grouped solely because they execute at the same time (e.g. init_everything).
6. Logical (Dangerous): Multi-branch switch doing unrelated tasks based on a flag.
7. Coincidental (Forbidden): Arbitrary groupings (e.g. utils.ts, misc.rs).

## 2. The 5 Levels of Module Coupling (Target: Data Coupling)
1. Data (Best): Modules communicate solely by passing discrete, typed arguments.
2. Stamp: Passing composite structs when only a few fields are needed; prune to pass only what is needed.
3. Control (Avoid): Passing flags (is_admin, mode) that alter internal control flow.
4. Common (Dangerous): Communicating via global shared mutable memory.
5. Content (Forbidden): Directly accessing or mutating another module's internal state.

## 3. Structural Factoring Heuristics
- Fan-out <= 7 (a module coordinates at most 7 subordinates).
- High fan-in is encouraged (maximize reuse of pure logic).
- File limit <= 300 lines, function limit <= 50 lines.
"#,
    )
}

/// 23. Data Dictionary & Mini-Specs Skill
pub fn data_dictionary_minispecs() -> EccSkill {
    EccSkill::new(
        "data-dictionary-minispecs",
        "Structured System Specification (Tom DeMarco): formal data dictionary definitions, structured English mini-specifications, and DFD conservation balancing",
        r#"# Structured Specification & Mini-Specs (Tom DeMarco)

## 1. Formal Data Dictionary Notation
- `=` is composed of (definition)
- `+` AND (concatenation)
- `[ | ]` OR (exclusive choice / discriminated union)
- `{}` Iteration / Array (0 or more occurrences)
- `()` Optional field (0 or 1 occurrence)
- `*...*` Semantic comment / unit invariant

## 2. Structured English Mini-Specifications
- Imperative Action Verbs: COMPUTE, VALIDATE, LOOKUP, DISPATCH, PERSIST, EMIT.
- Deterministic Control Structures: IF/THEN/ELSE, CASE/OF, FOR EACH, WHILE.
- Zero Ambiguity: Eliminate vague adjectives; all thresholds must be concrete constants.

## 3. Conservation & Balancing Rules
- Rule of Data Conservation: A process cannot create data from nothing or discard necessary data.
- Rule of Leveled Balancing: Child diagram inputs and outputs must exactly equal parent process bubble inputs and outputs.
"#,
    )
}

/// 24. Domain-Driven Design Skill
pub fn domain_driven_design() -> EccSkill {
    EccSkill::new(
        "domain-driven-design",
        "Domain-Driven Design (Eric Evans & Vlad Khononov): Ubiquitous Language, Bounded Contexts, Aggregate Roots, Value Objects, Domain Events, and Anti-Corruption Layers",
        r#"# Domain-Driven Design (Eric Evans & Vlad Khononov)

## 1. Strategic Design
- Ubiquitous Language: Shared, strictly defined domain vocabulary in code and speech.
- Bounded Contexts: Explicit architectural and linguistic boundaries; never merge contexts into god-models.
- Anti-Corruption Layer (ACL): Translate external/legacy models into pure internal domain types.

## 2. Tactical Design
- Value Objects: Immutable, self-validating, structural equality, no identity (e.g. Money, DocketNumber).
- Entities: Enduring identity across lifecycle; mutate only via domain methods.
- Aggregates & Aggregate Roots: Transactional consistency boundary; external callers reference ONLY the root.
- Domain Events: Past-tense immutable records (CaseRaffled, FeePaid) decoupling side effects.
"#,
    )
}

/// 25. Design by Contract Skill
pub fn design_by_contract() -> EccSkill {
    EccSkill::new(
        "design-by-contract",
        "Design by Contract (Bertrand Meyer): preconditions, postconditions, class invariants, defensive boundary validation, and fail-fast invariant enforcement",
        r#"# Design by Contract (Bertrand Meyer)

## 1. The Contract Triad
- Preconditions (`require`): Obligations on the caller. Violation indicates a caller bug; fail-fast.
- Postconditions (`ensure`): Guarantees made by the callee. Violation indicates a callee bug.
- Class/Aggregate Invariants (`invariant`): Truths that must hold before and after every public method.

## 2. Contract Principles
- Distinguish contract violations (programmer bugs -> panic/assert) from expected domain errors (user inputs -> Result::Err).
- Never catch and swallow contract violations.
- Derive property tests directly from contract preconditions and postconditions.
"#,
    )
}

/// 26. Statechart & FSM Modeling Skill
pub fn statechart_fsm_modeling() -> EccSkill {
    EccSkill::new(
        "statechart-fsm-modeling",
        "Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks): finite state machines, orthogonal regions, guarded transitions, entry/exit actions, and deadlock-free event lifecycles",
        r#"# Hierarchical Statecharts & FSM Modeling (David Harel & Ian Horrocks)

## 1. Statechart Formalisms
- Superstates & Substates: Hierarchical state clustering to eliminate transition explosion.
- Orthogonal Regions: Concurrent independent state machines in a single entity (e.g. Judicial Track || Financial Track).
- Guarded Transitions: Event [GuardCondition] / Action -> TargetState.
- Entry/Exit Actions: Guaranteed execution on entering and leaving states.

## 2. Determinism & Safety
- Run-to-Completion: Events are fully processed before the next event begins.
- Exhaustive Coverage: In Rust/TypeScript, model states as closed enums; handle every event explicitly.
- Make invalid transitions unrepresentable in the type system.
"#,
    )
}

/// 27. Data-Intensive Architecture Skill
pub fn data_intensive_architecture() -> EccSkill {
    EccSkill::new(
        "data-intensive-architecture",
        "Data-Intensive Applications Architecture (Martin Kleppmann): transactional isolation, ACID guarantees, idempotency keys, write-ahead logs, CQRS, eventual consistency, and distributed consensus",
        r#"# Designing Data-Intensive Architecture (Martin Kleppmann)

## 1. The Core Trinity
- Reliability: Fault tolerance; absorb individual component faults without system failures.
- Scalability: Characterize load (throughput, fan-out, p99 latencies) and scale bottlenecks.
- Maintainability: Operability, simplicity, and schema evolvability.

## 2. Transactions & Concurrency
- Isolation levels: Understand dirty reads, non-repeatable reads, phantom reads, and write skew.
- Use Serializable or explicit pessimistic locking (SELECT FOR UPDATE) for financial and judicial allocations.

## 3. Distributed Patterns
- Idempotency Keys: Enforce on every mutating endpoint and tool call.
- Write-Ahead Log (WAL): Canonical append-only log as source of truth; derived views updated asynchronously.
- CQRS: Separate write command validation from high-performance read query projections.
"#,
    )
}

/// 28. Deep Modules & Complexity Skill (John Ousterhout)
pub fn deep_modules_complexity() -> EccSkill {
    EccSkill::new(
        "deep-modules-complexity",
        "A Philosophy of Software Design (John Ousterhout): deep vs shallow modules, narrow interfaces hiding complex implementations, information hiding, defining errors out of existence, and eliminating pass-through abstractions",
        r#"# Deep Modules & Complexity Control (John Ousterhout)

## 1. Deep vs. Shallow Modules
- Deep Module: Simple, narrow interface concealing deep, sophisticated internal logic.
- Shallow Module: Wide or complex interface with trivial implementation; eliminate shallow wrappers.
- Information Hiding: Internal data formats, synchronization, and caching must never leak into API signatures.
- Information Leakage: Callers should never have to coordinate multi-step lifecycle sequences when a single call suffices.

## 2. Defining Errors Out of Existence
- Subsumption: Redefine semantics so edge cases are valid normal behavior (e.g. deleting non-existent item is a no-op).
- Masking: Recover or retry internally rather than bubbling transient errors.
- Aggregation: Handle errors at overarching subsystem boundaries.

## 3. Eliminating Pass-Through Anti-Patterns
- Flatten pass-through methods that merely forward parameters without transformation.
- Avoid pass-through arguments across layers using context or constructor bindings.
"#,
    )
}

/// 29. Legacy Code Seams & Characterization Skill (Michael Feathers)
pub fn legacy_seams_characterization() -> EccSkill {
    EccSkill::new(
        "legacy-seams-characterization",
        "Working Effectively with Legacy Code (Michael Feathers): test-harness establishment, identifying seams, sensing and separation, characterization testing, and non-destructive sprout/wrap methods",
        r#"# Working Effectively with Legacy Code & Seams (Michael Feathers)

## 1. The Legacy Dilemma
- Legacy code is code without automated tests (including newly AI-generated unverified code).
- Establish seams without altering production behavior to bring code under test.

## 2. Seams: Object, Link, and Compile Seams
- Object Seams: Inject traits, interfaces, or subclass overrides.
- Sensing: Use seams to observe side-effects and values computed inside opaque functions.
- Separation: Use seams to decouple external dependencies (databases, network) during tests.

## 3. Characterization Testing & Safe Interventions
- Characterization Tests: Pin existing black-box behavior before refactoring.
- Sprout Method/Class: Write new features as pure, independently tested sprouts.
- Wrap Method/Class: Decorate legacy calls without mutating internal mechanics.
"#,
    )
}

/// 30. Code Smells & Atomic Refactoring Catalog Skill (Martin Fowler)
pub fn catalog_refactoring_smells() -> EccSkill {
    EccSkill::new(
        "catalog-refactoring-smells",
        "Refactoring & Code Smells Catalog (Martin Fowler): deterministic detection of architectural code smells, behavioral preservation, and atomic AST transformations",
        r#"# Code Smells & Atomic Refactoring Catalog (Martin Fowler)

## 1. Diagnostic Code Smells
- Primitive Obsession: Replace raw strings/numbers with validated Value Objects.
- Feature Envy: Move methods to the data structures they envy.
- Data Clumps: Bundle recurring parameter groups into Parameter Objects/Structs.
- Divergent Change vs. Shotgun Surgery: Separate divergent responsibilities; consolidate shotgun edits.

## 2. Atomic Behavior-Preserving Transformations
- Extract Function: Decompose high cognitive load blocks into expressive helpers.
- Replace Temp with Query: Eliminate mutable temp variables with pure deterministic queries.
- Replace Conditional with Polymorphism/Match: Replace sprawling if-else ladders with exhaustive pattern matching or traits.
- The Refactoring Rhythm: Make one atomic transformation, run test suite, ensure green, commit.
"#,
    )
}

/// 31. Production Resiliency & Stability Patterns Skill (Michael Nygard)
pub fn production_resilience_release_it() -> EccSkill {
    EccSkill::new(
        "production-resilience-release-it",
        "Production Resiliency Engineering (Michael Nygard - Release It!): circuit breakers, bulkheads, timeouts, steady-state stability, anti-fragility, and defense against cascading failures and retry storms",
        r#"# Production Resiliency & Stability Patterns (Michael Nygard - Release It!)

## 1. Stability Anti-Patterns
- Cascading Failures: Unbounded thread or pool blocking bringing down upstream services.
- Retry Storms: Blind retries without exponential backoff and randomized jitter.
- Unbounded Queues: OOM crashes under backpressure; enforce hard bounds.
- Missing Timeouts: Network calls must always define explicit connect and read timeouts.

## 2. Core Stability Patterns
- Circuit Breaker: Closed, Open, and Half-Open states guarding fragile downstreams.
- Bulkheads: Partition thread pools, connections, and compute into isolated failure domains.
- Fail Fast: Validate preconditions early before locking or allocating resources.
- Steady State: Prevent memory/disk leaks with automatic purging and resource recycling.
- Load Shedding: Drop excess load with backpressure rather than entering latency death spirals.
"#,
    )
}

/// 32. Evolutionary Architecture & Fitness Functions Skill (Ford, Parsons, Kua)
pub fn evolutionary_fitness_functions() -> EccSkill {
    EccSkill::new(
        "evolutionary-fitness-functions",
        "Building Evolutionary Architectures (Neal Ford, Rebecca Parsons, Patrick Kua): architectural fitness functions, automated structural verification, boundary integrity, and preventing architectural drift across AI iterations",
        r#"# Evolutionary Architecture & Fitness Functions (Ford, Parsons, Kua)

## 1. Architectural Fitness Functions
- Automated, objective verification tests asserting architectural characteristics.
- Prevent architectural drift and erosion across multi-prompt AI development sessions.

## 2. Categories of Fitness Functions
- Layering & Direction: Strict inward dependency rules (Hexagonal/Clean); Domain never imports CLI/Web.
- Acyclic Dependencies: Enforce DAG structure across modules with zero circular references.
- Complexity Budgets: Automated gates on cyclomatic complexity and max file/function lengths.
- Performance & Compliance: Automated benchmark regression gates and vulnerability scanners.
"#,
    )
}

/// 33. Temporal Invariants & Formal Specification Skill (Lamport & Wayne)
pub fn temporal_invariants_tla() -> EccSkill {
    EccSkill::new(
        "temporal-invariants-tla",
        "Temporal Invariants & Formal Specification (Leslie Lamport & Hillel Wayne): safety invariants, liveness guarantees, state space exhaustion, and race-free concurrent and distributed state modeling",
        r#"# Temporal Invariants & State Space Verification (Lamport / Wayne)

## 1. Safety vs. Liveness
- Safety Properties ('Nothing bad happens'): Invariants that must hold across every reachable state.
- Liveness Properties ('Something good eventually happens'): Guarantees of forward progress without deadlock or livelock.

## 2. State Space Modeling Before Coding
- Minimal State Tuple: Define discrete state variables explicitly before writing async logic.
- Guarded Transitions: Explicitly define enabled conditions for every state mutation.
- Concurrency Interleaving: Model concurrent interleavings to eliminate race conditions.
- Type-Level Guarantees: Make invalid states unrepresentable in the type system.
"#,
    )
}

/// 34. Conceptual Integrity & Engineering Over Time Skill (Brooks & Winters)
pub fn conceptual_integrity_systems() -> EccSkill {
    EccSkill::new(
        "conceptual-integrity-systems",
        "Conceptual Integrity & Software Engineering at Scale (Fred Brooks & Titus Winters): unified architectural vision, second-system syndrome avoidance, Hyrum's law, and sustainability over time",
        r#"# Conceptual Integrity & Engineering Over Time (Brooks & Winters)

## 1. Conceptual Integrity (Fred Brooks)
- The Central Virtue: A unified architectural vision outweighs an accumulation of uncoordinated features.
- Singular Design Dialect: Enforce consistent naming, error handling, and concurrency patterns across all modules.
- Second-System Syndrome: Resist over-complicating extensions with speculative features.

## 2. Software Engineering Over Time (Titus Winters)
- Programming Integrated Over Time: Design code to be maintainable, upgradeable, and decay-resistant for years.
- Hyrum's Law: All observable behaviors become contractual dependencies; explicitly encapsulate internals.
- Shift-Left Verification: Catch regressions as early as possible in the development lifecycle.
"#,
    )
}

/// 35. Refactoring UI Skill (Adam Wathan & Steve Schoger)
pub fn refactoring_ui() -> EccSkill {
    EccSkill::new(
        "refactoring-ui",
        "Tactical visual design & layout refactoring (Wathan & Schoger): grayscale-first design, optical alignment, 8pt spatial grid, layered elevation shadows, typography contrast scales. Triggers: grayscale first, 8pt grid, optical balance, typography scale, layered shadows, visual hierarchy.",
        r#"# Refactoring UI Engineering Skill

## 1. Core Principles
- Grayscale First: Formulate hierarchy, whitespace, and visual balance in monochrome before introducing color.
- Hierarchy Over Pure Sizing: Use font weight, contrast (text-slate-900 vs text-slate-500), and spatial isolation.
- Systematic Spacing: Enforce an 8pt/4pt scale (4, 8, 12, 16, 24, 32, 48, 64px).
- Optical Balancing: Shift asymmetric shapes manually by 1–2px for visual centering and optical balance.
- Layered Elevation: Ambient soft drop shadows combined with directional key-light shadows.
"#,
    )
}

/// 36. Microinteractions & Tactile Feedback Skill (Dan Saffer)
pub fn microinteractions_design() -> EccSkill {
    EccSkill::new(
        "microinteractions-design",
        "Microinteractions & Tactile Feedback (Dan Saffer): trigger-rule-feedback-loops, spring physics, sub-50ms button press, optimistic UI, skeleton shimmers, tactile states. Triggers: spring physics, sub-50ms, button press, optimistic ui, tactile feedback, skeleton shimmer, microinteractions.",
        r#"# Microinteractions & Tactile Feedback Skill

## 1. 4-Part Interaction Anatomy
- Trigger: User-initiated or system-initiated event.
- Rules: State machine and programmatic constraints.
- Feedback: Real-time sensory response (<= 50ms).
- Loops & Modes: Recurrence parameters and transient UI states.

## 2. Spring Physics
- Damped Harmonic Oscillator: Replace mechanical bezier curves with physical springs (k=260, damping=20).
- Immediate Response: Instant feedback on pointerdown (scale-98, tint) rather than awaiting mouseup.
- Skeleton Shimmers: Preserve layout stability and eliminate Cumulative Layout Shift (CLS).
"#,
    )
}

/// 37. Laws of UX Skill (Jon Yablonski)
pub fn laws_of_ux() -> EccSkill {
    EccSkill::new(
        "laws-of-ux",
        "Laws of UX & Cognitive Ergonomics (Jon Yablonski): Doherty threshold (<400ms), Hick's law, progressive disclosure, Fitts's law, Miller's law (7±2), Jakob's law, Aesthetic-Usability effect. Triggers: doherty threshold, hicks law, progressive disclosure, fitts law, millers law, cognitive ergonomics, laws of ux.",
        r#"# Laws of UX Engineering Skill

## 1. Cognitive Ergonomics Standards
- Doherty Threshold: Provide visual acknowledgement in < 100ms; full render in < 400ms.
- Hick's Law: Employ progressive disclosure; present no more than 3-5 primary actions per context.
- Fitts's Law: Enlarge touch targets to >= 44x44px; anchor high-frequency actions to viewport edges/bottom.
- Miller's Law: Chunk complex data into discrete visual modules (cards, grouped rows).
- Jakob's Law: Adhere to familiar mental models for navigation and search.
- Aesthetic-Usability Effect: Visual polish and harmony increase user tolerance and perceived performance.
"#,
    )
}

/// 38. Design Systems & Token Architecture Skill (Alla Kholmatova & Brad Frost)
pub fn design_systems_tokens() -> EccSkill {
    EccSkill::new(
        "design-systems-tokens",
        "Design Systems & Atomic Token Architecture (Kholmatova & Frost): 3-tier token hierarchy, semantic tokens, atomic design composition (atoms-to-pages), slot patterns. Triggers: 3-tier token hierarchy, semantic tokens, atomic design, design systems, design tokens, slot composition.",
        r#"# Design Systems & Token Architecture Skill

## 1. 3-Tier Token Hierarchy
- Tier 1 (Global Primitives): Raw palette and scales (--slate-900, --space-4).
- Tier 2 (Semantic Intent): Contextual variables (--surface-canvas, --text-primary).
- Tier 3 (Component Binding): Bound scoped properties (--button-primary-bg).

## 2. Atomic Composition
- Atoms -> Molecules -> Organisms -> Templates -> Pages.
- Slot Composition: Favor compound slots over 30+ prop explosion anti-patterns.
"#,
    )
}

/// 39. About Face & Power-User Ergonomics Skill (Alan Cooper)
pub fn about_face_interaction_design() -> EccSkill {
    EccSkill::new(
        "about-face-interaction-design",
        "About Face & Power-User Ergonomics (Alan Cooper): software posture theory (sovereign posture, transient, daemonic), eliminate excise, reversible undo stacks, cmd+k command palette, undo banner. Triggers: sovereign posture, cmd+k, command palette, eliminate excise, undo banner, interaction design, about face.",
        r#"# About Face & Power-User Ergonomics Skill

## 1. Software Posture Theory
- Sovereign Posture: High-density, dark/subdued palettes, deep keyboard shortcuts, multi-pane workspaces.
- Transient Posture: Single-function utilities with immediate dismissibility.
- Daemonic Posture: Background processes and status indicators.

## 2. Elimination of Excise
- Reversible Actions: Replace intrusive modal confirmation alerts with optimistic deletes and 1-click Undo.
- Universal Command Palette: Index all actions and navigation under Cmd+K / Ctrl+K.
- State Persistence: Never discard window bounds, scroll offsets, or drafts.
"#,
    )
}

/// 40. Designing for Emotion & Delight Skill (Aarron Walter)
pub fn designing_for_emotion() -> EccSkill {
    EccSkill::new(
        "designing-for-emotion",
        "Designing for Emotion & Delight (Aarron Walter): Maslow emotional hierarchy, delight hierarchy, creative empty state, empathetic error state handling, brand personality, celebratory confetti milestones. Triggers: delight hierarchy, empathetic error state, celebratory confetti, empty state, emotional design, designing for emotion.",
        r#"# Designing for Emotion Skill

## 1. Emotional Hierarchy
- Functional -> Reliable -> Usable -> Pleasurable & Delightful.

## 2. Humanized Interactions
- Creative Empty States: Turn zero-data screens into narrative invitations with clear primary CTAs.
- Empathetic Errors: Explain clearly without jargon, confirm data safety, and offer 1-click recovery.
- Celebratory Milestones: Reward user completions with tasteful micro-delight (confetti, achievement badges).
"#,
    )
}

/// 41. Kronos Candlestick K-Line Discrete Tokenization Skill
pub fn kronos_kline_modeling() -> EccSkill {
    EccSkill::new(
        "kronos-kline-modeling",
        "Financial Candlestick (K-line) discrete tokenization using Binary Spherical Quantization (BSQ), autoregressive Transformer architectures, hierarchical dual decoding (s1 price tokens, s2 volume/residual tokens), and synthetic market generation. Triggers: kronos, k-line, candlestick, bsq, binary spherical quantization, dual decoding, market simulation, synthetic market generation, ohlcv sliding window.",
        r#"# Kronos Financial Candlestick Modeling Skill

## 1. Architectural Foundations
- Sliding-Window Invariant Normalization: Anchor OHLC values relative to reference price P_base = C_{t-W}; standardize volume against trailing rolling EMA.
- Binary Spherical Quantization (BSQ): Project continuous latent embeddings onto unit hypersphere S^{D-1}, followed by sign binarization into discrete codebook tokens: q_b = sign(z_b / ||z||_2).
- Hierarchical Dual Decoding: Autoregressively decompose interval generation into s1 (macro price direction & trend) and s2 (microstructure volume & residual spread conditioned on s1).
- Market Physics Invariants: Strictly preserve candlestick boundaries: Low <= min(Open, Close) <= max(Open, Close) <= High, Volume >= 0.

## 2. Quantitative Formulation & Loss
- Hypersphere projection: z_tilde = z / (||z||_2 + eps).
- Token ID: k_tau = sum_{b=0}^{B-1} 2^b * I(q_b > 0).
- Straight-Through Estimator (STE): z_q = z_tilde + stop_gradient(q(z_tilde) - z_tilde).
- Joint Likelihood: P(S_{1:T}) = prod_{t=1}^T P(s_1^t | s_1^{<t}, s_2^{<t}) * P(s_2^t | s_1^{<=t}, s_2^{<t}).
"#,
    )
}

/// 42. Microsoft Qlib Alpha Engineering Skill
pub fn qlib_alpha_engineering() -> EccSkill {
    EccSkill::new(
        "qlib-alpha-engineering",
        "Microsoft Qlib AI-driven quantitative investment pipeline, alpha factor engineering, Barra risk factor neutralization (market beta, size, industry, volatility), turnover cost/slippage modeling, and Top-K backtesting with RankIC evaluation. Triggers: qlib, alpha factors, barra risk neutralization, top-k backtesting, rankic, information coefficient, icir, portfolio turnover, slippage modeling.",
        r#"# Microsoft Qlib Alpha Engineering Skill

## 1. Core Principles
- Point-in-Time Data Integrity: Strict elimination of look-ahead bias and survivorship bias across cross-sectional universes.
- Barra Multi-Factor Neutralization: Orthogonalize raw alpha factor predictions against systematic risk exposures (Market Beta, Log Market Cap, Industry Dummies, Volatility).
- Turnover & Slippage Modeling: Model realistic turnover drag and execution frictions: Cost_t = Turnover_t * (c_fee + c_tax + Slippage(V_t)).
- Non-Parametric RankIC & ICIR: Evaluate factor predictive performance via cross-sectional Spearman rank correlation.

## 2. Mathematical Rigor
- OLS Risk Neutralization: f_t = X_t * beta_t + epsilon_t.
- Regularized Annihilator: M_t = I - X_t * (X_t^T * X_t + lambda * I)^{-1} * X_t^T; neutralized alpha f_tilde = M_t * f_t.
- RankIC = Corr(rank(f_tilde_t), rank(r_{t+1})).
- Annualized ICIR = (Mean(RankIC) / Std(RankIC)) * sqrt(252). Target Mean(RankIC) > 0.05, ICIR > 0.8.
"#,
    )
}

/// 43. Amazon Chronos & Salesforce MOIRAI Forecasting Skill
pub fn chronos_moirai_forecasting() -> EccSkill {
    EccSkill::new(
        "chronos-moirai-forecasting",
        "Universal time series foundation models (Amazon Chronos, Salesforce MOIRAI), zero-shot probabilistic forecasting, patch-based multi-resolution architectures, and quantile Value-at-Risk (VaR / CVaR) estimation. Triggers: chronos, moirai, zero-shot time series, probabilistic forecasting, quantile prediction, value-at-risk, var, cvar, patch forecasting, universal time series.",
        r#"# Chronos & MOIRAI Time Series Foundation Models Skill

## 1. Architectural Foundations
- Universal Time Series Modeling: Pre-trained foundation models replacing single-asset statistical fits across diverse financial instruments.
- Patch-Based Multi-Resolution Tokenization: Decompose real-valued series into multi-scale patches (8, 16, 32, 64) with Any-Variate attention.
- Zero-Shot Probabilistic Forecasting: Generate full quantile distributions y_hat^{(q)} for q in {0.01, 0.05, 0.10, 0.50, 0.90, 0.95, 0.99}.
- Quantile Value-at-Risk (VaR) & Expected Shortfall (CVaR): VaR_alpha = -y_hat^{(alpha)}, CVaR_alpha = -(1/alpha) * integral_0^alpha y_hat^{(u)} du.

## 2. Loss & Calibration
- Pinball Loss (Quantile Loss): L_q(y, y_hat) = max(q * (y - y_hat), (1 - q) * (y_hat - y)).
- Reversible Instance Normalization (RevIN): Affine de-trending and instance normalization preserving relative percentage volatility.
- Quantile Monotonicity: Enforce non-crossing quantile invariants: y_hat^{(q1)} <= y_hat^{(q2)} for all q1 < q2.
"#,
    )
}

/// 44. FinGPT & FinRL Multimodal Financial NLP Skill
pub fn fingpt_multimodal_nlp() -> EccSkill {
    EccSkill::new(
        "fingpt-multimodal-nlp",
        "Multimodal financial NLP (FinGPT) and deep reinforcement learning (FinRL) for SEC 10-K/10-Q filing analysis, sentiment alpha factor extraction, and optimal trade execution (PPO/DDPG). Triggers: fingpt, finrl, financial nlp, sec filing sentiment, 10-k analysis, 10-q filings, sentiment factors, reinforcement learning execution, almgren-chriss, ppo trade execution.",
        r#"# FinGPT & FinRL Multimodal Financial NLP Skill

## 1. Architectural Foundations
- Domain-Adapted Financial LLMs: Parameter-efficient fine-tuning (LoRA) on EDGAR SEC filings, financial news, and conference calls.
- Longitudinal Item Parsing: Differential semantic comparison of Item 1A (Risk Factors), Item 7 (MD&A), and Item 7A (Market Risk).
- Sentiment Factor Construction: Exponential half-life decay aggregation of sentiment polarity, neutralized against market sentiment beta.
- Deep RL Execution Optimization (FinRL): MDP formulation of TWAP/VWAP liquidation with Almgren-Chriss market impact penalties.

## 2. Execution MDP & Reward
- State: s_k = (r_k, Spread_k, Imbalance_k, q_k / Q, k / N).
- Action: a_k in [0, 1] (fraction of remaining inventory executed).
- Almgren-Chriss Reward: R_k = (P_k - P_benchmark) * Delta_q_k - eta * (Delta_q_k)^2 - lambda * sigma^2 * q_k^2 * tau.
- Timestamp Grounding: Always index filings by EDGAR acceptance_datetime, never fiscal period end.
"#,
    )
}

/// 45. Microsoft RD-Agent Autonomous Alpha Mining Skill
pub fn rdagent_factor_mining() -> EccSkill {
    EccSkill::new(
        "rdagent-factor-mining",
        "Microsoft RD-Agent autonomous quantitative R&D, automated alpha factor mining, market anomaly hypothesis formulation, symbolic factor expression trees, collinearity filtering, and factor half-life decay management. Triggers: rdagent, rd-agent, autonomous alpha mining, rd-agent alpha mining, factor mining, hypothesis generation, factor expression tree, automated quant r&d, factor decay.",
        r#"# Microsoft RD-Agent Autonomous Alpha Mining Skill

## 1. Autonomous R&D Loop
- Economic Hypothesis Formulation: Ground factor generation in market microstructure anomalies (liquidity premium, PEAD, idiosyncratic volatility).
- Symbolic Factor Expression Trees: Represent factors as DAG expression trees over unary time-series operators (Ts_Mean, Ts_Std, Ts_Rank, Ts_DecayLinear) and cross-sectional operators (Cs_Rank, Cs_ZScore, Cs_Neutralize).
- Factor Collinearity Pruning: Reject candidate factors with cross-sectional correlation rho > 0.60 against existing factor library.
- Factor Decay & Half-Life Monitoring: Fit RankIC(t) = RankIC_0 * 2^{-t / tau}; auto-retire factors when ICIR < 0.30 or turnover > 40%.

## 2. Evaluation & Overfitting Guardrails
- Deflated Sharpe Ratio (DSR) & Holm-Bonferroni correction for multiple hypothesis testing.
- Tree Complexity Penalties: Cap tree depth <= 6 and node count <= 15 to prevent curve fitting.
- Net-of-Fee Requirement: Candidate alpha must sustain Sharpe > 1.5 after 15 bps two-way friction.
"#,
    )
}

/// 51. math-nature-of-code Skill
pub fn math_nature_of_code() -> EccSkill {
    EccSkill::new(
        "math-nature-of-code",
        "Physics simulations, vector kinematics, harmonic oscillations, Hooke's spring-damper dynamics, particle systems, and Craig Reynolds autonomous steering (seek, arrive, flocking boids: separation, alignment, cohesion) based on 'The Nature of Code' by Daniel Shiffman. Triggers: nature-of-code, boids, flocking, autonomous-steering, reynolds-steering, hooke-spring, particle-system, physics-simulation, harmonic-oscillator, vector-kinematics.",
        r#"# The Nature of Code: Physics, Vectors & Autonomous Swarms
- Semi-implicit Euler integration: v_{t+dt} = v_t + a_t * dt; x_{t+dt} = x_t + v_{t+dt} * dt.
- Damped harmonic oscillator: F_spring = -k * (x - x_rest) - c * v.
- Craig Reynolds steering: F_steer = clamp(v_desired - v_current, F_max).
- Flocking emergence: F_total = w_s * F_sep + w_a * F_ali + w_c * F_coh.
"#,
    )
}

/// 52. math-linear-algebra-savov Skill
pub fn math_linear_algebra_savov() -> EccSkill {
    EccSkill::new(
        "math-linear-algebra-savov",
        "Intuitive linear algebra, vector spaces, dot products, orthogonal projections, Gram-Schmidt orthogonalization, eigenvalues/eigenvectors, and Singular Value Decomposition (SVD) for low-rank embeddings based on 'No Bullshit Guide to Linear Algebra' by Ivan Savov and Sheldon Axler. Triggers: linear-algebra, vector-spaces, dot-product, orthogonal-projection, svd, singular-value-decomposition, eigenvalues, eigenvectors, gram-schmidt, low-rank-matrix.",
        r#"# Linear Algebra Done Right: Vector Spaces & Projections
- Orthogonal projection: P_W = A * (A^T * A)^{-1} * A^T.
- Cosine similarity: cos(theta) = (u . v) / (||u|| * ||v|| + eps).
- SVD factorization: A = U * Sigma * V^T.
- Low-rank approximation: A_k = sum_{i=1}^k sigma_i * u_i * v_i^T.
"#,
    )
}

/// 53. math-high-dimensional-data Skill
pub fn math_high_dimensional_data() -> EccSkill {
    EccSkill::new(
        "math-high-dimensional-data",
        "High-dimensional geometry, unit ball surface crust concentration, random vector near-orthogonality, Johnson-Lindenstrauss dimension reduction, and Markov chain random walks based on 'Foundations of Data Science' by Blum, Hopcroft, and Kannan. Triggers: high-dimensional-geometry, unit-ball-crust, johnson-lindenstrauss, random-projection, curse-of-dimensionality, random-walks, markov-chains, embedding-geometry.",
        r#"# Foundations of Data Science: High-Dimensional Geometry
- Volume concentration: Vol(B_d(1 - eps)) / Vol(B_d(1)) = (1 - eps)^d <= e^{-eps * d}.
- High-d orthogonality: For random unit vectors u, v, |u . v| <= sqrt((2 * ln(2/delta)) / d).
- Johnson-Lindenstrauss lemma: Distances preserved within (1 +/- eps) with target dim k = O(eps^{-2} * ln n).
"#,
    )
}

/// 54. math-information-inference Skill
pub fn math_information_inference() -> EccSkill {
    EccSkill::new(
        "math-information-inference",
        "Information theory, Shannon entropy, mutual information, Kullback-Leibler (KL) divergence, cross-entropy, Bayesian inference, and token entropy for hallucination detection based on David J.C. MacKay. Triggers: information-theory, shannon-entropy, mutual-information, kl-divergence, token-entropy, perplexity, hallucination-detection, bayesian-inference, mcmc, cross-entropy.",
        r#"# Information Theory, Inference & Learning
- Shannon entropy: H(X) = -sum p(x) * log2(p(x)).
- KL divergence: D_{KL}(P || Q) = sum P(x) * log(P(x) / Q(x)).
- Mutual information: I(X; Y) = H(X) - H(X|Y).
- Token uncertainty: Hallucination spike when H_token > tau_entropy.
"#,
    )
}

/// 55. math-causal-inference Skill
pub fn math_causal_inference() -> EccSkill {
    EccSkill::new(
        "math-causal-inference",
        "Causal inference, Structural Causal Models (SCM), Directed Acyclic Graphs (DAGs), d-separation, backdoor criterion, do-calculus, and counterfactual analysis based on 'The Book of Why' by Judea Pearl. Triggers: causal-inference, causal-dag, do-calculus, backdoor-criterion, confounding-bias, counterfactuals, structural-causal-model, d-separation, root-cause-analysis.",
        r#"# Causal Inference & Do-Calculus
- Observational vs Interventional: P(Y | X=x) != P(Y | do(X=x)).
- Backdoor adjustment: P(Y | do(X=x)) = sum_z P(Y | X=x, Z=z) * P(Z=z).
- Counterfactual: P(Y_{X=x} | X=x', Y=y').
"#,
    )
}

/// 56. math-category-theory Skill
pub fn math_category_theory() -> EccSkill {
    EccSkill::new(
        "math-category-theory",
        "Category theory for software architecture, functors, monads, natural transformations, Kleisli categories, and Railway-Oriented Programming (Result/Option monads) based on 'Category Theory for Programmers' by Bartosz Milewski. Triggers: category-theory, functors, monads, kleisli-category, natural-transformations, compositionality, railway-oriented-programming, morphisms, monoid.",
        r#"# Category Theory for Programmers
- Morphism composition: (g . f)(x) = g(f(x)), associative with identity id_A.
- Functor F: maps objects A -> F(A) and morphisms f -> F(f).
- Monad (T, eta, mu): unit eta: I -> T, multiplication mu: T^2 -> T.
- Kleisli composition for error pipelines: f >=> g = mu . T(g) . f.
"#,
    )
}

/// 57. math-game-engine-geometry Skill
pub fn math_game_engine_geometry() -> EccSkill {
    EccSkill::new(
        "math-game-engine-geometry",
        "Game engine 3D mathematics, unit quaternions without gimbal lock, Spherical Linear Interpolation (Slerp), 4x4 affine transforms, and ray-AABB intersection tests based on Eric Lengyel. Triggers: game-engine-math, quaternions, slerp, gimbal-lock, affine-transform, ray-intersection, aabb-box, 3d-geometry, perspective-projection.",
        r#"# Foundations of Game Engine Development: Quaternions & Transforms
- Unit quaternion: q = [s, v] = cos(theta/2) + sin(theta/2) * (u_x i + u_y j + u_z k).
- Slerp interpolation: Slerp(q1, q2, t) = (sin((1-t)*theta)*q1 + sin(t*theta)*q2) / sin(theta).
- Vector rotation: v' = q * [0, v] * q^{-1}.
"#,
    )
}

/// 58. math-code-first-calculus Skill
pub fn math_code_first_calculus() -> EccSkill {
    EccSkill::new(
        "math-code-first-calculus",
        "Code-first calculus, numerical finite-difference gradients, gradient descent optimization, 2D/3D polygon rasterization, and coordinate transforms in Python based on Paul Orland. Triggers: code-first-calculus, numerical-gradient, gradient-descent, polygon-rasterization, coordinate-transform, finite-difference, math-for-programmers.",
        r#"# Math for Programmers: Code-First Calculus & Optimization
- Finite difference gradient: df/dx = (f(x + h) - f(x - h)) / (2 * h).
- Gradient descent step: theta_{t+1} = theta_t - alpha * grad(J(theta_t)).
- 2D cross product for polygon orientation: (B_x - A_x)*(C_y - A_y) - (B_y - A_y)*(C_x - A_x).
"#,
    )
}

/// 59. math-bayesian-reasoning Skill
pub fn math_bayesian_reasoning() -> EccSkill {
    EccSkill::new(
        "math-bayesian-reasoning",
        "Bayesian statistics, prior/posterior distributions, Beta-Binomial conjugate updating, Bayes factors, and Thompson Sampling multi-armed bandit routing based on 'Bayesian Statistics the Fun Way' by Will Kurt. Triggers: bayesian-statistics, bayes-theorem, beta-binomial, posterior-probability, thompson-sampling, multi-armed-bandit, prior-distribution, epistemic-uncertainty.",
        r#"# Bayesian Statistics: Belief Updating & Multi-Armed Bandits
- Bayes' rule: P(theta | D) = (P(D | theta) * P(theta)) / P(D).
- Beta-Binomial update: Beta(alpha + successes, beta + failures).
- Thompson Sampling: sample theta_i ~ Beta(alpha_i, beta_i), pick arm with max sample.
"#,
    )
}

/// 60. math-ml-foundations Skill
pub fn math_ml_foundations() -> EccSkill {
    EccSkill::new(
        "math-ml-foundations",
        "Mathematical foundations of machine learning, matrix calculus, Jacobians, Hessians, multivariate Gaussian distributions, PCA eigendecomposition, and continuous optimization based on Deisenroth, Faisal, and Ong. Triggers: machine-learning-math, matrix-calculus, jacobian, hessian, multivariate-gaussian, pca, continuous-optimization, lagrange-multipliers.",
        r#"# Mathematics for Machine Learning: Optimization & Distributions
- Multivariate Gaussian: N(x; mu, Sigma) = (2*pi)^{-d/2} * |Sigma|^{-1/2} * exp(-1/2 * (x-mu)^T * Sigma^{-1} * (x-mu)).
- Principal Component Analysis: Sigma * v_i = lambda_i * v_i.
- Lagrangian: L(x, lambda) = f(x) + sum lambda_i * g_i(x).
"#,
    )
}

/// 61. math-networks-crowds-markets Skill
pub fn math_networks_crowds_markets() -> EccSkill {
    EccSkill::new(
        "math-networks-crowds-markets",
        "Graph theory, network centrality, PageRank power iteration, information cascades, small-world networks, and multi-agent swarm topologies based on Easley and Kleinberg. Triggers: network-graph-theory, pagerank, network-centrality, betweenness-centrality, information-cascades, small-world, agent-swarm-topology, graph-laplacian.",
        r#"# Networks, Crowds & Markets: Graph Topology & Cascades
- PageRank iteration: r_{t+1} = d * M * r_t + (1 - d) / N * 1.
- Degree & betweenness centrality: C_B(v) = sum_{s != v != t} (sigma_{st}(v) / sigma_{st}).
- Small-world Watts-Strogatz: high clustering coefficient C with small characteristic path length L.
"#,
    )
}

/// 62. math-convex-optimization Skill
pub fn math_convex_optimization() -> EccSkill {
    EccSkill::new(
        "math-convex-optimization",
        "Convex optimization, convex sets and functions, Karush-Kuhn-Tucker (KKT) optimality conditions, Lagrangian duality, and model latency/cost allocation based on Stephen Boyd and Lieven Vandenberghe. Triggers: convex-optimization, kkt-conditions, lagrangian-duality, convex-sets, slaters-condition, quadratic-programming, dual-problem.",
        r#"# Convex Optimization: Duality & KKT Conditions
- Convex problem: min f_0(x) s.t. f_i(x) <= 0, h_j(x) = 0.
- KKT conditions: Stationarity (grad f_0 + sum lambda_i grad f_i + sum nu_j grad h_j = 0), Primal feasibility, Dual feasibility (lambda_i >= 0), Complementary slackness (lambda_i * f_i(x) = 0).
"#,
    )
}

/// 63. math-probability-logic Skill
pub fn math_probability_logic() -> EccSkill {
    EccSkill::new(
        "math-probability-logic",
        "Probability theory as extended formal logic under incomplete information, Cox's consistency theorems, and the Principle of Maximum Entropy (MaxEnt) based on E.T. Jaynes. Triggers: probability-as-logic, cox-theorems, maximum-entropy, maxent, epistemic-probability, bayesian-logic, inductive-reasoning, uninformative-priors.",
        r#"# Probability Theory: The Logic of Science
- Cox's consistency axioms: transitiveness of plausibility, complementary negations, structural uniqueness.
- Maximum Entropy Principle: max H(p) = -sum p_i ln p_i subject to expectation constraints sum p_i f_k(x_i) = <f_k>.
- Grounding uncertainty: reject false certainty without sufficient observational evidence.
"#,
    )
}

/// 64. math-programmers-discrete Skill
pub fn math_programmers_discrete() -> EccSkill {
    EccSkill::new(
        "math-programmers-discrete",
        "Programmer-focused discrete mathematics, Fast Fourier Transform (FFT) for motion/audio signals, Graph Laplacian, and spectral bisection based on Jeremy Kun. Triggers: programmers-discrete-math, fast-fourier-transform, fft, graph-laplacian, spectral-bisection, polynomial-rings, fourier-analysis, signal-processing.",
        r#"# A Programmer's Introduction to Mathematics: FFT & Graph Laplacians
- Discrete Fourier Transform: X_k = sum_{n=0}^{N-1} x_n * exp(-i * 2*pi * k * n / N).
- Cooley-Tukey FFT: divide-and-conquer O(N log N) frequency transformation.
- Graph Laplacian: L = D - A; Fiedler vector (2nd smallest eigenvalue) partitions graph minimally.
"#,
    )
}

/// 65. math-intuitive-calculus Skill
pub fn math_intuitive_calculus() -> EccSkill {
    EccSkill::new(
        "math-intuitive-calculus",
        "Intuitive differential and integral calculus, instantaneous velocity, exponential decay, accumulation beneath curves, and PID controllers with anti-windup clamping based on 'Calculus Made Easy' by Thompson and Gardner. Triggers: intuitive-calculus, rates-of-change, velocity-accumulation, exponential-decay, pid-controller, anti-windup, numerical-integration.",
        r#"# Calculus Made Easy: Intuitive Differentials & PID Dynamics
- Infinitesimals: dy/dx represents instantaneous rate of change.
- Chain rule: dy/dx = (dy/du) * (du/dx).
- PID controller with anti-windup: u(t) = K_p * e(t) + K_i * int_0^t e(tau) dtau + K_d * de/dt clamped to [u_min, u_max].
"#,
    )
}

/// 66. math-concrete-discrete Skill
pub fn math_concrete_discrete() -> EccSkill {
    EccSkill::new(
        "math-concrete-discrete",
        "Concrete discrete mathematics, recurrence relations, generating functions, discrete sums, binomial coefficients, and asymptotic bounds for agent token recursion based on Graham, Knuth, and Patashnik. Triggers: concrete-mathematics, recurrence-relations, generating-functions, discrete-sums, binomial-coefficients, knuth-math, asymptotic-analysis, token-budget-bound.",
        r#"# Concrete Mathematics: Discrete Recurrences & Asymptotics
- Recurrence solution via master theorem: T(n) = a * T(n/b) + f(n).
- Ordinary generating functions: G(z) = sum_{n=0}^infty a_n * z^n.
- Bounding recursive agent swarms: ensure branching factor b and depth d satisfy sum b^k <= Token_Budget.
"#,
    )
}

/// 67. math-visual-topology Skill
pub fn math_visual_topology() -> EccSkill {
    EccSkill::new(
        "math-visual-topology",
        "Visual geometry, Gaussian curvature, Euler characteristic, minimal surfaces, manifold topology, and 2-manifold mesh integrity validation based on David Hilbert and Stephan Cohn-Vossen. Triggers: visual-geometry, gaussian-curvature, euler-characteristic, manifold-topology, minimal-surfaces, mesh-validation, hilbert-geometry.",
        r#"# Geometry and the Imagination: Curvature & Manifolds
- Gaussian curvature: K = kappa_1 * kappa_2; Theorema Egregium (intrinsic invariance).
- Euler-Poincare characteristic: chi = V - E + F = 2 - 2g for closed surface of genus g.
- 2-manifold invariant: each edge shared by exactly two faces; vertex star homeomorphic to disk.
"#,
    )
}

/// 68. math-chaos-fractals Skill
pub fn math_chaos_fractals() -> EccSkill {
    EccSkill::new(
        "math-chaos-fractals",
        "Fractal geometry, scale invariance, power laws, Hausdorff dimension, Lorenz strange attractors, and deterministic chaos simulations based on Manfred Schroeder. Triggers: chaos-theory, fractals, strange-attractor, lorenz-attractor, power-laws, lyapunov-exponent, mandelbrot, scale-invariance.",
        r#"# Fractals, Chaos & Power Laws
- Hausdorff dimension: D = lim_{eps -> 0} (log N(eps) / log(1/eps)).
- Lorenz chaotic system: dx/dt = sigma*(y - x), dy/dt = x*(rho - z) - y, dz/dt = x*y - beta*z.
- Lyapunov sensitivity: delta x(t) approx delta x_0 * exp(lambda * t).
"#,
    )
}

/// 69. math-numerical-methods Skill
pub fn math_numerical_methods() -> EccSkill {
    EccSkill::new(
        "math-numerical-methods",
        "Scientific numerical computing, IEEE-754 floating-point stability, Runge-Kutta 4th Order (RK4) ODE solvers, Symplectic Velocity Verlet, and spline smoothing based on 'Numerical Recipes'. Triggers: numerical-methods, rk4, runge-kutta, floating-point-stability, velocity-verlet, cubic-spline, numerical-recipes, nan-prevention.",
        r#"# Numerical Recipes: RK4 & Floating-Point Stability
- Runge-Kutta 4th order: y_{n+1} = y_n + dt/6 * (k1 + 2*k2 + 2*k3 + k4).
- Symplectic Velocity Verlet: x(t+dt) = x(t) + v(t)*dt + 1/2*a(t)*dt^2; v(t+dt) = v(t) + 1/2*(a(t) + a(t+dt))*dt.
- Numerical safety: guard against catastrophic cancellation: sqrt(x + eps) - sqrt(x) = eps / (sqrt(x+eps) + sqrt(x)).
"#,
    )
}

/// 70. math-algorithmic-game-theory Skill
pub fn math_algorithmic_game_theory() -> EccSkill {
    EccSkill::new(
        "math-algorithmic-game-theory",
        "Algorithmic game theory, Nash equilibria, mechanism design, Vickrey-Clarke-Groves (VCG) truthful auctions, Price of Anarchy (PoA), and Shapley value attribution based on Nisan, Roughgarden, Tardos, and Vazirani. Triggers: algorithmic-game-theory, nash-equilibrium, mechanism-design, vcg-auction, price-of-anarchy, shapley-value, multi-agent-incentives, cooperative-games.",
        r#"# Algorithmic Game Theory: Multi-Agent Equilibrium & Incentives
- Nash equilibrium: no agent has incentive to unilaterally deviate u_i(s_i^*, s_{-i}^*) >= u_i(s_i, s_{-i}^*).
- Price of Anarchy: PoA = (Worst Nash Social Welfare) / (Optimal Social Welfare).
- Shapley value credit attribution: phi_i(v) = sum_{S subseteq N \ {i}} (|S|! * (|N| - |S| - 1)! / |N|!) * (v(S union {i}) - v(S)).
"#,
    )
}

/// Discover and load all ECC skills from a directory (scanning both `*.md` and `<dir>/SKILL.md`)
pub fn load_skills_from_dir(dir: impl AsRef<Path>) -> Vec<EccSkill> {
    let mut skills = Vec::new();
    let dir_ref = dir.as_ref();

    if !dir_ref.is_dir() {
        return skills;
    }

    if let Ok(entries) = fs::read_dir(dir_ref) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match EccSkill::from_file(&path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => warn!("Failed to load skill from '{}': {}", path.display(), e),
                }
            } else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                let skill_md_alt = path.join("skill.md");
                let target = if skill_md.is_file() {
                    Some(skill_md)
                } else if skill_md_alt.is_file() {
                    Some(skill_md_alt)
                } else {
                    None
                };

                if let Some(target_file) = target {
                    match EccSkill::from_file(&target_file) {
                        Ok(skill) => skills.push(skill),
                        Err(e) => warn!("Failed to load skill from '{}': {}", target_file.display(), e),
                    }
                }
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Retrieve an ECC skill by name, checking built-in skills first, then an optional disk directory
pub fn resolve_skill(name: &str, custom_dir: Option<&Path>) -> Option<EccSkill> {
    let default_skills_dir = Path::new(".ecc/skills");
    if custom_dir.is_none() || custom_dir == Some(default_skills_dir) {
        if let Some(skill) = global_dispatcher().get_skill(name) {
            return Some(skill);
        }
    }

    if let Some(skill) = find_built_in_skill(name) {
        return Some(skill);
    }

    if let Some(dir) = custom_dir {
        let lower = name.to_lowercase().replace('_', "-");

        // 1. Fast O(1) direct path lookup by directory name
        let candidate_dir = dir.join(&lower);
        if candidate_dir.is_dir() {
            let skill_md = candidate_dir.join("SKILL.md");
            let skill_md_alt = candidate_dir.join("skill.md");
            let target = if skill_md.is_file() {
                Some(skill_md)
            } else if skill_md_alt.is_file() {
                Some(skill_md_alt)
            } else {
                None
            };
            if let Some(file) = target {
                if let Ok(skill) = EccSkill::from_file(&file) {
                    return Some(skill);
                }
            }
        }

        // 2. Scan all loaded skills
        let loaded = load_skills_from_dir(dir);
        if let Some(skill) = loaded.iter().find(|s| {
            s.name.eq_ignore_ascii_case(&lower)
                || s.name.eq_ignore_ascii_case(lower.trim_start_matches("oracle-"))
                || s.name.eq_ignore_ascii_case(lower.trim_start_matches("oci-"))
                || format!("oracle-{}", s.name).eq_ignore_ascii_case(&lower)
                || format!("oci-{}", s.name).eq_ignore_ascii_case(&lower)
        }) {
            return Some(skill.clone());
        }
    }

    None
}

// =========================================================================
// Tagisan Automated Skill Dispatcher ("Right Tools/Skills for Right Job")
// =========================================================================

/// Pre-indexed metadata for deterministic sub-millisecond ranking
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    pub id: usize,
    pub name: String,
    pub domain: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub is_builtin: bool,
    /// Pre-computed normalized term frequency sparse vector: (term_id, normalized_weight) sorted by term_id
    pub tfidf_vector: Vec<(u32, f32)>,
    /// Pre-computed L2 norm
    pub norm: f32,
    /// Tokenized name terms for Jaccard overlap
    pub name_tokens: Vec<String>,
}

/// Result of dispatching a skill query
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchedSkill {
    pub skill: EccSkill,
    pub score: f32,
    pub matched_triggers: Vec<String>,
    pub domain: String,
}

/// In-memory hybrid skill dispatcher capable of ranking 3,840+ skills in < 0.5ms
pub struct SkillDispatcher {
    skills: Vec<SkillMetadata>,
    /// Inverted index: term_id -> list of skill IDs
    inverted_index: HashMap<u32, Vec<usize>>,
    /// Fast exact trigger map: lowercased_trigger -> list of skill IDs
    trigger_index: HashMap<String, Vec<usize>>,
    /// Fast exact name map: lowercased_name -> skill ID
    name_index: HashMap<String, usize>,
    /// Lexicon mapping term -> term_id
    vocab: HashMap<String, u32>,
    /// Inverse document frequencies
    idf: Vec<f32>,
    /// JIT cache of loaded full EccSkill bodies
    skill_cache: RwLock<HashMap<usize, EccSkill>>,
}

static GLOBAL_DISPATCHER: OnceLock<SkillDispatcher> = OnceLock::new();

/// Return a static reference to the lazily initialized global SkillDispatcher singleton
pub fn global_dispatcher() -> &'static SkillDispatcher {
    GLOBAL_DISPATCHER.get_or_init(SkillDispatcher::default_catalog)
}

impl SkillDispatcher {
    /// Initialize dispatcher from the standard locations (.ecc/skills + 20 built-ins)
    pub fn default_catalog() -> Self {
        let skills_dir = Path::new(".ecc/skills");
        let custom_dir = if skills_dir.exists() {
            Some(skills_dir)
        } else {
            None
        };
        Self::new(custom_dir)
    }

    /// Initialize dispatcher scanning built-ins and an optional custom skills directory
    pub fn new(custom_dir: Option<&Path>) -> Self {
        struct RawEntry {
            name: String,
            description: String,
            triggers: Vec<String>,
            file_path: Option<PathBuf>,
            is_builtin: bool,
        }

        let mut raw_entries: Vec<RawEntry> = Vec::new();
        let mut initial_cache = HashMap::new();

        // 1. Ingest built-in skills
        for s in all_built_in_skills() {
            let trigs = extract_triggers_from_text(&s.name, &s.description, &[]);
            let id = raw_entries.len();
            initial_cache.insert(id, s.clone());
            raw_entries.push(RawEntry {
                name: s.name.clone(),
                description: s.description.clone(),
                triggers: trigs,
                file_path: None,
                is_builtin: true,
            });
        }

        // 2. Fast scan disk directory frontmatters (reading first 3KB per file)
        if let Some(dir) = custom_dir {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let skill_md = path.join("SKILL.md");
                            let skill_md_alt = path.join("skill.md");
                            let target = if skill_md.is_file() {
                                Some(skill_md)
                            } else if skill_md_alt.is_file() {
                                Some(skill_md_alt)
                            } else {
                                None
                            };

                            if let Some(target_file) = target {
                                if let Ok(mut file) = File::open(&target_file) {
                                    let mut buf = [0u8; 8192];
                                    let read_bytes = file.read(&mut buf).unwrap_or(0);
                                    let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                    let folder_name = path.file_name().and_then(|f| f.to_str());
                                    if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, folder_name) {
                                        let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                        raw_entries.push(RawEntry {
                                            name,
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        });
                                    } else if let Some(fname) = folder_name {
                                        // Robust fallback for markdown files without standard --- delimiters
                                        let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(fname);
                                        let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                        let trigs = extract_triggers_from_text(fname, &desc, &[]);
                                        raw_entries.push(RawEntry {
                                            name: fname.to_string(),
                                            description: desc,
                                            triggers: trigs,
                                            file_path: Some(target_file),
                                            is_builtin: false,
                                        });
                                    }
                                }
                            }
                        } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                            if let Ok(mut file) = File::open(&path) {
                                let mut buf = [0u8; 8192];
                                let read_bytes = file.read(&mut buf).unwrap_or(0);
                                let header = String::from_utf8_lossy(&buf[..read_bytes]);
                                let file_stem = path.file_stem().and_then(|s| s.to_str());
                                if let Some((name, desc, raw_trigs)) = parse_frontmatter_metadata(&header, file_stem) {
                                    let trigs = extract_triggers_from_text(&name, &desc, &raw_trigs);
                                    raw_entries.push(RawEntry {
                                        name,
                                        description: desc,
                                        triggers: trigs,
                                        file_path: Some(path),
                                        is_builtin: false,
                                    });
                                } else if let Some(sname) = file_stem {
                                    let first_line = header.lines().find(|l| !l.trim().is_empty()).unwrap_or(sname);
                                    let desc = first_line.trim().trim_start_matches('#').trim().to_string();
                                    let trigs = extract_triggers_from_text(sname, &desc, &[]);
                                    raw_entries.push(RawEntry {
                                        name: sname.to_string(),
                                        description: desc,
                                        triggers: trigs,
                                        file_path: Some(path),
                                        is_builtin: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        let num_docs = raw_entries.len().max(1) as f32;

        // 3. Build Vocabulary & Document Frequencies
        let mut doc_term_counts: Vec<HashMap<String, (usize, usize)>> = Vec::with_capacity(raw_entries.len());
        let mut df: HashMap<String, usize> = HashMap::new();

        for entry in &raw_entries {
            let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
            for token in tokenize(&entry.name) {
                counts.entry(token).or_insert((0, 0)).0 += 1;
            }
            for token in tokenize(&entry.description) {
                counts.entry(token).or_insert((0, 0)).1 += 1;
            }
            for term in counts.keys() {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
            doc_term_counts.push(counts);
        }

        // Sort terms for deterministic IDs
        let mut sorted_vocab: Vec<String> = df.keys().cloned().collect();
        sorted_vocab.sort();

        let mut vocab = HashMap::new();
        let mut idf = Vec::with_capacity(sorted_vocab.len());

        for (idx, term) in sorted_vocab.into_iter().enumerate() {
            let doc_freq = df.get(&term).copied().unwrap_or(1) as f32;
            // Smoothed BM25-style IDF: ln(1.0 + (N - df + 0.5) / (df + 0.5)) + 1.0
            let term_idf = ((num_docs - doc_freq + 0.5) / (doc_freq + 0.5)).ln_1p() + 1.0;
            vocab.insert(term, idx as u32);
            idf.push(term_idf);
        }

        // 4. Build SkillMetadata, Inverted Index & Name/Trigger Indices
        let mut skills = Vec::with_capacity(raw_entries.len());
        let mut inverted_index: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut trigger_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut name_index: HashMap<String, usize> = HashMap::new();

        for (id, (entry, counts)) in raw_entries.into_iter().zip(doc_term_counts).enumerate() {
            let domain = infer_domain(&entry.name);
            let name_tokens = tokenize(&entry.name);

            // Calculate TF-IDF vector with field boosting (Name: 3.0x, Desc: 1.0x)
            let mut sparse_vec: Vec<(u32, f32)> = Vec::new();
            let mut norm_sq = 0.0f32;

            for (term, (name_count, desc_count)) in counts {
                if let Some(&tid) = vocab.get(&term) {
                    let weighted_count = (name_count as f32 * 3.0) + (desc_count as f32 * 1.0);
                    if weighted_count > 0.0 {
                        let tf = 1.0 + weighted_count.ln();
                        let weight = tf * idf[tid as usize];
                        sparse_vec.push((tid, weight));
                        norm_sq += weight * weight;
                    }
                }
            }

            // Sort sparse vector by term_id for fast two-pointer dot product
            sparse_vec.sort_by_key(|&(tid, _)| tid);

            let norm = norm_sq.sqrt();
            let normalized_vec: Vec<(u32, f32)> = if norm > 1e-6 {
                sparse_vec.into_iter().map(|(tid, w)| (tid, w / norm)).collect()
            } else {
                sparse_vec
            };

            for &(tid, _) in &normalized_vec {
                inverted_index.entry(tid).or_default().push(id);
            }

            // Index triggers
            for tr in &entry.triggers {
                let lower_tr = tr.to_lowercase();
                trigger_index.entry(lower_tr).or_default().push(id);
            }

            // Index name
            let lower_name = entry.name.to_lowercase();
            name_index.insert(lower_name.clone(), id);
            let spaced = lower_name.replace('-', " ");
            if spaced != lower_name {
                name_index.insert(spaced, id);
            }

            skills.push(SkillMetadata {
                id,
                name: entry.name,
                domain,
                description: entry.description,
                triggers: entry.triggers,
                file_path: entry.file_path,
                is_builtin: entry.is_builtin,
                tfidf_vector: normalized_vec,
                norm,
                name_tokens,
            });
        }

        Self {
            skills,
            inverted_index,
            trigger_index,
            name_index,
            vocab,
            idf,
            skill_cache: RwLock::new(initial_cache),
        }
    }

    /// Retrieve total number of indexed skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Retrieve read-only slice of all indexed skills
    pub fn skills(&self) -> &[SkillMetadata] {
        &self.skills
    }

    /// Retrieve single skill by exact or normalized name
    pub fn get_skill(&self, name: &str) -> Option<EccSkill> {
        let lower = name.to_lowercase().replace('_', "-");
        if let Some(&doc_id) = self.name_index.get(&lower) {
            return self.load_skill_by_id(doc_id);
        }

        if lower == "rust-tokio-concurrency" || lower == "rust-concurrency" || lower == "tokio-concurrency" {
            if let Some(&doc_id) = self.name_index.get("tokio-async-tuning") {
                if let Some(s) = self.load_skill_by_id(doc_id) {
                    return Some(s);
                }
            }
        }

        // Try prefixes
        for prefix in ["oracle-", "oci-", "azure-", "aws-", "ibm-", "sap-", "alibaba-", "sf-"] {
            let stripped = lower.trim_start_matches(prefix);
            if let Some(&doc_id) = self.name_index.get(stripped) {
                return self.load_skill_by_id(doc_id);
            }
        }

        find_built_in_skill(name)
    }

    /// Load full EccSkill body lazily from cache or disk
    pub fn load_skill_by_id(&self, doc_id: usize) -> Option<EccSkill> {
        if doc_id >= self.skills.len() {
            return None;
        }

        // 1. Check read lock
        {
            if let Ok(cache) = self.skill_cache.read() {
                if let Some(skill) = cache.get(&doc_id) {
                    return Some(skill.clone());
                }
            }
        }

        // 2. Read from disk
        let meta = &self.skills[doc_id];
        let skill = if let Some(ref path) = meta.file_path {
            EccSkill::from_file(path).unwrap_or_else(|_| {
                EccSkill::new(&meta.name, &meta.description, &meta.description)
            })
        } else {
            find_built_in_skill(&meta.name).unwrap_or_else(|| {
                EccSkill::new(&meta.name, &meta.description, &meta.description)
            })
        };

        // 3. Populate write lock
        if let Ok(mut cache) = self.skill_cache.write() {
            cache.insert(doc_id, skill.clone());
        }

        Some(skill)
    }

    /// Dynamically register an external or plugin skill into the in-memory index
    pub fn register_skill(&mut self, skill: EccSkill) {
        let id = self.skills.len();
        let domain = infer_domain(&skill.name);
        let trigs = extract_triggers_from_text(&skill.name, &skill.description, &[]);
        let name_tokens = tokenize(&skill.name);

        let lower_name = skill.name.to_lowercase();
        self.name_index.insert(lower_name.clone(), id);
        let spaced = lower_name.replace('-', " ");
        if spaced != lower_name {
            self.name_index.insert(spaced, id);
        }

        for tr in &trigs {
            self.trigger_index.entry(tr.to_lowercase()).or_default().push(id);
        }

        if let Ok(mut cache) = self.skill_cache.write() {
            cache.insert(id, skill.clone());
        }

        self.skills.push(SkillMetadata {
            id,
            name: skill.name,
            domain,
            description: skill.description,
            triggers: trigs,
            file_path: None,
            is_builtin: false,
            tfidf_vector: Vec::new(),
            norm: 0.0,
            name_tokens,
        });
    }

    /// Rank and dispatch top-K skills matching query in < 0.5ms
    pub fn dispatch(
        &self,
        query: &str,
        top_k: usize,
        domain_bias: Option<&str>,
    ) -> Vec<DispatchedSkill> {
        if self.skills.is_empty() || top_k == 0 {
            return Vec::new();
        }

        let lower_query = query.to_lowercase();
        let query_tokens = tokenize(query);

        // 1. Gather candidate skills via Inverted Index and Trigger Index
        let mut candidates = HashSet::new();

        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                if let Some(doc_ids) = self.inverted_index.get(&tid) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        // 2. Fast query n-grams lookup in trigger_index and name_index (1-gram to 5-gram)
        let words: Vec<&str> = lower_query.split_whitespace().collect();
        let max_n = 5.min(words.len());
        for n in 1..=max_n {
            for window in words.windows(n) {
                let phrase = window.join(" ");
                if let Some(doc_ids) = self.trigger_index.get(&phrase) {
                    candidates.extend(doc_ids.iter().copied());
                }
                if let Some(&doc_id) = self.name_index.get(&phrase) {
                    candidates.insert(doc_id);
                }
                let hyphenated = window.join("-");
                if let Some(&doc_id) = self.name_index.get(&hyphenated) {
                    candidates.insert(doc_id);
                }
                if let Some(doc_ids) = self.trigger_index.get(&hyphenated) {
                    candidates.extend(doc_ids.iter().copied());
                }
            }
        }

        // Fallback: if candidates empty, score all documents
        let candidate_list: Vec<usize> = if candidates.is_empty() {
            (0..self.skills.len()).collect()
        } else {
            candidates.into_iter().collect()
        };

        // 2. Compute query sparse TF-IDF vector
        let mut q_counts: HashMap<u32, usize> = HashMap::new();
        for token in &query_tokens {
            if let Some(&tid) = self.vocab.get(token) {
                *q_counts.entry(tid).or_insert(0) += 1;
            }
        }

        let mut query_vec: Vec<(u32, f32)> = Vec::new();
        let mut q_norm_sq = 0.0f32;

        for (tid, count) in q_counts {
            let tf = 1.0 + (count as f32).ln();
            let weight = tf * self.idf[tid as usize];
            query_vec.push((tid, weight));
            q_norm_sq += weight * weight;
        }

        query_vec.sort_by_key(|&(tid, _)| tid);

        let q_norm = q_norm_sq.sqrt();
        let normalized_q_vec: Vec<(u32, f32)> = if q_norm > 1e-6 {
            query_vec.into_iter().map(|(tid, w)| (tid, w / q_norm)).collect()
        } else {
            query_vec
        };

        // 3. Multi-factor scoring loop
        let mut ranked: Vec<(usize, f32, Vec<String>)> = Vec::with_capacity(candidate_list.len());

        for doc_id in candidate_list {
            let meta = &self.skills[doc_id];
            let mut score = 0.0f32;
            let mut matched_triggers = Vec::new();

            // 3a. Exact Name Match (+200.0)
            let lower_name = meta.name.to_lowercase();
            let spaced_name = lower_name.replace('-', " ");
            if lower_query.contains(&lower_name) || lower_query.contains(&spaced_name) {
                score += 200.0;
            }

            // 3b. Trigger Matches (+100 for first, +25 subsequent, max +150)
            let mut trigger_score = 0.0f32;
            for tr in &meta.triggers {
                if tr.len() >= 3 && lower_query.contains(tr.as_str()) {
                    if trigger_score == 0.0 {
                        trigger_score += 100.0;
                    } else if trigger_score < 150.0 {
                        trigger_score = (trigger_score + 25.0).min(150.0);
                    }
                    matched_triggers.push(tr.clone());
                }
            }
            score += trigger_score;

            // 3c. Domain Prefix & Stage Bias Boost
            let mut domain_score = 0.0f32;
            if let Some(bias) = domain_bias {
                if bias.eq_ignore_ascii_case(&meta.domain) {
                    domain_score += 35.0;
                }
            }
            if lower_query.contains(&meta.domain) || query_tokens.iter().any(|t| t == &meta.domain) {
                domain_score += 25.0;
            }
            score += domain_score.min(60.0);

            // 3d. Name Token Overlap (+30.0)
            if !meta.name_tokens.is_empty() {
                let overlap = meta.name_tokens.iter().filter(|t| query_tokens.contains(t)).count();
                score += (overlap as f32 / meta.name_tokens.len() as f32) * 30.0;
            }

            // 3e. Sublinear TF-IDF Cosine Similarity (+40.0)
            if !normalized_q_vec.is_empty() && !meta.tfidf_vector.is_empty() {
                let mut dot = 0.0f32;
                let mut p_q = 0;
                let mut p_d = 0;
                while p_q < normalized_q_vec.len() && p_d < meta.tfidf_vector.len() {
                    let (q_tid, q_val) = normalized_q_vec[p_q];
                    let (d_tid, d_val) = meta.tfidf_vector[p_d];
                    if q_tid == d_tid {
                        dot += q_val * d_val;
                        p_q += 1;
                        p_d += 1;
                    } else if q_tid < d_tid {
                        p_q += 1;
                    } else {
                        p_d += 1;
                    }
                }
                score += dot * 40.0;
            }

            if score >= 10.0 {
                ranked.push((doc_id, score, matched_triggers));
            }
        }

        // Sort descending by score, tie-break by name
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| self.skills[a.0].name.cmp(&self.skills[b.0].name))
        });

        // Take Top-K
        let mut results = Vec::with_capacity(top_k.min(ranked.len()));
        for (doc_id, score, matched_trigs) in ranked.into_iter().take(top_k) {
            if let Some(skill) = self.load_skill_by_id(doc_id) {
                results.push(DispatchedSkill {
                    domain: self.skills[doc_id].domain.clone(),
                    skill,
                    score,
                    matched_triggers: matched_trigs,
                });
            }
        }

        results
    }

    /// Search and format results as Markdown for LLM prompt injection or tool returns
    pub fn search_and_format(
        &self,
        query: &str,
        limit: usize,
        domain: Option<&str>,
        include_instructions: bool,
    ) -> String {
        let results = self.dispatch(query, limit, domain);
        if results.is_empty() {
            return format!("No engineering skills found matching query: \"{query}\"");
        }

        let mut output = format!("Found {} relevant engineering skills for \"{}\":\n\n", results.len(), query);
        for (idx, d) in results.iter().enumerate() {
            output.push_str(&format!(
                "### {}. {} [Score: {:.1} | Domain: {}]\n{}\n",
                idx + 1,
                d.skill.name,
                d.score,
                d.domain,
                d.skill.description
            ));

            if !d.matched_triggers.is_empty() {
                output.push_str(&format!("Matched Triggers: {:?}\n", d.matched_triggers));
            }

            if include_instructions && !d.skill.instructions.is_empty() {
                output.push_str("\n#### Instructions:\n");
                output.push_str(d.skill.instructions.trim());
                output.push_str("\n");
            }
            output.push_str("\n---\n");
        }

        output.trim_end().to_string()
    }

    /// Check if a provider identifier corresponds to a local LLM runtime (e.g. Ollama, local)
    pub fn is_local_provider(provider: &str) -> bool {
        let p = provider.trim().to_lowercase();
        p == "ollama"
            || p == "local"
            || p.starts_with("ollama")
            || p.starts_with("local")
            || p.contains("ollama")
            || p.contains("local")
            || p.contains("llama")
            || p.contains("vllm")
    }

    /// Format dispatched skills as a condensed Cheat Sheet for local LLMs (<1,000 tokens footprint for 8k context)
    pub fn format_cheat_sheet(skills: &[DispatchedSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = String::from("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]\n");
        for (idx, ds) in skills.iter().enumerate() {
            out.push_str(&format!("\n#### Skill {}: {} ({})\n", idx + 1, ds.skill.name, ds.domain));

            let mut rules: Vec<String> = Vec::new();
            for line in ds.skill.instructions.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with('#') {
                    let h_clean = trimmed.trim_start_matches('#').trim();
                    let h_lower = h_clean.to_lowercase();
                    if !h_clean.is_empty() && !h_lower.contains("overview") && !h_lower.contains("ecc") {
                        rules.push(format!("* [Context: {}]", h_clean));
                    }
                    continue;
                }
                if trimmed.starts_with('-') || trimmed.starts_with('*') {
                    let rule_text = trimmed.trim_start_matches(|c: char| c == '-' || c == '*' || c.is_whitespace());
                    if !rule_text.is_empty() {
                        rules.push(format!("- {}", rule_text));
                    }
                } else if trimmed.as_bytes().first().map_or(false, |b| b.is_ascii_digit()) && trimmed.contains('.') {
                    if let Some((_, rest)) = trimmed.split_once('.') {
                        let rule_text = rest.trim();
                        if !rule_text.is_empty() {
                            rules.push(format!("- {}", rule_text));
                        }
                    }
                } else {
                    let lower = trimmed.to_lowercase();
                    if lower.starts_with("rule:")
                        || lower.starts_with("invariant:")
                        || lower.starts_with("constraint:")
                        || lower.starts_with("always")
                        || lower.starts_with("never")
                        || lower.starts_with("must")
                    {
                        rules.push(format!("- {}", trimmed));
                    }
                }
            }

            if rules.is_empty() {
                for line in ds.skill.instructions.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        rules.push(format!("- {}", trimmed));
                    }
                }
            }

            let capped_rules = rules.into_iter().take(12);
            for r in capped_rules {
                out.push_str(&r);
                out.push('\n');
            }
        }
        out.trim().to_string()
    }

    /// Format dispatched skills as a Comprehensive Architectural Specification mode with full instructions, checklists, invariants
    pub fn format_cloud_guidelines(skills: &[DispatchedSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }
        let mut out = String::from("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]\n");
        for (idx, ds) in skills.iter().enumerate() {
            out.push_str(&format!(
                "\n---\n#### Skill {}: {} [Domain: {} | Match Score: {:.1}]\n",
                idx + 1, ds.skill.name, ds.domain, ds.score
            ));
            out.push_str(&format!("**Description:** {}\n\n", ds.skill.description.trim()));
            if !ds.matched_triggers.is_empty() {
                out.push_str(&format!("**Triggers:** {}\n\n", ds.matched_triggers.join(", ")));
            }
            out.push_str("#### Full Specification & Directives:\n");
            out.push_str(ds.skill.instructions.trim());
            out.push_str("\n");
        }
        out.trim().to_string()
    }

    /// Synthesizes provider-aware prompts injecting either condensed Cheat Sheet (for Local) or Comprehensive Guidelines (for Cloud)
    pub fn equip_prompt_for_provider(
        &self,
        base_prompt: &str,
        query: &str,
        provider: &str,
        explicit_skill: Option<&str>,
    ) -> (String, Vec<DispatchedSkill>) {
        let is_local = Self::is_local_provider(provider);
        let max_skills = if is_local { 2 } else { 4 };

        let skills: Vec<DispatchedSkill> = if let Some(skill_name) = explicit_skill {
            let clean = skill_name.trim();
            if !clean.is_empty() {
                if let Some(skill) = self.get_skill(clean) {
                    vec![DispatchedSkill {
                        domain: infer_domain(&skill.name),
                        skill,
                        score: 100.0,
                        matched_triggers: vec![clean.to_string()],
                    }]
                } else {
                    self.dispatch(clean, max_skills, None)
                }
            } else {
                self.dispatch(query, max_skills, None)
            }
        } else {
            self.dispatch(query, max_skills, None)
        };

        if skills.is_empty() {
            return (base_prompt.to_string(), Vec::new());
        }

        let section = if is_local {
            Self::format_cheat_sheet(&skills)
        } else {
            Self::format_cloud_guidelines(&skills)
        };

        let equipped_prompt = if base_prompt.trim().is_empty() {
            section
        } else {
            format!("{}\n\n{}", base_prompt.trim_end(), section)
        };

        (equipped_prompt, skills)
    }

    /// Auto-equips skills into a system prompt string
    pub fn equip_prompt(&self, base_prompt: &str, query: &str, limit: usize) -> String {
        let dispatched = self.dispatch(query, limit, None);
        if dispatched.is_empty() {
            return base_prompt.to_string();
        }
        let section = Self::format_cheat_sheet(&dispatched);
        if base_prompt.trim().is_empty() {
            section
        } else {
            format!("{}\n\n{}", base_prompt.trim_end(), section)
        }
    }
}

/// 31. Grokking Algorithms Skill (Aditya Bhargava)
pub fn grokking_algorithms() -> EccSkill {
    EccSkill::new(
        "grokking-algorithms",
        "Grokking Algorithms (Aditya Bhargava): Big-O complexity radar, spatial vs temporal trade-offs, graph search, greedy heuristics, dynamic programming memoization, hash table collision avoidance, eliminating O(N^2) loops. Triggers: grokking algorithms, big-o, complexity radar, spatial vs temporal, dynamic programming, memoization, hash table, nested loops, bfs, dfs, algorithmic optimization.",
        r#"# Grokking Algorithms & Complexity Control (Aditya Bhargava)

## 1. Big-O Complexity Radar
- Eliminate O(N^2) Anti-patterns: Never use nested loops over unindexed arrays if a hash map/set can reduce lookup to O(1).
- Spatial vs Temporal Trade-offs: Trade memory (caching, hash tables) for speed (CPU cycles) where appropriate.

## 2. Graph Search & Heuristics
- BFS vs DFS: Use BFS for shortest path/minimum edges. Use DFS for exhaustive search and backtracking.
- Greedy Heuristics: Use for optimization problems where a local optimum leads to a global optimum (e.g., fractional knapsack, set cover).

## 3. Dynamic Programming & Hashing
- DP Memoization: Cache subproblem results to avoid exponential recursion trees.
- Hash Table Collision Avoidance: Understand load factors and choose prime modulus sizes or good hash functions (e.g. SipHash) to prevent clustering.
"#,
    )
}

/// 32. OSTEP Mechanical Sympathy Skill (Arpaci-Dusseau & Petzold)
pub fn ostep_mechanical_sympathy() -> EccSkill {
    EccSkill::new(
        "ostep-mechanical-sympathy",
        "Operating Systems & Mechanical Sympathy (Remzi Arpaci-Dusseau & Charles Petzold): Virtualization, Concurrency, Persistence, epoll/kqueue async loops, cache locality. Triggers: ostep, mechanical sympathy, operating systems, cache locality, stack vs heap, concurrency, atomic operations, write-ahead logging, fsync, persistence, epoll, kqueue.",
        r#"# Mechanical Sympathy & Operating Systems (Arpaci-Dusseau & Petzold)

## 1. CPU & Memory Virtualization
- Cache Locality: Prefer contiguous memory structures (e.g. `Vec`, flat arrays) over pointer-chasing data structures (Linked Lists) to maximize L1/L2 cache hits.
- Stack vs Heap: Keep allocations on the stack where possible. Minimize heap fragmentation.

## 2. Concurrency Primitives
- Data Races & Deadlocks: Acquire multiple locks in a strict global order. Prefer message passing over shared memory if possible.
- Atomic Operations: Use lock-free atomics (Compare-and-Swap) for shared counters instead of heavy Mutexes.
- Async Event Loops: Understand epoll/kqueue fundamentals underlying Tokio/Node.js; never block the reactor thread.

## 3. Persistence & Storage
- fsync & Write-Ahead Logging: Use explicit fsync for durability guarantees. Keep append-only logs to minimize SSD page amplification and random writes.
"#,
    )
}

/// 33. System Design Building Blocks Skill (Alex Xu & Sahn Lam)
pub fn system_design_building_blocks() -> EccSkill {
    EccSkill::new(
        "system-design-building-blocks",
        "System Design Interview (Alex Xu): Caching topologies, Rate limiting, Distributed message queues, Sharding, Idempotency keys. Triggers: system design, caching topologies, cache aside, write through, thundering herd, rate limiting, token bucket, sliding window, backpressure, sharding, idempotency keys, message queues.",
        r#"# System Design Building Blocks (Alex Xu & Sahn Lam)

## 1. Caching Topologies
- Cache-Aside vs Write-Through: Choose appropriately. Use Cache-Aside for read-heavy workloads.
- Thundering Herd Prevention: Use probabilistic early expiration or single-flight request coalescing to prevent database stampedes on cache misses.

## 2. Rate Limiting & Flow Control
- Token Bucket & Sliding Window: Implement rate limiters at the API gateway to prevent resource exhaustion and abuse.
- Backpressure: Message queues must exert backpressure on publishers when consumer groups fall behind.

## 3. Distributed Data
- Sharding & Read Replicas: Distribute read pressure via replicas and partition data horizontally via consistent hashing.
- Idempotency Keys: Required for all mutating requests (e.g. POST, PUT) to survive network retries and duplicate deliveries safely.
"#,
    )
}

/// 34. The Pragmatic Programmer Skill (David Thomas & Andrew Hunt)
pub fn pragmatic_programmer_craft() -> EccSkill {
    EccSkill::new(
        "pragmatic-programmer-craft",
        "The Pragmatic Programmer (Thomas & Hunt): Tracer bullets vs prototypes, Orthogonality, Broken windows theory, DRY, Design by Contract, Crash early. Triggers: pragmatic programmer, tracer bullets, prototypes, orthogonality, broken windows, dry, design by contract, crash early, dead programs tell no lies.",
        r#"# Pragmatic Programmer Craft (David Thomas & Andrew Hunt)

## 1. Prototyping & Orthogonality
- Tracer Bullets vs Prototypes: Tracer bullets are structural skeletons intended for production. Prototypes are disposable learning exercises.
- Orthogonality: Changes in one module should not ripple into others. Design independent, decoupled components.

## 2. Code Quality & Broken Windows
- Broken Windows Theory: Fix small issues, warnings, and code smells immediately before they normalize entropy in the codebase.
- DRY across Models: Don't Repeat Yourself applies to data models, documentation, and configuration, not just code.

## 3. Resilience & Defense
- Crash Early / Dead Programs Tell No Lies: Panic/Abort immediately on impossible states or contract violations instead of limping along with corrupted state.
"#,
    )
}

/// 35. Site Reliability Engineering Skill (Betsy Beyer / Google SRE)
pub fn site_reliability_engineering() -> EccSkill {
    EccSkill::new(
        "site-reliability-engineering",
        "Google SRE (Betsy Beyer et al.): SLIs/SLOs/SLAs, Error budgets, Circuit breakers, Exponential backoff with full jitter, Graceful degradation, Distributed tracing. Triggers: site reliability engineering, sre, sli, slo, error budgets, circuit breakers, exponential backoff, jitter, graceful degradation, distributed tracing, structured logging.",
        r#"# Site Reliability Engineering (Google SRE)

## 1. Service Level Indicators & Error Budgets
- SLIs/SLOs: Define strict, measurable service level objectives (e.g., 99.9% of requests < 200ms).
- Error Budgets: Freeze feature deployments when the error budget is exhausted; redirect engineering to reliability.

## 2. Resilience Mechanisms
- Circuit Breakers: Fast-fail downstream calls when a dependency degrades to prevent cascading thread pool exhaustion.
- Exponential Backoff with Jitter: Always add randomization (jitter) to retry loops to avoid thundering herd synchronized retries.
- Graceful Degradation: Serve cached, stale, or partial data when non-critical backends fail.

## 3. Observability
- Distributed Tracing & Correlation IDs: Inject and propagate trace IDs (e.g. W3C Trace Context) across all network boundaries and asynchronous queues.
- Structured Logging: Log payloads as JSON (not flat text) with strict schemas for automated indexing and alerting.
"#,
    )
}

// -------------------------------------------------------------------------
// Helper Parsing & Lexical Functions
// -------------------------------------------------------------------------

/// Fast frontmatter metadata parser extracting (name, description, triggers) without reading entire file
fn parse_frontmatter_metadata(
    content: &str,
    default_name: Option<&str>,
) -> Option<(String, String, Vec<String>)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = &trimmed[3..];
    let end_idx = rest.find("\n---").or_else(|| rest.find("\r\n---"))?;
    let frontmatter_str = &rest[..end_idx];

    let mut name = String::new();
    let mut description = String::new();
    let mut triggers = Vec::new();

    let lines: Vec<&str> = frontmatter_str.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let raw_line = lines[i];
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let val = val.trim().trim_matches('"').trim_matches('\'').trim();

            match key.as_str() {
                "name" => {
                    if name.is_empty() {
                        name = val.to_string();
                    }
                }
                "description" => {
                    if description.is_empty() {
                        if val == "|" || val == "|-" || val == ">" || val == ">-" || val.is_empty() {
                            let mut desc_lines = Vec::new();
                            i += 1;
                            while i < lines.len() {
                                let next_raw = lines[i];
                                if next_raw.starts_with(' ') || next_raw.starts_with('\t') {
                                    desc_lines.push(next_raw.trim());
                                    i += 1;
                                } else if next_raw.trim().is_empty() {
                                    i += 1;
                                } else {
                                    break;
                                }
                            }
                            description = desc_lines.join(" ");
                            continue;
                        } else {
                            description = val.to_string();
                        }
                    }
                }
                "triggers" => {
                    if val.starts_with('[') && val.ends_with(']') {
                        let inner = &val[1..val.len() - 1];
                        for item in inner.split(',') {
                            let cleaned = item.trim().trim_matches('"').trim_matches('\'').trim();
                            if !cleaned.is_empty() {
                                triggers.push(cleaned.to_lowercase());
                            }
                        }
                    } else if val.is_empty() {
                        i += 1;
                        while i < lines.len() {
                            let next_raw = lines[i];
                            let next_trim = next_raw.trim();
                            if next_trim.starts_with('-') {
                                let item = next_trim.trim_start_matches('-').trim().trim_matches('"').trim_matches('\'').trim();
                                if !item.is_empty() {
                                    triggers.push(item.to_lowercase());
                                }
                                i += 1;
                            } else if next_trim.is_empty() {
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        continue;
                    } else {
                        triggers.push(val.to_lowercase());
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    if name.is_empty() {
        if let Some(def) = default_name {
            name = def.to_string();
        } else {
            return None;
        }
    }

    Some((name, description, triggers))
}

/// Extract candidate triggers from skill name, description, and declared triggers
pub fn extract_triggers_from_text(name: &str, description: &str, explicit_triggers: &[String]) -> Vec<String> {
    let mut triggers = Vec::new();
    let lower_name = name.to_lowercase();
    triggers.push(lower_name.clone());

    let spaced_name = lower_name.replace('-', " ");
    if spaced_name != lower_name {
        triggers.push(spaced_name);
    }
    let underscored_name = lower_name.replace('_', " ");
    if underscored_name != lower_name && !triggers.contains(&underscored_name) {
        triggers.push(underscored_name);
    }

    for t in explicit_triggers {
        let clean = t.trim().to_lowercase();
        if !clean.is_empty() && !triggers.contains(&clean) {
            triggers.push(clean);
        }
    }

    let lower_desc = description.to_lowercase();
    if let Some(pos) = lower_desc.find("triggers:") {
        let trigger_part = &description[pos + 9..];
        for token in trigger_part.split(&['"', '\'', ','][..]) {
            let clean = token.trim().trim_matches('.').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 60 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean);
            }
        }
    }
    if let Some(pos) = lower_desc.find("keywords:") {
        let kw_part = &description[pos + 9..];
        for token in kw_part.split(&[',', ';', '.'][..]) {
            let clean = token.trim().trim_matches('"').trim_matches('\'').trim().to_lowercase();
            if clean.len() >= 3 && clean.len() <= 40 && !clean.contains('\n') && !triggers.contains(&clean) {
                triggers.push(clean);
            }
        }
    }

    triggers
}

/// Tokenize alphanumeric text into normalized, filtered tokens
fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else {
            if current.len() >= 2 && !is_stop_word(&current) {
                tokens.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() >= 2 && !is_stop_word(&current) {
        tokens.push(current);
    }
    tokens
}

/// Infer high-level engineering domain from skill identifier
fn infer_domain(name: &str) -> String {
    let lower = name.to_lowercase();
    let prefixes = [
        ("azure", "azure"),
        ("aws", "aws"),
        ("amazon", "aws"),
        ("gcp", "gcp"),
        ("google", "gcp"),
        ("oci", "oracle"),
        ("oracle", "oracle"),
        ("ibm", "ibm"),
        ("openshift", "ibm"),
        ("sap", "sap"),
        ("abap", "sap"),
        ("rap", "sap"),
        ("alibaba", "alibaba"),
        ("aliyun", "alibaba"),
        ("salesforce", "salesforce"),
        ("sf", "salesforce"),
        ("agentforce", "salesforce"),
        ("rust", "rust"),
        ("tokio", "rust"),
        ("axum", "rust"),
        ("ratatui", "rust"),
        ("bun", "bun"),
        ("flutter", "flutter"),
        ("swift", "swift"),
        ("swiftui", "swift"),
        ("ios", "swift"),
        ("react", "react"),
        ("nextjs", "react"),
        ("vue", "vue"),
        ("svelte", "svelte"),
        ("angular", "angular"),
        ("typescript", "typescript"),
        ("ts", "typescript"),
        ("tailwind", "tailwind"),
        ("tdd", "test"),
        ("test", "test"),
        ("testing", "test"),
        ("vitest", "test"),
        ("playwright", "test"),
        ("security", "security"),
        ("sec", "security"),
        ("threat", "security"),
        ("audit", "security"),
        ("sandbox", "security"),
        ("math", "math"),
        ("vibe-math", "math"),
        ("boids", "math"),
        ("quaternion", "math"),
        ("calculus", "math"),
        ("geometry", "math"),
        ("fractal", "math"),
        ("kronos", "quant"),
        ("qlib", "quant"),
        ("chronos", "quant"),
        ("moirai", "quant"),
        ("fingpt", "quant"),
        ("finrl", "quant"),
        ("rdagent", "quant"),
        ("quant", "quant"),
        ("refactoring-ui", "ux"),
        ("microinteractions", "ux"),
        ("laws-of-ux", "ux"),
        ("design-systems", "ux"),
        ("about-face", "ux"),
        ("designing-for-emotion", "ux"),
        ("architect", "architecture"),
        ("architecture", "architecture"),
        ("design", "architecture"),
        ("review", "review"),
        ("clean", "review"),
        ("standards", "review"),
        ("sre", "sre"),
        ("devops", "sre"),
        ("devsecops", "security"),
        ("scrum", "agile"),
    ];
    for (prefix, dom) in prefixes {
        if lower.starts_with(prefix) {
            return dom.to_string();
        }
    }
    if let Some((first, _)) = lower.split_once('-') {
        if first.len() >= 3 {
            return first.to_string();
        }
    }
    "general".to_string()
}

/// Standard lexical stop-words to eliminate uninformative terms from TF-IDF indexing
fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an" | "the" | "and" | "or" | "but" | "if" | "then" | "else" | "when" | "at" | "by"
            | "for" | "with" | "about" | "against" | "between" | "into" | "through" | "during"
            | "before" | "after" | "above" | "below" | "to" | "from" | "up" | "down" | "in"
            | "out" | "on" | "off" | "over" | "under" | "again" | "further" | "once" | "here"
            | "there" | "all" | "any" | "both" | "each" | "few" | "more" | "most" | "other"
            | "some" | "such" | "no" | "nor" | "not" | "only" | "own" | "same" | "so" | "than"
            | "too" | "very" | "can" | "will" | "just" | "don" | "should" | "now" | "use"
            | "using" | "uses" | "used" | "this" | "that" | "these" | "those" | "is" | "are"
            | "was" | "were" | "be" | "been" | "being" | "have" | "has" | "had" | "having"
            | "do" | "does" | "did" | "doing"
    )
}

/// Standalone check if a provider identifier corresponds to a local LLM runtime (e.g. Ollama, local)
pub fn is_local_provider(provider: &str) -> bool {
    SkillDispatcher::is_local_provider(provider)
}

/// Standalone formatter for condensed Cheat Sheet format for local LLMs
pub fn format_cheat_sheet(skills: &[DispatchedSkill]) -> String {
    SkillDispatcher::format_cheat_sheet(skills)
}

/// Standalone formatter for Comprehensive Architectural Specification format for cloud LLMs
pub fn format_cloud_guidelines(skills: &[DispatchedSkill]) -> String {
    SkillDispatcher::format_cloud_guidelines(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local_provider() {
        assert!(is_local_provider("ollama"));
        assert!(is_local_provider("local"));
        assert!(is_local_provider("localhost"));
        assert!(is_local_provider("ollama-remote"));
        assert!(is_local_provider("llama.cpp"));

        assert!(!is_local_provider("anthropic"));
        assert!(!is_local_provider("openai"));
        assert!(!is_local_provider("gemini"));
        assert!(!is_local_provider("deepseek"));
        assert!(!is_local_provider("grok"));
        assert!(!is_local_provider("xai"));
    }

    #[test]
    fn test_equip_prompt_for_provider_formats() {
        let dispatcher = global_dispatcher();
        let query = "tokio async concurrency channel deadlock";

        // Local Ollama
        let (local_text, local_skills) = dispatcher.equip_prompt_for_provider(
            "Base prompt",
            query,
            "ollama",
            None,
        );
        assert!(!local_skills.is_empty());
        assert!(local_skills.len() <= 2);
        assert!(local_text.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
        assert!(!local_text.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS"));

        // Cloud Anthropic
        let (cloud_text, cloud_skills) = dispatcher.equip_prompt_for_provider(
            "Base prompt",
            query,
            "anthropic",
            None,
        );
        assert!(!cloud_skills.is_empty());
        assert!(cloud_skills.len() <= 4);
        assert!(cloud_text.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"));
        assert!(cloud_text.contains("Full Specification & Directives:"));
        assert!(!cloud_text.contains("[LOCAL LLM CHEAT SHEET"));
    }

    #[test]
    fn test_explicit_skill_alias_override() {
        let dispatcher = global_dispatcher();

        let (text, skills) = dispatcher.equip_prompt_for_provider(
            "Base",
            "irrelevant",
            "ollama",
            Some("rust-tokio-concurrency"),
        );
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill.name, "tokio-async-tuning");
        assert!(text.contains("tokio-async-tuning"));
        assert!(text.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
    }

    #[test]
    fn test_vibe_cs_books_skills_registered_and_dispatchable() {
        let skills = [
            "grokking-algorithms",
            "ostep-mechanical-sympathy",
            "system-design-building-blocks",
            "pragmatic-programmer-craft",
            "site-reliability-engineering",
        ];

        for skill_name in skills {
            let found = find_built_in_skill(skill_name);
            assert!(found.is_some(), "Skill '{}' should be registered in built-in skills", skill_name);
            let skill = found.unwrap();
            assert!(!skill.description.is_empty());
            assert!(!skill.instructions.is_empty());
        }

        let dispatcher = global_dispatcher();
        let dispatched = dispatcher.dispatch("eliminate O(N^2) nested loop with hash map and memoization", 3, None);
        assert!(!dispatched.is_empty());
        assert!(dispatched.iter().any(|d| d.skill.name == "grokking-algorithms"));

        let dispatched_sys = dispatcher.dispatch("caching topologies thundering herd rate limiting token bucket", 3, None);
        assert!(!dispatched_sys.is_empty());
        assert!(dispatched_sys.iter().any(|d| d.skill.name == "system-design-building-blocks"));
    }
}


