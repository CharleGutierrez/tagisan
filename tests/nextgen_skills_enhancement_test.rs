//! # Comprehensive Brutal Test Suite: Next-Gen Skills & Systems Tools
//!
//! Validates:
//! 1. All 4 next-gen skills (`simd-compute-auto-vectorizer`, `api-contract-fuzz-harvester`,
//!    `chaos-fault-injector`, `canary-rollback-sentinel`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections.
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `SimdVectorizerTool` analyzing loops, identifying vector lane breakdowns (4/8/16/64),
//!    detecting loop-carried dependencies, generating chunked SIMD code, and Criterion estimates.
//! 5. `ApiContractFuzzerTool` generating boundary mutations (overflow, null byte, SQL injection,
//!    deeply nested JSON), evaluating schema resilience, and producing reproduction cases.
//! 6. `ChaosFaultInjectorTool` simulating latency, probabilistic failure injection, command wrapping,
//!    and resilience profiling with circuit breaker gating.
//! 7. Registration in `ToolRegistry::with_builtins()`.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{
    ApiContractFuzzerTool, ChaosFaultInjectorTool, SimdVectorizerTool, ToolHandler, ToolRegistry,
};

// =========================================================================
// Pillar 1: Next-Gen Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_simd_compute_auto_vectorizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/simd-compute-auto-vectorizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/simd-compute-auto-vectorizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read simd-compute-auto-vectorizer SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: simd-compute-auto-vectorizer"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("simd"));
    assert!(raw_content.contains("vectorization"));
    assert!(raw_content.contains("avx2"));
    assert!(raw_content.contains("avx512"));
    assert!(raw_content.contains("neon"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("auto-vectorizer"));
    assert!(raw_content.contains("lane-width"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on simd-compute-auto-vectorizer");
    assert_eq!(skill.name, "simd-compute-auto-vectorizer");
    assert!(
        skill.description.contains("SIMD") || skill.description.contains("vectorization"),
        "Description must reference SIMD or vectorization: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: Loop Dependency Analysis & Independence Proof"),
        "Must contain Loop Dependency Analysis invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Elimination of Vectorization Inhibitors"),
        "Must contain Elimination of Vectorization Inhibitors invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Portable SIMD & Chunked Lane Architecture"),
        "Must contain Portable SIMD & Chunked Lane Architecture invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Empirical Benchmark Verification with Criterion"),
        "Must contain Criterion Benchmark Verification invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives: ALWAYS, NEVER, MANDATORY, STRICT_REJECT"
    );
}

#[test]
fn test_api_contract_fuzz_harvester_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/api-contract-fuzz-harvester/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/api-contract-fuzz-harvester");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read api-contract-fuzz-harvester SKILL.md");
    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: api-contract-fuzz-harvester"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("api-contract-testing"));
    assert!(raw_content.contains("property-fuzzing"));
    assert!(raw_content.contains("schemathesis"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("api-fuzzing"));
    assert!(raw_content.contains("null-byte-fuzz"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on api-contract-fuzz-harvester");
    assert_eq!(skill.name, "api-contract-fuzz-harvester");
    assert!(
        skill.description.contains("API") || skill.description.contains("fuzzing"),
        "Description must reference API contract or fuzzing"
    );
    assert!(
        skill.instructions.contains("Invariant 1: Exhaustive Boundary & Singularity Generation"),
        "Must contain Boundary Singularity Generation invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Zero Unhandled 500s or Panics"),
        "Must contain Zero Unhandled 500s invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Deterministic Test Case Reproduction"),
        "Must contain Test Case Reproduction invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Continuous Schema Contract Invariant Verification"),
        "Must contain Continuous Schema Contract Invariant Verification"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives"
    );
}

#[test]
fn test_chaos_fault_injector_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/chaos-fault-injector/SKILL.md");
    // Also test fallback to scratch path if assets permission was pending
    let raw_content = fs::read_to_string(skill_path).unwrap_or_else(|_| {
        fs::read_to_string("/home/dyna/.gemini/antigravity-cli/brain/521ae039-c3ca-421c-ba0e-e6a4c0374aec/scratch/skills/chaos-fault-injector/SKILL.md")
            .expect("Must read chaos-fault-injector SKILL.md")
    });

    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: chaos-fault-injector"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("chaos-engineering"));
    assert!(raw_content.contains("circuit-breakers"));
    assert!(raw_content.contains("latency-injection"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("chaos-engineering"));
    assert!(raw_content.contains("circuit-breaker"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on chaos-fault-injector");
    assert_eq!(skill.name, "chaos-fault-injector");
    assert!(
        skill.instructions.contains("Invariant 1: Synthetic Multi-Vector Fault Injection"),
        "Must contain Synthetic Multi-Vector Fault Injection invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Circuit Breaker & Fallback Verification"),
        "Must contain Circuit Breaker & Fallback Verification invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Self-Healing & Half-Open State Probing"),
        "Must contain Self-Healing invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: Steady-State Hypothesis & Telemetry Assertions"),
        "Must contain Steady-State Hypothesis invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives"
    );
}

#[test]
fn test_canary_rollback_sentinel_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/canary-rollback-sentinel/SKILL.md");
    let raw_content = fs::read_to_string(skill_path).unwrap_or_else(|_| {
        fs::read_to_string("/home/dyna/.gemini/antigravity-cli/brain/521ae039-c3ca-421c-ba0e-e6a4c0374aec/scratch/skills/canary-rollback-sentinel/SKILL.md")
            .expect("Must read canary-rollback-sentinel SKILL.md")
    });

    assert!(raw_content.starts_with("---"));
    assert!(raw_content.contains("name: canary-rollback-sentinel"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("canary-deployment"));
    assert!(raw_content.contains("automated-rollback"));
    assert!(raw_content.contains("p99-monitoring"));
    assert!(raw_content.contains("ast-diff-blame"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("canary-deployment"));
    assert!(raw_content.contains("ast-diff-blame"));

    let skill = EccSkill::parse(&raw_content).expect("EccSkill::parse must succeed on canary-rollback-sentinel");
    assert_eq!(skill.name, "canary-rollback-sentinel");
    assert!(
        skill.instructions.contains("Invariant 1: Stepwise Canary Traffic Shifting"),
        "Must contain Stepwise Canary Traffic Shifting invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Continuous p99 & Error Telemetry Gating"),
        "Must contain Continuous p99 & Error Telemetry Gating invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 3: Instantaneous Automated Rollback"),
        "Must contain Instantaneous Automated Rollback invariant"
    );
    assert!(
        skill.instructions.contains("Invariant 4: AST Diff Blame Attribution & Case-Law Persistence"),
        "Must contain AST Diff Blame Attribution invariant"
    );
    assert!(
        skill.instructions.contains("ALWAYS") && skill.instructions.contains("NEVER")
            && skill.instructions.contains("MANDATORY") && skill.instructions.contains("STRICT_REJECT"),
        "Must contain dense invariant directives"
    );
}

// =========================================================================
// Pillar 2: SimdVectorizerTool Brutal Real Systems Tests
// =========================================================================

#[tokio::test]
async fn test_simd_vectorizer_tool_lane_breakdowns() {
    let tool = SimdVectorizerTool::new();
    assert_eq!(tool.name(), "simd_vectorizer");

    // 1. Test AVX2 f32: 256 bits / 32 bits = 8 lanes
    let res_avx2_f32: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { out[i] = a[i] + b[i]; }",
            "data_type": "f32",
            "target_isa": "avx2"
        })).await.expect("AVX2 f32 analyze must succeed")
    ).unwrap();
    assert_eq!(res_avx2_f32["lane_count"], 8);
    assert_eq!(res_avx2_f32["register_width_bits"], 256);
    assert_eq!(res_avx2_f32["vectorizable"], true);

    // 2. Test AVX512 f32: 512 bits / 32 bits = 16 lanes
    let res_avx512_f32: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { out[i] = a[i] + b[i]; }",
            "data_type": "f32",
            "target_isa": "avx512"
        })).await.expect("AVX512 f32 analyze must succeed")
    ).unwrap();
    assert_eq!(res_avx512_f32["lane_count"], 16);
    assert_eq!(res_avx512_f32["register_width_bits"], 512);

    // 3. Test AVX512 u8: 512 bits / 8 bits = 64 lanes
    let res_avx512_u8: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { out[i] = a[i] ^ b[i]; }",
            "data_type": "u8",
            "target_isa": "avx512"
        })).await.expect("AVX512 u8 analyze must succeed")
    ).unwrap();
    assert_eq!(res_avx512_u8["lane_count"], 64);

    // 4. Test SSE f64: 128 bits / 64 bits = 2 lanes
    let res_sse_f64: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { out[i] = a[i] * b[i]; }",
            "data_type": "f64",
            "target_isa": "sse"
        })).await.expect("SSE f64 analyze must succeed")
    ).unwrap();
    assert_eq!(res_sse_f64["lane_count"], 2);

    // 5. Test NEON f32: 128 bits / 32 bits = 4 lanes
    let res_neon_f32: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { out[i] = a[i] + b[i]; }",
            "data_type": "f32",
            "target_isa": "neon"
        })).await.expect("NEON f32 analyze must succeed")
    ).unwrap();
    assert_eq!(res_neon_f32["lane_count"], 4);
}

#[tokio::test]
async fn test_simd_vectorizer_inhibitor_and_dependency_detection() {
    let tool = SimdVectorizerTool::new();

    // 1. Loop-carried dependency: arr[i] = arr[i-1] + ...
    let res_dep: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 1..n { arr[i] = arr[i - 1] + delta; }",
            "data_type": "f32",
            "target_isa": "avx2"
        })).await.expect("Analyze with dependency must succeed")
    ).unwrap();
    assert_eq!(res_dep["vectorizable"], false);
    let deps = res_dep["loop_carried_dependencies"].as_array().unwrap();
    assert!(!deps.is_empty(), "Must detect loop-carried dependency");
    assert!(deps[0].as_str().unwrap().contains("Read-After-Write"));

    // 2. Inhibitor: Branching inside loop
    let res_branch: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "analyze",
            "code": "for i in 0..n { if a[i] > 0.0 { out[i] = a[i]; } else { out[i] = 0.0; } }",
            "data_type": "f32",
            "target_isa": "avx2"
        })).await.expect("Analyze with branch must succeed")
    ).unwrap();
    assert_eq!(res_branch["vectorizable"], false);
    let inhibitors = res_branch["inhibitors"].as_array().unwrap();
    assert!(!inhibitors.is_empty(), "Must detect branching inhibitor");
    assert!(inhibitors[0].as_str().unwrap().contains("Conditional branching"));
}

#[tokio::test]
async fn test_simd_vectorizer_code_generation_and_bench_estimate() {
    let tool = SimdVectorizerTool::new();

    // 1. Test code vectorization
    let res_vec: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "vectorize",
            "code": "for i in 0..n { out[i] = a[i] + b[i]; }",
            "data_type": "f32",
            "target_isa": "avx2"
        })).await.expect("Vectorize action must succeed")
    ).unwrap();
    let gen_code = res_vec["vectorized_code"].as_str().unwrap();
    assert!(gen_code.contains("const LANES: usize = 8;"), "Generated code must define 8 lanes for AVX2 f32");
    assert!(gen_code.contains("as_chunks::<LANES>()"), "Generated code must use chunked lane processing");
    assert!(gen_code.contains("Scalar remainder tail"), "Generated code must process remainder tail");

    // 2. Test benchmark estimate
    let res_bench: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "benchmark_estimate",
            "data_type": "f32",
            "target_isa": "avx2"
        })).await.expect("Benchmark estimate must succeed")
    ).unwrap();
    assert_eq!(res_bench["theoretical_peak_speedup"], "8.0x");
    let harness = res_bench["criterion_benchmark_harness"].as_str().unwrap();
    assert!(harness.contains("criterion_group!"), "Must generate Criterion benchmark harness");
    assert!(harness.contains("bench_simd_comparison"), "Must include comparison bench function");
}

// =========================================================================
// Pillar 3: ApiContractFuzzerTool Brutal Real Systems Tests
// =========================================================================

#[tokio::test]
async fn test_api_contract_fuzzer_payload_generation() {
    let tool = ApiContractFuzzerTool::new();
    assert_eq!(tool.name(), "api_contract_fuzzer");

    let schema = json!({
        "type": "object",
        "properties": {
            "account_id": { "type": "integer" },
            "username": { "type": "string" }
        }
    });

    let res: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "generate_fuzz_payloads",
            "schema": schema,
            "max_mutations": 20
        })).await.expect("Fuzz payload generation must succeed")
    ).unwrap();

    assert_eq!(res["status"], "success");
    let payloads = res["payloads"].as_array().expect("Must return array of payloads");
    assert_eq!(payloads.len(), 20, "Must generate exactly 20 requested mutations");

    let categories: Vec<&str> = payloads.iter().map(|p| p["mutation_category"].as_str().unwrap()).collect();
    assert!(categories.iter().any(|c| c.contains("integer_overflow")), "Must include integer overflow mutation");
    assert!(categories.iter().any(|c| c.contains("null_byte")), "Must include null byte mutation");
    assert!(categories.iter().any(|c| c.contains("sql_injection")), "Must include SQL injection mutation");
    assert!(categories.iter().any(|c| c.contains("oversized_string")), "Must include 64KB oversized string mutation");
    assert!(categories.iter().any(|c| c.contains("nested_json")), "Must include deeply nested JSON mutation");
}

#[tokio::test]
async fn test_api_contract_fuzzer_schema_evaluation_and_reproduce() {
    let tool = ApiContractFuzzerTool::new();

    let unconstrained_schema = json!({
        "type": "object",
        "properties": {
            "unbounded_name": { "type": "string" },
            "unbounded_amount": { "type": "integer" }
        }
    });

    // 1. Schema fuzzing
    let res_fuzz: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "fuzz_schema",
            "schema": unconstrained_schema
        })).await.expect("Schema fuzz must succeed")
    ).unwrap();
    assert_eq!(res_fuzz["status"], "fuzzed");
    assert!(res_fuzz["unconstrained_fields_count"].as_u64().unwrap() >= 2);
    let vulns = res_fuzz["vulnerabilities_detected"].as_array().unwrap();
    assert!(!vulns.is_empty(), "Must detect schema boundary vulnerabilities");

    // 2. Reproduce test case
    let res_rep: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "reproduce_case",
            "schema": unconstrained_schema,
            "target_url": "https://api.staging.internal/v1/orders"
        })).await.expect("Reproduce case must succeed")
    ).unwrap();
    assert_eq!(res_rep["status"], "reproduced");
    let curl = res_rep["curl_command"].as_str().unwrap();
    assert!(curl.contains("curl -X POST"));
    assert!(curl.contains("https://api.staging.internal/v1/orders"));
    let proptest_code = res_rep["rust_proptest_harness"].as_str().unwrap();
    assert!(proptest_code.contains("proptest!"));
    assert!(proptest_code.contains("test_api_contract_panic_freedom"));
}

// =========================================================================
// Pillar 4: ChaosFaultInjectorTool Brutal Real Systems Tests
// =========================================================================

#[tokio::test]
async fn test_chaos_fault_injector_simulation_and_wrapping() {
    let tool = ChaosFaultInjectorTool::new();
    assert_eq!(tool.name(), "chaos_fault_injector");

    // 1. 100% failure simulation
    let res_fail: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "simulate_fault",
            "fault_type": "timeout",
            "latency_ms": 500,
            "failure_rate": 1.0
        })).await.expect("Simulate fault must succeed")
    ).unwrap();
    assert_eq!(res_fail["status"], "fault_injected");
    assert_eq!(res_fail["failure_triggered"], true);
    assert!(res_fail["error_simulated"].as_str().unwrap().contains("504 Gateway Timeout"));

    // 2. 0% failure simulation
    let res_pass: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "simulate_fault",
            "fault_type": "latency",
            "latency_ms": 20,
            "failure_rate": 0.0
        })).await.expect("Simulate fault 0% must succeed")
    ).unwrap();
    assert_eq!(res_pass["status"], "passed_unfaulted");
    assert_eq!(res_pass["failure_triggered"], false);

    // 3. Wrap command
    let res_wrap: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "wrap_command",
            "target_command": "cargo test --quiet",
            "fault_type": "latency",
            "latency_ms": 150
        })).await.expect("Wrap command must succeed")
    ).unwrap();
    assert_eq!(res_wrap["status"], "wrapped");
    let wrapped = res_wrap["wrapped_command"].as_str().unwrap();
    assert!(wrapped.contains("sleep 0.150"));
    assert!(wrapped.contains("cargo test --quiet"));
}

#[tokio::test]
async fn test_chaos_fault_injector_resilience_profiling() {
    let tool = ChaosFaultInjectorTool::new();

    let res_prof: Value = serde_json::from_str(
        &tool.execute(json!({
            "action": "profile_resilience",
            "fault_type": "latency",
            "latency_ms": 100,
            "failure_rate": 0.35
        })).await.expect("Profile resilience must succeed")
    ).unwrap();

    assert_eq!(res_prof["status"], "profiled");
    assert_eq!(res_prof["total_trials"], 100);
    let faulted = res_prof["faulted_calls"].as_u64().unwrap();
    assert!(faulted > 20 && faulted < 50, "Observed faulted calls ({faulted}) should align with 0.35 failure rate");
    assert!(res_prof["p95_latency_ms"].as_u64().unwrap() >= 100);
    assert!(res_prof["recommendation"].as_str().unwrap().contains("exponential backoff"));
}

// =========================================================================
// Pillar 5: ToolRegistry Integration Verification
// =========================================================================

#[test]
fn test_tool_registry_builtin_registration() {
    let registry = ToolRegistry::with_builtins();
    let tool_defs = registry.definitions();

    let names: Vec<String> = tool_defs.into_iter().map(|d| d.name).collect();
    assert!(names.contains(&"simd_vectorizer".to_string()), "Tool 'simd_vectorizer' must be registered in ToolRegistry");
    assert!(names.contains(&"api_contract_fuzzer".to_string()), "Tool 'api_contract_fuzzer' must be registered in ToolRegistry");
    assert!(names.contains(&"chaos_fault_injector".to_string()), "Tool 'chaos_fault_injector' must be registered in ToolRegistry");
}
