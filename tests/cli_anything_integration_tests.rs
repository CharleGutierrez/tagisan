use std::fs;
use std::path::{Path, PathBuf};

use tagisan::ecc::skills::{EccSkill, SkillDispatcher};
use tagisan::harness::analyzer::CodeAnalyzer;
use tagisan::harness::generator::HarnessGenerator;
use tagisan::harness::pipeline::{HarnessPipeline, HarnessPipelineOptions};
use tagisan::harness::runner::HarnessRunner;
use tagisan::harness::skill_packager::SkillPackager;
use tagisan::harness::spec::{ArgumentSpec, ArgumentType, CommandSpec, HarnessSpec, SourceType};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = format!("{}_{}_{}", prefix, std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("Failed to create temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn tempdir() -> std::io::Result<TempDir> {
    Ok(TempDir::new("tgs_cli_test"))
}

// =========================================================================
// 1. Python Source Code Analysis (Signatures, Docstrings, Types)
// =========================================================================

#[test]
fn test_python_source_analysis_signatures_and_docstrings() {
    let python_code = r#"
"""
Image Processing Utility Suite
High-performance computer vision helper functions.
"""

def resize_image(path: str, width: int = 800, height: int = 600, maintain_aspect: bool = True) -> dict:
    """
    Resize an input image to specified dimensions.

    Args:
        path (str): Filepath to the source image.
        width (int): Target pixel width.
        height (int): Target pixel height.
        maintain_aspect (bool): Whether to preserve aspect ratio.

    Returns:
        dict: Metadata with dimensions and file size.
    """
    return {"status": "resized", "width": width, "height": height}

def convert_format(source: str, target_format: str = "png") -> str:
    """
    Convert image to different encoding format.
    :param source: Input file path.
    :param target_format: Output extension.
    """
    return f"{source}.{target_format}"

class WatermarkProcessor:
    """Class that applies watermarks."""

    def apply_watermark(self, image_path: str, text: str, opacity: float = 0.5) -> bool:
        """Apply text watermark with specified opacity."""
        return True
"#;

    let analysis = CodeAnalyzer::analyze_source(python_code, SourceType::Python, Some("image_utils"), None)
        .expect("Analysis should succeed");

    assert_eq!(analysis.source_name, "image_utils");
    assert_eq!(analysis.source_type, SourceType::Python);
    assert!(analysis.module_docstring.is_some());
    assert!(analysis.module_docstring.as_ref().unwrap().contains("Image Processing Utility Suite"));

    // Verify functions
    assert_eq!(analysis.functions.len(), 2);

    let resize_fn = analysis.functions.iter().find(|f| f.name == "resize_image").expect("resize_image must be found");
    assert_eq!(resize_fn.parameters.len(), 4);
    assert_eq!(resize_fn.parameters[0].name, "path");
    assert_eq!(resize_fn.parameters[0].param_type, "str");
    assert!(resize_fn.parameters[0].required);
    assert_eq!(resize_fn.parameters[0].description.as_deref(), Some("Filepath to the source image."));

    assert_eq!(resize_fn.parameters[1].name, "width");
    assert_eq!(resize_fn.parameters[1].default_value.as_deref(), Some("800"));
    assert!(!resize_fn.parameters[1].required);

    let convert_fn = analysis.functions.iter().find(|f| f.name == "convert_format").expect("convert_format must be found");
    assert_eq!(convert_fn.parameters.len(), 2);
    assert_eq!(convert_fn.parameters[0].name, "source");
    assert_eq!(convert_fn.parameters[0].description.as_deref(), Some("Input file path."));

    // Verify classes
    assert_eq!(analysis.classes.len(), 1);
    let wm_class = &analysis.classes[0];
    assert_eq!(wm_class.name, "WatermarkProcessor");
    assert_eq!(wm_class.methods.len(), 1);
    let wm_method = &wm_class.methods[0];
    assert_eq!(wm_method.name, "apply_watermark");
    // self should be stripped
    assert_eq!(wm_method.parameters.len(), 3);
    assert_eq!(wm_method.parameters[0].name, "image_path");
    assert_eq!(wm_method.parameters[1].name, "text");
    assert_eq!(wm_method.parameters[2].name, "opacity");
}

// =========================================================================
// 2. Multi-Lingual Parameter Parsing (Rust, TypeScript, OpenAPI)
// =========================================================================

#[test]
fn test_multilingual_source_analysis_rust_ts_openapi() {
    // 2a. Rust analysis
    let rust_code = r#"
//! Data Transformation Toolkit
//! Fast cryptographic hash computations.

/// Compute SHA-256 digest of input message
pub fn compute_sha256(message: &str, rounds: u32) -> String {
    format!("hash_{}_{}", message, rounds)
}

pub struct TokenBucket;
"#;
    let rs_analysis = CodeAnalyzer::analyze_source(rust_code, SourceType::Rust, Some("crypto_tools"), None)
        .expect("Rust analysis should succeed");
    assert_eq!(rs_analysis.functions.len(), 1);
    assert_eq!(rs_analysis.functions[0].name, "compute_sha256");
    assert_eq!(rs_analysis.functions[0].parameters.len(), 2);
    assert_eq!(rs_analysis.functions[0].parameters[0].name, "message");
    assert_eq!(rs_analysis.classes.len(), 1);
    assert_eq!(rs_analysis.classes[0].name, "TokenBucket");

    // 2b. TypeScript analysis
    let ts_code = r#"
/**
 * Calculate order total with tax.
 * @param {number} subtotal Base price before tax
 * @param {number} taxRate Tax percentage multiplier
 */
export function calculateTotal(subtotal: number, taxRate: number = 0.08): number {
    return subtotal * (1 + taxRate);
}
"#;
    let ts_analysis = CodeAnalyzer::analyze_source(ts_code, SourceType::TypeScript, Some("billing"), None)
        .expect("TypeScript analysis should succeed");
    assert_eq!(ts_analysis.functions.len(), 1);
    let billing_fn = &ts_analysis.functions[0];
    assert_eq!(billing_fn.name, "calculateTotal");
    assert_eq!(billing_fn.parameters.len(), 2);
    assert_eq!(billing_fn.parameters[0].name, "subtotal");
    assert_eq!(billing_fn.parameters[0].description.as_deref(), Some("Base price before tax"));

    // 2c. OpenAPI analysis
    let openapi_json = r#"{
      "openapi": "3.0.0",
      "info": {
        "title": "Pet Store API",
        "description": "Endpoints for managing pets."
      },
      "paths": {
        "/pets": {
          "get": {
            "operationId": "list_pets",
            "summary": "List all pets",
            "parameters": [
              {
                "name": "limit",
                "in": "query",
                "required": false,
                "schema": { "type": "integer" },
                "description": "Max number of pets to return"
              }
            ]
          }
        }
      }
    }"#;
    let openapi_analysis = CodeAnalyzer::analyze_source(openapi_json, SourceType::OpenApi, Some("petstore"), None)
        .expect("OpenAPI analysis should succeed");
    assert_eq!(openapi_analysis.functions.len(), 1);
    assert_eq!(openapi_analysis.functions[0].name, "list_pets");
    assert_eq!(openapi_analysis.functions[0].parameters.len(), 1);
    assert_eq!(openapi_analysis.functions[0].parameters[0].name, "limit");
    assert_eq!(openapi_analysis.functions[0].parameters[0].description.as_deref(), Some("Max number of pets to return"));
}

// =========================================================================
// 3. Deterministic Generation of Executable Python CLI with --json
// =========================================================================

#[test]
fn test_deterministic_cli_harness_generation_with_json() {
    let mut spec = HarnessSpec::new("math-engine", PathBuf::from("math_engine.py"), SourceType::Python);
    spec.description = "Autonomous mathematical calculation engine".to_string();

    let mut add_cmd = CommandSpec::new("add-numbers", "add");
    add_cmd.description = "Add two numbers together".to_string();
    add_cmd.arguments.push(
        ArgumentSpec::new("a", ArgumentType::Float)
            .with_description("First operand")
            .with_required(true),
    );
    add_cmd.arguments.push(
        ArgumentSpec::new("b", ArgumentType::Float)
            .with_description("Second operand")
            .with_required(true),
    );
    spec.commands.push(add_cmd);

    let generated = HarnessGenerator::generate(&spec).expect("Harness generation must succeed");

    // Assert CLI contents
    assert!(generated.cli_code.contains("#!/usr/bin/env python3"));
    assert!(generated.cli_code.contains(".add_argument('--json'"));
    assert!(generated.cli_code.contains("'status': 'success'"));
    assert!(generated.cli_code.contains("'status': 'error'"));
    assert!(generated.cli_code.contains("handle_add_numbers"));
    assert!(generated.cli_code.contains("sys.exit(0)"));
    assert!(generated.cli_code.contains("sys.exit(1)"));

    // Assert Test contents
    assert!(generated.test_code.contains("class TestSynthesizedHarness(unittest.TestCase):"));
    assert!(generated.test_code.contains("test_01_help_flag"));
    assert!(generated.test_code.contains("test_03_add_numbers_json_success"));
    assert!(generated.test_code.contains("test_03_add_numbers_missing_args"));
}

// =========================================================================
// 4. Generation of RFC-004 Compliant SKILL.md Packages
// =========================================================================

#[test]
fn test_rfc004_skill_packager_compliance() {
    let mut spec = HarnessSpec::new("vector-db", PathBuf::from("vector_db.py"), SourceType::Python);
    spec.description = "Embedded vector indexing and nearest neighbor search".to_string();

    let mut search_cmd = CommandSpec::new("search", "query_similar");
    search_cmd.description = "Find nearest neighbors to a vector query".to_string();
    search_cmd.arguments.push(
        ArgumentSpec::new("query", ArgumentType::String)
            .with_description("Semantic query text")
            .with_required(true),
    );
    search_cmd.arguments.push(
        ArgumentSpec::new("k", ArgumentType::Integer)
            .with_description("Number of top results")
            .with_default("5"),
    );
    spec.commands.push(search_cmd);

    let generated = HarnessGenerator::generate(&spec).expect("Harness generation must succeed");
    let packaged = SkillPackager::package(&spec, &generated).expect("Skill packaging must succeed");

    // 1. YAML frontmatter validation
    assert!(packaged.skill_md_content.starts_with("---\n"));
    assert!(packaged.skill_md_content.contains("name: vector-db\n"));
    assert!(packaged.skill_md_content.contains("triggers:\n"));

    // 2. Local LLM Cheat Sheet validation
    assert!(packaged.skill_md_content.contains("### [LOCAL LLM CHEAT SHEET: ACTIONABLE CONSTRAINTS & INVARIANTS]"));
    assert!(packaged.skill_md_content.contains("Always pass `--json`"));
    assert!(packaged.skill_md_content.contains("- `search`: Find nearest neighbors to a vector query"));

    // 3. Cloud Comprehensive Specification validation
    assert!(packaged.skill_md_content.contains("### [COMPREHENSIVE ARCHITECTURAL SPECIFICATIONS & ENGINEERING SKILLS]"));
    assert!(packaged.skill_md_content.contains("##### `search`"));
    assert!(packaged.skill_md_content.contains("| Flag / Arg | Type | Required | Default | Description |"));
    assert!(packaged.skill_md_content.contains("```json\n{\n  \"status\": \"success\""));

    // 4. Ingest with Tagisan EccSkill parser to verify 100% compliance
    let parsed_skill = EccSkill::parse(&packaged.skill_md_content).expect("EccSkill must parse cleanly");
    assert_eq!(parsed_skill.name, "vector-db");
    assert!(parsed_skill.description.contains("Agent-native CLI harness for vector-db"));
    assert!(parsed_skill.instructions.contains("LOCAL LLM CHEAT SHEET"));
}

// =========================================================================
// 5. Auto-Registration into Skills Catalog & SkillDispatcher Discovery
// =========================================================================

#[test]
fn test_auto_registration_and_dispatcher_discovery() {
    let tmp = tempdir().expect("Temporary directory creation failed");
    let skills_dir = tmp.path().join("skills");
    fs::create_dir_all(&skills_dir).expect("Skills dir creation failed");

    let mut spec = HarnessSpec::new("crypto-hasher", PathBuf::from("crypto.py"), SourceType::Python);
    spec.description = "Cryptographic hashing and signature verification harness".to_string();

    let mut hash_cmd = CommandSpec::new("blake3-digest", "digest");
    hash_cmd.description = "Compute BLAKE3 256-bit message digest".to_string();
    hash_cmd.arguments.push(ArgumentSpec::new("payload", ArgumentType::String).with_required(true));
    spec.commands.push(hash_cmd);

    let generated = HarnessGenerator::generate(&spec).expect("Generate failed");
    let packaged = SkillPackager::package(&spec, &generated).expect("Package failed");

    // Install into temporary skills root
    let installed_dir = packaged.install_to(&skills_dir).expect("Installation must succeed");
    assert!(installed_dir.exists());
    assert!(installed_dir.join("SKILL.md").is_file());
    assert!(installed_dir.join("crypto_hasher_cli.py").is_file());

    // Verify discovery via Tagisan's SkillDispatcher
    let dispatcher = SkillDispatcher::new(Some(&skills_dir));
    let results = dispatcher.dispatch("blake3 cryptographic message digest", 3, None);
    assert!(!results.is_empty(), "Dispatcher must find the newly installed skill");
    assert_eq!(results[0].skill.name, "crypto-hasher");
}

// =========================================================================
// 6. AgentShield Security Enforcement
// =========================================================================

#[tokio::test]
async fn test_agentshield_security_enforcement() {
    let runner = HarnessRunner::new().expect("Runner init must succeed");

    // 6a. Attempt dangerous command arguments (command injection / destructive file removal)
    let malicious_args = vec!["run".to_string(), "; rm -rf / ;".to_string()];
    let blocked_res = runner.run("app_cli.py", &malicious_args, None).await;
    assert!(blocked_res.is_err(), "AgentShield must block destructive shell command injection");
    let err_msg = blocked_res.err().unwrap().to_string();
    assert!(err_msg.contains("AgentShield blocked harness command"));

    // 6b. Attempt dangerous file traversal (accessing sensitive credential paths)
    let credential_script = "/etc/shadow";
    let safe_args = vec!["--help".to_string()];
    let blocked_path = runner.run(credential_script, &safe_args, None).await;
    assert!(blocked_path.is_err(), "AgentShield must block access to /etc/shadow");
    let path_err = blocked_path.err().unwrap().to_string();
    assert!(path_err.contains("AgentShield blocked"));
}

// =========================================================================
// 7. End-to-End 7-Phase Pipeline Execution
// =========================================================================

#[tokio::test]
async fn test_e2e_7phase_pipeline_execution() {
    let tmp = tempdir().expect("Temp dir failed");
    let source_file = tmp.path().join("calculator.py");

    let source_code = r#"
"""
Basic arithmetic calculator library.
"""

def add(x: int, y: int) -> int:
    """Add two integers."""
    return x + y

def multiply(a: float, b: float) -> float:
    """Multiply two numbers."""
    return a * b
"#;
    fs::write(&source_file, source_code).expect("Write source file failed");

    let output_dir = tmp.path().join("out_harness");
    let skills_dir = tmp.path().join(".ecc").join("skills");
    fs::create_dir_all(&skills_dir).expect("Create skills dir failed");

    let options = HarnessPipelineOptions::new(&source_file)
        .with_name("calc-app")
        .with_output_dir(&output_dir)
        .with_run_tests(false);

    let pipeline = HarnessPipeline::new();
    let result = pipeline.execute(options).await.expect("Pipeline execution must succeed");

    // Verify 7 phases executed
    assert_eq!(result.phase_results.len(), 7);
    for phase in &result.phase_results {
        assert!(phase.success, "Phase '{}' should succeed: {}", phase.title, phase.message);
    }

    // Verify generated files on disk
    assert!(output_dir.join("calc_app_cli.py").is_file());
    assert!(output_dir.join("test_calc_app_cli.py").is_file());

    // Verify spec has extracted subcommands
    assert_eq!(result.spec.name, "calc-app");
    assert_eq!(result.spec.commands.len(), 2);
    assert!(result.spec.find_command("add").is_some());
    assert!(result.spec.find_command("multiply").is_some());
}
