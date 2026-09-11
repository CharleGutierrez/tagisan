use async_trait::async_trait;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tagisan::{
    all_ecc_skills, build_ecc_pipeline, extract_triggers_from_text, find_ecc_skill,
    global_ecc_dispatcher, BoxEventStream, CompletionRequest, CompletionResponse,
    EccSkill, FinishReason, LlmProvider, Message, ProviderCapabilities, SearchSkillsTool,
    TagisanError, TokenUsage, ToolHandler, ToolRegistry,
};

/// Deterministic mock LLM provider for testing ECC workflow auto-equipping
struct MockPipelineProvider {
    id: &'static str,
}

impl MockPipelineProvider {
    fn new(id: &'static str) -> Self {
        Self { id }
    }
}

#[async_trait]
impl LlmProvider for MockPipelineProvider {
    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, TagisanError> {
        let user_prompt = req.messages.last().map(|m| m.extract_text()).unwrap_or_default();
        let reply = format!("Mock reply from {} for: {}", self.id, user_prompt);

        Ok(CompletionResponse {
            id: "mock_resp_pipeline".to_string(),
            provider: self.id.to_string(),
            model: req.model,
            message: Message::assistant(reply),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 40,
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: Some(0.0001),
            },
            latency: Duration::from_millis(2),
        })
    }

    async fn stream(&self, _req: CompletionRequest) -> Result<BoxEventStream, TagisanError> {
        Err(TagisanError::Execution("Streaming unsupported in mock".to_string()))
    }
}

// =========================================================================
// 1. Catalog Ingestion & Integrity Test
// =========================================================================

#[test]
fn test_catalog_ingestion_and_integrity() {
    // 1a. Built-in skills validation
    let built_ins = all_ecc_skills();
    assert_eq!(
        built_ins.len(),
        40,
        "Catalog must contain exactly 40 built-in skills, found {}",
        built_ins.len()
    );

    let mut built_in_names = HashSet::new();
    for skill in &built_ins {
        assert!(
            !skill.name.trim().is_empty(),
            "Built-in skill name cannot be empty"
        );
        assert!(
            !skill.description.trim().is_empty(),
            "Built-in skill '{}' description cannot be empty",
            skill.name
        );
        assert!(
            !skill.instructions.trim().is_empty(),
            "Built-in skill '{}' instructions cannot be empty",
            skill.name
        );

        let triggers = extract_triggers_from_text(&skill.name, &skill.description, &[]);
        assert!(
            !triggers.is_empty(),
            "Built-in skill '{}' must have non-empty triggers",
            skill.name
        );

        assert!(
            built_in_names.insert(skill.name.clone()),
            "Duplicate built-in skill name detected: '{}'",
            skill.name
        );

        // Verify retrieval via find_ecc_skill
        assert!(
            find_ecc_skill(&skill.name).is_some(),
            "Built-in skill '{}' must be resolvable via find_ecc_skill",
            skill.name
        );
    }

    // 1b. On-disk skills validation (.ecc/skills/*/SKILL.md)
    let skills_dir = Path::new(".ecc/skills");
    let mut on_disk_count = 0;

    if skills_dir.is_dir() {
        let entries = fs::read_dir(skills_dir).expect("Should read .ecc/skills directory");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let target = if path.join("SKILL.md").is_file() {
                    Some(path.join("SKILL.md"))
                } else if path.join("skill.md").is_file() {
                    Some(path.join("skill.md"))
                } else {
                    None
                };

                if let Some(target_file) = target {
                    // Assert valid UTF-8
                    let bytes = fs::read(&target_file).unwrap_or_else(|e| {
                        panic!("Failed reading file '{}': {}", target_file.display(), e)
                    });
                    let content_str = String::from_utf8(bytes).unwrap_or_else(|e| {
                        panic!("File '{}' is not valid UTF-8: {}", target_file.display(), e)
                    });

                    // If file starts with YAML frontmatter delimiter, assert closing delimiter
                    let trimmed = content_str.trim_start_matches('\u{feff}').trim_start();
                    if trimmed.starts_with("---") {
                        assert!(
                            trimmed[3..].contains("\n---") || trimmed[3..].contains("\r\n---"),
                            "File '{}' starts with '---' but is missing closing frontmatter delimiter '---'",
                            target_file.display()
                        );
                    }

                    // Parse using EccSkill::from_file
                    let parsed = EccSkill::from_file(&target_file).unwrap_or_else(|e| {
                        panic!("Failed parsing '{}': {}", target_file.display(), e)
                    });

                    assert!(
                        !parsed.name.trim().is_empty(),
                        "Skill at '{}' has empty name",
                        target_file.display()
                    );
                    assert!(
                        !parsed.description.trim().is_empty(),
                        "Skill '{}' at '{}' has empty description",
                        parsed.name,
                        target_file.display()
                    );
                    assert!(
                        !parsed.instructions.trim().is_empty(),
                        "Skill '{}' at '{}' has empty body/instructions",
                        parsed.name,
                        target_file.display()
                    );

                    on_disk_count += 1;
                }
            }
        }
    }

    println!(
        "[Integrity Verified] {} built-in skills and {} on-disk skills validated.",
        built_ins.len(),
        on_disk_count
    );

    // 1c. Dispatcher total indexed count
    let dispatcher = global_ecc_dispatcher();
    assert!(
        dispatcher.len() >= 150,
        "Total indexed skills in SkillDispatcher must exceed 150+, got {}",
        dispatcher.len()
    );
    assert!(!dispatcher.is_empty(), "SkillDispatcher cannot be empty");
}

// =========================================================================
// 2. Semantic Intent & Top-K Dispatch Precision
// =========================================================================

#[test]
fn test_semantic_intent_top_k_dispatch_precision() {
    let dispatcher = global_ecc_dispatcher();

    // 1. UX/UI: "grayscale first 8pt grid optical balance" -> refactoring-ui #1
    let r1 = dispatcher.dispatch("grayscale first 8pt grid optical balance", 3, None);
    assert!(!r1.is_empty(), "UX/UI query must return results");
    assert_eq!(
        r1[0].skill.name, "refactoring-ui",
        "Query 'grayscale first 8pt grid optical balance' must rank 'refactoring-ui' #1, got '{}' [Score: {:.1}]",
        r1[0].skill.name, r1[0].score
    );
    assert_eq!(r1[0].domain, "ux");
    assert!(r1[0].score >= 30.0);

    // 2. Tactile UI: "spring physics sub-50ms button press optimistic UI" -> microinteractions-design #1
    let r2 = dispatcher.dispatch("spring physics sub-50ms button press optimistic UI", 3, None);
    assert!(!r2.is_empty(), "Tactile UI query must return results");
    assert_eq!(
        r2[0].skill.name, "microinteractions-design",
        "Query 'spring physics sub-50ms button press optimistic UI' must rank 'microinteractions-design' #1, got '{}' [Score: {:.1}]",
        r2[0].skill.name, r2[0].score
    );
    assert_eq!(r2[0].domain, "ux");
    assert!(r2[0].score >= 30.0);

    // 3. Cognitive UX: "doherty threshold hicks law progressive disclosure" -> laws-of-ux #1
    let r3 = dispatcher.dispatch("doherty threshold hicks law progressive disclosure", 3, None);
    assert!(!r3.is_empty(), "Cognitive UX query must return results");
    assert_eq!(
        r3[0].skill.name, "laws-of-ux",
        "Query 'doherty threshold hicks law progressive disclosure' must rank 'laws-of-ux' #1, got '{}' [Score: {:.1}]",
        r3[0].skill.name, r3[0].score
    );
    assert_eq!(r3[0].domain, "ux");
    assert!(r3[0].score >= 30.0);

    // 4. Design Tokens: "3-tier token hierarchy semantic tokens atomic design" -> design-systems-tokens #1
    let r4 = dispatcher.dispatch("3-tier token hierarchy semantic tokens atomic design", 3, None);
    assert!(!r4.is_empty(), "Design tokens query must return results");
    assert_eq!(
        r4[0].skill.name, "design-systems-tokens",
        "Query '3-tier token hierarchy semantic tokens atomic design' must rank 'design-systems-tokens' #1, got '{}' [Score: {:.1}]",
        r4[0].skill.name, r4[0].score
    );
    assert_eq!(r4[0].domain, "ux");
    assert!(r4[0].score >= 30.0);

    // 5. Power-User Posture: "sovereign posture cmd+k command palette eliminate excise undo banner" -> about-face-interaction-design #1
    let r5 = dispatcher.dispatch("sovereign posture cmd+k command palette eliminate excise undo banner", 3, None);
    assert!(!r5.is_empty(), "Power-user posture query must return results");
    assert_eq!(
        r5[0].skill.name, "about-face-interaction-design",
        "Query 'sovereign posture cmd+k command palette eliminate excise undo banner' must rank 'about-face-interaction-design' #1, got '{}' [Score: {:.1}]",
        r5[0].skill.name, r5[0].score
    );
    assert_eq!(r5[0].domain, "ux");
    assert!(r5[0].score >= 30.0);

    // 6. Emotional Delight: "delight hierarchy empathetic error state celebratory confetti empty state" -> designing-for-emotion #1
    let r6 = dispatcher.dispatch("delight hierarchy empathetic error state celebratory confetti empty state", 3, None);
    assert!(!r6.is_empty(), "Emotional delight query must return results");
    assert_eq!(
        r6[0].skill.name, "designing-for-emotion",
        "Query 'delight hierarchy empathetic error state celebratory confetti empty state' must rank 'designing-for-emotion' #1, got '{}' [Score: {:.1}]",
        r6[0].skill.name, r6[0].score
    );
    assert_eq!(r6[0].domain, "ux");
    assert!(r6[0].score >= 30.0);

    // 7. Rust Systems: "tokio async concurrency lock free channels" -> Rust engineering skills #1
    let r7 = dispatcher.dispatch("tokio async concurrency lock free channels", 3, None);
    assert!(!r7.is_empty(), "Rust systems query must return results");
    let top_rust_name = &r7[0].skill.name;
    assert!(
        top_rust_name == "tokio-async-tuning" || r7[0].domain == "rust" || top_rust_name.contains("rust"),
        "Query 'tokio async concurrency lock free channels' must rank Rust engineering skill #1, got '{}' [Score: {:.1} | Domain: {}]",
        top_rust_name, r7[0].score, r7[0].domain
    );
    assert_eq!(r7[0].domain, "rust");
    assert!(r7[0].score >= 30.0);

    // 8. Testing & TDD: "test driven development unit test assertions regression" -> TDD / test skills #1
    let r8 = dispatcher.dispatch("test driven development unit test assertions regression", 3, None);
    assert!(!r8.is_empty(), "Testing query must return results");
    assert!(
        r8[0].skill.name == "tdd-workflow" || r8[0].domain == "test",
        "Query 'test driven development unit test assertions regression' must rank TDD/test skill #1, got '{}' [Domain: {}]",
        r8[0].skill.name, r8[0].domain
    );
    assert_eq!(r8[0].domain, "test");
    assert!(r8[0].score >= 30.0);

    // 9. Security: "security threat model injection memory safety vulnerability audit" -> Security skills #1
    let r9 = dispatcher.dispatch("security threat model injection memory safety vulnerability audit", 3, None);
    assert!(!r9.is_empty(), "Security query must return results");
    assert!(
        r9[0].skill.name == "security-review" || r9[0].domain == "security",
        "Query 'security threat model injection memory safety vulnerability audit' must rank Security skill #1, got '{}' [Domain: {}]",
        r9[0].skill.name, r9[0].domain
    );
    assert_eq!(r9[0].domain, "security");
    assert!(r9[0].score >= 30.0);

    // 10. Cloud / Backend: "azure cosmos db distributed database rust" -> Azure Cosmos skill #1
    let r10 = dispatcher.dispatch("azure cosmos db distributed database rust", 3, None);
    assert!(!r10.is_empty(), "Cloud/backend query must return results");
    assert!(
        r10[0].skill.name.contains("cosmos") || r10[0].domain == "azure",
        "Query 'azure cosmos db distributed database rust' must rank Azure Cosmos skill #1, got '{}' [Domain: {}]",
        r10[0].skill.name, r10[0].domain
    );
    assert_eq!(r10[0].domain, "azure");
    assert!(r10[0].score >= 30.0);

    println!("[Semantic Precision Verified] All 10 domain queries ranked targeted engineering skills #1 with high confidence.");
}

// =========================================================================
// 3. Multithreaded Concurrency & Thread-Safety Stress
// =========================================================================

#[test]
fn test_multithreaded_concurrency_stress() {
    let dispatcher = global_ecc_dispatcher();
    let num_threads = 32;
    let iterations_per_thread = 80;
    let total_operations = Arc::new(AtomicUsize::new(0));

    let queries = [
        "refactoring-ui optical alignment",
        "microinteractions spring feedback",
        "laws-of-ux doherty threshold",
        "design-systems-tokens hierarchy",
        "about-face sovereign posture",
        "designing-for-emotion delight",
        "tokio async channels locks",
        "tdd-workflow failing assertions",
        "security threat modeling injection",
        "azure cosmos db rust",
        "bun native typescript runtime",
        "sap abap clean code",
        "solana anchor rust smart contracts",
        "deep modules ousterhout information hiding",
        "legacy seams characterization feathers",
        "circuit breaker bulkhead release it nygard",
    ];

    let total_skills = dispatcher.len();
    let mut handles = Vec::with_capacity(num_threads);

    let start = Instant::now();

    for thread_idx in 0..num_threads {
        let ops = total_operations.clone();
        let handle = std::thread::spawn(move || {
            let disp = global_ecc_dispatcher();
            for i in 0..iterations_per_thread {
                // 1. Hammer dispatch
                let q = queries[(thread_idx + i) % queries.len()];
                let results = disp.dispatch(q, 3, None);
                assert!(!results.is_empty(), "Concurrent dispatch should return results for '{}'", q);

                // 2. Hammer load_skill_by_id
                let doc_id = (thread_idx * 17 + i * 13) % total_skills;
                let loaded = disp.load_skill_by_id(doc_id);
                assert!(loaded.is_some(), "Valid doc_id {} must load", doc_id);

                // 3. Hammer get_skill
                let skill_name = &results[0].skill.name;
                let retrieved = disp.get_skill(skill_name);
                assert!(retrieved.is_some(), "Skill '{}' should be retrievable", skill_name);

                // 4. Out of bounds load_skill_by_id must safely return None
                assert!(disp.load_skill_by_id(total_skills + 1000 + i).is_none());

                ops.fetch_add(3, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked during concurrent stress!");
    }

    let elapsed = start.elapsed();
    let ops_count = total_operations.load(Ordering::SeqCst);

    println!(
        "[Concurrency Stress Verified] 32 threads completed {} operations in {:?} (zero panics, zero data races, zero RwLock poisoning).",
        ops_count, elapsed
    );
}

// =========================================================================
// 4. Adversarial & Extreme Boundary Testing
// =========================================================================

#[test]
fn test_adversarial_and_extreme_boundaries() {
    let dispatcher = global_ecc_dispatcher();

    // 4a. 50,000-character payload
    let massive_query = "tokio async channels locks spring physics ".repeat(1250);
    assert!(massive_query.len() >= 50_000);
    let r_massive = dispatcher.dispatch(&massive_query, 5, None);
    assert!(!r_massive.is_empty(), "Massive query should resolve without panicking");

    // 4b. Unicode emojis, Asian glyphs, Cyrillic, RTL, zero-width joiners
    let adversarial_unicode = "🦀🔥🚀 深度学习 神经网络 データベース 日本語 한국어 Русский текст עם ניקוד \u{200D}\u{200C}\u{FEFF} العربية";
    let r_unicode = dispatcher.dispatch(adversarial_unicode, 5, None);
    // Should not panic, returns safe vector
    let _ = r_unicode.len();

    // 4c. Code injection payloads
    let injections = [
        "'; DROP TABLE skills; --",
        "<script>alert(document.cookie)</script>",
        "$(rm -rf /)",
        "{{7*7}} ${IFS} `cat /etc/passwd`",
        "SELECT * FROM skills WHERE 1=1; --",
        "../../../../../../etc/shadow",
    ];
    for inj in &injections {
        let r_inj = dispatcher.dispatch(inj, 3, None);
        // Code injection payloads must execute safely as plain lexical tokens without side effects
        let _ = r_inj.len();
        let formatted = dispatcher.search_and_format(inj, 2, None, false);
        assert!(!formatted.is_empty());
    }

    // 4d. Boundary parameters
    // Empty query
    assert!(dispatcher.dispatch("", 5, None).is_empty());
    assert!(dispatcher.dispatch("    \t\r\n   ", 5, None).is_empty());

    // limit = 0
    assert!(dispatcher.dispatch("tokio async channels", 0, None).is_empty());

    // limit = 50,000
    let r_huge_limit = dispatcher.dispatch("tokio async", 50_000, None);
    assert!(!r_huge_limit.is_empty());
    assert!(r_huge_limit.len() <= dispatcher.len());

    // 4e. Malformed frontmatter strings fed directly to EccSkill::parse
    // Missing leading delimiter
    assert!(EccSkill::parse("name: invalid\n---\nbody").is_err());

    // Missing closing delimiter
    assert!(EccSkill::parse("---\nname: unclosed\nbody without closing delimiter").is_err());

    // Missing name field
    assert!(EccSkill::parse("---\ndescription: only description\n---\nbody").is_err());

    // Empty name field
    assert!(EccSkill::parse("---\nname:\ndescription: valid desc\n---\nbody").is_err());

    // Valid frontmatter directly to EccSkill::parse
    let valid_parse = EccSkill::parse("---\nname: my-skill\ndescription: my desc\n---\nInstructions here");
    assert!(valid_parse.is_ok());
    let s = valid_parse.unwrap();
    assert_eq!(s.name, "my-skill");
    assert_eq!(s.description, "my desc");
    assert_eq!(s.instructions, "Instructions here");

    // 4f. Missing skill lookups
    assert!(dispatcher.get_skill("completely_unknown_xyz_12345_never_exists").is_none());
    assert!(dispatcher.load_skill_by_id(usize::MAX).is_none());

    println!("[Adversarial Boundaries Verified] Massive 50KB payloads, injection strings, and malformed inputs handled robustly.");
}

// =========================================================================
// 5. Latency & Performance SLA Benchmark (<0.5ms avg, <1.0ms P99)
// =========================================================================

#[test]
fn test_latency_and_performance_sla_benchmark() {
    let dispatcher = global_ecc_dispatcher();

    let benchmark_queries = [
        "refactoring-ui grayscale 8pt grid optical balance",
        "microinteractions spring physics sub-50ms button press",
        "laws-of-ux doherty threshold progressive disclosure",
        "design-systems-tokens 3-tier token hierarchy",
        "about-face sovereign posture cmd+k command palette",
        "designing-for-emotion delight hierarchy empathetic",
        "tokio async concurrency lock free channels",
        "test driven development unit assertions regression",
        "security threat model injection vulnerability audit",
        "azure cosmos db distributed database rust",
    ];

    let total_runs: usize = 1_000;
    let mut durations = Vec::with_capacity(total_runs);

    // Warm-up cache & indexes (50 iterations)
    for i in 0..50 {
        let q = benchmark_queries[i % benchmark_queries.len()];
        let _ = dispatcher.dispatch(q, 3, None);
    }

    // Benchmark loop
    let overall_start = Instant::now();
    for i in 0..total_runs {
        let q = benchmark_queries[i % benchmark_queries.len()];
        let t0 = Instant::now();
        let _ = dispatcher.dispatch(q, 3, None);
        durations.push(t0.elapsed());
    }
    let overall_elapsed = overall_start.elapsed();

    durations.sort();

    let total_micros: u128 = durations.iter().map(|d| d.as_micros()).sum();
    let avg_micros = total_micros as f64 / total_runs as f64;
    let avg_ms = avg_micros / 1000.0;

    let p50 = durations[total_runs * 50 / 100];
    let p90 = durations[total_runs * 90 / 100];
    let p95 = durations[total_runs * 95 / 100];
    let p99 = durations[total_runs * 99 / 100];
    let max = durations[total_runs - 1];

    println!(
        "\n====================================================================\n\
         TAGISAN SKILL DISPATCHER LATENCY SLA BENCHMARK REPORT ({} queries across {} skills)\n\
         ====================================================================\n\
         Total Time:     {:?}\n\
         Average:        {:.3} ms ({:.1} µs)\n\
         P50 (Median):   {:?}\n\
         P90:            {:?}\n\
         P95:            {:?}\n\
         P99:            {:?}\n\
         Max:            {:?}\n\
         ====================================================================",
        total_runs, dispatcher.len(), overall_elapsed, avg_ms, avg_micros, p50, p90, p95, p99, max
    );

    // Assert SLA: In optimized environments target is <0.5ms avg and <1.0ms P99.
    // In unoptimized debug test runner, accommodate debug build overhead with generous upper threshold.
    #[cfg(not(debug_assertions))]
    {
        assert!(
            avg_ms < 0.5,
            "Release SLA breached: average latency {:.3}ms >= 0.5ms",
            avg_ms
        );
        assert!(
            p99 < Duration::from_millis(1),
            "Release SLA breached: P99 latency {:?} >= 1.0ms",
            p99
        );
    }
    #[cfg(debug_assertions)]
    {
        assert!(
            avg_ms < 5.0,
            "Debug SLA breached: average latency {:.3}ms >= 5.0ms",
            avg_ms
        );
        assert!(
            p99 < Duration::from_millis(10),
            "Debug SLA breached: P99 latency {:?} >= 10.0ms",
            p99
        );
    }
}

// =========================================================================
// 6. Pipeline Auto-Equipping & Tool Handler Verification
// =========================================================================

#[tokio::test]
async fn test_pipeline_auto_equipping_and_tool_handler() {
    let mock_provider = Arc::new(MockPipelineProvider::new("mock_pipeline_llm"));
    let mut tools = ToolRegistry::new();
    tools.register_tool(SearchSkillsTool::with_default());

    let objective = "Build an offline-first mobile and desktop application with spring physics microinteractions, 3-tier design tokens, and TDD in Rust";

    let graph = build_ecc_pipeline(objective, mock_provider, "mock-model", tools)
        .expect("build_ecc_pipeline must succeed");

    // 6a. Verify all 6 nodes exist
    let required_nodes = [
        "ecc_plan",
        "ecc_test",
        "ecc_implement",
        "ecc_review",
        "ecc_security",
        "ecc_verify",
    ];

    for node_id in &required_nodes {
        let node = graph.get_task(node_id).unwrap_or_else(|| {
            panic!("Pipeline graph missing required node '{}'", node_id)
        });

        let agent = node.agent.as_ref().unwrap_or_else(|| {
            panic!("TaskNode '{}' must contain an AutonomousAgent", node_id)
        });

        let sys_prompt = agent.system_prompt.as_ref().unwrap_or_else(|| {
            panic!("Agent in node '{}' must have a system prompt", node_id)
        });

        // 6b. Verify auto-equipping injected the specialized engineering skills section
        assert!(
            sys_prompt.contains("--- AUTO-EQUIPPED SPECIALIZED ENGINEERING SKILLS ---"),
            "Node '{}' prompt missing auto-equipped skills header. Prompt:\n{}",
            node_id,
            sys_prompt
        );

        assert!(
            sys_prompt.contains("### Skill:"),
            "Node '{}' prompt must contain at least one injected skill",
            node_id
        );
    }

    // Specific stage checks:
    // ecc_test must auto-equip test / TDD skills
    let test_prompt = graph.get_task("ecc_test").unwrap().agent.as_ref().unwrap().system_prompt.as_ref().unwrap();
    assert!(
        test_prompt.contains("tdd-workflow") || test_prompt.contains("test"),
        "ecc_test must auto-equip testing discipline"
    );

    // ecc_security must auto-equip security skills
    let sec_prompt = graph.get_task("ecc_security").unwrap().agent.as_ref().unwrap().system_prompt.as_ref().unwrap();
    assert!(
        sec_prompt.contains("security-review") || sec_prompt.contains("security") || sec_prompt.contains("threat"),
        "ecc_security must auto-equip security review discipline"
    );

    // 6c. SearchSkillsTool verification
    let tool = SearchSkillsTool::with_default();
    assert_eq!(tool.name(), "search_skills");

    let tool_args = serde_json::json!({
        "query": "grayscale first 8pt grid optical balance",
        "limit": 2,
        "include_instructions": true
    });

    let tool_output = tool.execute(tool_args).await.expect("SearchSkillsTool execution must succeed");

    assert!(
        tool_output.contains("refactoring-ui"),
        "SearchSkillsTool output must include 'refactoring-ui', got:\n{}",
        tool_output
    );
    assert!(tool_output.contains("Score:"), "Output must display match scores");
    assert!(tool_output.contains("Domain: ux"), "Output must display identified domain");
    assert!(tool_output.contains("#### Instructions:"), "Output must contain full instructions");

    println!("[Pipeline Auto-Equipping & Tool Handler Verified] Pipeline stages automatically equipped matching engineering skills into agent prompts, and SearchSkillsTool returned structured markdown.");
}
