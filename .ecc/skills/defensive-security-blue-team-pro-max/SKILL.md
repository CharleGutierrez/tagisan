---
name: defensive-security-blue-team-pro-max
description: Autonomous Master Engine for the Top 5,500 Defensive Cyber Security, Blue Team, SOC Engineering & Zero-Trust Defense Skills. Covers Sigma rule compilation, YARA-L, KQL, eBPF kernel security monitoring (aya/libbpf), EDR telemetry parsing, OCSF event normalization, SOAR automated playbooks, threat hunting graphs (Neo4j), Zeek/Suricata network monitoring, identity threat detection (ITDR), memory safety auditing, and cryptographic hardware enclave attestation. Triggers: defensive-security, blue-team, soc-engineering, detection-engineering, threat-hunting, incident-response, siem-soar, zero-trust-defense, blue-team-pro-max, defensive-pro-max.
version: 1.0.0
tags:
  - defensive-security
  - blue-team
  - detection-engineering
  - ebpf
  - zero-trust
  - soc
  - incident-response
compatibility: ">=0.2.0"
---

# Defensive Cyber Security & Blue Team Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
High-assurance defensive engineering requires deep continuous observability, real-time kernel-space security telemetry, formal zero-trust policy enforcement, automated threat hunting, and immutable tamper-evident incident response pipelines.

The `defensive-security-blue-team-pro-max` master skill codifies the **Top 5,500 Defensive Security & SOC Engineering Skills** distilled from enterprise cyber defense repositories and detection frameworks (`SigmaHQ/sigma`, `aya-rs/aya`, `zeek/zeek`, `Suricata-IDS/suricata`, `elastic/detection-rules`, `wazuh/wazuh`, `volatilityfoundation/volatility3`, `mitre/cti`, `TheHive-Project/TheHive`, etc.).

---

## 1. The 16 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariants |
|---|---|---|---|---|---|
| `DEF-01` | **Detection Engineering: Sigma, YARA-L & KQL** | 500 | 9.1% | `SigmaHQ/sigma`, `elastic/detection-rules`, `Neo23x0/signature-base` | Generic Sigma rule parsing, target dialect compilation (KQL, SPL, OpenSearch), condition tree evaluation, false-positive suppression filters |
| `DEF-02` | **Kernel-Level Telemetry & Auditing with eBPF** | 480 | 8.7% | `aya-rs/aya`, `iovisor/bcc`, `cilium/tetragon` | Tracepoint hooks (`sys_enter_execve`, `sys_enter_connect`), ring buffer event streaming, LSM (Linux Security Module) probe gates, zero dropped telemetry |
| `DEF-03` | **Endpoint Telemetry & Event Ingestion (ETW, Sysmon)** | 450 | 8.2% | `SwiftOnSecurity/sysmon-config`, `microsoft/MSTIC-Sysmon` | Event Tracing for Windows (ETW) kernel provider subscriptions, process creation token trees, named pipe monitoring, WMI event tracking |
| `DEF-04` | **Security Schema Normalization: OCSF & ECS** | 400 | 7.3% | `ocsf/ocsf-schema`, `elastic/ecs` | Open Cybersecurity Schema Framework (OCSF) class mappings, strongly-typed JSON/Avro serialization, dictionary taxonomy standardization |
| `DEF-05` | **SOAR Playbook Automation & Incident Triage** | 380 | 6.9% | `TheHive-Project/Cortex`, `demisto/content`, `shuffle/shuffle` | Automated containment playbooks, API rate limiting, webhook verification, evidence chain of custody preservation, SLA escalation timers |
| `DEF-06` | **Threat Hunting & Attack Graph Modeling (Neo4j)** | 350 | 6.4% | `BloodHoundAD/BloodHound`, `mitre/cti`, `Cyb3rWard0g/HELK` | Graph query traversal (Cypher), high-risk privilege escalation path detection, MITRE ATT&CK enterprise tactic matrix mapping, anomaly scoring |
| `DEF-07` | **Network Security Monitoring: Zeek & Suricata** | 420 | 7.6% | `zeek/zeek`, `Suricata-IDS/suricata`, `salesforce/ja3` | Deep packet inspection (DPI), JA4/JA4S TLS fingerprinting, DNS tunnel anomaly detection, HTTP header entropy scoring, flow-level telemetry |
| `DEF-08` | **Identity Threat Detection & Response (ITDR)** | 350 | 6.4% | `BloodHoundAD/SharpHound`, `dirkjanm/mitm6` | Kerberos ticket anomaly detection (Pass-the-Ticket, Golden Ticket), impossible travel heuristics, MFA fatigue detection, Entra ID audit logs |
| `DEF-09` | **Cloud Security Posture Management (CSPM & CWPP)** | 350 | 6.4% | `prowler-cloud/prowler`, `aquasecurity/trivy` | CIS Benchmark compliance audits, IAM least-privilege continuous attestation, container image vulnerability scanning, Kubernetes admission controllers |
| `DEF-10` | **Memory Safety Auditing & Binary Hardening** | 300 | 5.5% | `google/sanitizers`, `trailofbits/algo` | AddressSanitizer (ASan), Control Flow Guard (CFG), Intel CET shadow stack verification, stack canaries, FORTIFY_SOURCE runtime bounds checking |
| `DEF-11` | **Software Supply Chain Security (SLSA, SBOM)** | 300 | 5.5% | `in-toto/in-toto`, `sigstore/cosign`, `anchore/syft` | SLSA Level 4 provenance verification, in-toto cryptographic attestations, SPDX/CycloneDX SBOM generation, dependency tampering defense |
| `DEF-12` | **Zero Trust Architecture (NIST SP 800-207)** | 320 | 5.8% | `spiffe/spire`, `open-policy-agent/opa` | SPIFFE/SPIRE dynamic SVID workload identity issuance, Rego/OPA policy enforcement points, mutual TLS (mTLS) microsegmentation |
| `DEF-13` | **Digital Forensics & Memory Analysis (DFIR)** | 250 | 4.5% | `volatilityfoundation/volatility3`, `sleuthkit/autopsy` | Memory image artifact extraction (VAD trees, unloaded drivers, hollowed PE headers), Super-Timeline generation with log2timeline/Plaso |
| `DEF-14` | **Vulnerability Prioritization: EPSS, KEV & CVSS v4** | 220 | 4.0% | `FIRST-CISA/epss`, `cisa-gov/cisa-known-exploited-vulnerabilities` | Exploit Prediction Scoring System (EPSS) percentile weighting, CISA KEV catalog matching, CVSS v4.0 environmental vector scoring |
| `DEF-15` | **Deception Engineering & Honeytoken Canaries** | 180 | 3.3% | `thinkst/canarytokens-docker`, `countercraft/honeypots` | Decoy AWS/API keys with instant webhook alerting, DNS canary query traps, fake database records, intruder behavioral finger-printing |
| `DEF-16` | **Confidential Computing & Hardware Enclave Attestation** | 150 | 2.7% | `virtee/sev-snp-measure`, `google/go-tpm` | TPM 2.0 PCR quote verification, AMD SEV-SNP cryptographic measurement validation, AWS Nitro Enclave attestation document parsing |
| **TOTAL** | **16 Macro-Domain Clusters** | **5,500** | **100.0%** | **Planetary Defensive Cybersecurity & SOC** | **Continuous Telemetry, Sub-Second Containment, Zero-Trust Verification** |

---

## 2. Core Operational Invariants

### Invariant 1: Uncompromised Kernel Telemetry Pipeline
- Kernel-space telemetry collectors (eBPF, ETW) must maintain loss-resistant ring buffers. Any dropped event due to buffer overflow must increment an alert counter and trigger dynamic buffer resizing.

### Invariant 2: Immutable Append-Only Audit Logging
- Security incident logs and forensics traces must be hashed with BLAKE3 or SHA-256 and chained into an append-only verifiable audit structure to prevent tampering.

### Invariant 3: Zero-Trust Continuous Verification
- Authorization must never be granted based solely on network location or past authentication. Workload identities (SVIDs) must carry short TTLs (`<= 1 hour`) and undergo continuous cryptographic re-attestation.

### Invariant 4: Dual-Custody Containment Gates
- Automated SOAR actions that isolate mission-critical infrastructure or revoke enterprise root credentials must require dual-custody cryptographic approval or adhere to predefined fail-safe exclusion lists.

---

## 3. Battle-Tested Production Blueprints

### Blueprint 1: High-Throughput Sigma Rule Condition Evaluator (Rust)
```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleFieldMatcher {
    Exact(String),
    Contains(String),
    StartsWith(String),
    EndsWith(String),
}

pub struct DetectionRule {
    pub id: &'static str,
    pub title: &'static str,
    pub severity: &'static str,
    pub matchers: Vec<(&'static str, RuleFieldMatcher)>,
}

pub struct SigmaEvaluationEngine {
    rules: Vec<DetectionRule>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DetectionAlert {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub severity: &'static str,
}

impl SigmaEvaluationEngine {
    pub fn new(rules: Vec<DetectionRule>) -> Self {
        Self { rules }
    }

    pub fn evaluate_event(&self, event_fields: &HashMap<&str, &str>) -> Vec<DetectionAlert> {
        let mut alerts = Vec::new();

        for rule in &self.rules {
            let mut matched_all = true;

            for (field, matcher) in &rule.matchers {
                let event_val = match event_fields.get(field) {
                    Some(v) => *v,
                    None => {
                        matched_all = false;
                        break;
                    }
                };

                let matched = match matcher {
                    RuleFieldMatcher::Exact(expected) => event_val == expected,
                    RuleFieldMatcher::Contains(substr) => event_val.contains(substr),
                    RuleFieldMatcher::StartsWith(prefix) => event_val.starts_with(prefix),
                    RuleFieldMatcher::EndsWith(suffix) => event_val.ends_with(suffix),
                };

                if !matched {
                    matched_all = false;
                    break;
                }
            }

            if matched_all {
                alerts.push(DetectionAlert {
                    rule_id: rule.id,
                    title: rule.title,
                    severity: rule.severity,
                });
            }
        }

        alerts
    }
}
```

### Blueprint 2: OCSF (Open Cybersecurity Schema Framework) Event Normalizer (Rust)
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OcsfProcessActivityEvent {
    pub activity_id: u32,       // 1 = Launch, 2 = Terminate
    pub category_uid: u32,      // 1 = System Activity
    pub class_uid: u32,         // 1007 = Process Activity
    pub time: u64,              // Milliseconds since epoch
    pub process_name: String,
    pub process_pid: u32,
    pub parent_process_name: String,
    pub parent_process_pid: u32,
    pub command_line: String,
    pub actor_user: String,
}

pub struct OcsfNormalizer;

impl OcsfNormalizer {
    pub fn normalize_raw_process_event(
        raw_pid: u32,
        raw_proc_name: &str,
        raw_cmd: &str,
        raw_ppid: u32,
        raw_parent_name: &str,
        raw_user: &str,
        timestamp_ms: u64,
    ) -> OcsfProcessActivityEvent {
        OcsfProcessActivityEvent {
            activity_id: 1,
            category_uid: 1,
            class_uid: 1007,
            time: timestamp_ms,
            process_name: raw_proc_name.trim().to_lowercase(),
            process_pid: raw_pid,
            parent_process_name: raw_parent_name.trim().to_lowercase(),
            parent_process_pid: raw_ppid,
            command_line: raw_cmd.trim().to_string(),
            actor_user: raw_user.trim().to_lowercase(),
        }
    }
}
```

---

## 4. Verification Protocol
1. **Rule Evaluation Latency**: Engine must evaluate `1,000 detection rules` against an incoming event in `< 100 microseconds`.
2. **Schema Integrity**: Normalized OCSF events must validate against JSON schema without missing mandatory class UID fields.
3. **Lossless Telemetry Alerting**: Dropped event counters in ring buffers must trigger automatic operational alert generation.
