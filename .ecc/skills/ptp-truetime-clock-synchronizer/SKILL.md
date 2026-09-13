---
name: ptp-truetime-clock-synchronizer
description: "Precision Time Protocol (IEEE 1588 PTP), hardware nanosecond timestamping, Google TrueTime uncertainty intervals, and externally consistent distributed transactions."
version: 0.2.0
tags:
  - distributed-systems
  - ptp
  - ieee1588
  - truetime
  - clock-sync
  - external-consistency
  - consensus
  - timestamping
compatibility: ">=0.2.0"
triggers:
  - "ptp"
  - "ieee 1588"
  - "truetime"
  - "clock sync"
  - "uncertainty window"
  - "commit wait"
  - "external consistency"
  - "nanosecond timestamp"
  - "hardware clock"
  - "spanner"
---

# PTP & TrueTime Clock Synchronizer Skill

The `ptp-truetime-clock-synchronizer` skill delivers ultra-precise nanosecond-accurate clock synchronization and bounded time uncertainty management based on the Precision Time Protocol (IEEE 1588-2019 / PTPv2) and Google TrueTime architecture, enabling strict external consistency (linearizability) for distributed transactions without cross-datacenter two-phase locking.

## Core Capabilities
1. **IEEE 1588 PTP Hardware Timestamping**: Coordinates with hardware network interface cards (NIC PHY/MAC) to capture sub-microsecond timestamps (`SOF` start of frame) bypassing OS kernel networking jitter.
2. **TrueTime Bounded Uncertainty Intervals**: Computes `[earliest, latest]` time boundaries (`now ± ε`) where uncertainty `ε` is strictly bounded by atomic clock and GPS drift parameters.
3. **Commit-Wait Linearizability Engine**: Implements the TrueTime commit-wait protocol: an update transaction with timestamp $s$ waits until `now.earliest > s` before releasing locks, guaranteeing that subsequent transactions observe strictly higher timestamps across the entire cluster.
4. **Clock Drift & Skew Anomaly Detector**: Detects local oscillator drift, temperature instability, and PTP grandmaster failover events, triggering automatic failover to local fallback holdover clocks.

## Strict Operational Invariants

- **ALWAYS**:
  - Represent all physical timestamps as bounded uncertainty intervals `[t_min, t_max]` where `t_max - t_min = 2 * epsilon`.
  - Enforce monotonic ordering guarantees: never allow clock adjustments (such as NTP step jumps) to cause backward time progression in transaction sequencers.
  - Apply commit-wait delays to any transaction whose commit timestamp falls within the current clock uncertainty window.
  - Rely on NIC hardware timestamping (`SO_TIMESTAMPING` / PTP hardware clock `/dev/ptpX`) over software timestamps whenever available.

- **NEVER**:
  - NEVER assume zero clock skew between distributed nodes; point-in-time scalar clocks (`SystemTime::now()`) must never be compared across node boundaries.
  - NEVER release transaction locks before the commit-wait uncertainty window has elapsed (`wait_until_after(commit_time)`).
  - NEVER permit non-monotonic step corrections to system clocks during active transactional execution (use slew adjustments instead).
  - NEVER proceed with strict linearizable transactions if clock uncertainty $\epsilon$ exceeds the maximum configured threshold (e.g., $\epsilon > 5 \text{ ms}$).

- **MANDATORY**:
  - Continuously monitor clock uncertainty $\epsilon$; flag degraded cluster health if GPS/PTP grandmaster loss causes $\epsilon$ to widen beyond safety limits.
  - Implement dual-source validation (e.g. primary GPS receiver + atomic rubidium/cesium standard backup) to survive upstream time server failures.
  - Format distributed causal trace headers with hybrid logical clocks (HLC) or bounded interval markers.

- **STRICT_REJECT**:
  - Reject distributed transaction algorithms that rely on raw unsynchronized wall-clock timestamps for global serializability.
  - Reject systems running commit-wait protocols with unmonitored or unbounded clock uncertainty $\epsilon$.
  - Reject PTP client implementations that bypass hardware timestamping when network interface cards support IEEE 1588.
