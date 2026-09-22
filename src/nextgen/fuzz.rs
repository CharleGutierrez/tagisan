//! # Synthetic Environment Synthesis & Differential Fuzzing (`tgs nextgen fuzz`)
//!
//! Provides property-based differential fuzzing for agent tool arguments,
//! code generation invariants, and LLM structured outputs.
//!
//! Mutation Strategies:
//! - Boundary Numbers (integer extrema, floating-point subnormals, NaN/Inf)
//! - Unicode Homoglyphs & Control Characters (Cyrillic lookalikes, ZWJ/ZWSP, RTL overrides)
//! - Injection Payloads (SQLi, shell expansion, format strings, path traversal)
//! - Concurrent Collisions (timestamp race conditions, monotonic ID duplicates)

use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Invariants that must hold across all execution paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyInvariant {
    /// Numeric output must stay within bounds [min, max]
    BoundsSafety { min: i64, max: i64 },
    /// Output string must not leak format string tokens (%x, %p, %s, %n)
    NoFormatStringVulnerability,
    /// Output or executed query must not contain unescaped SQL injection patterns
    NoSqlInjectionExploit,
    /// Byte sequences must be strictly valid UTF-8 without broken surrogate pairs
    Utf8Integrity,
    /// Execution must not result in an error or panic state
    NoCrashOrPanic,
    /// Output must be non-empty
    NonEmptyOutput,
    /// Custom named invariant
    Custom { name: String },
}

/// Mutation strategies for synthetic fuzz generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationStrategy {
    /// Integer extrema, float subnormals, NaN, Inf, off-by-one
    BoundaryNumbers,
    /// Cyrillic lookalikes, zero-width spaces, RTL overrides, emojis
    UnicodeHomoglyphs,
    /// SQLi, format string, shell injection, path traversal
    InjectionPayloads,
    /// Race collision payloads, duplicate monotonic IDs, timestamp sync
    ConcurrentCollisions,
    /// Composite strategy interleaving all mutations
    Composite,
}

impl MutationStrategy {
    pub fn name(&self) -> &'static str {
        match self {
            Self::BoundaryNumbers => "BoundaryNumbers",
            Self::UnicodeHomoglyphs => "UnicodeHomoglyphs",
            Self::InjectionPayloads => "InjectionPayloads",
            Self::ConcurrentCollisions => "ConcurrentCollisions",
            Self::Composite => "Composite",
        }
    }
}

/// Generated input for a fuzz target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FuzzInput {
    pub raw_string: String,
    pub numeric_val: Option<i64>,
    pub strategy_used: String,
    pub iteration: usize,
}

/// Output captured from fuzz target execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FuzzTargetOutput {
    pub output_str: String,
    pub numeric_output: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_micros: u64,
}

impl FuzzTargetOutput {
    pub fn success(output_str: impl Into<String>) -> Self {
        Self {
            output_str: output_str.into(),
            numeric_output: None,
            success: true,
            error_message: None,
            duration_micros: 0,
        }
    }

    pub fn success_with_num(output_str: impl Into<String>, num: i64) -> Self {
        Self {
            output_str: output_str.into(),
            numeric_output: Some(num),
            success: true,
            error_message: None,
            duration_micros: 0,
        }
    }

    pub fn failure(err: impl Into<String>) -> Self {
        let err_str = err.into();
        Self {
            output_str: String::new(),
            numeric_output: None,
            success: false,
            error_message: Some(err_str),
            duration_micros: 0,
        }
    }
}

/// Detailed counterexample capturing an invariant violation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CounterExample {
    pub iteration: usize,
    pub strategy: String,
    pub failing_input: String,
    pub violated_invariant: String,
    pub error_trace: String,
    pub minimal_reproduced_input: String,
}

/// Comprehensive report produced by AgenticFuzzEngine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FuzzReport {
    pub total_iterations: usize,
    pub passed_iterations: usize,
    pub violations_found: usize,
    pub counterexamples: Vec<CounterExample>,
    pub duration_ms: u64,
    pub strategies_tested: Vec<String>,
}

impl FuzzReport {
    pub fn has_violations(&self) -> bool {
        self.violations_found > 0
    }
}

/// Synthetic Environment Synthesis and Differential Fuzzing Engine
#[derive(Debug, Clone)]
pub struct AgenticFuzzEngine {
    pub strategies: Vec<MutationStrategy>,
    pub invariants: Vec<PropertyInvariant>,
    pub max_iterations: usize,
    pub shrink_counterexamples: bool,
}

impl Default for AgenticFuzzEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AgenticFuzzEngine {
    pub fn new() -> Self {
        Self {
            strategies: vec![MutationStrategy::Composite],
            invariants: vec![
                PropertyInvariant::NoCrashOrPanic,
                PropertyInvariant::Utf8Integrity,
            ],
            max_iterations: 100,
            shrink_counterexamples: true,
        }
    }

    pub fn with_strategy(mut self, strategy: MutationStrategy) -> Self {
        if !self.strategies.contains(&strategy) {
            self.strategies.push(strategy);
        }
        self
    }

    pub fn with_invariant(mut self, invariant: PropertyInvariant) -> Self {
        self.invariants.push(invariant);
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Generate static corpus for boundary number mutations
    fn generate_boundary_numbers() -> Vec<(String, Option<i64>)> {
        vec![
            ("0".to_string(), Some(0)),
            ("1".to_string(), Some(1)),
            ("-1".to_string(), Some(-1)),
            (i64::MIN.to_string(), Some(i64::MIN)),
            (i64::MAX.to_string(), Some(i64::MAX)),
            ((i32::MAX as i64).to_string(), Some(i32::MAX as i64)),
            ((i32::MIN as i64).to_string(), Some(i32::MIN as i64)),
            ("9223372036854775808".to_string(), None), // i64::MAX + 1
            ("-9223372036854775809".to_string(), None), // i64::MIN - 1
            ("NaN".to_string(), None),
            ("Infinity".to_string(), None),
            ("-Infinity".to_string(), None),
            ("1e308".to_string(), None),
            ("1e-308".to_string(), None),
            ("0.0000000000000001".to_string(), None),
            ("-0".to_string(), Some(0)),
        ]
    }

    /// Generate static corpus for unicode homoglyphs & stealth control chars
    fn generate_unicode_homoglyphs() -> Vec<String> {
        vec![
            "аdmin".to_string(),                 // Cyrillic 'а' (U+0430) lookalike for Latin 'a'
            "pаsswоrd".to_string(),              // Cyrillic 'а' and 'о'
            "tagisan\u{200B}agent".to_string(),  // Zero-Width Space
            "tagisan\u{200C}agent".to_string(),  // Zero-Width Non-Joiner
            "tagisan\u{200D}agent".to_string(),  // Zero-Width Joiner
            "tagisan\u{FEFF}agent".to_string(),  // Byte Order Mark
            "agent\u{202E}txt.exe".to_string(),  // RTL Override spoofing extension
            "👨‍👩‍👧‍👦".to_string(),                  // Multi-byte emoji family cluster
            "Z̵a̶l̷g̸o̶_̶t̶e̵x̷t̸".to_string(),             // Combining diacritics
            "null\u{0000}byte".to_string(),       // Embedded null byte
            "\u{FFFF}".to_string(),              // Unassigned non-character
        ]
    }

    /// Generate static corpus for injection payloads
    fn generate_injections() -> Vec<String> {
        vec![
            "' OR '1'='1".to_string(),
            "1; DROP TABLE agents; --".to_string(),
            "' UNION SELECT null, username, password FROM users --".to_string(),
            "admin' --".to_string(),
            "%s%s%s%s%s%s%s%s%s%s".to_string(),
            "%x%x%x%x%n".to_string(),
            "%p%p%p%p".to_string(),
            "$(id)".to_string(),
            "; cat /etc/passwd;".to_string(),
            "`whoami`".to_string(),
            "| uname -a".to_string(),
            "../../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\win.ini".to_string(),
            "<script>alert('xss')</script>".to_string(),
            "{\"__proto__\": {\"admin\": true}}".to_string(),
        ]
    }

    /// Generate static corpus for concurrent collisions
    fn generate_concurrent_collisions() -> Vec<String> {
        vec![
            "TX-COLLISION-00000000-0000-0000-0000-000000000000".to_string(),
            "TX-DUP-MONOTONIC-SEQ-42".to_string(),
            "RACE-INTERLEAVED-THREAD-1-LOCK".to_string(),
            "SIMULTANEOUS-TIMESTAMP-EPOCH-0".to_string(),
            "OUT-OF-ORDER-ACK-FRAME-999".to_string(),
        ]
    }

    /// Generate synthetic corpus of fuzz inputs
    pub fn generate_corpus(&self, count: usize) -> Vec<FuzzInput> {
        let mut corpus = Vec::new();
        let boundaries = Self::generate_boundary_numbers();
        let homoglyphs = Self::generate_unicode_homoglyphs();
        let injections = Self::generate_injections();
        let collisions = Self::generate_concurrent_collisions();

        let mut b_idx = 0;
        let mut h_idx = 0;
        let mut i_idx = 0;
        let mut c_idx = 0;

        for iteration in 0..count {
            let strat = self.strategies[iteration % self.strategies.len()];
            let (raw_string, numeric_val, strat_name) = match strat {
                MutationStrategy::BoundaryNumbers => {
                    let item = &boundaries[b_idx % boundaries.len()];
                    b_idx += 1;
                    (item.0.clone(), item.1, "BoundaryNumbers")
                }
                MutationStrategy::UnicodeHomoglyphs => {
                    let item = &homoglyphs[h_idx % homoglyphs.len()];
                    h_idx += 1;
                    (item.clone(), None, "UnicodeHomoglyphs")
                }
                MutationStrategy::InjectionPayloads => {
                    let item = &injections[i_idx % injections.len()];
                    i_idx += 1;
                    (item.clone(), None, "InjectionPayloads")
                }
                MutationStrategy::ConcurrentCollisions => {
                    let item = &collisions[c_idx % collisions.len()];
                    c_idx += 1;
                    (item.clone(), None, "ConcurrentCollisions")
                }
                MutationStrategy::Composite => {
                    let pick = iteration % 4;
                    match pick {
                        0 => {
                            let item = &boundaries[b_idx % boundaries.len()];
                            b_idx += 1;
                            (item.0.clone(), item.1, "Composite:BoundaryNumbers")
                        }
                        1 => {
                            let item = &homoglyphs[h_idx % homoglyphs.len()];
                            h_idx += 1;
                            (item.clone(), None, "Composite:UnicodeHomoglyphs")
                        }
                        2 => {
                            let item = &injections[i_idx % injections.len()];
                            i_idx += 1;
                            (item.clone(), None, "Composite:InjectionPayloads")
                        }
                        _ => {
                            let item = &collisions[c_idx % collisions.len()];
                            c_idx += 1;
                            (item.clone(), None, "Composite:ConcurrentCollisions")
                        }
                    }
                }
            };

            corpus.push(FuzzInput {
                raw_string,
                numeric_val,
                strategy_used: strat_name.to_string(),
                iteration,
            });
        }

        corpus
    }

    /// Assert whether the target output complies with a specific invariant
    pub fn check_single_invariant(
        &self,
        input: &FuzzInput,
        output: &FuzzTargetOutput,
        invariant: &PropertyInvariant,
    ) -> Option<(String, String)> {
        match invariant {
            PropertyInvariant::BoundsSafety { min, max } => {
                if let Some(num) = output.numeric_output {
                    if num < *min || num > *max {
                        return Some((
                            format!("BoundsSafety[{}, {}]", min, max),
                            format!("Value {} outside allowed range [{}, {}]", num, min, max),
                        ));
                    }
                }
            }
            PropertyInvariant::NoCrashOrPanic => {
                if !output.success {
                    return Some((
                        "NoCrashOrPanic".to_string(),
                        output.error_message.clone().unwrap_or_else(|| "Target execution failed".to_string()),
                    ));
                }
            }
            PropertyInvariant::NonEmptyOutput => {
                if output.output_str.trim().is_empty() {
                    return Some((
                        "NonEmptyOutput".to_string(),
                        "Target generated empty output string".to_string(),
                    ));
                }
            }
            PropertyInvariant::Utf8Integrity => {
                // Check if output string contains replacement character indicating corruption
                if output.output_str.contains('\u{FFFD}') {
                    return Some((
                        "Utf8Integrity".to_string(),
                        "Output string contains Unicode replacement character (U+FFFD)".to_string(),
                    ));
                }
            }
            PropertyInvariant::NoFormatStringVulnerability => {
                let suspicious = ["%x", "%p", "%s", "%n", "%999999"];
                for token in suspicious {
                    if output.output_str.contains(token) && input.raw_string.contains(token) {
                        return Some((
                            "NoFormatStringVulnerability".to_string(),
                            format!("Unsanitized format string specifier '{}' reflected in output", token),
                        ));
                    }
                }
            }
            PropertyInvariant::NoSqlInjectionExploit => {
                let raw_lower = output.output_str.to_lowercase();
                if raw_lower.contains("drop table") || raw_lower.contains("or '1'='1") || raw_lower.contains("union select") {
                    return Some((
                        "NoSqlInjectionExploit".to_string(),
                        "SQL injection payload triggered unescaped query execution or reflection".to_string(),
                    ));
                }
            }
            PropertyInvariant::Custom { name } => {
                if !output.success {
                    return Some((
                        format!("CustomInvariant({})", name),
                        output.error_message.clone().unwrap_or_else(|| "Custom invariant failed".to_string()),
                    ));
                }
            }
        }
        None
    }

    /// Assert whether the target output complies with all configured invariants
    pub fn check_invariants(
        &self,
        input: &FuzzInput,
        output: &FuzzTargetOutput,
    ) -> Option<(String, String)> {
        for invariant in &self.invariants {
            if let Some(violation) = self.check_single_invariant(input, output, invariant) {
                return Some(violation);
            }
        }
        None
    }

    /// Binary minimization / Delta Debugging to shrink counterexamples down to minimal reproducer
    pub fn shrink<F>(
        &self,
        failing_input: &str,
        target: &F,
        invariant: &PropertyInvariant,
    ) -> String
    where
        F: Fn(&FuzzInput) -> FuzzTargetOutput,
    {
        if failing_input.len() <= 1 {
            return failing_input.to_string();
        }

        let mut current = failing_input.to_string();

        // 1. Try removing chunks (delta reduction)
        let chunk_sizes = [current.len() / 2, current.len() / 4, 1];
        for &chunk_size in &chunk_sizes {
            if chunk_size == 0 {
                continue;
            }
            let mut i = 0;
            while i + chunk_size <= current.len() {
                // Ensure character boundaries
                if !current.is_char_boundary(i) || !current.is_char_boundary(i + chunk_size) {
                    i += 1;
                    continue;
                }
                let mut candidate = current.clone();
                candidate.replace_range(i..i + chunk_size, "");

                let test_input = FuzzInput {
                    raw_string: candidate.clone(),
                    numeric_val: None,
                    strategy_used: "Shrinker".to_string(),
                    iteration: 0,
                };

                let out = target(&test_input);
                if self.check_single_invariant(&test_input, &out, invariant).is_some() {
                    // Candidate still violates invariant; retain reduced string
                    current = candidate;
                } else {
                    i += chunk_size;
                }
            }
        }

        current
    }

    /// Execute differential fuzzing campaign against a target closure
    pub fn fuzz<F>(&self, target: F) -> FuzzReport
    where
        F: Fn(&FuzzInput) -> FuzzTargetOutput,
    {
        let start_time = Instant::now();
        let corpus = self.generate_corpus(self.max_iterations);
        let mut passed_iterations = 0;
        let mut counterexamples = Vec::new();

        for input in &corpus {
            let target_start = Instant::now();
            let mut output = target(input);
            output.duration_micros = target_start.elapsed().as_micros() as u64;

            if let Some((inv_name, err_trace)) = self.check_invariants(input, &output) {
                let minimal = if self.shrink_counterexamples && !input.raw_string.is_empty() {
                    // Shrink with the first invariant
                    self.shrink(&input.raw_string, &target, &self.invariants[0])
                } else {
                    input.raw_string.clone()
                };

                counterexamples.push(CounterExample {
                    iteration: input.iteration,
                    strategy: input.strategy_used.clone(),
                    failing_input: input.raw_string.clone(),
                    violated_invariant: inv_name,
                    error_trace: err_trace,
                    minimal_reproduced_input: minimal,
                });
            } else {
                passed_iterations += 1;
            }
        }

        let elapsed = start_time.elapsed().as_millis() as u64;
        let strategies_tested = self.strategies.iter().map(|s| s.name().to_string()).collect();

        FuzzReport {
            total_iterations: corpus.len(),
            passed_iterations,
            violations_found: counterexamples.len(),
            counterexamples,
            duration_ms: elapsed,
            strategies_tested,
        }
    }
}
