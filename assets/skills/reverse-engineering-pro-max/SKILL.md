---
name: reverse-engineering-pro-max
description: Autonomous Master Engine for the Top 5,500 Reverse Engineering Skills found across GitHub. Covers binary analysis, decompilation theory, dynamic instrumentation (Frida, DynamoRIO), symbolic execution (angr, Z3), firmware extraction (Binwalk), defensive triage (Volatility, YARA, capa), protocol reverse engineering, mobile app analysis (JADX), VM/bytecode reversing, deobfuscation, kernel rootkit analysis, cryptographic primitive identification, file format templates, SCADA/automotive reversing, and game asset analysis. Triggers: reverse, reversing, ghidra, radare2, frida, ida, decompilation, binary analysis, disassembly, angr, z3, binwalk, yara, volatility, jadx, smali, capa, protocol reversing, deobfuscation.
version: 1.0.0
tags:
  - reverse-engineering
  - binary-analysis
  - decompilation
  - dynamic-instrumentation
  - symbolic-execution
  - firmware-reversing
  - defensive-triage
  - protocol-reversing
  - mobile-reversing
  - deobfuscation
compatibility: ">=0.2.0"
---

# Reverse Engineering Pro Max: The Sovereign 5,500-Skill Engine

## Purpose & Scope
Reverse engineering is the scientific discipline of deconstructing compiled software, firmware, proprietary protocols, and binary architectures to extract their design specifications, reconstruct algorithms, audit security properties, and ensure interoperability.

The `reverse-engineering-pro-max` skill codifies the **Top 5,500 Reverse Engineering Skills** discovered across the global open-source software ecosystem on GitHub (`NationalSecurityAgency/ghidra`, `radareorg/radare2`, `frida/frida`, `angr/angr`, `Z3Prover/z3`, `volatilityfoundation/volatility3`, `mandiant/capa`, `wireshark/wireshark`, `skylot/jadx`, `ReFirmLabs/binwalk`, `kaitai-io/kaitai_struct`, etc.).

---

## 1. The 15 Macro-Domain Clusters (5,500 Skills)

| Cluster Code | Macro-Domain Cluster | Skills | Percent | Benchmark GitHub Repositories | Core Operational Invariant |
|---|---|---|---|---|---|
| `REV-01` | **Disassembly & Binary Analysis Foundations** | 500 | 9.1% | `radareorg/radare2`, `rizinorg/rizin`, `capstone-engine/capstone` | Deterministic instruction decoding, recursive CFG traversal, ELF/PE/Mach-O section parsing |
| `REV-02` | **Decompilation Theory, AST Reconstruction & SSA Form** | 450 | 8.2% | `NationalSecurityAgency/ghidra`, `lifting-bits/remill`, `angr/ail` | Dominator tree structuring, natural loop recovery, type propagation, P-Code/LLVM IR lifting |
| `REV-03` | **Dynamic Binary Instrumentation (DBI) & Hooking** | 450 | 8.2% | `frida/frida`, `DynamoRIO/dynamorio`, `microsoft/Detours` | Zero-crash inline trampolines, IAT/EAT/PLT/GOT hooks, Stalker trace containment |
| `REV-04` | **Symbolic Execution, SMT Solving & Program Synthesis** | 450 | 8.2% | `angr/angr`, `Z3Prover/z3`, `JonathanSalwan/Triton`, `trailofbits/manticore` | Sound path constraint formulation, SAT/SMT solving, state-explosion mitigation, concolic execution |
| `REV-05` | **Firmware Extraction, Embedded Systems & Hardware Reversing** | 400 | 7.3% | `ReFirmLabs/binwalk`, `firmadyne/firmadyne`, `openwrt/openwrt` | Non-destructive flash dumping, SquashFS/JFFS2 carving, UART/JTAG pinout extraction, UEFI DXE reversing |
| `REV-06` | **Defensive Malware Triage, Memory Forensics & Threat Hunting** | 400 | 7.3% | `volatilityfoundation/volatility3`, `VirusTotal/yara`, `mandiant/capa` | Strict sandbox containment, VAD tree process hollowing triage, YARA-X rule synthesis, capability mapping |
| `REV-07` | **Protocol Reverse Engineering & Network Dissection** | 400 | 7.3% | `wireshark/wireshark`, `kaitai-io/kaitai_struct`, `secdev/scapy` | Wireshark C/Lua dissector compilation, frame delimiter inference, state machine recovery, TLS interception |
| `REV-08` | **Android, Dalvik/ART & Mobile App Reversing** | 400 | 7.3% | `skylot/jadx`, `iBotPeaches/Apktool`, `JesusFreke/smali`, `objection/objection` | Dalvik bytecode disassembling, Smali editing, JADX AST deobfuscation, ART hook binding, SSL unpinning |
| `REV-09` | **WebAssembly (Wasm), Bytecode VMs & JIT Reversing** | 400 | 7.3% | `WebAssembly/wabt`, `icsharpcode/ILSpy`, `dotnet/ilspy`, `java-decompiler/jd-gui` | Wasm binary decoding, stack-machine state emulation, .NET CIL & JVM bytecode decompilation, pyc reconstruction |
| `REV-10` | **Anti-Analysis, Obfuscation & Deobfuscation Engineering** | 350 | 6.4% | `JonathanSalwan/Tigress_protection`, `mrphrazer/oblivion`, `OWASP/owasp-mastg` | Control-flow unflattening, opaque predicate elimination, anti-debug evasion bypassing, string decryptor lifting |
| `REV-11` | **Kernel Internals, Drivers & Rootkit Forensics** | 350 | 6.4% | `iovisor/bcc`, `libbpf/libbpf`, `therexg/kernel-reversing` | NTOSKRNL IRP dispatching, SSDT hook discovery, DKOM process unhiding, eBPF probe telemetry verification |
| `REV-12` | **Cryptographic Primitive Identification & Math Reversing** | 350 | 6.4% | `polymorf/findcrypt-yara`, `RobinDavid/python-idb`, `Legrandin/pycryptodome` | S-box & IV constant fingerprinting, modular exponentiation parameter recovery, PRNG/LFSR cycle tracking |
| `REV-13` | **File Format Reversing & Binary Template Engineering** | 250 | 4.5% | `SweetScape/010Editor-Templates`, `kaitai-io/kaitai_struct_formats`, `mateus/polyfile` | 010 Editor binary templates (.bt), Kaitai Struct (.ksy) declarative schemas, chunked container parsing |
| `REV-14` | **Industrial Systems, SCADA & Automotive Bus Reversing** | 200 | 3.6% | `FreeOpcUa/opcua-asyncio`, `craig/can-utils`, `wireshark/wireshark` | CAN bus frame bitmask decoding, DBC schema synthesis, UDS diagnostic services, Modbus/DNP3 mapping |
| `REV-15` | **Modern Game Engine, Asset & Shader Reversing** | 150 | 2.7% | `cheat-engine/cheat-engine`, `Perfare/AssetStudio`, `EpicGames/UnrealEngine` | Pointer multi-level offset resolution, Unreal GObjects/GNames reflection dumping, Unity IL2CPP metadata extraction |
| **TOTAL** | **15 Macro-Domain Clusters** | **5,500** | **100.0%** | **Global Open-Source GitHub Ecosystem** | **Clean-Room Specification, Deterministic Decompilation, Sound Program Analysis** |

---

## 2. Core Operational Invariants

### Invariant 1: Airgapped & Contained Analysis
Untrusted or unknown binaries must never be executed natively on host systems. Dynamic instrumentation, emulation, and triage must strictly occur within isolated QEMU hypervisors, hardware sandboxes, or containerized testbeds with egress disabled.

### Invariant 2: Mathematical Decompilation Equivalence
Control Flow Graph (CFG) structuring and SSA variable recovery must maintain functional equivalence to original machine code semantics. Phi nodes, pointer dereferences, and calling conventions must be formally verified against architectural ABIs (System V AMD64, Microsoft x64, ARM AAPCS).

### Invariant 3: Non-Destructive Hardware Interfacing
Embedded hardware debugging (JTAG, SWD, UART) and flash memory dumping (SPI, I2C, NAND/eMMC) must preserve target device state. Voltage levels (1.8V, 3.3V, 5.0V) must be electrically level-shifted before bus sniffing.

### Invariant 4: Clean-Room Engineering & Interoperability
Reverse-engineered protocols, file formats, and APIs must produce clean declarative specifications (Kaitai Struct schemas, OpenAPI/gRPC definitions, Rust type definitions) adhering to clean-room legal precedents for functional compatibility.
