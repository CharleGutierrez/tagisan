use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::json;
use tagisan::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
use tagisan::perl::{PerlExecutionResult, PerlRuntime};
use tagisan::python::{PythonExecutionResult, PythonRuntime};
use tagisan::tools::{PerlEvalTool, PerlRunTool, PythonEvalTool, PythonRunTool, ToolHandler, ToolRegistry};
use tagisan::types::ContentBlock;

// =========================================================================
// 1. Multithreaded Parallel Concurrency (24+ workers, 240+ iterations)
// =========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_brutal_multithreaded_parallel_concurrency() {
    let worker_count = 24;
    let iterations_per_worker = 10;
    let total_expected = worker_count * iterations_per_worker;

    let base_temp = std::env::temp_dir().join(format!(
        "tgs_stress_concurrency_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    tokio::fs::create_dir_all(&base_temp)
        .await
        .expect("Create base stress temp dir");

    let py_runtime = Arc::new(PythonRuntime::new().expect("Python runtime initialization"));
    let pl_runtime = Arc::new(PerlRuntime::new().expect("Perl runtime initialization"));

    let successful_evals = Arc::new(AtomicUsize::new(0));
    let successful_files = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut handles = Vec::with_capacity(worker_count);

    for worker_id in 0..worker_count {
        let py_rt = py_runtime.clone();
        let pl_rt = pl_runtime.clone();
        let temp_dir = base_temp.clone();
        let success_evals = successful_evals.clone();
        let success_files = successful_files.clone();
        let errors = error_count.clone();

        let handle = tokio::spawn(async move {
            for iter in 0..iterations_per_worker {
                // 1. Python Concurrent Eval
                let expected_py_ans = (worker_id * 1000 + iter) * 3;
                let py_code = format!("print(({worker_id} * 1000 + {iter}) * 3)");
                match py_rt.eval_code(&py_code, Some(15)).await {
                    Ok(res) => {
                        if res.exit_code == 0 && res.stdout.trim() == expected_py_ans.to_string() {
                            success_evals.fetch_add(1, Ordering::Relaxed);
                        } else {
                            eprintln!("Worker {worker_id} iter {iter} py eval mismatch: {:?}", res);
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        eprintln!("Worker {worker_id} iter {iter} py eval error: {e}");
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // 2. Perl Concurrent Eval
                let expected_pl_marker = format!("TGS_PL_W{worker_id}_I{iter}_OK");
                let pl_code = format!(
                    "my $w = {worker_id}; my $i = {iter}; print \"TGS_PL_W${{w}}_I${{i}}_OK\\n\";"
                );
                match pl_rt.eval_code(&pl_code, Some(15)).await {
                    Ok(res) => {
                        if res.exit_code == 0 && res.stdout.trim() == expected_pl_marker {
                            success_evals.fetch_add(1, Ordering::Relaxed);
                        } else {
                            eprintln!("Worker {worker_id} iter {iter} pl eval mismatch: {:?}", res);
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        eprintln!("Worker {worker_id} iter {iter} pl eval error: {e}");
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }

                // 3. Collision-Proof File Execution with unique Blake3 hashes
                let unique_hash = blake3::hash(
                    format!(
                        "worker_{worker_id}_iter_{iter}_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_nanos()
                    )
                    .as_bytes(),
                )
                .to_hex();

                let py_script_path = temp_dir.join(format!("script_{unique_hash}.py"));
                let pl_script_path = temp_dir.join(format!("script_{unique_hash}.pl"));

                // Write Python script
                let py_content = format!(
                    "import sys\na, b = int(sys.argv[1]), int(sys.argv[2])\nprint(f'SUM={{a + b}}_HASH_{unique_hash}')\n"
                );
                if let Err(e) = tokio::fs::write(&py_script_path, py_content).await {
                    eprintln!("Worker {worker_id} iter {iter} write py file error: {e}");
                    errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                // Write Perl script
                let pl_content = format!(
                    "my ($a, $b) = @ARGV;\nmy $sum = $a * $b;\nprint \"PROD=${{sum}}_HASH_{unique_hash}\\n\";\n"
                );
                if let Err(e) = tokio::fs::write(&pl_script_path, pl_content).await {
                    eprintln!("Worker {worker_id} iter {iter} write pl file error: {e}");
                    errors.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                // Execute Python script file
                let py_res = py_rt
                    .run_file(&py_script_path, &["10".to_string(), "25".to_string()], Some(15))
                    .await;
                // Execute Perl script file
                let pl_res = pl_rt
                    .run_file(&pl_script_path, &["6".to_string(), "7".to_string()], Some(15))
                    .await;

                // Cleanup immediately
                let _ = tokio::fs::remove_file(&py_script_path).await;
                let _ = tokio::fs::remove_file(&pl_script_path).await;

                // Verify results
                match (py_res, pl_res) {
                    (Ok(py_out), Ok(pl_out)) => {
                        let py_ok = py_out.exit_code == 0
                            && py_out.stdout.contains(&format!("SUM=35_HASH_{unique_hash}"));
                        let pl_ok = pl_out.exit_code == 0
                            && pl_out.stdout.contains(&format!("PROD=42_HASH_{unique_hash}"));
                        if py_ok && pl_ok {
                            success_files.fetch_add(1, Ordering::Relaxed);
                        } else {
                            eprintln!("Worker {worker_id} iter {iter} file output validation failed");
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    (Err(e1), _) => {
                        eprintln!("Worker {worker_id} iter {iter} py file run err: {e1}");
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                    (_, Err(e2)) => {
                        eprintln!("Worker {worker_id} iter {iter} pl file run err: {e2}");
                        errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Await all workers
    for h in handles {
        h.await.expect("Worker thread join");
    }

    let elapsed = start_time.elapsed();
    let evals_done = successful_evals.load(Ordering::SeqCst);
    let files_done = successful_files.load(Ordering::SeqCst);
    let errs = error_count.load(Ordering::SeqCst);

    println!(
        "Parallel Stress Concurrency: Completed {} evals (expected {}), {} files (expected {}) in {:.2}s. Errors: {}",
        evals_done,
        total_expected * 2,
        files_done,
        total_expected,
        elapsed.as_secs_f64(),
        errs
    );

    assert_eq!(errs, 0, "Zero concurrency errors allowed");
    assert_eq!(
        evals_done,
        total_expected * 2,
        "All Python and Perl evals must succeed"
    );
    assert_eq!(
        files_done, total_expected,
        "All Python and Perl concurrent file executions must succeed"
    );

    // Verify temp directory is completely clean (zero leftover temp files)
    let mut dir_entries = tokio::fs::read_dir(&base_temp)
        .await
        .expect("Read temp dir");
    let mut leftover_count = 0;
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        eprintln!("Lingering file found: {:?}", entry.path());
        leftover_count += 1;
    }
    assert_eq!(
        leftover_count, 0,
        "Zero lingering temporary files allowed after stress run"
    );

    // Clean up base directory
    let _ = tokio::fs::remove_dir_all(&base_temp).await;
}

// =========================================================================
// 2. Strict Timeout & Process Kill Enforcement
// =========================================================================

#[tokio::test]
async fn test_strict_timeout_and_process_kill_python() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    let infinite_py = "import time\nwhile True:\n    time.sleep(0.1)\n";
    let start = Instant::now();
    let res = runtime
        .eval(infinite_py, Duration::from_secs(1), None, None)
        .await;
    let elapsed = start.elapsed();

    assert!(
        res.is_err(),
        "Python infinite loop must return an error on timeout"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("timed out"),
        "Error message must indicate timeout: {err_msg}"
    );

    println!("Python timeout kill elapsed: {:?}", elapsed);
    assert!(
        elapsed >= Duration::from_millis(900),
        "Timeout must enforce at least the requested 1-second limit"
    );
    assert!(
        elapsed <= Duration::from_millis(3000),
        "Timeout kill must terminate within ~1-2 seconds, not hang"
    );

    // Verify runtime is immediately healthy for subsequent evaluation
    let recover_res = runtime
        .eval_code("print('PY_RECOVERED')", Some(5))
        .await
        .expect("Python recovery eval");
    assert_eq!(recover_res.exit_code, 0);
    assert!(recover_res.stdout.contains("PY_RECOVERED"));
}

#[tokio::test]
async fn test_strict_timeout_and_process_kill_perl() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    let infinite_pl = "while (1) { select(undef, undef, undef, 0.1); }";
    let start = Instant::now();
    let res = runtime
        .eval(infinite_pl, Duration::from_secs(1), None, None)
        .await;
    let elapsed = start.elapsed();

    assert!(
        res.is_err(),
        "Perl infinite loop must return an error on timeout"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("timed out"),
        "Error message must indicate timeout: {err_msg}"
    );

    println!("Perl timeout kill elapsed: {:?}", elapsed);
    assert!(
        elapsed >= Duration::from_millis(900),
        "Timeout must enforce at least the requested 1-second limit"
    );
    assert!(
        elapsed <= Duration::from_millis(3000),
        "Timeout kill must terminate within ~1-2 seconds, not hang"
    );

    // Verify runtime is immediately healthy for subsequent evaluation
    let recover_res = runtime
        .eval_code("print 'PL_RECOVERED';", Some(5))
        .await
        .expect("Perl recovery eval");
    assert_eq!(recover_res.exit_code, 0);
    assert!(recover_res.stdout.contains("PL_RECOVERED"));
}

// =========================================================================
// 3. Output Safety Buffer & Memory Bounds (5MB+ data truncation)
// =========================================================================

#[tokio::test]
async fn test_output_safety_buffer_python_large() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    // Output 5,000,000 bytes
    let code = "print('A' * 5_000_000)";
    let res: PythonExecutionResult = runtime
        .eval_code(code, Some(15))
        .await
        .expect("Python 5MB evaluation");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(
        res.stdout.contains("... [Output truncated: exceeded 1MB limit]"),
        "Output must indicate 1MB truncation"
    );
    assert!(
        res.stdout.len() <= PythonRuntime::MAX_OUTPUT_BYTES + 512,
        "Output string length must be bounded to <= 1MB + truncation notice (actual: {} bytes)",
        res.stdout.len()
    );
}

#[tokio::test]
async fn test_output_safety_buffer_perl_large() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    // Output 5,000,000 bytes
    let code = "print 'B' x 5_000_000;";
    let res: PerlExecutionResult = runtime
        .eval_code(code, Some(15))
        .await
        .expect("Perl 5MB evaluation");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(
        res.stdout.contains("... [Output truncated: exceeded 1MB limit]"),
        "Output must indicate 1MB truncation"
    );
    assert!(
        res.stdout.len() <= PerlRuntime::MAX_OUTPUT_BYTES + 512,
        "Output string length must be bounded to <= 1MB + truncation notice (actual: {} bytes)",
        res.stdout.len()
    );
}

// =========================================================================
// 4. Stream Separation & Exit Code Fidelity
// =========================================================================

#[tokio::test]
async fn test_stream_separation_and_exit_code_python() {
    let runtime = PythonRuntime::new().expect("Python runtime initialization");

    // 1. Strict stream isolation
    let stream_script = r#"
import sys
sys.stdout.write("PY_EXCLUSIVE_STDOUT\n")
sys.stderr.write("PY_EXCLUSIVE_STDERR\n")
"#;
    let res = runtime
        .eval_code(stream_script, Some(10))
        .await
        .expect("Stream test eval");

    assert_eq!(res.exit_code, 0);
    assert!(
        res.stdout.contains("PY_EXCLUSIVE_STDOUT"),
        "stdout must receive stdout data"
    );
    assert!(
        !res.stdout.contains("PY_EXCLUSIVE_STDERR"),
        "stdout must not bleed stderr data"
    );
    assert!(
        res.stderr.contains("PY_EXCLUSIVE_STDERR"),
        "stderr must receive stderr data"
    );
    assert!(
        !res.stderr.contains("PY_EXCLUSIVE_STDOUT"),
        "stderr must not bleed stdout data"
    );

    // 2. Non-zero exit code fidelity (77)
    let exit_script = "import sys\nsys.exit(77)\n";
    let exit_res = runtime
        .eval_code(exit_script, Some(10))
        .await
        .expect("Exit 77 execution");
    assert_eq!(exit_res.exit_code, 77);
    assert!(!exit_res.success);
    assert!(!exit_res.is_success());

    // 3. Syntax error diagnostics
    let syntax_error_script = "def invalid_syntax(\n    x = 10\n    return x + \n";
    let syntax_res = runtime
        .eval(syntax_error_script, Duration::from_secs(5), None, None)
        .await
        .expect("Syntax error process execution");
    assert_ne!(syntax_res.exit_code, 0);
    assert!(!syntax_res.success);
    assert!(
        syntax_res.stderr.contains("SyntaxError"),
        "stderr must contain actionable SyntaxError trace: {}",
        syntax_res.stderr
    );
}

#[tokio::test]
async fn test_stream_separation_and_exit_code_perl() {
    let runtime = PerlRuntime::new().expect("Perl runtime initialization");

    // 1. Strict stream isolation
    let stream_script = r#"
print STDOUT "PL_EXCLUSIVE_STDOUT\n";
print STDERR "PL_EXCLUSIVE_STDERR\n";
"#;
    let res = runtime
        .eval_code(stream_script, Some(10))
        .await
        .expect("Stream test eval");

    assert_eq!(res.exit_code, 0);
    assert!(
        res.stdout.contains("PL_EXCLUSIVE_STDOUT"),
        "stdout must receive stdout data"
    );
    assert!(
        !res.stdout.contains("PL_EXCLUSIVE_STDERR"),
        "stdout must not bleed stderr data"
    );
    assert!(
        res.stderr.contains("PL_EXCLUSIVE_STDERR"),
        "stderr must receive stderr data"
    );
    assert!(
        !res.stderr.contains("PL_EXCLUSIVE_STDOUT"),
        "stderr must not bleed stdout data"
    );

    // 2. Non-zero exit code fidelity (88)
    let exit_script = "exit 88;";
    let exit_res = runtime
        .eval_code(exit_script, Some(10))
        .await
        .expect("Exit 88 execution");
    assert_eq!(exit_res.exit_code, 88);
    assert!(!exit_res.success);
    assert!(!exit_res.is_success());

    // 3. Syntax error diagnostics
    let syntax_error_script = "sub broken { my $a = ;;;; }}}";
    let syntax_res = runtime
        .eval(syntax_error_script, Duration::from_secs(5), None, None)
        .await
        .expect("Syntax error process execution");
    assert_ne!(syntax_res.exit_code, 0);
    assert!(!syntax_res.success);
    assert!(
        syntax_res.stderr.contains("syntax error") || syntax_res.stderr.contains("Compilation failed"),
        "stderr must contain actionable Perl syntax error trace: {}",
        syntax_res.stderr
    );
}

// =========================================================================
// 5. Adversarial AgentShield Security & Obfuscation Resilience
// =========================================================================

#[tokio::test]
async fn test_agentshield_direct_threats_blocked() {
    // 1. Python direct threats
    let threats_py = vec![
        ("os.system", json!({ "code": "import os\nos.system('whoami')" })),
        ("subprocess.Popen", json!({ "code": "import subprocess\nsubprocess.Popen(['cmd.exe', '/c', 'dir'])" })),
        ("shutil.rmtree", json!({ "code": "import shutil\nshutil.rmtree('/etc')" })),
        ("pty.spawn", json!({ "code": "import pty\npty.spawn('/bin/bash')" })),
    ];

    for (name, payload) in threats_py {
        let verdict = AgentShieldScanner::scan_tool_call("python_eval", &payload);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }),
            "AgentShield must block direct Python threat '{name}': got {:?}",
            verdict
        );
    }

    // 2. Perl direct threats
    let threats_pl = vec![
        ("system(", json!({ "code": "system('whoami');" })),
        ("exec(", json!({ "code": "exec('/bin/sh');" })),
        ("piped open", json!({ "code": "open(F, '| /bin/sh');" })),
        ("root unlink", json!({ "code": "unlink '/';" })),
    ];

    for (name, payload) in threats_pl {
        let verdict = AgentShieldScanner::scan_tool_call("perl_eval", &payload);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }),
            "AgentShield must block direct Perl threat '{name}': got {:?}",
            verdict
        );
    }
}

#[tokio::test]
async fn test_agentshield_adversarial_evasion_and_obfuscation_blocked() {
    // 1. Python evasion techniques
    let evasions_py = vec![
        (
            "getattr(__import__('os'), 'system')",
            json!({ "code": "getattr(__import__('os'), 'system')('id')" }),
        ),
        (
            "eval(base64.b64decode(...))",
            json!({ "code": "import base64\neval(base64.b64decode('b3Muc3lzdGVtKCdkaXInKQ=='))" }),
        ),
        (
            "__builtins__.__dict__['eval']",
            json!({ "code": "__builtins__.__dict__['eval'](\"print('sandbox_escape')\")" }),
        ),
        (
            "Python hex escape os.system",
            json!({ "code": "eval(\"\\x6f\\x73\\x2e\\x73\\x79\\x73\\x74\\x65\\x6d('whoami')\")" }),
        ),
        (
            "Python hex escape getattr",
            json!({ "code": "getattr(__import__('\\x6f\\x73'), '\\x73\\x79\\x73\\x74\\x65\\x6d')('id')" }),
        ),
    ];

    for (label, payload) in evasions_py {
        let verdict = AgentShieldScanner::scan_tool_call("python_eval", &payload);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }),
            "AgentShield must block Python evasion technique '{label}': got {:?}",
            verdict
        );
    }

    // 2. Perl evasion techniques
    let evasions_pl = vec![
        (
            "Perl eval(pack('H*', ...))",
            json!({ "code": "eval(pack('H*', '73797374656d282777686f616d692729'));" }),
        ),
        (
            "Perl eval pack(\"H*\", ...)",
            json!({ "code": "eval pack(\"H*\", \"73797374656d282777686f616d692729\");" }),
        ),
        (
            "Perl eval with hex escapes",
            json!({ "code": "eval(\"\\x73\\x79\\x73\\x74\\x65\\x6d('whoami');\");" }),
        ),
        (
            "Perl direct hex escape",
            json!({ "code": "\\x73\\x79\\x73\\x74\\x65\\x6d('whoami');" }),
        ),
    ];

    for (label, payload) in evasions_pl {
        let verdict = AgentShieldScanner::scan_tool_call("perl_eval", &payload);
        assert!(
            matches!(verdict, AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }),
            "AgentShield must block Perl evasion technique '{label}': got {:?}",
            verdict
        );
    }
}

#[tokio::test]
async fn test_agentshield_no_false_positives_on_benign_code() {
    // 1. Benign Python payloads
    let benign_py = vec![
        (
            "Math calculation",
            json!({ "code": "import math\nval = math.isqrt(256) * 10 + math.factorial(5)\nprint(f'VAL={val}')" }),
        ),
        (
            "JSON manipulation",
            json!({ "code": "import json\nd = {'engine': 'Tagisan', 'version': 1.0, 'valid': True}\nprint(json.dumps(d))" }),
        ),
        (
            "String formatting",
            json!({ "code": "name = 'AgentShield'\nmetric = 99.99\nprint(f'{name} accuracy: {metric:.2f}%')" }),
        ),
        (
            "Regex processing",
            json!({ "code": "import re\nmatches = re.findall(r'\\b[A-Z]\\w+', 'The Quick Brown Fox')\nprint(matches)" }),
        ),
    ];

    for (label, payload) in benign_py {
        let verdict = AgentShieldScanner::scan_tool_call("python_eval", &payload);
        assert_eq!(
            verdict,
            AgentShieldVerdict::Allow,
            "AgentShield must ALLOW benign Python payload: {label}"
        );
    }

    // 2. Benign Perl payloads
    let benign_pl = vec![
        (
            "Perl Math calculation",
            json!({ "code": "my $res = 42 * 100 + sqrt(144); print \"RES=$res\\n\";" }),
        ),
        (
            "Perl JSON/string representation",
            json!({ "code": "my $json = '{\"status\": \"healthy\", \"metrics\": [1, 2, 3]}'; print $json;" }),
        ),
        (
            "Perl Regex with alternation pipe",
            json!({ "code": "my $text = \"apple and banana and orange\"; $text =~ s/apple|orange/fruit/g; print $text;" }),
        ),
        (
            "Perl Array split and join",
            json!({ "code": "my @parts = split(/,/, 'tagisan,rust,bun,python,perl'); print join(' -> ', @parts);" }),
        ),
    ];

    for (label, payload) in benign_pl {
        let verdict = AgentShieldScanner::scan_tool_call("perl_eval", &payload);
        assert_eq!(
            verdict,
            AgentShieldVerdict::Allow,
            "AgentShield must ALLOW benign Perl payload: {label}"
        );
    }
}

// =========================================================================
// 6. Tool Handler Fuzzing Resilience
// =========================================================================

#[tokio::test]
async fn test_tool_handler_fuzzing_resilience() {
    let registry = ToolRegistry::with_builtins();

    let tools_to_fuzz = ["python_eval", "python_run", "perl_eval", "perl_run"];

    for tool_name in tools_to_fuzz {
        // 1. Missing parameters: empty object
        let r1 = registry.execute_call("fuzz_call_1", tool_name, &json!({})).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r1 {
            assert!(is_error, "{tool_name} with empty object must return error");
            assert!(content.contains("Missing required parameter"));
        } else {
            panic!("Expected ContentBlock::ToolResult");
        }

        // 2. Malformed JSON: unexpected types
        let r2 = registry.execute_call("fuzz_call_2", tool_name, &json!({ "code": 12345, "file_path": 12345 })).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r2 {
            assert!(is_error, "{tool_name} with wrong type must return error");
            assert!(content.contains("Missing required parameter"));
        } else {
            panic!("Expected ContentBlock::ToolResult");
        }

        // 3. Null payload
        let r3 = registry.execute_call("fuzz_call_3", tool_name, &serde_json::Value::Null).await;
        if let ContentBlock::ToolResult { is_error, .. } = r3 {
            assert!(is_error, "{tool_name} with null payload must return error");
        } else {
            panic!("Expected ContentBlock::ToolResult");
        }
    }

    // 4. Empty code parameter for eval tools
    for eval_tool in ["python_eval", "perl_eval"] {
        let r_empty = registry.execute_call("fuzz_eval_empty", eval_tool, &json!({ "code": "" })).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r_empty {
            assert!(is_error, "{eval_tool} with empty code must return error");
            assert!(content.contains("cannot be empty"));
        }

        let r_spaces = registry.execute_call("fuzz_eval_spaces", eval_tool, &json!({ "code": "   \n\t  " })).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r_spaces {
            assert!(is_error, "{eval_tool} with whitespace code must return error");
            assert!(content.contains("cannot be empty"));
        }
    }

    // 5. Non-existent script files for run tools
    for run_tool in ["python_run", "perl_run"] {
        let bogus_path = "C:\\nonexistent_bogus_dir_9999\\definitely_not_a_real_script.ext";
        let r_missing = registry.execute_call("fuzz_run_missing", run_tool, &json!({ "file_path": bogus_path })).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r_missing {
            assert!(is_error, "{run_tool} with non-existent file must return error");
            assert!(content.contains("not found"));
        }

        let r_empty_path = registry.execute_call("fuzz_run_empty_path", run_tool, &json!({ "file_path": "" })).await;
        if let ContentBlock::ToolResult { is_error, content, .. } = r_empty_path {
            assert!(is_error, "{run_tool} with empty path must return error");
            assert!(content.contains("cannot be empty"));
        }
    }

    // 6. Direct ToolHandler execute fuzzing (without registry wrapper)
    let py_eval = PythonEvalTool::new();
    assert!(py_eval.execute(json!({})).await.is_err());
    assert!(py_eval.execute(json!({ "code": "" })).await.is_err());

    let py_run = PythonRunTool::new();
    assert!(py_run.execute(json!({})).await.is_err());
    assert!(py_run.execute(json!({ "file_path": "C:\\nonexistent\\foo.py" })).await.is_err());

    let pl_eval = PerlEvalTool::new();
    assert!(pl_eval.execute(json!({})).await.is_err());
    assert!(pl_eval.execute(json!({ "code": "" })).await.is_err());

    let pl_run = PerlRunTool::new();
    assert!(pl_run.execute(json!({})).await.is_err());
    assert!(pl_run.execute(json!({ "file_path": "C:\\nonexistent\\foo.pl" })).await.is_err());
}
