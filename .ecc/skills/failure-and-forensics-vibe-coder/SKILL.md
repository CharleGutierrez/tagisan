---
name: failure-and-forensics-vibe-coder
description: "Master Failure & Forensic Analysis skill for vibe code developers: systematic triage, core dump forensics, eBPF telemetry, invariant testing, and root cause analysis across 50 canonical engineering texts."
---

# Failure & Forensic Analysis for the Vibe Code Developer

## 1. Mission & Operational Philosophy

Vibe coding enables rapid software construction by leveraging Large Language Models (LLMs) to synthesize business logic, API integrations, and system configuration. However, this velocity introduces high-severity failure modes:
- **Semantic Opacity**: The engineer runs code whose internal branch invariants, resource lifecycles, and edge conditions were synthesized by an LLM and never manually verified.
- **Hallucination Loops**: When an automated test or runtime error occurs, blindly feeding the error back to the LLM often results in superficial symptom masking, compounding errors, and architectural drift.
- **Unverified Dependencies & Supply-Chain Drift**: LLMs hallucinate non-existent package methods, pull in unvetted or abandoned libraries, or omit lockfile constraints.
- **Opaque Production Crashes**: Silent panics, unhandled promise rejections, connection pool exhaustion, memory leaks, and cascading timeouts that yield zero actionable diagnostics.

This skill synthesizes the core principles and forensic practices of **50 canonical engineering books** into immediate, deterministic workflows. When code breaks, the vibe coder stops guessing and executes the scientific method.

---

## 2. The Ten Pillars of Forensic Analysis

```
                               ┌────────────────────────────────────────────────────────┐
                               │   Vibe Code Forensic Investigation Framework          │
                               └──────────────────────────┬─────────────────────────────┘
                                                          │
         ┌───────────────────────────────┬────────────────┼──────────────────────────────┬──────────────────────────────┐
         ▼                               ▼                ▼                              ▼                              ▼
  [1. Scientific Debugging]    [2. Systems Telemetry]   [3. Resilient Architecture]    [4. Incident Ops & SRE]   [5. Memory & Binary Forensics]
  - Hypothesis Testing         - USE Method (Gregg)     - Stability Patterns (Nygard)  - Blameless RCA (Okes)    - GDB / Core Dumps
  - Delta Debugging (Zeller)   - eBPF / bpftrace        - Consensus & Wal (Petrov)     - Error Budgets (Beyer)   - Dynamic Binary Analysis
  - 9 Rules (Agans)            - High-Cardinality Logs  - Anti-Fragility (Taleb)       - Incident Command (ICS)  - Symbol & Linking Audits
         │                               │                │                              │                              │
         ▼                               ▼                ▼                              ▼                              ▼
  [6. Digital Network Forensics] [7. Exploit & Fuzzing] [8. Concurrency Verification]  [9. Systemic Accidents]   [10. Legacy Code Seams]
  - PCAP Packet Traces         - Input Fuzzing (Miller) - TLA+ Invariants (Lamport)    - Normal Accidents (Perrow)- Characterization Tests
  - TLS & DNS Resolution       - STRIDE Threat Modeling - Spin Model Checking          - Safety STAMP (Leveson)   - Seam Extraction (Feathers)
  - Egress Security Monitoring - Memory Exploits (Dowd) - Protocol Validation (Holzmann)- Human Error (Dekker)    - Pinning Down Dependencies
```

---

## 3. Immediate Action Playbook for Vibe Coding Failures

### Playbook A: Breaking the LLM Hallucination Loop
**Trigger**: An LLM-generated patch fails, and a subsequent prompt generates code that causes a different failure or reintroduces an earlier defect.
**Protocol**:
1. **Cease Prompting**: Do not submit another prompt describing the secondary error.
2. **Revert to Baseline**: Execute `git reset --hard` or `git checkout` to the clean commit before the hallucination cascade started.
3. **Isolate Minimal Reproduction (Delta Debugging)**: Using Andreas Zeller's `ddmin` algorithm, eliminate all superfluous parameters, endpoints, and data payloads until a minimal test case reproducing the failure remains (<= 15 lines of code).
4. **Inspect State Transitions (David Agans: Rule 3 - "Quit Thinking and Look")**: Add explicit assertions or trace execution using `strace -f` or language-level debug flags. Never guess the variable state.
5. **Formulate Explicit Invariant**: Define the failed invariant: "Function $F$ expects input $x \in [0, 100]$, but receives $null$ during token refresh."
6. **Constrain LLM Generation**: Prompt the LLM strictly with:
   - The minimal failing test case.
   - The exact observed invariant violation.
   - A negative constraint: "Do not modify the public API signature or change any files outside `src/auth/token.rs`."

### Playbook B: Triage for Opaque Production Crashes & Silent Hangs
**Trigger**: A microservice crashes without a traceback, dies with exit code 137 (OOM), or stops processing requests (hung event loop).
**Protocol**:
1. **Apply the USE Method (Brendan Gregg)**:
   - **Utilization**: Measure CPU percentage, resident memory set (RSS), and disk I/O (`top -b -n 1`, `vmstat 1 5`).
   - **Saturation**: Measure OS run queue length, context switches, swap in/out rates, and TCP listen backlogs (`ss -lnt`).
   - **Errors**: Query the Linux kernel ring buffer for OOM-killer invocations or segmentation faults (`dmesg -T | grep -iE 'oom|segfault|kill'`).
2. **Dynamic Syscall Inspection**:
   - For a hung process: Attach `strace -p <PID> -f -T -tt -e trace=all` to identify whether the thread is blocked on a mutex, epoll wait, or an unclosed socket read.
   - In eBPF environments: Execute `bpftrace -e 'tracepoint:syscalls:sys_enter_read { @[comm] = count(); }'` to monitor blocking I/O calls.
3. **Core Dump Autopsy**:
   - Inspect the generated core dump via GDB: `gdb <binary> <core> -ex "thread apply all bt" -ex "quit"`.
   - Verify whether stack overflow, corrupt heap metadata, or null pointer dereference caused the fatal signal.

### Playbook C: Auditing Unverified Dependencies & Supply-Chain Drift
**Trigger**: LLM suggests installing a third-party package or modifies `package.json`, `Cargo.toml`, or `requirements.txt`.
**Protocol**:
1. **Lockfile Enforcement**: Always commit and check lockfiles (`Cargo.lock`, `package-lock.json`, `poetry.lock`). Reject any build that updates lockfiles without human review.
2. **Cryptographic Integrity & Known Vulnerabilities**: Run `cargo deny check`, `npm audit --audit-level=high`, or `pip-audit`.
3. **Egress Containment (Richard Bejtlich)**: Inspect network activity during tests: `lsof -i -P -n | grep <process>`. Ensure test suites do not make unauthorized outbound requests to external endpoints.
4. **Wrap with Anti-Corruption Seams (Michael Feathers & Eric Evans)**: Never allow third-party library types to penetrate domain logic. Wrap all external dependencies in adapter classes/traits.

### Playbook D: Concurrency Deadlocks & Race Condition Elimination
**Trigger**: Flaky tests that pass in isolation but fail intermittently during parallel CI runs.
**Protocol**:
1. **Sanitizer Compilation**: Recompile with thread sanitizer enabled (`cargo test --target ... -Zsanitizer=thread` or `gcc -fsanitize=thread`).
2. **Lock Order Hierarchies**: Audit all acquisition points of mutexes/read-write locks. Invariant: If Lock $A$ is acquired before Lock $B$ anywhere in the codebase, Lock $B$ must *never* be acquired before Lock $A$.
3. **Bounded Channels (Michael T. Nygard)**: Replace unbounded memory queues with bounded channels (`tokio::sync::mpsc::channel(capacity)`) and explicit backpressure / rejection metrics.

---

## 4. The 50 Canonical Forensic Literature References

| # | Exact Title | Author(s) | Publisher | Year / Edition | ISBN-10 | ISBN-13 | Core Thesis & Forensic Technique | Vibe Coder Actionable Translation |
|---|---|---|---|---|---|---|---|---|
| 1 | Why Programs Fail: A Guide to Systematic Debugging | Andreas Zeller | Morgan Kaufmann | 2009 (2nd Ed.) | 0123745152 | 978-0123745156 | Cause-effect chains, automated delta debugging (`ddmin`), state infection tracking | Replaces random prompting with automated minimization of failing inputs. |
| 2 | Debugging: The 9 Indispensable Rules for Finding Even the Most Elusive Software and Hardware Problems | David J. Agans | AMACOM | 2002 | 0814474578 | 978-0814474570 | 9 heuristics: Understand the system, make it fail, quit thinking and look | Disciplined rules to stop guessing what AI code does and observe actual state. |
| 3 | Effective Debugging: 66 Specific Ways to Debug Software and Systems | Diomidis Spinellis | Addison-Wesley | 2016 | 0134394798 | 978-0134394794 | 66 concrete recipes: debuggers, assertions, profilers, dynamic analysis | Provides pragmatic CLI and tool-based verification recipes over manual review. |
| 4 | Root Cause Analysis: The Core of Problem Solving and Corrective Action | Duke Okes | ASQ Quality Press | 2019 (2nd Ed.) | 0873899822 | 978-0873899826 | Structured RCA, 5-Whys, Ishikawa diagrams, error recurrence prevention | Prevents accepting superficial LLM symptom-masking patches. |
| 5 | Root Cause Analysis: Improving Performance for Bottom-Line Results | Robert J. Latino, Kenneth C. Latino, Mark A. Latino | CRC Press | 2019 (5th Ed.) | 1138332453 | 978-1138332454 | FMEA, Logic Trees, physical vs. latent vs. human root causes | Differentiates code crash from lack of CI guards and unpinned dependencies. |
| 6 | Systems Performance: Enterprise and the Cloud | Brendan Gregg | Addison-Wesley | 2020 (2nd Ed.) | 0136820158 | 978-0136820154 | USE method (Utilization, Saturation, Errors), flame graphs, Linux internals | Systematically diagnoses why AI-generated code runs slow or saturates hardware. |
| 7 | BPF Performance Tools: Deep Analysis and Early Detection for Linux Systems | Brendan Gregg | Addison-Wesley | 2019 | 0136554822 | 978-0136554820 | eBPF telemetry, `bpftrace`, zero-overhead kernel/user tracing | Provides visibility into compiled AI binaries without modifying source code. |
| 8 | Observability Engineering: Achieving Production Excellence | Charity Majors, Liz Fong-Jones, George Miranda | O'Reilly Media | 2022 | 1492076864 | 978-1492076865 | High-cardinality wide structured events, exploratory query triage | Mandates structured JSON event logs with context rather than naive `print` logs. |
| 9 | Distributed Tracing in Practice: Instrumenting, Analyzing, and Debugging Systems | Austin Parker, Daniel Spoonhower, Jonathan Mace, Ben Sigelman, Rebecca Isaacs | O'Reilly Media | 2020 | 1492056634 | 978-1492056638 | OpenTelemetry, context propagation, latency path decomposition | Traces requests across multi-service architectures generated by LLM prompts. |
| 10 | The Linux Programming Interface: A Linux and UNIX System Programming Handbook | Michael Kerrisk | No Starch Press | 2010 | 1593272200 | 978-1593272203 | Linux kernel syscalls: epoll, signals, memory layout, file descriptors | Bridges high-level code assumptions with low-level Linux OS behavior. |
| 11 | Release It!: Design and Deploy Production-Ready Software | Michael T. Nygard | Pragmatic Bookshelf | 2018 (2nd Ed.) | 1680502395 | 978-1680502398 | Circuit Breakers, Bulkheads, Timeouts, shedding load, antipattern elimination | Wraps optimistic AI "happy-path" code with defensive resilience patterns. |
| 12 | Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems | Martin Kleppmann | O'Reilly Media | 2017 | 1449373321 | 978-1449373320 | Consistency, replication lag, transactions, consensus, event streams | Enforces formal data invariants against silent data corruption and split-brain. |
| 13 | Chaos Engineering: System Resiliency in Practice | Casey Rosenthal, Nora Jones | O'Reilly Media | 2020 | 1492043869 | 978-1492043867 | Hypothesis-driven fault injection, blast radius containment | Validates AI resilience by intentionally cutting downstream services under load. |
| 14 | Database Internals: A Deep Dive into How Distributed Data Systems Work | Alex Petrov | O'Reilly Media | 2019 | 1492040347 | 978-1492040347 | B-trees, LSM-trees, WAL logging, distributed consensus and recovery | Diagnoses storage corruption and locking deadlocks caused by AI ORM queries. |
| 15 | Fault-Tolerant Systems | Israel Koren, C. Mani Krishna | Morgan Kaufmann | 2020 (2nd Ed.) | 0128180498 | 978-0128180495 | Hardware/software redundancy, Byzantine faults, checkpoint recovery | Provides voting and rollback architectures for multi-agent LLM systems. |
| 16 | Production-Ready Microservices: Building Standardized Systems Across an Engineering Organization | Susan J. Fowler | O'Reilly Media | 2016 | 1491965975 | 978-1491965979 | 8 production readiness pillars: stability, monitoring, scalability, security | Comprehensive quality gate checklist before deploying AI-generated services. |
| 17 | Building Microservices: Designing Fine-Grained Systems | Sam Newman | O'Reilly Media | 2021 (2nd Ed.) | 1492034029 | 978-1492034025 | Bounded contexts, async messaging, Saga pattern, evolutionary schemas | Prevents vibe coders from creating brittle, tightly-coupled distributed monoliths. |
| 18 | Patterns of Enterprise Application Architecture | Martin Fowler | Addison-Wesley | 2002 | 0321127420 | 978-0321127426 | Enterprise patterns: Unit of Work, Repository, Data Mapper, Concurrency | Imposes disciplined design patterns onto raw AI code generation. |
| 19 | Enterprise Integration Patterns: Designing, Building, and Deploying Messaging Solutions | Gregor Hohpe, Bobby Woolf | Addison-Wesley | 2003 | 0321200683 | 978-0321200686 | 65 messaging patterns: Dead Letter Channel, Idempotent Receiver | Ensures asynchronous AI message processing never silently drops data. |
| 20 | Antifragile: Things That Gain from Disorder | Nassim Nicholas Taleb | Random House | 2012 | 1400067820 | 978-1400067824 | Convex payoffs, eliminating fragility, learning from stressors | Reframes errors as feedback that strengthens automated regression guards. |
| 21 | Site Reliability Engineering: How Google Runs Production Systems | Betsy Beyer, Chris Jones, Jennifer Petoff, Niall Richard Murphy | O'Reilly Media | 2016 | 149192912X | 978-1491929124 | SLOs, SLIs, Error Budgets, toil reduction, blameless postmortems | Provides data-driven guardrails to throttle AI shipping velocity when budgets burn. |
| 22 | The Site Reliability Workbook: Practical Ways to Implement SRE | Betsy Beyer, Niall Richard Murphy, David K. Rensin, Kent Kawahara, Stephen Thorne | O'Reilly Media | 2018 | 1492029505 | 978-1492029502 | Multi-window multi-burn-rate alerting, non-abstract design, postmortems | Practical recipes for setting up automated reliability monitoring on AI services. |
| 23 | Building Secure and Reliable Systems: Best Practices for Designing, Implementing, and Maintaining Systems | Heather Adkins, Betsy Beyer, Paul Blankinship, Piotr Lewandowski, Ana Oprea, Adam Stubblefield | O'Reilly Media | 2020 | 1492083127 | 978-1492083122 | Convergence of security and reliability: defense in depth, least privilege | Catches privilege escalation and insecure defaults generated by LLMs. |
| 24 | Seeking SRE: Conversations About Running Production Systems at Scale | David N. Blank-Edelman | O'Reilly Media | 2018 | 1491978864 | 978-1491978863 | SRE organizational adoption, psychological safety, on-call health | Helps lean vibe coding teams implement sustainable reliability engineering. |
| 25 | Incident Management for Operations: A Guide to Developing and Managing an Incident Response Plan | Rob Schnepp, Ron Vidal, Chris Hawley | O'Reilly Media | 2017 | 1491954310 | 978-1491954317 | Incident Command System (ICS) for technology operations | Enforces calm, structured triage during production outages without panic prompting. |
| 26 | The Practice of Cloud System Administration: Designing and Operating Large Distributed Systems, Volume 2 | Thomas A. Limoncelli, Nicole Forsgren, Strata R. Chalup | Addison-Wesley | 2014 | 032194318X | 978-0321943187 | Immutable infrastructure, automated deployments, disaster recovery | Eliminates configuration drift caused by ad-hoc hotfixes on production boxes. |
| 27 | Accelerate: The Science of Lean Software and DevOps: Building and Scaling High Performing Technology Organizations | Nicole Forsgren, Jez Humble, Gene Kim | IT Revolution Press | 2018 | 1942788339 | 978-1942788331 | DORA metrics: Deployment Frequency, Lead Time, MTTR, Change Failure Rate | Benchmarks whether AI velocity is creating unsustainable operational debt. |
| 28 | The Art of Memory Forensics: Detecting Malware and Threats in Windows, Linux, and Mac Memory | Michael Hale Ligh, Andrew Case, Jamie Levy, AAron Walters | Wiley | 2014 | 1118825993 | 978-1118825990 | Volatile RAM analysis, pool scanning, address space extraction | Inspects process memory dumps to diagnose hard crashes and secret leaks. |
| 29 | Practical Malware Analysis: The Hands-On Guide to Dissecting Malicious Software | Michael Sikorski, Andrew Honig | No Starch Press | 2012 | 1593272901 | 978-1593272906 | Static/dynamic binary analysis, x86 disassembly, behavior sandboxing | Audits suspicious dependencies and hallucinated external packages. |
| 30 | Practical Binary Analysis: Build Your Own Linux Tools for Binary Instrumentation, Analysis, and Disassembly | Dennis Andriesse | No Starch Press | 2018 | 1593279124 | 978-1593279127 | ELF layout, symbol tables, relocation entries, dynamic binary instrumentation | Diagnoses dynamic linker errors and segfaults in native compiled modules. |
| 31 | The IDA Pro Book: The Unofficial Guide to the World's Most Popular Disassembler | Chris Eagle | No Starch Press | 2011 (2nd Ed.) | 1593272898 | 978-1593272890 | Control-flow graph recovery, type reconstruction, binary auditing | Allows decompilation and audit of closed-source third-party dependencies. |
| 32 | Rootkits and Bootkits: Reversing Modern Malware and Next-Gen Threats | Alex Matrosov, Eugene Rodionov, Sergey Bratus | No Starch Press | 2019 | 1593277164 | 978-1593277161 | Kernel rootkits, bootloader manipulation, hypervisor bypasses | Secures container environments against kernel escapes from untrusted code. |
| 33 | Windows Internals, Part 1: System architecture, processes, threads, memory management, and more | Pavel Yosifovich, David A. Solomon, Alex Ionescu, Mark E. Russinovich | Microsoft Press | 2017 (7th Ed.) | 0735684189 | 978-0735684188 | Windows NT architecture, thread scheduling, virtual memory, WinDbg | Diagnoses Windows-specific crashes, kernel bug checks, and handle leaks. |
| 34 | File System Forensic Analysis | Brian Carrier | Addison-Wesley | 2005 | 0321268172 | 978-0321268174 | Volume structures, NTFS/Ext metadata, inodes, journaling forensics | Recovers unlinked files and investigates data corruption from buggy AI I/O. |
| 35 | Network Forensics: Tracking Hackers through Cyberspace | Sherri Davidoff, Jonathan Ham | Prentice Hall | 2012 | 0132564718 | 978-0132564717 | Packet capture (PCAP) analysis, protocol reconstruction, flow traces | Inspects wire-level HTTP/TLS failures between distributed microservices. |
| 36 | The Practice of Network Security Monitoring: Understanding Incident Detection and Response | Richard Bejtlich | No Starch Press | 2013 | 1593275099 | 978-1593275099 | Full-content, session, and statistical network analysis with Zeek/Suricata | Detects rogue outbound data exfiltration or unverified telemetry calls. |
| 37 | Practical Forensic Imaging: Securing Digital Evidence with Linux Tools | Bruce Nikkel | No Starch Press | 2016 | 1593277938 | 978-1593277932 | Bit-stream disk imaging, raw block storage, write-blocking verification | Preserves bit-for-bit volume images before attempting destructive data recovery. |
| 38 | Software Forensics: Collecting Evidence from the Scene of a Digital Crime | Robert M. Slade | McGraw-Hill | 2004 | 0072228296 | 978-0072228298 | Code stylometry, authorship attribution, compiler identification | Identifies code origin and detects stylistic drift across multi-agent generations. |
| 39 | The Art of Software Security Assessment: Identifying and Preventing Software Vulnerabilities | Mark Dowd, John McDonald, Justin Schuh | Addison-Wesley | 2006 | 0321444426 | 978-0321444424 | Memory corruption, integer overflows, format strings, type safety | Provides security audit rubrics to catch unsafe AI-generated arithmetic and pointer math. |
| 40 | Fuzzing for Software Security Testing and Quality Assurance | Ari Takanen, Jared D. DeMott, Charlie Miller | Artech House | 2018 (2nd Ed.) | 1608078507 | 978-1608078509 | Coverage-guided mutation fuzzing, crash triage, exploitability analysis | Automatically tortures AI-generated input parsers to expose unhandled crashes. |
| 41 | Security Engineering: A Guide to Building Dependable Distributed Systems | Ross J. Anderson | Wiley | 2020 (3rd Ed.) | 1119642787 | 978-1119642787 | Distributed security architecture, protocol failures, side-channel attacks | Ensures systemic cryptographic correctness beyond simple syntax checks. |
| 42 | Threat Modeling: Designing for Security | Adam Shostack | Wiley | 2014 | 1118809998 | 978-1118809990 | STRIDE framework, Data Flow Diagrams (DFDs), attack trees | Systematically threat-models vibe architectures before synthesizing code. |
| 43 | Design and Validation of Computer Protocols | Gerard J. Holzmann | Prentice Hall | 1991 | 0135302544 | 978-0135302545 | Protocol reachability analysis, deadlocks, state space validation | Validates asynchronous state machines to prevent protocol hang states. |
| 44 | The Spin Model Checker: Primer and Reference Manual | Gerard J. Holzmann | Addison-Wesley | 2003 | 0321228626 | 978-0321228628 | PROMELA modeling, LTL verification, automated counterexample traces | Discovers subtle interleaving bugs in concurrent AI algorithms via model checking. |
| 45 | Specifying Systems: The TLA+ Language and Tools for Hardware and Software Engineers | Leslie Lamport | Addison-Wesley | 2002 | 032114306X | 978-0321143068 | Temporal Logic of Actions (TLA+), safety and liveness invariant checks | Formally defines invariants before prompting code, guaranteeing mathematical validity. |
| 46 | Normal Accidents: Living with High-Risk Technologies | Charles Perrow | Princeton Univ. Press | 1999 | 0691004129 | 978-0691004129 | Normal Accident Theory: interactive complexity and tight coupling | Warns against chaining multiple unverified AI agents without decoupling. |
| 47 | The Field Guide to Understanding 'Human Error' | Sidney Dekker | CRC Press | 2014 (3rd Ed.) | 1472439058 | 978-1472439055 | Systemic safety, local rationality, blameless engineering culture | Directs postmortems to systemic design flaws rather than developer error. |
| 48 | Engineering a Safer World: Systems Thinking Applied to Safety | Nancy G. Leveson | MIT Press | 2012 | 0262016621 | 978-0262016629 | STAMP and CAST models: safety as a dynamic control problem | Designs safety constraint feedback loops continuously enforced by automation. |
| 49 | To Engineer Is Human: The Role of Failure in Successful Design | Henry Petroski | Vintage | 1992 | 0679734163 | 978-0679734161 | Structural case studies; failure as the engine of engineering advancement | Treats AI hallucinations and defects as essential insights for building guards. |
| 50 | Working Effectively with Legacy Code | Michael C. Feathers | Prentice Hall | 2004 | 0131177052 | 978-0131177055 | Characterization tests, seams, breaking dependencies, safe refactoring | Wraps opaque AI code in characterization tests to refactor without regressions. |

---

## 5. Verification Checklist for the Vibe Code Developer

Before committing or releasing any code synthesized by an LLM:
1. **[ ] Deterministic Test Harness**: Every bug report must be accompanied by an automated regression test that fails before the fix and passes after.
2. **[ ] Lockfiles Pinned**: All package dependencies must have cryptographic checksums in lockfiles.
3. **[ ] Boundaries Bounded**: Ensure all network calls, queues, and thread pools have explicit timeouts and bounded buffers (no infinite timeouts).
4. **[ ] High-Cardinality Telemetry**: Ensure critical state transitions and error handlers emit structured contextual events (user, tenant, duration, error code).
5. **[ ] Seam Isolation**: Keep AI-generated code behind explicit interfaces so that defective implementations can be swapped or rolled back with zero domain disruption.
