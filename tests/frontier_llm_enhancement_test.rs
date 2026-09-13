use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use tagisan::{
    extract_triggers_from_text, BlastRisk, CalculateBlastRadiusTool,
    CodebaseGraph, EccSkill, GroundedInferenceTool, PythonEvalTool,
    QueryCodeGraphTool, SymbolKind, ToolHandler,
};

/// Helper RAII struct for deterministic cleanup of temporary test directories
struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "tgs_frontier_{prefix}_{}_{}",
            std::process::id(),
            std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("Failed to create temp directory");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// =========================================================================
// Test 1: Skill Parsing & Frontmatter Integrity
// =========================================================================

#[test]
fn test_frontier_ast_graph_navigator_skill_parsing_and_invariants() {
    // Check multiple candidate locations where the skill is deployed
    let candidate_paths = [
        PathBuf::from("/home/dyna/TGS Projects/.tagisan/skills/frontier-ast-graph-navigator/SKILL.md"),
        PathBuf::from("/home/dyna/TGS Projects/.ecc/skills/frontier-ast-graph-navigator/SKILL.md"),
        PathBuf::from(".tagisan/skills/frontier-ast-graph-navigator/SKILL.md"),
        PathBuf::from(".ecc/skills/frontier-ast-graph-navigator/SKILL.md"),
    ];

    let mut found_any = false;
    for path in &candidate_paths {
        if path.exists() {
            found_any = true;
            let content = fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

            let skill = EccSkill::parse(&content)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));

            // Verify metadata
            assert_eq!(skill.name, "frontier-ast-graph-navigator");
            assert!(
                skill.description.contains("AST codebase graph navigation"),
                "Description missing AST graph navigation: {}",
                skill.description
            );
            assert!(
                skill.description.contains("blast-radius"),
                "Description missing blast-radius: {}",
                skill.description
            );
            assert!(
                skill.description.contains("dialectical critique"),
                "Description missing dialectical critique: {}",
                skill.description
            );
            assert!(
                skill.description.contains("closed-loop TDD"),
                "Description missing closed-loop TDD: {}",
                skill.description
            );

            // Verify all 4 Critical Pillars in instructions
            assert!(
                skill.instructions.contains("Pillar 1: AST Codebase Knowledge Graph & Transitive Blast-Radius Navigation"),
                "Skill missing Pillar 1 in instructions"
            );
            assert!(
                skill.instructions.contains("Pillar 2: Adversarial Dialectical Critique (Lakandiwa Synthesis)"),
                "Skill missing Pillar 2 in instructions"
            );
            assert!(
                skill.instructions.contains("Pillar 3: Closed-Loop TDD & Formal Invariant Proving"),
                "Skill missing Pillar 3 in instructions"
            );
            assert!(
                skill.instructions.contains("Pillar 4: Hierarchical Episodic Memory & Dynamic Context Paging"),
                "Skill missing Pillar 4 in instructions"
            );

            // Verify tool operation protocols
            assert!(skill.instructions.contains("query_code_graph"));
            assert!(skill.instructions.contains("calculate_blast_radius"));
            assert!(skill.instructions.contains("LOW RISK"));
            assert!(skill.instructions.contains("MEDIUM RISK"));
            assert!(skill.instructions.contains("HIGH RISK"));
            assert!(skill.instructions.contains("CRITICAL RISK"));

            // Verify trigger extraction
            let triggers = extract_triggers_from_text(&skill.name, &skill.description, &[]);
            assert!(
                !triggers.is_empty(),
                "Triggers must be extracted from frontmatter"
            );
            assert!(
                triggers.iter().any(|t| t.contains("graph") || t.contains("ast") || t.contains("blast")),
                "Expected AST/graph/blast trigger in {:?}",
                triggers
            );
        }
    }

    assert!(
        found_any,
        "Expected at least one frontier-ast-graph-navigator SKILL.md file to exist"
    );
}

// =========================================================================
// Test 2: Live AST Call Graph Creation & Symbol Queries
// =========================================================================

#[test]
fn test_live_ast_call_graph_creation_and_queries() {
    let temp = TempDirGuard::new("graph_queries");
    let src_dir = temp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // Author service.rs with structs, methods, functions, and internal calls
    let service_rs = r#"
pub struct OrderProcessor;

impl OrderProcessor {
    pub fn process_order(&self) {
        validate_payment();
        record_audit_log();
    }
}

pub fn validate_payment() {
    charge_gateway();
}

pub fn record_audit_log() {}

pub fn charge_gateway() {}
"#;
    fs::write(src_dir.join("service.rs"), service_rs).unwrap();

    // Author controller.rs calling process_order
    let controller_rs = r#"
pub fn handle_checkout() {
    let proc = OrderProcessor;
    proc.process_order();
}

pub fn external_webhook() {
    handle_checkout();
}
"#;
    fs::write(src_dir.join("controller.rs"), controller_rs).unwrap();

    // Build knowledge graph
    let graph = CodebaseGraph::build_from_dir(temp.path(), 100)
        .expect("CodebaseGraph::build_from_dir failed");

    // Query definitions
    let order_proc = graph.find_symbol("OrderProcessor");
    assert!(!order_proc.is_empty(), "OrderProcessor struct not found");
    assert_eq!(order_proc[0].kind, SymbolKind::Struct);

    let process_order = graph.find_symbol("process_order");
    assert!(!process_order.is_empty(), "process_order method not found");
    assert_eq!(process_order[0].kind, SymbolKind::Method);

    let validate_pay = graph.find_symbol("validate_payment");
    assert!(!validate_pay.is_empty(), "validate_payment fn not found");
    assert_eq!(validate_pay[0].kind, SymbolKind::Function);

    // Query callers: validate_payment is called by process_order
    let callers = graph.find_callers("validate_payment");
    assert!(
        !callers.is_empty(),
        "Expected callers for validate_payment, found 0"
    );
    assert!(
        callers.iter().any(|(s, _)| s.name == "process_order" || s.qualified_name.contains("process_order")),
        "Expected process_order to be caller of validate_payment. Callers: {:?}",
        callers.iter().map(|(s, _)| &s.qualified_name).collect::<Vec<_>>()
    );

    // Query callees: process_order calls validate_payment and record_audit_log
    let callees = graph.find_callees("process_order");
    assert!(
        callees.len() >= 2,
        "Expected at least 2 callees for process_order, found {}",
        callees.len()
    );
    let callee_names: Vec<_> = callees.iter().map(|(s, _)| s.name.as_str()).collect();
    assert!(
        callee_names.contains(&"validate_payment"),
        "callees should contain validate_payment: {:?}",
        callee_names
    );
    assert!(
        callee_names.contains(&"record_audit_log"),
        "callees should contain record_audit_log: {:?}",
        callee_names
    );

    // Check stats
    let stats = graph.stats();
    assert!(stats.total_nodes >= 6, "Expected at least 6 nodes, got {}", stats.total_nodes);
    assert!(stats.total_edges >= 4, "Expected at least 4 edges, got {}", stats.total_edges);
    assert_eq!(stats.files_count, 2);
}

// =========================================================================
// Test 3: Transitive Blast Radius Computation & Risk Tiers
// =========================================================================

#[test]
fn test_blast_radius_computation_and_all_risk_tiers() {
    let temp = TempDirGuard::new("blast_radius");
    let src_dir = temp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    // 1. Target with LOW risk (0-2 dependents):
    // isolated_leaf() is called only by single_caller()
    let low_rs = r#"
pub fn isolated_leaf() {}

pub fn single_caller() {
    isolated_leaf();
}
"#;
    fs::write(src_dir.join("low.rs"), low_rs).unwrap();

    // 2. Target with MEDIUM risk (3-5 dependents):
    // medium_hub() is called by m1(), m2(), m3(), m4()
    let medium_rs = r#"
pub fn medium_hub() {}

pub fn m1() { medium_hub(); }
pub fn m2() { medium_hub(); }
pub fn m3() { medium_hub(); }
pub fn m4() { medium_hub(); }
"#;
    fs::write(src_dir.join("medium.rs"), medium_rs).unwrap();

    // 3. Target with HIGH risk (6-12 dependents):
    // high_hub() is called by h1..h7 (7 callers)
    let high_rs = r#"
pub fn high_hub() {}

pub fn h1() { high_hub(); }
pub fn h2() { high_hub(); }
pub fn h3() { high_hub(); }
pub fn h4() { high_hub(); }
pub fn h5() { high_hub(); }
pub fn h6() { high_hub(); }
pub fn h7() { high_hub(); }
"#;
    fs::write(src_dir.join("high.rs"), high_rs).unwrap();

    // 4. Target with CRITICAL risk:
    // Either 13+ callers OR Trait with >= 3 implementing types
    // Let's create critical_trait with 3 implementing types: StructA, StructB, StructC
    let critical_rs = r#"
pub trait CriticalContract {
    fn execute_contract(&self);
}

pub struct ServiceA;
impl CriticalContract for ServiceA {
    fn execute_contract(&self) {}
}

pub struct ServiceB;
impl CriticalContract for ServiceB {
    fn execute_contract(&self) {}
}

pub struct ServiceC;
impl CriticalContract for ServiceC {
    fn execute_contract(&self) {}
}
"#;
    fs::write(src_dir.join("critical.rs"), critical_rs).unwrap();

    // Build the graph
    let graph = CodebaseGraph::build_from_dir(temp.path(), 100)
        .expect("Build graph for blast radius");

    // Evaluate LOW tier
    let low_report = graph.calculate_blast_radius("isolated_leaf", 3)
        .expect("calculate_blast_radius for isolated_leaf");
    assert_eq!(low_report.risk_level, BlastRisk::Low);
    assert_eq!(low_report.risk_level.as_str(), "LOW");
    assert!(low_report.direct_callers.len() <= 2);
    assert!(
        low_report.recommendations.iter().any(|r| r.contains("Low blast radius") || r.contains("safe")),
        "Expected Low risk recommendation in {:?}",
        low_report.recommendations
    );

    // Evaluate MEDIUM tier
    let med_report = graph.calculate_blast_radius("medium_hub", 3)
        .expect("calculate_blast_radius for medium_hub");
    assert_eq!(med_report.risk_level, BlastRisk::Medium);
    assert_eq!(med_report.risk_level.as_str(), "MEDIUM");
    assert_eq!(med_report.direct_callers.len(), 4);
    assert!(
        med_report.recommendations.iter().any(|r| r.contains("Medium blast radius")),
        "Expected Medium risk recommendation in {:?}",
        med_report.recommendations
    );

    // Evaluate HIGH tier
    let high_report = graph.calculate_blast_radius("high_hub", 3)
        .expect("calculate_blast_radius for high_hub");
    assert_eq!(high_report.risk_level, BlastRisk::High);
    assert_eq!(high_report.risk_level.as_str(), "HIGH");
    assert_eq!(high_report.direct_callers.len(), 7);
    assert!(
        high_report.recommendations.iter().any(|r| r.contains("HIGH RISK")),
        "Expected High risk recommendation in {:?}",
        high_report.recommendations
    );

    // Evaluate CRITICAL tier
    let crit_report = graph.calculate_blast_radius("CriticalContract", 3)
        .expect("calculate_blast_radius for CriticalContract");
    assert_eq!(crit_report.risk_level, BlastRisk::Critical);
    assert_eq!(crit_report.risk_level.as_str(), "CRITICAL");
    assert!(
        crit_report.implementing_types.len() >= 3,
        "Expected at least 3 implementing types, got {}",
        crit_report.implementing_types.len()
    );
    assert!(
        crit_report.recommendations.iter().any(|r| r.contains("CRITICAL ARCHITECTURAL HUB") || r.contains("BREAKING CHANGES FORBIDDEN")),
        "Expected Critical risk recommendation in {:?}",
        crit_report.recommendations
    );
}

// =========================================================================
// Test 4: Execution of QueryCodeGraphTool via ToolHandler::execute
// =========================================================================

#[tokio::test]
async fn test_query_code_graph_tool_execution() {
    let temp = TempDirGuard::new("tool_query");
    let src_dir = temp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let code = r#"
pub struct TokenBudgetTracker {
    total_tokens: usize,
}

impl TokenBudgetTracker {
    pub fn track_tokens(&mut self, amount: usize) {
        self.total_tokens += amount;
    }
}

pub fn record_usage(tracker: &mut TokenBudgetTracker) {
    tracker.track_tokens(42);
}
"#;
    fs::write(src_dir.join("tracker.rs"), code).unwrap();

    let tool = QueryCodeGraphTool::new().with_working_dir(temp.path());
    assert_eq!(tool.name(), "query_code_graph");
    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");

    // 1. Query Definition
    let def_args = json!({
        "query": "TokenBudgetTracker",
        "direction": "definition",
        "path": "."
    });
    let def_out = tool.execute(def_args).await.expect("Tool execute definition failed");
    assert!(def_out.contains("### Symbol Definitions Matching `TokenBudgetTracker`"));
    assert!(def_out.contains("TokenBudgetTracker"));
    assert!(def_out.contains("struct"));

    // 2. Query Callers
    let caller_args = json!({
        "query": "track_tokens",
        "direction": "callers",
        "path": "."
    });
    let caller_out = tool.execute(caller_args).await.expect("Tool execute callers failed");
    assert!(caller_out.contains("### Incoming Callers of `track_tokens`"));
    assert!(caller_out.contains("record_usage"));

    // 3. Query Callees
    let callee_args = json!({
        "query": "record_usage",
        "direction": "callees",
        "path": "."
    });
    let callee_out = tool.execute(callee_args).await.expect("Tool execute callees failed");
    assert!(callee_out.contains("### Outgoing Calls from `record_usage`"));
    assert!(callee_out.contains("track_tokens"));

    // 4. Query Non-Existent Symbol
    let missing_args = json!({
        "query": "NonExistentSymbol_XYZ",
        "direction": "definition",
        "path": "."
    });
    let missing_out = tool.execute(missing_args).await.expect("Tool execute missing failed");
    assert!(missing_out.contains("No symbols found matching query"));
}

// =========================================================================
// Test 5: Execution of CalculateBlastRadiusTool via ToolHandler::execute
// =========================================================================

#[tokio::test]
async fn test_calculate_blast_radius_tool_execution() {
    let temp = TempDirGuard::new("tool_blast");
    let src_dir = temp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let code = r#"
pub fn central_dispatch() {
    subsystem_worker();
}

pub fn subsystem_worker() {}

pub fn entry_point() {
    central_dispatch();
}
"#;
    fs::write(src_dir.join("dispatch.rs"), code).unwrap();

    let tool = CalculateBlastRadiusTool::new().with_working_dir(temp.path());
    assert_eq!(tool.name(), "calculate_blast_radius");
    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");

    // Execute blast radius on central_dispatch
    let args = json!({
        "target": "central_dispatch",
        "max_depth": 3,
        "path": "."
    });
    let out = tool.execute(args).await.expect("Tool execute blast radius failed");

    assert!(out.contains("# Blast Radius Analysis: `central_dispatch`"));
    assert!(out.contains("**Assessed Risk Level**:"));
    assert!(out.contains("**Total Affected Symbols**:"));
    assert!(out.contains("Direct Callers"));
    assert!(out.contains("entry_point"));
    assert!(out.contains("Refactoring Recommendations"));

    // Check invalid symbol returns error
    let invalid_args = json!({
        "target": "CompletelyUnknownSymbol_404",
        "path": "."
    });
    let err_res = tool.execute(invalid_args).await;
    assert!(err_res.is_err(), "Expected error for unknown symbol blast radius");
}

// =========================================================================
// Test 6: Validation of GroundedInferenceTool & PythonEvalTool Descriptors
// =========================================================================

#[tokio::test]
async fn test_frontier_auxiliary_tools_descriptors() {
    let ground_tool = GroundedInferenceTool::new();
    assert_eq!(ground_tool.name(), "grounded_inference");
    let g_schema = ground_tool.parameters_schema();
    assert_eq!(g_schema["type"], "object");
    assert!(g_schema["properties"].get("task").is_some());
    assert!(g_schema["properties"].get("language").is_some());

    let python_tool = PythonEvalTool::new();
    assert_eq!(python_tool.name(), "python_eval");
    let p_schema = python_tool.parameters_schema();
    assert_eq!(p_schema["type"], "object");
    assert!(p_schema["properties"].get("code").is_some());
}

// =========================================================================
// Test 7: CodebaseGraph Serialization & DOT Export
// =========================================================================

#[test]
fn test_codebase_graph_serialization_and_dot_export() {
    let temp = TempDirGuard::new("graph_export");
    let src_dir = temp.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();

    let code = r#"
pub fn alfa() {
    bravo();
}
pub fn bravo() {}
"#;
    fs::write(src_dir.join("lib.rs"), code).unwrap();

    let graph = CodebaseGraph::build_from_dir(temp.path(), 50).unwrap();

    // Verify DOT format
    let dot = graph.export_dot();
    assert!(dot.contains("digraph CodebaseGraph {"), "DOT missing digraph header");
    assert!(dot.contains("->"), "DOT missing edges");
    assert!(dot.contains("alfa"), "DOT missing node alfa");
    assert!(dot.contains("bravo"), "DOT missing node bravo");

    // Verify JSON format
    let json_str = graph.export_json().unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(json_val["nodes"].is_array());
    assert!(json_val["edges"].is_array());
    assert!(json_val["stats"].is_object());
    assert!(json_val["stats"]["total_nodes"].as_u64().unwrap() >= 2);
}
