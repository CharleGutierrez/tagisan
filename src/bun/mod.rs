pub mod diagnostics;
pub mod runtime;
pub mod sandbox;
pub mod sqlite_memory;
pub mod worker_pool;

pub use diagnostics::{DiagnosticSeverity, TsDiagnostic, TsDiagnosticParser};
pub use runtime::{BunExecutionResult, BunRuntime};
pub use sandbox::{BunSandbox, BunSandboxConfig};
pub use sqlite_memory::TagisanSqliteStore;
pub use worker_pool::{BunWorker, BunWorkerPool, WorkerPoolStats};

// =========================================================================
// Zero-Copy C-FFI Bridge (bun:ffi + Rust cdylib)
// =========================================================================

use crate::ecc::agentshield::{AgentShieldScanner, AgentShieldVerdict};
use crate::memory::embedding::cosine_similarity;
use std::ffi::{c_char, CStr};
use std::slice;

static VERSION_C_STR: &[u8] = b"tagisan-0.1.0-bun-native\0";

/// Returns Tagisan native FFI engine version string (null-terminated)
#[no_mangle]
pub extern "C" fn tagisan_ffi_version() -> *const c_char {
    VERSION_C_STR.as_ptr() as *const c_char
}

/// Zero-copy SIMD-accelerated cosine similarity calculation over raw f32 pointers
#[no_mangle]
pub extern "C" fn tagisan_ffi_cosine_similarity(
    a: *const f32,
    b: *const f32,
    len: usize,
) -> f32 {
    if a.is_null() || b.is_null() || len == 0 {
        return 0.0;
    }

    let slice_a = unsafe { slice::from_raw_parts(a, len) };
    let slice_b = unsafe { slice::from_raw_parts(b, len) };

    cosine_similarity(slice_a, slice_b)
}

/// Computes BLAKE3 cryptographic digest for raw byte buffer and writes 64-character hex to out_hex
/// Returns 0 on success, -1 on invalid/null pointers
#[no_mangle]
pub extern "C" fn tagisan_ffi_blake3_digest(
    data: *const u8,
    len: usize,
    out_hex: *mut u8,
) -> i32 {
    if data.is_null() || out_hex.is_null() {
        return -1;
    }

    let input_slice = unsafe { slice::from_raw_parts(data, len) };
    let hash = blake3::hash(input_slice);
    let hex_str = hash.to_hex();
    let hex_bytes = hex_str.as_bytes(); // 64 ASCII bytes

    unsafe {
        std::ptr::copy_nonoverlapping(hex_bytes.as_ptr(), out_hex, 64);
    }
    0
}

/// Fast native C-ABI scan of JavaScript/TypeScript code using AgentShield AST security heuristics
/// Returns 0 for Allow, 1 for Block
#[no_mangle]
pub extern "C" fn tagisan_ffi_shield_scan(code: *const c_char) -> i32 {
    if code.is_null() {
        return 0;
    }

    let c_str = unsafe { CStr::from_ptr(code) };
    let code_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return 1, // Invalid UTF-8 blocked
    };

    let payload = serde_json::json!({
        "code": code_str,
    });

    match AgentShieldScanner::scan_tool_call("bun_eval", &payload) {
        AgentShieldVerdict::Allow => 0,
        AgentShieldVerdict::Block { .. } => 1,
    }
}

