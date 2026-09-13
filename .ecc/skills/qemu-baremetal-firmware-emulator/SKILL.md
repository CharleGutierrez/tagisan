---
name: qemu-baremetal-firmware-emulator
description: "QEMU & Renode bare-metal firmware emulation (ARM Cortex-M, RISC-V, UEFI), MMIO verification, linker script memory map validation, and automated UART test harnesses."
version: 0.2.0
tags:
  - qemu
  - renode
  - baremetal
  - firmware
  - riscv
  - arm-cortex-m
  - uefi
  - mmio
  - linker-script
  - emulation
compatibility: ">=0.2.0"
triggers:
  - "qemu"
  - "renode"
  - "baremetal"
  - "firmware emulation"
  - "arm cortex-m"
  - "riscv"
  - "riscv64"
  - "riscv32"
  - "uefi"
  - "mmio"
  - "linker script"
  - "memory map"
  - "uart test harness"
---

# QEMU Baremetal Firmware Emulator Skill

The `qemu-baremetal-firmware-emulator` skill delivers automated bare-metal firmware execution, SoC peripheral emulation, linker script memory collision audits, and headless automated UART test harnesses across RISC-V (RV32/RV64), ARM Cortex-M, and x86_64 UEFI platforms.

## Core Capabilities

1. **Target Architecture Machine Synthesis**: Generates optimized QEMU invocation command lines and machine configurations (`virt` for RISC-V, `lm3s6965evb`/`stm32f4-discovery` for Cortex-M, `q35` + OVMF for UEFI).
2. **Linker Script & Physical SoC Memory Map Auditing**: Validates linker memory definitions (`FLASH`, `RAM`) against SoC hardware limits, detecting memory collisions, bank overflow, and alignment violations.
3. **Automated Headless UART Test Harness**: Synthesizes non-interactive test harnesses capturing serial console output, asserting boot messages, and catching hard faults or kernel panics.
4. **Deterministic Exit & Semihosting Interfacing**: Wires hardware test devices (SiFive test device `0x100000`, `isa-debug-exit`, or semihosting) to propagate pass/fail exit codes directly to the host CI runner.

## Strict Operational Invariants

- **ALWAYS**:
  - Validate linker script memory origins and lengths against physical SoC hardware specifications before executing firmware under emulation.
  - Run QEMU bare-metal test harnesses with `-nographic` and redirect serial output to stdio or socket for automated harness parsing.
  - Equip bare-metal firmware with a deterministic exit mechanism (e.g. SiFive test device `0x100000`, `isa-debug-exit`, or ARM semihosting `bkpt 0xAB`).
  - Enforce explicit execution timeouts on all emulated test runs to detect infinite spinlocks and unhandled exception loops.

- **NEVER**:
  - NEVER allow linker script sections (`.text`, `.rodata`, `.data`, `.bss`) to exceed the physical capacity of designated Flash or SRAM memory banks.
  - NEVER execute bare-metal emulation with graphical displays or sound devices enabled unless testing video buffer drivers.
  - NEVER ignore memory overlap collisions between distinct memory regions or reserved MMIO address spaces.
  - NEVER rely on uninitialized stack or heap memory; ensure BSS zeroing and DATA copying routines execute prior to main entry.

- **MANDATORY**:
  - MANDATORY verify 4-byte or 8-byte word alignment for interrupt vector tables and linker section start addresses.
  - MANDATORY configure GDB remote server stubs (`-s -S`) on port 1234 when running diagnostic stepping for hard-fault traps.
  - MANDATORY monitor UART streams for hardware exception signatures ("HardFault", "Panic", "Illegal Instruction", "Store Access Fault").

- **STRICT_REJECT**:
  - STRICT_REJECT firmware binaries whose memory layout causes section collision or spills outside target hardware address boundaries.
  - STRICT_REJECT emulation configurations lacking deterministic automated shutdown or timeout mechanisms.
  - STRICT_REJECT unverified MMIO peripheral access beyond mapped physical controller boundaries.
