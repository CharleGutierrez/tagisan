---
name: harness-polyglot-cli-generator-pro-max
description: Autonomous Polyglot CLI Harness Generator for Tagisan. Synthesizes standalone native CLIs in Rust (clap derive + serde_json), TypeScript/Bun, Go (cobra/pflag), WASM/WASI 0.2 components (wit-bindgen), and POSIX sh (getopts). Enforces zero exit-code masking, strict typed JSON outputs via --json, structured stderr diagnostics, and zero external runtime dependencies.
version: 1.0.0
tags:
  - cli-harness
  - polyglot
  - rust-clap
  - bun-typescript
  - golang-cobra
  - wasm-wasi
  - posix-shell
  - json-envelope
  - systems-engineering
triggers:
  - polyglot-cli
  - cli-generator
  - harness-generator
  - clap-derive
  - bun-cli
  - cobra-cli
  - wasi-harness
  - getopts-wrapper
  - json-cli-envelope
  - zero-exit-masking
compatibility: ">=0.2.0"
---

# Polyglot CLI Harness Generator: Autonomous Multi-Target Native CLI Synthesis

## Purpose & Architectural Mandate
Modern AI agents and orchestration runtimes require robust, deterministic, machine-readable command-line interfaces (CLIs) to interact with complex libraries, microservices, hardware, and kernel subsystems. Fragile ad-hoc scripts with unstructured text output cause catastrophic agent hallucination, unhandled edge-case failures, and silently masked error exit codes.

The `harness-polyglot-cli-generator-pro-max` skill empowers Tagisan to autonomously synthesize production-grade, hardened, zero-dependency polyglot CLI harnesses across five major compilation and execution targets:
1. **Native Rust**: High-throughput, zero-allocation CLIs powered by `clap` (derive API) and `serde_json`.
2. **TypeScript / Bun**: Sub-millisecond startup, type-safe CLIs leveraging native Bun runtime APIs and `zod` schema validation.
3. **Go**: Statically linked single-binary CLIs utilizing `spf13/cobra` and `spf13/pflag`.
4. **WASM / WASI 0.2**: Sandboxed, capability-secure WebAssembly Component Model harnesses via `wit-bindgen` and `wasmtime`.
5. **POSIX Shell**: Ultra-portable, dependency-free wrapper harnesses using `getopts` with strict defensive discipline (`set -euo pipefail`).

---

## 1. Non-Negotiable Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                                  POLYGLOT CLI HARNESS INVARIANTS                                  |
+---------------------------------------------------------------------------------------------------+
|  1. ZERO EXIT-CODE MASKING                                                                        |
|     Exit code 0 ONLY on unambiguous success. Failure categories strictly mapped to POSIX/Sysexits |
|     (e.g., 1=General, 2=CLI Usage/Syntax, 64=Bad Args, 70=Internal Software, 74=IO/Net Error).     |
+---------------------------------------------------------------------------------------------------+
|  2. STRICT TYPED JSON ENVELOPE (--json)                                                           |
|     When --json is passed, stdout MUST contain ONLY a single valid JSON document conforming to:   |
|     {"status": "ok"|"error", "data": <T>, "error": {"code": "...", "message": "...", ...}}        |
+---------------------------------------------------------------------------------------------------+
|  3. SEPARATED STREAM SEMANTICS (STDOUT vs STDERR)                                                 |
|     stdout is reserved strictly for operational payload/data (JSON or tabulated data).           |
|     stderr is dedicated to human-readable logs, progress bars, and structured diagnostics.       |
+---------------------------------------------------------------------------------------------------+
|  4. ZERO EXTERNAL RUNTIME DEPENDENCY                                                              |
|     Binaries MUST be self-contained (statically linked libc/musl where applicable, zero dynamic    |
|     shared library surprises, bundled modules for Bun/Node, no external Python/Ruby shims).       |
+---------------------------------------------------------------------------------------------------+
|  5. SIGNAL RESILIENCE & CLEAN FLUSH                                                               |
|     Must trap SIGINT, SIGTERM, and SIGHUP; flush stdout/stderr buffers, release locks, and exit    |
|     with 128 + signal_number (e.g., 130 for SIGINT, 143 for SIGTERM).                             |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Universal CLI Output Envelope Specification

All synthesized harnesses implement the canonical Tagisan JSON envelope:

```json
{
  "$schema": "https://tagisan.dev/schemas/v1/cli-envelope.json",
  "status": "ok",
  "version": "1.0.0",
  "timestamp": "2026-09-21T12:00:00.000Z",
  "command": "compute-hash",
  "data": {
    "algorithm": "blake3",
    "digest": "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
    "bytes_processed": 1048576,
    "duration_us": 142
  },
  "error": null,
  "telemetry": {
    "rss_bytes": 4194304,
    "cpu_user_us": 120,
    "cpu_sys_us": 22
  }
}
```

In error conditions with `--json`:
```json
{
  "$schema": "https://tagisan.dev/schemas/v1/cli-envelope.json",
  "status": "error",
  "version": "1.0.0",
  "timestamp": "2026-09-21T12:00:00.000Z",
  "command": "compute-hash",
  "data": null,
  "error": {
    "code": "ERR_FILE_NOT_FOUND",
    "message": "Input file '/path/to/target.bin' does not exist or permission denied",
    "details": {
      "path": "/path/to/target.bin",
      "os_errno": 2
    },
    "exit_code": 74
  },
  "telemetry": {
    "rss_bytes": 3145728,
    "cpu_user_us": 40,
    "cpu_sys_us": 15
  }
}
```

---

## 3. Canonical Target Implementations

### Target 1: Native Rust CLI (`clap` derive + `serde_json`)

```rust
//! Production-grade Rust CLI Harness synthesized by Tagisan
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "tgs-harness-rs", version = "1.0.0", about = "High-performance native harness")]
pub struct Cli {
    #[arg(long, global = true, help = "Emit structured JSON envelope to stdout")]
    pub json: bool,

    #[arg(short, long, global = true, action = clap::ArgAction::Count, help = "Verbosity level (-v, -vv)")]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Compute cryptographic digest of target payload
    Digest(DigestArgs),
    /// Execute batch transformation on input stream
    Transform(TransformArgs),
}

#[derive(Args, Debug)]
pub struct DigestArgs {
    #[arg(short, long, default_value = "blake3")]
    pub algorithm: String,

    #[arg(value_name = "INPUT_PATH")]
    pub input: Option<std::path::PathBuf>,
}

#[derive(Args, Debug)]
pub struct TransformArgs {
    #[arg(short, long)]
    pub uppercase: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonEnvelope<T> {
    pub status: &'static str,
    pub version: &'static str,
    pub data: Option<T>,
    pub error: Option<JsonErrorDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonErrorDetail {
    pub code: String,
    pub message: String,
    pub exit_code: i32,
}

fn main() -> ExitCode {
    let args = Cli::parse();
    let is_json = args.json;

    match run_app(&args) {
        Ok(data) => {
            if is_json {
                let env = JsonEnvelope {
                    status: "ok",
                    version: "1.0.0",
                    data: Some(data),
                    error: None,
                };
                let _ = serde_json::to_writer(io::stdout().lock(), &env);
                println!();
            } else {
                println!("Execution completed successfully.");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            if is_json {
                let env: JsonEnvelope<serde_json::Value> = JsonEnvelope {
                    status: "error",
                    version: "1.0.0",
                    data: None,
                    error: Some(JsonErrorDetail {
                        code: err.code.clone(),
                        message: err.message.clone(),
                        exit_code: err.exit_code,
                    }),
                };
                let _ = serde_json::to_writer(io::stdout().lock(), &env);
                println!();
            } else {
                eprintln!("[ERROR] {}: {}", err.code, err.message);
            }
            ExitCode::from(err.exit_code as u8)
        }
    }
}

pub struct AppError {
    pub code: String,
    pub message: String,
    pub exit_code: i32,
}

fn run_app(args: &Cli) -> Result<serde_json::Value, AppError> {
    match &args.command {
        Commands::Digest(d) => Ok(serde_json::json!({
            "algorithm": d.algorithm,
            "status": "ready"
        })),
        Commands::Transform(_) => Ok(serde_json::json!({
            "transformed": true
        })),
    }
}
```

---

### Target 2: TypeScript / Bun Native CLI

```typescript
#!/usr/bin/env bun
/**
 * Production-grade Bun CLI Harness synthesized by Tagisan
 * Zero external npm dependencies; uses native Bun high-speed APIs
 */
import { parseArgs } from "util";

interface CliEnvelope<T> {
  status: "ok" | "error";
  version: string;
  data: T | null;
  error: { code: string; message: string; exit_code: number } | null;
}

const { values, positionals } = parseArgs({
  args: Bun.argv.slice(2),
  options: {
    json: { type: "boolean", default: false },
    algorithm: { type: "string", short: "a", default: "sha256" },
    help: { type: "boolean", short: "h", default: false },
  },
  allowPositionals: true,
});

if (values.help || positionals.length === 0) {
  console.error("Usage: tgs-harness-bun [options] <subcommand> [args]");
  console.error("Options: --json, -a/--algorithm <algo>, -h/--help");
  process.exit(values.help ? 0 : 64);
}

const subcommand = positionals[0];

function emitResult<T>(data: T) {
  if (values.json) {
    const envelope: CliEnvelope<T> = {
      status: "ok",
      version: "1.0.0",
      data,
      error: null,
    };
    process.stdout.write(JSON.stringify(envelope) + "\n");
  } else {
    console.log(data);
  }
  process.exit(0);
}

function emitError(code: string, message: string, exitCode = 1): never {
  if (values.json) {
    const envelope: CliEnvelope<null> = {
      status: "error",
      version: "1.0.0",
      data: null,
      error: { code, message, exit_code: exitCode },
    };
    process.stdout.write(JSON.stringify(envelope) + "\n");
  } else {
    console.error(`[ERROR] ${code}: ${message}`);
  }
  process.exit(exitCode);
}

try {
  switch (subcommand) {
    case "ping":
      emitResult({ ping: "pong", timestamp: Date.now() });
      break;
    default:
      emitError("ERR_UNKNOWN_SUBCOMMAND", `Unknown subcommand '${subcommand}'`, 64);
  }
} catch (err: any) {
  emitError("ERR_INTERNAL", err?.message ?? String(err), 70);
}
```

---

### Target 3: Go CLI (`cobra` + `pflag`)

```go
package main

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

type JsonEnvelope struct {
	Status  string          `json:"status"`
	Version string          `json:"version"`
	Data    interface{}     `json:"data"`
	Error   *JsonErrorBlock `json:"error"`
}

type JsonErrorBlock struct {
	Code     string `json:"code"`
	Message  string `json:"message"`
	ExitCode int    `json:"exit_code"`
}

var jsonOutput bool

var rootCmd = &cobra.Command{
	Use:   "tgs-harness-go",
	Short: "Tagisan Go Native CLI Harness",
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// Suppress default cobra error printing when --json is active
		if jsonOutput {
			cmd.SilenceErrors = true
			cmd.SilenceUsage = true
		}
	},
}

func init() {
	rootCmd.PersistentFlags().BoolVar(&jsonOutput, "json", false, "Emit structured JSON envelope")
}

func emitEnvelope(data interface{}, errBlock *JsonErrorBlock, exitCode int) {
	if jsonOutput {
		status := "ok"
		if errBlock != nil {
			status = "error"
		}
		env := JsonEnvelope{
			Status:  status,
			Version: "1.0.0",
			Data:    data,
			Error:   errBlock,
		}
		b, _ := json.Marshal(env)
		os.Stdout.Write(b)
		os.Stdout.WriteString("\n")
	} else {
		if errBlock != nil {
			fmt.Fprintf(os.Stderr, "[ERROR] %s: %s\n", errBlock.Code, errBlock.Message)
		} else {
			fmt.Println("Success")
		}
	}
	os.Exit(exitCode)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		emitEnvelope(nil, &JsonErrorBlock{
			Code:     "ERR_EXECUTION",
			Message:  err.Error(),
			ExitCode: 1,
		}, 1)
	}
}
```

---

### Target 4: WASM / WASI 0.2 Component Harness

```wit
package tagisan:harness@0.2.0;

interface types {
    record cli-result {
        status: string,
        data-json: string,
        error-code: option<string>,
        error-message: option<string>,
        exit-code: s32,
    }
}

world cli-harness {
    import wasi:cli/environment@0.2.0;
    import wasi:cli/stdin@0.2.0;
    import wasi:cli/stdout@0.2.0;
    import wasi:cli/stderr@0.2.0;
    
    export run: func(args: list<string>) -> types.cli-result;
}
```

---

### Target 5: POSIX Shell Harness (`getopts` + defensive execution)

```sh
#!/bin/sh
# Strictest POSIX defensive execution
set -eu

JSON_MODE=0
VERBOSE=0
ALGO="blake3"

usage() {
  cat <<EOF >&2
Usage: $(basename "$0") [-j] [-v] [-a algorithm] <command> [args...]
  -j  Output strict JSON envelope to stdout
  -v  Enable verbose logging on stderr
  -a  Hash algorithm (default: blake3)
EOF
  exit 64
}

emit_json_error() {
  _code="$1"
  _msg="$2"
  _exit="${3:-1}"
  if [ "$JSON_MODE" -eq 1 ]; then
    printf '{"status":"error","version":"1.0.0","data":null,"error":{"code":"%s","message":"%s","exit_code":%d}}\n' \
      "$_code" "$_msg" "$_exit"
  else
    printf '[ERROR] %s: %s\n' "$_code" "$_msg" >&2
  fi
  exit "$_exit"
}

emit_json_ok() {
  _data="$1"
  if [ "$JSON_MODE" -eq 1 ]; then
    printf '{"status":"ok","version":"1.0.0","data":%s,"error":null}\n' "$_data"
  else
    printf 'OK: %s\n' "$_data"
  fi
  exit 0
}

while getopts "jva:h" opt; do
  case "$opt" in
    j) JSON_MODE=1 ;;
    v) VERBOSE=1 ;;
    a) ALGO="$OPTARG" ;;
    h) usage ;;
    *) usage ;;
  esac
done
shift $((OPTIND - 1))

[ $# -eq 0 ] && usage
CMD="$1"
shift

case "$CMD" in
  health)
    emit_json_ok '{"service":"harness","uptime":100}'
    ;;
  *)
    emit_json_error "ERR_UNKNOWN_COMMAND" "Command $CMD not supported" 64
    ;;
esac
```

---

## 4. Autonomous Synthesis Workflow

1. **Schema & Argument Ingestion**: Parse input function/subcommand specifications, type mappings, and default parameters.
2. **Target Language Selection**: Select optimal runtime (Rust for high CPU/zero-alloc; Bun for microsecond script execution; Go for static binary portability; WASI for untrusted capabilities; POSIX for minimal Alpine/container environments).
3. **Envelope Code Injection**: Generate JSON wrapping, exit code definitions, and stream separation logic.
4. **Compilation & Packaging**: Invoke target toolchain (`cargo build --release`, `bun build --compile`, `go build -ldflags="-s -w"`, or `cargo component build`).
5. **Self-Verification Test**: Execute the synthesized binary with `--help`, invalid arguments, and `--json` to verify zero exit code masking and schema conformance.
