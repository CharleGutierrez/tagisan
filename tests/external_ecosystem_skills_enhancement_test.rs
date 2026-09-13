//! # Comprehensive Brutal Test Suite: External Systems & Ecosystem Integrations
//!
//! Validates:
//! 1. All 4 Wave 7 external systems skills (`z3-smt-symbolic-constraint-solver`,
//!    `rr-time-travel-deterministic-debugger`, `qemu-baremetal-firmware-emulator`,
//!    `tla-consensus-formal-model-checker`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections
//!    (`ALWAYS`, `NEVER`, `MANDATORY`, `STRICT_REJECT`).
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `Z3SmtSolverTool` SMT-LIB2 script synthesis, constraint solving, and equivalence verification.
//! 5. `RrTimeTravelDebuggerTool` recording session planning, GDB script synthesis, and Heisenbug bisection.
//! 6. `QemuBaremetalEmulatorTool` machine config generation, memory map validation, and harness synthesis.
//! 7. `TlaConsensusCheckerTool` TLA+ spec synthesis, CFG generation, and state space analysis.
//! 8. Registration in `ToolRegistry::with_builtins()` and `ToolRegistry::with_builtins_in_dir()`.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{
    QemuBaremetalEmulatorTool, RrTimeTravelDebuggerTool, TlaConsensusCheckerTool, ToolHandler,
    ToolRegistry, Z3SmtSolverTool,
};

// =========================================================================
// Pillar 1: Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_z3_smt_symbolic_constraint_solver_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/z3-smt-symbolic-constraint-solver/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/z3-smt-symbolic-constraint-solver");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read z3-smt-symbolic-constraint-solver SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: z3-smt-symbolic-constraint-solver"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("smt"));
    assert!(raw_content.contains("z3"));
    assert!(raw_content.contains("cvc5"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("symbolic execution"));
    assert!(raw_content.contains("constraint solver"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "z3-smt-symbolic-constraint-solver");
    assert!(!skill.description.is_empty());

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"), "Must contain ALWAYS directive");
    assert!(instructions.contains("NEVER"), "Must contain NEVER directive");
    assert!(instructions.contains("MANDATORY"), "Must contain MANDATORY directive");
    assert!(instructions.contains("STRICT_REJECT"), "Must contain STRICT_REJECT directive");
    assert!(instructions.contains("QF_BV"), "Must reference QF_BV bitvector logic");
    assert!(instructions.contains("check-sat"), "Must mandate check-sat");
    assert!(instructions.contains("unsat"), "Must reference unsatisfiability proofs");
}

#[test]
fn test_rr_time_travel_deterministic_debugger_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/rr-time-travel-deterministic-debugger/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/rr-time-travel-deterministic-debugger");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read rr-time-travel-deterministic-debugger SKILL.md");
    assert!(raw_content.contains("name: rr-time-travel-deterministic-debugger"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("rr"));
    assert!(raw_content.contains("time-travel"));
    assert!(raw_content.contains("perf-event"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("reverse execution"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "rr-time-travel-deterministic-debugger");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("taskset"));
    assert!(instructions.contains("ASLR"));
    assert!(instructions.contains("perf_event_paranoid"));
}

#[test]
fn test_qemu_baremetal_firmware_emulator_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/qemu-baremetal-firmware-emulator/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/qemu-baremetal-firmware-emulator");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read qemu-baremetal-firmware-emulator SKILL.md");
    assert!(raw_content.contains("name: qemu-baremetal-firmware-emulator"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("qemu"));
    assert!(raw_content.contains("baremetal"));
    assert!(raw_content.contains("riscv"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("firmware emulation"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "qemu-baremetal-firmware-emulator");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("linker script"));
    assert!(instructions.contains("-nographic"));
}

#[test]
fn test_tla_consensus_formal_model_checker_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/tla-consensus-formal-model-checker/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/tla-consensus-formal-model-checker");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read tla-consensus-formal-model-checker SKILL.md");
    assert!(raw_content.contains("name: tla-consensus-formal-model-checker"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("tla-plus") || raw_content.contains("tlc"));
    assert!(raw_content.contains("consensus"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("raft"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "tla-consensus-formal-model-checker");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("TypeOK"));
    assert!(instructions.contains("ElectionSafety"));
}

// =========================================================================
// Pillar 2: Discovery & Alias Resolution
// =========================================================================

#[test]
fn test_all_built_in_skills_contains_all_four_wave7_skills() {
    let skills = all_built_in_skills();
    let skill_names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    assert!(skill_names.contains(&"z3-smt-symbolic-constraint-solver"), "Must contain z3-smt-symbolic-constraint-solver");
    assert!(skill_names.contains(&"rr-time-travel-deterministic-debugger"), "Must contain rr-time-travel-deterministic-debugger");
    assert!(skill_names.contains(&"qemu-baremetal-firmware-emulator"), "Must contain qemu-baremetal-firmware-emulator");
    assert!(skill_names.contains(&"tla-consensus-formal-model-checker"), "Must contain tla-consensus-formal-model-checker");
}

#[test]
fn test_find_built_in_skill_wave7_aliases() {
    // 1. Z3 SMT
    let s1_direct = find_built_in_skill("z3-smt-symbolic-constraint-solver").expect("Direct lookup must succeed");
    assert_eq!(s1_direct.name, "z3-smt-symbolic-constraint-solver");
    let s1_alias_z3 = find_built_in_skill("z3").expect("Alias 'z3' must resolve");
    assert_eq!(s1_alias_z3.name, "z3-smt-symbolic-constraint-solver");
    let s1_alias_cvc5 = find_built_in_skill("cvc5").expect("Alias 'cvc5' must resolve");
    assert_eq!(s1_alias_cvc5.name, "z3-smt-symbolic-constraint-solver");
    let s1_alias_smt = find_built_in_skill("smt").expect("Alias 'smt' must resolve");
    assert_eq!(s1_alias_smt.name, "z3-smt-symbolic-constraint-solver");

    // 2. rr debugger
    let s2_direct = find_built_in_skill("rr-time-travel-deterministic-debugger").expect("Direct lookup must succeed");
    assert_eq!(s2_direct.name, "rr-time-travel-deterministic-debugger");
    let s2_alias_rr = find_built_in_skill("rr").expect("Alias 'rr' must resolve");
    assert_eq!(s2_alias_rr.name, "rr-time-travel-deterministic-debugger");
    let s2_alias_tt = find_built_in_skill("time-travel").expect("Alias 'time-travel' must resolve");
    assert_eq!(s2_alias_tt.name, "rr-time-travel-deterministic-debugger");

    // 3. QEMU emulator
    let s3_direct = find_built_in_skill("qemu-baremetal-firmware-emulator").expect("Direct lookup must succeed");
    assert_eq!(s3_direct.name, "qemu-baremetal-firmware-emulator");
    let s3_alias_qemu = find_built_in_skill("qemu").expect("Alias 'qemu' must resolve");
    assert_eq!(s3_alias_qemu.name, "qemu-baremetal-firmware-emulator");
    let s3_alias_bm = find_built_in_skill("baremetal").expect("Alias 'baremetal' must resolve");
    assert_eq!(s3_alias_bm.name, "qemu-baremetal-firmware-emulator");

    // 4. TLA+ consensus
    let s4_direct = find_built_in_skill("tla-consensus-formal-model-checker").expect("Direct lookup must succeed");
    assert_eq!(s4_direct.name, "tla-consensus-formal-model-checker");
    let s4_alias_tla = find_built_in_skill("tla").expect("Alias 'tla' must resolve");
    assert_eq!(s4_alias_tla.name, "tla-consensus-formal-model-checker");
    let s4_alias_tlc = find_built_in_skill("tlc").expect("Alias 'tlc' must resolve");
    assert_eq!(s4_alias_tlc.name, "tla-consensus-formal-model-checker");
}

// =========================================================================
// Pillar 3: Z3SmtSolverTool Execution Tests
// =========================================================================

#[tokio::test]
async fn test_z3_smt_solver_tool_synthesize_smt_lib() {
    let tool = Z3SmtSolverTool::new();
    assert_eq!(tool.name(), "z3_smt_solver");

    let args = json!({
        "action": "synthesize_smt_lib",
        "logic": "QF_BV",
        "declarations": [
            { "name": "x", "sort": "(_ BitVec 32)" },
            { "name": "y", "sort": "(_ BitVec 32)" }
        ],
        "assertions": [
            "(bvuge x (_ bv10 32))",
            "(bvule x (_ bv100 32))",
            "(bvugt (bvadd x y) (_ bv50 32))"
        ],
        "check_sat": true,
        "get_model": true
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["logic"], "QF_BV");
    assert_eq!(res["variable_count"], 2);
    assert_eq!(res["assertion_count"], 3);

    let script = res["smt_lib_script"].as_str().unwrap();
    assert!(script.contains("(set-logic QF_BV)"));
    assert!(script.contains("(declare-const x (_ BitVec 32))"));
    assert!(script.contains("(declare-const y (_ BitVec 32))"));
    assert!(script.contains("(assert (bvuge x (_ bv10 32)))"));
    assert!(script.contains("(check-sat)"));
    assert!(script.contains("(get-model)"));
}

#[tokio::test]
async fn test_z3_smt_solver_tool_solve_constraints_sat_and_unsat() {
    let tool = Z3SmtSolverTool::new();

    // 1. Satisfiable constraint system
    let args_sat = json!({
        "action": "solve_constraints",
        "variables": [
            { "name": "x", "min": 0, "max": 50 },
            { "name": "y", "min": 0, "max": 50 }
        ],
        "constraints": [
            "x + y == 42",
            "x > 20",
            "y > 10"
        ]
    });

    let res_str_sat = tool.execute(args_sat).await.expect("Execution must succeed");
    let res_sat: Value = serde_json::from_str(&res_str_sat).expect("Valid JSON");

    assert_eq!(res_sat["status"], "SUCCESS");
    assert_eq!(res_sat["verdict"], "SAT");
    assert!(res_sat["satisfiable"].as_bool().unwrap());

    let model = &res_sat["model"];
    let x = model["x"].as_i64().unwrap();
    let y = model["y"].as_i64().unwrap();
    assert_eq!(x + y, 42);
    assert!(x > 20);
    assert!(y > 10);

    // 2. Unsatisfiable constraint system
    let args_unsat = json!({
        "action": "solve_constraints",
        "variables": [
            { "name": "x", "min": 0, "max": 100 }
        ],
        "constraints": [
            "x == 10",
            "x == 20"
        ]
    });

    let res_str_unsat = tool.execute(args_unsat).await.expect("Execution must succeed");
    let res_unsat: Value = serde_json::from_str(&res_str_unsat).expect("Valid JSON");

    assert_eq!(res_unsat["status"], "SUCCESS");
    assert_eq!(res_unsat["verdict"], "UNSAT");
    assert!(!res_unsat["satisfiable"].as_bool().unwrap());
    assert!(res_unsat["model"].is_null());
}

#[tokio::test]
async fn test_z3_smt_solver_tool_verify_equivalence() {
    let tool = Z3SmtSolverTool::new();

    // 1. Proved equivalent: x << 1 vs x * 2
    let args_eq = json!({
        "action": "verify_equivalence",
        "expression_a": "x << 1",
        "expression_b": "x * 2",
        "variables": ["x"],
        "bit_width": 32
    });

    let res_str_eq = tool.execute(args_eq).await.expect("Execution must succeed");
    let res_eq: Value = serde_json::from_str(&res_str_eq).expect("Valid JSON");

    assert_eq!(res_eq["status"], "SUCCESS");
    assert_eq!(res_eq["proof_verdict"], "PROVED_EQUIVALENT");
    assert!(res_eq["equivalent"].as_bool().unwrap());
    assert!(!res_eq["negation_satisfiable"].as_bool().unwrap());
    assert!(res_eq["counterexample"].is_null());
    assert!(res_eq["smt_lib_negation_proof"].as_str().unwrap().contains("(assert (distinct x << 1 x * 2))"));

    // 2. Disproved inequivalent: x + 1 vs x + 2
    let args_ineq = json!({
        "action": "verify_equivalence",
        "expression_a": "x + 1",
        "expression_b": "x + 2",
        "variables": ["x"],
        "bit_width": 32
    });

    let res_str_ineq = tool.execute(args_ineq).await.expect("Execution must succeed");
    let res_ineq: Value = serde_json::from_str(&res_str_ineq).expect("Valid JSON");

    assert_eq!(res_ineq["status"], "SUCCESS");
    assert_eq!(res_ineq["proof_verdict"], "DISPROVED_INEQUIVALENT");
    assert!(!res_ineq["equivalent"].as_bool().unwrap());
    assert!(res_ineq["negation_satisfiable"].as_bool().unwrap());
    assert!(!res_ineq["counterexample"].is_null());
}

// =========================================================================
// Pillar 4: RrTimeTravelDebuggerTool Execution Tests
// =========================================================================

#[tokio::test]
async fn test_rr_time_travel_debugger_tool_plan_recording_session() {
    let tool = RrTimeTravelDebuggerTool::new();
    assert_eq!(tool.name(), "rr_time_travel_debugger");

    let args = json!({
        "action": "plan_recording_session",
        "binary_path": "/usr/local/bin/server_app",
        "arguments": ["--workers", "1", "--deterministic"],
        "cpu_core": 0,
        "disable_aslr": true
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["pinned_cpu_core"], 0);
    assert!(res["aslr_disabled"].as_bool().unwrap());

    let rr_cmd = res["rr_record_command"].as_str().unwrap();
    assert!(rr_cmd.contains("taskset -c 0"));
    assert!(rr_cmd.contains("setarch x86_64 -R"));
    assert!(rr_cmd.contains("rr record --num-cores=1"));
    assert!(rr_cmd.contains("/usr/local/bin/server_app"));

    assert_eq!(res["prerequisites"]["kernel.perf_event_paranoid"], "<= 1");
    assert_eq!(res["prerequisites"]["kernel.randomize_va_space"], "0");
}

#[tokio::test]
async fn test_rr_time_travel_debugger_tool_synthesize_gdb_script() {
    let tool = RrTimeTravelDebuggerTool::new();

    let args = json!({
        "action": "synthesize_gdb_script",
        "trace_path": "~/.local/share/rr/server_app-0",
        "breakpoints": ["crash_handler", "faulty_subroutine"],
        "watchpoints": ["*0x7fffffffe048"],
        "reverse_commands": ["reverse-continue", "print *buffer", "backtrace"],
        "event_target": 12450
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["trace_path"], "~/.local/share/rr/server_app-0");

    let script = res["gdb_script"].as_str().unwrap();
    assert!(script.contains("target extended-remote :1234"));
    assert!(script.contains("rr replay -u 12450"));
    assert!(script.contains("break crash_handler"));
    assert!(script.contains("watch -l *0x7fffffffe048"));
    assert!(script.contains("reverse-continue"));
}

#[tokio::test]
async fn test_rr_time_travel_debugger_tool_bisect_heisenbug() {
    let tool = RrTimeTravelDebuggerTool::new();

    let args = json!({
        "action": "bisect_heisenbug",
        "start_event_tick": 1000,
        "end_event_tick": 10000,
        "target_address": "0x7fffffffe048",
        "corrupted_value": "0xDEADBEEF",
        "expected_value": "0x00000000"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["total_event_span"], 9000);
    // ceil(log2(9000)) = 14 steps
    assert_eq!(res["bisection_steps_required"], 14);

    let schedule = res["bisection_schedule"].as_array().unwrap();
    assert_eq!(schedule.len(), 14);
    assert_eq!(schedule[0]["step_number"], 1);
    assert_eq!(schedule[0]["target_event_tick"], 5500); // 1000 + 4500

    let rr_cmds = res["rr_commands"].as_array().unwrap();
    assert!(rr_cmds[0].as_str().unwrap().contains("rr replay -u 5500"));
}

// =========================================================================
// Pillar 5: QemuBaremetalEmulatorTool Execution Tests
// =========================================================================

#[tokio::test]
async fn test_qemu_baremetal_emulator_tool_generate_machine_config_riscv_and_arm() {
    let tool = QemuBaremetalEmulatorTool::new();
    assert_eq!(tool.name(), "qemu_baremetal_emulator");

    // 1. RISC-V 64
    let args_riscv = json!({
        "action": "generate_machine_config",
        "target_arch": "riscv64",
        "kernel_image": "target/firmware.elf",
        "memory_mb": 128
    });

    let res_str = tool.execute(args_riscv).await.expect("Execution must succeed");
    let res_rv: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res_rv["status"], "SUCCESS");
    assert_eq!(res_rv["target_arch"], "riscv64");
    assert_eq!(res_rv["qemu_binary"], "qemu-system-riscv64");
    assert_eq!(res_rv["machine_type"], "virt");
    assert_eq!(res_rv["cpu_model"], "rv64");
    assert!(res_rv["qemu_command"].as_str().unwrap().contains("-bios none"));
    assert!(res_rv["qemu_command"].as_str().unwrap().contains("-nographic"));

    // 2. ARM Cortex-M4
    let args_arm = json!({
        "action": "generate_machine_config",
        "target_arch": "arm-cortex-m4",
        "kernel_image": "target/thumbv7em.elf"
    });

    let res_str_arm = tool.execute(args_arm).await.expect("Execution must succeed");
    let res_arm: Value = serde_json::from_str(&res_str_arm).expect("Valid JSON");

    assert_eq!(res_arm["qemu_binary"], "qemu-system-arm");
    assert_eq!(res_arm["machine_type"], "lm3s6965evb");
    assert_eq!(res_arm["cpu_model"], "cortex-m4");
}

#[tokio::test]
async fn test_qemu_baremetal_emulator_tool_generate_machine_config_with_gdb() {
    let tool = QemuBaremetalEmulatorTool::new();

    let args = json!({
        "action": "generate_machine_config",
        "target_arch": "riscv64",
        "kernel_image": "target/firmware.elf",
        "enable_gdb_server": true
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert!(res["gdb_server_enabled"].as_bool().unwrap());
    assert_eq!(res["gdb_connection_string"], "target remote :1234");
    assert!(res["qemu_command"].as_str().unwrap().contains(" -s -S"));
}

#[tokio::test]
async fn test_qemu_baremetal_emulator_tool_validate_memory_map_valid_and_collision() {
    let tool = QemuBaremetalEmulatorTool::new();

    let hw_regions = json!([
        { "name": "FLASH", "origin": "0x08000000", "length": "0x00100000" },
        { "name": "SRAM", "origin": "0x20000000", "length": "0x00020000" }
    ]);

    // 1. Valid Linker Layout
    let valid_linker = json!([
        { "name": "text", "origin": "0x08000000", "length": "0x00040000" },
        { "name": "rodata", "origin": "0x08040000", "length": "0x00020000" },
        { "name": "data", "origin": "0x20000000", "length": "0x00008000" },
        { "name": "bss", "origin": "0x20008000", "length": "0x00008000" }
    ]);

    let args_valid = json!({
        "action": "validate_memory_map",
        "hardware_regions": hw_regions,
        "linker_regions": valid_linker
    });

    let res_str_valid = tool.execute(args_valid).await.expect("Execution must succeed");
    let res_valid: Value = serde_json::from_str(&res_str_valid).expect("Valid JSON");

    assert_eq!(res_valid["status"], "SUCCESS");
    assert_eq!(res_valid["validation_verdict"], "VALID_MEMORY_MAP");
    assert!(res_valid["is_valid"].as_bool().unwrap());
    assert_eq!(res_valid["overlap_hazards_count"], 0);

    // 2. Collision Linker Layout (Overlapping sections)
    let collision_linker = json!([
        { "name": "text", "origin": "0x08000000", "length": "0x00040000" },
        { "name": "rodata", "origin": "0x08020000", "length": "0x00040000" } // Overlaps with text
    ]);

    let args_coll = json!({
        "action": "validate_memory_map",
        "hardware_regions": hw_regions,
        "linker_regions": collision_linker
    });

    let res_str_coll = tool.execute(args_coll).await.expect("Execution must succeed");
    let res_coll: Value = serde_json::from_str(&res_str_coll).expect("Valid JSON");

    assert_eq!(res_coll["status"], "SUCCESS");
    assert_eq!(res_coll["validation_verdict"], "COLLISION_DETECTED");
    assert!(!res_coll["is_valid"].as_bool().unwrap());
    assert!(res_coll["overlap_hazards_count"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_qemu_baremetal_emulator_tool_synthesize_test_harness() {
    let tool = QemuBaremetalEmulatorTool::new();

    let args = json!({
        "action": "synthesize_test_harness",
        "target_arch": "riscv64",
        "expected_boot_strings": [
            "Booting kernel...",
            "UART driver ready",
            "Kernel test suite: ALL_PASS"
        ],
        "timeout_seconds": 25,
        "panic_keywords": ["PANIC", "HardFault", "Store Access Fault"]
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["target_arch"], "riscv64");
    assert_eq!(res["exit_mechanism"], "sifive_test");
    assert_eq!(res["timeout_seconds"], 25);
    assert_eq!(res["monitored_assertions_count"], 3);

    let code = res["harness_script"].as_str().unwrap();
    assert!(code.contains("qemu-system-riscv64"));
    assert!(code.contains("Booting kernel..."));
    assert!(code.contains("Store Access Fault"));
}

// =========================================================================
// Pillar 6: TlaConsensusCheckerTool Execution Tests
// =========================================================================

#[tokio::test]
async fn test_tla_consensus_checker_tool_synthesize_tla_spec() {
    let tool = TlaConsensusCheckerTool::new();
    assert_eq!(tool.name(), "tla_consensus_checker");

    // 1. Raft Specification
    let args_raft = json!({
        "action": "synthesize_tla_spec",
        "protocol": "raft",
        "module_name": "RaftConsensusSpec",
        "node_count": 3
    });

    let res_str_raft = tool.execute(args_raft).await.expect("Execution must succeed");
    let res_raft: Value = serde_json::from_str(&res_str_raft).expect("Valid JSON");

    assert_eq!(res_raft["status"], "SUCCESS");
    assert_eq!(res_raft["protocol"], "raft");
    assert_eq!(res_raft["module_name"], "RaftConsensusSpec");
    assert!(res_raft["safety_invariants"].as_array().unwrap().iter().any(|v| v == "ElectionSafety"));

    let tla_raft = res_raft["tla_code"].as_str().unwrap();
    assert!(tla_raft.contains("MODULE RaftConsensusSpec"));
    assert!(tla_raft.contains("TypeOK =="));
    assert!(tla_raft.contains("ElectionSafety =="));
    assert!(tla_raft.contains("LogMatching =="));

    // 2. Paxos Specification
    let args_paxos = json!({
        "action": "synthesize_tla_spec",
        "protocol": "paxos"
    });

    let res_str_paxos = tool.execute(args_paxos).await.expect("Execution must succeed");
    let res_paxos: Value = serde_json::from_str(&res_str_paxos).expect("Valid JSON");
    assert_eq!(res_paxos["protocol"], "paxos");
    assert!(res_paxos["safety_invariants"].as_array().unwrap().iter().any(|v| v == "Agreement"));

    // 3. Two-Phase Commit Specification
    let args_2pc = json!({
        "action": "synthesize_tla_spec",
        "protocol": "two_phase_commit"
    });

    let res_str_2pc = tool.execute(args_2pc).await.expect("Execution must succeed");
    let res_2pc: Value = serde_json::from_str(&res_str_2pc).expect("Valid JSON");
    assert_eq!(res_2pc["protocol"], "two_phase_commit");
    assert!(res_2pc["safety_invariants"].as_array().unwrap().iter().any(|v| v == "Consistency"));
}

#[tokio::test]
async fn test_tla_consensus_checker_tool_generate_cfg_file() {
    let tool = TlaConsensusCheckerTool::new();

    let args = json!({
        "action": "generate_cfg_file",
        "spec_name": "RaftConsensusSpec",
        "constants": {
            "Server": "{s1, s2, s3}",
            "Value": "{v1, v2}"
        },
        "symmetry_set": "Server",
        "invariants": ["TypeOK", "ElectionSafety", "LogMatching"],
        "properties": ["Liveness"]
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["invariants_count"], 3);
    assert_eq!(res["properties_count"], 1);

    let cfg = res["cfg_content"].as_str().unwrap();
    assert!(cfg.contains("SPECIFICATION Spec"));
    assert!(cfg.contains("CONSTANTS"));
    assert!(cfg.contains("Server = {s1, s2, s3}"));
    assert!(cfg.contains("SYMMETRY Permutations(Server)"));
    assert!(cfg.contains("ElectionSafety"));
    assert!(cfg.contains("Liveness"));
}

#[tokio::test]
async fn test_tla_consensus_checker_tool_analyze_state_space() {
    let tool = TlaConsensusCheckerTool::new();

    let args = json!({
        "action": "analyze_state_space",
        "protocol": "raft",
        "node_count": 3,
        "max_terms_or_rounds": 3
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["protocol"], "raft");
    assert_eq!(res["node_count"], 3);

    let envelope = &res["state_space_envelope"];
    // 3! = 6
    assert_eq!(envelope["symmetry_reduction_factor"], 6);
    assert!(envelope["projected_reachable_states"].as_u64().unwrap() > 0);
    assert!(envelope["bfs_diameter_bound"].as_u64().unwrap() >= 10);
    assert_eq!(envelope["tlc_memory_recommendation_gb"], 4);
    assert!(envelope["recommended_tlc_flags"].as_str().unwrap().contains("-workers 8"));
}

// =========================================================================
// Pillar 7: ToolRegistry Registration & Re-export Tests
// =========================================================================

#[test]
fn test_tool_registry_with_builtins_contains_wave7_tools() {
    let registry = ToolRegistry::with_builtins();

    assert!(registry.contains("z3_smt_solver"), "with_builtins() must contain z3_smt_solver");
    assert!(registry.contains("rr_time_travel_debugger"), "with_builtins() must contain rr_time_travel_debugger");
    assert!(registry.contains("qemu_baremetal_emulator"), "with_builtins() must contain qemu_baremetal_emulator");
    assert!(registry.contains("tla_consensus_checker"), "with_builtins() must contain tla_consensus_checker");

    let z3 = registry.get("z3_smt_solver").expect("z3 tool must be retrievable");
    assert_eq!(z3.name(), "z3_smt_solver");

    let rr = registry.get("rr_time_travel_debugger").expect("rr tool must be retrievable");
    assert_eq!(rr.name(), "rr_time_travel_debugger");

    let qemu = registry.get("qemu_baremetal_emulator").expect("qemu tool must be retrievable");
    assert_eq!(qemu.name(), "qemu_baremetal_emulator");

    let tla = registry.get("tla_consensus_checker").expect("tla tool must be retrievable");
    assert_eq!(tla.name(), "tla_consensus_checker");
}

#[test]
fn test_tool_registry_with_builtins_in_dir_contains_wave7_tools() {
    let dir = std::path::PathBuf::from("/home/dyna/TGS Projects/tagisan");
    let registry = ToolRegistry::with_builtins_in_dir(dir);

    assert!(registry.contains("z3_smt_solver"), "with_builtins_in_dir must contain z3_smt_solver");
    assert!(registry.contains("rr_time_travel_debugger"), "with_builtins_in_dir must contain rr_time_travel_debugger");
    assert!(registry.contains("qemu_baremetal_emulator"), "with_builtins_in_dir must contain qemu_baremetal_emulator");
    assert!(registry.contains("tla_consensus_checker"), "with_builtins_in_dir must contain tla_consensus_checker");
}
