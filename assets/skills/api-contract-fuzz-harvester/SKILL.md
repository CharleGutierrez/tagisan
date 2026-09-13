---
name: api-contract-fuzz-harvester
description: Autonomous API contract & property fuzzing (Schemathesis/OpenAPI/gRPC). Generates boundary integers, null bytes, long buffer payloads, unicode fuzz, captures 500s/panics, and generates reproducible test cases.
version: 1.0.0
tags:
  - api-contract-testing
  - property-fuzzing
  - schemathesis
  - openapi-fuzzing
  - grpc-fuzzing
  - boundary-testing
  - panic-harvesting
triggers:
  - api-fuzzing
  - contract-fuzzing
  - schemathesis
  - openapi
  - grpc-fuzz
  - boundary-payloads
  - null-byte-fuzz
  - panic-harvesting
  - api-resilience
compatibility: ">=0.2.0"
---

# API Contract & Property Fuzz Harvester: Autonomous Boundary Stress & Panic Capture

## Purpose & Scope
Modern REST, OpenAPI, and gRPC endpoints frequently pass standard unit and integration tests because conventional tests only exercise nominal, valid payloads. In production, unvalidated user input, malicious edge-cases, integer overflows, unexpected null bytes, and schema mismatches trigger unhandled HTTP 500 Internal Server Errors, panics, and Denial-of-Service crashes.

This skill equips autonomous agents with property-based API contract fuzzing and panic harvesting. By synthesizing extreme boundary mutations, generating deeply nested structures, injecting null bytes and SQL/command injection vectors, and verifying schema compliance against OpenAPI/gRPC specifications, the harvester uncovers hidden crashes and produces deterministic reproduction fixtures.

---

## 1. Operational Invariants (API Contract Fuzzing Protocol)

### Invariant 1: Exhaustive Boundary & Singularity Generation
- **MANDATORY**: Test generators must synthesize inputs across all canonical boundary singularities:
  1. **Numeric Boundaries**: `0`, `-1`, `1`, `i32::MIN`, `i32::MAX`, `i64::MIN`, `i64::MAX`, `u64::MAX`, float `NaN`, `+Infinity`, `-Infinity`, subnormals (`1e-324`), and negative values in unsigned fields.
  2. **String Singularities**: Null bytes (`\0`), control characters, unescaped quotes, bidirectional text (`\u{202E}`), emoji sequences, invalid UTF-8 byte sequences, and oversized buffers (>64KB).
  3. **Injection Vectors**: Metacharacters for SQL (`' OR '1'='1' --`), shell expansions (`$(whoami)`), and script injections (`<script>`).
  4. **Structural Extremes**: Deeply nested JSON payloads (>50 levels), empty objects (`{}`), empty arrays (`[]`), and heterogeneous arrays.

### Invariant 2: Zero Unhandled 500s or Panics
- **STRICT_REJECT**: Under no circumstance is an API allowed to respond with HTTP 500 Internal Server Error, gRPC `UNKNOWN`, crash, or panic due to malformed, unexpected, or oversized client input.
- **ALWAYS**: The server must reject invalid inputs gracefully with clean 4xx Bad Request or gRPC `INVALID_ARGUMENT` responses containing structured, machine-parseable error envelopes.

### Invariant 3: Deterministic Test Case Reproduction
- **MANDATORY**: Whenever a failure, 500 status, crash, or contract violation is harvested, the engine must serialize a deterministic, minimal reproduction artifact containing:
  1. Raw HTTP request / gRPC payload with exact headers and query parameters.
  2. Standalone `curl` reproduction command.
  3. Executable proptest / unit test harness fixture with fixed random seed.
- **NEVER**: Report an API vulnerability or bug without providing an automated reproducible test case.

### Invariant 4: Continuous Schema Contract Invariant Verification
- **ALWAYS**: Verify that successful (2xx) responses strictly comply with the documented OpenAPI/JSON Schema: required fields must be present, data types must match, and unexpected enum variants must be prohibited.
- **MANDATORY**: Enforce idempotency properties on `PUT` and `DELETE` requests: repeating the same operation with identical parameters must produce consistent server state.

---

## 2. Canonical Fuzzing Harnesses

### Standalone Curl Reproduction Template
```bash
curl -X POST "http://localhost:8080/api/v1/resource" \
  -H "Content-Type: application/json" \
  -d '{"id": 2147483648, "username": "admin\u0000payload", "metadata": {"depth": {"nested": true}}}'
```

### Rust Proptest Property Assertion
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_api_payload_deserialization(
        num in any::<i64>(),
        payload in "\\PC*"
    ) {
        let json_body = serde_json::json!({
            "amount": num,
            "description": payload
        });
        // Handler must never panic, only return Ok or Err(Validation)
        let result = handle_request(&json_body);
        prop_assert!(result.is_ok() || result.unwrap_err().is_validation_error());
    }
}
```

---

## 3. Tool Invocations

Use `api_contract_fuzzer` to generate mutations, evaluate schema vulnerability, and generate reproduction cases:
- `api_contract_fuzzer(action: "generate_fuzz_payloads", schema: { ... }, max_mutations: 20)`
- `api_contract_fuzzer(action: "fuzz_schema", schema: { ... })`
- `api_contract_fuzzer(action: "reproduce_case", schema: { ... }, target_url: "http://...")`
