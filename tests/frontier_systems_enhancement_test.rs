//! # Comprehensive Brutal Test Suite: Frontier Systems Skills & Systems Tools
//!
//! Validates:
//! 1. All 4 Frontier Systems skills (`verilog-chisel-fpga-synthesizer`, `xdp-dpdk-kernel-bypass-firewall`,
//!    `spdk-nvme-direct-storage-engine`, `ptp-truetime-clock-synchronizer`) parse cleanly with `EccSkill::parse`.
//! 2. Valid YAML frontmatter, tags, triggers, and dense operational invariant sections (`ALWAYS`, `NEVER`, `MANDATORY`, `STRICT_REJECT`).
//! 3. Discovery in `all_built_in_skills()` and `find_built_in_skill()` (including aliases).
//! 4. `FpgaVerilogSynthesizerTool` synthesizable RTL generation, timing closure STA, and resource budgeting.
//! 5. `XdpPacketFilterTool` line-rate BPF rule synthesis, verifier efficiency, and 100 Gbps throughput simulation.
//! 6. `SpdkNvmeStorageTool` hugepage DMA memory planning, polled-mode driver harness, and PCIe IOPS envelope analysis.
//! 7. Registration in `ToolRegistry::with_builtins()` and `ToolRegistry::with_builtins_in_dir()`.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tagisan::ecc::{all_built_in_skills, find_built_in_skill, EccSkill};
use tagisan::tools::{
    FpgaVerilogSynthesizerTool, SpdkNvmeStorageTool, ToolHandler, ToolRegistry,
    XdpPacketFilterTool,
};

// =========================================================================
// Pillar 1: Frontier Systems Skills Parsing, Frontmatter & Invariant Verification
// =========================================================================

#[test]
fn test_verilog_chisel_fpga_synthesizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/verilog-chisel-fpga-synthesizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/verilog-chisel-fpga-synthesizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read verilog-chisel-fpga-synthesizer SKILL.md");
    assert!(raw_content.starts_with("---"), "Frontmatter must begin with '---'");
    assert!(raw_content.contains("name: verilog-chisel-fpga-synthesizer"));
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("fpga"));
    assert!(raw_content.contains("verilog"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("rtl"));
    assert!(raw_content.contains("timing closure"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "verilog-chisel-fpga-synthesizer");
    assert!(!skill.description.is_empty());

    // Verify dense invariants
    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"), "Must contain ALWAYS directive");
    assert!(instructions.contains("NEVER"), "Must contain NEVER directive");
    assert!(instructions.contains("MANDATORY"), "Must contain MANDATORY directive");
    assert!(instructions.contains("STRICT_REJECT"), "Must contain STRICT_REJECT directive");
    assert!(instructions.contains("synchronous"), "Must mandate synchronous reset");
    assert!(instructions.contains("setup"), "Must mandate setup slack checks");
}

#[test]
fn test_xdp_dpdk_kernel_bypass_firewall_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/xdp-dpdk-kernel-bypass-firewall/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/xdp-dpdk-kernel-bypass-firewall");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read xdp-dpdk-kernel-bypass-firewall SKILL.md");
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("xdp"));
    assert!(raw_content.contains("dpdk"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("line rate"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "xdp-dpdk-kernel-bypass-firewall");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("XDP_DROP"));
    assert!(instructions.contains("bounds"));
}

#[test]
fn test_spdk_nvme_direct_storage_engine_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/spdk-nvme-direct-storage-engine/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/spdk-nvme-direct-storage-engine");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read spdk-nvme-direct-storage-engine SKILL.md");
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("spdk"));
    assert!(raw_content.contains("nvme"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("polled mode"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "spdk-nvme-direct-storage-engine");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("hugepage"));
    assert!(instructions.contains("4KB") || instructions.contains("4096"));
}

#[test]
fn test_ptp_truetime_clock_synchronizer_skill_parsing_and_invariants() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/ptp-truetime-clock-synchronizer/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/ptp-truetime-clock-synchronizer");

    let raw_content = fs::read_to_string(skill_path).expect("Failed to read ptp-truetime-clock-synchronizer SKILL.md");
    assert!(raw_content.contains("tags:"));
    assert!(raw_content.contains("ptp"));
    assert!(raw_content.contains("truetime"));
    assert!(raw_content.contains("triggers:"));
    assert!(raw_content.contains("uncertainty window") || raw_content.contains("commit wait"));

    let skill = EccSkill::parse(&raw_content).expect("Skill must parse cleanly with EccSkill::parse");
    assert_eq!(skill.name, "ptp-truetime-clock-synchronizer");

    let instructions = &skill.instructions;
    assert!(instructions.contains("ALWAYS"));
    assert!(instructions.contains("NEVER"));
    assert!(instructions.contains("MANDATORY"));
    assert!(instructions.contains("STRICT_REJECT"));
    assert!(instructions.contains("commit-wait"));
    assert!(instructions.contains("uncertainty"));
}

// =========================================================================
// Pillar 2: Discovery & Alias Resolution
// =========================================================================

#[test]
fn test_all_built_in_skills_contains_all_four_frontier_skills() {
    let skills = all_built_in_skills();
    let skill_names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();

    assert!(skill_names.contains(&"verilog-chisel-fpga-synthesizer"), "Must contain verilog-chisel-fpga-synthesizer");
    assert!(skill_names.contains(&"xdp-dpdk-kernel-bypass-firewall"), "Must contain xdp-dpdk-kernel-bypass-firewall");
    assert!(skill_names.contains(&"spdk-nvme-direct-storage-engine"), "Must contain spdk-nvme-direct-storage-engine");
    assert!(skill_names.contains(&"ptp-truetime-clock-synchronizer"), "Must contain ptp-truetime-clock-synchronizer");
}

#[test]
fn test_find_built_in_skill_frontier_aliases() {
    // 1. verilog-chisel-fpga-synthesizer
    let s1_direct = find_built_in_skill("verilog-chisel-fpga-synthesizer").expect("Direct lookup must succeed");
    assert_eq!(s1_direct.name, "verilog-chisel-fpga-synthesizer");

    let s1_alias = find_built_in_skill("fpga").expect("Alias 'fpga' must resolve");
    assert_eq!(s1_alias.name, "verilog-chisel-fpga-synthesizer");

    let s1_verilog = find_built_in_skill("verilog").expect("Alias 'verilog' must resolve");
    assert_eq!(s1_verilog.name, "verilog-chisel-fpga-synthesizer");

    // 2. xdp-dpdk-kernel-bypass-firewall
    let s2_direct = find_built_in_skill("xdp-dpdk-kernel-bypass-firewall").expect("Direct lookup must succeed");
    assert_eq!(s2_direct.name, "xdp-dpdk-kernel-bypass-firewall");

    let s2_xdp = find_built_in_skill("xdp").expect("Alias 'xdp' must resolve");
    assert_eq!(s2_xdp.name, "xdp-dpdk-kernel-bypass-firewall");

    let s2_dpdk = find_built_in_skill("dpdk").expect("Alias 'dpdk' must resolve");
    assert_eq!(s2_dpdk.name, "xdp-dpdk-kernel-bypass-firewall");

    // 3. spdk-nvme-direct-storage-engine
    let s3_direct = find_built_in_skill("spdk-nvme-direct-storage-engine").expect("Direct lookup must succeed");
    assert_eq!(s3_direct.name, "spdk-nvme-direct-storage-engine");

    let s3_spdk = find_built_in_skill("spdk").expect("Alias 'spdk' must resolve");
    assert_eq!(s3_spdk.name, "spdk-nvme-direct-storage-engine");

    let s3_nvme = find_built_in_skill("nvme").expect("Alias 'nvme' must resolve");
    assert_eq!(s3_nvme.name, "spdk-nvme-direct-storage-engine");

    // 4. ptp-truetime-clock-synchronizer
    let s4_direct = find_built_in_skill("ptp-truetime-clock-synchronizer").expect("Direct lookup must succeed");
    assert_eq!(s4_direct.name, "ptp-truetime-clock-synchronizer");

    let s4_truetime = find_built_in_skill("truetime").expect("Alias 'truetime' must resolve");
    assert_eq!(s4_truetime.name, "ptp-truetime-clock-synchronizer");

    let s4_ptp = find_built_in_skill("ptp").expect("Alias 'ptp' must resolve");
    assert_eq!(s4_ptp.name, "ptp-truetime-clock-synchronizer");
}

// =========================================================================
// Pillar 3: FpgaVerilogSynthesizerTool Execution & Invariant Tests
// =========================================================================

#[tokio::test]
async fn test_fpga_verilog_synthesizer_tool_module_synthesis() {
    let tool = FpgaVerilogSynthesizerTool::new();
    assert_eq!(tool.name(), "fpga_verilog_synthesizer");

    let args = json!({
        "action": "synthesize_module",
        "module_name": "gemm_pe_accelerator",
        "target_device": "xilinx_ultrascale",
        "clock_frequency_mhz": 400,
        "pipeline_stages": 4
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["module_name"], "gemm_pe_accelerator");
    assert_eq!(res["pipeline_stages"], 4);
    assert_eq!(res["clock_frequency_mhz"], 400);
    assert!(res["axi_compliant"].as_bool().unwrap());
    assert!(res["synchronous_reset"].as_bool().unwrap());

    let verilog = res["verilog_code"].as_str().unwrap();
    assert!(verilog.contains("module gemm_pe_accelerator"));
    assert!(verilog.contains("s_axis_tdata"));
    assert!(verilog.contains("m_axis_tdata"));
    assert!(verilog.contains("always @(posedge clk)"));
    assert!(verilog.contains("if (!rst_n)"));
}

#[tokio::test]
async fn test_fpga_verilog_synthesizer_tool_timing_closure_met() {
    let tool = FpgaVerilogSynthesizerTool::new();

    let args = json!({
        "action": "analyze_timing_closure",
        "module_name": "mac_unit",
        "clock_frequency_mhz": 250,
        "pipeline_stages": 4
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["timing_verdict"], "TIMING_MET");
    assert!(res["timing_closure_passed"].as_bool().unwrap());
    assert!(res["setup_slack_ns"].as_f64().unwrap() > 0.0);
    assert!(res["hold_slack_ns"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_fpga_verilog_synthesizer_tool_timing_closure_violation() {
    let tool = FpgaVerilogSynthesizerTool::new();

    let args = json!({
        "action": "analyze_timing_closure",
        "module_name": "deep_combinatorial_mac",
        "clock_frequency_mhz": 1000,
        "pipeline_stages": 1
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["timing_verdict"], "TIMING_VIOLATION");
    assert!(!res["timing_closure_passed"].as_bool().unwrap());
    assert!(res["setup_slack_ns"].as_f64().unwrap() < 0.0);
    assert!(res["recommendations"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_fpga_verilog_synthesizer_tool_resource_estimation() {
    let tool = FpgaVerilogSynthesizerTool::new();

    let args = json!({
        "action": "estimate_resource_utilization",
        "module_name": "tensor_core_pe",
        "target_device": "xilinx_ultrascale",
        "pipeline_stages": 4
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["resource_budget_status"], "WITHIN_BUDGET");
    assert!(res["estimated_resources"]["lut_count"].as_u64().unwrap() > 0);
    assert!(res["estimated_resources"]["flip_flop_count"].as_u64().unwrap() > 0);
    assert!(res["estimated_resources"]["dsp_slice_count"].as_u64().unwrap() > 0);
    assert!(res["utilization_percentages"]["lut_percentage"].as_f64().unwrap() < 1.0);
}

// =========================================================================
// Pillar 4: XdpPacketFilterTool Execution & Invariant Tests
// =========================================================================

#[tokio::test]
async fn test_xdp_packet_filter_tool_rule_generation() {
    let tool = XdpPacketFilterTool::new();
    assert_eq!(tool.name(), "xdp_packet_filter");

    let args = json!({
        "action": "generate_xdp_rule",
        "filter_type": "ddos_syn_flood",
        "interface_speed_gbps": 100,
        "rule_spec": {
            "blocked_ip": "198.51.100.25",
            "blocked_port": 443
        }
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert_eq!(res["filter_type"], "ddos_syn_flood");
    assert_eq!(res["interface_speed_gbps"], 100);
    assert_eq!(res["xdp_action_on_match"], "XDP_DROP");
    assert!(res["bpf_verifier_invariants"]["bounds_checks_enforced"].as_bool().unwrap());
    assert!(res["bpf_verifier_invariants"]["single_cache_line_access"].as_bool().unwrap());
    assert!(res["bpf_verifier_invariants"]["zero_heap_allocations"].as_bool().unwrap());

    let bpf = res["bpf_c_code"].as_str().unwrap();
    assert!(bpf.contains("SEC(\"xdp\")"));
    assert!(bpf.contains("XDP_DROP"));
    assert!(bpf.contains("XDP_PASS"));
    assert!(bpf.contains("data_end"));
}

#[tokio::test]
async fn test_xdp_packet_filter_tool_efficiency_analysis() {
    let tool = XdpPacketFilterTool::new();

    let args = json!({
        "action": "analyze_filter_efficiency",
        "filter_type": "ip_blacklist"
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["bpf_verifier_compliance"], "PASS");
    assert_eq!(res["cache_lines_accessed"], 1);
    assert_eq!(res["heap_allocations_fast_path"], 0);
    assert!(res["stack_frame_bytes"].as_u64().unwrap() <= 512);
    assert!(res["branch_predictability_score"].as_f64().unwrap() > 95.0);
}

#[tokio::test]
async fn test_xdp_packet_filter_tool_line_speed_throughput_simulation() {
    let tool = XdpPacketFilterTool::new();

    let args = json!({
        "action": "simulate_line_speed_throughput",
        "interface_speed_gbps": 100,
        "packet_size_bytes": 64
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    let mpps = res["wire_packet_rate_mpps"].as_f64().unwrap();
    let ns_budget = res["per_packet_time_budget_ns"].as_f64().unwrap();

    assert!(mpps > 140.0 && mpps < 155.0, "Expected ~148.8 Mpps, got {mpps}");
    assert!(ns_budget > 6.0 && ns_budget < 7.5, "Expected ~6.72 ns budget, got {ns_budget}");
    assert!(res["line_speed_achievable"].as_bool().unwrap());
    assert_eq!(res["xdp_processing_budget_verdict"], "FEASIBLE_WITH_XDP_NATIVE_DRIVER");
}

// =========================================================================
// Pillar 5: SpdkNvmeStorageTool Execution & Invariant Tests
// =========================================================================

#[tokio::test]
async fn test_spdk_nvme_storage_tool_ring_buffer_planning() {
    let tool = SpdkNvmeStorageTool::new();
    assert_eq!(tool.name(), "spdk_nvme_storage");

    let args = json!({
        "action": "plan_io_ring_buffers",
        "queue_depth": 128,
        "block_size_bytes": 4096,
        "hugepage_size_mb": 2
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["queue_depth"], 128);
    assert_eq!(res["block_size_bytes"], 4096);
    assert!(res["page_alignment_4kb"].as_bool().unwrap());
    assert!(res["lockless_queue_guarantee"].as_bool().unwrap());
    assert_eq!(res["dma_memory_status"], "PHYSICALLY_CONTIGUOUS_VALIDATED");
    assert!(res["hugepages_required"].as_u64().unwrap() >= 1);
}

#[tokio::test]
async fn test_spdk_nvme_storage_tool_harness_synthesis() {
    let tool = SpdkNvmeStorageTool::new();

    let args = json!({
        "action": "synthesize_spdk_harness",
        "queue_depth": 64,
        "block_size_bytes": 4096
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    assert_eq!(res["status"], "SUCCESS");
    assert!(res["features"]["kernel_bypass"].as_bool().unwrap());
    assert!(res["features"]["interrupt_free_polling"].as_bool().unwrap());
    assert!(res["features"]["zero_copy_dma"].as_bool().unwrap());
    assert_eq!(res["features"]["alignment_bytes"], 4096);

    let code = res["spdk_c_harness"].as_str().unwrap();
    assert!(code.contains("spdk_dma_zmalloc"));
    assert!(code.contains("spdk_nvme_ns_cmd_write"));
    assert!(code.contains("spdk_nvme_qpair_process_completions"));
}

#[tokio::test]
async fn test_spdk_nvme_storage_tool_iops_envelope_benchmark() {
    let tool = SpdkNvmeStorageTool::new();

    let args = json!({
        "action": "benchmark_iops_envelope",
        "queue_depth": 64,
        "block_size_bytes": 4096,
        "pcie_generation": 4,
        "pcie_lanes": 4
    });

    let res_str = tool.execute(args).await.expect("Execution must succeed");
    let res: Value = serde_json::from_str(&res_str).expect("Output must be valid JSON");

    let max_iops = res["max_theoretical_iops"].as_u64().unwrap();
    assert!(max_iops > 1_500_000, "PCIe Gen4 x4 must sustain multi-million IOPS, got {max_iops}");
    assert_eq!(res["iops_envelope_rating"], "MULTI_MILLION_IOPS_CAPABLE");
    assert!(res["projected_queue_latency_us"].as_f64().unwrap() < 50.0);
}

// =========================================================================
// Pillar 6: ToolRegistry Registration & Re-export Tests
// =========================================================================

#[test]
fn test_tool_registry_with_builtins_contains_frontier_tools() {
    let registry = ToolRegistry::with_builtins();

    assert!(registry.contains("fpga_verilog_synthesizer"), "ToolRegistry::with_builtins() must contain fpga_verilog_synthesizer");
    assert!(registry.contains("xdp_packet_filter"), "ToolRegistry::with_builtins() must contain xdp_packet_filter");
    assert!(registry.contains("spdk_nvme_storage"), "ToolRegistry::with_builtins() must contain spdk_nvme_storage");

    let fpga = registry.get("fpga_verilog_synthesizer").expect("fpga tool must be retrievable");
    assert_eq!(fpga.name(), "fpga_verilog_synthesizer");

    let xdp = registry.get("xdp_packet_filter").expect("xdp tool must be retrievable");
    assert_eq!(xdp.name(), "xdp_packet_filter");

    let spdk = registry.get("spdk_nvme_storage").expect("spdk tool must be retrievable");
    assert_eq!(spdk.name(), "spdk_nvme_storage");
}

#[test]
fn test_tool_registry_with_builtins_in_dir_contains_frontier_tools() {
    let dir = std::path::PathBuf::from("/home/dyna/TGS Projects/tagisan");
    let registry = ToolRegistry::with_builtins_in_dir(dir);

    assert!(registry.contains("fpga_verilog_synthesizer"), "with_builtins_in_dir must contain fpga_verilog_synthesizer");
    assert!(registry.contains("xdp_packet_filter"), "with_builtins_in_dir must contain xdp_packet_filter");
    assert!(registry.contains("spdk_nvme_storage"), "with_builtins_in_dir must contain spdk_nvme_storage");
}
