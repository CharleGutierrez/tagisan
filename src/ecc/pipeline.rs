use crate::agent::AutonomousAgent;
use crate::dag::graph::WorkflowGraph;
use crate::dag::node::{RetryPolicy, TaskNode};
use crate::ecc::presets;
use crate::error::Result;
use crate::providers::LlmProvider;
use crate::tools::ToolRegistry;
use std::sync::Arc;
use std::time::Duration;

/// Constructs the canonical 5-stage ECC Engineering Workflow Pipeline DAG:
/// 
/// ```text
///        [1. Plan (Architect)]
///                 │
///                 ▼
///       [2. Test (TDD Engineer)]
///                 │
///                 ▼
///     [3. Implement (Autonomous Coder)]
///            /          \
///           ▼            ▼
///   [4a. Code Review]  [4b. Security Audit]   <-- Runs Concurrently!
///           \            /
///            ▼          ▼
///       [5. Verify & Synthesize]
/// ```
pub fn build_ecc_pipeline(
    objective: &str,
    provider: Arc<dyn LlmProvider>,
    model: &str,
    mut tools: ToolRegistry,
) -> Result<WorkflowGraph> {
    // 0. Ensure search_skills discovery tool is available to all pipeline agents
    if !tools.contains("search_skills") {
        tools.register_tool(crate::tools::builtin::SearchSkillsTool::with_default());
    }

    let dispatcher = crate::ecc::skills::global_dispatcher();

    // Helper: Dynamically auto-equip relevant engineering skills into system prompt
    let equip_skills = |base_prompt: &str, stage_query: &str, domain_bias: Option<&str>| -> String {
        let full_query = format!("{} {}", objective, stage_query);
        let dispatched = dispatcher.dispatch(&full_query, 2, domain_bias);
        let mut prompt = base_prompt.to_string();
        if !dispatched.is_empty() {
            prompt.push_str("\n\n--- AUTO-EQUIPPED SPECIALIZED ENGINEERING SKILLS ---");
            for d in dispatched {
                prompt.push_str(&format!(
                    "\n\n### Skill: {} (Match Score: {:.1} | Domain: {})\n{}\n",
                    d.skill.name, d.score, d.domain, d.skill.instructions
                ));
            }
        }
        prompt
    };

    let mut graph = WorkflowGraph::new();

    // 1. Plan (Architect Agent)
    let architect = presets::architect();
    let arch_prompt = equip_skills(
        &architect.system_prompt,
        "system architecture modular boundary design API contracts",
        Some("architecture"),
    );
    let plan_agent = AutonomousAgent::new(provider.clone(), model, tools.clone())
        .with_system_prompt(arch_prompt)
        .with_temperature(0.5)
        .with_agentshield(true);

    let plan_node = TaskNode::new(
        "ecc_plan",
        "1. Architectural Planning (ECC Architect)",
        format!(
            "OBJECTIVE:\n\"{}\"\n\n\
            TASK:\n\
            1. Formulate a robust architectural specification and modular component design.\n\
            2. Define explicit type signatures, interfaces, error handling strategies, and boundary constraints.\n\
            3. Identify critical edge cases, concurrency invariants, and scale considerations.",
            objective
        ),
    )
    .with_agent(plan_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(2), 2.0));

    // 2. Test (TDD Engineer Agent)
    let tdd = presets::tdd_engineer();
    let tdd_prompt = equip_skills(
        &tdd.system_prompt,
        "test driven development unit tests assertions fuzzing regression verification",
        Some("test"),
    );
    let tdd_agent = AutonomousAgent::new(provider.clone(), model, tools.clone())
        .with_system_prompt(tdd_prompt)
        .with_temperature(0.3)
        .with_agentshield(true);

    let test_node = TaskNode::new(
        "ecc_test",
        "2. Test-Driven Specification (ECC TDD Engineer)",
        format!(
            "OBJECTIVE:\n\"{}\"\n\n\
            ARCHITECTURAL SPECIFICATION:\n\
            {{ecc_plan.output}}\n\n\
            TASK:\n\
            1. Write comprehensive unit tests and regression assertions BEFORE implementation.\n\
            2. Cover happy paths, edge cases, error conditions, and boundary values.\n\
            3. Provide deterministic assertions that prove the solution works under stress.",
            objective
        ),
    )
    .with_agent(tdd_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(2), 2.0));

    // 3. Implement (Autonomous Coder)
    let coder_prompt = equip_skills(
        "You are a Senior Implementation Engineer operating under the ECC framework.\n\
        Write clean, idiomatic, fully functional, and production-grade code that satisfies the architecture and passes all tests.",
        "production implementation clean idiomatic resilience",
        None,
    );
    let coder_agent = AutonomousAgent::new(provider.clone(), model, tools.clone())
        .with_system_prompt(coder_prompt)
        .with_temperature(0.2)
        .with_agentshield(true);

    let implement_node = TaskNode::new(
        "ecc_implement",
        "3. Production Implementation (ECC Coder)",
        format!(
            "OBJECTIVE:\n\"{}\"\n\n\
            ARCHITECTURAL SPECIFICATION:\n\
            {{ecc_plan.output}}\n\n\
            TEST SPECIFICATION & ASSERTIONS:\n\
            {{ecc_test.output}}\n\n\
            TASK:\n\
            1. Implement the complete, working production solution in clean, idiomatic code.\n\
            2. Satisfy all specifications from the architectural plan.\n\
            3. Ensure all tests and assertions are guaranteed to pass cleanly.",
            objective
        ),
    )
    .with_agent(coder_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(2), 2.0));

    // 4a. Review (Code Reviewer)
    let reviewer = presets::code_reviewer();
    let review_prompt = equip_skills(
        &reviewer.system_prompt,
        "code review cleanliness cyclomatic complexity standards",
        Some("review"),
    );
    let review_agent = AutonomousAgent::new(provider.clone(), model, tools.clone())
        .with_system_prompt(review_prompt)
        .with_temperature(0.4)
        .with_agentshield(true);

    let review_node = TaskNode::new(
        "ecc_review",
        "4a. Code Quality & Idioms Review (ECC Reviewer)",
        "IMPLEMENTATION TO REVIEW:\n\
        {ecc_implement.output}\n\n\
        TASK:\n\
        1. Scrutinize the implementation for code clarity, maintainability, and idiomatic practices.\n\
        2. Verify proper error handling, modularity, and DRY principles.\n\
        3. Provide categorized feedback ([Blocker], [Suggestion], [Nitpick]).",
    )
    .with_agent(review_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(5), 2.0));

    // 4b. Security Audit (Security Auditor) - Parallel branch with Review
    let security = presets::security_auditor();
    let sec_prompt = equip_skills(
        &security.system_prompt,
        "security audit threat modeling injection memory safety vulnerability",
        Some("security"),
    );
    let security_agent = AutonomousAgent::new(provider.clone(), model, tools.clone())
        .with_system_prompt(sec_prompt)
        .with_temperature(0.3)
        .with_agentshield(true);

    let security_node = TaskNode::new(
        "ecc_security",
        "4b. Vulnerability & Threat Audit (ECC Security Auditor)",
        "IMPLEMENTATION TO AUDIT:\n\
        {ecc_implement.output}\n\n\
        TASK:\n\
        1. Probe for injection vulnerabilities, race conditions, memory leaks, and unchecked inputs.\n\
        2. Identify any unhandled boundary conditions or potential denial-of-service risks.\n\
        3. Recommend concrete hardening steps for any detected vulnerabilities.",
    )
    .with_agent(security_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(5), 2.0));

    // 5. Verify & Synthesize (Chief Adjudicator / Lakandiwa)
    let verify_prompt = equip_skills(
        "You are the Chief Verification Adjudicator under the ECC framework.\n\
        Synthesize all upstream outputs into a rock-solid, production-verified final deliverable.",
        "verification evaluation adjudication rubrics hallucination",
        Some("verification"),
    );
    let verify_agent = AutonomousAgent::new(provider.clone(), model, tools)
        .with_system_prompt(verify_prompt)
        .with_temperature(0.2)
        .with_agentshield(true);

    let verify_node = TaskNode::new(
        "ecc_verify",
        "5. Final Verification & Synthesis (ECC Chief Adjudicator)",
        format!(
            "ORIGINAL OBJECTIVE:\n\"{}\"\n\n\
            ARCHITECTURAL PLAN:\n\
            {{ecc_plan.output}}\n\n\
            IMPLEMENTATION:\n\
            {{ecc_implement.output}}\n\n\
            CODE REVIEW FEEDBACK:\n\
            {{ecc_review.output}}\n\n\
            SECURITY AUDIT FINDINGS:\n\
            {{ecc_security.output}}\n\n\
            TASK:\n\
            1. Apply any critical fixes identified by the Code Review and Security Audit.\n\
            2. Present the definitive, audited, verified implementation ready for production.\n\
            3. Summarize testing strategies and deployment readiness.",
            objective
        ),
    )
    .with_agent(verify_agent)
    .with_retry_policy(RetryPolicy::exponential(2, Duration::from_secs(5), 2.0));

    // Register all nodes
    graph.add_task(plan_node)?;
    graph.add_task(test_node)?;
    graph.add_task(implement_node)?;
    graph.add_task(review_node)?;
    graph.add_task(security_node)?;
    graph.add_task(verify_node)?;

    // Register dependency edges
    graph.add_dependency("ecc_plan", "ecc_test")?;
    graph.add_dependency("ecc_test", "ecc_implement")?;
    graph.add_dependency("ecc_implement", "ecc_review")?;
    graph.add_dependency("ecc_implement", "ecc_security")?;
    graph.add_dependency("ecc_review", "ecc_verify")?;
    graph.add_dependency("ecc_security", "ecc_verify")?;

    // Validate graph is strictly acyclic
    graph.validate()?;

    Ok(graph)
}
