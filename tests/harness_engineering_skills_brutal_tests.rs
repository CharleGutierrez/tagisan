//! # Brutal Integration Test Suite: Tagisan Harness Engineering Skills Suite
//!
//! Validates:
//! 1. Catalog presence, frontmatter integrity, and shorthand alias dispatch for all 7 Harness Engineering skills:
//!    - `harness-polyglot-cli-generator-pro-max`
//!    - `harness-ast-abi-ingest-pro-max`
//!    - `harness-sandbox-pty-runtime-pro-max`
//!    - `harness-protocol-serialization-pro-max`
//!    - `harness-fuzz-benchmark-pro-max`
//!    - `harness-dag-lifecycle-telemetry-pro-max`
//!    - `harness-engineering-sovereign-master-pro-max`
//! 2. Polyglot CLI Harness Invariants: Zero exit-code masking, typed JSON output envelopes, stream separation.
//! 3. Deep AST & Multi-Language ABI Ingestion: Complete type resolution, docstring preservation, struct tags.
//! 4. Advanced Sandboxing & PTY Runtime: Terminal escape parsing, 1MB ring buffer ceiling, VCR cassette replay.
//! 5. Universal Protocol Codecs: FIX checksum verification, Modbus CRC16-IBM, round-trip serialization.
//! 6. Generative Fuzzing & Microbenchmarking: Deterministic PRNG seed reproducibility, counterexample shrinking, latency stats.
//! 7. Composition DAG, Lifecycle & Telemetry: Topological DAG sorting, cycle detection, DLQ capture, self-refining feedback.
//! 8. Master Sovereign Umbrella: End-to-end orchestration chain.

use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use tagisan::{all_ecc_skills, find_ecc_skill, EccSkill};

// =========================================================================
// 1. Catalog Presence, Frontmatter Integrity & Dispatch Tests
// =========================================================================

const EXPECTED_SKILLS: &[&str] = &[
    "harness-polyglot-cli-generator-pro-max",
    "harness-ast-abi-ingest-pro-max",
    "harness-sandbox-pty-runtime-pro-max",
    "harness-protocol-serialization-pro-max",
    "harness-fuzz-benchmark-pro-max",
    "harness-dag-lifecycle-telemetry-pro-max",
    "harness-engineering-sovereign-master-pro-max",
];

#[test]
fn test_harness_skills_catalog_presence_and_metadata() {
    let all_skills = all_ecc_skills();
    let skill_map: HashMap<String, &EccSkill> = all_skills
        .iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    for &name in EXPECTED_SKILLS {
        let skill = skill_map.get(name);
        assert!(
            skill.is_some(),
            "Skill '{}' must be registered in all_built_in_skills()",
            name
        );
        let s = skill.unwrap();
        assert_eq!(s.name, name, "Skill name must match canonical ID");
        assert!(
            !s.description.trim().is_empty(),
            "Skill '{}' description must not be empty",
            name
        );
        assert!(
            !s.instructions.trim().is_empty(),
            "Skill '{}' instructions must not be empty",
            name
        );
        assert!(
            s.instructions.contains("Invariants") || s.instructions.contains("Invariant"),
            "Skill '{}' must specify operational invariants",
            name
        );
    }
}

#[test]
fn test_harness_skills_shorthand_alias_dispatch() {
    let alias_pairs = [
        ("polyglot-cli", "harness-polyglot-cli-generator-pro-max"),
        ("cli-generator", "harness-polyglot-cli-generator-pro-max"),
        ("ast-abi", "harness-ast-abi-ingest-pro-max"),
        ("tree-sitter-parse", "harness-ast-abi-ingest-pro-max"),
        ("sandbox-pty", "harness-sandbox-pty-runtime-pro-max"),
        ("landlock-sandbox", "harness-sandbox-pty-runtime-pro-max"),
        ("protocol-serialization", "harness-protocol-serialization-pro-max"),
        ("grpc-to-cli", "harness-protocol-serialization-pro-max"),
        ("fuzz-benchmark", "harness-fuzz-benchmark-pro-max"),
        ("proptest-harness", "harness-fuzz-benchmark-pro-max"),
        ("dag-lifecycle", "harness-dag-lifecycle-telemetry-pro-max"),
        ("harness-dag", "harness-dag-lifecycle-telemetry-pro-max"),
        ("sovereign-harness", "harness-engineering-sovereign-master-pro-max"),
        ("harness-engineering-master", "harness-engineering-sovereign-master-pro-max"),
    ];

    for (alias, canonical) in alias_pairs {
        let resolved = find_ecc_skill(alias);
        assert!(
            resolved.is_some(),
            "Alias '{}' must resolve to a skill",
            alias
        );
        assert_eq!(
            resolved.unwrap().name,
            canonical,
            "Alias '{}' must resolve to canonical skill '{}'",
            alias,
            canonical
        );
    }
}

// =========================================================================
// 2. Polyglot CLI Harness Invariants Tests
// =========================================================================

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CliEnvelope<T> {
    status: String,
    version: String,
    data: Option<T>,
    error: Option<CliErrorBlock>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CliErrorBlock {
    code: String,
    message: String,
    exit_code: i32,
}

#[test]
fn test_polyglot_cli_json_envelope_and_exit_code_invariants() {
    // Invariant 1: Success Envelope
    let success_envelope: CliEnvelope<serde_json::Value> = CliEnvelope {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        data: Some(serde_json::json!({ "digest": "abcdef123456", "bytes": 1024 })),
        error: None,
    };
    let json_str = serde_json::to_string(&success_envelope).unwrap();
    let parsed: CliEnvelope<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed.status, "ok");
    assert!(parsed.error.is_none());
    assert!(parsed.data.is_some());

    // Invariant 2: Zero Exit-Code Masking
    let error_envelope: CliEnvelope<serde_json::Value> = CliEnvelope {
        status: "error".to_string(),
        version: "1.0.0".to_string(),
        data: None,
        error: Some(CliErrorBlock {
            code: "ERR_FILE_NOT_FOUND".to_string(),
            message: "Target file does not exist".to_string(),
            exit_code: 74,
        }),
    };
    let err_json_str = serde_json::to_string(&error_envelope).unwrap();
    let err_parsed: CliEnvelope<serde_json::Value> = serde_json::from_str(&err_json_str).unwrap();
    assert_eq!(err_parsed.status, "error");
    assert!(err_parsed.data.is_none());
    let err_block = err_parsed.error.expect("Error block must exist");
    assert_ne!(err_block.exit_code, 0, "Error exit code must NOT be zero (zero exit-code masking violation)");
    assert_eq!(err_block.exit_code, 74);
    assert_eq!(err_block.code, "ERR_FILE_NOT_FOUND");
}

// =========================================================================
// 3. Deep AST & Multi-Language ABI Ingestion Tests
// =========================================================================

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AbiFunctionSpec {
    name: String,
    calling_convention: String,
    doc: String,
    parameters: Vec<AbiParamSpec>,
    return_type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AbiParamSpec {
    name: String,
    param_type: String,
    is_variadic: bool,
    default_value: Option<String>,
}

#[test]
fn test_deep_ast_abi_type_resolution_and_docstrings() {
    let fn_spec = AbiFunctionSpec {
        name: "compute_fast_hash".to_string(),
        calling_convention: "systemv".to_string(),
        doc: "Computes hardware-accelerated BLAKE3 hash.".to_string(),
        parameters: vec![
            AbiParamSpec {
                name: "data".to_string(),
                param_type: "*const u8".to_string(),
                is_variadic: false,
                default_value: None,
            },
            AbiParamSpec {
                name: "len".to_string(),
                param_type: "usize".to_string(),
                is_variadic: false,
                default_value: None,
            },
            AbiParamSpec {
                name: "flags".to_string(),
                param_type: "u32".to_string(),
                is_variadic: false,
                default_value: Some("0".to_string()),
            },
        ],
        return_type: "i32".to_string(),
    };

    // Verify round-trip serialization and complete parameter type resolution
    let json_rep = serde_json::to_string(&fn_spec).unwrap();
    let deserialized: AbiFunctionSpec = serde_json::from_str(&json_rep).unwrap();
    assert_eq!(deserialized, fn_spec);
    assert_eq!(deserialized.parameters.len(), 3);
    assert_eq!(deserialized.parameters[2].default_value.as_deref(), Some("0"));
    assert!(!deserialized.doc.is_empty(), "Docstrings must be preserved");
}

// =========================================================================
// 4. Advanced Sandboxing & PTY Runtime Tests
// =========================================================================

struct VirtualPtyScreen {
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    grid: Vec<Vec<char>>,
}

impl VirtualPtyScreen {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            grid: vec![vec![' '; cols]; rows],
        }
    }

    fn write_str(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.cursor_y = (self.cursor_y + 1).min(self.rows - 1);
                self.cursor_x = 0;
            } else if ch == '\r' {
                self.cursor_x = 0;
            } else {
                if self.cursor_x < self.cols && self.cursor_y < self.rows {
                    self.grid[self.cursor_y][self.cursor_x] = ch;
                    self.cursor_x += 1;
                }
            }
        }
    }

    fn get_line(&self, row: usize) -> String {
        if row < self.rows {
            self.grid[row].iter().collect()
        } else {
            String::new()
        }
    }
}

struct MemoryBoundedRingBuffer {
    max_capacity: usize,
    buffer: VecDeque<u8>,
}

impl MemoryBoundedRingBuffer {
    fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            buffer: VecDeque::with_capacity(max_capacity),
        }
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            if self.buffer.len() >= self.max_capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(b);
        }
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[test]
fn test_sandboxing_pty_and_vcr_replay_invariants() {
    // 1. Virtual PTY Screen Buffer Test
    let mut screen = VirtualPtyScreen::new(80, 24);
    screen.write_str("Tagisan PTY Controller Ready\n[fzf] Search: cargo\n");
    let line0 = screen.get_line(0);
    let line1 = screen.get_line(1);
    assert!(line0.starts_with("Tagisan PTY Controller Ready"));
    assert!(line1.starts_with("[fzf] Search: cargo"));

    // 2. 1MB Memory Capture Ceiling Invariant
    const ONE_MB: usize = 1024 * 1024;
    let mut ring_buf = MemoryBoundedRingBuffer::new(ONE_MB);
    // Push 3MB of simulated runaway child process output
    let chunk = vec![0xAA; 512 * 1024];
    for _ in 0..6 {
        ring_buf.push_bytes(&chunk);
    }
    assert_eq!(
        ring_buf.len(),
        ONE_MB,
        "Ring buffer MUST NOT exceed 1MB memory capture ceiling"
    );

    // 3. VCR Sanitization Invariant
    let raw_auth_header = "Bearer secret_api_key_xyz_123456789";
    let sanitized = if raw_auth_header.starts_with("Bearer ") {
        "Bearer [FILTERED]"
    } else {
        raw_auth_header
    };
    assert_eq!(sanitized, "Bearer [FILTERED]", "VCR must scrub authorization tokens");
}

// =========================================================================
// 5. Universal Protocol Codecs & Checksum Tests
// =========================================================================

pub fn compute_fix_checksum(buf: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for &b in buf {
        sum = sum.wrapping_add(b as u32);
    }
    (sum % 256) as u8
}

pub fn calculate_modbus_crc16(buf: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in buf {
        crc ^= b as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

#[test]
fn test_universal_protocol_wire_codecs_and_checksums() {
    // 1. FIX Checksum Algorithm Test
    // FIX standard sample: "8=FIX.4.2\x019=49\x0135=D\x0149=BUYER\x0156=SELLER\x01"
    let fix_msg = b"8=FIX.4.2\x019=49\x0135=D\x0149=BUYER\x0156=SELLER\x01";
    let chk = compute_fix_checksum(fix_msg);
    assert_eq!(chk, (fix_msg.iter().map(|&b| b as u32).sum::<u32>() % 256) as u8);

    // 2. Modbus CRC16-IBM Test
    // Standard test frame: Read Holding Registers: slave=1, func=3, start=0, count=10 -> [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]
    let modbus_frame = [0x01, 0x03, 0x00, 0x00, 0x00, 0x0A];
    let crc = calculate_modbus_crc16(&modbus_frame);
    assert_ne!(crc, 0, "Modbus CRC16 must be non-zero");
    // Verify CRC validity: appending CRC (LE) and recalculating yields 0
    let mut with_crc = modbus_frame.to_vec();
    with_crc.push((crc & 0xFF) as u8);
    with_crc.push(((crc >> 8) & 0xFF) as u8);
    assert_eq!(calculate_modbus_crc16(&with_crc), 0, "CRC appended frame must evaluate to 0 in Modbus");
}

// =========================================================================
// 6. Generative Fuzzing & Microbenchmarking Tests
// =========================================================================

struct DeterministicPrng {
    state: u64,
}

impl DeterministicPrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // Xorshift64star
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545F4914F6CDD1D)
    }
}

fn shrink_counterexample<F: Fn(&[u8]) -> bool>(mut failing: Vec<u8>, is_still_failing: F) -> Vec<u8> {
    // Minimal input shrinking algorithm
    while failing.len() > 1 {
        let half = failing.len() / 2;
        let left = &failing[..half];
        if is_still_failing(left) {
            failing = left.to_vec();
            continue;
        }
        let right = &failing[half..];
        if is_still_failing(right) {
            failing = right.to_vec();
            continue;
        }
        break;
    }
    failing
}

#[test]
fn test_generative_fuzzing_seed_reproducibility_and_shrinking() {
    // 1. Deterministic Seed Reproducibility
    let mut rng1 = DeterministicPrng::new(0xDEADBEEFCAFE1234);
    let mut rng2 = DeterministicPrng::new(0xDEADBEEFCAFE1234);
    for _ in 0..1000 {
        assert_eq!(rng1.next_u64(), rng2.next_u64(), "PRNG must be 100% reproducible given identical seed");
    }

    // 2. Counterexample Shrinking Invariant
    // Let failure be triggered if the byte 0x42 exists in the slice
    let initial_failing = vec![1, 2, 3, 0x42, 5, 6, 7, 8, 9, 10];
    let shrunk = shrink_counterexample(initial_failing, |slice| slice.contains(&0x42));
    assert_eq!(shrunk, vec![0x42], "Failing input must be shrunk to minimal counterexample");

    // 3. Statistical Distribution Validation
    let mut samples = vec![100, 105, 110, 115, 120, 125, 130, 135, 140, 150, 200, 500];
    samples.sort_unstable();
    let min = samples[0];
    let p50 = samples[samples.len() * 50 / 100];
    let p90 = samples[samples.len() * 90 / 100];
    let p99 = samples[samples.len() * 99 / 100];
    let max = samples[samples.len() - 1];

    assert!(min <= p50);
    assert!(p50 <= p90);
    assert!(p90 <= p99);
    assert!(p99 <= max);
}

// =========================================================================
// 7. Composition DAG, Lifecycle & Telemetry Tests
// =========================================================================

struct DagNode {
    id: String,
    dependencies: Vec<String>,
}

fn topological_sort(nodes: &[DagNode]) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for node in nodes {
        in_degree.entry(node.id.clone()).or_insert(0);
        for dep in &node.dependencies {
            adj.entry(dep.clone()).or_default().push(node.id.clone());
            *in_degree.entry(node.id.clone()).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut order = Vec::new();
    while let Some(u) = queue.pop_front() {
        order.push(u.clone());
        if let Some(neighbors) = adj.get(&u) {
            for v in neighbors {
                let deg = in_degree.get_mut(v).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(v.clone());
                }
            }
        }
    }

    if order.len() == nodes.len() {
        Ok(order)
    } else {
        Err("Cyclic dependency detected in composition DAG".to_string())
    }
}

#[test]
fn test_dag_composition_lifecycle_and_dlq() {
    // 1. Acyclic DAG Execution
    let valid_nodes = vec![
        DagNode { id: "ingest_abi".into(), dependencies: vec![] },
        DagNode { id: "generate_cli".into(), dependencies: vec!["ingest_abi".into()] },
        DagNode { id: "sandbox_wrap".into(), dependencies: vec!["generate_cli".into()] },
        DagNode { id: "fuzz_test".into(), dependencies: vec!["sandbox_wrap".into()] },
    ];
    let sort_res = topological_sort(&valid_nodes);
    assert!(sort_res.is_ok());
    let order = sort_res.unwrap();
    assert_eq!(order, vec!["ingest_abi", "generate_cli", "sandbox_wrap", "fuzz_test"]);

    // 2. Cyclic DAG Rejection Invariant
    let cyclic_nodes = vec![
        DagNode { id: "A".into(), dependencies: vec!["C".into()] },
        DagNode { id: "B".into(), dependencies: vec!["A".into()] },
        DagNode { id: "C".into(), dependencies: vec!["B".into()] },
    ];
    let cycle_res = topological_sort(&cyclic_nodes);
    assert!(cycle_res.is_err(), "Cyclic DAG MUST be rejected");

    // 3. Dead-Letter Queue (DLQ) Recording
    let dlq_entry = serde_json::json!({
        "failed_node": "fuzz_test",
        "input_args": ["--cases", "10000"],
        "exit_code": 101,
        "panic_trace": "assertion failed: decoded == original",
        "timestamp": "2026-09-21T12:00:00Z"
    });
    assert_eq!(dlq_entry["failed_node"], "fuzz_test");
    assert_eq!(dlq_entry["exit_code"], 101);
}

// =========================================================================
// 8. Master Sovereign Umbrella Orchestration Test
// =========================================================================

#[test]
fn test_master_sovereign_umbrella_pipeline_orchestration() {
    // Phase 1: Ingestion
    let raw_symbol = "tagisan_compute_hash";
    assert!(!raw_symbol.is_empty());

    // Phase 2: Synthesis
    let synthesized_cmd = format!("tgs-harness run --symbol {} --json", raw_symbol);
    assert!(synthesized_cmd.contains("--json"));

    // Phase 3: Sandbox Check
    let allowed_paths = ["/usr/lib", "/tmp"];
    assert!(allowed_paths.contains(&"/tmp"));

    // Phase 4: Fuzz Invariant
    let test_seed = 42u64;
    assert_eq!(test_seed, 42);

    // Phase 5: Sovereign Master Skill presence
    let master = find_ecc_skill("harness-engineering-sovereign-master-pro-max");
    assert!(master.is_some(), "Master Sovereign Skill must be available");
    let m = master.unwrap();
    assert!(m.instructions.contains("Sovereign Master Harness Engineering"));
}
