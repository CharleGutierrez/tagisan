use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tagisan::bun::{BunExecutionResult, BunRuntime};
use tagisan::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict, ThreatLevel};
use tagisan::tools::bun::BunEvalTool;
use tagisan::tools::{ToolHandler, ToolRegistry};
use tagisan::{AutonomousAgent, EngineContext, InteractiveRepl, ReplCommand};

#[tokio::test]
async fn test_bun_discovery_and_version() {
    let bun_path = BunRuntime::find_bun();
    assert!(
        bun_path.is_some(),
        "Bun binary must be discovered automatically on the system"
    );

    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");
    assert!(runtime.is_available(), "Bun runtime must be available");
    assert!(
        runtime.bun_path().is_file(),
        "Bun path must point to an actual file"
    );

    let version = runtime.version().await.expect("bun --version should succeed");
    println!("Discovered Bun Version: {version}");
    assert!(
        version.starts_with("1."),
        "Expected Bun version 1.x, got '{version}'"
    );
}

#[tokio::test]
async fn test_bun_eval_typescript_syntax_and_generics() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let code = r#"
        interface Matrix<T> {
            data: T[][];
            rows: number;
            cols: number;
        }

        function createIdentity(size: number): Matrix<number> {
            const data: number[][] = [];
            for (let i = 0; i < size; i++) {
                data[i] = [];
                for (let j = 0; j < size; j++) {
                    data[i][j] = i === j ? 1 : 0;
                }
            }
            return { data, rows: size, cols: size };
        }

        const m = createIdentity(3);
        const trace = m.data.reduce((acc, row, idx) => acc + row[idx], 0);
        console.log(`Identity Matrix 3x3 Trace: ${trace}`);
    "#;

    let res: BunExecutionResult = runtime
        .eval(code, Duration::from_secs(10), None, None)
        .await
        .expect("Evaluation should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(res.stdout.contains("Identity Matrix 3x3 Trace: 3"));
    assert!(res.duration_ms < 5000);
}

#[tokio::test]
async fn test_bun_eval_pure_expression() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let res = runtime
        .eval("Math.pow(2, 10)", Duration::from_secs(10), None, None)
        .await
        .expect("Expression eval should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert_eq!(res.stdout.trim(), "1024");
}

#[tokio::test]
async fn test_bun_eval_top_level_await_esm() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let code = r#"
        import { randomUUID } from "node:crypto";
        
        async function fetchValue(x: number): Promise<number> {
            return new Promise((resolve) => setTimeout(() => resolve(x * 10), 5));
        }

        const [v1, v2] = await Promise.all([fetchValue(3), fetchValue(7)]);
        const uuid = randomUUID();
        console.log(`Total: ${v1 + v2}, UUID valid: ${uuid.length === 36}`);
    "#;

    let res = runtime
        .eval(code, Duration::from_secs(10), None, None)
        .await
        .expect("Async ESM evaluation should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(res.stdout.contains("Total: 100, UUID valid: true"));
}

#[tokio::test]
async fn test_bun_run_script_with_args_and_env() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("test_script_{}.ts", std::process::id()));

    let script_content = r#"
        const args = process.argv.slice(2);
        const tagisanEnv = process.env["TAGISAN_TEST_KEY"] || "not_set";
        console.log(`ARGS: [${args.join(", ")}] | ENV: ${tagisanEnv}`);
    "#;

    tokio::fs::write(&script_path, script_content)
        .await
        .expect("Writing temp script must succeed");

    let mut env_map = HashMap::new();
    env_map.insert("TAGISAN_TEST_KEY".to_string(), "BUN_RUST_PROD".to_string());

    let args = vec!["arg1".to_string(), "arg2".to_string(), "alpha_beta".to_string()];

    let res = runtime
        .run_script(&script_path, &args, Duration::from_secs(10), Some(env_map), None)
        .await
        .expect("Running script should succeed");

    let _ = tokio::fs::remove_file(&script_path).await;

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    assert!(res.stdout.contains("ARGS: [arg1, arg2, alpha_beta]"));
    assert!(res.stdout.contains("ENV: BUN_RUST_PROD"));
}

#[tokio::test]
async fn test_bun_test_runner_real_assertions() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("real_bun_{}.test.ts", std::process::id()));

    let test_content = r#"
        import { describe, test, expect } from "bun:test";

        describe("Real Bun Test Suite", () => {
            test("numerical precision", () => {
                expect(1 + 1).toBe(2);
                expect(Math.sqrt(16)).toBe(4);
            });

            test("string manipulation and array operations", () => {
                const arr = ["tagisan", "bun", "rust"];
                expect(arr.map(s => s.toUpperCase())).toEqual(["TAGISAN", "BUN", "RUST"]);
            });
        });
    "#;

    tokio::fs::write(&test_file, test_content)
        .await
        .expect("Writing test file must succeed");

    let res = runtime
        .test(&test_file, &[], Duration::from_secs(15), None)
        .await
        .expect("bun test execution should succeed");

    let _ = tokio::fs::remove_file(&test_file).await;

    assert_eq!(res.exit_code, 0);
    assert!(res.success);
    let output = res.combined_output();
    assert!(output.contains("pass") || output.contains("✓"));
}

#[tokio::test]
async fn test_bun_build_bundle_generation() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let base_dir = std::env::temp_dir().join(format!("bun_build_test_{}", std::process::id()));
    let out_dir = base_dir.join("dist");
    tokio::fs::create_dir_all(&base_dir)
        .await
        .expect("Create temp dir");

    let entry_file = base_dir.join("entry.ts");
    let helper_file = base_dir.join("helper.ts");

    let helper_code = r#"
        export function computeHash(val: number): number {
            return (val * 2654435761) >>> 0;
        }
    "#;

    let entry_code = r#"
        import { computeHash } from "./helper";
        const h = computeHash(42);
        console.log(`Computed Hash: ${h}`);
    "#;

    tokio::fs::write(&helper_file, helper_code)
        .await
        .expect("Write helper");
    tokio::fs::write(&entry_file, entry_code)
        .await
        .expect("Write entry");

    let res = runtime
        .build(&entry_file, &out_dir, true, "bun", Duration::from_secs(15), None)
        .await
        .expect("Bun build should succeed");

    assert_eq!(res.exit_code, 0);
    assert!(res.success);

    let bundle_js = out_dir.join("entry.js");
    assert!(bundle_js.is_file(), "Bundle entry.js should exist");

    // Execute generated bundle with Bun to verify output
    let run_res = runtime
        .run_script(&bundle_js, &[], Duration::from_secs(10), None, None)
        .await
        .expect("Running bundled script should succeed");

    assert_eq!(run_res.exit_code, 0);
    assert!(run_res.stdout.contains("Computed Hash:"));

    let _ = tokio::fs::remove_dir_all(&base_dir).await;
}

#[tokio::test]
async fn test_bun_timeout_and_sigkill_cleanup() {
    let runtime = BunRuntime::new().expect("BunRuntime::new should succeed");

    let infinite_loop_code = "while(true) { /* tight infinite loop */ }";

    // 1-second timeout
    let timeout_duration = Duration::from_secs(1);
    let start = std::time::Instant::now();
    let res = runtime.eval(infinite_loop_code, timeout_duration, None, None).await;

    let elapsed = start.elapsed();
    assert!(
        res.is_err(),
        "Infinite loop must return an error due to timeout"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("timed out"),
        "Error message should mention timeout: {err_msg}"
    );
    // Timeout should take approximately ~1s, definitely not > 3s
    assert!(
        elapsed < Duration::from_secs(3),
        "Timeout should abort promptly within bounds"
    );
}

#[tokio::test]
async fn test_agentshield_blocks_malicious_bun_scripts() {
    // 1. Fork bomb patterns
    let fork_bomb = ":(){ :|:& };:";
    let fb_verdict = AgentShieldScanner::scan_code(fork_bomb);
    assert!(matches!(fb_verdict, AgentShieldVerdict::Block { .. }));

    let js_fork_bomb = "while(true) { child_process.fork(); }";
    let js_fb_verdict = AgentShieldScanner::scan_code(js_fork_bomb);
    assert!(matches!(
        js_fb_verdict,
        AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }
    ));

    // 2. Sensitive credential & system file access
    let shadow_read = r#"
        import fs from "node:fs";
        const content = fs.readFileSync("/etc/shadow", "utf-8");
        console.log(content);
    "#;
    let shadow_verdict = AgentShieldScanner::scan_code(shadow_read);
    assert!(matches!(
        shadow_verdict,
        AgentShieldVerdict::Block { threat_level: ThreatLevel::Critical, .. }
    ));

    let bun_shadow = "const file = Bun.file('/etc/shadow');";
    let bun_shadow_verdict = AgentShieldScanner::scan_code(bun_shadow);
    assert!(matches!(bun_shadow_verdict, AgentShieldVerdict::Block { .. }));

    let ssh_key_read = "const key = Bun.file('~/.ssh/id_rsa');";
    let ssh_verdict = AgentShieldScanner::scan_code(ssh_key_read);
    assert!(matches!(ssh_verdict, AgentShieldVerdict::Block { .. }));

    // 3. Destructive operations
    let rm_root = "Bun.spawn(['rm', '-rf', '/']);";
    let rm_verdict = AgentShieldScanner::scan_code(rm_root);
    assert!(matches!(rm_verdict, AgentShieldVerdict::Block { .. }));

    // 4. Safe legitimate code allowed
    let safe_ts = "const add = (a: number, b: number): number => a + b; console.log(add(2, 3));";
    let safe_verdict = AgentShieldScanner::scan_code(safe_ts);
    assert_eq!(safe_verdict, AgentShieldVerdict::Allow);

    // 5. BunEvalTool should directly reject blocked code via execute()
    let eval_tool = BunEvalTool::new();
    let blocked_args = serde_json::json!({
        "code": "const secret = await Bun.file('/etc/shadow').text();"
    });
    let tool_res = eval_tool.execute(blocked_args).await;
    assert!(tool_res.is_err(), "Tool must block access to /etc/shadow");
    let err_str = tool_res.unwrap_err().to_string();
    assert!(err_str.contains("AgentShield blocked tool 'bun_eval'"));
}

#[tokio::test]
async fn test_bun_tools_registry_integration() {
    let registry = ToolRegistry::with_builtins();

    // Verify registration of all 5 Bun tools
    assert!(registry.contains("bun_eval"), "Registry must have bun_eval");
    assert!(registry.contains("bun_run"), "Registry must have bun_run");
    assert!(registry.contains("bun_test"), "Registry must have bun_test");
    assert!(registry.contains("bun_install"), "Registry must have bun_install");
    assert!(registry.contains("bun_build"), "Registry must have bun_build");

    // Execute bun_eval tool through ToolRegistry
    let valid_args = serde_json::json!({
        "code": "const x: number = 7 * 6; console.log(`The answer is ${x}`);"
    });

    let tool_result = registry.execute_call("call_bun_1", "bun_eval", &valid_args).await;
    if let tagisan::types::ContentBlock::ToolResult { content, is_error, .. } = tool_result {
        assert!(!is_error, "Execution should succeed");
        assert!(content.contains("The answer is 42"));
    } else {
        panic!("Expected ToolResult content block");
    }

    // Execute bun_eval tool with malicious code through ToolRegistry -> blocked
    let malicious_args = serde_json::json!({
        "code": "const x = Bun.file('/etc/shadow');"
    });
    let blocked_result = registry.execute_call("call_bun_2", "bun_eval", &malicious_args).await;
    if let tagisan::types::ContentBlock::ToolResult { content, is_error, .. } = blocked_result {
        assert!(is_error, "Malicious call must result in is_error=true");
        assert!(content.contains("AgentShield blocked"));
    } else {
        panic!("Expected ToolResult content block");
    }
}

#[tokio::test]
async fn test_concurrent_multithreaded_bun_execution() {
    let runtime = Arc::new(BunRuntime::new().expect("BunRuntime::new should succeed"));

    let num_tasks = 16;
    let mut handles = Vec::new();

    for i in 0..num_tasks {
        let rt = runtime.clone();
        let handle = tokio::spawn(async move {
            let code = format!(
                "const n: number = {}; const fib = (n: number): number => n <= 1 ? n : fib(n - 1) + fib(n - 2); console.log(`FIB_${{n}}=${{fib(10)}}`);",
                i
            );
            let res = rt
                .eval(&code, Duration::from_secs(10), None, None)
                .await
                .expect("Concurrent eval must succeed");
            assert_eq!(res.exit_code, 0);
            assert!(res.stdout.contains("FIB_"));
            assert!(res.stdout.contains("=55"));
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.expect("Task must complete without panic");
    }
}

#[tokio::test]
async fn test_repl_bun_command_execution() {
    let ctx = EngineContext::new(5.0);
    let agent = AutonomousAgent::new(
        ctx.get_provider("gemini").unwrap_or_else(|_| ctx.get_provider("ollama").unwrap_or_else(|_| {
            let p: Arc<dyn tagisan::providers::LlmProvider> = Arc::new(tagisan::providers::gemini::GeminiProvider::new("dummy-key"));
            p
        })),
        "test-model",
        ToolRegistry::with_builtins(),
    );

    let mut repl = InteractiveRepl::new(agent, "test-repl-bun", "test-model", ctx);

    // Test /bun evaluation command
    let cmd = ReplCommand::Bun("const sum: number = [1, 2, 3, 4].reduce((a, b) => a + b, 0); console.log('SUM:', sum);".to_string());
    let out = repl.execute_command(cmd).await.expect("Repl command execution should succeed");

    assert!(out.is_some());
    let text = out.unwrap();
    assert!(text.contains("Bun execution completed"));
    assert!(text.contains("SUM: 10"));

    // Test /bun empty code usage message
    let empty_cmd = ReplCommand::Bun("".to_string());
    let empty_out = repl.execute_command(empty_cmd).await.expect("Empty command should succeed");
    assert!(empty_out.unwrap().contains("Usage: /bun <ts_code>"));
}
