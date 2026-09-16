//! Brutal Integration & Verification Tests for Top 150 Skills in Microsoft 365 Copilot & Vibe Code Development in Tagisan (tgs)

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use tagisan::{
    all_ecc_skills, find_ecc_skill, global_ecc_dispatcher, load_ecc_skills_from_dir,
};

pub const MS365_COPILOT_SKILLS_150: [&str; 150] = [
    // Cluster 1: Microsoft 365 Copilot Architecture & Platform Foundations (1-10)
    "copilot-platform-extensibility-matrix",
    "copilot-orchestration-copilot-engine",
    "copilot-semantic-index-architecture",
    "copilot-m365-tenant-boundary-isolation",
    "copilot-declarative-agent-runtime",
    "copilot-plugin-manifest-v1",
    "copilot-teams-toolkit-scaffolding",
    "copilot-hardware-npu-directml-telemetry",
    "copilot-license-sku-governance",
    "copilot-copilot-studio-vs-pro-dev",
    // Cluster 2: Declarative Agents, App Manifests & Teams Toolkit (11-20)
    "copilot-declarative-agent-manifest-v1-2",
    "copilot-teams-app-manifest-v1-17",
    "copilot-openapi-actions-declarative",
    "copilot-onedrive-sharepoint-grounding",
    "copilot-web-grounding-bing-search",
    "copilot-declarative-agent-conversation-starters",
    "copilot-teams-toolkit-environment-variables",
    "copilot-teams-app-packaging-validation",
    "copilot-declarative-agent-publishing-admin",
    "copilot-declarative-agent-telemetry-monitoring",
    // Cluster 3: Microsoft Copilot Studio, Topic Orchestration & Multi-Agent Swarms (21-30)
    "copilot-studio-generative-topics",
    "copilot-studio-dynamic-chaining",
    "copilot-studio-custom-topic-state-machine",
    "copilot-studio-multi-agent-swarms",
    "copilot-studio-knowledge-sources",
    "copilot-studio-power-fx-formulas",
    "copilot-studio-bot-framework-composer-interop",
    "copilot-studio-channel-deployment",
    "copilot-studio-analytics-conversation-transcripts",
    "copilot-studio-alm-solution-lifecycle",
    // Cluster 4: Microsoft Graph API, Semantic Index & Knowledge Substrate (31-40)
    "copilot-graph-rest-api-v1-beta",
    "copilot-graph-delta-queries-sync",
    "copilot-graph-batching-json-requests",
    "copilot-graph-connectors-schema-registration",
    "copilot-graph-connectors-acl-crawling",
    "copilot-graph-webhooks-change-notifications",
    "copilot-graph-jwe-encrypted-notifications",
    "copilot-graph-meeting-transcript-parsing",
    "copilot-graph-sharepoint-driveitems",
    "copilot-graph-search-query-api",
    // Cluster 5: Custom Engine Agents & Teams AI Library (41-50)
    "copilot-teams-ai-library-core-architecture",
    "copilot-teams-ai-action-planner",
    "copilot-teams-ai-turn-state-management",
    "copilot-teams-ai-streaming-responses",
    "copilot-teams-ai-adaptive-card-routing",
    "copilot-teams-ai-message-extensions",
    "copilot-teams-ai-feedback-loop-handlers",
    "copilot-teams-ai-authentication-turn-handler",
    "copilot-teams-ai-custom-engine-rag",
    "copilot-teams-ai-testing-and-debugging",
    // Cluster 6: Semantic Kernel (C# & Python) & Autonomous Agent Plugins (51-60)
    "copilot-semantic-kernel-kernel-architecture",
    "copilot-sk-native-plugins-csharp-python",
    "copilot-sk-prompt-functions-yaml",
    "copilot-sk-auto-function-calling",
    "copilot-sk-filters-and-hooks",
    "copilot-sk-agent-framework-chat-completion",
    "copilot-sk-memory-and-vector-connectors",
    "copilot-sk-openapi-plugin-generator",
    "copilot-sk-process-framework-event-driven",
    "copilot-sk-enterprise-observability",
    // Cluster 7: Power Platform, Power Apps & Power Automate Vibe Workflows (61-70)
    "copilot-power-automate-cloud-flows-ai",
    "copilot-power-apps-copilot-studio-embedding",
    "copilot-power-fx-natural-language-formulas",
    "copilot-custom-connectors-openapi-oauth",
    "copilot-dataverse-web-api-crud",
    "copilot-power-pages-ai-site-generation",
    "copilot-ai-builder-document-processing",
    "copilot-power-platform-cli-pac",
    "copilot-power-platform-dlp-connector-policies",
    "copilot-power-automate-error-handling-scopes",
    // Cluster 8: Office JavaScript / TypeScript Scripts & Add-ins Engineering (71-80)
    "copilot-office-scripts-excel-typescript",
    "copilot-office-scripts-power-automate-sync",
    "copilot-excel-javascript-api-custom-functions",
    "copilot-word-javascript-api-content-controls",
    "copilot-outlook-javascript-api-mail-drafting",
    "copilot-office-add-in-unified-manifest",
    "copilot-office-js-batching-context-sync",
    "copilot-office-add-in-sso-entra-id",
    "copilot-powerpoint-presentation-generation",
    "copilot-office-add-in-deployment-centralized",
    // Cluster 9: Vibe Coding Paradigms, Conversational Flow & Rapid Prototyping (81-90)
    "copilot-vibe-coding-conversational-flow",
    "copilot-tracer-bullets-m365-architecture",
    "copilot-rapid-feedback-loops-m365",
    "copilot-human-in-the-loop-steering",
    "copilot-exploratory-spike-prototyping",
    "copilot-conversational-tdd-synthesis",
    "copilot-fail-fast-diagnostic-surfacing",
    "copilot-context-scaffolding-agents",
    "copilot-dialectical-code-review-agents",
    "copilot-flow-state-ergonomics",
    // Cluster 10: Advanced Prompt Engineering, Context Grounding & In-Context Reasoning (91-100)
    "copilot-prompt-cot-few-shot-crafting",
    "copilot-lost-in-the-middle-mitigation",
    "copilot-context-grounding-citations",
    "copilot-system-instructions-declarative",
    "copilot-structured-json-repair-schemas",
    "copilot-in-context-memory-pruning",
    "copilot-plan-and-solve-decomposition",
    "copilot-metacognitive-self-reflection",
    "copilot-rag-query-expansion-hyde",
    "copilot-prompt-token-budget-optimizer",
    // Cluster 11: Enterprise Security, RBAC, Purview DLP & Cryptographic Shielding (101-110)
    "copilot-entra-id-workload-identity",
    "copilot-oauth2-obo-flow-exchange",
    "copilot-continuous-access-evaluation-cae",
    "copilot-purview-sensitivity-labeling",
    "copilot-purview-dlp-data-fencing",
    "copilot-agentshield-prompt-injection-defense",
    "copilot-jwe-token-cryptography",
    "copilot-rbac-least-privilege-scoping",
    "copilot-zero-egress-air-gap-fencing",
    "copilot-cryptographic-audit-receipts",
    // Cluster 12: Enterprise RAG, Azure OpenAI, Vector Indexing & Hybrid Search (111-120)
    "copilot-azure-openai-enterprise-deployment",
    "copilot-azure-ai-search-hybrid-hnsw",
    "copilot-semantic-ranker-l2-reranking",
    "copilot-document-chunking-token-strategies",
    "copilot-text-embedding-vector-space",
    "copilot-graph-rag-knowledge-graphs",
    "copilot-contextual-compression-retrieval",
    "copilot-azure-openai-private-link-network",
    "copilot-enterprise-rag-citation-pipeline",
    "copilot-rag-cache-semantic-memoization",
    // Cluster 13: Adaptive Cards, Fluent UI & Conversational Ergonomics (121-130)
    "copilot-adaptive-cards-v1-5-templating",
    "copilot-universal-actions-for-teams",
    "copilot-fluent-ui-blazor-react-styling",
    "copilot-adaptive-card-form-input-validation",
    "copilot-interactive-cards-live-updates",
    "copilot-conversational-information-architecture",
    "copilot-adaptive-card-blast-radius-reports",
    "copilot-mobile-teams-card-ergonomics",
    "copilot-dark-high-contrast-adaptive-theme",
    "copilot-teams-task-modules-dialogs",
    // Cluster 14: Evals, Behavioral Guardrails, Red Teaming & Copilot Safety (131-140)
    "copilot-ragas-eval-rag-triad",
    "copilot-adversarial-red-teaming-probes",
    "copilot-azure-ai-content-safety-api",
    "copilot-groundedness-hallucination-detector",
    "copilot-prompt-jailbreak-canary-tokens",
    "copilot-copilot-studio-test-framework",
    "copilot-llm-as-a-judge-rubric-evals",
    "copilot-latency-token-throughput-benchmarking",
    "copilot-responsible-ai-impact-assessment",
    "copilot-production-telemetry-drift-detection",
    // Cluster 15: Cross-Ecosystem Enterprise Connectors (SAP, Salesforce, Jira, ServiceNow via Graph & OpenAPI) (141-150)
    "copilot-sap-s4hana-odata-connector",
    "copilot-salesforce-rest-graph-connector",
    "copilot-servicenow-incident-cmdb-connector",
    "copilot-jira-cloud-confluence-rest-connector",
    "copilot-workday-raas-human-capital-connector",
    "copilot-dynamics-365-dataverse-deep-sync",
    "copilot-sql-server-azure-sql-graph-bridge",
    "copilot-cross-system-identity-reconciliation",
    "copilot-rate-limit-federation-enterprise-apis",
    "copilot-end-to-end-enterprise-action-choreography",
];

// =========================================================================
// 1. Discovery of all 150 M365 Copilot Skills on Disk
// =========================================================================

#[test]
fn test_all_150_ms365_copilot_skills_discovered_on_disk() {
    let skills_dir = Path::new(".ecc/skills");
    assert!(skills_dir.is_dir(), ".ecc/skills directory must exist");

    let loaded = load_ecc_skills_from_dir(skills_dir);
    let loaded_map: HashSet<String> = loaded.into_iter().map(|s| s.name).collect();

    for skill in &MS365_COPILOT_SKILLS_150 {
        assert!(
            loaded_map.contains(*skill),
            "M365 Copilot skill '{}' must be discovered in .ecc/skills directory",
            skill
        );
    }
}

// =========================================================================
// 2. Built-in Registration of all 150 M365 Copilot Skills (assert >= 645 total skills)
// =========================================================================

#[test]
fn test_all_150_ms365_copilot_skills_built_in_registration() {
    let all_skills = all_ecc_skills();
    assert!(
        all_skills.len() >= 645,
        "Expected at least 645 total built-in skills including M365 Copilot suite, found {}",
        all_skills.len()
    );

    for skill_name in &MS365_COPILOT_SKILLS_150 {
        let found = find_ecc_skill(skill_name);
        assert!(
            found.is_some(),
            "M365 Copilot skill '{}' must be registered in built-in skills",
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
// 3. High-Leverage M365 Copilot & Vibe Coding Alias Lookups
// =========================================================================

#[test]
fn test_ms365_copilot_skills_alias_lookups() {
    // Cluster 1: Architecture & Platform
    assert_eq!(find_ecc_skill("copilot-platform-extensibility").unwrap().name, "copilot-platform-extensibility-matrix");
    assert_eq!(find_ecc_skill("declarative-vs-custom-engine").unwrap().name, "copilot-platform-extensibility-matrix");
    assert_eq!(find_ecc_skill("semantic-index").unwrap().name, "copilot-semantic-index-architecture");
    assert_eq!(find_ecc_skill("tenant-isolation").unwrap().name, "copilot-m365-tenant-boundary-isolation");
    assert_eq!(find_ecc_skill("teamsapp-yml").unwrap().name, "copilot-teams-toolkit-scaffolding");

    // Cluster 2: Declarative Agents & Teams Toolkit
    assert_eq!(find_ecc_skill("declarativeagent-json").unwrap().name, "copilot-declarative-agent-manifest-v1-2");
    assert_eq!(find_ecc_skill("manifest-json-v1-17").unwrap().name, "copilot-teams-app-manifest-v1-17");
    assert_eq!(find_ecc_skill("sharepoint-grounding").unwrap().name, "copilot-onedrive-sharepoint-grounding");
    assert_eq!(find_ecc_skill("bing-search-grounding").unwrap().name, "copilot-web-grounding-bing-search");

    // Cluster 3: Copilot Studio
    assert_eq!(find_ecc_skill("copilot-studio-generative").unwrap().name, "copilot-studio-generative-topics");
    assert_eq!(find_ecc_skill("dynamic-chaining").unwrap().name, "copilot-studio-dynamic-chaining");
    assert_eq!(find_ecc_skill("agent-swarms").unwrap().name, "copilot-studio-multi-agent-swarms");
    assert_eq!(find_ecc_skill("power-fx-copilot-studio").unwrap().name, "copilot-studio-power-fx-formulas");

    // Cluster 4: Microsoft Graph
    assert_eq!(find_ecc_skill("graph-rest-api").unwrap().name, "copilot-graph-rest-api-v1-beta");
    assert_eq!(find_ecc_skill("graph-delta-queries").unwrap().name, "copilot-graph-delta-queries-sync");
    assert_eq!(find_ecc_skill("graph-batching").unwrap().name, "copilot-graph-batching-json-requests");
    assert_eq!(find_ecc_skill("graph-connectors").unwrap().name, "copilot-graph-connectors-schema-registration");
    assert_eq!(find_ecc_skill("jwe-encrypted-notifications").unwrap().name, "copilot-graph-jwe-encrypted-notifications");

    // Cluster 5: Teams AI Library
    assert_eq!(find_ecc_skill("teams-ai-library").unwrap().name, "copilot-teams-ai-library-core-architecture");
    assert_eq!(find_ecc_skill("actionplanner-config").unwrap().name, "copilot-teams-ai-action-planner");
    assert_eq!(find_ecc_skill("streaming-responses").unwrap().name, "copilot-teams-ai-streaming-responses");

    // Cluster 6: Semantic Kernel
    assert_eq!(find_ecc_skill("sk-kernel-architecture").unwrap().name, "copilot-semantic-kernel-kernel-architecture");
    assert_eq!(find_ecc_skill("kernel-function-decorator").unwrap().name, "copilot-sk-native-plugins-csharp-python");
    assert_eq!(find_ecc_skill("sk-auto-function-calling").unwrap().name, "copilot-sk-auto-function-calling");

    // Cluster 7: Power Platform
    assert_eq!(find_ecc_skill("power-automate-copilot").unwrap().name, "copilot-power-automate-cloud-flows-ai");
    assert_eq!(find_ecc_skill("power-fx-vibe-coding").unwrap().name, "copilot-power-fx-natural-language-formulas");
    assert_eq!(find_ecc_skill("dataverse-crud").unwrap().name, "copilot-dataverse-web-api-crud");

    // Cluster 8: Office JavaScript & TypeScript
    assert_eq!(find_ecc_skill("office-scripts-excel").unwrap().name, "copilot-office-scripts-excel-typescript");
    assert_eq!(find_ecc_skill("excel-custom-functions").unwrap().name, "copilot-excel-javascript-api-custom-functions");
    assert_eq!(find_ecc_skill("context-sync-optimization").unwrap().name, "copilot-office-js-batching-context-sync");

    // Cluster 9: Vibe Coding
    assert_eq!(find_ecc_skill("vibe-coding-flow").unwrap().name, "copilot-vibe-coding-conversational-flow");
    assert_eq!(find_ecc_skill("tracer-bullets-m365").unwrap().name, "copilot-tracer-bullets-m365-architecture");
    assert_eq!(find_ecc_skill("teams-dev-tunnels").unwrap().name, "copilot-rapid-feedback-loops-m365");

    // Cluster 10: Advanced Prompts
    assert_eq!(find_ecc_skill("cot-prompting").unwrap().name, "copilot-prompt-cot-few-shot-crafting");
    assert_eq!(find_ecc_skill("lost-in-the-middle").unwrap().name, "copilot-lost-in-the-middle-mitigation");
    assert_eq!(find_ecc_skill("structured-json-repair").unwrap().name, "copilot-structured-json-repair-schemas");

    // Cluster 11: Enterprise Security
    assert_eq!(find_ecc_skill("oauth2-obo-flow").unwrap().name, "copilot-oauth2-obo-flow-exchange");
    assert_eq!(find_ecc_skill("purview-sensitivity-labeling").unwrap().name, "copilot-purview-sensitivity-labeling");
    assert_eq!(find_ecc_skill("agentshield-defense").unwrap().name, "copilot-agentshield-prompt-injection-defense");

    // Cluster 12: Enterprise RAG
    assert_eq!(find_ecc_skill("azure-openai-deployment").unwrap().name, "copilot-azure-openai-enterprise-deployment");
    assert_eq!(find_ecc_skill("azure-ai-search-hybrid").unwrap().name, "copilot-azure-ai-search-hybrid-hnsw");
    assert_eq!(find_ecc_skill("graph-rag-knowledge").unwrap().name, "copilot-graph-rag-knowledge-graphs");

    // Cluster 13: Adaptive Cards
    assert_eq!(find_ecc_skill("adaptive-cards-templating").unwrap().name, "copilot-adaptive-cards-v1-5-templating");
    assert_eq!(find_ecc_skill("action-execute").unwrap().name, "copilot-universal-actions-for-teams");

    // Cluster 14: Evals & Safety
    assert_eq!(find_ecc_skill("ragas-eval-triad").unwrap().name, "copilot-ragas-eval-rag-triad");
    assert_eq!(find_ecc_skill("jailbreak-canary-tokens").unwrap().name, "copilot-prompt-jailbreak-canary-tokens");

    // Cluster 15: Enterprise Connectors
    assert_eq!(find_ecc_skill("sap-odata-copilot").unwrap().name, "copilot-sap-s4hana-odata-connector");
    assert_eq!(find_ecc_skill("salesforce-copilot").unwrap().name, "copilot-salesforce-rest-graph-connector");
    assert_eq!(find_ecc_skill("servicenow-copilot").unwrap().name, "copilot-servicenow-incident-cmdb-connector");
}

// =========================================================================
// 4. Content Completeness & Operational Invariants
// =========================================================================

#[test]
fn test_ms365_copilot_skills_content_completeness_and_safety_invariants() {
    let mut total_always = 0;
    let mut total_never = 0;
    let mut total_mandatory = 0;

    for skill_name in &MS365_COPILOT_SKILLS_150 {
        let skill = find_ecc_skill(skill_name).expect("Skill must be registered");

        if skill.instructions.contains("ALWAYS") {
            total_always += 1;
        }
        if skill.instructions.contains("NEVER") {
            total_never += 1;
        }
        if skill.instructions.contains("MANDATORY") {
            total_mandatory += 1;
        }

        // Verify standard frontmatter structure
        assert!(
            skill.instructions.contains("name: "),
            "Skill '{}' must have name field in YAML",
            skill_name
        );
        assert!(
            skill.instructions.contains("description: "),
            "Skill '{}' must have description field in YAML",
            skill_name
        );
        assert!(
            skill.instructions.contains("triggers: "),
            "Skill '{}' must have triggers field in YAML",
            skill_name
        );
    }

    // Assert that strict operational invariants are deeply embedded across the suite
    assert!(
        total_always >= 100,
        "Expected at least 100 skills to contain ALWAYS directives, got {}",
        total_always
    );
    assert!(
        total_never >= 20,
        "Expected at least 20 skills to contain NEVER directives, got {}",
        total_never
    );
    assert!(
        total_mandatory >= 100,
        "Expected at least 100 skills to contain MANDATORY directives, got {}",
        total_mandatory
    );
}

// =========================================================================
// 5. Semantic Intent Dispatching for M365 Copilot Queries
// =========================================================================

#[test]
fn test_semantic_intent_dispatching_for_ms365_copilot_queries() {
    let dispatcher = global_ecc_dispatcher();

    let test_cases = [
        ("declarative agent manifest declarativeAgent.json with sharepoint and bing search grounding", "copilot-declarative-agent-manifest-v1-2"),
        ("teams ai library turncontext actionplanner applicationbuilder streaming responses", "copilot-teams-ai-library-core-architecture"),
        ("copilot studio generative topics triggers and generative answers nodes", "copilot-studio-generative-topics"),
        ("semantic-kernel-core sk-kernel-architecture service collection registration", "copilot-semantic-kernel-kernel-architecture"),
        ("microsoft graph delta query change tracking odata delta token sync", "copilot-graph-delta-queries-sync"),
        ("microsoft purview data loss prevention dlp sensitivity labeling data fencing", "copilot-purview-dlp-data-fencing"),
        ("adaptive cards v1.5 templating universal action execute user specific views", "copilot-adaptive-cards-v1-5-templating"),
        ("office scripts excel workbook range mutation typescript power automate", "copilot-office-scripts-excel-typescript"),
        ("sap-odata-copilot s4hana-connector bapi cds integration", "copilot-sap-s4hana-odata-connector"),
        ("copilot vibe coding flow conversational refactoring iterative steering", "copilot-vibe-coding-conversational-flow"),
    ];

    for (query, expected_skill) in &test_cases {
        let results = dispatcher.dispatch(query, 5, None);
        assert!(!results.is_empty(), "Dispatch returned no results for query '{}'", query);

        let found = results.iter().any(|d| d.skill.name == *expected_skill);
        assert!(
            found,
            "Query '{}' should have dispatched to '{}', but returned: {:?}",
            query,
            expected_skill,
            results.iter().map(|d| (&d.skill.name, d.score)).collect::<Vec<_>>()
        );

        let matched = results.iter().find(|d| d.skill.name == *expected_skill).unwrap();
        assert!(
            matched.score > 0.0,
            "Dispatched skill '{}' score must be > 0.0, got {}",
            expected_skill,
            matched.score
        );
    }
}

// =========================================================================
// 6. Multithreaded Concurrent Dispatching (50 Threads)
// =========================================================================

#[test]
fn test_multithreaded_concurrent_ms365_copilot_dispatching_50_threads() {
    let dispatcher = Arc::new(global_ecc_dispatcher());
    let queries = Arc::new(vec![
        "declarative agent manifest sharepoint grounding",
        "teams ai library action planner streaming",
        "copilot studio dynamic topic chaining",
        "semantic kernel native plugins",
        "graph api delta queries sync",
        "purview sensitivity labels dlp",
        "adaptive cards universal action execute",
        "office scripts excel typescript",
        "sap odata enterprise connector",
        "vibe coding conversational flow",
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
    println!(
        "  [+] Executed {} concurrent M365 Copilot skill dispatches across {} threads in {:?} ({:.2} µs/op)",
        total_dispatches,
        thread_count,
        elapsed,
        (elapsed.as_micros() as f64) / (total_dispatches as f64)
    );
}
