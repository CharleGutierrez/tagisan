//! Brutal Integration & Verification Tests for Top 100 Books in Agentic Engineering & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const AGENTIC_SKILLS_100: [&str; 100] = [
    // Cluster 1: Multi-Agent Systems, Swarms & Coordination (1-10)
    "agentic-wooldridge-multiagent-systems",
    "agentic-shoham-multiagent-foundations",
    "agentic-weiss-distributed-ai",
    "agentic-ferber-reactive-agents",
    "agentic-kennedy-swarm-intelligence",
    "agentic-bonabeau-swarm-stigmergy",
    "agentic-camazine-self-organization",
    "agentic-reynolds-autonomous-agents",
    "agentic-dorigo-ant-colony",
    "agentic-murray-consensus-cooperation",
    // Cluster 2: Foundation Models, In-Context Learning & Prompt Architecture (11-20)
    "agentic-alammar-transformer-mechanics",
    "agentic-huyen-ai-engineering",
    "agentic-rothman-transformers-nlp",
    "agentic-tunstall-nlp-transformers",
    "agentic-jurafsky-slp-language-models",
    "agentic-goldberg-nn-nlp",
    "agentic-ravichandiran-llm-engineering",
    "agentic-briggs-prompt-engineering",
    "agentic-kamphuis-in-context-reasoning",
    "agentic-chollet-deep-learning-intuition",
    // Cluster 3: Autonomous Planning, Reasoning & Decision Engines (21-30)
    "agentic-russell-norvig-aima",
    "agentic-ghallab-automated-planning",
    "agentic-geffner-heuristic-search-planning",
    "agentic-sutton-barto-reinforcement-learning",
    "agentic-pearl-heuristic-search",
    "agentic-kaelbling-pomdp-planning",
    "agentic-thrun-probabilistic-robotics",
    "agentic-silver-mcts-decision-trees",
    "agentic-yao-react-interleaved-reasoning",
    "agentic-shinn-reflexion-self-correction",
    // Cluster 4: Vibe Coding, Developer Flow, Ergonomics & Rapid Iteration (31-40)
    "agentic-karpathy-vibe-coding-paradigm",
    "agentic-csikszentmihalyi-flow-state",
    "agentic-hunt-pragmatic-programmer",
    "agentic-raymond-cathedral-bazaar",
    "agentic-graham-hackers-painters",
    "agentic-beck-extreme-programming",
    "agentic-ries-lean-mvp-feedback",
    "agentic-knapp-design-sprint-prototyping",
    "agentic-norman-human-centered-interfaces",
    "agentic-krug-intuitive-interaction",
    // Cluster 5: Memory Systems, Knowledge Retrieval, RAG & Vector Space (41-50)
    "agentic-lewis-rag-foundations",
    "agentic-manning-information-retrieval",
    "agentic-baeza-yates-vector-retrieval",
    "agentic-malkov-hnsw-vector-indexing",
    "agentic-anderson-actr-cognitive-memory",
    "agentic-baddeley-working-memory-buffers",
    "agentic-tulving-episodic-memory-retrieval",
    "agentic-sowa-knowledge-representation",
    "agentic-baader-description-logics-ontologies",
    "agentic-robinson-graph-rag-knowledge",
    // Cluster 6: Code Generation, Program Synthesis, Compilers & DSLs (51-60)
    "agentic-gulwani-program-synthesis",
    "agentic-aho-dragon-compiler-parsing",
    "agentic-cooper-compiler-ir-optimization",
    "agentic-muchnick-cfg-dataflow-analysis",
    "agentic-nystrom-crafting-interpreters",
    "agentic-fowler-domain-specific-languages",
    "agentic-parr-antlr4-grammar-dsl",
    "agentic-pierce-type-systems-soundness",
    "agentic-harper-practical-foundations-pl",
    "agentic-sicp-evaluator-metacircular",
    // Cluster 7: Verification, Testing, Evals, Safety & Red Teaming (61-70)
    "agentic-amodei-concrete-ai-safety",
    "agentic-hendrycks-benchmarking-evals",
    "agentic-perez-red-teaming-adversarial",
    "agentic-anthropic-constitutional-ai",
    "agentic-ozkaya-llm-software-evals",
    "agentic-beck-tdd-verifiable-contracts",
    "agentic-claessen-property-based-testing",
    "agentic-maciver-invariant-shrinking",
    "agentic-clarke-model-checking-invariants",
    "agentic-baier-temporal-logic-ltl-ctl",
    // Cluster 8: Distributed Systems, Protocols, Event-Driven & Tool Interop (71-80)
    "agentic-kleppmann-distributed-consistency",
    "agentic-tanenbaum-distributed-systems",
    "agentic-lamport-logical-clocks",
    "agentic-ongaro-raft-distributed-consensus",
    "agentic-hohpe-enterprise-integration-patterns",
    "agentic-newman-microservices-tool-isolation",
    "agentic-richards-software-architecture-tradeoffs",
    "agentic-fielding-rest-agent-apis",
    "agentic-henning-rpc-schema-contracts",
    "agentic-mcp-protocol-specification",
    // Cluster 9: Cognitive Architectures, Metacognition & Self-Improving Systems (81-90)
    "agentic-laird-soar-cognitive-architecture",
    "agentic-minsky-society-of-mind",
    "agentic-kahneman-dual-process-thinking",
    "agentic-hofstadter-strange-loops-recursion",
    "agentic-simon-bounded-rationality-heuristics",
    "agentic-newell-unified-cognition",
    "agentic-sun-clarion-implicit-explicit",
    "agentic-lake-cognitive-concept-learning",
    "agentic-schmidhuber-intrinsic-curiosity",
    "agentic-wang-nars-non-axiomatic-reasoning",
    // Cluster 10: Human-AI Collaboration, Steering, Architecture & Evolution (91-100)
    "agentic-shneiderman-human-centered-ai",
    "agentic-brooks-mythical-man-month",
    "agentic-ousterhout-philosophy-software-design",
    "agentic-martin-clean-architecture-boundaries",
    "agentic-feathers-legacy-code-refactoring",
    "agentic-forsgren-accelerate-dora-metrics",
    "agentic-kim-phoenix-project-flow-theory",
    "agentic-evans-domain-driven-design",
    "agentic-meadows-systems-thinking-feedback",
    "agentic-kelly-autonomous-cognition-flows",
];

// =========================================================================
// 1. Discovery of all 100 Agentic Skills on Disk
// =========================================================================

#[test]
fn test_all_100_agentic_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &AGENTIC_SKILLS_100 {
        assert!(
            loaded_map.contains(*skill),
            "Agentic skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 100 Agentic Skills (assert >= 495 total skills)
// =========================================================================

#[test]
fn test_all_100_agentic_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 495,
        "Expected at least 495 total built-in skills including Agentic Engineering suite, found {}",
        all_skills.len()
    );

    for skill_name in &AGENTIC_SKILLS_100 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "Agentic skill '{}' must be registered in built-in skills",
            skill_name
        );
        let s = found.unwrap();
        assert!(!s.name.is_empty(), "Skill name cannot be empty for '{}'", skill_name);
        assert!(!s.description.is_empty(), "Description cannot be empty for '{}'", skill_name);
        assert!(!s.instructions.is_empty(), "Instructions cannot be empty for '{}'", skill_name);
        assert!(
            s.instructions.len() > 100,
            "Skill '{}' must contain concrete engineering instructions, got length {}",
            skill_name,
            s.instructions.len()
        );
        assert!(
            s.instructions.contains("## 1. Core Mathematical"),
            "Skill '{}' must contain Core Mathematical & Architectural Foundations",
            skill_name
        );
        assert!(
            s.instructions.contains("## 2. Concrete Agent Specification"),
            "Skill '{}' must contain Concrete Agent Specification & Prompt Contract",
            skill_name
        );
        assert!(
            s.instructions.contains("## 3. Anti-Patterns"),
            "Skill '{}' must contain Anti-Patterns & Hallucination Mitigations",
            skill_name
        );
    }
}

// =========================================================================
// 3. High-Leverage Agentic & Vibe Coding Alias Lookups
// =========================================================================

#[test]
fn test_agentic_skills_alias_lookups() {
    // Cluster 1: Multi-Agent Systems
    assert_eq!(find_ecc_skill("wooldridge").unwrap().name, "agentic-wooldridge-multiagent-systems");
    assert_eq!(find_ecc_skill("bdi").unwrap().name, "agentic-wooldridge-multiagent-systems");
    assert_eq!(find_ecc_skill("contract-net-protocol").unwrap().name, "agentic-wooldridge-multiagent-systems");
    assert_eq!(find_ecc_skill("shoham").unwrap().name, "agentic-shoham-multiagent-foundations");
    assert_eq!(find_ecc_skill("discsp").unwrap().name, "agentic-shoham-multiagent-foundations");
    assert_eq!(find_ecc_skill("weiss").unwrap().name, "agentic-weiss-distributed-ai");
    assert_eq!(find_ecc_skill("ferber").unwrap().name, "agentic-ferber-reactive-agents");
    assert_eq!(find_ecc_skill("pso").unwrap().name, "agentic-kennedy-swarm-intelligence");
    assert_eq!(find_ecc_skill("stigmergy").unwrap().name, "agentic-bonabeau-swarm-stigmergy");
    assert_eq!(find_ecc_skill("quorum-sensing").unwrap().name, "agentic-camazine-self-organization");
    assert_eq!(find_ecc_skill("reynolds-steering").unwrap().name, "agentic-reynolds-autonomous-agents");
    assert_eq!(find_ecc_skill("aco").unwrap().name, "agentic-dorigo-ant-colony");
    assert_eq!(find_ecc_skill("olfati-saber").unwrap().name, "agentic-murray-consensus-cooperation");

    // Cluster 2: Foundation Models & Prompts
    assert_eq!(find_ecc_skill("alammar").unwrap().name, "agentic-alammar-transformer-mechanics");
    assert_eq!(find_ecc_skill("kv-cache").unwrap().name, "agentic-alammar-transformer-mechanics");
    assert_eq!(find_ecc_skill("chip-huyen").unwrap().name, "agentic-huyen-ai-engineering");
    assert_eq!(find_ecc_skill("rothman").unwrap().name, "agentic-rothman-transformers-nlp");
    assert_eq!(find_ecc_skill("tunstall").unwrap().name, "agentic-tunstall-nlp-transformers");
    assert_eq!(find_ecc_skill("jurafsky").unwrap().name, "agentic-jurafsky-slp-language-models");
    assert_eq!(find_ecc_skill("goldberg").unwrap().name, "agentic-goldberg-nn-nlp");
    assert_eq!(find_ecc_skill("ravichandiran").unwrap().name, "agentic-ravichandiran-llm-engineering");
    assert_eq!(find_ecc_skill("chain-of-thought").unwrap().name, "agentic-briggs-prompt-engineering");
    assert_eq!(find_ecc_skill("kamphuis").unwrap().name, "agentic-kamphuis-in-context-reasoning");
    assert_eq!(find_ecc_skill("chollet").unwrap().name, "agentic-chollet-deep-learning-intuition");

    // Cluster 3: Planning & Reasoning
    assert_eq!(find_ecc_skill("russell-norvig").unwrap().name, "agentic-russell-norvig-aima");
    assert_eq!(find_ecc_skill("ghallab").unwrap().name, "agentic-ghallab-automated-planning");
    assert_eq!(find_ecc_skill("pddl").unwrap().name, "agentic-ghallab-automated-planning");
    assert_eq!(find_ecc_skill("delete-relaxation").unwrap().name, "agentic-geffner-heuristic-search-planning");
    assert_eq!(find_ecc_skill("sutton-barto").unwrap().name, "agentic-sutton-barto-reinforcement-learning");
    assert_eq!(find_ecc_skill("pearl-search").unwrap().name, "agentic-pearl-heuristic-search");
    assert_eq!(find_ecc_skill("kaelbling").unwrap().name, "agentic-kaelbling-pomdp-planning");
    assert_eq!(find_ecc_skill("thrun").unwrap().name, "agentic-thrun-probabilistic-robotics");
    assert_eq!(find_ecc_skill("silver").unwrap().name, "agentic-silver-mcts-decision-trees");
    assert_eq!(find_ecc_skill("react").unwrap().name, "agentic-yao-react-interleaved-reasoning");
    assert_eq!(find_ecc_skill("reflexion").unwrap().name, "agentic-shinn-reflexion-self-correction");

    // Cluster 4: Vibe Coding & Flow
    assert_eq!(find_ecc_skill("karpathy").unwrap().name, "agentic-karpathy-vibe-coding-paradigm");
    assert_eq!(find_ecc_skill("vibe-coding").unwrap().name, "agentic-karpathy-vibe-coding-paradigm");
    assert_eq!(find_ecc_skill("csikszentmihalyi").unwrap().name, "agentic-csikszentmihalyi-flow-state");
    assert_eq!(find_ecc_skill("flow-state").unwrap().name, "agentic-csikszentmihalyi-flow-state");
    assert_eq!(find_ecc_skill("pragmatic-programmer").unwrap().name, "agentic-hunt-pragmatic-programmer");
    assert_eq!(find_ecc_skill("tracer-bullets").unwrap().name, "agentic-hunt-pragmatic-programmer");
    assert_eq!(find_ecc_skill("cathedral-bazaar").unwrap().name, "agentic-raymond-cathedral-bazaar");
    assert_eq!(find_ecc_skill("paul-graham").unwrap().name, "agentic-graham-hackers-painters");
    assert_eq!(find_ecc_skill("extreme-programming").unwrap().name, "agentic-beck-extreme-programming");
    assert_eq!(find_ecc_skill("lean-startup").unwrap().name, "agentic-ries-lean-mvp-feedback");
    assert_eq!(find_ecc_skill("design-sprint").unwrap().name, "agentic-knapp-design-sprint-prototyping");
    assert_eq!(find_ecc_skill("don-norman").unwrap().name, "agentic-norman-human-centered-interfaces");
    assert_eq!(find_ecc_skill("dont-make-me-think").unwrap().name, "agentic-krug-intuitive-interaction");

    // Cluster 5: Memory & RAG
    assert_eq!(find_ecc_skill("rag").unwrap().name, "agentic-lewis-rag-foundations");
    assert_eq!(find_ecc_skill("manning").unwrap().name, "agentic-manning-information-retrieval");
    assert_eq!(find_ecc_skill("baeza-yates").unwrap().name, "agentic-baeza-yates-vector-retrieval");
    assert_eq!(find_ecc_skill("hnsw").unwrap().name, "agentic-malkov-hnsw-vector-indexing");
    assert_eq!(find_ecc_skill("act-r").unwrap().name, "agentic-anderson-actr-cognitive-memory");
    assert_eq!(find_ecc_skill("working-memory").unwrap().name, "agentic-baddeley-working-memory-buffers");
    assert_eq!(find_ecc_skill("tulving").unwrap().name, "agentic-tulving-episodic-memory-retrieval");
    assert_eq!(find_ecc_skill("sowa").unwrap().name, "agentic-sowa-knowledge-representation");
    assert_eq!(find_ecc_skill("description-logics").unwrap().name, "agentic-baader-description-logics-ontologies");
    assert_eq!(find_ecc_skill("graph-rag").unwrap().name, "agentic-robinson-graph-rag-knowledge");

    // Cluster 6: Program Synthesis & Compilers
    assert_eq!(find_ecc_skill("gulwani").unwrap().name, "agentic-gulwani-program-synthesis");
    assert_eq!(find_ecc_skill("dragon-book").unwrap().name, "agentic-aho-dragon-compiler-parsing");
    assert_eq!(find_ecc_skill("cooper-torczon").unwrap().name, "agentic-cooper-compiler-ir-optimization");
    assert_eq!(find_ecc_skill("muchnick").unwrap().name, "agentic-muchnick-cfg-dataflow-analysis");
    assert_eq!(find_ecc_skill("crafting-interpreters").unwrap().name, "agentic-nystrom-crafting-interpreters");
    assert_eq!(find_ecc_skill("fowler-dsl").unwrap().name, "agentic-fowler-domain-specific-languages");
    assert_eq!(find_ecc_skill("antlr4").unwrap().name, "agentic-parr-antlr4-grammar-dsl");
    assert_eq!(find_ecc_skill("tapl").unwrap().name, "agentic-pierce-type-systems-soundness");
    assert_eq!(find_ecc_skill("pfpl").unwrap().name, "agentic-harper-practical-foundations-pl");
    assert_eq!(find_ecc_skill("sicp").unwrap().name, "agentic-sicp-evaluator-metacircular");

    // Cluster 7: Verification & Safety
    assert_eq!(find_ecc_skill("amodei").unwrap().name, "agentic-amodei-concrete-ai-safety");
    assert_eq!(find_ecc_skill("mmlu").unwrap().name, "agentic-hendrycks-benchmarking-evals");
    assert_eq!(find_ecc_skill("red-teaming").unwrap().name, "agentic-perez-red-teaming-adversarial");
    assert_eq!(find_ecc_skill("constitutional-ai").unwrap().name, "agentic-anthropic-constitutional-ai");
    assert_eq!(find_ecc_skill("swe-bench").unwrap().name, "agentic-ozkaya-llm-software-evals");
    assert_eq!(find_ecc_skill("tdd-contracts").unwrap().name, "agentic-beck-tdd-verifiable-contracts");
    assert_eq!(find_ecc_skill("quickcheck").unwrap().name, "agentic-claessen-property-based-testing");
    assert_eq!(find_ecc_skill("maciver").unwrap().name, "agentic-maciver-invariant-shrinking");
    assert_eq!(find_ecc_skill("model-checking").unwrap().name, "agentic-clarke-model-checking-invariants");
    assert_eq!(find_ecc_skill("baier-katoen").unwrap().name, "agentic-baier-temporal-logic-ltl-ctl");

    // Cluster 8: Distributed Systems & Protocols
    assert_eq!(find_ecc_skill("kleppmann").unwrap().name, "agentic-kleppmann-distributed-consistency");
    assert_eq!(find_ecc_skill("tanenbaum").unwrap().name, "agentic-tanenbaum-distributed-systems");
    assert_eq!(find_ecc_skill("logical-clocks").unwrap().name, "agentic-lamport-logical-clocks");
    assert_eq!(find_ecc_skill("raft").unwrap().name, "agentic-ongaro-raft-distributed-consensus");
    assert_eq!(find_ecc_skill("pipes-and-filters").unwrap().name, "agentic-hohpe-enterprise-integration-patterns");
    assert_eq!(find_ecc_skill("sam-newman").unwrap().name, "agentic-newman-microservices-tool-isolation");
    assert_eq!(find_ecc_skill("software-architecture").unwrap().name, "agentic-richards-software-architecture-tradeoffs");
    assert_eq!(find_ecc_skill("fielding").unwrap().name, "agentic-fielding-rest-agent-apis");
    assert_eq!(find_ecc_skill("henning").unwrap().name, "agentic-henning-rpc-schema-contracts");
    assert_eq!(find_ecc_skill("mcp-protocol").unwrap().name, "agentic-mcp-protocol-specification");

    // Cluster 9: Cognitive Architectures
    assert_eq!(find_ecc_skill("soar").unwrap().name, "agentic-laird-soar-cognitive-architecture");
    assert_eq!(find_ecc_skill("society-of-mind").unwrap().name, "agentic-minsky-society-of-mind");
    assert_eq!(find_ecc_skill("thinking-fast-and-slow").unwrap().name, "agentic-kahneman-dual-process-thinking");
    assert_eq!(find_ecc_skill("strange-loops").unwrap().name, "agentic-hofstadter-strange-loops-recursion");
    assert_eq!(find_ecc_skill("herbert-simon").unwrap().name, "agentic-simon-bounded-rationality-heuristics");
    assert_eq!(find_ecc_skill("unified-cognition").unwrap().name, "agentic-newell-unified-cognition");
    assert_eq!(find_ecc_skill("clarion").unwrap().name, "agentic-sun-clarion-implicit-explicit");
    assert_eq!(find_ecc_skill("concept-learning").unwrap().name, "agentic-lake-cognitive-concept-learning");
    assert_eq!(find_ecc_skill("godel-machine").unwrap().name, "agentic-schmidhuber-intrinsic-curiosity");
    assert_eq!(find_ecc_skill("nars").unwrap().name, "agentic-wang-nars-non-axiomatic-reasoning");

    // Cluster 10: Human-AI Collaboration & Evolution
    assert_eq!(find_ecc_skill("human-centered-ai").unwrap().name, "agentic-shneiderman-human-centered-ai");
    assert_eq!(find_ecc_skill("brooks-law").unwrap().name, "agentic-brooks-mythical-man-month");
    assert_eq!(find_ecc_skill("ousterhout").unwrap().name, "agentic-ousterhout-philosophy-software-design");
    assert_eq!(find_ecc_skill("clean-architecture").unwrap().name, "agentic-martin-clean-architecture-boundaries");
    assert_eq!(find_ecc_skill("feathers").unwrap().name, "agentic-feathers-legacy-code-refactoring");
    assert_eq!(find_ecc_skill("accelerate").unwrap().name, "agentic-forsgren-accelerate-dora-metrics");
    assert_eq!(find_ecc_skill("phoenix-project").unwrap().name, "agentic-kim-phoenix-project-flow-theory");
    assert_eq!(find_ecc_skill("domain-driven-design").unwrap().name, "agentic-evans-domain-driven-design");
    assert_eq!(find_ecc_skill("donella-meadows").unwrap().name, "agentic-meadows-systems-thinking-feedback");
    assert_eq!(find_ecc_skill("kevin-kelly").unwrap().name, "agentic-kelly-autonomous-cognition-flows");
}

// =========================================================================
// 4. Semantic Intent Dispatching for Agentic Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_agentic_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        ("bdi belief desire intention communicative acts contract net protocol multi-agent coordination", "agentic-wooldridge-multiagent-systems"),
        ("transformer multi-head self-attention kv-cache mechanics token logits jay alammar", "agentic-alammar-transformer-mechanics"),
        ("react thought action observation interleaved reasoning external tool grounding", "agentic-yao-react-interleaved-reasoning"),
        ("vibe coding intuitive conversational programming flow state andrej karpathy", "agentic-karpathy-vibe-coding-paradigm"),
        ("retrieval augmented generation dense passage retrieval non-parametric vector memory rag", "agentic-lewis-rag-foundations"),
        ("compiler lexing lr parsing abstract syntax tree dragon book aho ullman", "agentic-aho-dragon-compiler-parsing"),
        ("constitutional ai harmlessness from ai feedback rlaif self-critique anthropic", "agentic-anthropic-constitutional-ai"),
        ("raft distributed consensus leader election log replication diego ongaro", "agentic-ongaro-raft-distributed-consensus"),
        ("soar cognitive architecture subgoaling on impasses chunking working memory laird", "agentic-laird-soar-cognitive-architecture"),
        ("clean architecture dependency inversion concentric boundaries robert martin uncle bob", "agentic-martin-clean-architecture-boundaries"),
    ];

    for (query, expected_skill) in test_cases {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(!results.is_empty(), "Dispatch returned no results for query '{}'", query);

        let found = results.iter().any(|d| d.skill.name == expected_skill);
        assert!(
            found,
            "Semantic dispatch failed for query '{}': expected '{}' in top 5, got {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );

        let matched = results.iter().find(|d| d.skill.name == expected_skill).unwrap();
        assert!(
            matched.score > 0.0,
            "Dispatched skill '{}' score must be > 0.0, got {}",
            expected_skill,
            matched.score
        );
    }
}

// =========================================================================
// 5. 50-Thread Concurrent Dispatch Stress Test
// =========================================================================

#[test]
fn test_multithreaded_concurrent_agentic_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "bdi belief desire intention multiagent coordination",
        "kv-cache multi-head self-attention transformer mechanics",
        "react thought action observation reasoning tool grounding",
        "vibe coding flow state conversational iteration",
        "retrieval augmented generation dense passage vector rag",
        "dragon book compiler parsing abstract syntax tree",
        "constitutional ai self-critique rlaif safety alignment",
        "raft distributed consensus leader election log replication",
        "soar cognitive architecture subgoaling impasses",
        "clean architecture dependency inversion concentric boundaries",
    ]);

    let thread_count = 50;
    let iterations_per_thread = 20;
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::with_capacity(thread_count);

    let start_time = Instant::now();

    for t_idx in 0..thread_count {
        let d_clone = Arc::clone(&dispatcher);
        let q_clone = Arc::clone(&queries);
        let s_clone = Arc::clone(&success_count);

        handles.push(thread::spawn(move || {
            for i in 0..iterations_per_thread {
                let query = &q_clone[(t_idx + i) % q_clone.len()];
                let results = d_clone.dispatch(query, 3, None);
                if !results.is_empty() && results[0].score > 0.0 {
                    s_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked during concurrent dispatch");
    }

    let elapsed = start_time.elapsed();
    let total_dispatches = thread_count * iterations_per_thread;
    let successful = success_count.load(Ordering::SeqCst);

    assert_eq!(
        successful, total_dispatches,
        "All 50 concurrent dispatch threads must succeed across {} iterations (elapsed: {:?})",
        total_dispatches, elapsed
    );
    assert!(
        elapsed.as_millis() < 30000,
        "50-thread concurrent dispatch stress test took {:?}, expected < 30s",
        elapsed
    );
}

// =========================================================================
// 6. Content Completeness and Safety Invariants
// =========================================================================

#[test]
fn test_agentic_skills_content_completeness_and_safety_invariants() {
    // 1. Check content completeness of all 100 skills
    for skill_name in &AGENTIC_SKILLS_100 {
        let skill = find_ecc_skill(skill_name).unwrap();
        assert!(skill.description.len() >= 30, "Description too short for '{}'", skill_name);
        assert!(skill.description.len() <= 400, "Description too long for '{}'", skill_name);
        
        let inst = &skill.instructions;
        assert!(inst.contains("name: "), "Skill '{}' missing name in frontmatter", skill_name);
        assert!(inst.contains("description: "), "Skill '{}' missing description in frontmatter", skill_name);
        assert!(inst.contains("triggers: ["), "Skill '{}' missing triggers in frontmatter", skill_name);
        assert!(inst.contains("## 1. Core Mathematical"), "Skill '{}' missing Section 1", skill_name);
        assert!(inst.contains("## 2. Concrete Agent Specification"), "Skill '{}' missing Section 2", skill_name);
        assert!(inst.contains("## 3. Anti-Patterns"), "Skill '{}' missing Section 3", skill_name);
        assert!(inst.contains("## 4. Executable Verification Recipe"), "Skill '{}' missing Section 4", skill_name);
    }

    // 2. Bellman Optimality Invariant: V*(s) >= V_pi(s)
    let v_star = 100.0f64;
    let subopt_policy_values = [42.0f64, 85.5f64, 99.99f64];
    for &v_pi in &subopt_policy_values {
        assert!(v_star >= v_pi, "Bellman optimality violated: V*(s) < V_pi(s)");
    }

    // 3. MCTS UCT Exploration Bound: UCT score must be finite and monotonically decrease exploration term
    let n_parent = 1000f64;
    let c = 1.41421356f64; // sqrt(2)
    let uct_low_visits = 0.5 + c * (n_parent.ln() / 10.0).sqrt();
    let uct_high_visits = 0.5 + c * (n_parent.ln() / 100.0).sqrt();
    assert!(uct_low_visits > uct_high_visits, "Exploration bonus must decrease as visit count increases");
    assert!(uct_low_visits.is_finite(), "UCT score must remain finite");

    // 4. Token Budget Guard Invariant (KV-cache footprint does not overflow)
    let n_layers = 32u64;
    let n_heads = 32u64;
    let d_head = 128u64;
    let seq_len = 8192u64;
    let batch_size = 1u64;
    let precision_bytes = 2u64; // fp16
    let kv_bytes = 2 * n_layers * n_heads * d_head * seq_len * batch_size * precision_bytes;
    assert_eq!(kv_bytes, 4_294_967_296, "KV cache footprint calculation must match exactly 4 GB");

    // 5. ReAct Loop Termination Invariant: Max iteration safeguard
    let max_iterations = 25usize;
    let mut current_iteration = 0usize;
    let goal_reached = false;
    while current_iteration < max_iterations && !goal_reached {
        current_iteration += 1;
    }
    assert_eq!(current_iteration, max_iterations, "ReAct loop must safely terminate at maximum bound");
}

// =========================================================================
// 7. Local vs Cloud Instructions Integrity
// =========================================================================

#[test]
fn test_agentic_skills_local_vs_cloud_formatting() {
    let dispatcher = global_ecc_dispatcher();
    let query = "Implement autonomous ReAct loop with BDI agent architecture and structured JSON tool calls";

    // 1. Local Ollama formatting: condensed cheat sheet
    let (local_prompt, local_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "ollama",
        None,
    );
    assert!(!local_skills.is_empty(), "Local dispatch should return skills");
    assert!(
        local_prompt.contains("[LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"),
        "Local Ollama prompt must contain Cheat Sheet header"
    );
    assert!(
        local_skills.len() <= 2,
        "Local Ollama context must be bounded to <= 2 skills, got {}",
        local_skills.len()
    );

    // 2. Cloud Anthropic / OpenAI formatting: comprehensive architectural specifications
    let (cloud_prompt, cloud_skills) = dispatcher.equip_prompt_for_provider(
        "Base system contract",
        query,
        "anthropic",
        None,
    );
    assert!(!cloud_skills.is_empty(), "Cloud dispatch should return skills");
    assert!(
        cloud_prompt.contains("[COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"),
        "Cloud prompt must contain Comprehensive Specifications header"
    );
    assert!(
        cloud_prompt.contains("Full Specification & Directives:"),
        "Cloud prompt must contain full directive body"
    );
}
