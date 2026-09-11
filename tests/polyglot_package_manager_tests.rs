//! Brutal Integration & Security Test Suite for Tagisan Polyglot Package Management Engine
//!
//! Covers:
//! 1. Perl 5 Package Management (cpanm / local::lib / pure-Perl sandboxing).
//! 2. Python 3 Package Management (uv / pip / binary wheels sandboxing).
//! 3. Bun TS/JS Package Management (bun add / ignore-scripts sandboxing).
//! 4. AgentShield Invariant Defense:
//!    - Zero catastrophic deletions (rm -rf /, raw device writes) across all managers.
//!    - Zero credential theft (.env, id_rsa, /etc/shadow) across all managers.
//!    - Native C compilation gating: blocked by default; enabled only with explicit `allow_native=true`.
//!    - Persistent security: even with `allow_native=true`, credential access and root deletion remain strictly blocked.
//! 5. Autonomous JIT Dependency Resolution for Perl, Python, and TypeScript.
//! 6. ToolRegistry integration and CLI Action parity.

use serde_json::json;
use tagisan::bun::runtime::BunRuntime;
use tagisan::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
use tagisan::perl::runtime::PerlRuntime;
use tagisan::python::runtime::PythonRuntime;
use tagisan::tools::bun::{extract_missing_package, BunAutoResolveTool, BunInstallTool};
use tagisan::tools::perl::{
    extract_missing_perl_module, PerlAutoResolveTool, PerlInstallTool,
};
use tagisan::tools::python::{
    extract_missing_python_package, PythonAutoResolveTool, PythonInstallTool,
};
use tagisan::tools::{ToolHandler, ToolRegistry};

// =========================================================================
// 1. Perl 5 CPAN & local::lib Engine Tests
// =========================================================================

#[tokio::test]
async fn test_perl_runtime_discovery_and_module_check() {
    let runtime = PerlRuntime::new().expect("Perl 5 runtime should be discovered");
    assert!(runtime.is_available(), "Perl runtime must be available");

    let ver = runtime.version().await.expect("Perl version should execute");
    assert!(ver.contains("perl") || ver.contains("5."), "Version string must indicate Perl 5");

    // Core module 'strict' must be available
    let has_strict = runtime.check_module("strict").await.expect("check_module must succeed");
    assert!(has_strict, "Core module 'strict' must be present in standard Perl 5");

    // Core module 'warnings' must be available
    let has_warnings = runtime.check_module("warnings").await.expect("check_module must succeed");
    assert!(has_warnings, "Core module 'warnings' must be present in standard Perl 5");

    // Non-existent module must return false
    let has_fake = runtime.check_module("TagisanFakeModuleNonExistent999").await.expect("check_module must succeed");
    assert!(!has_fake, "Non-existent module must report false");
}

#[test]
fn test_perl_missing_module_error_extraction() {
    let stderr1 = "Can't locate Path/Tiny.pm in @INC (you may need to install the Path::Tiny module) (@INC contains: /usr/lib/perl5 ...)";
    assert_eq!(extract_missing_perl_module(stderr1), Some("Path::Tiny".to_string()));

    let stderr2 = "Can't locate JSON/MaybeXS.pm in @INC\nBEGIN failed--compilation aborted at script.pl line 2.\n";
    assert_eq!(extract_missing_perl_module(stderr2), Some("JSON::MaybeXS".to_string()));

    let stderr3 = "Can't locate Foo/Bar/Baz.pm in @INC at test.pl line 1.\n";
    assert_eq!(extract_missing_perl_module(stderr3), Some("Foo::Bar::Baz".to_string()));

    let normal_err = "syntax error at test.pl line 4, near 'foo'";
    assert_eq!(extract_missing_perl_module(normal_err), None);
}

#[tokio::test]
async fn test_perl_install_tool_metadata_and_validation() {
    let tool = PerlInstallTool::new();
    assert_eq!(tool.name(), "perl_install");
    assert!(tool.description().contains("CPAN"));

    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["modules"].is_object());
    assert!(schema["properties"]["allow_native"].is_object());

    // Executing with empty modules array should return an error
    let res = tool.execute(json!({ "modules": [] })).await;
    assert!(res.is_err(), "Empty modules list must return an error");
}

#[tokio::test]
async fn test_perl_auto_resolve_tool_metadata_and_parsing() {
    let tool = PerlAutoResolveTool::new();
    assert_eq!(tool.name(), "perl_auto_resolve");
    assert!(tool.description().contains("CPAN"));

    // Missing 'stderr' parameter should error out
    let res = tool.execute(json!({})).await;
    assert!(res.is_err(), "Missing 'stderr' must return error");

    // Unrelated error with no missing module
    let res2 = tool.execute(json!({ "stderr": "syntax error at foo.pl line 1" })).await;
    assert!(res2.is_err(), "Non-missing module error must be reported");
    assert!(res2.unwrap_err().to_string().contains("Could not detect any missing CPAN module"));
}

// =========================================================================
// 2. Python 3 uv / pip Engine Tests
// =========================================================================

#[tokio::test]
async fn test_python_runtime_discovery_and_uv_detection() {
    let runtime = PythonRuntime::new().expect("Python 3 runtime should be discovered");
    assert!(runtime.is_available(), "Python runtime must be available");

    let ver = runtime.version().await.expect("Python version should execute");
    assert!(ver.contains("Python 3"), "Version string must indicate Python 3");

    // uv toolchain discovery
    let uv_opt = PythonRuntime::find_uv();
    println!("Discovered uv path: {:?}", uv_opt);
}

#[test]
fn test_python_missing_package_error_extraction() {
    let stderr1 = "Traceback (most recent call last):\n  File \"app.py\", line 1, in <module>\nModuleNotFoundError: No module named 'requests'";
    assert_eq!(extract_missing_python_package(stderr1), Some("requests".to_string()));

    let stderr2 = "ModuleNotFoundError: No module named 'pydantic.v1'";
    assert_eq!(extract_missing_python_package(stderr2), Some("pydantic".to_string()));

    let stderr3 = "ModuleNotFoundError: No module named \"fastapi\"";
    assert_eq!(extract_missing_python_package(stderr3), Some("fastapi".to_string()));

    let normal_err = "ValueError: invalid literal for int() with base 10: 'abc'";
    assert_eq!(extract_missing_python_package(normal_err), None);
}

#[tokio::test]
async fn test_python_install_tool_metadata_and_validation() {
    let tool = PythonInstallTool::new();
    assert_eq!(tool.name(), "python_install");
    assert!(tool.description().contains("Python"));

    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["packages"].is_object());
    assert!(schema["properties"]["allow_native"].is_object());

    // Executing with empty packages list should return an error
    let res = tool.execute(json!({ "packages": [] })).await;
    assert!(res.is_err(), "Empty packages list must return error");
}

#[tokio::test]
async fn test_python_auto_resolve_tool_metadata_and_parsing() {
    let tool = PythonAutoResolveTool::new();
    assert_eq!(tool.name(), "python_auto_resolve");
    assert!(tool.description().contains("Python"));

    // Missing 'stderr' parameter should error out
    let res = tool.execute(json!({})).await;
    assert!(res.is_err(), "Missing 'stderr' must return error");

    // Unrelated error with no missing package
    let res2 = tool.execute(json!({ "stderr": "ZeroDivisionError: division by zero" })).await;
    assert!(res2.is_err(), "Non-missing module error must be reported");
    assert!(res2.unwrap_err().to_string().contains("Could not detect any missing Python package"));
}

// =========================================================================
// 3. Bun TS/JS Package Management Engine Tests
// =========================================================================

#[tokio::test]
async fn test_bun_runtime_discovery() {
    let runtime = BunRuntime::new().expect("Bun runtime should be discovered");
    let ver = runtime.version().await.expect("Bun version should execute");
    assert!(!ver.trim().is_empty(), "Bun version string must not be empty");
}

#[test]
fn test_bun_missing_package_extraction() {
    let stderr1 = "error: Cannot find package 'lodash' from '/workspace/index.ts'";
    assert_eq!(extract_missing_package(stderr1), Some("lodash".to_string()));

    let stderr2 = "error: Cannot find module '@types/node'";
    assert_eq!(extract_missing_package(stderr2), Some("@types/node".to_string()));

    let normal_err = "ReferenceError: foo is not defined";
    assert_eq!(extract_missing_package(normal_err), None);
}

#[tokio::test]
async fn test_bun_install_tool_metadata() {
    let tool = BunInstallTool::new();
    assert_eq!(tool.name(), "bun_install");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["allow_native"].is_object());
    assert!(schema["properties"]["packages"].is_object());
    assert!(schema["properties"]["dev"].is_object());
}

#[tokio::test]
async fn test_bun_auto_resolve_tool_metadata() {
    let tool = BunAutoResolveTool::new();
    assert_eq!(tool.name(), "bun_auto_resolve");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["allow_native"].is_object());
    assert!(schema["properties"]["code"].is_object());
    assert!(schema["properties"]["script_path"].is_object());
}

// =========================================================================
// 4. AgentShield Security Invariants (Brutal Defense Tests)
// =========================================================================

#[test]
fn test_agentshield_blocks_catastrophic_deletion_across_package_managers() {
    let destructive_payloads = [
        ("perl_install", json!({ "modules": ["Path::Tiny; rm -rf /"] })),
        ("perl_install", json!({ "modules": ["Path::Tiny", "rm -rf /"] })),
        ("python_install", json!({ "packages": ["requests; rm -rf /"] })),
        ("python_install", json!({ "packages": ["requests", "mkfs.ext4 /dev/sda"] })),
        ("python_install", json!({ "packages": ["flask | dd if=/dev/zero of=/dev/sda"] })),
        ("bun_install", json!({ "packages": ["express; rm -rf ~"] })),
        ("bun_install", json!({ "packages": ["lodash", "> /dev/nvme0n1"] })),
    ];

    for (tool_name, args) in destructive_payloads {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }),
            "AgentShield must block catastrophic deletion on '{tool_name}' with args: {:?}",
            args
        );
    }
}

#[test]
fn test_agentshield_blocks_credential_theft_across_package_managers() {
    let exfil_payloads = [
        ("perl_install", json!({ "modules": ["../../.env"] })),
        ("perl_install", json!({ "modules": ["/etc/shadow"] })),
        ("perl_install", json!({ "modules": ["~/.ssh/id_rsa"] })),
        ("python_install", json!({ "packages": [".env"] })),
        ("python_install", json!({ "packages": ["/etc/shadow"] })),
        ("python_install", json!({ "packages": ["id_ed25519"] })),
        ("bun_install", json!({ "packages": ["../../.env"] })),
        ("bun_install", json!({ "packages": ["/etc/passwd"] })),
        ("bun_install", json!({ "packages": [".ssh/id_rsa"] })),
    ];

    for (tool_name, args) in exfil_payloads {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield must block credential exfiltration on '{tool_name}' with args: {:?}",
            args
        );
    }
}

#[test]
fn test_agentshield_blocks_credential_leaks_in_auto_resolve_stderr() {
    let auto_resolve_threats = [
        ("perl_auto_resolve", json!({ "stderr": "Can't locate /etc/shadow.pm in @INC" })),
        ("perl_auto_resolve", json!({ "stderr": "Can't locate Foo.pm: leaked secret in .env" })),
        ("python_auto_resolve", json!({ "stderr": "ModuleNotFoundError: No module named '/etc/shadow'" })),
        ("python_auto_resolve", json!({ "stderr": "Error reading ~/.ssh/id_rsa: ModuleNotFoundError" })),
    ];

    for (tool_name, args) in auto_resolve_threats {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield must block credential leak in error stream for '{tool_name}': {:?}",
            args
        );
    }
}

#[test]
fn test_agentshield_native_compilation_gating() {
    // 1. When allow_native is false (the default), native compiler tools are BLOCKED
    let native_threats_default = [
        ("perl_install", json!({ "modules": ["gcc"], "allow_native": false })),
        ("perl_install", json!({ "modules": ["make"], "allow_native": false })),
        ("python_install", json!({ "packages": ["clang"], "allow_native": false })),
        ("python_install", json!({ "packages": ["--build-from-source"], "allow_native": false })),
        ("bun_install", json!({ "packages": ["node-gyp"], "allow_native": false })),
    ];

    for (tool_name, args) in native_threats_default {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::High, .. }),
            "AgentShield must block native compilation tool without allow_native=true for '{tool_name}': {:?}",
            args
        );
    }

    // 2. When allow_native is true, native compilation indicators are PERMITTED
    let allowed_native = [
        ("perl_install", json!({ "modules": ["make"], "allow_native": true })),
        ("python_install", json!({ "packages": ["clang"], "allow_native": true })),
        ("bun_install", json!({ "packages": ["node-gyp"], "allow_native": true })),
    ];

    for (tool_name, args) in allowed_native {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Allow),
            "AgentShield must allow native compilation tool when allow_native=true for '{tool_name}': {:?}",
            args
        );
    }

    // 3. CRITICAL INVARIANT: Even when allow_native is true, CATASTROPHIC DELETION AND CREDENTIAL THEFT ARE STILL BLOCKED!
    let bypass_attempts = [
        ("perl_install", json!({ "modules": ["gcc; rm -rf /"], "allow_native": true })),
        ("perl_install", json!({ "modules": ["/etc/shadow"], "allow_native": true })),
        ("python_install", json!({ "packages": ["clang", ".env"], "allow_native": true })),
        ("python_install", json!({ "packages": ["make; rm -rf ~"], "allow_native": true })),
        ("bun_install", json!({ "packages": ["node-gyp", "../../.env"], "allow_native": true })),
        ("bun_install", json!({ "packages": ["node-gyp; mkfs /dev/sda"], "allow_native": true })),
    ];

    for (tool_name, args) in bypass_attempts {
        let verdict = AgentShieldScanner::scan_tool_call(tool_name, &args);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { .. }),
            "AgentShield must NEVER allow deletion or credential theft even with allow_native=true! '{tool_name}': {:?}",
            args
        );
    }
}

// =========================================================================
// 5. ToolRegistry Integration Tests
// =========================================================================

#[test]
fn test_tool_registry_contains_polyglot_package_managers() {
    let registry = ToolRegistry::with_builtins();

    let expected_tools = [
        "bun_install",
        "bun_auto_resolve",
        "python_install",
        "python_auto_resolve",
        "perl_install",
        "perl_auto_resolve",
    ];

    for tool_name in expected_tools {
        let tool = registry.get(tool_name);
        assert!(tool.is_some(), "ToolRegistry must contain tool '{tool_name}'");
        let t = tool.unwrap();
        assert_eq!(t.name(), tool_name);
        assert!(!t.description().is_empty());
        assert_eq!(t.parameters_schema()["type"], "object");
    }
}

#[test]
fn test_tool_registry_in_dir_contains_polyglot_package_managers() {
    let registry = ToolRegistry::with_builtins_in_dir("/tmp/sandbox");

    let expected_tools = [
        "bun_install",
        "bun_auto_resolve",
        "python_install",
        "python_auto_resolve",
        "perl_install",
        "perl_auto_resolve",
    ];

    for tool_name in expected_tools {
        let tool = registry.get(tool_name);
        assert!(tool.is_some(), "ToolRegistry (in dir) must contain tool '{tool_name}'");
    }
}
