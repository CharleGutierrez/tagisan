//! # Comprehensive Brutal Test Suite: External Ecosystem Skills & Systems Tools
//!
//! Validates:
//! 1. All 4 External Ecosystem skills (`ebpf-kernel-telemetry-tracer`, `kani-rust-formal-verifier`,
//!    `triton-cuda-tensor-kernel-fuser`, `kubernetes-operator-crd-sentinel`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections (`ALWAYS`, `NEVER`, `MANDATORY`, `STRICT_REJECT`).
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `EbpfTelemetryTracerTool` probe analysis, code synthesis, and bottleneck diagnosis.
//! 5. `KaniFormalVerifierTool` panic auditing, proof harness synthesis, and bounds verification.
//! 6. `TritonKernelFuserTool` tile analysis, bank conflict checks, and kernel synthesis.
//! 7. Registration in `ToolRegistry::with_builtins()` and `ToolRegistry::with_builtins_in_dir()`.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{
    EbpfTelemetryTracerTool, KaniFormalVerifierTool, TritonKernelFuserTool, ToolHandler,
    ToolRegistry,
};

// =========================================================================
// Pillar 1: External Ecosystem Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_ebpf_kernel_telemetry_tracer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/ebpf-kernel-telemetry-tracer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/ebpf-kernel-telemetry-tracer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read ebpf-kernel-telemetry-tracer SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: ebpf-kernel-telemetry-tracer"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("ebpf"));
    assert!(raw_content.contains("kernel-telemetry"));
    assert!(raw_content.contains("libbpf"));
    assert!(raw_content.contains("aya"));
    assert!(raw_content.contains("bpf-verifier"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("ebpf-tracer"));
    assert!(raw_content.contains("kprobe-telemetry"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on ebpf-kernel-telemetry-tracer");
    assert_eq!(skill.name, "ebpf-kernel-telemetry-tracer");
    assert!(
        skill.description.contains("eBPF") || skill.description.contains("telemetry") || skill.description.contains("observability"),
        "Description must reference eBPF or telemetry: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: BPF Verifier Compliance & Stack Limit Proofs"),
        "Must contain Invariant 1: BPF Verifier Compliance"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Zero Kernel Panic Hazard & Safe Pointer Dereferencing"),
        "Must contain Invariant 2: Zero Kernel Panic Hazard"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Minimal Ringbuffer Overhead & Non-Blocking Asynchronous Exfiltration"),
        "Must contain Invariant 3: Minimal Ringbuffer Overhead"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Futex Contention & CPU Cache-Miss Hardware Counter Diagnostics"),
        "Must contain Invariant 4: Futex Contention & CPU Cache-Miss"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_kani_rust_formal_verifier_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/kani-rust-formal-verifier/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/kani-rust-formal-verifier");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read kani-rust-formal-verifier SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: kani-rust-formal-verifier"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("kani"));
    assert!(raw_content.contains("formal-verification"));
    assert!(raw_content.contains("smt-solver"));
    assert!(raw_content.contains("bounded-model-checking"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("kani-verifier"));
    assert!(raw_content.contains("zero-panic-proof"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on kani-rust-formal-verifier");
    assert_eq!(skill.name, "kani-rust-formal-verifier");
    assert!(
        skill.description.contains("Kani") || skill.description.contains("formal") || skill.description.contains("verification"),
        "Description must reference Kani or verification: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Mathematical Proof of Panic Freedom & Panic Path Elimination"),
        "Must contain Invariant 1: Mathematical Proof of Panic Freedom"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Exhaustive Bounded Induction & Loop Unrolling"),
        "Must contain Invariant 2: Exhaustive Bounded Induction"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Pointer Validity & Spatial/Temporal Memory Safety Proofs"),
        "Must contain Invariant 3: Pointer Validity"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Safe Proof Harness Generation & Non-Deterministic Inoculation"),
        "Must contain Invariant 4: Safe Proof Harness Generation"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_triton_cuda_tensor_kernel_fuser_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/triton-cuda-tensor-kernel-fuser/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/triton-cuda-tensor-kernel-fuser");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read triton-cuda-tensor-kernel-fuser SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: triton-cuda-tensor-kernel-fuser"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("triton"));
    assert!(raw_content.contains("cuda"));
    assert!(raw_content.contains("gpu-kernel"));
    assert!(raw_content.contains("flash-attention"));
    assert!(raw_content.contains("bank-conflicts"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("triton-kernel"));
    assert!(raw_content.contains("flash-attention-kernel"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on triton-cuda-tensor-kernel-fuser");
    assert_eq!(skill.name, "triton-cuda-tensor-kernel-fuser");
    assert!(
        skill.description.contains("Triton") || skill.description.contains("kernel") || skill.description.contains("tensor"),
        "Description must reference Triton or kernel: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Coalesced Global Memory Load/Store & Vectorized Transfers"),
        "Must contain Invariant 1: Coalesced Global Memory"
    );
    assert!(
        skill.instructions.contains("Invariant 2: 32-Way Shared Memory Bank Conflict Elimination & Swizzling"),
        "Must contain Invariant 2: Shared Memory Bank Conflict Elimination"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Tensor Core Warp Shuffle Communication & Register Tiling"),
        "Must contain Invariant 3: Tensor Core Warp Shuffle"
    );
    assert!(
        skill.instructions.contains("Invariant 4: FlashAttention Tiling & Online Softmax Normalization"),
        "Must contain Invariant 4: FlashAttention Tiling"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_kubernetes_operator_crd_sentinel_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/kubernetes-operator-crd-sentinel/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/kubernetes-operator-crd-sentinel");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read kubernetes-operator-crd-sentinel SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: kubernetes-operator-crd-sentinel"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("kubernetes"));
    assert!(raw_content.contains("kube-rs"));
    assert!(raw_content.contains("operator"));
    assert!(raw_content.contains("crd-reconciliation"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("crd-controller"));
    assert!(raw_content.contains("operator-status-patch"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on kubernetes-operator-crd-sentinel");
    assert_eq!(skill.name, "kubernetes-operator-crd-sentinel");
    assert!(
        skill.description.contains("Kubernetes") || skill.description.contains("Operator") || skill.description.contains("CRD"),
        "Description must reference Kubernetes, Operator, or CRD: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Strict Idempotence in Controller Reconciliation Loops"),
        "Must contain Invariant 1: Strict Idempotence"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Finalizer Lifecycle Safety & Leak-Proof Resource Deletion"),
        "Must contain Invariant 2: Finalizer Lifecycle Safety"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Atomic Status Subresource Patching & Optimistic Concurrency Control"),
        "Must contain Invariant 3: Atomic Status Subresource Patching"
    );
    assert!(
        skill.instructions.contains("Invariant 4: High-Availability Lease-Based Leader Election Failover"),
        "Must contain Invariant 4: High-Availability Leader Election"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

// =========================================================================
// Pillar 2: Skills Registry Discovery & Alias Resolution
// =========================================================================

#[test]
fn test_all_built_in_skills_contains_all_four_ecosystem_skills() {
    let skills = all_built_in_skills();
    let skill_names: Vec<String> = skills.into_iter().map(|s| s.name).collect();

    assert!(
        skill_names.contains(&"ebpf-kernel-telemetry-tracer".to_string()),
        "ebpf-kernel-telemetry-tracer must be in all_built_in_skills()"
    );
    assert!(
        skill_names.contains(&"kani-rust-formal-verifier".to_string()),
        "kani-rust-formal-verifier must be in all_built_in_skills()"
    );
    assert!(
        skill_names.contains(&"triton-cuda-tensor-kernel-fuser".to_string()),
        "triton-cuda-tensor-kernel-fuser must be in all_built_in_skills()"
    );
    assert!(
        skill_names.contains(&"kubernetes-operator-crd-sentinel".to_string()),
        "kubernetes-operator-crd-sentinel must be in all_built_in_skills()"
    );
}

#[test]
fn test_find_built_in_skill_direct_and_aliases() {
    // 1. ebpf-kernel-telemetry-tracer
    assert!(find_built_in_skill("ebpf-kernel-telemetry-tracer").is_some());
    assert_eq!(find_built_in_skill("ebpf").unwrap().name, "ebpf-kernel-telemetry-tracer");
    assert_eq!(find_built_in_skill("ebpf-tracer").unwrap().name, "ebpf-kernel-telemetry-tracer");
    assert_eq!(find_built_in_skill("aya").unwrap().name, "ebpf-kernel-telemetry-tracer");
    assert_eq!(find_built_in_skill("libbpf").unwrap().name, "ebpf-kernel-telemetry-tracer");

    // 2. kani-rust-formal-verifier
    assert!(find_built_in_skill("kani-rust-formal-verifier").is_some());
    assert_eq!(find_built_in_skill("kani").unwrap().name, "kani-rust-formal-verifier");
    assert_eq!(find_built_in_skill("kani-verifier").unwrap().name, "kani-rust-formal-verifier");
    assert_eq!(find_built_in_skill("formal-verifier").unwrap().name, "kani-rust-formal-verifier");

    // 3. triton-cuda-tensor-kernel-fuser
    assert!(find_built_in_skill("triton-cuda-tensor-kernel-fuser").is_some());
    assert_eq!(find_built_in_skill("triton").unwrap().name, "triton-cuda-tensor-kernel-fuser");
    assert_eq!(find_built_in_skill("triton-kernel").unwrap().name, "triton-cuda-tensor-kernel-fuser");
    assert_eq!(find_built_in_skill("cuda-fuser").unwrap().name, "triton-cuda-tensor-kernel-fuser");

    // 4. kubernetes-operator-crd-sentinel
    assert!(find_built_in_skill("kubernetes-operator-crd-sentinel").is_some());
    assert_eq!(find_built_in_skill("kubernetes-operator").unwrap().name, "kubernetes-operator-crd-sentinel");
    assert_eq!(find_built_in_skill("kube-rs").unwrap().name, "kubernetes-operator-crd-sentinel");
    assert_eq!(find_built_in_skill("crd-sentinel").unwrap().name, "kubernetes-operator-crd-sentinel");
    assert_eq!(find_built_in_skill("crd-reconciliation").unwrap().name, "kubernetes-operator-crd-sentinel");
}

// =========================================================================
// Pillar 3: EbpfTelemetryTracerTool Invariant & Execution Tests
// =========================================================================

#[tokio::test]
async fn test_ebpf_telemetry_tracer_tool_probe_analysis() {
    let tool = EbpfTelemetryTracerTool::new();
    assert_eq!(tool.name(), "ebpf_telemetry_tracer");

    let args = json!({
        "action": "analyze_kernel_probes",
        "probe_type": "kprobe",
        "event_name": "sys_enter_futex",
        "target_process": "high_throughput_worker"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["probe_type"], "kprobe");
    assert_eq!(res["event_name"], "sys_enter_futex");
    assert_eq!(res["target_process"], "high_throughput_worker");

    // BPF verifier constraints
    let verifier = &res["bpf_verifier_compliance"];
    assert_eq!(verifier["max_stack_allowed_bytes"], 512);
    assert_eq!(verifier["status"], "VERIFIER_PASS");
    assert!(verifier["bounded_loops_verified"].as_bool().unwrap());
    assert!(verifier["zero_panic_hazard"].as_bool().unwrap());

    // Transport verification
    let transport = &res["telemetry_exfiltration"];
    assert_eq!(transport["transport"], "BPF_MAP_TYPE_RINGBUF");
    assert!(transport["non_blocking_submission"].as_bool().unwrap());
}

#[tokio::test]
async fn test_ebpf_telemetry_tracer_tool_code_synthesis() {
    let tool = EbpfTelemetryTracerTool::new();
    let args = json!({
        "action": "synthesize_bpf_program",
        "probe_type": "tracepoint",
        "event_name": "sched_switch",
        "target_process": "tgs_engine"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    let bpf_code = res["synthesized_bpf_code"].as_str().expect("Must have bpf code");
    assert!(bpf_code.contains("#![no_std]"), "eBPF must be no_std");
    assert!(bpf_code.contains("#![no_main]"), "eBPF must be no_main");
    assert!(bpf_code.contains("#[tracepoint]"), "Must have tracepoint macro");
    assert!(bpf_code.contains("TELEMETRY_RINGBUF"), "Must declare ring buffer map");
    assert!(bpf_code.contains("tgs_engine"), "Must contain target process filter");

    let loader_code = res["synthesized_userspace_loader"].as_str().expect("Must have loader code");
    assert!(loader_code.contains("RingBuf"), "Loader must consume RingBuf");
    assert!(loader_code.contains("TELEMETRY_RINGBUF"), "Loader must bind to TELEMETRY_RINGBUF");
}

#[tokio::test]
async fn test_ebpf_telemetry_tracer_tool_bottleneck_diagnosis() {
    let tool = EbpfTelemetryTracerTool::new();

    // 1. Futex lock contention diagnosis
    let args_futex = json!({
        "action": "diagnose_perf_bottleneck",
        "event_name": "futex_wait",
        "target_process": "db_writer"
    });
    let res_futex: Value = serde_json::from_str(&tool.execute(args_futex).await.unwrap()).unwrap();
    assert_eq!(res_futex["bottleneck_category"], "FUTEX_LOCK_CONTENTION");
    assert!(res_futex["remediation"].as_array().unwrap().len() >= 2);

    // 2. CPU LLC cache miss diagnosis
    let args_cache = json!({
        "action": "diagnose_perf_bottleneck",
        "event_name": "perf_count_hw_cache_misses",
        "target_process": "matrix_multiply"
    });
    let res_cache: Value = serde_json::from_str(&tool.execute(args_cache).await.unwrap()).unwrap();
    assert_eq!(res_cache["bottleneck_category"], "CPU_LLC_CACHE_MISSES");
    assert!(res_cache["telemetry_metrics"]["cacheline_bouncing_detected"].as_bool().unwrap());
}

// =========================================================================
// Pillar 4: KaniFormalVerifierTool Invariant & Execution Tests
// =========================================================================

#[tokio::test]
async fn test_kani_formal_verifier_tool_panic_auditing_hazards() {
    let tool = KaniFormalVerifierTool::new();
    assert_eq!(tool.name(), "kani_formal_verifier");

    let hazardous_code = r#"
pub fn risky_computation(data: &[u64], idx: usize, divisor: u64) -> u64 {
    let item = data[idx]; // Direct slice indexing
    let opt = Some(item);
    let val = opt.unwrap(); // Unchecked unwrap
    let quotient = val / divisor; // Unchecked division
    if quotient > 1000 {
        panic!("Overflow detected!"); // Explicit panic
    }
    quotient
}
"#;

    let args = json!({
        "action": "audit_panic_freedom",
        "code": hazardous_code
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["panic_freedom_verdict"], "PANIC_HAZARDS_DETECTED");
    assert!(res["hazards_detected_count"].as_u64().unwrap() >= 3);
    assert!(res["safety_score"].as_u64().unwrap() < 100);

    let findings = res["findings"].as_array().unwrap();
    let hazard_types: Vec<&str> = findings.iter().filter_map(|f| f["hazard_type"].as_str()).collect();
    assert!(hazard_types.contains(&"UNCHECKED_UNWRAP"));
    assert!(hazard_types.contains(&"POTENTIAL_DIVIDE_BY_ZERO"));
    assert!(hazard_types.contains(&"EXPLICIT_PANIC_MACRO"));
}

#[tokio::test]
async fn test_kani_formal_verifier_tool_panic_auditing_clean() {
    let tool = KaniFormalVerifierTool::new();

    let clean_code = r#"
pub fn verified_safe_add(a: u64, b: u64) -> Option<u64> {
    a.checked_add(b)
}
"#;

    let args = json!({
        "action": "audit_panic_freedom",
        "code": clean_code
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["panic_freedom_verdict"], "PROVABLY_PANIC_FREE");
    assert_eq!(res["safety_score"], 100);
    assert_eq!(res["hazards_detected_count"], 0);
}

#[tokio::test]
async fn test_kani_formal_verifier_tool_harness_synthesis() {
    let tool = KaniFormalVerifierTool::new();

    let code = r#"
pub fn compute_bounded_index(base: u64, offset: u64, capacity: usize) -> u64 {
    (base.wrapping_add(offset)) % (capacity as u64)
}
"#;

    let args = json!({
        "action": "synthesize_proof_harness",
        "code": code,
        "target_function": "compute_bounded_index"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["target_function"], "compute_bounded_index");
    assert_eq!(res["unwind_bound"], 16);

    let harness = res["synthesized_harness"].as_str().unwrap();
    assert!(harness.contains("#[kani::proof]"), "Must have #[kani::proof]");
    assert!(harness.contains("#[kani::unwind(16)]"), "Must have unwind bound");
    assert!(harness.contains("kani::any()"), "Must generate non-deterministic inputs");
    assert!(harness.contains("kani::assume"), "Must establish preconditions");
    assert!(harness.contains("kani::assert!"), "Must assert invariants");
}

#[tokio::test]
async fn test_kani_formal_verifier_tool_bounds_verification() {
    let tool = KaniFormalVerifierTool::new();

    let code = r#"
pub fn read_buffer_item(buf: &[u8], i: usize) -> u8 {
    buf[i]
}
"#;

    let args = json!({
        "action": "verify_bounds_invariants",
        "code": code,
        "target_function": "read_buffer_item"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    let static_analysis = &res["static_bounds_analysis"];
    assert_eq!(static_analysis["bounds_safety_verdict"], "PROVABLY_BOUNDED");
    assert!(static_analysis["slice_accesses_analyzed"].as_u64().unwrap() >= 1);
}

// =========================================================================
// Pillar 5: TritonKernelFuserTool Invariant & Execution Tests
// =========================================================================

#[tokio::test]
async fn test_triton_kernel_fuser_tool_tile_analysis() {
    let tool = TritonKernelFuserTool::new();
    assert_eq!(tool.name(), "triton_kernel_fuser");

    let args = json!({
        "action": "analyze_tile_sizes",
        "operation": "gemm",
        "block_size_m": 128,
        "block_size_n": 128,
        "block_size_k": 32
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["operation"], "gemm");
    assert_eq!(res["tile_configuration"]["BLOCK_SIZE_M"], 128);
    assert_eq!(res["tile_configuration"]["BLOCK_SIZE_N"], 128);
    assert_eq!(res["tile_configuration"]["BLOCK_SIZE_K"], 32);

    let sram = &res["sram_footprint"];
    assert!(sram["shared_memory_per_block_kb"].as_f64().unwrap() > 0.0);
    assert_eq!(sram["sram_fit_verdict"], "OPTIMAL_SRAM_FIT");

    let occupancy = &res["warp_and_occupancy"];
    assert!(occupancy["warps_per_block"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_triton_kernel_fuser_tool_bank_conflicts() {
    let tool = TritonKernelFuserTool::new();

    // 1. Conflict test with stride 32 (multiple of 32 -> 32-way conflict)
    let args_conflict = json!({
        "action": "check_bank_conflicts",
        "operation": "gemm",
        "block_size_m": 128,
        "block_size_n": 128,
        "block_size_k": 32
    });

    let res_conflict: Value = serde_json::from_str(&tool.execute(args_conflict).await.unwrap()).unwrap();
    let bank_analysis = &res_conflict["bank_conflict_analysis"];
    assert_eq!(bank_analysis["conflict_degree"], 32);
    assert!(bank_analysis["has_bank_conflicts"].as_bool().unwrap());
    assert!(bank_analysis["serialization_penalty_cycles"].as_u64().unwrap() > 0);

    // Remediation must provide padded stride 33 to resolve conflicts
    let remediation = &res_conflict["remediation"];
    assert_eq!(remediation["recommended_padded_stride"], 33);
    assert_eq!(remediation["resolved_conflict_degree"], "1-WAY (CONFLICT_FREE)");

    // 2. Conflict-free test with stride 33 (GCD(33, 32) = 1)
    let args_clean = json!({
        "action": "check_bank_conflicts",
        "operation": "gemm",
        "block_size_m": 128,
        "block_size_n": 128,
        "block_size_k": 33
    });

    let res_clean: Value = serde_json::from_str(&tool.execute(args_clean).await.unwrap()).unwrap();
    assert_eq!(res_clean["bank_conflict_analysis"]["conflict_degree"], 1);
    assert!(!res_clean["bank_conflict_analysis"]["has_bank_conflicts"].as_bool().unwrap());
}

#[tokio::test]
async fn test_triton_kernel_fuser_tool_kernel_synthesis() {
    let tool = TritonKernelFuserTool::new();

    // 1. Synthesize fused GEMM kernel
    let args_gemm = json!({
        "action": "synthesize_triton_kernel",
        "operation": "gemm",
        "block_size_m": 128,
        "block_size_n": 128,
        "block_size_k": 32
    });

    let res_gemm: Value = serde_json::from_str(&tool.execute(args_gemm).await.unwrap()).unwrap();
    let kernel_code = res_gemm["synthesized_triton_kernel"].as_str().unwrap();
    assert!(kernel_code.contains("@triton.jit"), "Must be a Triton JIT kernel");
    assert!(kernel_code.contains("tl.dot(a, b)"), "Must use tl.dot tensor core intrinsics");
    assert!(kernel_code.contains("BLOCK_SIZE_M: tl.constexpr = 128"));

    // 2. Synthesize FlashAttention kernel
    let args_flash = json!({
        "action": "synthesize_triton_kernel",
        "operation": "flash_attention",
        "block_size_m": 64,
        "block_size_n": 64,
        "block_size_k": 64
    });

    let res_flash: Value = serde_json::from_str(&tool.execute(args_flash).await.unwrap()).unwrap();
    let flash_code = res_flash["synthesized_triton_kernel"].as_str().unwrap();
    assert!(flash_code.contains("fused_flash_attention_kernel"));
    assert!(flash_code.contains("m_ij = tl.maximum(m_i, tl.max(qk, 1))"), "Must implement online softmax max tracking");
    assert!(flash_code.contains("l_i = l_i * alpha + l_ij"), "Must implement online softmax denominator renormalization");
}

// =========================================================================
// Pillar 6: ToolRegistry Registration & Re-export Tests
// =========================================================================

#[test]
fn test_tool_registry_with_builtins_contains_ecosystem_tools() {
    let registry = ToolRegistry::with_builtins();

    assert!(registry.contains("ebpf_telemetry_tracer"), "ToolRegistry::with_builtins() must contain ebpf_telemetry_tracer");
    assert!(registry.contains("kani_formal_verifier"), "ToolRegistry::with_builtins() must contain kani_formal_verifier");
    assert!(registry.contains("triton_kernel_fuser"), "ToolRegistry::with_builtins() must contain triton_kernel_fuser");

    let ebpf = registry.get("ebpf_telemetry_tracer").expect("ebpf tool must be retrievable");
    assert_eq!(ebpf.name(), "ebpf_telemetry_tracer");

    let kani = registry.get("kani_formal_verifier").expect("kani tool must be retrievable");
    assert_eq!(kani.name(), "kani_formal_verifier");

    let triton = registry.get("triton_kernel_fuser").expect("triton tool must be retrievable");
    assert_eq!(triton.name(), "triton_kernel_fuser");
}

#[test]
fn test_tool_registry_with_builtins_in_dir_contains_ecosystem_tools() {
    let dir = std::path::PathBuf::from("/home/dyna/TGS Projects/tagisan");
    let registry = ToolRegistry::with_builtins_in_dir(dir);

    assert!(registry.contains("ebpf_telemetry_tracer"), "with_builtins_in_dir must contain ebpf_telemetry_tracer");
    assert!(registry.contains("kani_formal_verifier"), "with_builtins_in_dir must contain kani_formal_verifier");
    assert!(registry.contains("triton_kernel_fuser"), "with_builtins_in_dir must contain triton_kernel_fuser");
}
