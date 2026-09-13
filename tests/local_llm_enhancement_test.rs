//! # Brutal Verification Test Suite: Local LLM Supercharger & Grammar Decoding
//!
//! Validates:
//! 1. Constrained Structured Output / Grammar Decoding format parameter on CompletionRequest
//!    (both string format "json" and JSON Schema serde_json::Value), ensuring proper serialization.
//! 2. Production skill "local-llm-supercharger" parsing and invariant extraction with EccSkill::parse.
//! 3. Python evaluation error capture with tracebacks (ZeroDivisionError, SyntaxError) and agent feedback.

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tagisan::ecc::{
    all_built_in_skills, find_built_in_skill, load_skills_from_dir, EccSkill,
};
use tagisan::tools::{
    extract_missing_python_package, PythonEvalTool, ToolHandler, ToolRegistry,
};
use tagisan::types::{ChatSession, CompletionRequest, ContentBlock};

// =========================================================================
// Pillar 1: Constrained Structured Output & Format Serialization Tests
// =========================================================================

#[test]
fn test_completion_request_format_builder_and_default_none() {
    let req = CompletionRequest::new("dolphin-phi:latest", "Hello world");
    assert!(req.format.is_none(), "Default format should be None");

    let req_json_str = serde_json::to_string(&req).expect("Failed to serialize CompletionRequest");
    assert!(
        !req_json_str.contains("\"format\""),
        "When format is None, 'format' key must not be serialized: {}",
        req_json_str
    );
}

#[test]
fn test_completion_request_with_string_format_json() {
    let req = CompletionRequest::new("dolphin-phi:latest", "Output JSON")
        .with_format("json");

    assert_eq!(req.format, Some(Value::String("json".to_string())));

    let req_val: Value = serde_json::to_value(&req).expect("Failed to serialize to Value");
    assert_eq!(
        req_val.get("format"),
        Some(&Value::String("json".to_string())),
        "format must serialize to string \"json\""
    );
}

#[test]
fn test_completion_request_with_json_format_convenience() {
    let req = CompletionRequest::new("qwen2.5:7b", "Summarize")
        .with_json_format();

    assert_eq!(req.format, Some(Value::String("json".to_string())));

    let req_val: Value = serde_json::to_value(&req).expect("Failed to serialize to Value");
    assert_eq!(
        req_val.get("format"),
        Some(&Value::String("json".to_string()))
    );
}

#[test]
fn test_completion_request_with_structured_json_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "confidence": { "type": "number" },
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        },
        "required": ["name", "confidence"]
    });

    let req = CompletionRequest::new("deepseek-r1:14b", "Extract structured entities")
        .with_format(schema.clone());

    assert_eq!(req.format, Some(schema.clone()));

    let req_val: Value = serde_json::to_value(&req).expect("Failed to serialize to Value");
    let serialized_format = req_val.get("format").expect("format field must exist");
    assert_eq!(serialized_format, &schema);
    assert_eq!(serialized_format["properties"]["name"]["type"], "string");
    assert_eq!(serialized_format["required"][0], "name");
}

#[test]
fn test_completion_request_with_opt_format() {
    let req_none = CompletionRequest::new("llama3:8b", "Prompt")
        .with_opt_format(None);
    assert!(req_none.format.is_none());

    let req_some = CompletionRequest::new("llama3:8b", "Prompt")
        .with_opt_format(Some(json!({"type": "array"})));
    assert_eq!(req_some.format, Some(json!({"type": "array"})));
}

#[test]
fn test_chat_session_format_propagation() {
    let mut session = ChatSession::new()
        .with_system("System instructions")
        .with_format("json");

    session.add_user_message("Generate json payload");
    let req = session.build_request("dolphin-phi:latest");

    assert_eq!(req.format, Some(Value::String("json".to_string())));
    assert_eq!(req.system_prompt.as_deref(), Some("System instructions"));
    assert_eq!(req.messages.len(), 1);
}

// =========================================================================
// Pillar 2: "local-llm-supercharger" Skill Parsing & Invariant Extraction
// =========================================================================

#[test]
fn test_skill_parse_from_local_tagisan_dir() {
    let skill_path = Path::new("/home/dyna/TGS Projects/.tagisan/skills/local-llm-supercharger/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in .tagisan/skills/local-llm-supercharger");

    let skill = EccSkill::from_file(skill_path).expect("Failed to parse SKILL.md from disk");
    assert_eq!(skill.name, "local-llm-supercharger");
    assert!(
        skill.description.contains("CodeAct") || skill.description.contains("Python REPL"),
        "Description must contain CodeAct / Python REPL reference: {}",
        skill.description
    );
    assert!(
        skill.instructions.contains("Invariant 1: In-Weights Arithmetic Prohibition"),
        "Must contain Invariant 1"
    );
    assert!(
        skill.instructions.contains("Invariant 2: Complex JSON / Data Transformations"),
        "Must contain Invariant 2"
    );
    assert!(
        skill.instructions.contains("Systematic Error Recovery & Traceback Self-Healing"),
        "Must contain Traceback Self-Healing protocol"
    );
    assert!(
        skill.instructions.contains("Test-Time Self-Critique & Reflection Checklist"),
        "Must contain Reflection Checklist"
    );
}

#[test]
fn test_skill_parse_from_assets_catalog_dir() {
    let skill_path = Path::new("/home/dyna/TGS Projects/tagisan/assets/skills/local-llm-supercharger/SKILL.md");
    assert!(skill_path.is_file(), "SKILL.md must exist in assets/skills/local-llm-supercharger");

    let content = fs::read_to_string(skill_path).expect("Failed to read assets skill file");
    let skill = EccSkill::parse(&content).expect("Failed to parse assets SKILL.md");
    assert_eq!(skill.name, "local-llm-supercharger");
}

#[test]
fn test_skill_built_in_catalog_registration() {
    let all_skills = all_built_in_skills();
    let found = all_skills.iter().find(|s| s.name == "local-llm-supercharger");
    assert!(found.is_some(), "local-llm-supercharger must be present in built-in skills catalog");

    let retrieved = find_built_in_skill("local-llm-supercharger");
    assert!(retrieved.is_some(), "find_built_in_skill must resolve local-llm-supercharger");
    let skill = retrieved.unwrap();
    assert_eq!(skill.name, "local-llm-supercharger");
    assert!(skill.instructions.contains("Traceback"));
}

#[test]
fn test_skill_dir_discovery() {
    let dir = Path::new("/home/dyna/TGS Projects/.tagisan/skills");
    let loaded = load_skills_from_dir(dir);
    let found = loaded.iter().find(|s| s.name == "local-llm-supercharger");
    assert!(found.is_some(), "load_skills_from_dir must discover local-llm-supercharger in .tagisan/skills");
}

// =========================================================================
// Pillar 3: Python Evaluation Error Capture, Tracebacks & Tool Feedback
// =========================================================================

#[tokio::test]
async fn test_python_eval_success_execution() {
    let tool = PythonEvalTool::new();
    if !tagisan::python::PythonRuntime::new().map(|r| r.is_available()).unwrap_or(false) {
        eprintln!("Skipping python runtime test: Python 3 not detected on host system");
        return;
    }

    let code = "print(355 / 113)";
    let output = tool
        .execute(json!({ "code": code }))
        .await
        .expect("Python execution should succeed");

    assert!(output.contains("Exit Code: 0"), "Exit code must be 0: {}", output);
    assert!(output.contains("3.14159"), "Output must contain calculation result: {}", output);
}

#[tokio::test]
async fn test_python_eval_traceback_capture_zero_division() {
    let tool = PythonEvalTool::new();
    if !tagisan::python::PythonRuntime::new().map(|r| r.is_available()).unwrap_or(false) {
        eprintln!("Skipping python runtime test: Python 3 not detected on host system");
        return;
    }

    // Explicit division by zero to trigger ZeroDivisionError traceback
    let code = "x = 42\ny = 0\nprint(x / y)";
    let output = tool
        .execute(json!({ "code": code }))
        .await
        .expect("Tool execute returns Ok with captured process exit status");

    assert!(
        output.contains("Exit Code: 1") || output.contains("Exit Code: 255"),
        "Non-zero exit code expected: {}",
        output
    );
    assert!(
        output.contains("--- STDERR ---"),
        "STDERR block must be present: {}",
        output
    );
    assert!(
        output.contains("Traceback (most recent call last):"),
        "Python traceback header must be present: {}",
        output
    );
    assert!(
        output.contains("ZeroDivisionError: division by zero"),
        "ZeroDivisionError message must be present for model reflection: {}",
        output
    );
}

#[tokio::test]
async fn test_python_eval_syntax_error_capture() {
    let tool = PythonEvalTool::new();
    if !tagisan::python::PythonRuntime::new().map(|r| r.is_available()).unwrap_or(false) {
        eprintln!("Skipping python runtime test: Python 3 not detected on host system");
        return;
    }

    let code = "def broken_fn(\n  print('missing paren')";
    let output = tool
        .execute(json!({ "code": code }))
        .await
        .expect("Tool execute returns Ok with process status");

    assert!(
        output.contains("Exit Code: 1") || output.contains("Exit Code: 255"),
        "Exit code must be non-zero for syntax errors: {}",
        output
    );
    assert!(
        output.contains("SyntaxError"),
        "SyntaxError must be captured in output: {}",
        output
    );
}

#[test]
fn test_python_missing_package_extraction() {
    let stderr_sample = r#"
Traceback (most recent call last):
  File "<string>", line 1, in <module>
ModuleNotFoundError: No module named 'scipy'
"#;

    let extracted = extract_missing_python_package(stderr_sample);
    assert_eq!(extracted, Some("scipy".to_string()));

    let nested_sample = "ModuleNotFoundError: No module named 'scipy.optimize'";
    let extracted_nested = extract_missing_python_package(nested_sample);
    assert_eq!(extracted_nested, Some("scipy".to_string()));
}

#[tokio::test]
async fn test_tool_registry_execution_and_content_block_formatting() {
    let mut registry = ToolRegistry::new();
    registry.register_tool(PythonEvalTool::new());

    if !tagisan::python::PythonRuntime::new().map(|r| r.is_available()).unwrap_or(false) {
        return;
    }

    // 1. Tool call that fails inside Python
    let call_id = "call_python_err_001";
    let args = json!({ "code": "raise ValueError('Invalid tensor dimension')" });
    let block = registry.execute_call(call_id, "python_eval", &args).await;

    match block {
        ContentBlock::ToolResult { tool_call_id, content, is_error: _ } => {
            assert_eq!(tool_call_id, call_id);
            assert!(
                content.contains("ValueError: Invalid tensor dimension"),
                "Captured traceback must be returned in tool result content: {}",
                content
            );
            assert!(content.contains("Exit Code: 1") || content.contains("Exit Code: 255"));
        }
        other => panic!("Expected ToolResult, got {:?}", other),
    }

    // 2. Unregistered tool call returns an explicit error
    let unregistered = registry.execute_call("call_fake", "non_existent_tool", &json!({})).await;
    match unregistered {
        ContentBlock::ToolResult { is_error, content, .. } => {
            assert!(is_error, "Unregistered tool must return is_error: true");
            assert!(content.contains("is not registered in ToolRegistry"));
        }
        other => panic!("Expected ToolResult, got {:?}", other),
    }
}
