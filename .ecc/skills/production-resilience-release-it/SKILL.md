---
name: production-resilience-release-it
description: "Production Resiliency Engineering (Michael Nygard - Release It!): circuit breakers, bulkheads, timeouts, steady-state stability, anti-fragility, and defense against cascading failures and retry storms."
triggers: ["release it", "nygard", "circuit breaker", "bulkhead", "retry storm", "cascading failure", "dogpiling", "steady state", "timeouts", "fail fast", "shed load", "backpressure"]
---

# Production Resiliency and Stability Patterns (Michael Nygard - Release It!)

This skill equips the agent with the production-hardening and stability engineering patterns from Michael Nygard's *Release It!* to ensure services survive network partitions, load spikes, and upstream outages.

## 1. Stability Anti-Patterns to Eliminate
1. **Cascading Failures**:
   - An outage in one downstream service causes callers to block, exhausting their connection pools and bringing down the entire microservice ecosystem.
2. **Retry Storms and Dogpiling**:
   - When a service stumbles, clients simultaneously retry with static delays, amplifying traffic and ensuring the service cannot recover.
   - *Defense*: Exponential backoff with full randomized jitter; circuit breakers.
3. **Unbounded Queues and Thread Exhaustion**:
   - Uncapped queues cause Out-Of-Memory (OOM) crashes under backpressure. Always bound queue depth.
4. **Missing Timeouts**:
   - Sockets and HTTP clients left with infinite default timeouts. Every single network I/O call must have an explicit connect and read timeout.

## 2. Core Stability Patterns
1. **Circuit Breaker**:
   - **Closed**: Requests flow normally. Failures increment an error counter.
   - **Open**: When failure threshold is breached, fail requests immediately without touching downstream resources.
   - **Half-Open**: After a cooldown period, permit a single probe request to check if downstream has recovered.
2. **Bulkheads (Failure Domain Isolation)**:
   - Partition resources (thread pools, connection pools, CPU cores) so failure in one partition cannot exhaust resources needed for core operations.
3. **Fail Fast**:
   - Validate basic prerequisites (API keys, network reachability, parameter bounds) upfront before acquiring expensive locks or allocating resources.
4. **Steady State**:
   - Systems must run indefinitely without human intervention. Avoid unbounded log files, runaway disk caches, or memory leaks. Implement automatic data purging and log rotation.
5. **Backpressure and Load Shedding**:
   - When saturated, reject excess requests early with HTTP 429 / 503 or dropped frames rather than queueing them into high-latency death spirals.
