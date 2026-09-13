---
name: chaos-fault-injector
description: Autonomous chaos engineering. Injects network latency spikes, packet drops, clock drift, process kill, and memory pressure; verifies circuit breakers and fallback recovery.
version: 1.0.0
tags:
  - chaos-engineering
  - fault-injection
  - circuit-breakers
  - latency-injection
  - resilience-testing
  - packet-drop
  - clock-drift
triggers:
  - chaos-engineering
  - fault-injector
  - latency-spikes
  - packet-drop
  - clock-drift
  - memory-pressure
  - circuit-breaker
  - resilience
  - chaos-monkey
compatibility: ">=0.2.0"
---

# Chaos Fault Injector: Autonomous Chaos Engineering & Resilience Assurance

## Purpose & Scope
Distributed systems fail in complex, non-deterministic ways. Network partitions, saturated queues, noisy neighbors, transient clock drift, packet loss, and sudden out-of-memory process kills routinely cascade into total system outages when failure recovery mechanisms are untried.

This skill equips autonomous agents with proactive chaos engineering. By injecting controlled, synthetic failure vectors into network, kernel, process, and memory subsystems, agents empirically validate that circuit breakers trip, bulkhead pools isolate faults, timeouts trigger gracefully, and self-healing algorithms restore steady-state operations without manual intervention.

---

## 1. Operational Invariants (Chaos Engineering Protocol)

### Invariant 1: Synthetic Multi-Vector Fault Injection
- **MANDATORY**: Resilience testing must exercise all five core fault archetypes:
  1. **Network Latency Spikes**: Inject synthetic jitter and latency increases (+200ms to +5,000ms).
  2. **Packet Drops & SITA**: Simulate probabilistic network loss (1% to 50% packet drop rates).
  3. **Clock Drift**: Simulate clock skew (+/- 500ms) to uncover distributed timestamp consensus failures.
  4. **Process Termination**: Issue unannounced SIGTERM and SIGKILL signals to worker processes.
  5. **Memory & CPU Pressure**: Restrict available heap memory to provoke GC pressure and OOM containment.
- **ALWAYS**: Establish an automated safety blast-radius ceiling with a hard timeout abort to prevent uncontained infrastructure damage.

### Invariant 2: Circuit Breaker & Fallback Verification
- **MANDATORY**: Downstream service integrations must incorporate circuit breakers (e.g. Closed -> Open -> Half-Open). When the failure rate exceeds 50% over a sliding window or latency exceeds the p99 SLA budget, the breaker must trip to Open within <= 3 consecutive failures.
- **STRICT_REJECT**: Reject any critical service integration lacking an automated, non-throwing fallback path (e.g. stale cache response, degraded functional mode, or default safe payload) when a circuit breaker is open.

### Invariant 3: Self-Healing & Half-Open State Probing
- **ALWAYS**: Test the recovery phase following chaos injection. After the fault subsides and the probe timeout expires, the circuit breaker must transition to Half-Open, canary probe downstream health, and automatically resume full traffic in Closed state.
- **NEVER**: Require manual process restarts or operator interventions to recover from transient downstream outages.
- **NEVER**: Leave leaked threads, hanging sockets, or zombie child processes following a chaos injection cycle.

### Invariant 4: Steady-State Hypothesis & Telemetry Assertions
- **MANDATORY**: Formulate an explicit steady-state hypothesis before initiating chaos (e.g. "Cluster throughput remains >= 90% of nominal baseline; end-user error rate remains <= 0.1%"). Continuously record telemetry to prove or disprove the hypothesis.

---

## 2. Canonical Resilience Patterns

### Circuit Breaker & Fallback Implementation (Rust)
```rust
pub async fn fetch_with_circuit_breaker<T, F, Fut, B>(
    circuit: &mut CircuitBreaker,
    primary_call: F,
    fallback_call: B,
) -> Result<T, Error>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
    B: FnOnce() -> T,
{
    if circuit.is_open() {
        // Graceful degradation fallback
        return Ok(fallback_call());
    }

    match tokio::time::timeout(circuit.timeout_duration(), primary_call()).await {
        Ok(Ok(val)) => {
            circuit.record_success();
            Ok(val)
        }
        Ok(Err(err)) => {
            circuit.record_failure();
            Ok(fallback_call())
        }
        Err(_timeout) => {
            circuit.record_failure();
            Ok(fallback_call())
        }
    }
}
```

---

## 3. Tool Invocations

Use `chaos_fault_injector` to simulate synthetic faults, wrap shell commands, and profile resilience:
- `chaos_fault_injector(action: "simulate_fault", fault_type: "latency", latency_ms: 250, failure_rate: 0.3)`
- `chaos_fault_injector(action: "wrap_command", target_command: "curl -s http://api/health", fault_type: "timeout")`
- `chaos_fault_injector(action: "profile_resilience", fault_type: "latency", latency_ms: 100, failure_rate: 0.2)`
