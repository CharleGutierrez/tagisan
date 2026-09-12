//! Brutal Performance, Caching, and Low-Resource Hardware Optimization Tests for Tagisan Skills
//! Specifically engineered to validate performance on low-spec hardware (Intel Core i3 7th Gen 2C/4T, 8GB RAM, SSD/HDD).

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tagisan::{
    all_ecc_skills, compute_dir_fingerprint,
    CachedCatalog, SkillDispatcher, SKILLS_CACHE_MAGIC, SKILLS_CACHE_VERSION,
};

/// Helper to create a temporary test directory with dummy and realistic skills
struct TestWorkspace {
    root: PathBuf,
    skills_dir: PathBuf,
    cache_path: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("tgs_perf_test_{}_{}_{}", name, std::process::id(), Instant::now().elapsed().as_nanos()));
        let skills_dir = root.join(".ecc").join("skills");
        let cache_path = root.join(".tagisan").join("skills.cache");
        fs::create_dir_all(&skills_dir).expect("Failed to create test skills dir");
        fs::create_dir_all(cache_path.parent().unwrap()).expect("Failed to create .tagisan dir");
        Self {
            root,
            skills_dir,
            cache_path,
        }
    }

    fn write_skill(&self, slug: &str, title: &str, triggers: &[&str], body: &str) -> PathBuf {
        let dir = self.skills_dir.join(slug);
        fs::create_dir_all(&dir).expect("Failed to create skill subfolder");
        let file = dir.join("SKILL.md");
        let trigs_yaml = triggers.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
        let content = format!(
            "---\nname: {}\ndescription: \"{}\"\ntriggers: [{}]\n---\n\n# {}\n\n{}",
            slug, title, trigs_yaml, title, body
        );
        fs::write(&file, content).expect("Failed to write test SKILL.md");
        file
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn test_01_cold_build_creates_valid_binary_cache() {
    let ws = TestWorkspace::new("cold_build");
    ws.write_skill("custom-inventory-calc", "Inventory Optimization", &["inventory", "eoq"], "Calculate EOQ formulas.");
    ws.write_skill("custom-credit-check", "Credit Validation", &["credit", "risk"], "Perform credit scoring.");

    assert!(!ws.cache_path.exists(), "Cache file must not exist prior to cold build");

    let start = Instant::now();
    let dispatcher = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    let cold_duration = start.elapsed();

    println!("Cold build with {} skills took: {:?}", dispatcher.len(), cold_duration);
    assert!(ws.cache_path.exists(), "Cache file must be generated after cold build");

    // Inspect binary cache structure directly
    let file = File::open(&ws.cache_path).expect("Failed to open written cache file");
    let mmap = unsafe { memmap2::Mmap::map(&file).expect("Failed to mmap cache") };
    let catalog: CachedCatalog = bincode::deserialize(&mmap).expect("Failed to deserialize CachedCatalog");

    assert_eq!(catalog.magic, SKILLS_CACHE_MAGIC);
    assert_eq!(catalog.version, SKILLS_CACHE_VERSION);
    assert_eq!(catalog.builtin_count, all_ecc_skills().len());
    assert!(catalog.skills.len() >= all_ecc_skills().len() + 2);
    assert!(catalog.dir_fingerprint != 0);

    // Verify metadata was captured
    let custom_skill = catalog.skills.iter().find(|s| s.name == "custom-inventory-calc");
    assert!(custom_skill.is_some(), "Custom skill must be present in serialized catalog");
    assert_eq!(custom_skill.unwrap().is_builtin, false);
}

#[test]
fn test_02_hot_load_latency_is_sub_five_milliseconds() {
    let ws = TestWorkspace::new("hot_load");
    for i in 0..20 {
        ws.write_skill(
            &format!("custom-test-skill-{}", i),
            &format!("Custom Test Skill Description {}", i),
            &["test", &format!("trigger-{}", i)],
            &format!("Instructions body for skill {} with extensive details.", i),
        );
    }

    // Initial cold build
    let _cold = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    assert!(ws.cache_path.exists());

    // Brutal Hot Load Benchmark: execute 20 hot loads from disk mmap cache
    let mut hot_durations = Vec::new();
    for _ in 0..20 {
        let start = Instant::now();
        let hot_disp = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
        let dur = start.elapsed();
        hot_durations.push(dur);
        assert!(hot_disp.len() >= 240);
    }

    let min_dur = hot_durations.iter().min().unwrap();
    let max_dur = hot_durations.iter().max().unwrap();
    let avg_dur = hot_durations.iter().sum::<Duration>() / (hot_durations.len() as u32);

    println!(
        "⚡ Hot Cache Load Benchmarks (20 iterations): Min={:?}, Max={:?}, Avg={:?}",
        min_dur, max_dur, avg_dur
    );

    // Hot load from mmap binary cache must be under 5ms in release (< 250ms in debug with parallel test runners)
    let threshold_ms = if cfg!(debug_assertions) { 250 } else { 5 };
    assert!(
        *min_dur < Duration::from_millis(threshold_ms),
        "Minimum hot load time {:?} exceeded {}ms threshold",
        min_dur, threshold_ms
    );
    assert!(
        avg_dur < Duration::from_millis(threshold_ms),
        "Average hot load time {:?} exceeded {}ms threshold",
        avg_dur, threshold_ms
    );
}

#[test]
fn test_03_cache_invalidation_on_skill_mutation_addition_deletion() {
    let ws = TestWorkspace::new("invalidation");
    let file1 = ws.write_skill("dynamic-flow-a", "Initial Flow A", &["flow-a"], "Body A");

    // 1. Initial build
    let disp1 = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    let initial_count = disp1.len();
    let initial_fp = compute_dir_fingerprint(Some(&ws.skills_dir));
    assert!(initial_fp != 0);

    // Sleep briefly to ensure filesystem timestamp ticks
    thread::sleep(Duration::from_millis(50));

    // 2. Modify existing skill file
    fs::write(&file1, "---\nname: dynamic-flow-a\ndescription: \"Updated Description A\"\ntriggers: [flow-a, updated]\n---\n# Updated\nNew body.")
        .expect("Failed to overwrite skill file");

    let updated_fp = compute_dir_fingerprint(Some(&ws.skills_dir));
    assert_ne!(initial_fp, updated_fp, "Directory fingerprint MUST change after file modification");

    // Hot load should automatically detect invalidation and rebuild
    let disp2 = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    let updated_skill = disp2.skills().iter().find(|s| s.name == "dynamic-flow-a").unwrap();
    assert_eq!(updated_skill.description, "Updated Description A");

    // Sleep briefly again
    thread::sleep(Duration::from_millis(50));

    // 3. Add new skill file
    ws.write_skill("dynamic-flow-b", "Flow B", &["flow-b"], "Body B");
    let disp3 = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    assert_eq!(disp3.len(), initial_count + 1);

    // Sleep briefly again
    thread::sleep(Duration::from_millis(50));

    // 4. Delete skill file
    fs::remove_dir_all(ws.skills_dir.join("dynamic-flow-b")).expect("Failed to remove dir");
    let disp4 = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    assert_eq!(disp4.len(), initial_count);
}

#[test]
fn test_04_builtin_o1_fast_bypass() {
    let ws = TestWorkspace::new("builtin_bypass");
    // Duplicate existing built-in skill name on disk
    ws.write_skill("tdd-workflow", "Disk Duplicate of TDD", &["tdd"], "Disk instructions.");

    let disp = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    let tdd = disp.skills().iter().find(|s| s.name == "tdd-workflow").unwrap();

    // Must be flagged as builtin, not disk duplicate
    assert!(tdd.is_builtin, "Built-in skill must have fast bypass precedence");
    assert_eq!(tdd.file_path, None, "Built-in skill must not reference disk file");
}

#[test]
fn test_05_lazy_jit_body_loading_and_memory_reduction() {
    let ws = TestWorkspace::new("lazy_jit");
    let huge_body = "Line of business logic rule for functional analysis.\n".repeat(500); // ~25KB
    ws.write_skill("huge-spec-skill", "Huge Specification Skill", &["huge-spec"], &huge_body);

    let disp = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    let meta = disp.skills().iter().find(|s| s.name == "huge-spec-skill").unwrap();

    // Verify metadata is compact
    assert_eq!(meta.name, "huge-spec-skill");
    assert_eq!(meta.description, "Huge Specification Skill");

    // Load full skill body JIT
    let loaded = disp.load_skill_by_id(meta.id).expect("Should lazily load skill body");
    assert!(loaded.instructions.contains("Line of business logic rule"));
    assert!(loaded.instructions.len() > 10_000);

    // Second load hits in-memory cache
    let cached = disp.load_skill_by_id(meta.id).expect("Should fetch from read cache");
    assert_eq!(loaded, cached);

    // Verify built-in skills JIT loading without initial memory burden
    let tdd_meta = disp.skills().iter().find(|s| s.name == "tdd-workflow").unwrap();
    let tdd_skill = disp.load_skill_by_id(tdd_meta.id).unwrap();
    assert!(tdd_skill.instructions.contains("Phase 1: Test Formulation"));
}

#[test]
fn test_06_cold_vs_hot_query_dispatch_parity() {
    let ws = TestWorkspace::new("parity");
    ws.write_skill("ba-custom-story-slicer", "Custom Story Slicing", &["story-slicing", "invest"], "Slice stories by happy path.");

    // Cold build
    let cold_disp = SkillDispatcher::build_from_scratch(Some(&ws.skills_dir));
    let _ = cold_disp.save_to_cache(&ws.cache_path, compute_dir_fingerprint(Some(&ws.skills_dir)));

    // Hot load from cache
    let hot_disp = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));

    let queries = [
        "event storming domain model",
        "double-entry general ledger",
        "wiegers requirements engineering",
        "story slicing invest",
        "tokio async tuning concurrency",
        "three way match purchase order",
    ];

    for query in queries {
        let cold_results = cold_disp.dispatch(query, 5, None);
        let hot_results = hot_disp.dispatch(query, 5, None);

        assert_eq!(cold_results.len(), hot_results.len(), "Query '{}' count mismatch", query);
        for (i, (c, h)) in cold_results.iter().zip(&hot_results).enumerate() {
            assert_eq!(c.skill.name, h.skill.name, "Query '{}' rank {} name mismatch", query, i);
            assert!((c.score - h.score).abs() < 1e-4, "Query '{}' rank {} score mismatch", query, i);
            assert_eq!(c.domain, h.domain, "Query '{}' rank {} domain mismatch", query, i);
        }
    }
}

#[test]
fn test_07_brutal_concurrent_50_thread_dispatch_stress() {
    let ws = TestWorkspace::new("concurrent_stress");
    for i in 0..10 {
        ws.write_skill(
            &format!("concurrent-agent-skill-{}", i),
            &format!("Concurrent Agent Workflow {}", i),
            &["concurrent", &format!("agent-{}", i)],
            &format!("Instruction set for concurrent worker {}", i),
        );
    }

    let disp = Arc::new(SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path)));
    let total_queries = Arc::new(AtomicUsize::new(0));

    let queries = Arc::new(vec![
        "event storming aggregate bounded context",
        "double entry debit credit balance sheet",
        "wiegers requirements functional specification",
        "tokio async task joinset",
        "kanban cycle time throughput lead time",
        "bpmn boundary timer event",
        "wsjf cost of delay prioritization",
        "concurrent agent workflow",
    ]);

    let start = Instant::now();
    let mut handles = Vec::new();
    let num_threads = 50;
    let queries_per_thread = 500;

    for thread_id in 0..num_threads {
        let d = Arc::clone(&disp);
        let q_list = Arc::clone(&queries);
        let counter = Arc::clone(&total_queries);

        handles.push(thread::spawn(move || {
            for i in 0..queries_per_thread {
                let q = &q_list[(thread_id + i) % q_list.len()];
                let results = d.dispatch(q, 3, None);
                assert!(!results.is_empty(), "Thread {} query '{}' returned no matches", thread_id, q);
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        h.join().expect("Worker thread panicked during concurrent dispatch");
    }

    let elapsed = start.elapsed();
    let executed = total_queries.load(Ordering::SeqCst);
    let qps = (executed as f64) / elapsed.as_secs_f64();

    println!(
        "🔥 Brutal Concurrent Stress Test: 50 threads executed {} queries in {:?} ({:.0} QPS)",
        executed, elapsed, qps
    );

    assert_eq!(executed, num_threads * queries_per_thread);
    let min_qps = if cfg!(debug_assertions) { 3_500.0 } else { 10_000.0 };
    assert!(qps > min_qps, "Dispatch throughput {:.0} QPS was below target {:.0} QPS", qps, min_qps);
}

#[test]
fn test_08_corrupted_cache_self_healing_resilience() {
    let ws = TestWorkspace::new("corrupted_cache");
    ws.write_skill("resilience-skill", "Resilience Under Chaos", &["resilience"], "Body");

    // Initial valid build
    let _ = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    assert!(ws.cache_path.exists());

    // Corrupt cache file with random trash bytes
    fs::write(&ws.cache_path, b"CORRUPTED_GARBAGE_BYTES_THAT_ARE_NOT_BINCODE").expect("Failed to corrupt cache");

    // Must not panic, must self-heal by rebuilding and overwriting with valid cache!
    let healed_disp = SkillDispatcher::load_or_build_with_cache(Some(&ws.skills_dir), Some(&ws.cache_path));
    assert!(healed_disp.len() >= 220);

    // Verify cache was recovered and is valid again
    let file = File::open(&ws.cache_path).expect("Failed to reopen healed cache");
    let mmap = unsafe { memmap2::Mmap::map(&file).expect("Failed to mmap healed cache") };
    let catalog: CachedCatalog = bincode::deserialize(&mmap).expect("Cache must be valid after self-healing");
    assert_eq!(catalog.magic, SKILLS_CACHE_MAGIC);
    assert_eq!(catalog.version, SKILLS_CACHE_VERSION);
}

#[test]
fn test_09_real_world_skills_dir_caching_and_speed() {
    let real_skills_dir = Path::new(".ecc/skills");
    if !real_skills_dir.is_dir() {
        println!("Skipping test_09: .ecc/skills directory does not exist in current dir");
        return;
    }

    let temp_cache = std::env::temp_dir().join(format!("tgs_real_skills_{}_{}.cache", std::process::id(), Instant::now().elapsed().as_nanos()));
    let _ = fs::remove_file(&temp_cache);

    // 1. Cold build
    let start_cold = Instant::now();
    let cold_disp = SkillDispatcher::load_or_build_with_cache(Some(real_skills_dir), Some(&temp_cache));
    let cold_dur = start_cold.elapsed();
    println!("Real-world cold build with {} skills took: {:?}", cold_disp.len(), cold_dur);

    assert!(temp_cache.exists(), "Cache file must exist at {}", temp_cache.display());

    // 2. Hot load from cache
    let start_hot = Instant::now();
    let hot_disp = SkillDispatcher::load_or_build_with_cache(Some(real_skills_dir), Some(&temp_cache));
    let hot_dur = start_hot.elapsed();
    println!("⚡ Real-world hot load with {} skills took: {:?}", hot_disp.len(), hot_dur);

    assert_eq!(cold_disp.len(), hot_disp.len());

    // Cleanup
    let _ = fs::remove_file(&temp_cache);
}
