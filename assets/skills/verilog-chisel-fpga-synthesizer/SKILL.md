---
name: verilog-chisel-fpga-synthesizer
description: "Hardware Description Languages (HDL), Verilog/SystemVerilog, and Chisel hardware accelerator synthesis, FPGA timing closure analysis, clock-domain crossing (CDC) verification, and resource budgeting."
version: 0.2.0
tags:
  - hardware
  - fpga
  - verilog
  - systemverilog
  - chisel
  - asic
  - timing-closure
  - cdc
compatibility: ">=0.2.0"
triggers:
  - "verilog"
  - "systemverilog"
  - "chisel"
  - "fpga"
  - "rtl"
  - "timing closure"
  - "cdc"
  - "axi-stream"
  - "synthesis"
  - "lut"
  - "dsp48"
  - "bram"
---

# Verilog & Chisel FPGA Synthesizer Skill

The `verilog-chisel-fpga-synthesizer` skill orchestrates hardware design, register-transfer level (RTL) synthesis, static timing analysis (STA), clock-domain crossing (CDC) proofs, and target FPGA resource budgeting (LUTs, Flip-Flops, DSP slices, UltraRAM, Block RAMs) across Xilinx UltraScale+, Intel Agilex, and Lattice iCE40 platforms.

## Core Capabilities
1. **Synthesizable RTL Synthesis**: Generates clean, synthesizable Verilog-2005, SystemVerilog-2017, and Chisel3/Scala RTL modules with strict single-edge synchronous resets and standardized AXI4-Stream handshakes.
2. **Static Timing Closure Analysis**: Calculates target clock period $T_{clk}$, logic depth levels, wire routing delays, and checks setup/hold slack ($T_{slack} = T_{clk} - (T_{logic} + T_{route}) \ge 0$).
3. **Clock-Domain Crossing (CDC) Verification**: Mandates dual-flip-flop synchronizers, Gray-coded pointers for asynchronous FIFOs, and handshaking pulses for crossing independent clock domains.
4. **FPGA Resource Budgeting**: Accurately estimates logic fabric utilization, mapping multiplier/accumulator structures into dedicated DSP blocks (DSP48E2/DSP-X) to avoid slice exhaustion.

## Strict Operational Invariants

- **ALWAYS**:
  - Enforce synchronous, active-high or active-low reset logic per platform standard (e.g. active-low `rst_n` for AXI bus compliance).
  - Register all module output ports in pipelined datapath stages to isolate combinatorial delays from inter-module routing networks.
  - Implement full backpressure support on ready/valid streaming interfaces: `tvalid` must not depend combinatorially on `tready`.
  - Maintain explicit bit widths on all arithmetic expressions and constants (`8'd0`, `32'hFFFF_FFFF`) to prevent synthesis bit-truncation warnings.

- **NEVER**:
  - NEVER permit inferred latches caused by incomplete `case` or `if/else` assignments in combinatorial `always @(*)` / `always_comb` blocks.
  - NEVER route clocks or asynchronous resets through general-purpose combinatorial logic gates (no glitch-prone gated clocks).
  - NEVER cross asynchronous clock domains without verified synchronizers (dual-FF, async FIFO with Gray-code pointers, or DMUX handshakes).
  - NEVER use non-synthesizable simulation constructs (`#delay`, `initial` blocks without RAM initialization, `$display`) in production RTL modules.

- **MANDATORY**:
  - Pipeline deep arithmetic logic paths (DSP multiply-accumulate chains, wide multiplexers, priority encoders) across multiple balanced clock stages when clock frequency exceeds 250 MHz.
  - Assert both forward progress (`tready && tvalid`) and queue boundary conditions (FIFO empty/full flags) using SystemVerilog Concurrent Assertions (SVA) or formal verification properties.
  - Separate datapath logic from state machine control registers (`fsm_state`, `next_state`).

- **STRICT_REJECT**:
  - Reject any hardware module design that exhibits negative setup slack (timing violations) at the target synthesis frequency.
  - Reject designs with missing default branches in state machine decoders that can lead to stuck-at unrecoverable hardware states.
  - Reject unsynchronized multi-bit signals crossing asynchronous clock domains.
