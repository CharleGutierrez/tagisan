//! Brutal Integration Tests for Host Memory Governor & Anti-Freeze Shield in Tagisan (TGS)

use tagisan::governor::{HostMemoryGovernor, LinuxMemInfo, MemoryPressureTier};
use tagisan::ecc::{find_preset, all_presets};
use tagisan::swarm::repl::{InteractiveRepl, ReplCommand};

#[test]
fn test_linux_meminfo_live_or_default() {
    let mem = LinuxMemInfo::read_host();
    assert!(mem.mem_total_kb > 0, "Total RAM must be positive");
    assert!(mem.available_pct() >= 0.0 && mem.available_pct() <= 100.0);
    assert!(mem.total_gb() > 0.0);
}

#[test]
fn test_memory_pressure_tiers() {
    let gov = HostMemoryGovernor::new();

    // 1. Healthy green state: 3.5GB available out of 8GB (43.7% available)
    let green_mem = LinuxMemInfo {
        mem_total_kb: 8 * 1024 * 1024,
        mem_free_kb: 2 * 1024 * 1024,
        mem_available_kb: 3500 * 1024,
        buffers_kb: 100 * 1024,
        cached_kb: 1400 * 1024,
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 6 * 1024 * 1024, // 25% swap used
        dirty_kb: 32 * 1024,
        timestamp_ms: 0,
    };
    assert_eq!(gov.evaluate_pressure(&green_mem), MemoryPressureTier::GreenNormal);
    assert!(gov.can_spawn_subagent(&green_mem));

    // 2. Yellow warning state: 1.5GB available out of 8GB (18.7% available)
    let yellow_mem = LinuxMemInfo {
        mem_available_kb: 1500 * 1024,
        ..green_mem
    };
    assert_eq!(gov.evaluate_pressure(&yellow_mem), MemoryPressureTier::YellowWarning);
    assert!(gov.can_spawn_subagent(&yellow_mem));
    assert_eq!(gov.recommended_concurrency(&yellow_mem), 2);
    assert_eq!(gov.recommended_cargo_jobs(&yellow_mem), 1);

    // 3. Red critical state: 0.8GB available out of 8GB (10.0% available)
    let red_mem = LinuxMemInfo {
        mem_available_kb: 800 * 1024,
        ..green_mem
    };
    assert_eq!(gov.evaluate_pressure(&red_mem), MemoryPressureTier::RedCritical);
    assert!(!gov.can_spawn_subagent(&red_mem), "Red tier must block subagent spawning");
    assert_eq!(gov.recommended_concurrency(&red_mem), 1);
    assert_eq!(gov.recommended_cargo_jobs(&red_mem), 1);

    // 4. Swap saturation critical state (>85% swap full)
    let swap_exhausted = LinuxMemInfo {
        swap_total_kb: 8 * 1024 * 1024,
        swap_free_kb: 500 * 1024, // >90% swap used
        ..green_mem
    };
    assert_eq!(gov.evaluate_pressure(&swap_exhausted), MemoryPressureTier::RedCritical);
}

#[test]
fn test_8gb_workstation_detection() {
    let gov = HostMemoryGovernor::new();

    let mem_8gb = LinuxMemInfo {
        mem_total_kb: 7860 * 1024, // ~7.8GB
        ..LinuxMemInfo::default()
    };
    assert!(gov.is_8gb_workstation(&mem_8gb));

    let mem_32gb = LinuxMemInfo {
        mem_total_kb: 32 * 1024 * 1024,
        ..LinuxMemInfo::default()
    };
    assert!(!gov.is_8gb_workstation(&mem_32gb));
}

#[test]
fn test_malloc_trim_execution() {
    let gov = HostMemoryGovernor::new();
    // Verify that trim_heap executes safely on any host platform without crashing
    let _ = gov.trim_heap();
}

#[test]
fn test_memory_audit_and_repl_banner() {
    let gov = HostMemoryGovernor::new();
    let audit = gov.audit();

    assert!(audit.metrics.mem_total_kb > 0);
    assert!(!audit.summary.is_empty());

    let banner = gov.format_repl_banner();
    assert!(banner.contains("TAGISAN ANTI-FREEZE HOST MEMORY GOVERNOR"));
    assert!(banner.contains("Workstation Profile"));
    assert!(banner.contains("Physical RAM"));
    assert!(banner.contains("Available Headroom"));
}

#[test]
fn test_repl_mem_commands() {
    assert_eq!(
        InteractiveRepl::parse_command("/mem"),
        ReplCommand::HostMem("".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/mem status"),
        ReplCommand::HostMem("status".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/mem trim"),
        ReplCommand::HostMem("trim".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/mem guard"),
        ReplCommand::HostMem("guard".to_string())
    );
    assert_eq!(
        InteractiveRepl::parse_command("/mem help"),
        ReplCommand::HostMem("help".to_string())
    );
}

#[test]
fn test_ecc_low_memory_worker_preset() {
    let all = all_presets();
    assert!(all.iter().any(|p| p.name == "low-memory-worker"));

    let preset = find_preset("low-memory-worker").expect("low-memory-worker preset must be registered");
    assert_eq!(preset.name, "low-memory-worker");
    assert!(preset.tools.contains(&"read_file".to_string()));
    assert!(preset.system_prompt.contains("Host Memory Governor"));
}
