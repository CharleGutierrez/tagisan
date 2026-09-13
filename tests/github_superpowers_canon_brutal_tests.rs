//! Brutal Integration Tests for 500 GitHub Superpowers Canon in Tagisan (tgs)
//!
//! Verifies:
//! 1. On-Disk Discovery of "github-superpowers-vibe-coder" skill in .ecc/skills
//! 2. Parsing and Invariant Verification of "github-superpowers-architect" agent in .ecc/agents
//! 3. Markdown Canon Completeness (All 500 skills across 10 sections with verified GitHub URLs)
//! 4. Mathematical & Structural Invariants:
//!    - Exact 500 unique item IDs (1 through 500)
//!    - Exactly 50 items per section across all 10 technical sections
//!    - 500 distinct GitHub repository URLs (zero duplicates across entire canon)
//!    - Key flagship projects present across each section
//! 5. High-Concurrency Stress Test across 50 threads

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::ecc::{load_skills_from_dir, EccAgent};

#[test]
fn test_01_github_superpowers_skill_discovery_on_disk() {
    println!("\n=== TEST 1: GitHub Superpowers Skill Discovery on Disk ===");
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_skills_from_dir(skills_dir);
    let skill_opt = loaded.iter().find(|s| s.name == "github-superpowers-vibe-coder");

    assert!(
        skill_opt.is_some(),
        "Expected github-superpowers-vibe-coder to be discovered on disk"
    );

    let skill = skill_opt.unwrap();
    assert!(!skill.description.is_empty(), "Skill description must not be empty");
    assert!(
        skill.instructions.contains("Pillar I"),
        "Skill instructions must contain Pillar I"
    );
    assert!(
        skill.instructions.contains("Pillar X"),
        "Skill instructions must contain Pillar X"
    );
    assert!(
        skill.instructions.contains("The 5 Iron Laws of GitHub Engineering Superpowers"),
        "Skill instructions must contain iron laws"
    );
    println!("  [✓] Discovered skill: {} ({} instructions bytes)", skill.name, skill.instructions.len());
}

#[test]
fn test_02_github_superpowers_architect_agent_parsing() {
    println!("\n=== TEST 2: GitHub Superpowers Architect Agent Parsing ===");
    let agent_path = Path::new(".ecc/agents/github-superpowers-architect.md");
    assert!(agent_path.is_file(), "Agent file .ecc/agents/github-superpowers-architect.md must exist");

    let agent = EccAgent::from_file(agent_path).expect("Failed to parse github-superpowers-architect.md");
    assert_eq!(agent.name, "github-superpowers-architect");
    assert_eq!(agent.recommended_model.as_deref(), Some("deepseek-reasoner"));
    assert!(agent.tools.contains(&"read_file".to_string()));
    assert!(agent.tools.contains(&"write_file".to_string()));
    assert!(agent.tools.contains(&"edit_file".to_string()));
    assert!(agent.tools.contains(&"run_command".to_string()));
    assert!(agent.tools.contains(&"calculator".to_string()));
    assert!(agent.system_prompt.contains("GitHub Superpower Skills Architect"));
    assert!(agent.system_prompt.contains("Diagnostic & Audit Protocol"));
    println!("  [✓] Parsed agent: {} with {} tools and model {:?}", agent.name, agent.tools.len(), agent.recommended_model);
}

#[test]
fn test_03_github_500_skills_document_completeness() {
    println!("\n=== TEST 3: GitHub 500 Skills Document Completeness ===");
    let doc_path = Path::new("docs/TOP_500_GITHUB_SKILLS_FOR_TGS_SUPERPOWERS.md");
    assert!(doc_path.is_file(), "Document docs/TOP_500_GITHUB_SKILLS_FOR_TGS_SUPERPOWERS.md must exist");

    let content = fs::read_to_string(doc_path).expect("Failed to read document");
    assert!(content.len() > 100_000, "Document must exceed 100KB (found {} bytes)", content.len());

    let sections = [
        "Section 1: Formal Verification, SMT Solvers & Provable Mathematics (001 - 050)",
        "Section 2: Linux Kernel, eBPF Telemetry, Low-Latency & Hardware Bypass (051 - 100)",
        "Section 3: Autonomous SWE, AST Mutation, Patch Synthesis & Self-Healing (101 - 150)",
        "Section 4: High-Performance GPU Inference, Tensor Kernels & Quantization (151 - 200)",
        "Section 5: High-Frequency Trading, Financial Engineering & Microstructure (201 - 250)",
        "Section 6: Distributed Databases, Vector Storage & Columnar Analytics (251 - 300)",
        "Section 7: Cloud-Native, Kubernetes Operators, Service Mesh & Chaos Engineering (301 - 350)",
        "Section 8: Binary Exploitation, Security Auditing, Fuzzing & Forensics (351 - 400)",
        "Section 9: Compiler Toolchains, Language Runtimes & Polyglot Virtual Machines (401 - 450)",
        "Section 10: Swarm Intelligence, Meta-Cognition & Autonomous Agentics (451 - 500)",
    ];

    for section in &sections {
        assert!(content.contains(section), "Missing section in document: {}", section);
    }
    println!("  [✓] All 10 technical sections verified in document");
}

#[test]
fn test_04_structural_invariants_and_500_unique_skills() {
    println!("\n=== TEST 4: Structural Invariants & 500 Unique Skills Validation ===");
    let doc_path = Path::new("docs/TOP_500_GITHUB_SKILLS_FOR_TGS_SUPERPOWERS.md");
    let content = fs::read_to_string(doc_path).expect("Failed to read document");

    let mut skill_ids = Vec::new();
    let mut github_urls = HashSet::new();
    let mut section_counts = [0usize; 10];
    let mut current_section: Option<usize> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        for i in 1..=10 {
            let header_prefix = format!("## Section {}:", i);
            if trimmed.starts_with(&header_prefix) {
                current_section = Some(i - 1);
                break;
            }
        }

        if trimmed.starts_with('|') && !trimmed.starts_with("| #") && !trimmed.contains("---") {
            let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
            // Expected cols: ["", "#", "Skill Identifier", "GitHub Project", "Superpower Unlocked in `tgs`", ""]
            if cols.len() >= 5 {
                if let Ok(num) = cols[1].parse::<usize>() {
                    skill_ids.push(num);
                    if let Some(s) = current_section {
                        section_counts[s] += 1;
                    }

                    // Extract URL from markdown link [owner/repo](https://github.com/...)
                    let repo_col = cols[3];
                    if let Some(start_idx) = repo_col.find("https://github.com/") {
                        if let Some(end_idx) = repo_col[start_idx..].find(')') {
                            let url = &repo_col[start_idx..start_idx + end_idx];
                            assert!(
                                !github_urls.contains(url),
                                "Duplicate GitHub URL found: {} (at skill #{})",
                                url,
                                num
                            );
                            github_urls.insert(url.to_string());
                        } else {
                            panic!("Malformed markdown link in col 3: {}", repo_col);
                        }
                    } else {
                        panic!("Missing https://github.com/ link in col 3: {}", repo_col);
                    }
                }
            }
        }
    }

    assert_eq!(skill_ids.len(), 500, "Exactly 500 skills must be parsed");
    assert_eq!(github_urls.len(), 500, "Exactly 500 unique GitHub URLs must be present");

    for (idx, &num) in skill_ids.iter().enumerate() {
        assert_eq!(num, idx + 1, "Skill numbering must be strictly 1..500 without gaps");
    }

    for (s_idx, &count) in section_counts.iter().enumerate() {
        assert_eq!(
            count, 50,
            "Section {} must contain exactly 50 skills, found {}",
            s_idx + 1,
            count
        );
    }

    // Flagship project assertions across sections
    let flagship_repos = [
        "https://github.com/leanprover/lean4",
        "https://github.com/iovisor/bcc",
        "https://github.com/tree-sitter/tree-sitter",
        "https://github.com/vllm-project/vllm",
        "https://github.com/quickfix/quickfix",
        "https://github.com/tikv/tikv",
        "https://github.com/cilium/cilium",
        "https://github.com/AFLplusplus/AFLplusplus",
        "https://github.com/rust-lang/rust",
        "https://github.com/microsoft/autogen",
    ];

    for repo in &flagship_repos {
        assert!(github_urls.contains(*repo), "Flagship repo missing: {}", repo);
    }

    println!("  [✓] Verified 500/500 unique skills, 50 per section, 500 unique GitHub URLs, and all 10 flagship repos");
}

#[test]
fn test_05_multithreaded_concurrency_stress_50_threads() {
    println!("\n=== TEST 5: 50-Thread High-Concurrency Stress Test ===");
    let doc_path = Path::new("docs/TOP_500_GITHUB_SKILLS_FOR_TGS_SUPERPOWERS.md");
    let content = Arc::new(fs::read_to_string(doc_path).expect("Failed to read document"));
    let agent_path = Path::new(".ecc/agents/github-superpowers-architect.md");
    let agent_content = Arc::new(fs::read_to_string(agent_path).expect("Failed to read agent"));

    let success_counter = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();
    let mut handles = Vec::with_capacity(50);

    for _ in 0..50 {
        let doc_clone = Arc::clone(&content);
        let agent_clone = Arc::clone(&agent_content);
        let counter_clone = Arc::clone(&success_counter);

        let handle = thread::spawn(move || {
            assert!(doc_clone.contains("leanprover/lean4"));
            assert!(doc_clone.contains("iovisor/bcc"));
            assert!(doc_clone.contains("vllm-project/vllm"));
            assert!(doc_clone.contains("microsoft/autogen"));
            assert!(agent_clone.contains("deepseek-reasoner"));
            assert!(agent_clone.contains("github-superpowers-architect"));

            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Thread panicked during concurrent stress test");
    }

    let elapsed = start.elapsed();
    let count = success_counter.load(Ordering::SeqCst);
    assert_eq!(count, 50, "All 50 threads must complete successfully");
    println!("  [✓] 50 threads executed stress validation in {:.2?} ({} ops/sec)", elapsed, (50.0 / elapsed.as_secs_f64()) as usize);
}
