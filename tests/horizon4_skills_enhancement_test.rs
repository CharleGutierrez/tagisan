//! # Comprehensive Brutal Test Suite: Horizon 4 Skills & Systems Tools
//!
//! Validates:
//! 1. All 4 Horizon 4 skills (`binary-protocol-zero-copy-synthesizer`, `compiler-ir-lifter-optimizer`,
//!    `post-quantum-constant-time-auditor`, `legacy-systems-rejuvenator`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections.
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `BinaryProtocolSynthesizerTool` wire layout analysis, alignment/padding computation,
//!    zero-copy Rust code synthesis, and memory safety verification.
//! 5. `CompilerIrOptimizerTool` assembly/IR inhibitor auditing, pointer aliasing analysis,
//!    and microarchitectural optimization generation.
//! 6. `ConstantTimeAuditorTool` side-channel timing leak detection (branches, memory indexing, division)
//!    and cryptographic buffer zeroization verification.
//! 7. Registration in `ToolRegistry::with_builtins()` and `ToolRegistry::with_builtins_in_dir()`.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{
    BinaryProtocolSynthesizerTool, CompilerIrOptimizerTool, ConstantTimeAuditorTool, ToolHandler,
    ToolRegistry,
};

// =========================================================================
// Pillar 1: Horizon 4 Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_binary_protocol_zero_copy_synthesizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/binary-protocol-zero-copy-synthesizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/binary-protocol-zero-copy-synthesizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read binary-protocol-zero-copy-synthesizer SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: binary-protocol-zero-copy-synthesizer"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("zero-copy"));
    assert!(raw_content.contains("binary-protocols"));
    assert!(raw_content.contains("zerocopy"));
    assert!(raw_content.contains("nom"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("zero-copy"));
    assert!(raw_content.contains("byte-alignment"));
    assert!(raw_content.contains("packet-parsing"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on binary-protocol-zero-copy-synthesizer");
    assert_eq!(skill.name, "binary-protocol-zero-copy-synthesizer");
    assert!(
        skill.description.contains("Zero-copy") || skill.description.contains("zerocopy"),
        "Description must reference zero-copy: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Byte-Alignment & Transmutation Safety Proof"),
        "Must contain Byte-Alignment & Transmutation Safety Proof invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Infallible Bounds Checks & Panicking Elimination"),
        "Must contain Infallible Bounds Checks invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Endianness Normalization & Wire Invariants"),
        "Must contain Endianness Normalization invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Zero-Copy Parsing Pipeline with `nom` / Safe Transmutation"),
        "Must contain Zero-Copy Parsing Pipeline invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_compiler_ir_lifter_optimizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/compiler-ir-lifter-optimizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/compiler-ir-lifter-optimizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read compiler-ir-lifter-optimizer SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: compiler-ir-lifter-optimizer"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("compiler-optimization"));
    assert!(raw_content.contains("llvm-ir"));
    assert!(raw_content.contains("cranelift"));
    assert!(raw_content.contains("pointer-aliasing"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("compiler-optimizer"));
    assert!(raw_content.contains("branch-elimination"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on compiler-ir-lifter-optimizer");
    assert_eq!(skill.name, "compiler-ir-lifter-optimizer");
    assert!(
        skill.description.contains("LLVM-IR") || skill.description.contains("optimization"),
        "Description must reference LLVM-IR or optimization"
    );
    assert!(
        skill.instructions.contains("Invariant 1: Pointer Aliasing Elimination & Restrict Disjointness"),
        "Must contain Pointer Aliasing Elimination invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Branch Elimination & Predicated Execution"),
        "Must contain Branch Elimination invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Hot-Loop Register Allocation & Spill Elimination"),
        "Must contain Register Allocation invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Inline Assembly & Target Architecture Safeguards"),
        "Must contain Inline Assembly Safeguards invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_post_quantum_constant_time_auditor_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/post-quantum-constant-time-auditor/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/post-quantum-constant-time-auditor");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read post-quantum-constant-time-auditor SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: post-quantum-constant-time-auditor"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("constant-time"));
    assert!(raw_content.contains("cryptography"));
    assert!(raw_content.contains("post-quantum"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("timing-leak"));
    assert!(raw_content.contains("zeroize"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on post-quantum-constant-time-auditor");
    assert_eq!(skill.name, "post-quantum-constant-time-auditor");
    assert!(
        skill.description.contains("Post-quantum") || skill.description.contains("constant-time"),
        "Description must reference Post-quantum or constant-time"
    );
    assert!(
        skill.instructions.contains("Invariant 1: Elimination of Secret-Dependent Conditional Branching"),
        "Must contain Secret-Dependent Conditional Branching invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Elimination of Secret-Dependent Memory Indexing"),
        "Must contain Secret-Dependent Memory Indexing invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Cryptographic Memory Scrubbing & Zeroization"),
        "Must contain Memory Scrubbing & Zeroization invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Post-Quantum Lattice & Noise Invariants"),
        "Must contain Post-Quantum Lattice invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_legacy_systems_rejuvenator_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/legacy-systems-rejuvenator/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/legacy-systems-rejuvenator");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read legacy-systems-rejuvenator SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: legacy-systems-rejuvenator"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("legacy-rejuvenation"));
    assert!(raw_content.contains("c-to-rust"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("raw-pointer-elimination"));
    assert!(raw_content.contains("lifetime-synthesis"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on legacy-systems-rejuvenator");
    assert_eq!(skill.name, "legacy-systems-rejuvenator");
    assert!(
        skill.description.contains("C/C++") || skill.description.contains("rejuvenation"),
        "Description must reference C/C++ or rejuvenation"
    );
    assert!(
        skill.instructions.contains("Invariant 1: Borrow-Checker Lifetime Synthesis & Ownership"),
        "Must contain Borrow-Checker Lifetime Synthesis invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Raw Pointer Elimination & Safe Indexing"),
        "Must contain Raw Pointer Elimination invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Unsafe FFI Encapsulation & RAII Bound"),
        "Must contain Unsafe FFI Encapsulation invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Differential Property-Based Equivalence Proofs"),
        "Must contain Differential Property-Based Equivalence invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

// =========================================================================
// Pillar 2: Built-in Registry Discovery & Alias Resolution
// =========================================================================

#[test]
fn test_horizon4_skills_in_all_built_in_skills() {
    let builtins = all_built_in_skills();
    let names: Vec<&str> = builtins.iter().map(|s| s.name.as_str()).collect();

    assert!(names.contains(&"binary-protocol-zero-copy-synthesizer"), "Must contain binary-protocol-zero-copy-synthesizer");
    assert!(names.contains(&"compiler-ir-lifter-optimizer"), "Must contain compiler-ir-lifter-optimizer");
    assert!(names.contains(&"post-quantum-constant-time-auditor"), "Must contain post-quantum-constant-time-auditor");
    assert!(names.contains(&"legacy-systems-rejuvenator"), "Must contain legacy-systems-rejuvenator");
}

#[test]
fn test_horizon4_skills_alias_resolution() {
    // Exact lookups
    assert!(find_built_in_skill("binary-protocol-zero-copy-synthesizer").is_some());
    assert!(find_built_in_skill("compiler-ir-lifter-optimizer").is_some());
    assert!(find_built_in_skill("post-quantum-constant-time-auditor").is_some());
    assert!(find_built_in_skill("legacy-systems-rejuvenator").is_some());

    // Aliases
    assert_eq!(
        find_built_in_skill("zero-copy").map(|s| s.name),
        Some("binary-protocol-zero-copy-synthesizer".to_string())
    );
    assert_eq!(
        find_built_in_skill("binary-protocol").map(|s| s.name),
        Some("binary-protocol-zero-copy-synthesizer".to_string())
    );
    assert_eq!(
        find_built_in_skill("zerocopy").map(|s| s.name),
        Some("binary-protocol-zero-copy-synthesizer".to_string())
    );
    assert_eq!(
        find_built_in_skill("compiler-optimizer").map(|s| s.name),
        Some("compiler-ir-lifter-optimizer".to_string())
    );
    assert_eq!(
        find_built_in_skill("llvm-ir").map(|s| s.name),
        Some("compiler-ir-lifter-optimizer".to_string())
    );
    assert_eq!(
        find_built_in_skill("constant-time").map(|s| s.name),
        Some("post-quantum-constant-time-auditor".to_string())
    );
    assert_eq!(
        find_built_in_skill("pqc").map(|s| s.name),
        Some("post-quantum-constant-time-auditor".to_string())
    );
    assert_eq!(
        find_built_in_skill("legacy-rejuvenator").map(|s| s.name),
        Some("legacy-systems-rejuvenator".to_string())
    );
    assert_eq!(
        find_built_in_skill("c-to-rust").map(|s| s.name),
        Some("legacy-systems-rejuvenator".to_string())
    );
}

// =========================================================================
// Pillar 3: BinaryProtocolSynthesizerTool Verification
// =========================================================================

#[tokio::test]
async fn test_binary_protocol_synthesizer_analyze_wire_format() {
    let tool = BinaryProtocolSynthesizerTool::new();
    let wire_spec = json!({
        "fields": [
            { "name": "magic", "type": "[u8; 4]" },
            { "name": "version", "type": "u8" },
            { "name": "sequence", "type": "u32" },
            { "name": "payload_len", "type": "u16" }
        ]
    });

    let args = json!({
        "action": "analyze_wire_format",
        "protocol_name": "SensorPacket",
        "wire_spec": wire_spec,
        "endianness": "big",
        "packed": false
    });

    let res_str = tool.execute(args).await.expect("execute must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("result must be valid json");

    assert_eq!(res["protocol_name"], "SensorPacket");
    assert_eq!(res["endianness"], "big");
    assert_eq!(res["struct_alignment"], 4);
    // [u8; 4] at 0 (size 4) -> u8 at 4 (size 1) -> 3 bytes padding -> u32 at 8 (size 4) -> u16 at 12 (size 2) -> 2 bytes tail padding -> 16 total
    assert_eq!(res["total_size_bytes"], 16);
    assert!(res["total_padding_bytes"].as_u64().unwrap() >= 3);
    assert_eq!(res["fields"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn test_binary_protocol_synthesizer_synthesize_zerocopy() {
    let tool = BinaryProtocolSynthesizerTool::new();
    let wire_spec = json!({
        "fields": [
            { "name": "sequence", "type": "u32" },
            { "name": "flags", "type": "u16" },
            { "name": "temperature", "type": "f32" }
        ]
    });

    let args = json!({
        "action": "synthesize_zerocopy",
        "protocol_name": "TelemetryFrame",
        "wire_spec": wire_spec,
        "endianness": "little"
    });

    let res_str = tool.execute(args).await.expect("synthesize_zerocopy must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("result must be valid json");

    assert_eq!(res["protocol_name"], "TelemetryFrame");
    let code = res["generated_rust_code"].as_str().expect("code must be present");
    assert!(code.contains("#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]"));
    assert!(code.contains("#[repr(C)]"));
    assert!(code.contains("pub struct TelemetryFrameHeader"));
    assert!(code.contains("U32<LE>"));
    assert!(code.contains("U16<LE>"));
    assert!(code.contains("parse_from_prefix(buf: &[u8])"));
    assert!(code.contains("zerocopy::Ref::<_, Self>::from_bytes"));
}

#[tokio::test]
async fn test_binary_protocol_synthesizer_verify_safety() {
    let tool = BinaryProtocolSynthesizerTool::new();
    // Packed struct with misaligned u32
    let wire_spec = json!({
        "fields": [
            { "name": "flag", "type": "u8" },
            { "name": "counter", "type": "u32" }
        ]
    });

    let args = json!({
        "action": "verify_safety",
        "protocol_name": "PackedMisaligned",
        "wire_spec": wire_spec,
        "packed": true
    });

    let res_str = tool.execute(args).await.expect("verify_safety must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("result must be valid json");

    assert_eq!(res["protocol_name"], "PackedMisaligned");
    assert_eq!(res["safety_verdict"], "CRITICAL_HAZARDS");
    assert!(res["hazard_count"].as_u64().unwrap() > 0);
    let hazards = res["hazards"].as_array().unwrap();
    assert!(hazards.iter().any(|h| h["hazard"].as_str().unwrap().contains("Unaligned")));
}

// =========================================================================
// Pillar 4: CompilerIrOptimizerTool Verification
// =========================================================================

#[tokio::test]
async fn test_compiler_ir_optimizer_analyze_assembly() {
    let tool = CompilerIrOptimizerTool::new();
    let snippet = r#"
    pub fn hot_loop(data: &[f32], factor: f32) -> f32 {
        let mut sum = 0.0;
        for i in 0..data.len() {
            if data[i] > 0.0 {
                sum += data[i] * factor;
            }
        }
        sum
    }
    "#;

    let args = json!({
        "action": "analyze_assembly",
        "code": snippet,
        "target_arch": "x86_64"
    });

    let res_str = tool.execute(args).await.expect("analyze_assembly must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json response");

    assert_eq!(res["target_arch"], "x86_64");
    assert!(res["inhibitors_detected"].as_u64().unwrap() > 0);
    let inhibitors = res["inhibitors"].as_array().unwrap();
    assert!(inhibitors.iter().any(|i| i["type"] == "BranchDivergence"));
    assert!(res["estimated_cycle_penalty"].as_u64().unwrap() >= 20);
}

#[tokio::test]
async fn test_compiler_ir_optimizer_detect_aliasing_penalties() {
    let tool = CompilerIrOptimizerTool::new();
    let raw_ptr_code = r#"
    pub unsafe fn compute(a: *mut f32, b: *const f32, count: usize) {
        for i in 0..count {
            *a.add(i) += *b.add(i) * 2.0;
        }
    }
    "#;

    let args = json!({
        "action": "detect_aliasing_penalties",
        "code": raw_ptr_code,
        "target_arch": "x86_64"
    });

    let res_str = tool.execute(args).await.expect("detect_aliasing_penalties must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json");

    assert!(res["aliasing_hazards_count"].as_u64().unwrap() > 0);
    assert_eq!(res["memory_reload_risk"], true);
    let hazards = res["aliasing_hazards"].as_array().unwrap();
    assert!(hazards.iter().any(|h| h["hazard"] == "RawPointerAmbiguity"));
}

#[tokio::test]
async fn test_compiler_ir_optimizer_generate_optimizations() {
    let tool = CompilerIrOptimizerTool::new();
    let code = r#"
    pub fn vector_kernel(a: &[f32], b: &[f32], out: &mut [f32]) {
        for i in 0..a.len() {
            if a[i] > 0.0 {
                out[i] = a[i] + b[i];
            } else {
                panic!("Negative value encountered");
            }
        }
    }
    "#;

    let args = json!({
        "action": "generate_optimizations",
        "code": code,
        "target_arch": "x86_64"
    });

    let res_str = tool.execute(args).await.expect("generate_optimizations must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json");

    let opt_code = res["optimized_code"].as_str().expect("optimized_code must exist");
    assert!(opt_code.contains("#[inline(always)]"));
    assert!(opt_code.contains("target_feature"));
    assert!(opt_code.contains("#[cold]"));
    let applied = res["applied_passes"].as_array().unwrap();
    assert!(applied.iter().any(|p| p.as_str().unwrap().contains("inline(always)")));
    assert!(applied.iter().any(|p| p.as_str().unwrap().contains("Branchless")));
}

// =========================================================================
// Pillar 5: ConstantTimeAuditorTool Verification
// =========================================================================

#[tokio::test]
async fn test_constant_time_auditor_detects_branch_and_indexing_leaks() {
    let tool = ConstantTimeAuditorTool::new();
    let vulnerable_crypto = r#"
    pub fn verify_signature(secret_key: &[u8], candidate: &[u8], sbox: &[u8; 256]) -> bool {
        for i in 0..secret_key.len() {
            // Leak 1: Secret-dependent branch
            if secret_key[i] == 0 {
                return false;
            }
            // Leak 2: Secret-dependent memory lookup
            val = sbox[secret_key[i]];
            // Leak 3: Variable-time division
            let div_val = 1000 / secret_key[i];
            // Leak 4: Early-exit comparison
            if secret_key != candidate {
                return false;
            }
        }
        true
    }
    "#;

    let args = json!({
        "action": "audit_timing_leaks",
        "code": vulnerable_crypto,
        "sensitive_variables": ["secret_key"]
    });

    let res_str = tool.execute(args).await.expect("audit_timing_leaks must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json");

    assert_eq!(res["audit_verdict"], "TIMING_LEAKS_DETECTED");
    assert!(res["leak_count"].as_u64().unwrap() >= 3);

    let leaks = res["detected_leaks"].as_array().unwrap();
    assert!(leaks.iter().any(|l| l["leak_type"] == "SecretDependentConditionalBranch"));
    assert!(leaks.iter().any(|l| l["leak_type"] == "SecretDependentMemoryIndexing"));
    assert!(leaks.iter().any(|l| l["leak_type"] == "VariableTimeArithmetic"));
    assert!(leaks.iter().any(|l| l["leak_type"] == "EarlyExitComparison"));
}

#[tokio::test]
async fn test_constant_time_auditor_passes_clean_code() {
    let tool = ConstantTimeAuditorTool::new();
    let secure_crypto = r#"
    use subtle::{Choice, ConstantTimeEq, ConditionallySelectable};

    pub fn verify_token_constant_time(secret_key: &[u8; 32], candidate: &[u8; 32]) -> bool {
        let is_match: Choice = secret_key.ct_eq(candidate);
        is_match.into()
    }
    "#;

    let args = json!({
        "action": "audit_timing_leaks",
        "code": secure_crypto,
        "sensitive_variables": ["secret_key"]
    });

    let res_str = tool.execute(args).await.expect("audit_timing_leaks must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json");

    assert_eq!(res["audit_verdict"], "SECURE_CONSTANT_TIME");
    assert_eq!(res["leak_count"], 0);
}

#[tokio::test]
async fn test_constant_time_auditor_verify_zeroization() {
    let tool = ConstantTimeAuditorTool::new();
    let unscrubbed_code = r#"
    pub struct SecretContext {
        pub private_seed: [u8; 32],
    }
    "#;

    let args = json!({
        "action": "verify_secret_zeroization",
        "code": unscrubbed_code,
        "sensitive_variables": ["private_seed"]
    });

    let res_str = tool.execute(args).await.expect("verify_secret_zeroization must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("valid json");

    assert_eq!(res["zeroization_verdict"], "UNPROTECTED_SENSITIVE_BUFFERS");
    assert_eq!(res["has_zeroize_derive"], false);
    assert_eq!(res["unscrubbed_variables_count"], 1);

    // Now test scrubbed code
    let scrubbed_code = r#"
    use zeroize::{Zeroize, ZeroizeOnDrop};

    #[derive(Zeroize, ZeroizeOnDrop)]
    pub struct SecretContext {
        pub private_seed: [u8; 32],
    }
    "#;

    let clean_args = json!({
        "action": "verify_secret_zeroization",
        "code": scrubbed_code,
        "sensitive_variables": ["private_seed"]
    });

    let clean_res_str = tool.execute(clean_args).await.expect("verify_secret_zeroization clean must succeed");
    let clean_res: Value = serde_json::from_str(&clean_res_str).expect("valid json");

    assert_eq!(clean_res["zeroization_verdict"], "FULLY_SCRUBBED");
    assert_eq!(clean_res["has_zeroize_derive"], true);
    assert_eq!(clean_res["unscrubbed_variables_count"], 0);
}

// =========================================================================
// Pillar 6: ToolRegistry Registration
// =========================================================================

#[test]
fn test_tool_registry_with_builtins_contains_horizon4_tools() {
    let registry = ToolRegistry::with_builtins();
    let schema = registry.definitions();
    let names: Vec<&str> = schema.iter().map(|t| t.name.as_str()).collect();

    assert!(names.contains(&"binary_protocol_synthesizer"), "Must contain binary_protocol_synthesizer");
    assert!(names.contains(&"compiler_ir_optimizer"), "Must contain compiler_ir_optimizer");
    assert!(names.contains(&"constant_time_auditor"), "Must contain constant_time_auditor");
}

#[test]
fn test_tool_registry_with_builtins_in_dir_contains_horizon4_tools() {
    let temp_dir = std::env::temp_dir();
    let registry = ToolRegistry::with_builtins_in_dir(temp_dir);
    let schema = registry.definitions();
    let names: Vec<&str> = schema.iter().map(|t| t.name.as_str()).collect();

    assert!(names.contains(&"binary_protocol_synthesizer"), "Must contain binary_protocol_synthesizer in dir");
    assert!(names.contains(&"compiler_ir_optimizer"), "Must contain compiler_ir_optimizer in dir");
    assert!(names.contains(&"constant_time_auditor"), "Must contain constant_time_auditor in dir");
}
